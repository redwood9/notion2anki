use dotenvy::dotenv;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;

#[derive(Debug)]
struct Flashcard {
    question: String,
    answer: String,
}

#[derive(Deserialize, Debug)]
struct NotionPage {
    id: String,
    properties: serde_json::Value,
}

#[derive(Deserialize, Debug)]
struct NotionQueryResponse {
    results: Vec<NotionPage>,
}

#[derive(Deserialize, Debug)]
struct NotionBlock {
    id: String,
    paragraph: Option<ParagraphContent>,
    heading_1: Option<HeadingContent>,
    heading_2: Option<HeadingContent>,
    heading_3: Option<HeadingContent>,
}

#[derive(Deserialize, Debug)]
struct ParagraphContent {
    rich_text: Vec<RichText>,
}

#[derive(Deserialize, Debug)]
struct HeadingContent {
    rich_text: Vec<RichText>,
}

#[derive(Deserialize, Debug)]
struct RichText {
    plain_text: String,
}

async fn fetch_notion_database() -> Result<Vec<NotionPage>, reqwest::Error> {
    let notion_api_key = env::var("NOTION_API_KEY").expect("NOTION_API_KEY must be set");
    let database_id = env::var("NOTION_DATABASE_ID").expect("NOTION_DATABASE_ID must be set");
    let url = format!("https://api.notion.com/v1/databases/{}/query", database_id);

    let client = Client::new();
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", notion_api_key))
        .header("Notion-Version", "2022-06-28")
        .header("Content-Type", "application/json")
        .json(&json!({
            "filter": {
                "property": "Status",
                "select": {
                    "equals": "Ready to Import"
                }
            }
        }))
        .send()
        .await?;

    let query_response: NotionQueryResponse = response.json().await?;
    Ok(query_response.results)
}

async fn fetch_page_content(page_id: &str) -> Result<String, reqwest::Error> {
    let notion_api_key = env::var("NOTION_API_KEY").expect("NOTION_API_KEY must be set");
    let url = format!("https://api.notion.com/v1/blocks/{}/children?page_size=100", page_id);

    let client = Client::new();
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", notion_api_key))
        .header("Notion-Version", "2022-06-28")
        .send()
        .await?;

    let body = response.text().await?;
    Ok(body)
}

fn parse_flashcards(content: &str) -> Vec<Flashcard> {
    let mut flashcards = Vec::new();
    // Improved regex to handle multi-line answers
    let re = Regex::new(r"(?ms)(?:^|\n)问题[:：]\s*(.*?)\s*\n答案[:：]\s*([\s\S]*?)(?=\n问题[:：]|\n*$)").unwrap();
    
    for capture in re.captures_iter(content) {
        if let (Some(question), Some(answer)) = (capture.get(1), capture.get(2)) {
            flashcards.push(Flashcard {
                question: question.as_str().trim().to_string(),
                answer: answer.as_str().trim().to_string(),
            });
        }
    }
    
    flashcards
}

async fn add_note_to_anki(flashcard: &Flashcard) -> Result<(), reqwest::Error> {
    let anki_connect_url = env::var("ANKI_CONNECT_URL")
        .unwrap_or_else(|_| "http://localhost:8765".to_string());
    
    let client = Client::new();
    let response = client
        .post(&anki_connect_url)
        .json(&json!({
            "action": "addNote",
            "version": 6,
            "params": {
                "note": {
                    "deckName": "Notion Import",
                    "modelName": "Basic",
                    "fields": {
                        "Front": flashcard.question,
                        "Back": flashcard.answer
                    }
                }
            }
        }))
        .send()
        .await?;
    
    if response.status().is_success() {
        println!("Added card: {}", flashcard.question);
    } else {
        eprintln!("Failed to add card: {}", response.text().await?);
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    let pages = fetch_notion_database().await?;
    println!("Found {} pages to import", pages.len());
    
    let mut success_count = 0;
    for page in pages {
        let content = fetch_page_content(&page.id).await?;
        let flashcards = parse_flashcards(&content);
        
        for flashcard in flashcards {
            if add_note_to_anki(&flashcard).await.is_ok() {
                success_count += 1;
            }
        }
    }
    
    println!("Successfully imported {} flashcards to Anki", success_count);
    Ok(())
}

use dotenvy::dotenv;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use std::env;

#[derive(Debug)]
struct Flashcard {
    question: String,
    answer: String,
}

#[derive(Deserialize, Debug)]
struct NotionPage {
    id: String,
}

#[derive(Deserialize, Debug)]
struct NotionSearchResponse {
    results: Vec<NotionPage>,
}

async fn fetch_all_pages(debug_mode: bool) -> Result<Vec<NotionPage>, Box<dyn std::error::Error>> {
    let notion_api_key = env::var("NOTION_API_KEY").expect("NOTION_API_KEY must be set");
    let url = "https://api.notion.com/v1/search";

    let client = Client::new();
    let request_body = json!({
        "filter": {
            "value": "page",
            "property": "object"
        },
        "page_size": 100
    });
    
    if debug_mode {
        println!("DEBUG: Fetching all pages - Request URL: {}", url);
        println!("DEBUG: Request body: {}", serde_json::to_string_pretty(&request_body).unwrap());
    }

    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", notion_api_key))
        .header("Notion-Version", "2022-06-28")
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    let response_text = response.text().await?;
    
    if debug_mode {
        println!("DEBUG: Fetch all pages response: {}", response_text);
    }

    let search_response: NotionSearchResponse = serde_json::from_str(&response_text)?;
    Ok(search_response.results)
}

async fn fetch_page_content(page_id: &str, debug_mode: bool) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let notion_api_key = env::var("NOTION_API_KEY").expect("NOTION_API_KEY must be set");
    let client = Client::new();
    
    // Get page blocks
    let blocks_url = format!("https://api.notion.com/v1/blocks/{}/children?page_size=100", page_id);
    if debug_mode {
        println!("DEBUG: Fetching page blocks: {}", blocks_url);
    }
    
    let blocks_response = client
        .get(&blocks_url)
        .header("Authorization", format!("Bearer {}", notion_api_key))
        .header("Notion-Version", "2022-06-28")
        .send()
        .await?;
    
    let blocks_json: Value = blocks_response.json().await?;
    let blocks = blocks_json["results"].as_array().cloned().unwrap_or_default();
    
    Ok(blocks)
}

fn convert_blocks_to_markdown(blocks: &[Value]) -> String {
    let mut markdown = String::new();
    
    for block in blocks {
        if let Some(block_type) = block["type"].as_str() {
            match block_type {
                "heading_1" => {
                    if let Some(text) = extract_rich_text(&block["heading_1"]["rich_text"]) {
                        markdown.push_str(&format!("# {}\n\n", text));
                    }
                },
                "heading_2" => {
                    if let Some(text) = extract_rich_text(&block["heading_2"]["rich_text"]) {
                        markdown.push_str(&format!("## {}\n\n", text));
                    }
                },
                "heading_3" => {
                    if let Some(text) = extract_rich_text(&block["heading_3"]["rich_text"]) {
                        markdown.push_str(&format!("### {}\n\n", text));
                    }
                },
                "paragraph" => {
                    if let Some(text) = extract_rich_text(&block["paragraph"]["rich_text"]) {
                        markdown.push_str(&format!("{}\n\n", text));
                    }
                },
                "bulleted_list_item" => {
                    if let Some(text) = extract_rich_text(&block["bulleted_list_item"]["rich_text"]) {
                        markdown.push_str(&format!("- {}\n", text));
                    }
                },
                "code" => {
                    if let Some(text) = extract_rich_text(&block["code"]["rich_text"]) {
                        let language = block["code"]["language"].as_str().unwrap_or("");
                        markdown.push_str(&format!("```{}\n{}\n```\n\n", language, text));
                    }
                },
                _ => {
                    // For unsupported types, just add a newline
                    markdown.push('\n');
                }
            }
        }
    }
    
    markdown
}

fn extract_rich_text(rich_text: &Value) -> Option<String> {
    if let Some(array) = rich_text.as_array() {
        let mut text = String::new();
        for item in array {
            if let Some(t) = item["plain_text"].as_str() {
                text.push_str(t);
            }
        }
        Some(text)
    } else {
        None
    }
}

fn parse_flashcards_from_markdown(markdown: &str, debug_mode: bool) -> Vec<Flashcard> {
    let mut flashcards = Vec::new();
    let mut in_code_block = false;
    let mut current_question = None;
    let mut current_answer = String::new();
    
    for line in markdown.lines() {
        let line = line.trim();
        
        // Detect code blocks
        if line.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        
        // Only process lines inside code blocks
        if !in_code_block {
            continue;
        }
        
        // Support both Chinese and English markers
        if line.starts_with("问题:") || line.starts_with("问题：") || 
           line.starts_with("Question:") || line.starts_with("Question：") {
            // Save previous flashcard if exists
            if let Some(question) = current_question.take() {
                if !current_answer.is_empty() {
                    flashcards.push(Flashcard {
                        question,
                        answer: current_answer.trim().to_string(),
                    });
                    current_answer.clear();
                }
            }
            // Start new question
            current_question = Some(
                line.trim_start_matches("问题:")
                    .trim_start_matches("问题：")
                    .trim_start_matches("Question:")
                    .trim_start_matches("Question：")
                    .trim()
                    .to_string()
            );
        } else if line.starts_with("答案:") || line.starts_with("答案：") || 
                  line.starts_with("Answer:") || line.starts_with("Answer：") || 
                  line.starts_with("回答:") || line.starts_with("回答：") {
            if current_question.is_some() {
                current_answer.push_str(
                    line.trim_start_matches("答案:")
                        .trim_start_matches("答案：")
                        .trim_start_matches("Answer:")
                        .trim_start_matches("Answer：")
                        .trim_start_matches("回答:")
                        .trim_start_matches("回答：")
                        .trim()
                );
                current_answer.push('\n');
            }
        } else if current_question.is_some() && !line.is_empty() {
            if !current_answer.is_empty() {
                current_answer.push('\n');
            }
            current_answer.push_str(line);
        }
    }
    
    // Add last flashcard
    if let Some(question) = current_question {
        if !current_answer.is_empty() {
            flashcards.push(Flashcard {
                question,
                answer: current_answer.trim().to_string(),
            });
        }
    }
    
    if debug_mode {
        println!("DEBUG: Total parsed flashcards: {}", flashcards.len());
    }
    
    flashcards
}

async fn add_note_to_anki(flashcard: &Flashcard, debug_mode: bool) -> Result<(), Box<dyn std::error::Error>> {
    let anki_connect_url = env::var("ANKI_CONNECT_URL")
        .unwrap_or_else(|_| "http://localhost:8765".to_string());
    let anki_model_name = env::var("ANKI_MODEL_NAME")
        .unwrap_or_else(|_| "Basic".to_string());
    
    // Use "Basic" if available, otherwise try "基本"
    let model_name = if anki_model_name == "Basic" {
        "基本"
    } else {
        &anki_model_name
    };
    
    let note_data = json!({
        "action": "addNote",
        "version": 6,
        "params": {
            "note": {
                "deckName": "Notion Import",
                "modelName": model_name,
                "fields": {
                    "Front": flashcard.question,
                    "Back": flashcard.answer
                }
            }
        }
    });
    
    if debug_mode {
        println!("DEBUG: Adding note to Anki: {}", serde_json::to_string_pretty(&note_data).unwrap());
    }
    
    let client = Client::new();
    let response = client
        .post(&anki_connect_url)
        .json(&note_data)
        .send()
        .await?;
    
    let response_text = response.text().await?;
    
    if debug_mode {
        println!("DEBUG: Anki-Connect raw response: {}", response_text);
    }
    
    let response_json: Value = serde_json::from_str(&response_text)?;
    
    if debug_mode {
        println!("DEBUG: Anki-Connect parsed response: {}", serde_json::to_string_pretty(&response_json)?);
    }
    
    // Check if the operation was successful
    if response_json["error"].is_null() {
        println!("Added card: {}", flashcard.question);
        Ok(())
    } else {
        Err(format!("Anki-Connect error: {}", response_json["error"]).into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    // Read debug mode setting
    let debug_mode = env::var("DEBUG_MODE")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);
    
    if debug_mode {
        println!("DEBUG: Debug mode enabled");
    }
    
    let pages = fetch_all_pages(debug_mode).await?;
    println!("Found {} pages to import", pages.len());
    
    let mut success_count = 0;
    for page in pages {
        if debug_mode {
            println!("DEBUG: Processing page: {}", page.id);
        }
        
        let blocks = fetch_page_content(&page.id, debug_mode).await?;
        let markdown = convert_blocks_to_markdown(&blocks);
        let flashcards = parse_flashcards_from_markdown(&markdown, debug_mode);
        
        for flashcard in flashcards {
            if add_note_to_anki(&flashcard, debug_mode).await.is_ok() {
                success_count += 1;
            }
        }
    }
    
    println!("Successfully imported {} flashcards to Anki", success_count);
    Ok(())
}

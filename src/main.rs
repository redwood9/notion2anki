use dotenvy::dotenv;
use reqwest::Client;
use serde::Deserialize;
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
}

#[derive(Deserialize, Debug)]
struct NotionSearchResponse {
    results: Vec<NotionPage>,
}


async fn fetch_all_pages() -> Result<Vec<NotionPage>, reqwest::Error> {
    let notion_api_key = env::var("NOTION_API_KEY").expect("NOTION_API_KEY must be set");
    let url = "https://api.notion.com/v1/search";

    let client = Client::new();
    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", notion_api_key))
        .header("Notion-Version", "2022-06-28")
        .header("Content-Type", "application/json")
        .json(&json!({
            "filter": {
                "value": "page",
                "property": "object"
            },
            "page_size": 100
        }))
        .send()
        .await?;

    let search_response: NotionSearchResponse = response.json().await?;
    Ok(search_response.results)
}

async fn fetch_page_content(page_id: &str) -> Result<String, reqwest::Error> {
    let notion_api_key = env::var("NOTION_API_KEY").expect("NOTION_API_KEY must be set");
    let client = Client::new();
    
    // 获取页面元数据
    let page_data = client
        .get(&format!("https://api.notion.com/v1/pages/{}", page_id))
        .header("Authorization", format!("Bearer {}", notion_api_key))
        .header("Notion-Version", "2022-06-28")
        .send()
        .await?
        .text()
        .await?;
    
    // 获取页面内容块
    let blocks_data = client
        .get(&format!("https://api.notion.com/v1/blocks/{}/children?page_size=100", page_id))
        .header("Authorization", format!("Bearer {}", notion_api_key))
        .header("Notion-Version", "2022-06-28")
        .send()
        .await?
        .text()
        .await?;
    
    // 合并页面数据和块数据
    Ok(format!("{{\"page\": {}, \"blocks\": {}}}", page_data, blocks_data))
}

fn parse_flashcards(content: &str) -> Vec<Flashcard> {
    
    let mut flashcards = Vec::new();
    let mut current_question = String::new();
    let mut current_answer = String::new();
    let mut in_flashcard = false;
    
    // 解析JSON内容
    let page_content: serde_json::Value = serde_json::from_str(content).unwrap_or_default();
    let empty_vec = Vec::new();
    let blocks = page_content["blocks"].as_array().unwrap_or(&empty_vec);
    
    for block in blocks {
        if let Some(block_obj) = block.as_object() {
            let text = if let Some(para) = block_obj.get("paragraph") {
                para["rich_text"][0]["plain_text"].as_str().unwrap_or("").to_string()
            } else if let Some(heading) = block_obj.get("heading_1") {
                heading["rich_text"][0]["plain_text"].as_str().unwrap_or("").to_string()
            } else if let Some(heading) = block_obj.get("heading_2") {
                heading["rich_text"][0]["plain_text"].as_str().unwrap_or("").to_string()
            } else if let Some(heading) = block_obj.get("heading_3") {
                heading["rich_text"][0]["plain_text"].as_str().unwrap_or("").to_string()
            } else {
                continue;
            };
            
            // 检测问题开始
            if text.starts_with("问题:") || text.starts_with("问题：") {
                if in_flashcard && !current_question.is_empty() {
                    flashcards.push(Flashcard {
                        question: current_question.trim().to_string(),
                        answer: current_answer.trim().to_string(),
                    });
                    current_answer.clear();
                }
                current_question = text.replacen("问题:", "", 1).replacen("问题：", "", 1).trim().to_string();
                in_flashcard = true;
            } 
            // 检测答案开始
            else if in_flashcard && (text.starts_with("答案:") || text.starts_with("答案：")) {
                current_answer.push_str(&text.replacen("答案:", "", 1).replacen("答案：", "", 1));
            }
            // 收集答案内容
            else if in_flashcard && !current_question.is_empty() {
                if !current_answer.is_empty() {
                    current_answer.push('\n');
                }
                current_answer.push_str(&text);
            }
        }
    }
    
    // 添加最后一个卡片
    if in_flashcard && !current_question.is_empty() {
        flashcards.push(Flashcard {
            question: current_question.trim().to_string(),
            answer: current_answer.trim().to_string(),
        });
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
    
    let pages = fetch_all_pages().await?;
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

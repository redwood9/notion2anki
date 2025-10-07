use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::Path;

/// Notion2Anki - Import flashcards from Notion to Anki
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path (supports JSON or TOML format)
    #[arg(short, long)]
    config: Option<String>,

    /// Notion API key
    #[arg(long)]
    notion_api_key: Option<String>,

    /// Anki-Connect URL
    #[arg(long)]
    anki_connect_url: Option<String>,

    /// Enable or disable debug mode
    #[arg(long)]
    debug: Option<bool>,
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    /// Notion API key
    notion_api_key: String,
    /// Enable detailed debug logging
    debug_mode: bool,
    /// Anki-Connect URL
    anki_connect_url: String,
}

impl Config {
    /// Create configuration from command line arguments and environment variables
    /// Priority: CLI args > Config file > Environment variables > Default values
    /// Once a priority level is hit, lower priorities are not checked
    fn from_args_and_env(args: &Args) -> Result<Self, String> {
        let mut config = Config {
            notion_api_key: String::new(),
            debug_mode: false,
            anki_connect_url: "http://localhost:8765".to_string(),
        };

        // Priority 1: Command line arguments (highest priority)
        let has_cli_args = args.notion_api_key.is_some() || 
                          args.anki_connect_url.is_some() || 
                          args.debug.is_some();

        if has_cli_args {
            // Get configuration from command line arguments
            if let Some(notion_key) = &args.notion_api_key {
                config.notion_api_key = notion_key.clone();
            }
            if let Some(anki_url) = &args.anki_connect_url {
                config.anki_connect_url = anki_url.clone();
            }
            if let Some(debug_value) = args.debug {
                config.debug_mode = debug_value;
            }
            
            // If CLI args are incomplete, supplement from config file
            if let Some(config_path) = &args.config {
                let file_config = Self::load_from_file(config_path)?;
                if config.notion_api_key.is_empty() {
                    config.notion_api_key = file_config.notion_api_key;
                }
                if config.anki_connect_url == "http://localhost:8765" {
                    config.anki_connect_url = file_config.anki_connect_url;
                }
                if args.debug.is_none() {
                    config.debug_mode = file_config.debug_mode;
                }
            }
        }
        // Priority 2: Configuration file (second priority)
        else if let Some(config_path) = &args.config {
            config = Self::load_from_file(config_path)?;
        }
        // Priority 3: Environment variables (lowest priority)
        else {
            if let Ok(notion_key) = env::var("NOTION_API_KEY") {
                config.notion_api_key = notion_key;
            }
            if let Ok(anki_url) = env::var("ANKI_CONNECT_URL") {
                config.anki_connect_url = anki_url;
            }
            if let Ok(debug_mode) = env::var("DEBUG_MODE") {
                config.debug_mode = debug_mode.to_lowercase() == "true";
            }
        }

        // Validate required parameters
        config.validate()?;

        Ok(config)
    }

    /// Load configuration from file
    fn load_from_file(path: &str) -> Result<Self, String> {
        let path = Path::new(path);
        if !path.exists() {
            return Err(format!("Configuration file does not exist: {}", path.display()));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read configuration file: {}", e))?;

        if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            toml::from_str(&content)
                .map_err(|e| format!("Failed to parse TOML configuration file: {}", e))
        } else {
            serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse JSON configuration file: {}", e))
        }
    }

    /// Validate required parameters
    fn validate(&self) -> Result<(), String> {
        if self.notion_api_key.is_empty() {
            return Err("Missing required parameter: NOTION_API_KEY".to_string());
        }
        if self.anki_connect_url.is_empty() {
            return Err("Missing required parameter: ANKI_CONNECT_URL".to_string());
        }
        Ok(())
    }

    /// Print usage help
    fn print_usage() {
        println!("Usage:");
        println!("  notion2anki [OPTIONS]");
        println!();
        println!("Options:");
        println!("  -c, --config <FILE>           Specify configuration file path (JSON or TOML format)");
        println!("  --notion-api-key <KEY>        Notion API key");
        println!("  --anki-connect-url <URL>      Anki-Connect URL");
        println!("  --debug <true|false>          Enable or disable debug mode");
        println!("  -h, --help                    Show help information");
        println!();
        println!("Configuration Priority (Hit-based Priority):");
        println!("  1. Command line arguments (highest) - Once hit, no other sources are used");
        println!("  2. Configuration file (second) - Used when --config is specified");
        println!("  3. Environment variables (lowest) - Used when no CLI args and no config file");
        println!();
        println!("Environment Variables:");
        println!("  NOTION_API_KEY          Notion API key");
        println!("  ANKI_CONNECT_URL        Anki-Connect URL (default: http://localhost:8765)");
        println!("  DEBUG_MODE              Enable debug mode (true/false)");
        println!();
        println!("Configuration File Example (config.toml):");
        println!("  notion_api_key = \"your_notion_api_key\"");
        println!("  anki_connect_url = \"http://localhost:8765\"");
        println!("  debug_mode = false");
        println!();
        println!("Configuration File Example (config.json):");
        println!("  {{");
        println!("    \"notion_api_key\": \"your_notion_api_key\",");
        println!("    \"anki_connect_url\": \"http://localhost:8765\",");
        println!("    \"debug_mode\": false");
        println!("  }}");
    }
}

#[derive(Debug)]
struct Flashcard {
    question: String,
    answer: String,
}

#[derive(Deserialize, Debug)]
struct NotionPage {
    id: String,
    properties: Value,
}

#[derive(Deserialize, Debug)]
struct NotionSearchResponse {
    results: Vec<NotionPage>,
}

fn extract_page_title(page: &NotionPage) -> String {
    // Try to extract title from properties
    if let Some(title_prop) = page.properties.get("title") {
        if let Some(title_array) = title_prop.get("title").and_then(|t| t.as_array()) {
            if let Some(first_title) = title_array.first() {
                if let Some(plain_text) = first_title.get("plain_text").and_then(|t| t.as_str()) {
                    return plain_text.to_string();
                }
            }
        }
    }
    // Fallback to page ID if title cannot be extracted
    format!("Page-{}", &page.id[..8])
}

async fn fetch_all_pages(config: &Config) -> Result<Vec<NotionPage>, Box<dyn std::error::Error>> {
    let notion_api_key = &config.notion_api_key;
    let url = "https://api.notion.com/v1/search";

    let client = Client::new();
    let request_body = json!({
        "filter": {
            "value": "page",
            "property": "object"
        },
        "page_size": 100
    });
    
    if config.debug_mode {
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
    
    if config.debug_mode {
        println!("DEBUG: Fetch all pages response: {}", response_text);
    }

    let search_response: NotionSearchResponse = serde_json::from_str(&response_text)?;
    Ok(search_response.results)
}

async fn fetch_and_parse_page_content(page_id: &str, config: &Config) -> Result<Vec<Flashcard>, Box<dyn std::error::Error>> {
    let notion_api_key = &config.notion_api_key;
    let client = Client::new();
    
    let mut all_flashcards = Vec::new();
    let mut start_cursor: Option<String> = None;
    let mut page_number = 1;
    let mut total_blocks = 0;
    
    loop {
        // Get page blocks with pagination
        let mut blocks_url = format!("https://api.notion.com/v1/blocks/{}/children?page_size=100", page_id);
        if let Some(cursor) = &start_cursor {
            blocks_url.push_str(&format!("&start_cursor={}", cursor));
        }
        
        if config.debug_mode {
            println!("DEBUG: Fetching page blocks (batch {}): {}", page_number, blocks_url);
        } else {
            println!("Fetching batch {} data...", page_number);
        }
        
        let blocks_response = client
            .get(&blocks_url)
            .header("Authorization", format!("Bearer {}", notion_api_key))
            .header("Notion-Version", "2022-06-28")
            .send()
            .await?;
        
        let blocks_json: Value = blocks_response.json().await?;
        
        if config.debug_mode {
            println!("DEBUG: blocks_json (batch {}):: {}", page_number, blocks_json.to_string());
        }
        
        // Process current batch of blocks
        if let Some(results) = blocks_json["results"].as_array() {
            let current_count = results.len();
            total_blocks += current_count;
            println!("Batch {}: Fetched {} blocks", page_number, current_count);
            
            // Analyze current batch immediately
            println!("Analyzing batch {} data...", page_number);
            let markdown = convert_blocks_to_markdown(results);
            let flashcards = parse_flashcards_from_markdown(&markdown, config, page_number);
            
            println!("Batch {}: Parsed {} flashcards\n", page_number, flashcards.len());
            all_flashcards.extend(flashcards);
        }
        
        // Check if there are more pages
        let has_more = blocks_json["has_more"].as_bool().unwrap_or(false);
        if !has_more {
            println!("All data fetched successfully");
            break;
        }
        
        // Get next page cursor
        start_cursor = blocks_json["next_cursor"].as_str().map(|s| s.to_string());
        if start_cursor.is_none() {
            break;
        }
        
        page_number += 1;
    }
    
    println!("Total blocks fetched: {}", total_blocks);
    println!("Total flashcards parsed: {}\n", all_flashcards.len());
    Ok(all_flashcards)
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

fn parse_flashcards_from_markdown(markdown: &str, config: &Config, batch_number: usize) -> Vec<Flashcard> {
    let mut flashcards = Vec::new();
    let mut in_code_block = false;
    let mut current_question = None;
    let mut current_answer = String::new();
    let mut current_code_block = String::new();
    let mut code_block_count = 0;
    
    for line in markdown.lines() {
        let line = line.trim();
        
        // Detect code blocks
        if line.starts_with("```") {
            if in_code_block {
                // End of code block - print it
                code_block_count += 1;
                println!("=== Batch {} - Code Block {} ===", batch_number, code_block_count);
                println!("{}", current_code_block);
                println!("=== End of Code Block ===\n");
                current_code_block.clear();
            }
            in_code_block = !in_code_block;
            continue;
        }
        
        // Collect code block content
        if in_code_block {
            current_code_block.push_str(line);
            current_code_block.push('\n');
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
    
    if config.debug_mode {
        println!("DEBUG: Total parsed flashcards in batch {}: {}", batch_number, flashcards.len());
    }
    
    flashcards
}

async fn create_deck_if_not_exists(deck_name: &str, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let anki_connect_url = &config.anki_connect_url;
    let client = Client::new();
    
    let create_deck_data = json!({
        "action": "createDeck",
        "version": 6,
        "params": {
            "deck": deck_name
        }
    });
    
    if config.debug_mode {
        println!("DEBUG: Creating deck: {}", serde_json::to_string_pretty(&create_deck_data).unwrap());
    }
    
    let response = client
        .post(anki_connect_url)
        .json(&create_deck_data)
        .send()
        .await?;
    
    let response_text = response.text().await?;
    let response_json: Value = serde_json::from_str(&response_text)?;
    
    if config.debug_mode {
        println!("DEBUG: Create deck response: {}", serde_json::to_string_pretty(&response_json)?);
    }
    
    Ok(())
}

async fn clear_deck(deck_name: &str, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let anki_connect_url = &config.anki_connect_url;
    let client = Client::new();
    
    // Get all cards in the deck
    let find_cards_data = json!({
        "action": "findCards",
        "version": 6,
        "params": {
            "query": format!("deck:\"{}\"", deck_name)
        }
    });
    
    if config.debug_mode {
        println!("DEBUG: Finding cards in deck: {}", serde_json::to_string_pretty(&find_cards_data).unwrap());
    }
    
    let response = client
        .post(anki_connect_url)
        .json(&find_cards_data)
        .send()
        .await?;
    
    let response_text = response.text().await?;
    let response_json: Value = serde_json::from_str(&response_text)?;
    
    if let Some(card_ids) = response_json["result"].as_array() {
        if !card_ids.is_empty() {
            println!("Clearing {} cards from deck \"{}\"", card_ids.len(), deck_name);
            
            // Delete all cards
            let delete_cards_data = json!({
                "action": "deleteNotes",
                "version": 6,
                "params": {
                    "notes": card_ids
                }
            });
            
            if config.debug_mode {
                println!("DEBUG: Deleting cards: {}", serde_json::to_string_pretty(&delete_cards_data).unwrap());
            }
            
            let delete_response = client
                .post(anki_connect_url)
                .json(&delete_cards_data)
                .send()
                .await?;
            
            let delete_response_text = delete_response.text().await?;
            
            if config.debug_mode {
                println!("DEBUG: Delete cards response: {}", delete_response_text);
            }
        } else {
            println!("Deck \"{}\" is empty, no need to clear", deck_name);
        }
    }
    
    Ok(())
}

async fn add_note_to_anki(flashcard: &Flashcard, deck_name: &str, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let anki_connect_url = &config.anki_connect_url;
    let anki_model_name = env::var("ANKI_MODEL_NAME")
        .unwrap_or_else(|_| "Basic".to_string());
    
    // Use "基本" (Chinese Basic) if available, otherwise use specified model
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
                "deckName": deck_name,
                "modelName": model_name,
                "fields": {
                    "Front": flashcard.question,
                    "Back": flashcard.answer
                }
            }
        }
    });
    
    if config.debug_mode {
        println!("DEBUG: Adding note to Anki: {}", serde_json::to_string_pretty(&note_data).unwrap());
    }
    
    let client = Client::new();
    let response = client
        .post(anki_connect_url)
        .json(&note_data)
        .send()
        .await?;
    
    let response_text = response.text().await?;
    
    if config.debug_mode {
        println!("DEBUG: Anki-Connect raw response: {}", response_text);
    }
    
    let response_json: Value = serde_json::from_str(&response_text)?;
    
    if config.debug_mode {
        println!("DEBUG: Anki-Connect parsed response: {}", serde_json::to_string_pretty(&response_json)?);
    }
    
    // Check if the operation was successful
    if response_json["error"].is_null() {
        Ok(())
    } else {
        Err(format!("Anki-Connect error: {}", response_json["error"]).into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = Args::parse();
    
    // Create configuration
    let config = match Config::from_args_and_env(&args) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("Error: {}", error);
            println!();
            Config::print_usage();
            std::process::exit(1);
        }
    };
    
    if config.debug_mode {
        println!("DEBUG: Debug mode enabled");
        println!("DEBUG: Configuration: {:?}", config);
    }
    
    let pages = fetch_all_pages(&config).await?;
    println!("Found {} pages to import", pages.len());
    
    let mut success_count = 0;
    for page in pages {
        // Extract page title to use as deck name
        let deck_name = extract_page_title(&page);
        println!("\n========================================");
        println!("Processing page: \"{}\" (ID: {})", deck_name, page.id);
        println!("========================================\n");
        
        if config.debug_mode {
            println!("DEBUG: Processing page: {}", page.id);
        }
        
        // Create deck if not exists
        create_deck_if_not_exists(&deck_name, &config).await?;
        
        // Clear existing cards in the deck (full update)
        clear_deck(&deck_name, &config).await?;
        
        // Fetch and parse page content (with pagination and batch processing)
        let flashcards = fetch_and_parse_page_content(&page.id, &config).await?;
        
        // Import all flashcards to Anki at once
        if !flashcards.is_empty() {
            println!("Importing {} flashcards to deck \"{}\"...", flashcards.len(), deck_name);
            for (index, flashcard) in flashcards.iter().enumerate() {
                if add_note_to_anki(flashcard, &deck_name, &config).await.is_ok() {
                    success_count += 1;
                    println!("  [{}/{}] Successfully added card", index + 1, flashcards.len());
                } else {
                    println!("  [{}/{}] Failed to add card", index + 1, flashcards.len());
                }
            }
        } else {
            println!("No importable flashcards found in this page");
        }
        
        println!("\nCompleted importing page \"{}\"", deck_name);
    }
    
    println!("\n========================================");
    println!("Successfully imported {} flashcards to Anki", success_count);
    println!("========================================");
    Ok(())
}

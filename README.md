# Notion to Anki Importer (Rust CLI)

This tool imports flashcards from Notion pages into Anki using Anki-Connect. 
It parses pages with content in a specific flashcard format.

## Prerequisites

1. Rust 1.70+ (install via [rustup](https://rustup.rs/))
2. Anki installed with [Anki-Connect](https://foosoft.net/projects/anki-connect/) add-on
3. Notion integration token

## Setup

1. Build the project:
```bash
cargo build --release
```

2. Create a `.env` file with your credentials:
```env
NOTION_API_KEY=your_integration_token_here
NOTION_DATABASE_ID=your_database_id_here
ANKI_CONNECT_URL=http://localhost:8765  # Default value
```

3. Configure Notion database:
- Create a database with a **Status** property (Select type) to filter import-ready cards
- Pages should contain flashcards in this format:
  ```
  问题: 你的问题内容
  答案: 你的答案内容
  ```
  - Both Chinese (`问题：`) and English (`问题:`) colons are supported
  - Answers can be on the same line or on new lines
  - Multiple flashcards per page are supported
  - Example:
    ```
    问题: What is the capital of France?
    答案: Paris
    
    问题: 水的化学式是什么？
    答案: 
    H₂O
    ```

- Share your database with your Notion integration

## Usage

1. Set card status to "Ready to Import" in Notion
2. Run the importer:
```bash
./target/release/notion2anki
```

## Configuration Options

- `NOTION_API_KEY`: Your Notion integration token
- `NOTION_DATABASE_ID`: ID of your Notion database
- `ANKI_CONNECT_URL`: Anki-Connect endpoint (default: http://localhost:8765)

## Notes

- Cards will be imported to a deck named "Notion Import"
- Uses the "Basic" card model by default
- After import, you should update card status in Notion manually

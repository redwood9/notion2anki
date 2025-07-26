# Notion to Anki Importer (Rust CLI)

This tool imports flashcards from all accessible Notion pages into Anki using Anki-Connect. 
It automatically scans and processes any pages containing flashcards in the specified format.

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
ANKI_CONNECT_URL=http://localhost:8765  # Default value
```

3. Share your Notion pages with the integration:
   - In Notion, share each page containing flashcards with your integration
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

## Usage

1. Run the importer:
```bash
./target/release/notion2anki
```

The tool will:
- Fetch all pages accessible to your Notion integration
- Parse pages for flashcards in the specified format
- Import valid flashcards into Anki

## Configuration Options

- `NOTION_API_KEY`: Your Notion integration token
- `ANKI_CONNECT_URL`: Anki-Connect endpoint (default: http://localhost:8765)

## Notes

- Cards will be imported to a deck named "Notion Import"
- Uses the "Basic" card model by default
- Only pages shared with your integration will be processed

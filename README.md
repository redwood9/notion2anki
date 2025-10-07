# Notion to Anki Importer (Rust CLI)

This tool imports flashcards from all accessible Notion pages into Anki using Anki-Connect. 
It automatically scans and processes any pages containing flashcards in the specified format.

## Prerequisites

1. Rust 1.70+ (install via [rustup](https://rustup.rs/))
2. Anki installed with [Anki-Connect](https://foosoft.net/projects/anki-connect/) add-on
3. Notion integration token

## Installation

1. Build the project:
```bash
cargo build --release
```

## Configuration Methods

### Method 1: Configuration File

Create a configuration file `config.toml` or `config.json`:

**config.toml:**
```toml
notion_api_key = "your_notion_api_key_here"
anki_connect_url = "http://localhost:8765"
debug_mode = false
```

**config.json:**
```json
{
  "notion_api_key": "your_notion_api_key_here",
  "anki_connect_url": "http://localhost:8765",
  "debug_mode": false
}
```

### Method 2: Environment Variables

Create a `.env` file:
```env
NOTION_API_KEY=your_integration_token_here
ANKI_CONNECT_URL=http://localhost:8765
DEBUG_MODE=false
```

### Method 3: Command Line Arguments

Pass parameters directly via command line:
```bash
./target/release/notion2anki --notion-api-key "your_key" --anki-connect-url "http://localhost:8765" --debug
```

## Usage

### Command Line Options

```bash
# Use configuration file
./target/release/notion2anki --config config.toml

# Use command line arguments
./target/release/notion2anki --notion-api-key "your_key" --anki-connect-url "http://localhost:8765"

# Enable debug mode
./target/release/notion2anki --debug true

# Disable debug mode
./target/release/notion2anki --debug false

# Show help
./target/release/notion2anki --help
```

### Command Line Arguments

- `-c, --config <file>`: Specify configuration file path (JSON or TOML format)
- `--notion-api-key <key>`: Notion API key
- `--anki-connect-url <url>`: Anki-Connect URL
- `--debug <true|false>`: Enable or disable debug mode
- `-h, --help`: Show help information

### Configuration Priority (Hit-based Priority)

1. **Command line arguments** (highest priority) - Once hit, no other sources are used
2. **Configuration file** (second priority) - Used when `--config` is specified
3. **Environment variables** (lowest priority) - Used when no CLI args and no config file
4. **Default values** - Used as fallback

## Notion Page Setup

1. In Notion, share pages containing flashcards with your integration
2. Questions and answers must be in code blocks
3. Pages should contain flashcards in this format:

```
问题: Your question content
答案: Your answer content
```

- Both Chinese and English colons are supported
- Answers can be on the same line or new lines
- Multiple flashcards per page are supported
- Example:

```
问题: What is the capital of France?
答案: Paris

问题: 水的化学式是什么？
回答: 
H₂O

Question: What is Newton's first law?
Answer: An object at rest stays at rest
```

## Running

```bash
./target/release/notion2anki
```

The tool will:
- Fetch all pages accessible to your Notion integration
- Use each page's title as the Anki deck name
- Parse pages for flashcards in the specified format
- Display all code blocks found in each page
- Clear existing cards in the deck (full update)
- Import valid flashcards into Anki
- Generate detailed log file when DEBUG_MODE is enabled

## Configuration Options

- `NOTION_API_KEY`: Your Notion integration token (required)
- `ANKI_CONNECT_URL`: Anki-Connect endpoint (required, default: http://localhost:8765)
- `DEBUG_MODE`: Set to "true" to enable detailed debug logging (optional, default: false)

## Debugging

To troubleshoot issues, enable debug mode in your configuration. This will:
- Log detailed debug information to console
- Generate a log file with detailed execution trace
- Include:
  - Notion API requests and responses
  - Parsed flashcard content
  - Anki import details

## Notes

- Each Notion page creates a separate Anki deck using the page title as the deck name
- If a deck already exists, it will be cleared and updated with new cards (full update)
- All code blocks in each page will be displayed during processing
- Uses the "Basic" (基本) card model by default
- Only pages shared with your integration will be processed
- Detailed logs are saved when DEBUG_MODE is enabled

---

# Notion to Anki 导入工具 (Rust CLI)

这个工具从所有可访问的 Notion 页面导入闪卡到 Anki，使用 Anki-Connect。
它会自动扫描和处理包含指定格式闪卡的页面。

## 前置要求

1. Rust 1.70+ (通过 [rustup](https://rustup.rs/) 安装)
2. 安装有 [Anki-Connect](https://foosoft.net/projects/anki-connect/) 插件的 Anki
3. Notion 集成令牌

## 安装

1. 构建项目:
```bash
cargo build --release
```

## 配置方式

### 方式 1: 使用配置文件

创建配置文件 `config.toml` 或 `config.json`:

**config.toml:**
```toml
notion_api_key = "your_notion_api_key_here"
anki_connect_url = "http://localhost:8765"
debug_mode = false
```

**config.json:**
```json
{
  "notion_api_key": "your_notion_api_key_here",
  "anki_connect_url": "http://localhost:8765",
  "debug_mode": false
}
```

### 方式 2: 使用环境变量

创建 `.env` 文件:
```env
NOTION_API_KEY=your_integration_token_here
ANKI_CONNECT_URL=http://localhost:8765
DEBUG_MODE=false
```

### 方式 3: 使用命令行参数

直接通过命令行传递参数:
```bash
./target/release/notion2anki --notion-api-key "your_key" --anki-connect-url "http://localhost:8765" --debug
```

## 使用方法

### 命令行选项

```bash
# 使用配置文件
./target/release/notion2anki --config config.toml

# 使用命令行参数
./target/release/notion2anki --notion-api-key "your_key" --anki-connect-url "http://localhost:8765"

# 启用调试模式
./target/release/notion2anki --debug true

# 禁用调试模式
./target/release/notion2anki --debug false

# 查看帮助
./target/release/notion2anki --help
```

### 命令行参数说明

- `-c, --config <文件>`: 指定配置文件路径 (JSON 或 TOML 格式)
- `--notion-api-key <密钥>`: Notion API 密钥
- `--anki-connect-url <URL>`: Anki-Connect URL
- `--debug <true|false>`: 启用或禁用调试模式
- `-h, --help`: 显示帮助信息

### 配置优先级 (命中式优先级)

1. **命令行参数** (最高优先级) - 命中后不从其他源获取
2. **配置文件** (次高优先级) - 指定 --config 时使用
3. **环境变量** (最末优先级) - 无命令行参数且无配置文件时使用
4. **默认值** - 作为后备使用

## Notion 页面设置

1. 在 Notion 中，将包含闪卡的页面分享给你的集成
2. 问题和答案必须在代码块中
3. 页面应包含以下格式的闪卡:

```
问题: 你的问题内容
答案: 你的答案内容
```

- 支持中英文冒号
- 答案可以在同一行或新行
- 每页支持多个闪卡
- 示例:

```
问题: What is the capital of France?
答案: Paris

问题: 水的化学式是什么？
回答: 
H₂O

Question: What is Newton's first law?
Answer: An object at rest stays at rest
```

## 运行

```bash
./target/release/notion2anki
```

工具将:
- 获取你的 Notion 集成可访问的所有页面
- 使用每个页面的标题作为 Anki 牌组名
- 解析页面中的闪卡格式
- 显示每个页面中找到的所有代码块
- 清空牌组中的现有卡片（全量更新）
- 将有效闪卡导入到 Anki
- 在启用 DEBUG_MODE 时生成详细的日志文件

## 配置选项

- `NOTION_API_KEY`: 你的 Notion 集成令牌 (必需)
- `ANKI_CONNECT_URL`: Anki-Connect 端点 (必需，默认: http://localhost:8765)
- `DEBUG_MODE`: 设置为 "true" 启用详细调试日志 (可选，默认: false)

## 调试

要排查问题，请在配置中启用调试模式。这将:
- 在控制台记录详细的调试信息
- 生成包含详细执行跟踪的日志文件
- 包括:
  - Notion API 请求和响应
  - 解析的闪卡内容
  - Anki 导入详情

## 注意事项

- 每个 Notion 页面创建一个独立的 Anki 牌组，使用页面标题作为牌组名
- 如果牌组已存在，将清空并用新卡片更新（全量更新）
- 处理过程中会显示每个页面的所有代码块
- 默认使用 "Basic" (基本) 卡片模型
- 只有与你的集成共享的页面才会被处理
- 启用 DEBUG_MODE 时，详细日志会保存到日志文件中
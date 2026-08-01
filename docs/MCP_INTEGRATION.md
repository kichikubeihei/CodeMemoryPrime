# MCP Integration & Client Setup Guide

This guide provides step-by-step instructions for connecting **CodeMemoryPrime (CMP)** to your favorite AI coding assistant or IDE.

---

## 1. Client Configuration Walkthroughs

### 🔹 Claude Desktop (macOS / Windows)
1. Open Claude Desktop preferences.
2. Edit `claude_desktop_config.json`:
   - **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
   - **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
3. Add the server entry:
```json
{
  "mcpServers": {
    "CodeMemoryPrime": {
      "command": "/absolute/path/to/CodeMemoryPrime/target/release/cmp",
      "args": [],
      "env": {
        "MCP_LLM_PROVIDER": "ollama",
        "MCP_LLM_BASE_URL": "http://127.0.0.1:11434",
        "MCP_LLM_GEN_MODEL": "qwen2.5-coder:7b",
        "MCP_LLM_EMBED_MODEL": "nomic-embed-text"
      }
    }
  }
}
```
4. Restart Claude Desktop.

---

### 🔹 Cursor IDE
1. Open Cursor Settings -> **Features** -> **MCP Servers**.
2. Click **+ Add New MCP Server**.
3. Fill in:
   - **Name**: `CodeMemoryPrime`
   - **Type**: `command`
   - **Command**: `/absolute/path/to/CodeMemoryPrime/target/release/cmp`
4. Click **Save**.

---

### 🔹 Windsurf IDE
1. Open `~/.codeium/windsurf/mcp_config.json`.
2. Insert the `CodeMemoryPrime` JSON configuration block.
3. Save and refresh MCP servers.

---

## 2. Recommended Agent Prompts

Once **CodeMemoryPrime** is connected, try these natural language prompts with your AI assistant:

### 🔍 Indexing & Code Search
- *"Index this workspace under project name 'my_app'."*
- *"Search codebase for where we handle JWT token validation."*
- *"Find all functions that import the database connection pool."*

### 🧠 Persistent Memory & Rules
- *"Save this architectural decision into memory: We use PostgreSQL for relational data and Redis for session cache."*
- *"Search memories for our database design rules."*
- *"Consolidate raw interactions into permanent memory facts."*

### 🛠️ Code Review & Refactoring
- *"Explain the implementation of `handle_request` in `handlers.rs`."*
- *"Refactor `parser.rs` to extract helper functions."*
- *"Run a security check on our authentication handler."*

### 📊 System Diagnostics
- *"Check CodeMemoryPrime project health."*
- *"List available local LLM models and configure settings."*

# CodeMemoryPrime (CMP) 🚀

> **The Blazing-Fast, Rust-Powered MCP Codebase Intelligence & Persistent Memory Engine for AI Coding Agents.**

[![License: BSL-1.1](https://img.shields.io/badge/License-BSL_1.1-blue.svg)](LICENSE.md)
[![Language: Rust](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)
[![MCP Spec: 2024-11-05](https://img.shields.io/badge/MCP_Spec-2024--11--05-green.svg)](https://modelcontextprotocol.io)

**CodeMemoryPrime (CMP)** is a single-binary, high-performance Model Context Protocol (MCP) server that connects local AI coding assistants (Claude Desktop, Cursor, Windsurf, Gemini CLI, Antigravity) to an AST-aware codebase RAG engine, factual memory store, and comprehensive developer toolkit.

---

## ✨ Key Features

- **🚀 Sub-200ms Re-Indexing**: Uses **Sub-Chunk SHA-256 Incremental Hashing** to skip untouched functions and re-index modified codebases in **0.192 seconds**.
- **🌳 AST Parent-Child Context**: Parses Tree-sitter language ASTs, attaching file imports and parent struct/class signatures directly to function snippets so LLMs never lose type context.
- **⚡ Single Compiled Native Binary**: Written in pure Rust with zero Node.js/npm or Python runtime dependencies. Boots in under 1ms.
- **🔍 Hybrid RRF RAG Search**: Combines semantic vector similarity, FTS5 keyword search, Reciprocal Rank Fusion (RRF), and dependency graph cross-referencing.
- **🛠️ 31 Integrated Developer Tools**: Complete suite including AST indexing, memory consolidation, file patching (`patch_file`), safe command execution, unified `git` CLI, refactoring, and project health checks.
- **🤖 Provider Agnostic**: Works natively with local **Ollama** (`qwen2.5-coder`, `nomic-embed-text`) or any OpenAI-compatible API (**LM Studio**, **vLLM**, **OpenAI**, **Groq**).
- **💡 Built-in Auto-Detection & Setup**: Automatically detects running local LLM endpoints, lists available local models, and provides actionable setup guidance.

---

## 🚀 Quick Start

### 1. Build from Source
```bash
git clone https://github.com/your-username/CodeMemoryPrime.git
cd CodeMemoryPrime
cargo build --release
```
The compiled binary will be located at `./target/release/cmp`.

---

### 2. Configure Your MCP Client

Add **CodeMemoryPrime** to your client configuration file:

#### 🔹 Ollama Configuration (Default / Free & Local)
```json
{
  "mcpServers": {
    "CodeMemoryPrime": {
      "command": "/path/to/CodeMemoryPrime/target/release/cmp",
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

#### 🔹 LM Studio / OpenAI / vLLM Configuration
```json
{
  "mcpServers": {
    "CodeMemoryPrime": {
      "command": "/path/to/CodeMemoryPrime/target/release/cmp",
      "args": [],
      "env": {
        "MCP_LLM_PROVIDER": "openai",
        "MCP_LLM_BASE_URL": "http://localhost:1234/v1",
        "MCP_LLM_GEN_MODEL": "qwen2.5-coder-7b-instruct",
        "MCP_LLM_EMBED_MODEL": "text-embedding-nomic-embed-text-v1.5",
        "MCP_LLM_API_KEY": "lm-studio"
      }
    }
  }
}
```

#### Config File Locations:
- **Claude Desktop**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Cursor**: `~/.cursor/mcp.json`
- **Windsurf**: `~/.codeium/windsurf/mcp_config.json`
- **Gemini CLI / Antigravity**: `~/.gemini/mcp_config.json`

---

## 🛠️ Tool Suite (31 MCP Tools)

| Tool Category | Available Tools |
|---------------|-----------------|
| **Codebase & RAG** | `index_workspace`, `search_codebase`, `get_dependencies` |
| **Persistent Memory** | `save_interaction`, `search_memories`, `consolidate_memories` |
| **File Operations** | `read_file`, `write_file`, `patch_file`, `list_files` |
| **Shell & Version Control** | `run_command` (allowlist protected), `git` |
| **AI Refactoring & Review** | `explain_code`, `refactor_code`, `review_code`, `check_security`, `optimize_code`, `generate_tests` |
| **Docs & Framework Specs** | `index_framework_specifications`, `search_framework_specifications`, `generate_documentation`, `get_documentation` |
| **Plugins & Analytics** | `modularize_code`, `extract_plugin`, `publish_plugin`, `recommend_plugins`, `log_token_usage`, `get_token_analytics` |
| **System & Health** | `project_health`, `summarize_project`, `configure_settings` |

---

## 📜 Business Source License 1.1 (BSL 1.1)

CodeMemoryPrime is licensed under the **Business Source License 1.1 (BSL 1.1)**.

- **Free for**: All personal, hobby, educational, open-source development, and commercial entities with **≤ 3 developers** AND **< $100,000 USD in annual revenue**.
- **Commercial Licensing**: Organizations exceeding the Additional Use Grant require a paid Commercial License.
- **Change License**: Converts automatically to **Apache 2.0** 3 years after each release.

For commercial licensing details, visit [https://codememoryprime.com/license](https://codememoryprime.com/license).

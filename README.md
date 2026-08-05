# CodeMemoryPrime (cmp)

```
+-----------------------------------------------------------------------+
|  ___  _   _  ___                                                      |
| / __|| | | || _ \  CodeMemoryPrime (cmp)                              |
| |(__ | _|_ ||  _/  A lightweight memory & codebase helper for AI tools|
| \___||_| |_||_|                                                       |
+-----------------------------------------------------------------------+
```

[![License: BSL-1.1](https://img.shields.io/badge/License-BSL_1.1-blue.svg)](LICENSE.md)
[![Language: Rust](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)

CodeMemoryPrime (CMP) is a small, fast tool that gives your AI coding assistant (like Claude Desktop, Cursor, or Windsurf) a long-term memory. 

If you've ever had an AI forget how your project works halfway through a chat, or waste your API tokens re-reading the same files over and over, this fixes that. It indexes your codebase locally so your AI assistant can instantly look up functions, remember architectural decisions, and check documentation without getting confused.

---

[ What makes it useful? ]

- Instant Re-indexing: When you save a file, it only re-indexes what changed (usually takes less than 0.2 seconds).
- Doesn't Lose Context: When it looks up a function, it keeps the surrounding class, struct, and import info so the AI actually understands how the code fits together.
- No Heavy Setup: It's a single compiled binary written in Rust. No installing Node packages, Python environments, or extra background services.
- Keeps Memory Between Sessions: Saves project facts, rules, and decisions so you don't have to re-explain your setup every time you open a new chat.
- Works Offline: Integrates natively with local Ollama models (like qwen2.5-coder and nomic-embed-text) or local endpoints like LM Studio.

---

[ Quick Start ]

1. Build from source:

   git clone https://github.com/rickieblevins/CodeMemoryPrime.git
   cd CodeMemoryPrime
   cargo build --release

   Your binary will be ready at ./target/release/cmp

2. Plug it into your AI assistant:

   Add CodeMemoryPrime to your client configuration file. 

   For Ollama (Free & Local):

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

   Where to find your config file:
   - Claude Desktop: ~/Library/Application Support/Claude/claude_desktop_config.json
   - Cursor: ~/.cursor/mcp.json
   - Windsurf: ~/.codeium/windsurf/mcp_config.json
   - Antigravity / Gemini CLI: ~/.gemini/mcp_config.json

---

[ Included Tools (35 Tools) ]

CodeMemoryPrime registers tools for your AI to use automatically:

- Code Search & Indexing: index_workspace, search_codebase, get_dependencies
- Long-Term Memory: save_interaction, search_memories, consolidate_memories
- File Management: read_file, write_file, patch_file, list_files
- Git & Safety Checkpoints: run_command, git, create_checkpoint, restore_checkpoint, list_checkpoints
- Refactoring & Diagnostics: explain_code, refactor_code, review_code, check_security, optimize_code, generate_tests, diagnose_compiler_error
- Docs & Framework RAG: index_framework_specifications, search_framework_specifications, generate_documentation, get_documentation
- Modularization & Analytics: modularize_code, extract_plugin, publish_plugin, recommend_plugins, log_token_usage, get_token_analytics
- Health & System: project_health, summarize_project, configure_settings

---

[ License ]

CodeMemoryPrime is licensed under the Business Source License 1.1 (BSL 1.1).

- Free to use for: Personal projects, hobbyists, students, open-source work, and small teams (3 or fewer developers, under $100k annual revenue).
- Commercial License: Required for larger teams or companies exceeding the free tier.
- Converts to Apache 2.0 (open source) automatically 3 years after release.

Questions or commercial licenses? Check out https://www.codememoryprime.com/

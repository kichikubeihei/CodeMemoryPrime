# Connecting CodeMemoryPrime to Your AI Tools

```
+-----------------------------------------------------------------------+
|  ___ _  _ ___                                                        |
| / __| || | _ \  CodeMemoryPrime (cmp)                                 |
| | (__| __ |  _/  Setup & Integration Guide                            |
| \___|_||_|_|                                                          |
+-----------------------------------------------------------------------+
```

Here is how to hook up CodeMemoryPrime to your favorite AI assistant or IDE editor.

---

[ Client Setup ]

1. Claude Desktop (macOS & Windows)

   Find your config file:
   - macOS: ~/Library/Application Support/Claude/claude_desktop_config.json
   - Windows: %APPDATA%\Claude\claude_desktop_config.json

   Add this block:

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

2. Cursor IDE

   1. Open Cursor Settings -> Features -> MCP Servers.
   2. Click + Add New MCP Server.
   3. Set Name to `CodeMemoryPrime`, Type to `command`, and Command path to `/path/to/target/release/cmp`.
   4. Save!

3. Windsurf IDE

   1. Open ~/.codeium/windsurf/mcp_config.json
   2. Paste the CodeMemoryPrime JSON config block shown above.
   3. Save and refresh.

4. Antigravity / Gemini CLI

   1. Open ~/.gemini/mcp_config.json
   2. Add the CodeMemoryPrime JSON config block.

---

[ Things to Try Asking Your AI ]

Once connected, try talking naturally to your AI:

- Indexing: "Index this folder under project name 'my_project'."
- Searching Code: "Find where we process user login tokens."
- Remembering Rules: "Save this rule into memory: We always use HSL colors for CSS."
- Diagnostics: "Diagnose this compiler error."
- Health: "Check CodeMemoryPrime project health."

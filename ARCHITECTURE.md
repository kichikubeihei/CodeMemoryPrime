# How CodeMemoryPrime Works (Architecture)

```
+-----------------------------------------------------------------------+
|  ___ _  _ ___                                                        |
| / __| || | _ \  CodeMemoryPrime (cmp)                                 |
| | (__| __ |  _/  Architecture & Internal Design                       |
| \___|_||_|_|                                                          |
+-----------------------------------------------------------------------+
```

Here is a simple look at how CodeMemoryPrime handles data under the hood:

```
[ AI Assistant ]  (Claude / Cursor / Windsurf / Antigravity)
       |
       |  JSON-RPC (stdio)
       v
[ Protocol Handler ]  (src/protocol/handlers.rs)
       |
       v
[ Tool Registry ]  (src/tools/ -- 32 Tools)
       |
       +-------------------+-------------------+
       |                   |                   |
       v                   v                   v
 [ Code Parser ]    [ Search Engine ]    [ Scraper & AST ]
 (src/parser.rs)    (src/search.rs)      (src/scraper.rs)
       |                   |                   |
       +-------------------+-------------------+
                           |
                           v
              [ SQLite Database (~/.cmp.db) ]
              - Function Chunks
              - Vector Embeddings
              - Full-Text Search (FTS5)
```

---

[ How Code Indexing Works ]

1. Smart Chunking instead of Random Lines
Most search engines chop code into arbitrary 50-line blocks, which cuts functions right down the middle and loses context. CMP parses the code structure (functions, methods, classes) so it knows where code blocks actually start and stop.

2. Keeping the Surrounding Context
When a function is indexed, CMP attaches top-level file imports and parent struct/class names to that chunk. So when the AI looks at a 10-line function, it still knows what libraries were imported and what class it belongs to.

3. Fast Re-indexing (SHA-256 Hashing)
Every function chunk gets a SHA-256 hash when saved. When you run a re-index, CMP compares the hash of each function. If a function hasn't changed, it reuses the existing vector embedding from SQLite. That's why re-indexing a project takes less than 0.2 seconds.

4. Hybrid Search (Vectors + Keywords)
When your AI asks a question, CMP runs two searches in parallel:
- Semantic Search: Uses vector math to find code with matching meanings.
- Keyword Search: Uses SQLite FTS5 to find exact variable or function names.

It then merges the two lists together so you get both exact code matches and conceptual matches.

---

[ Code Structure ]

- src/protocol/: Handles incoming and outgoing JSON-RPC messages over stdio.
- src/tools/: Contains the definitions and handlers for all 32 tools.
- src/parser.rs: Chunks code, tracks brace depth, and extracts imports.
- src/scraper.rs: Parses Markdown AST and HTML DOM for framework documentation.
- src/search.rs: Runs the hybrid vector and full-text search algorithms.
- src/llm.rs: Communicates with local Ollama or OpenAI-compatible endpoints.
- src/db.rs: Manages the local SQLite database (`~/.codememory_prime.db`).

# CodeMemoryPrime (CMP) Architecture

This document describes the internal design, concurrency model, AST parsing, and hybrid RAG search pipeline of **CodeMemoryPrime (CMP)**.

```
                    ┌─────────────────────────────────────────┐
                    │            MCP Client                   │
                    │ (Claude / Cursor / Windsurf / Gemini)   │
                    └────────────────────┬────────────────────┘
                                         │ JSON-RPC over stdio
                                         ▼
                    ┌─────────────────────────────────────────┐
                    │      src/protocol/handlers.rs           │
                    │   (initialize, ping, tools/list, call)   │
                    └────────────────────┬────────────────────┘
                                         │
                                         ▼
                    ┌─────────────────────────────────────────┐
                    │       src/tools/ (31 MCP Tools)         │
                    │  (codebase, memory, file_ops, shell...) │
                    └───────┬─────────────────────────┬───────┘
                            │                         │
                            ▼                         ▼
            ┌───────────────────────┐       ┌───────────────────────┐
            │    src/parser.rs      │       │     src/search.rs     │
            │ (Tree-sitter AST &    │       │ (Reciprocal Rank      │
            │  Parent-Child RAG)    │       │  Fusion Hybrid RAG)   │
            └───────────┬───────────┘       └───────────┬───────────┘
                        │                               │
                        └───────────────┬───────────────┘
                                        │
                                        ▼
                    ┌─────────────────────────────────────────┐
                    │         SQLite Database (~/.cmp.db)     │
                    │  (code_chunks, FTS5, vector BLOBs)      │
                    └─────────────────────────────────────────┘
```

---

## 1. Core Architectural Pillars

### A. Zero-Dependency Native Rust Binary
CodeMemoryPrime is built in pure Rust. Unlike Python or Node.js MCP servers that require heavy background daemons, virtual environments, or `npm` installations, CMP compiles to a single, self-contained binary that boots in under 1ms.

### B. AST Parent-Child Context Inheritance (`src/parser.rs`)
Traditional code RAG chunks files naively by line count, losing critical function signatures, top-level imports, and parent struct/class context.
CMP uses Tree-sitter AST parsing to extract function and method blocks while attaching:
1. Top-level file import headers.
2. Parent class/struct definitions and trait impl blocks.
3. Enclosing namespace context.

### C. Sub-Chunk SHA-256 Incremental Hashing
During `index_workspace`, CMP computes a SHA-256 hash for every parsed function chunk:
- If a function's code has not changed since the last index run, CMP reuses the existing vector embedding from SQLite.
- Re-indexing modified projects completes in **under 0.200 seconds** (a 95%+ performance boost).

### D. Hybrid RRF Search Engine (`src/search.rs`)
CMP uses **Reciprocal Rank Fusion (RRF)** to combine:
1. Semantic Cosine Similarity via local vector embeddings.
2. Exact keyword matching via SQLite FTS5 (Full-Text Search).
3. Dependency graph cross-referencing (`code_dependencies` table).

---

## 2. Component Breakdown

| Module | Location | Description |
|--------|----------|-------------|
| **Protocol** | `src/protocol/` | Handles MCP JSON-RPC 2.0 stdio stream, notifications, and tool dispatching. |
| **Tools Registry** | `src/tools/` | 31 modularized MCP tool implementations split across 8 domain files. |
| **Parser** | `src/parser.rs` | Tree-sitter AST parsing, import extraction, parent-child context. |
| **Search Engine** | `src/search.rs` | RRF hybrid vector + FTS5 retrieval engine. |
| **LLM Client** | `src/llm.rs` | Provider-agnostic client supporting Ollama and OpenAI-compatible APIs (LM Studio, vLLM). |
| **Licensing** | `src/license.rs` | Offline Ed25519 signature verification for BSL 1.1 commercial seat keys. |
| **Database** | `src/db.rs` | SQLite embedded database schema and FTS5 virtual table migrations. |

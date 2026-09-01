# CodeMemoryPrime Core (`mcp-coder-memory-rust`)

[![Language: Rust](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License: BSL-1.1](https://img.shields.io/badge/License-BSL_1.1-blue.svg)](LICENSE.md)
[![Model Context Protocol](https://img.shields.io/badge/MCP-2.0_Stateless-blueviolet)](https://modelcontextprotocol.io/)

The foundational high-performance Rust memory, graph, and AST intelligence engine powering **CodeMemoryPrime**, **CodeMemoryPrime-Pro**, **Lore Titan**, **RuleForge**, and **AIMACS**.

---

## 🌟 Core Architecture & Capabilities

```
+-----------------------------------------------------------------------------------+
|                        UNIVERSAL KNOWLEDGE & MEMORY CORE                         |
+-----------------------------------------------------------------------------------+
|  1. Sub-1ms Knowledge Graph  | SQLite Recursive CTEs, Flat Graph Edge Queries     |
|  2. Polymorphic Profiles     | Code, Lore (Novels/Bibles), TTRPG, Agent Memory    |
|  3. Solution & Failure Vault | SHA-256 HMAC Signatures & 60-Day Staleness TTL     |
|  4. Decentralized Mesh Sync  | Cross-Machine Delta Packaging (`export`/`import`)  |
|  5. Anti-Monolith Invariants | Strict LOC Audits (≤ 200 UI / ≤ 250 Logic)         |
+-----------------------------------------------------------------------------------+
```

### 1. Sub-Millisecond Knowledge Graph (`knowledge_graph.rs`)
* **Recursive CTE Graph Traversals**: Traverses complex entity subgraphs up to arbitrary depths in sub-1ms using indexed SQLite queries.
* **Flat Edge Queries (`query_all_edges_flat`)**: Streams nodes and weighted edges directly to frontend state for **60 FPS visualizers** in Svelte 5 and Canvas/WebGL.

### 2. Polymorphic Domain Profiles (`profiles/`)
* Seamlessly converts raw nodes into typed schemas:
  * `DomainProfile::Code` — AST symbol relationships, imports, and caller graphs.
  * `DomainProfile::Lore` — Characters, factions, locations, timeline events, and canon rules.
  * `DomainProfile::Ttrpg` — Systems, mechanics, character stats, and item matrices.
  * `DomainProfile::Agent` — Long-term agent goals, user preferences, and interaction histories.

### 3. Solution Vault & Failure Vault
* **Solution Vault**: Cryptographically stores verified algorithms with objective test pass rates and compiler exit codes ($0$).
* **Failure Vault**: Intercepts known historical dead-ends and regressions before an agent attempts a flawed implementation.

### 4. Decentralized Multi-Device Mesh Sync (`mesh_sync.rs`)
* Exports and non-destructively merges cryptographically signed delta packages (`MemoryDeltaPackage`) across **Laptop**, **Workstation**, **Home PC**, and **Mobile Devices**.

---

## 🛠️ Module Overview

| Module | Purpose |
| :--- | :--- |
| `src/knowledge_graph.rs` | Recursive CTE subgraph traversals & flat edge queries for visualizers. |
| `src/profiles/mod.rs` | Polymorphic entity converters (`LoreEntity`, `TtrpgEntity`, `AgentEntity`). |
| `src/mesh_sync.rs` | Cross-device memory export, HMAC signing, and conflict-free SQLite imports. |
| `src/solution_vault.rs` | Solution storage with objective metrics and staleness decay scores. |
| `src/failure_vault.rs` | Dead-end recording and proactive dead-end query interception. |
| `src/handoff.rs` | Cross-session handoff persistence and AI diary storage. |
| `src/tailscale_roster.rs` | Real-time query and synchronization of Tailscale GPU model rosters. |
| `src/adversarial_test.rs`| Automated generation of adversarial test harnesses. |

---

## 🧪 Testing

Run the full Rust test suite across all modules:
```bash
cargo test
```

---

## 📜 License

Licensed under the **Business Source License 1.1 (BSL 1.1)**.
Free for personal use, hobbyists, students, open-source work, and small development teams ($\le 3$ developers AND $< \$100,000$ annual revenue). Converts to **Apache 2.0** after 3 years.

(C) 2026 CodeMemoryPrime / Rickie Blevins.

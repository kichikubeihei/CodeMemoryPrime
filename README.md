# CodeMemoryPrime Core (`codememory-prime`)

[![Language: Rust](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License: BSL-1.1](https://img.shields.io/badge/License-BSL_1.1-blue.svg)](LICENSE.md)
[![Model Context Protocol](https://img.shields.io/badge/MCP-2.0_Stateless-blueviolet)](https://modelcontextprotocol.io/)
[![One-Command Install](https://img.shields.io/badge/Install-npx_codememory--prime_install-brightgreen)](https://www.npmjs.com/package/codememory-prime)

**CodeMemoryPrime Core** is a bare-metal Rust memory, graph, and AST intelligence engine for AI coding assistants. It equips agents in Google Antigravity, Claude Desktop, Cursor, and Windsurf with **sub-millisecond codebase indexing, cross-session architectural persistence, proactive regression prevention, and decentralized multi-device synchronization**.

---

## ⚡ 1-Command Universal Installation

Install and configure CodeMemoryPrime across all your AI development environments:

```bash
# Automatically detects and configures Claude Desktop, Cursor, and Windsurf
npx codememory-prime install --all
```

Or via direct shell installer:
```bash
curl -fsSL https://raw.githubusercontent.com/CodeMemoryPrime/CodeMemoryPrime-Pro/main/scripts/install.sh | bash
```

To verify status at any time:
```bash
npx codememory-prime status
```

---

## 🚀 Core Features & Capabilities

```
+-----------------------------------------------------------------------------------+
|                        UNIVERSAL KNOWLEDGE & MEMORY CORE                         |
+-----------------------------------------------------------------------------------+
|  1. Sub-1ms Knowledge Graph  | SQLite Recursive CTEs, Flat Graph Edge Queries     |
|  2. Polymorphic Profiles     | Code (AST), Algorithms, Architecture, Agent Memory |
|  3. Solution & Failure Vault | SHA-256 HMAC Signatures & 60-Day Staleness TTL     |
|  4. Decentralized Mesh Sync  | Cross-Machine Delta Packaging (Zero Cloud Fees)    |
|  5. Pure Bare-Metal Rust     | Self-contained native binary, bundled SQLite       |
+-----------------------------------------------------------------------------------+
```

### 1. Sub-Millisecond AST Knowledge Graph
* **Indexed SQLite CTEs**: Executes complex entity and symbol traversals in under 1 millisecond.
* **Caller & Callee Hierarchies**: Maps function and type relationships across projects so the AI understands structural dependencies without reading whole files.
* **Streamlined Context**: Drastically reduces prompt token consumption compared to flat-file dumps or vector chunking.

### 2. Solution Vault & Failure Vault
* **Solution Vault**: Stores verified algorithms and implementation patterns alongside test pass rates and compiler exit codes ($0$).
* **Failure Vault**: Records regression causes and dead-ends to prevent AI agents from getting stuck in repetitive debugging loops.

### 3. Cross-Session AI Diary & Handoffs
* Automatically saves work-in-progress state, verified steps, and next goals.
* Seamlessly loads context when opening a new chat or switching branches.

### 4. Decentralized Multi-Device Mesh Sync
* Exports and merges cryptographically signed delta packages (`MemoryDeltaPackage`) between your laptop, workstation, and remote environments.
* Requires zero cloud subscriptions or third-party database hosting.

### 5. Pure Bare-Metal Rust
* Single native binary with bundled SQLite.
* Zero Python, Docker, or external database daemon dependencies.

---

## 📊 Core vs. Pro Edition

| Feature | Core (Public) | Pro Edition |
| :--- | :---: | :---: |
| Sub-Millisecond AST Code Indexing | ✅ | ✅ |
| Solution & Failure Memory Vaults | ✅ | ✅ |
| Cross-Session AI Diary Handoffs | ✅ | ✅ |
| Multi-Device Mesh Synchronization | ✅ | ✅ |
| **Compound Workflows (`preflight`, `gauntlet`, `handoff`)** | ❌ | ✅ |
| **Sovereign Task Ledger & Headless `cmp-worker` Daemon** | ❌ | ✅ |
| **Anti-Monolith LOC Auditing (≤200 UI / ≤250 Logic)** | ❌ | ✅ |
| **DevSecOps Supply Chain & Secret Scanning** | ❌ | ✅ |
| **Automated Adversarial Test Harness Generation** | ❌ | ✅ |
| **$0.00 Local GPU Cluster / Tailscale Offloading** | ❌ | ✅ |

*Learn more about CodeMemoryPrime-Pro in the [Pro Repository](https://github.com/CodeMemoryPrime/CodeMemoryPrime-Pro).*

---

## 🛠️ Module Architecture

| Module | Purpose |
| :--- | :--- |
| `src/knowledge_graph.rs` | Recursive CTE subgraph traversals & flat edge queries for visualizers. |
| `src/profiles/mod.rs` | Polymorphic entity converters (`CodeEntity`, `AlgorithmEntity`, `ArchitectureEntity`, `AgentEntity`). |
| `src/solution_vault.rs` | Solution storage with objective pass metrics and staleness decay scores. |
| `src/failure_vault.rs` | Dead-end recording and proactive bug regression prevention. |
| `src/mesh_sync.rs` | Cross-device memory export, HMAC signing, and conflict-free SQLite imports. |
| `src/handoff.rs` | Cross-session handoff persistence and AI diary storage. |
| `src/tailscale_roster.rs` | Query and synchronization of local GPU model rosters over Tailscale. |
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
Free for personal use, hobbyists, students, open-source work, and small development teams ($\le 3$ developers AND $< \$100,000$ annual revenue). Converts automatically to **Apache 2.0** after 3 years.

(C) 2026 CodeMemoryPrime Contributors.

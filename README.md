# CodeMemoryPrime Core (`codememory-prime`)

[![Language: Rust](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License: BSL-1.1](https://img.shields.io/badge/License-BSL_1.1-blue.svg)](LICENSE.md)
[![Model Context Protocol](https://img.shields.io/badge/MCP-2.0_Stateless-blueviolet)](https://modelcontextprotocol.io/)
[![Zero Cloud Dependencies](https://img.shields.io/badge/SQLite-Bundled_FTS5_%2B_AST-success)](https://github.com/kichikubeihei/CodeMemoryPrime)

**CodeMemoryPrime Core (`cmp`)** is a bare-metal, high-performance Rust memory, graph, and AST intelligence engine for AI coding assistants. It equips agents across **Claude Desktop, Cursor, Windsurf, Claude Code CLI, VS Code (Roo Code / Cline), and Google Antigravity** with **persistent architectural memory, sub-millisecond AST symbol discovery, proactive dead-end interception, and 70–85% token-quota reduction**.

---

## ⚡ 1-Command Universal Installation

Install the native `cmp` binary and automatically wire up all detected AI coding assistants (**Claude Desktop, Cursor, Windsurf, Claude Code CLI, VS Code, and Google Antigravity**) in one command:

```bash
curl -fsSL https://raw.githubusercontent.com/kichikubeihei/CodeMemoryPrime/main/scripts/install.sh | bash
```

Or install directly via **Cargo** and run the built-in IDE auto-installer:

```bash
cargo install --git https://github.com/kichikubeihei/CodeMemoryPrime.git --bin cmp --force && cmp install --all
```

> [!TIP]
> The installer compiles or downloads the self-contained native binary (with bundled SQLite FTS5—zero external runtime dependencies), initializes `~/.codememory_prime.db`, and safely merges the `"CodeMemoryPrime"` MCP configuration into all detected editors with automatic `.bak` backups.

Verify your installation and system health at any time:

```bash
# Inspect binary path, SQLite DB status, Grimoire contracts, and configured IDEs
cmp status

# Run full preflight audit (SQLite integrity, Merkle root verification, local/Tailscale GPU check)
cmp preflight
```

---

## 🔥 Stop Burning Your AI Coding Quota: Slash Context by 70–85%

If you use Claude Sonnet/Opus, Cursor Pro, or Gemini Pro, you have likely experienced the frustration of **running out of your weekly or five-hour token quota after just a few coding sessions**.

Most AI coding setups burn through hundreds of thousands of invisible tokens due to architectural inefficiencies. CodeMemoryPrime was engineered from the ground up to stop this quota drain through **4 core mechanisms**:

```
+-------------------------------------------------------------------------------------------------+
|                                 HOW CODEMEMORYPRIME SAVES YOUR QUOTA                            |
+-------------------------------------------------------------------------------------------------+
|  1. 4-Pillar Schema Consolidation | Drops MCP tool schema tax from 15,000 to ~1,500 tokens/turn |
|  2. AST Symbol Graph Navigation   | Sub-10ms symbol queries replace 500-line raw file dumps     |
|  3. Turn-1 Persistent Handoffs    | 400-token session briefing replaces re-explaining chat logs |
|  4. Bounded Output & Anti-Slop    | Auto-trims compiler noise & strips conversational filler    |
+-------------------------------------------------------------------------------------------------+
```

### 1. 4-Pillar Schema Consolidation (Saves ~13,500 tokens *per turn*)
* **The Problem**: Standard MCP servers register 30 to 50 individual tools. Because LLMs must inject the full JSON schema of every available tool into *every single turn* of the prompt, a 40-tool server consumes 12,000–18,000 tokens before you even type a message! Over 10 turns, that is 150,000+ tokens wasted purely on tool declarations.
* **The Fix**: CodeMemoryPrime consolidates capabilities into **4 dense, unified pillars** (`cmp_search`, `cmp_architect`, `cmp_audit`, and `cmp_system`) alongside focused high-leverage tools. MCP schema tax drops by up to **90% on every single interaction turn**.

### 2. AST Symbol Graph vs. Raw File Dumps (Saves 4,000+ tokens per lookup)
* **The Problem**: Agents typically read entire 400–800 line files into the prompt just to understand one struct definition, helper function, or import list.
* **The Fix**: `cmp_search` performs incremental git-diff re-indexing and uses SQLite Recursive CTEs + AST chunking (including dedicated Svelte 5 / SFC component segmentation). It returns the exact symbol, line numbers, callers, and callees in ~200–350 tokens instead of dumping thousands of lines of raw code into your context window.

### 3. Turn-1 Session Handoffs vs. Re-explaining Chat History (Saves 8,000+ tokens per restart)
* **The Problem**: Starting a new chat or returning to work the next day forces developers to paste previous chat logs or write lengthy paragraphs explaining what was already built and what needs to happen next.
* **The Fix**: At the end of a session, call `cmp_system(action="save_handoff")`. When opening a new chat or switching branches, `cmp_system(action="load_handoff")` instantly injects a structured 400-token briefing containing completed steps, active files, open questions, and `prohibited_repetition` guardrails.

### 4. Bounded Terminal Output & Anti-Slop Sanitization (Saves 5,000+ tokens per test run)
* **The Problem**: Running `cargo test`, `npm test`, or `git log` can easily dump 10,000+ tokens of terminal noise into the prompt context, blowing through rate limits in seconds.
* **The Fix**: CMP's **Rust Token Killer** (`execute_and_bound_cmd`, `bound_shell_output`, and `cmp_system(action="trim_output")`) strips ANSI escape codes and bounds stdout/stderr to intelligent head/tail slices while preserving compiler error tracebacks.

---

## 📊 Token Consumption Benchmark

| Interaction Type | Traditional AI Coding | With CodeMemoryPrime | Savings |
| :--- | :---: | :---: | :---: |
| **MCP Tool Schema Overhead (Per Turn)** | ~15,000 tokens | **~1,500 tokens** | **90.0%** |
| **Code Search & Symbol Inspection** | ~4,500 tokens *(full file)* | **~350 tokens** *(AST slice)* | **92.2%** |
| **Session Continuation / State Load** | ~8,000 tokens *(chat paste)* | **~400 tokens** *(handoff)* | **95.0%** |
| **Terminal / Test Output Dumps** | ~6,000 tokens *(raw log)* | **~1,200 tokens** *(bounded)* | **80.0%** |
| **Typical 20-Turn Coding Session** | **~350,000 tokens** | **~65,000 tokens** | **🔥 81.4% Net Savings** |

---

## 🏛️ The 4 Consolidated Pillars & Capabilities

### 1. `cmp_search` — Codebase Navigation, Complexity Radar & Web Ingestion
* **Sub-10ms Hybrid RRF AST Search (`action="search"`, `action="find_by_file"`)**: Combines SQLite FTS5 multi-term tokenization, AST symbol extraction (Rust, TypeScript/JS, Python, Go, Svelte 5 SFCs), and vector similarity with automatic git-diff incremental re-indexing.
* **Proactive Dead-End Interception**: Automatically queries the Failure Vault during every code search and warns the agent if a matching approach previously caused crashes or regressions.
* **Architectural Complexity Radar (`action="complexity_radar"`, `action="file_metrics"`)**: Identifies high-urgency refactoring hotspots by tracking nesting depth, function count, afferent/efferent coupling, instability index, token cost, and code smells (e.g., `ASYNC_WATERFALL_IN_LOOP`).
* **Workspace Route Tracer (`action="trace_routes"`)**: Automatically maps HTTP/API endpoints and frontend routes across your repository.
* **1-Step Web & Documentation Ingestion (`url_to_ingest`)**: Fetches, cleans, and indexes external documentation URLs directly into local SQLite project memory.

### 2. `cmp_architect` — Solution Vault, Failure Vault, UI Studio & Safe Refactoring
* **Solution Vault (`vault_solution`, `query_solution_vault`)**: Stores verified algorithms and implementation patterns alongside compiler exit codes (`0`) and test pass rates, protected by **SHA-256 HMAC anti-tamper signatures** and 60-day staleness decay.
* **Failure Vault (`record_failure`, `query_dead_ends`)**: Records regression causes, stack traces, severity (`FATAL_CRASH`, `SECURITY_EXPLOIT`, `PARADIGM_LIMITATION`, `PERFORMANCE_WALL`), and suggested alternatives so agents never repeat dead-ends.
* **Open-Source Grimoire Algorithmica (`query_grimoire`, `get_grimoire_contract`)**: Instant semantic lookup into peer-reviewed, contract-verified algorithms across Python, TypeScript, Go, Dart, and Rust.
* **Anti-Monolith & Blast Radius Evaluator (`evaluate_refactor`)**: Scans workspaces for oversized monoliths and calculates failure-aware blast radius before refactoring.
* **Architectural Slop Interceptor (`intercept_architectural_slop`)**: Static gate that rejects swallowed exceptions (`catch {}`, `except: pass`, `if let Err(_) = ... {}`), leftover `todo!()` / `unimplemented!()` macros, unresolved Promise mocks, and tight-loop heap allocations (`.clone()` in loops).
* **UI Design Studio & A11y / Zero-CLS Auditor (`audit_ui_a11y`, `store_ui_component`, `query_ui_components`, `store_design_pattern`, `query_design_patterns`)**: Audits Svelte/React/HTML components for missing ARIA attributes and Cumulative Layout Shift (CLS) risks, and vaults reusable typed UI components.
* **AdaptivePaste Domain Projection (`adaptive_paste`)**: Automatically translates generic vaulted algorithms into your project's domain vocabulary.
* **Spec Alignment & SemVer Advisor (`align_spec`, `lock_decisions`, `semver_advice`, `apply_semver`)**: Locks Architectural Decision Records (ADRs) and computes automated Semantic Versioning bumps.

### 3. `cmp_audit` — DevSecOps, Cryptographic Integrity & Quality Contracts
* **Cryptographic Memory Integrity (`full_repo_audit`)**: Verifies HMAC-SHA256 signatures across stored memories and computes a **Merkle Tree branch root** to detect database tampering.
* **DevSecOps Secret & RCE Scanner (`security_check`, `audit_secrets`)**: Scans code for exposed API keys (Anthropic, OpenAI, Google/Gemini, AWS, GitHub PATs, private keys) and dangerous RCE patterns (`curl | bash`, `rm -rf /`, `eval()`, unsandboxed `exec()`).
* **Dynamic Task Ledger Gates (`lock_task_gates`, `verify_task_gate`, `get_task_ledger`)**: Locks objective compiler, linter, and test commands that must return exit code `0` before a task is marked complete.
* **Grimoire Gauntlet & Harvesting (`run_grimoire_gauntlet`, `harvest_winning_algorithm`)**: Runs candidate algorithms through property tests, 5 Anti-Cheat Sentinels, and multi-scale asymptotic benchmarks.
* **Adversarial Test Generation (`generate_adversarial_tests`)**: Generates boundary-condition test harnesses (empty states, max payloads, overflow, concurrency, null safety).
* **Proactive Repo Solution Miner (`mine_repo_solutions`)**: Scans passing repositories to extract verified functions into the Solution Vault.
* **Context Hygiene (`clean_stale_memory`, `prune_slop`)**: Purges orphaned AST chunks when files are deleted/renamed and strips conversational AI filler.

### 4. `cmp_system` — Cross-Session Intelligence, Research Vault & Model Routing
* **Turn-1 Session Handoffs (`save_handoff`, `load_handoff`)**: Persists structured cross-session handoffs with epistemic evidence tiers and `prohibited_repetition` lists.
* **Session Observations & Timeline (`record_observation`, `search_observations`, `get_session_timeline`)**: Granular chronological event ledger tracking tool executions and file modifications across sessions.
* **Cryptographic Research Vault (`record_research`, `query_research`)**: Indexes technical papers, YouTube architecture breakdowns, and actionable upgrade blueprints across all projects.
* **Quota-Aware Model Router (`recommend_model`)**: Recommends the optimal model tier ($0.00 local/Tailscale Ollama GPU vs. Cloud Flash/Sonnet/Pro) based on task type and payload size.
* **Tailscale GPU Roster & Grimoire Sync (`sync_memory`, `sync_grimoire_collective`)**: Auto-discovers live Ollama models on your local or Tailscale GPU node and exports signed memory deltas.
* **Standardized Workflow Guides (`get_workflow_guide`)**: Built-in step-by-step blueprints for `feature_delivery`, `arena_tournament`, `safe_refactor`, `ui_design_critic`, and `component_vault`.

### 🛠️ Standalone High-Leverage & Orchestrator Tools
In addition to the 4 pillars, `cmp` exposes direct high-leverage helpers for zero-friction agent execution:
* `cmp_preflight`: Full system health check (license tier, SQLite DB size, local/Tailscale Ollama connectivity, and Merkle root integrity).
* `execute_and_bound_cmd` & `bound_shell_output`: Executes shell commands (`cargo test`, `npm test`) and bounds stdout/stderr in Rust before returning to the LLM.
* `sync_memory`, `record_research`, `query_research`: Direct shortcuts to the synchronization and research engines.
* **6 Specialist Orchestrator Aliases**: `orchestrate_code_search`, `orchestrate_refactor_and_fix`, `orchestrate_security_audit`, `orchestrate_memory_and_context`, `orchestrate_spec_and_test`, and `orchestrate_full_repo_audit`.

---

## 📜 Open-Source Grimoire Algorithmica: Contract-Driven High-Level Algorithms

The **Grimoire Algorithmica** is an open-source, contract-verified collective repository of optimal algorithms for high-level languages (**Python, TypeScript, Go, Dart, Rust**). Instead of paying cloud LLMs to repeatedly write boilerplates from scratch, agents pull verified, benchmarked algorithms from local SQLite memory in **<2ms**.

```
+---------------------------------------------------------------------------------------------------+
|                            THE GRIMOIRE GAUNTLET HARVESTING PIPELINE                              |
+---------------------------------------------------------------------------------------------------+
|  [Agent Submission] -> [5 Anti-Cheat Sentinels] -> [Air-Gapped Sandbox] -> [SQLite Collective DB] |
|   (Untrusted Code)      (AST & Asymptotic Curve)   (MicroVM / Container)    (Signed HMAC Delta)   |
+---------------------------------------------------------------------------------------------------+
```

### 🎯 10 Built-In High-Level Algorithmic Contracts
1. **`spatial_kd_tree`** ($K$-Dimensional spatial partitioning, nearest neighbor, range radius)
2. **`consistent_hashing`** (Virtual node ring, zero-rebalance key distribution, Ketama-style)
3. **`bloom_filter`** (Zero-false-negative probabilistic set membership, murmur3/siphash)
4. **`lru_cache`** ($O(1)$ Doubly-linked hash map with deterministic capacity eviction)
5. **`rate_limiter_token_bucket`** (High-throughput token bucket with sub-millisecond replenishment)
6. **`aho_corasick`** (Multi-pattern dictionary substring matching with failure transitions)
7. **`tarjan_scc`** (Strongly Connected Components in directed graphs via DFS low-link)
8. **`suffix_automaton`** (Linear-time directed acyclic word graph for substring queries)
9. **`hyperloglog`** (Distinct count cardinality estimation with 64-bit harmonic mean)
10. **`dijkstra_indexed_heap`** (Shortest path using indexed binary min-heap with decrease-key)

### 🛡️ 5 Anti-Cheat Sentinels & The 3-Strikes Ban System
To protect the collective from LLM gaming, hallucinated shortcuts, and malicious exploits, submissions must survive **5 verification sentinels**:
* **AST Security Firewall**: Blocks banned namespaces and dangerous primitives (`os`, `sys`, `subprocess`, `socket`, `child_process`, `fs`, `eval`, `exec`).
* **Reflection & Environment Tampering Interception**: Detects `getattr()`, `__subclasses__()`, `globals()`, and dynamic scope introspection.
* **Assertion & Test Runner Anti-Monkey-Patching**: Blocks overriding test frameworks (`unittest.TestCase.assert...`, `jest.fn`).
* **Static Hardcoding & Lookup Table Detection**: Intercepts lookup tables masquerading as computation.
* **Multi-Scale Asymptotic Curve Fitting ($N=10 \to 100,000$)**: Executes inputs across increasing magnitudes; submissions claiming $O(1)$ or $O(N \log N)$ that scale at $O(N^2)$ are disqualified.
* **3-Strikes Quarantine**: Honest bugs receive 0 strikes; deliberate AST evasion or test tampering incurs +1 strike, with permanent dual-fingerprint quarantine (**Client IP + Ed25519 Public Key**) at Strike 3.

---

## ⚙️ Multi-Client Setup Guide

### Automatic Setup (Recommended)
Once `cmp` is installed, run:
```bash
cmp install --all
```
This automatically detects and configures **Claude Desktop**, **Claude Code CLI**, **Cursor**, **Windsurf**, **VS Code (Roo Code / Cline)**, and **Google Antigravity**.

### Manual Client Configuration
If you prefer to edit your MCP configuration files manually:

#### Claude Desktop (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS or `%APPDATA%\Claude\claude_desktop_config.json` on Windows)
```json
{
  "mcpServers": {
    "CodeMemoryPrime": {
      "command": "cmp",
      "args": []
    }
  }
}
```

#### Cursor (`~/.cursor/mcp.json`) & Windsurf (`~/.codeium/windsurf/mcp_config.json`)
```json
{
  "mcpServers": {
    "CodeMemoryPrime": {
      "command": "cmp",
      "args": []
    }
  }
}
```

#### Google Antigravity (`~/.gemini/antigravity/mcp_config.json`)
```json
{
  "mcpServers": {
    "CodeMemoryPrime": {
      "command": "cmp",
      "args": []
    }
  }
}
```

#### Claude Code CLI
```bash
claude mcp add CodeMemoryPrime -- cmp
```

---

## 💻 CLI Reference

```bash
cmp                      # Start MCP stdio JSON-RPC server (default)
cmp serve                # Start MCP stdio JSON-RPC server (explicit alias)
cmp install [--all]      # Auto-detect and configure Claude, Cursor, Windsurf, VS Code & Antigravity
cmp install --dry-run    # Preview IDE configuration changes without writing files
cmp status               # Inspect binary path, SQLite DB, Grimoire contracts & IDE wiring
cmp preflight            # Run full system preflight & cryptographic Merkle integrity audit
cmp sync-memory [URL]    # Sync local/Tailscale Ollama GPU model roster & export delta snapshot
cmp research [KEYWORD]   # Query recorded research papers, video analyses & architecture upgrades
cmp --help               # Show CLI usage summary
```

---

## 🛠️ Building from Source

CodeMemoryPrime is written in pure Rust with bundled SQLite (`rusqlite` bundled feature) and requires zero external system libraries:

```bash
# 1. Clone repository
git clone https://github.com/kichikubeihei/CodeMemoryPrime.git
cd CodeMemoryPrime

# 2. Build ultra-optimized release binary (ThinLTO, abort panic, stripped symbols)
cargo build --release

# 3. Wire into all detected AI editors
./target/release/cmp install --all

# 4. Run the test suite
cargo test
```

---

## 📊 CodeMemoryPrime (Core) vs. CodeMemoryPrime-Pro

| Capability | CodeMemoryPrime (Core) | CodeMemoryPrime-Pro |
| :--- | :---: | :---: |
| **4-Pillar Consolidated MCP Architecture (`cmp_search`, `cmp_architect`, `cmp_audit`, `cmp_system`)** | ✅ | ✅ |
| **Sub-10ms AST Knowledge Graph, Hybrid RRF Search & Git-Diff Incremental Indexing** | ✅ | ✅ |
| **Architectural Complexity Radar (`complexity_radar`, `file_metrics`) & Route Tracer** | ✅ | ✅ |
| **Solution & Failure Memory Vaults (SHA-256 HMAC + Merkle Integrity)** | ✅ | ✅ |
| **Open-Source Grimoire Algorithmica (10 Contracts + 5 Anti-Cheat Sentinels)** | ✅ | ✅ |
| **Turn-1 Session Handoffs, Observation Timeline & Research Vault** | ✅ | ✅ |
| **Rust Token Killer (`execute_and_bound_cmd`, `bound_shell_output`) & Anti-Slop Pruning** | ✅ | ✅ |
| **UI Design Studio, A11y / Zero-CLS Auditor & Adversarial Test Generation** | ✅ | ✅ |
| **1-Command Universal IDE Auto-Installer (`cmp install --all` & `scripts/install.sh`)** | ✅ | ✅ |
| **Tailscale $0.00 Ollama GPU Roster Discovery (`sync_memory`)** | ✅ | ✅ |
| **Accelerated CSA-Doc Shorthand Pipeline & Batch Video/URL Distillation (`cmp-ingest`)** | ❌ | ✅ |
| **Headless Autonomous Worktree Execution Daemon (`cmp-worker`) & Firstmate Worktrees** | ❌ | ✅ |
| **Spynel Real-Time Web Mission Control UI (`cmp-control` on Port 7799)** | ❌ | ✅ |
| **Cross-Terminal Multi-Agent Swarm Event Bus & Automated Cloudflare R2 / Mesh Daemon** | ❌ | ✅ |
| **Low-Level C-ABI (`grimoire.h`), BVH/SPSC SIMD Headers & Dynamic Exploit Verification** | ❌ | ✅ |

---

## 📜 License

Licensed under the **Business Source License 1.1 (BSL 1.1)**.
* **100% Free** for personal use, hobbyists, students, open-source work, and small development teams ($\le 3$ developers AND $< \$100,000$ annual revenue).
* Converts automatically to **Apache 2.0** after 3 years.

(C) 2026 CodeMemoryPrime Contributors.

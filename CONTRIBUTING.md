# Contributing to CodeMemoryPrime (CMP)

Thank you for your interest in contributing to **CodeMemoryPrime (CMP)**! We welcome bug fixes, documentation improvements, new MCP tools, and performance optimizations.

---

## 🛠️ Development Setup

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (1.75 or newer)
- [Ollama](https://ollama.com) (or an OpenAI-compatible local endpoint like LM Studio)

### Building & Testing Locally

1. **Clone the repository**:
   ```bash
   git clone https://github.com/your-username/CodeMemoryPrime.git
   cd CodeMemoryPrime
   ```

2. **Run tests**:
   ```bash
   cargo test
   ```

3. **Build release binary**:
   ```bash
   cargo build --release
   ```

---

## 🧩 Adding a New MCP Tool

To add a new tool to **CodeMemoryPrime**:
1. Open `src/tools/` and choose the appropriate domain file (e.g., `codebase.rs`, `file_ops.rs`, `refactor.rs`).
2. Add your JSON schema definition to `list_schemas()`.
3. Add your execution logic to `handle_call()`.
4. Compile and verify with `cargo build --release`.

---

## 📜 Licensing Guidelines

CodeMemoryPrime is licensed under the **Business Source License 1.1 (BSL 1.1)**. All contributions submitted to this repository will be covered by the same license terms.

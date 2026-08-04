# Contributing to CodeMemoryPrime (cmp)

```
+-----------------------------------------------------------------------+
|  ___ _  _ ___                                                        |
| / __| || | _ \  CodeMemoryPrime (cmp)                                 |
| | (__| __ |  _/  Contributing Guide                                   |
| \___|_||_|_|                                                          |
+-----------------------------------------------------------------------+
```

First off, thanks for wanting to help out! Whether you're fixing a typo, adding a new MCP tool, or speeding up something under the hood, all contributions are appreciated.

---

[ What You Need ]

- Rust 1.75 or newer installed on your machine.
- Ollama (or any local LLM provider like LM Studio) running locally if you want to test AI tools.

---

[ How to Build & Run Tests ]

1. Grab the code:
   git clone https://github.com/rickieblevins/CodeMemoryPrime.git
   cd CodeMemoryPrime

2. Run the test suite:
   cargo test

3. Build the binary:
   cargo build --release

The binary will build at ./target/release/cmp

---

[ Want to add a new MCP Tool? ]

Adding tools is pretty straightforward:

1. Head over to `src/tools/` and pick the right domain file (like `codebase.rs`, `refactor.rs`, or `file_ops.rs`).
2. Add your tool's JSON schema definition inside `list_schemas()`.
3. Add your tool's logic inside `handle_call()`.
4. Run `cargo test` to make sure everything compiles cleanly!

---

[ License Note ]

CodeMemoryPrime uses the Business Source License 1.1 (BSL 1.1). Any contributions you submit will fall under the same license.

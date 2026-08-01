# Security Policy

## 🔒 Privacy & Local Security Guarantees

CodeMemoryPrime (CMP) is designed from the ground up to be **local-first and privacy-focused**:
- **Zero Cloud Data Telemetry**: Your codebase indices, SQLite memory databases (`~/.codememory_prime.db`), and embeddings stay 100% on your local machine.
- **Local AI Inference**: When using Ollama or LM Studio, no code snippets or queries leave your local network.

---

## 🛡️ Safe Command Allowlisting

The `run_command` MCP tool enforces an explicit executable allowlist to prevent arbitrary shell command injection:

**Allowed Executable Prefixes**:
`cargo`, `npm`, `npx`, `git`, `python`, `python3`, `pytest`, `ls`, `pwd`, `cmp`, `agy`.

---

## 📬 Reporting Vulnerabilities

If you discover a security flaw or vulnerability in CodeMemoryPrime:
1. Please do **NOT** open a public GitHub issue.
2. Email your findings directly to `security@codememoryprime.com` or contact the maintainers privately.
3. We will acknowledge receipt within 48 hours and work with you to patch the issue promptly.

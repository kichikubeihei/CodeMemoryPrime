# Security Policy

```
+-----------------------------------------------------------------------+
|  ___ _  _ ___                                                        |
| / __| || | _ \  CodeMemoryPrime (cmp)                                 |
| | (__| __ |  _/  Security & Privacy Policy                            |
| \___|_||_|_|                                                          |
+-----------------------------------------------------------------------+
```

We take privacy and security seriously. Nobody wants their private codebase or API keys floating around in random cloud logs.

---

[ Local-First & Privacy ]

- Your Data Stays on Your Machine: Your code indices, SQLite memory database (`~/.codememory_prime.db`), and embeddings are stored 100% locally.
- No Telemetry: We don't track your queries, your project names, or your code.
- Local AI Support: When paired with Ollama or LM Studio, your code never leaves your local network.

---

[ Command Guardrails ]

To keep your system safe while letting the AI run necessary builds, the `run_command` tool restricts commands to an approved list of common developer utilities:

Allowed command prefixes:
`cargo`, `npm`, `npx`, `git`, `python`, `python3`, `pytest`, `ls`, `pwd`, `cmp`, `agy`.

---

[ Found a Bug or Vulnerability? ]

If you spot a security issue or vulnerability in CodeMemoryPrime:

1. Please don't post it in a public GitHub issue right away.
2. Send an email directly to `license@rickieblevins.com` so we can look into it.
3. We'll get back to you within 48 hours and get a fix out as fast as possible.

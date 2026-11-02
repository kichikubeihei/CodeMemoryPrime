# CodeMemoryPrime & CMP-Pro — The Complete Beginner's Step-by-Step Setup Guide

> **Welcome!** Whether you are installing the free **Open-Source Core (`CodeMemoryPrime`)** or the full autonomous **Pro Edition (`CodeMemoryPrime-Pro`)**, this guide walks you through everything step-by-step on **macOS**, **Windows**, and **Linux**—even if you have never configured an MCP server or local AI model before.

---

## 💡 What is CodeMemoryPrime? (In Plain English)

Think of your AI coding assistant (Claude Desktop, Cursor, Windsurf, VS Code, or Antigravity) as a brilliant developer with short-term amnesia. Every time you start a new chat, it forgets your project's architecture, re-reads hundreds of lines of code from scratch, and sometimes repeats the exact same bugs you fixed yesterday.

**CodeMemoryPrime** is a **sub-millisecond local memory bank & verification engine** that runs 100% privately on your computer:
1. **Instant AST Codebase Recall (`cmp_search`)**: Indexes your symbols, functions, and caller/callee graphs into a single local SQLite file (`~/.codememory_prime.db`) so your AI finds exact definitions in **0.4ms** without reading entire files.
2. **Cryptographic Solution & Failure Vaults (`cmp_architect`)**: Remembers proven code patterns (**Solution Vault**) and blocks the AI from repeating past mistakes (**Failure Vault**) before it edits a file.
3. **Grimoire Algorithmica**: Gives your AI a library of contract-verified, O(1)/O(N) computer science spells so it stops hallucinating buggy boilerplate.
4. **Pro Dual-Brain & Autonomous Daemons (`CMP-Pro`)**: Adds **70–85% cloud token compression (`bifrost-wire`)**, **zero-token shorthand compilation (`bifrost-dsl`)**, **24/7 background AST gardening & local unit-test synthesis (`cmp-gardener`)**, and **isolated Git worktree task runners (`cmp-worker`)**.

---

## ⚡ Quickest Way to Install (1-Command Auto-Installer)

If you want to get up and running in **under 60 seconds**, our universal installer automatically detects your OS, sets up the native binary, and wires up Claude Desktop, Cursor, Windsurf, VS Code, Claude Code CLI, and Antigravity for you!

### Option 1: Install CodeMemoryPrime-Pro (Commercial / Pro Suite)
```bash
curl -fsSL https://codememoryprime.com/install-pro.sh | bash
```
*(Or via Rust Cargo)*:
```bash
cargo install --git https://github.com/kichikubeihei/CodeMemoryPrime-Pro.git --bin cmp-pro --force && cmp-pro install --all
```

### Option 2: Install CodeMemoryPrime (Free Open-Source Core — BSL 1.1)
```bash
git clone https://github.com/kichikubeihei/CodeMemoryPrime.git
cd CodeMemoryPrime
cargo build --release
```

---

## 📋 Full Step-by-Step Setup Checklist (For Beginners)

If you prefer a guided, click-by-click walkthrough, we will cover:
- **Step 1**: (Optional) Install **Ollama** so your own computer's GPU can compress prompts and synthesize unit tests for **$0.00**.
- **Step 2**: Download the best free local coding model for your computer's RAM.
- **Step 3**: Install the `cmp-pro` (or `codememory-prime`) binary on macOS, Windows, or Linux.
- **Step 4**: (Pro Users) Activate your offline `CMP_LICENSE_KEY`.
- **Step 5**: Connect CodeMemoryPrime to your favorite AI IDE.
- **Step 6**: Try your first AI prompts!

---

## 🦙 Step 1: Install Ollama (Optional — Enables $0.00 Dual-Brain Local AI)

> **Note**: CodeMemoryPrime's SQLite AST search, Solution Vault, Failure Vault, and Grimoire Algorithmica work **100% standalone without Ollama**. Installing Ollama is optional and unlocks **$0.00 local doc analysis**, **`bifrost-wire` 70–85% cloud token compression**, and **`cmp-gardener` local unit test generation**.

### 🍏 On macOS (Apple Silicon M1/M2/M3/M4 & Intel):
1. Go to **[https://ollama.com/download/mac](https://ollama.com/download/mac)**.
2. Click **Download for macOS**, unzip `Ollama-darwin.zip`, and drag **Ollama** into your **Applications** folder.
3. Launch **Ollama** from Applications. When you see the small llama icon in your top menu bar, Ollama is running at `http://localhost:11434`!

### 🪟 On Windows (Windows 10 & Windows 11):
1. Go to **[https://ollama.com/download/windows](https://ollama.com/download/windows)**.
2. Download and run `OllamaSetup.exe`.
3. Follow the installer prompts. When you see the llama icon in your system tray (bottom-right clock area), Ollama is running!

### 🐧 On Linux (Ubuntu, Debian, Fedora, Arch):
Open your terminal and run:
```bash
curl -fsSL https://ollama.com/install.sh | sh
```

---

## 🧠 Step 2: Pull a Free Local Coding Model (Based on Your RAM)

Open your Terminal (macOS/Linux) or Command Prompt / PowerShell (Windows) and run the command that matches your system memory:

| Your Computer's RAM | Recommended Command | Best For |
| :--- | :--- | :--- |
| **8 GB RAM** (Entry Laptops) | `ollama pull qwen2.5-coder:1.5b` | Ultra-fast lightweight local compression |
| **16 GB RAM** (MacBook Air/Pro, Mid-Range PC) | `ollama pull qwen2.5-coder:7b` ⭐ **(RECOMMENDED)** | Ideal balance of speed and coding accuracy |
| **32 GB+ RAM** (Mac Studio/Max, RTX Gaming PC) | `ollama pull qwen2.5-coder:14b` (or `32b`) | Deep local code synthesis & unit test generation |

*(Using a dedicated GPU PC on your home network or Tailscale? Simply set `export OLLAMA_HOST="http://YOUR-GPU-IP:11434"` and CodeMemoryPrime will automatically offload local inference to it!)*

---

## ⬇️ Step 3: Install the CodeMemoryPrime Binary

### Method A: One-Command Automatic Setup (macOS & Linux)
Open Terminal and run:
```bash
curl -fsSL https://codememoryprime.com/install-pro.sh | bash
```
This installs `cmp-pro` into `~/.local/bin/cmp-pro` and automatically configures your IDEs.

### Method B: Direct Binary Download (macOS, Windows, Linux)
Download the pre-compiled standalone executable for your OS:
- 🍏 **macOS**: `https://codememoryprime.com/downloads/cmp-pro-macos`
- 🪟 **Windows**: `https://codememoryprime.com/downloads/cmp-pro-windows.exe`
- 🐧 **Linux**: `https://codememoryprime.com/downloads/cmp-pro-linux`

1. Create a folder for the binary:
   - **Windows**: `C:\CodeMemoryPrime\` (place `cmp-pro-windows.exe` inside)
   - **macOS / Linux**: `~/CodeMemoryPrime/` (place `cmp-pro` inside and run `chmod +x ~/CodeMemoryPrime/cmp-pro`)

---

## 🔑 Step 4: Set Your Offline License Key (Pro Edition Only)

*If you are using the free Open-Source Core edition under BSL 1.1 (personal projects, students, open source, or teams $\le 3$ devs / $<\$100\text{k}$ revenue), you can skip this step!*

When you purchase **CodeMemoryPrime-Pro**, you receive an offline Ed25519 cryptographic key (`CMP-LICENSE-...`) that never phones home.

### 🍏 On macOS & 🐧 Linux:
Run this in Terminal (replacing with your key):
```bash
mkdir -p ~/.cmp
echo "CMP-LICENSE-YOUR_KEY_HERE" > ~/.cmp/license.key
```
*(Or export `CMP_LICENSE_KEY="CMP-LICENSE-..."` in your `~/.zshrc` or `~/.bashrc`.)*

### 🪟 On Windows (PowerShell):
```powershell
[System.Environment]::SetEnvironmentVariable('CMP_LICENSE_KEY', 'CMP-LICENSE-YOUR_KEY_HERE', 'User')
```

---

## 🔌 Step 5: Connect CodeMemoryPrime to Your AI Editor

If you ran `cmp-pro install --all`, your editors are **already configured**! You can verify anytime by running:
```bash
cmp-pro status
```

If you prefer to configure your editor manually, use the snippets below:

### Option A: Claude Desktop (`claude_desktop_config.json`)
- **macOS Path**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows Path**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "CodeMemoryPrime-Pro": {
      "command": "cmp-pro",
      "args": [],
      "env": {
        "OLLAMA_HOST": "http://localhost:11434"
      }
    }
  }
}
```
*(On Windows, if `cmp-pro` is not in your `PATH`, set `"command": "C:\\CodeMemoryPrime\\cmp-pro-windows.exe"`.)*

### Option B: Cursor (`~/.cursor/mcp.json`), Windsurf, VS Code (Cline/Roo), or Antigravity
Add the exact same `mcpServers` block above into your editor's MCP configuration file, or run:
```bash
cmp-pro install --all
```

### Option C: Claude Code CLI
```bash
claude mcp add CodeMemoryPrime-Pro -- cmp-pro
```

---

## 🎯 Step 6: How to Use CodeMemoryPrime (Copy-Paste Prompts)

Once you restart your AI editor, try pasting any of these prompts into your chat:

### 1. Pre-Flight Context & Dead-End Check (Start of Any Task)
> *"Run `cmp_preflight` for this project so we load our latest session handoff, inspect matching AST symbols, and check the Failure Vault for any known dead-ends before we write code."*

### 2. Sub-Millisecond AST Symbol & Caller Search
> *"Use `cmp_search` to locate the authentication handler and show its caller/callee graph without reading the entire file."*

### 3. Lock & Verify Objective Test Gates (Autonomous Gauntlet)
> *"Run `cmp_gauntlet` with our test command `cargo test` (or `npm test`) to verify our changes pass every check and vault the verified solution into the Solution Vault."*

### 4. Full DevSecOps & Anti-Slop Repository Audit
> *"Use `cmp_audit` with action `full_repo_audit` to scan this repository for hardcoded secrets, monolith files over 250 lines, and leftover TODO/placeholder slop."*

### 5. Save Session Handoff (End of Session)
> *"Run `cmp_handoff` to cryptographically sign and save our current progress and next steps for our next coding session."*

---

## 🌱 Bonus (Pro): Run the 24/7 Background Gardener & Bifrost Wire

### Run `cmp-gardener` (Zero-Token Background AST Indexer & Unit Test Synthesizer)
Want CodeMemoryPrime-Pro to automatically index your repository and use your local GPU to draft unit tests for untested functions while your computer is idle?
```bash
# Run one pass right now in your project folder:
cmp-gardener --once

# Or leave it running every 5 minutes in the background:
cmp-gardener --interval 300
```

### Run `bifrost-wire` (70–85% Cloud Token Compression Benchmark)
Test the Dual-Brain Semantic Wire Protocol right from your terminal:
```bash
cargo run -p bifrost-wire -- --benchmark
```

---

## ❓ Troubleshooting & FAQ

- **Where is my memory stored?**
  By default, everything is stored in a single fast SQLite WAL database at `~/.codememory_prime.db` (or `%USERPROFILE%\.codememory_prime.db` on Windows). You can override this path anytime by setting `CMP_DB_PATH="/custom/path/cmp.db"`.
- **Does CodeMemoryPrime ever upload my source code to the cloud?**
  **Never.** 100% of AST parsing, SQLite indexing, Failure/Solution Vault storage, and Ed25519 license verification run locally on your machine.
- **Need help or support?**
  Visit **[https://codememoryprime.com](https://codememoryprime.com)** or open an issue on **[GitHub](https://github.com/kichikubeihei/CodeMemoryPrime)**!

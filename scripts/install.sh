#!/usr/bin/env bash
set -e

# Colors
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color
BOLD='\033[1m'

echo -e "${CYAN}${BOLD}"
cat << 'EOF'
  ██████╗███╗   ███╗██████╗ 
 ██╔════╝████╗ ████║██╔══██╗
 ██║     ██╔████╔██║██████╔╝
 ██║     ██║╚██╔╝██║██╔═══╝ 
 ╚██████╗██║ ╚═╝ ██║██║     
  ╚═════╝╚═╝     ╚═╝╚═╝     
EOF
echo -e "${NC}  ${BOLD}CodeMemoryPrime Core (cmp): One-Command Universal Installer${NC}\n"

# 1. Detect Architecture & OS
RAW_OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
RAW_ARCH="$(uname -m | tr '[:upper:]' '[:lower:]')"

case "${RAW_OS}" in
    darwin*) OS="darwin" ;;
    linux*)  OS="linux" ;;
    msys*|mingw*|cygwin*) OS="windows" ;;
    *) echo -e "${RED}[✘] Unsupported OS: ${RAW_OS}${NC}"; exit 1 ;;
esac

case "${RAW_ARCH}" in
    x86_64|amd64) ARCH="x86_64" ;;
    arm64|aarch64) ARCH="arm64" ;;
    *) echo -e "${RED}[✘] Unsupported Architecture: ${RAW_ARCH}${NC}"; exit 1 ;;
esac

echo -e "${BOLD}Target System:${NC} ${OS}-${ARCH}"

INSTALL_DIR="${HOME}/.local/bin"
mkdir -p "${INSTALL_DIR}"

CMP_BIN="${INSTALL_DIR}/cmp"
GITHUB_REPO="kichikubeihei/CodeMemoryPrime"
TARBALL="cmp-${OS}-${ARCH}.tar.gz"

# 2. Check for pre-built release binary download
DOWNLOAD_SUCCESS=false

if [ "$1" != "--build-from-source" ]; then
    echo -e "${CYAN}[i] Checking GitHub Releases (${GITHUB_REPO}) for pre-compiled ${TARBALL}...${NC}"
    LATEST_URL="https://github.com/${GITHUB_REPO}/releases/latest/download/${TARBALL}"

    TMP_DIR="$(mktemp -d)"
    trap 'rm -rf "${TMP_DIR}"' EXIT

    if command -v curl >/dev/null 2>&1; then
        if curl -fsSL -o "${TMP_DIR}/${TARBALL}" "${LATEST_URL}" 2>/dev/null; then
            DOWNLOAD_SUCCESS=true
        fi
    elif command -v wget >/dev/null 2>&1; then
        if wget -q -O "${TMP_DIR}/${TARBALL}" "${LATEST_URL}" 2>/dev/null; then
            DOWNLOAD_SUCCESS=true
        fi
    fi

    if [ "${DOWNLOAD_SUCCESS}" = "true" ]; then
        echo -e "${GREEN}[✔] Downloaded pre-compiled release! Unpacking to ${INSTALL_DIR}...${NC}"
        tar -xzf "${TMP_DIR}/${TARBALL}" -C "${TMP_DIR}"
        mv "${TMP_DIR}/cmp" "${CMP_BIN}"
        chmod +x "${CMP_BIN}"
    fi
fi

# 3. Fallback: Compile from source via Cargo if release tarball is not published
if [ ! -x "${CMP_BIN}" ]; then
    if command -v cargo >/dev/null 2>&1; then
        echo -e "${YELLOW}[i] Building optimized native binary via Cargo (bundled SQLite)...${NC}"
        SCRIPT_DIR=""
        if [ -n "${BASH_SOURCE[0]}" ] && [ -f "${BASH_SOURCE[0]}" ]; then
            SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
        fi
        REPO_DIR=""
        if [ -n "${SCRIPT_DIR}" ]; then
            REPO_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
        fi

        if [ -n "${REPO_DIR}" ] && [ -f "${REPO_DIR}/Cargo.toml" ]; then
            cargo build --release --manifest-path "${REPO_DIR}/Cargo.toml" --bin cmp
            cp "${REPO_DIR}/target/release/cmp" "${CMP_BIN}"
            chmod +x "${CMP_BIN}"
        else
            cargo install --git "https://github.com/${GITHUB_REPO}.git" --bin cmp --force
            if [ -f "${HOME}/.cargo/bin/cmp" ]; then
                cp "${HOME}/.cargo/bin/cmp" "${CMP_BIN}"
                chmod +x "${CMP_BIN}"
            fi
        fi
    elif [ -x "$(command -v cmp 2>/dev/null)" ]; then
        CMP_BIN="$(command -v cmp)"
        echo -e "${GREEN}[✔] Using existing binary on PATH:${NC} ${CMP_BIN}"
    else
        echo -e "${RED}[✘] Cargo is required to compile CodeMemoryPrime on ${OS}-${ARCH}.${NC}"
        echo -e "${YELLOW}Install Rust in 10 seconds via: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"
        exit 1
    fi
fi

echo -e "${GREEN}[✔] Binary Ready:${NC} ${CMP_BIN}\n"

# 4. PATH Check
if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
    echo -e "${YELLOW}[!] Notice:${NC} ${INSTALL_DIR} is not in your current \$PATH."
    echo -e "    Add this to your ~/.zshrc or ~/.bashrc:"
    echo -e "    ${BOLD}export PATH=\"${INSTALL_DIR}:\$PATH\"${NC}\n"
    export PATH="${INSTALL_DIR}:${PATH}"
fi

# 5. Detect Local or Tailscale Ollama Node (Optional $0.00 Inference)
echo -e "${CYAN}[i] Scanning for optional local/Tailscale Ollama GPU nodes...${NC}"
if curl -s --max-time 1 http://127.0.0.1:11434/api/tags >/dev/null 2>&1; then
    echo -e "${GREEN}[✔] Local Ollama node detected at http://127.0.0.1:11434!${NC}"
else
    echo -e "${YELLOW}[i] No local Ollama node required (bundled SQLite FTS5 & AST engine runs 100% standalone).${NC}"
fi
echo ""

# 6. Execute Native Auto-Installer to wire all detected IDEs
echo -e "${CYAN}[i] Wiring MCP configuration across Claude Desktop, Cursor, Windsurf, Claude Code, VS Code & Antigravity...${NC}"
"${CMP_BIN}" install --all "$@"

echo -e "\n${GREEN}${BOLD}✨ CodeMemoryPrime Installation Complete!${NC}"
echo -e "• Run ${BOLD}cmp status${NC} anytime to inspect your IDE wiring and SQLite memory database."
echo -e "• Run ${BOLD}cmp preflight${NC} to execute a full system preflight and cryptographic integrity check."

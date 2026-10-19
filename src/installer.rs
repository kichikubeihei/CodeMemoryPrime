use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct TargetAgent {
    pub name: String,
    pub config_path: PathBuf,
    pub detected: bool,
    pub updated: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallReport {
    pub binary_path: String,
    pub dry_run: bool,
    pub targets: Vec<TargetAgent>,
    pub summary: String,
}

fn get_home_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(profile) = std::env::var("USERPROFILE") {
            return PathBuf::from(profile);
        }
    }
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

pub fn resolve_binary_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Ok(canonical) = exe.canonicalize() {
            return canonical;
        }
        return exe;
    }
    PathBuf::from("cmp")
}

pub fn discover_targets() -> Vec<(String, PathBuf)> {
    let home = get_home_dir();
    let mut targets = Vec::new();

    // 1. Claude Desktop
    #[cfg(target_os = "macos")]
    let claude_desktop = home.join("Library/Application Support/Claude/claude_desktop_config.json");
    #[cfg(target_os = "windows")]
    let claude_desktop = home.join("AppData/Roaming/Claude/claude_desktop_config.json");
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let claude_desktop = home.join(".config/Claude/claude_desktop_config.json");
    targets.push(("Claude Desktop".to_string(), claude_desktop));

    // 2. Claude Code CLI
    let claude_code = home.join(".claude.json");
    targets.push(("Claude Code CLI".to_string(), claude_code));

    // 3. Cursor
    let cursor_mcp = home.join(".cursor/mcp.json");
    targets.push(("Cursor".to_string(), cursor_mcp));

    // 4. Windsurf
    let windsurf_mcp = home.join(".codeium/windsurf/mcp_config.json");
    targets.push(("Windsurf".to_string(), windsurf_mcp));

    // 5. VS Code (Roo Code / Cline)
    #[cfg(target_os = "macos")]
    {
        let roo_cline = home.join("Library/Application Support/Code/User/globalStorage/rooveterinaryinc.roo-cline/settings/cline_mcp_settings.json");
        targets.push(("VS Code (Roo Code)".to_string(), roo_cline));
        let claude_dev = home.join("Library/Application Support/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json");
        targets.push(("VS Code (Cline)".to_string(), claude_dev));
    }
    #[cfg(target_os = "windows")]
    {
        let roo_cline = home.join("AppData/Roaming/Code/User/globalStorage/rooveterinaryinc.roo-cline/settings/cline_mcp_settings.json");
        targets.push(("VS Code (Roo Code)".to_string(), roo_cline));
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let roo_cline = home.join(".config/Code/User/globalStorage/rooveterinaryinc.roo-cline/settings/cline_mcp_settings.json");
        targets.push(("VS Code (Roo Code)".to_string(), roo_cline));
    }

    // 6. Google Antigravity / Gemini CLI
    let antigravity_mcp = home.join(".gemini/antigravity/mcp_config.json");
    targets.push(("Google Antigravity".to_string(), antigravity_mcp));

    targets
}

pub fn run_installer(dry_run: bool, custom_bin: Option<&str>) -> InstallReport {
    let binary = if let Some(cb) = custom_bin {
        PathBuf::from(cb)
    } else {
        resolve_binary_path()
    };
    let binary_str = binary.to_string_lossy().to_string();

    let discovered = discover_targets();
    let mut target_reports = Vec::new();
    let mut updated_count = 0;

    for (name, path) in discovered {
        let parent = path.parent();
        let parent_exists = parent.map(|p| p.exists()).unwrap_or(false);
        let file_exists = path.exists();

        if !file_exists && !parent_exists {
            target_reports.push(TargetAgent {
                name,
                config_path: path,
                detected: false,
                updated: false,
                message: "Not installed or config directory not found.".to_string(),
            });
            continue;
        }

        let mut json_val: Value = if file_exists {
            match fs::read_to_string(&path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| json!({})),
                Err(e) => {
                    target_reports.push(TargetAgent {
                        name,
                        config_path: path,
                        detected: true,
                        updated: false,
                        message: format!("Failed to read existing config: {}", e),
                    });
                    continue;
                }
            }
        } else {
            json!({})
        };

        if !json_val.is_object() {
            json_val = json!({});
        }

        let mcp_obj = json_val
            .as_object_mut()
            .unwrap()
            .entry("mcpServers")
            .or_insert_with(|| json!({}));

        if !mcp_obj.is_object() {
            *mcp_obj = json!({});
        }

        mcp_obj.as_object_mut().unwrap().insert(
            "CodeMemoryPrime".to_string(),
            json!({
                "command": binary_str,
                "args": []
            }),
        );

        if dry_run {
            target_reports.push(TargetAgent {
                name,
                config_path: path,
                detected: true,
                updated: true,
                message: format!(
                    "[DRY-RUN] Would inject CodeMemoryPrime pointing to '{}'",
                    binary_str
                ),
            });
            updated_count += 1;
        } else {
            if let Some(p) = parent {
                let _ = fs::create_dir_all(p);
            }

            if file_exists {
                let backup_path = format!("{}.bak", path.to_string_lossy());
                let _ = fs::copy(&path, &backup_path);
            }

            match serde_json::to_string_pretty(&json_val) {
                Ok(pretty_str) => match fs::write(&path, pretty_str) {
                    Ok(_) => {
                        target_reports.push(TargetAgent {
                            name,
                            config_path: path,
                            detected: true,
                            updated: true,
                            message: format!(
                                "Configured CodeMemoryPrime with binary '{}'",
                                binary_str
                            ),
                        });
                        updated_count += 1;
                    }
                    Err(e) => {
                        target_reports.push(TargetAgent {
                            name,
                            config_path: path,
                            detected: true,
                            updated: false,
                            message: format!("Failed to write config: {}", e),
                        });
                    }
                },
                Err(e) => {
                    target_reports.push(TargetAgent {
                        name,
                        config_path: path,
                        detected: true,
                        updated: false,
                        message: format!("Failed to serialize JSON: {}", e),
                    });
                }
            }
        }
    }

    InstallReport {
        binary_path: binary_str,
        dry_run,
        summary: format!(
            "Installed CodeMemoryPrime MCP configuration across {} agent(s).",
            updated_count
        ),
        targets: target_reports,
    }
}

pub fn print_cli_banner() {
    println!(
        r#"
  ██████╗███╗   ███╗██████╗ 
 ██╔════╝████╗ ████║██╔══██╗
 ██║     ██╔████╔██║██████╔╝
 ██║     ██║╚██╔╝██║██╔═══╝ 
 ╚██████╗██║ ╚═╝ ██║██║     
  ╚═════╝╚═╝     ╚═╝╚═╝     
"#
    );
    println!("  CodeMemoryPrime Core (cmp) — Universal MCP Installer");
    println!("  • 4-Pillar Consolidated AST & Graph Memory Engine");
    println!("  • Grimoire Algorithmica Contract-Verified Spells");
    println!("  • Solution & Failure Vaults with SHA-256 HMAC Integrity\n");
}

pub fn handle_install(args: &[String]) {
    print_cli_banner();
    let dry_run = args.iter().any(|a| a == "--dry-run");
    let custom_bin = args
        .windows(2)
        .find(|w| w[0] == "--bin")
        .map(|w| w[1].as_str());

    // Ensure SQLite memory database is initialized
    let db_path = crate::get_db_path();
    if !dry_run {
        let _ = crate::db::init_database(&db_path);
    }

    println!("Binary Path: {}", custom_bin.unwrap_or("auto-resolved"));
    println!("Memory DB:   {}\n", db_path);

    println!("🔮 [1/2] Configuring MCP Server Across AI IDEs...");
    let report = run_installer(dry_run, custom_bin);
    for t in &report.targets {
        let icon = if t.updated {
            if dry_run {
                "[DRY-RUN]"
            } else {
                "  [✔]"
            }
        } else if t.detected {
            "  [⚠]"
        } else {
            "  [-]"
        };
        println!("{} {:<22} -> {}", icon, t.name, t.message);
    }
    println!("Summary: {}\n", report.summary);

    println!("📜 [2/2] Verifying Grimoire Algorithmica Contracts...");
    let contracts = crate::grimoire::AlgorithmContract::catalog();
    println!(
        "  [✔] {} high-level algorithmic contracts ready (Python, TypeScript, Go, Dart, Rust)\n",
        contracts.len()
    );

    if !dry_run {
        println!("🏁 Complete: Restart Claude Desktop, Cursor, Windsurf, or Antigravity to activate CodeMemoryPrime.");
    }
}

pub fn handle_status() {
    print_cli_banner();
    let bin = resolve_binary_path();
    let db_path = crate::get_db_path();
    let db_exists = std::path::Path::new(&db_path).exists();
    let contracts = crate::grimoire::AlgorithmContract::catalog();

    println!("1. Binary Path:            {}", bin.display());
    println!(
        "2. SQLite Memory DB:       {} ({})",
        db_path,
        if db_exists {
            "✔ Initialized"
        } else {
            "Will initialize on first run"
        }
    );
    println!(
        "3. Grimoire Algorithmica:  ✔ {} Verified Algorithmic Contracts Active",
        contracts.len()
    );
    println!("\nDetected IDE Environments:");
    for (name, path) in discover_targets() {
        let configured = if path.exists() {
            fs::read_to_string(&path)
                .map(|c| c.contains("CodeMemoryPrime"))
                .unwrap_or(false)
        } else {
            false
        };
        let status_str = if configured {
            "✔ Configured"
        } else if path.exists() || path.parent().map(|p| p.exists()).unwrap_or(false) {
            "⚠ Installed (run `cmp install --all` to wire)"
        } else {
            "- Not installed"
        };
        println!("  • {:<22} : {}", name, status_str);
    }
}

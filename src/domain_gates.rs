use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRule {
    pub domain_id: String,
    pub name: String,
    pub trigger_paths: Vec<String>,
    pub trigger_keywords: Vec<String>,
    pub rule_markdown: String,
    pub priority: i32,
    pub updated_at: String,
}

pub fn init_domain_rules_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS domain_rules (
            domain_id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            trigger_paths TEXT NOT NULL,
            trigger_keywords TEXT NOT NULL,
            rule_markdown TEXT NOT NULL,
            priority INTEGER DEFAULT 10,
            updated_at TEXT NOT NULL
        );",
        [],
    )?;

    seed_default_domains(conn)?;
    Ok(())
}

pub fn seed_default_domains(conn: &Connection) -> Result<()> {
    // Purge any legacy non-coding domain rules
    let _ = conn.execute(
        "DELETE FROM domain_rules WHERE domain_id NOT IN ('rust_systems_concurrency', 'modern_web_engineering', 'database_systems_architecture', 'high_performance_compute', 'cmp_core_standards', 'go_modern_guidelines')",
        [],
    );

    let count: i64 = conn.query_row("SELECT COUNT(*) FROM domain_rules", [], |r| r.get(0)).unwrap_or(0);
    if count >= 6 {
        return Ok(());
    }

    let defaults = vec![
        DomainRule {
            domain_id: "rust_systems_concurrency".to_string(),
            name: "Rust Systems & Concurrency Standards".to_string(),
            trigger_paths: vec![".rs".to_string(), "cargo.toml".to_string()],
            trigger_keywords: vec![
                "rust".to_string(),
                "async".to_string(),
                "tokio".to_string(),
                "thread".to_string(),
                "atomic".to_string(),
                "mutex".to_string(),
                "channel".to_string(),
            ],
            rule_markdown: "### 🦀 Rust Systems & Concurrency Standards\n1. Panic-Free: Production code must never call `unwrap()` or `expect()`. Propagate errors via `Result<T, E>`.\n2. Lock-Free & Concurrency: Prefer lock-free atomics (`AtomicBool`, `AtomicUsize`) or SPSC/crossbeam queues over coarse mutexes in hot loops.\n3. Zero-Allocation: Minimize heap allocations in inner loops; use pre-allocated buffers and slices (`&[T]`, `&mut [T]`).\n4. Formally Proven: Algorithms must maintain strict inductive loop invariants and model checking properties.".to_string(),
            priority: 95,
            updated_at: chrono::Utc::now().to_rfc3339(),
        },
        DomainRule {
            domain_id: "modern_web_engineering".to_string(),
            name: "Modern Web & Svelte 5 Standards".to_string(),
            trigger_paths: vec![".svelte".to_string(), ".ts".to_string(), ".tsx".to_string()],
            trigger_keywords: vec![
                "svelte".to_string(),
                "runes".to_string(),
                "$state".to_string(),
                "$derived".to_string(),
                "react".to_string(),
                "css".to_string(),
                "tailwind".to_string(),
                "a11y".to_string(),
            ],
            rule_markdown: "### 🌐 Modern Web & Svelte 5 Standards\n1. Svelte 5 Native: Use strictly Svelte 5 Runes (`$state`, `$derived`, `$effect`, `$props`). Never use legacy Svelte 4 store syntax (`writable`, `$store`).\n2. Performance & CLS: Guarantee zero Cumulative Layout Shift (`CLS = 0.00`). Explicitly dimension images and dynamic containers.\n3. Accessibility (a11y): Enforce semantic HTML5, explicit ARIA roles/labels, full keyboard navigation, and minimum 48x48px tap targets.\n4. Responsive & Fluid: Use container queries and clamp() for typography; no rigid breakpoint overflows.".to_string(),
            priority: 90,
            updated_at: chrono::Utc::now().to_rfc3339(),
        },
        DomainRule {
            domain_id: "database_systems_architecture".to_string(),
            name: "Database Systems & Query Optimization".to_string(),
            trigger_paths: vec![".sql".to_string(), "migrations".to_string(), "models".to_string()],
            trigger_keywords: vec![
                "sql".to_string(),
                "sqlite".to_string(),
                "postgres".to_string(),
                "query".to_string(),
                "index".to_string(),
                "transaction".to_string(),
                "database".to_string(),
                "migration".to_string(),
            ],
            rule_markdown: "### 🗄️ Database Systems & Query Optimization\n1. Eager Loading: Always eager-load relations to completely eliminate N+1 query bottlenecks.\n2. Indexing Strategy: Ensure all foreign keys, composite filters, and sort columns have covering SQLite/PostgreSQL indexes.\n3. Transactional ACID Integrity: Wrap multi-table mutations in atomic transactions with explicit rollback on error.\n4. Read Isolation: Exploratory agent analysis must use read-only transactions or read replicas.".to_string(),
            priority: 85,
            updated_at: chrono::Utc::now().to_rfc3339(),
        },
        DomainRule {
            domain_id: "high_performance_compute".to_string(),
            name: "High-Performance Compute & Cache Blocking".to_string(),
            trigger_paths: vec!["math".to_string(), "simd".to_string(), "matrix".to_string(), "algorithm".to_string()],
            trigger_keywords: vec![
                "cache".to_string(),
                "blocking".to_string(),
                "simd".to_string(),
                "vector".to_string(),
                "tiling".to_string(),
                "stride".to_string(),
                "l1".to_string(),
                "l2".to_string(),
            ],
            rule_markdown: "### ⚡ High-Performance Compute & Cache Blocking\n1. Cache-Line Tiling: Nested loops on multidimensional data must use tile blocking (`BLOCK_SIZE = 32..64`) to eliminate L1/L2 cache line misses.\n2. Vectorization & SIMD: Align arrays to 32/64-byte boundaries for auto-vectorization or explicit SIMD intrinsics.\n3. Anti-Cheat Integrity: Code must execute real, load-bearing computation; pseudo-optimizations (like fake Morton reordering with naive inner loops) are rejected.".to_string(),
            priority: 80,
            updated_at: chrono::Utc::now().to_rfc3339(),
        },
        DomainRule {
            domain_id: "cmp_core_standards".to_string(),
            name: "CodeMemoryPrime Core Engineering Standards".to_string(),
            trigger_paths: vec!["CodeMemoryPrime".to_string(), "CodeMemoryPrime-Pro".to_string(), "mcp-coder-memory-rust".to_string()],
            trigger_keywords: vec![
                "codememory".to_string(),
                "cmp".to_string(),
                "vault".to_string(),
                "hmac".to_string(),
                "mesh".to_string(),
                "r2".to_string(),
                "tailscale".to_string(),
            ],
            rule_markdown: "### 🧠 CodeMemoryPrime Core Standards\n1. Anti-Monolith LOC Bounds: UI components <= 200 LOC, logic modules <= 250 LOC.\n2. Cryptographic Memory: All vaults must be sealed with HMAC-SHA256 signatures before persistence.\n3. Zero-Egress Sync: Default multi-device synchronization to Cloudflare R2 and Tailscale mesh daemons.".to_string(),
            priority: 100,
            updated_at: chrono::Utc::now().to_rfc3339(),
        },
        DomainRule {
            domain_id: "go_modern_guidelines".to_string(),
            name: "Go Modern Guidelines (JetBrains Canonical Best Practices)".to_string(),
            trigger_paths: vec![".go".to_string(), "go.mod".to_string()],
            trigger_keywords: vec![
                "golang".to_string(),
                "go".to_string(),
                "goroutine".to_string(),
                "channel".to_string(),
                "struct".to_string(),
                "interface".to_string(),
            ],
            rule_markdown: "### 🐹 Go Modern Guidelines (JetBrains Canonical Standards)\n1. Standard Library First: Use Go 1.22+ `slog` for structured logging, `net/http` enhanced routing with path variables (`GET /items/{id}`), and standard `math/rand/v2`.\n2. Iterators & Slices: Use Go 1.23 range over functions / iterators (`iter.Seq`), `slices`, and `maps` packages over manual index loops.\n3. Error Wrapping: Always wrap errors with `%w` using `fmt.Errorf(\"...: %w\", err)` and inspect via `errors.Is` / `errors.As`.\n4. Concurrency Safety: Always pass `context.Context` as first argument; manage goroutine lifecycles with `errgroup.Group` rather than naked `sync.WaitGroup`.".to_string(),
            priority: 85,
            updated_at: chrono::Utc::now().to_rfc3339(),
        },
    ];

    for r in defaults {
        let paths_json = serde_json::to_string(&r.trigger_paths).unwrap_or_default();
        let kw_json = serde_json::to_string(&r.trigger_keywords).unwrap_or_default();
        conn.execute(
            "INSERT OR REPLACE INTO domain_rules (domain_id, name, trigger_paths, trigger_keywords, rule_markdown, priority, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![r.domain_id, r.name, paths_json, kw_json, r.rule_markdown, r.priority, r.updated_at],
        )?;
    }

    Ok(())
}

pub fn get_active_domain_rules(
    conn: &Connection,
    project_name: &str,
    query_or_path: &str,
) -> Vec<DomainRule> {
    let _ = init_domain_rules_table(conn);
    let mut stmt = match conn.prepare(
        "SELECT domain_id, name, trigger_paths, trigger_keywords, rule_markdown, priority, updated_at
         FROM domain_rules ORDER BY priority DESC"
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let p_lower = project_name.to_lowercase();
    let q_lower = query_or_path.to_lowercase();

    let rows = stmt.query_map([], |row| {
        let paths_str: String = row.get(2)?;
        let kw_str: String = row.get(3)?;
        Ok(DomainRule {
            domain_id: row.get(0)?,
            name: row.get(1)?,
            trigger_paths: serde_json::from_str(&paths_str).unwrap_or_default(),
            trigger_keywords: serde_json::from_str(&kw_str).unwrap_or_default(),
            rule_markdown: row.get(4)?,
            priority: row.get(5)?,
            updated_at: row.get(6)?,
        })
    });

    let mut matched = Vec::new();
    if let Ok(iter) = rows {
        for item in iter.flatten() {
            let mut is_match = false;

            for tp in &item.trigger_paths {
                let tp_lower = tp.to_lowercase();
                if p_lower.contains(&tp_lower) || q_lower.contains(&tp_lower) {
                    is_match = true;
                    break;
                }
            }

            if !is_match {
                for kw in &item.trigger_keywords {
                    let kw_lower = kw.to_lowercase();
                    if q_lower.contains(&kw_lower) {
                        is_match = true;
                        break;
                    }
                }
            }

            if is_match {
                matched.push(item);
            }
        }
    }

    // Also load workspace-local rules from .cmp/rules.json if present (keeps private schemas isolated)
    for base_dir in &[query_or_path, ".", "src"] {
        let local_cfg = std::path::Path::new(base_dir).join(".cmp").join("rules.json");
        if local_cfg.exists() {
            if let Ok(content) = std::fs::read_to_string(&local_cfg) {
                if let Ok(custom_rules) = serde_json::from_str::<Vec<DomainRule>>(&content) {
                    for cr in custom_rules {
                        if !matched.iter().any(|m| m.domain_id == cr.domain_id) {
                            matched.push(cr);
                        }
                    }
                }
            }
            break;
        }
    }

    matched
}

pub fn format_active_domain_markdown(rules: &[DomainRule]) -> String {
    if rules.is_empty() {
        return "No project-specific domain rules active. Standard clean context applies.".to_string();
    }

    let mut out = String::from("## 🎯 Active Dynamic Knowledge Domains (Gated Context):\n\n");
    for r in rules {
        out.push_str(&format!("### Domain: {}\n", r.name));
        out.push_str(&r.rule_markdown);
        out.push_str("\n\n");
    }
    out
}

pub fn list_all_domain_rules(conn: &Connection) -> Result<Vec<DomainRule>> {
    let _ = init_domain_rules_table(conn);
    let mut stmt = conn.prepare(
        "SELECT domain_id, name, trigger_paths, trigger_keywords, rule_markdown, priority, updated_at
         FROM domain_rules ORDER BY priority DESC"
    )?;

    let rows = stmt.query_map([], |row| {
        let paths_str: String = row.get(2)?;
        let kw_str: String = row.get(3)?;
        Ok(DomainRule {
            domain_id: row.get(0)?,
            name: row.get(1)?,
            trigger_paths: serde_json::from_str(&paths_str).unwrap_or_default(),
            trigger_keywords: serde_json::from_str(&kw_str).unwrap_or_default(),
            rule_markdown: row.get(4)?,
            priority: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;

    let mut rules = Vec::new();
    for r in rows.flatten() {
        rules.push(r);
    }
    Ok(rules)
}

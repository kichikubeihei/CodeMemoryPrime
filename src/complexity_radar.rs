use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetrics {
    pub file_path: String,
    pub project_name: String,
    pub total_lines: usize,
    pub function_count: usize,
    pub max_nesting_depth: usize,
    pub file_inputs: String,
    pub file_outputs: String,
    pub afferent_coupling: usize,
    pub efferent_coupling: usize,
    pub instability_index: f64,
    pub token_cost: usize,
    pub optimization_status: String,
    pub optimized_by_model: Option<String>,
    pub optimized_at: Option<u64>,
    pub paradigm_version: Option<String>,
    pub optimization_rationale: Option<String>,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMetrics {
    pub id: String,
    pub file_path: String,
    pub project_name: String,
    pub function_name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub loc: usize,
    pub inputs: String,
    pub outputs: String,
    pub call_nesting_depth: usize,
    pub cyclomatic_complexity: usize,
    pub crap_score: f64,
    pub is_async: bool,
    pub await_count: usize,
    pub has_await_in_loop: bool,
    pub is_pure: bool,
    pub unwrap_count: usize,
    pub swallowed_catches: usize,
    pub token_cost: usize,
    pub urgency_score: usize, // 0 to 100
    pub dominant_smell: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsDeltaReport {
    pub file_path: String,
    pub loc_delta: i64,
    pub nesting_delta: i64,
    pub peak_urgency_delta: i64,
    pub crap_delta: f64,
    pub waterfalls_eliminated: i64,
    pub swallowed_catches_cleaned: i64,
    pub summary_markdown: String,
}

/// Initializes the file_metrics and function_metrics tables in SQLite.
pub fn init_metrics_tables(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS file_metrics (
            file_path TEXT PRIMARY KEY,
            project_name TEXT NOT NULL,
            total_lines INTEGER NOT NULL,
            function_count INTEGER NOT NULL,
            max_nesting_depth INTEGER NOT NULL,
            file_inputs TEXT,
            file_outputs TEXT,
            afferent_coupling INTEGER DEFAULT 0,
            efferent_coupling INTEGER DEFAULT 0,
            instability_index REAL DEFAULT 0.0,
            token_cost INTEGER NOT NULL,
            optimization_status TEXT DEFAULT 'UNASSESSED',
            optimized_by_model TEXT,
            optimized_at INTEGER,
            paradigm_version TEXT,
            optimization_rationale TEXT,
            checksum TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS function_metrics (
            id TEXT PRIMARY KEY,
            file_path TEXT NOT NULL,
            project_name TEXT NOT NULL,
            function_name TEXT NOT NULL,
            start_line INTEGER NOT NULL,
            end_line INTEGER NOT NULL,
            loc INTEGER NOT NULL,
            inputs TEXT NOT NULL,
            outputs TEXT NOT NULL,
            call_nesting_depth INTEGER NOT NULL,
            cyclomatic_complexity INTEGER NOT NULL,
            crap_score REAL NOT NULL,
            is_async INTEGER DEFAULT 0,
            await_count INTEGER DEFAULT 0,
            has_await_in_loop INTEGER DEFAULT 0,
            is_pure INTEGER DEFAULT 0,
            unwrap_count INTEGER DEFAULT 0,
            swallowed_catches INTEGER DEFAULT 0,
            token_cost INTEGER NOT NULL,
            urgency_score INTEGER NOT NULL,
            dominant_smell TEXT NOT NULL,
            FOREIGN KEY(file_path) REFERENCES file_metrics(file_path) ON DELETE CASCADE
        )",
        [],
    )?;

    // Indexes for sub-5ms radar queries
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_fn_urgency ON function_metrics(project_name, urgency_score DESC)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_file_proj_opt ON file_metrics(project_name, optimization_status)", []);

    Ok(())
}

/// Microsecond single-pass AST heuristic parser.
pub fn analyze_source_code(file_path: &str, content: &str, project_name: &str) -> (FileMetrics, Vec<FunctionMetrics>) {
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();
    let token_cost = (content.len() as f64 / 3.8).ceil() as usize;

    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let checksum = hex::encode(hasher.finalize());

    let ext = Path::new(file_path).extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    let is_rust = ext == "rs";
    let _is_ts_js = ["ts", "tsx", "js", "jsx", "svelte"].contains(&ext.as_str());

    let mut functions = Vec::new();
    let mut file_max_nesting = 0;

    let mut file_inputs = Vec::new();
    let mut file_outputs = Vec::new();

    // Regexes for functions across languages
    let fn_regex_rs = Regex::new(r#"^\s*(?:pub\s+)?(?:async\s+)?fn\s+([a-zA-Z0-9_]+)\s*(?:<[^>]+>)?\s*\(([^\)]*)\)(?:\s*->\s*([^\{]+))?"#).unwrap();
    let fn_regex_ts = Regex::new(r#"(?:export\s+)?(?:default\s+)?(?:async\s+)?function\s+([a-zA-Z0-9_]+)\s*(?:<[^>]+>)?\s*\(([^\)]*)\)(?:\s*:\s*([^{=]+))?|(?:export\s+)?(?:const|let|var)\s+([a-zA-Z0-9_]+)\s*(?::\s*[^=]+)?=\s*(?:async\s*)?(?:\(([^\)]*)\)|[a-zA-Z0-9_]+)\s*=>|^\s*(?:public\s+|private\s+|protected\s+|static\s+|async\s+)*([a-zA-Z0-9_]+)\s*\(([^\)]*)\)(?:\s*:\s*([^{=]+))?\s*\{"#).unwrap();

    let mut in_function = false;
    let mut curr_fn_name = String::new();
    let mut curr_fn_start = 1;
    let mut curr_fn_inputs = String::new();
    let mut curr_fn_outputs = String::new();
    let mut curr_fn_is_async = false;
    let mut curr_fn_await_count = 0;
    let mut curr_fn_await_in_loop = false;
    let mut curr_fn_branches = 1;
    let mut curr_fn_unwraps = 0;
    let mut curr_fn_swallowed = 0;
    let mut curr_fn_has_state_mutation = false;
    let mut curr_fn_max_nesting = 0;
    let mut curr_fn_brace_balance = 0;
    let mut loop_depth = 0;

    for (idx, line) in lines.iter().enumerate() {
        let line_num = idx + 1;
        let trimmed = line.trim();

        // Calculate indentation depth (spaces / 4 or tabs)
        let leading_spaces = line.chars().take_while(|c| *c == ' ' || *c == '\t').map(|c| if c == '\t' { 4 } else { 1 }).sum::<usize>();
        let nesting_level = leading_spaces / 4;
        if nesting_level > file_max_nesting {
            file_max_nesting = nesting_level;
        }

        // File-level input/output detection
        if line.contains("$props()") || line.contains("interface Props") || line.contains("Args") {
            file_inputs.push(trimmed.to_string());
        }
        if line.contains("export default") || line.contains("pub struct ") || line.contains("pub enum ") {
            file_outputs.push(trimmed.to_string());
        }

        // Check for function declaration if not already in one
        if !in_function {
            let mut detected = None;
            if is_rust {
                if let Some(caps) = fn_regex_rs.captures(line) {
                    let name = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                    let in_params = caps.get(2).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
                    let out_ret = caps.get(3).map(|m| m.as_str().trim().to_string()).unwrap_or_else(|| "()".to_string());
                    detected = Some((name, in_params, out_ret, line.contains("async ")));
                }
            } else {
                if let Some(caps) = fn_regex_ts.captures(line) {
                    let name = caps.get(1).or_else(|| caps.get(4)).or_else(|| caps.get(6)).map(|m| m.as_str().to_string()).unwrap_or_default();
                    let in_params = caps.get(2).or_else(|| caps.get(5)).or_else(|| caps.get(7)).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
                    let out_ret = caps.get(3).or_else(|| caps.get(8)).map(|m| m.as_str().trim().to_string()).unwrap_or_else(|| "void".to_string());
                    detected = Some((name, in_params, out_ret, line.contains("async ") || line.contains("async(")));
                }
            }

            if let Some((name, in_p, out_r, is_async)) = detected {
                if !name.is_empty() && name != "if" && name != "switch" && name != "catch" {
                    in_function = true;
                    curr_fn_name = name;
                    curr_fn_start = line_num;
                    curr_fn_inputs = in_p;
                    curr_fn_outputs = out_r;
                    curr_fn_is_async = is_async;
                    curr_fn_await_count = 0;
                    curr_fn_await_in_loop = false;
                    curr_fn_branches = 1;
                    curr_fn_unwraps = 0;
                    curr_fn_swallowed = 0;
                    curr_fn_has_state_mutation = false;
                    curr_fn_max_nesting = 0;
                    curr_fn_brace_balance = 0;
                    loop_depth = 0;
                }
            }
        }

        // Inside function body analysis
        if in_function {
            let open_b = line.matches('{').count();
            let close_b = line.matches('}').count();
            curr_fn_brace_balance += open_b as i32 - close_b as i32;

            if nesting_level > curr_fn_max_nesting {
                curr_fn_max_nesting = nesting_level;
            }

            // Loop tracking
            if trimmed.starts_with("for ") || trimmed.starts_with("for(") || trimmed.starts_with("while ") || trimmed.starts_with("while(") {
                loop_depth += 1;
            }

            // Async & waterfall tracking
            if trimmed.contains("await ") || trimmed.contains(".await") {
                curr_fn_await_count += 1;
                if loop_depth > 0 {
                    curr_fn_await_in_loop = true;
                }
            }

            // Branches (Cyclomatic)
            let branch_tokens = ["if ", "if(", "match ", "while ", "case ", "&&", "||", "catch ", "?."];
            for t in branch_tokens {
                curr_fn_branches += line.matches(t).count();
            }

            // Brittleness & Slop
            if trimmed.contains(".unwrap()") || trimmed.contains(".expect(") || trimmed.contains("panic!(") {
                curr_fn_unwraps += 1;
            }
            if trimmed.contains("catch {}") || trimmed.contains("catch () {}") || trimmed.contains("except: pass") || trimmed.contains("Err(_) => {}") {
                curr_fn_swallowed += 1;
            }

            // State mutation / impurity
            if trimmed.contains("$state") || trimmed.contains("this.") || trimmed.contains("set(") || trimmed.contains("mut ") || trimmed.contains("=") && !trimmed.contains("==") && !trimmed.contains("===") && !trimmed.contains("const ") {
                curr_fn_has_state_mutation = true;
            }

            // End of function condition
            if curr_fn_brace_balance <= 0 && line_num > curr_fn_start {
                in_function = false;
                let loc = line_num.saturating_sub(curr_fn_start) + 1;
                let fn_tokens = loc * 8;

                // Compute CRAP score (estimated coverage 0.5)
                let crap_score = ((curr_fn_branches as f64).powi(2) * 0.125 + (curr_fn_branches as f64)).round();

                // Compute composite Refactor Urgency Score (0-100)
                let mut urgency = 0;
                let mut dominant_smell = "CLEAN".to_string();

                if curr_fn_await_in_loop {
                    urgency += 35;
                    dominant_smell = "ASYNC_WATERFALL_IN_LOOP".to_string();
                }
                if curr_fn_max_nesting > 3 {
                    urgency += (curr_fn_max_nesting - 3) * 15;
                    dominant_smell = "DEEP_COGNITIVE_NESTING".to_string();
                }
                if curr_fn_branches > 8 {
                    urgency += (curr_fn_branches - 8) * 4;
                    dominant_smell = "CYCLOMATIC_COMPLEXITY".to_string();
                }
                if curr_fn_swallowed > 0 {
                    urgency += curr_fn_swallowed * 25;
                    dominant_smell = "SWALLOWED_ERROR_SLOP".to_string();
                }
                if loc > 60 {
                    urgency += (loc - 60) / 4;
                    dominant_smell = "MONOLITHIC_FUNCTION_LOC".to_string();
                }
                if curr_fn_unwraps > 3 {
                    urgency += curr_fn_unwraps * 5;
                    dominant_smell = "BRITTLE_UNWRAPS".to_string();
                }

                let final_urgency = urgency.min(100);

                functions.push(FunctionMetrics {
                    id: format!("{}:{}", file_path, curr_fn_name),
                    file_path: file_path.to_string(),
                    project_name: project_name.to_string(),
                    function_name: curr_fn_name.clone(),
                    start_line: curr_fn_start,
                    end_line: line_num,
                    loc,
                    inputs: curr_fn_inputs.clone(),
                    outputs: curr_fn_outputs.clone(),
                    call_nesting_depth: curr_fn_max_nesting,
                    cyclomatic_complexity: curr_fn_branches,
                    crap_score,
                    is_async: curr_fn_is_async,
                    await_count: curr_fn_await_count,
                    has_await_in_loop: curr_fn_await_in_loop,
                    is_pure: !curr_fn_has_state_mutation && !curr_fn_is_async,
                    unwrap_count: curr_fn_unwraps,
                    swallowed_catches: curr_fn_swallowed,
                    token_cost: fn_tokens,
                    urgency_score: final_urgency,
                    dominant_smell,
                });
            }
        }
    }

    let file_metrics = FileMetrics {
        file_path: file_path.to_string(),
        project_name: project_name.to_string(),
        total_lines,
        function_count: functions.len(),
        max_nesting_depth: file_max_nesting,
        file_inputs: file_inputs.join("; "),
        file_outputs: file_outputs.join("; "),
        afferent_coupling: 0,
        efferent_coupling: 0,
        instability_index: 0.0,
        token_cost,
        optimization_status: if total_lines > 300 || file_max_nesting > 4 { "NEEDS_REFACTOR".to_string() } else { "UNASSESSED".to_string() },
        optimized_by_model: None,
        optimized_at: None,
        paradigm_version: None,
        optimization_rationale: None,
        checksum,
    };

    (file_metrics, functions)
}

/// Indexes and persists file and function metrics into SQLite with microsecond latency.
pub fn index_file_metrics(conn: &Connection, file_path: &str, content: &str, project_name: &str) -> Result<(FileMetrics, Vec<FunctionMetrics>)> {
    init_metrics_tables(conn)?;
    let (file_m, fns) = analyze_source_code(file_path, content, project_name);

    // Compute Coupling from code_dependencies table
    let ce: usize = conn.query_row(
        "SELECT COUNT(*) FROM code_dependencies WHERE source_file = ?1",
        params![file_path],
        |r| r.get(0),
    ).unwrap_or(0);

    let ca: usize = conn.query_row(
        "SELECT COUNT(*) FROM code_dependencies WHERE import_path LIKE ?1",
        params![format!("%{}%", Path::new(file_path).file_stem().and_then(|s| s.to_str()).unwrap_or(file_path))],
        |r| r.get(0),
    ).unwrap_or(0);

    let instability = if ca + ce > 0 { ce as f64 / (ca + ce) as f64 } else { 0.5 };

    conn.execute(
        "INSERT INTO file_metrics (
            file_path, project_name, total_lines, function_count, max_nesting_depth,
            file_inputs, file_outputs, afferent_coupling, efferent_coupling, instability_index,
            token_cost, optimization_status, checksum
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
        ON CONFLICT(file_path) DO UPDATE SET
            total_lines = excluded.total_lines,
            function_count = excluded.function_count,
            max_nesting_depth = excluded.max_nesting_depth,
            file_inputs = excluded.file_inputs,
            file_outputs = excluded.file_outputs,
            afferent_coupling = excluded.afferent_coupling,
            efferent_coupling = excluded.efferent_coupling,
            instability_index = excluded.instability_index,
            token_cost = excluded.token_cost,
            checksum = excluded.checksum",
        params![
            file_m.file_path,
            file_m.project_name,
            file_m.total_lines as i64,
            file_m.function_count as i64,
            file_m.max_nesting_depth as i64,
            file_m.file_inputs,
            file_m.file_outputs,
            ca as i64,
            ce as i64,
            instability,
            file_m.token_cost as i64,
            file_m.optimization_status,
            file_m.checksum,
        ],
    )?;

    // Clear old function metrics for this file
    let _ = conn.execute("DELETE FROM function_metrics WHERE file_path = ?1", params![file_path]);

    for f in &fns {
        let _ = conn.execute(
            "INSERT INTO function_metrics (
                id, file_path, project_name, function_name, start_line, end_line, loc,
                inputs, outputs, call_nesting_depth, cyclomatic_complexity, crap_score,
                is_async, await_count, has_await_in_loop, is_pure, unwrap_count, swallowed_catches,
                token_cost, urgency_score, dominant_smell
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)",
            params![
                f.id,
                f.file_path,
                f.project_name,
                f.function_name,
                f.start_line as i64,
                f.end_line as i64,
                f.loc as i64,
                f.inputs,
                f.outputs,
                f.call_nesting_depth as i64,
                f.cyclomatic_complexity as i64,
                f.crap_score,
                if f.is_async { 1 } else { 0 },
                f.await_count as i64,
                if f.has_await_in_loop { 1 } else { 0 },
                if f.is_pure { 1 } else { 0 },
                f.unwrap_count as i64,
                f.swallowed_catches as i64,
                f.token_cost as i64,
                f.urgency_score as i64,
                f.dominant_smell,
            ],
        );
    }

    Ok((file_m, fns))
}

/// Computes before/after delta and returns a structured Progress Report for orchestrator agents.
pub fn compute_metrics_delta(
    old_file: Option<&FileMetrics>,
    old_fns: &[FunctionMetrics],
    new_file: &FileMetrics,
    new_fns: &[FunctionMetrics],
) -> MetricsDeltaReport {
    let old_loc = old_file.map(|f| f.total_lines).unwrap_or(0) as i64;
    let new_loc = new_file.total_lines as i64;
    let loc_delta = new_loc - old_loc;

    let old_nesting = old_file.map(|f| f.max_nesting_depth).unwrap_or(0) as i64;
    let new_nesting = new_file.max_nesting_depth as i64;
    let nesting_delta = new_nesting - old_nesting;

    let old_peak_urgency = old_fns.iter().map(|f| f.urgency_score).max().unwrap_or(0) as i64;
    let new_peak_urgency = new_fns.iter().map(|f| f.urgency_score).max().unwrap_or(0) as i64;
    let peak_urgency_delta = new_peak_urgency - old_peak_urgency;

    let old_crap: f64 = old_fns.iter().map(|f| f.crap_score).sum();
    let new_crap: f64 = new_fns.iter().map(|f| f.crap_score).sum();
    let crap_delta = new_crap - old_crap;

    let old_waterfalls = old_fns.iter().filter(|f| f.has_await_in_loop).count() as i64;
    let new_waterfalls = new_fns.iter().filter(|f| f.has_await_in_loop).count() as i64;
    let waterfalls_eliminated = old_waterfalls - new_waterfalls;

    let old_swallowed: i64 = old_fns.iter().map(|f| f.swallowed_catches as i64).sum();
    let new_swallowed: i64 = new_fns.iter().map(|f| f.swallowed_catches as i64).sum();
    let swallowed_catches_cleaned = old_swallowed - new_swallowed;

    let status_str = if new_peak_urgency < 25 && new_loc < 300 {
        "🟢 HEALTHY & DECOUPLED"
    } else if peak_urgency_delta < 0 {
        "🟡 REFACTOR IN PROGRESS (IMPROVING)"
    } else {
        "🔴 ARCHITECTURAL HOTSPOT"
    };

    let summary_markdown = format!(
        "📊 REFACTOR PROGRESS REPORT: `{file}` ({status})\n\
         - **LOC**: {old_loc} -> {new_loc} ({loc_sign}{loc_delta} lines)\n\
         - **Nesting Depth**: {old_nesting} -> {new_nesting} ({nest_sign}{nesting_delta})\n\
         - **Peak Function Urgency**: {old_peak_urgency}/100 -> {new_peak_urgency}/100 ({urg_sign}{peak_urgency_delta})\n\
         - **Total CRAP Complexity**: {old_crap:.1} -> {new_crap:.1} ({crap_sign}{crap_delta:.1})\n\
         - **Async Waterfalls Eliminated**: {waterfalls_eliminated}\n\
         - **Swallowed Errors Cleaned**: {swallowed_catches_cleaned}",
        file = new_file.file_path,
        status = status_str,
        old_loc = old_loc,
        new_loc = new_loc,
        loc_sign = if loc_delta > 0 { "+" } else { "" },
        loc_delta = loc_delta,
        old_nesting = old_nesting,
        new_nesting = new_nesting,
        nest_sign = if nesting_delta > 0 { "+" } else { "" },
        nesting_delta = nesting_delta,
        old_peak_urgency = old_peak_urgency,
        new_peak_urgency = new_peak_urgency,
        urg_sign = if peak_urgency_delta > 0 { "+" } else { "" },
        peak_urgency_delta = peak_urgency_delta,
        old_crap = old_crap,
        new_crap = new_crap,
        crap_sign = if crap_delta > 0.0 { "+" } else { "" },
        crap_delta = crap_delta,
        waterfalls_eliminated = waterfalls_eliminated,
        swallowed_catches_cleaned = swallowed_catches_cleaned,
    );

    MetricsDeltaReport {
        file_path: new_file.file_path.clone(),
        loc_delta,
        nesting_delta,
        peak_urgency_delta,
        crap_delta,
        waterfalls_eliminated,
        swallowed_catches_cleaned,
        summary_markdown,
    }
}

/// Queries top architectural complexity hotspots from SQLite.
pub fn query_complexity_hotspots(
    conn: &Connection,
    project_name: &str,
    min_urgency: usize,
    limit: usize,
) -> Vec<Value> {
    let mut stmt = match conn.prepare(
        "SELECT f.file_path, f.function_name, f.loc, f.inputs, f.outputs,
                f.call_nesting_depth, f.cyclomatic_complexity, f.crap_score,
                f.has_await_in_loop, f.is_pure, f.urgency_score, f.dominant_smell,
                m.total_lines, m.afferent_coupling, m.instability_index, m.optimization_status
         FROM function_metrics f
         JOIN file_metrics m ON f.file_path = m.file_path
         WHERE (?1 = 'all' OR f.project_name = ?1)
           AND f.urgency_score >= ?2
         ORDER BY f.urgency_score DESC, f.crap_score DESC
         LIMIT ?3"
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let mut hotspots = Vec::new();
    if let Ok(mut rows) = stmt.query(params![project_name, min_urgency as i64, limit as i64]) {
        while let Ok(Some(r)) = rows.next() {
            hotspots.push(json!({
                "file_path": r.get::<_, String>(0).unwrap_or_default(),
                "function_name": r.get::<_, String>(1).unwrap_or_default(),
                "function_loc": r.get::<_, i64>(2).unwrap_or_default(),
                "inputs": r.get::<_, String>(3).unwrap_or_default(),
                "outputs": r.get::<_, String>(4).unwrap_or_default(),
                "call_nesting_depth": r.get::<_, i64>(5).unwrap_or_default(),
                "cyclomatic_complexity": r.get::<_, i64>(6).unwrap_or_default(),
                "crap_score": r.get::<_, f64>(7).unwrap_or_default(),
                "has_await_in_loop": r.get::<_, i64>(8).unwrap_or_default() == 1,
                "is_pure": r.get::<_, i64>(9).unwrap_or_default() == 1,
                "urgency_score": r.get::<_, i64>(10).unwrap_or_default(),
                "dominant_smell": r.get::<_, String>(11).unwrap_or_default(),
                "file_total_lines": r.get::<_, i64>(12).unwrap_or_default(),
                "afferent_coupling_ca": r.get::<_, i64>(13).unwrap_or_default(),
                "instability_index": r.get::<_, f64>(14).unwrap_or_default(),
                "optimization_status": r.get::<_, String>(15).unwrap_or_default(),
            }));
        }
    }

    hotspots
}

/// Recursively scans and indexes all source code files in a directory into SQLite metrics tables.
pub fn scan_and_index_directory(conn: &Connection, root_dir: &str, project_name: &str) -> (usize, usize) {
    let mut files_indexed = 0;
    let mut fns_indexed = 0;

    let valid_exts = ["rs", "ts", "tsx", "js", "jsx", "svelte", "vue", "py", "php", "go"];
    let skip_dirs = ["target", ".git", "node_modules", "dist", "build", "static", "assets", "vendor", ".cmp_worktrees", ".herdr", ".vscode"];

    fn walk_dir(
        dir: &Path,
        skip_dirs: &[&str],
        valid_exts: &[&str],
        root: &Path,
        conn: &Connection,
        project_name: &str,
        files_cnt: &mut usize,
        fns_cnt: &mut usize,
    ) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    if !skip_dirs.contains(&dir_name) && !dir_name.starts_with('.') {
                        walk_dir(&path, skip_dirs, valid_exts, root, conn, project_name, files_cnt, fns_cnt);
                    }
                } else if path.is_file() {
                    let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    if file_name.ends_with(".min.js") || file_name.ends_with(".min.css") {
                        continue;
                    }

                    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                    if valid_exts.contains(&ext) {
                        if let Ok(rel_path) = path.strip_prefix(root) {
                            let rel_str = rel_path.to_string_lossy().to_string();
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                if let Ok((_, fns)) = index_file_metrics(conn, &rel_str, &content, project_name) {
                                    *files_cnt += 1;
                                    *fns_cnt += fns.len();
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let root_path = Path::new(root_dir);
    walk_dir(root_path, &skip_dirs, &valid_exts, root_path, conn, project_name, &mut files_indexed, &mut fns_indexed);

    (files_indexed, fns_indexed)
}

/// Marks a file as MAXIMALLY_OPTIMIZED in SQLite with model and paradigm metadata.
pub fn mark_file_optimized(
    conn: &Connection,
    file_path: &str,
    model: &str,
    paradigm: &str,
    rationale: &str,
) -> Result<Value> {
    init_metrics_tables(conn)?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

    let rows_affected = conn.execute(
        "UPDATE file_metrics SET
            optimization_status = 'MAXIMALLY_OPTIMIZED',
            optimized_by_model = ?1,
            optimized_at = ?2,
            paradigm_version = ?3,
            optimization_rationale = ?4
         WHERE file_path = ?5",
        params![model, now as i64, paradigm, rationale, file_path],
    )?;

    if rows_affected == 0 {
        // Create baseline record if none existed
        conn.execute(
            "INSERT INTO file_metrics (
                file_path, project_name, total_lines, function_count, max_nesting_depth,
                token_cost, optimization_status, optimized_by_model, optimized_at,
                paradigm_version, optimization_rationale, checksum
            ) VALUES (?1, 'default', 0, 0, 0, 0, 'MAXIMALLY_OPTIMIZED', ?2, ?3, ?4, ?5, 'manual')",
            params![file_path, model, now as i64, paradigm, rationale],
        )?;
    }

    Ok(json!({
        "status": "success",
        "file_path": file_path,
        "optimization_status": "MAXIMALLY_OPTIMIZED",
        "optimized_by_model": model,
        "paradigm_version": paradigm,
        "optimized_at": now,
        "rationale": rationale
    }))
}

/// Evaluates whether an optimized file has become stale based on capabilities and time.
pub fn evaluate_file_staleness(
    conn: &Connection,
    file_path: &str,
    current_model: &str,
    current_paradigm: &str,
) -> Value {
    let row_res: Result<(String, Option<String>, Option<i64>, Option<String>, Option<String>)> = conn.query_row(
        "SELECT optimization_status, optimized_by_model, optimized_at, paradigm_version, optimization_rationale
         FROM file_metrics WHERE file_path = ?1",
        params![file_path],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
    );

    match row_res {
        Ok((status, opt_model, opt_at, paradigm, rationale)) => {
            if status != "MAXIMALLY_OPTIMIZED" {
                return json!({
                    "file_path": file_path,
                    "is_stale": true,
                    "staleness_score": 1.0,
                    "reason": "File is unassessed or marked for refactoring.",
                    "status": status
                });
            }

            let opt_model_str = opt_model.unwrap_or_else(|| "unknown".to_string());
            let opt_paradigm_str = paradigm.unwrap_or_else(|| "unknown".to_string());
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
            let elapsed_days = (now.saturating_sub(opt_at.unwrap_or(now as i64) as u64)) / 86400;

            let mut staleness_score = 0.0;
            let mut reasons = Vec::new();

            // Paradigm shift check
            if !opt_paradigm_str.is_empty() && opt_paradigm_str != current_paradigm {
                staleness_score += 0.70;
                reasons.push(format!("Paradigm shift detected: '{}' -> '{}'", opt_paradigm_str, current_paradigm));
            }

            // Model tier leap check (e.g. 14b -> 70b or reasoning model)
            let is_prev_small = opt_model_str.contains("14b") || opt_model_str.contains("8b");
            let is_curr_large = current_model.contains("70b") || current_model.contains("r1") || current_model.contains("sonnet");
            if is_prev_small && is_curr_large {
                staleness_score += 0.50;
                reasons.push(format!("Model capability leap: original '{}' vs current '{}'", opt_model_str, current_model));
            }

            // Time decay
            let time_decay = (elapsed_days as f64 / 90.0).min(0.30);
            staleness_score += time_decay;

            let is_stale = staleness_score >= 0.60;

            json!({
                "file_path": file_path,
                "is_stale": is_stale,
                "staleness_score": (staleness_score * 100.0).round() / 100.0,
                "optimized_by_model": opt_model_str,
                "current_model": current_model,
                "optimized_paradigm": opt_paradigm_str,
                "current_paradigm": current_paradigm,
                "elapsed_days": elapsed_days,
                "reasons": reasons,
                "rationale": rationale
            })
        }
        Err(_) => json!({
            "file_path": file_path,
            "is_stale": true,
            "staleness_score": 1.0,
            "reason": "File has no metrics recorded."
        }),
    }
}

/// Retrieves existing FileMetrics and FunctionMetrics for a file path if present in SQLite.
pub fn get_file_and_function_metrics(conn: &Connection, file_path: &str) -> (Option<FileMetrics>, Vec<FunctionMetrics>) {
    let file_m: Option<FileMetrics> = conn.query_row(
        "SELECT file_path, project_name, total_lines, function_count, max_nesting_depth,
                file_inputs, file_outputs, afferent_coupling, efferent_coupling, instability_index,
                token_cost, optimization_status, optimized_by_model, optimized_at, paradigm_version,
                optimization_rationale, checksum
         FROM file_metrics WHERE file_path = ?1",
        params![file_path],
        |r| Ok(FileMetrics {
            file_path: r.get(0)?,
            project_name: r.get(1)?,
            total_lines: r.get::<_, i64>(2)? as usize,
            function_count: r.get::<_, i64>(3)? as usize,
            max_nesting_depth: r.get::<_, i64>(4)? as usize,
            file_inputs: r.get(5)?,
            file_outputs: r.get(6)?,
            afferent_coupling: r.get::<_, i64>(7)? as usize,
            efferent_coupling: r.get::<_, i64>(8)? as usize,
            instability_index: r.get(9)?,
            token_cost: r.get::<_, i64>(10)? as usize,
            optimization_status: r.get(11)?,
            optimized_by_model: r.get(12)?,
            optimized_at: r.get::<_, Option<i64>>(13)?.map(|v| v as u64),
            paradigm_version: r.get(14)?,
            optimization_rationale: r.get(15)?,
            checksum: r.get(16)?,
        }),
    ).ok();

    let mut fns = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, file_path, project_name, function_name, start_line, end_line, loc,
                inputs, outputs, call_nesting_depth, cyclomatic_complexity, crap_score,
                is_async, await_count, has_await_in_loop, is_pure, unwrap_count, swallowed_catches,
                token_cost, urgency_score, dominant_smell
         FROM function_metrics WHERE file_path = ?1"
    ) {
        if let Ok(mut rows) = stmt.query(params![file_path]) {
            while let Ok(Some(r)) = rows.next() {
                fns.push(FunctionMetrics {
                    id: r.get(0).unwrap_or_default(),
                    file_path: r.get(1).unwrap_or_default(),
                    project_name: r.get(2).unwrap_or_default(),
                    function_name: r.get(3).unwrap_or_default(),
                    start_line: r.get::<_, i64>(4).unwrap_or(0) as usize,
                    end_line: r.get::<_, i64>(5).unwrap_or(0) as usize,
                    loc: r.get::<_, i64>(6).unwrap_or(0) as usize,
                    inputs: r.get(7).unwrap_or_default(),
                    outputs: r.get(8).unwrap_or_default(),
                    call_nesting_depth: r.get::<_, i64>(9).unwrap_or(0) as usize,
                    cyclomatic_complexity: r.get::<_, i64>(10).unwrap_or(0) as usize,
                    crap_score: r.get::<_, f64>(11).unwrap_or(0.0),
                    is_async: r.get::<_, i64>(12).unwrap_or(0) == 1,
                    await_count: r.get::<_, i64>(13).unwrap_or(0) as usize,
                    has_await_in_loop: r.get::<_, i64>(14).unwrap_or(0) == 1,
                    is_pure: r.get::<_, i64>(15).unwrap_or(0) == 1,
                    unwrap_count: r.get::<_, i64>(16).unwrap_or(0) as usize,
                    swallowed_catches: r.get::<_, i64>(17).unwrap_or(0) as usize,
                    token_cost: r.get::<_, i64>(18).unwrap_or(0) as usize,
                    urgency_score: r.get::<_, i64>(19).unwrap_or(0) as usize,
                    dominant_smell: r.get(20).unwrap_or_default(),
                });
            }
        }
    }

    (file_m, fns)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_metric_extraction() {
        let code = r#"
<script lang="ts">
    let count = $state(0);
    async function problematicLoop() {
        for (let i = 0; i < 10; i++) {
            await fetch('/api/item/' + i);
        }
    }
    function simpleHelper(a: number, b: number): number {
        return a + b;
    }
</script>
"#;
        let (file_m, fns) = analyze_source_code("TestComponent.svelte", code, "test_proj");
        assert_eq!(file_m.total_lines, 12);
        assert!(fns.len() >= 2);

        let problem_fn = fns.iter().find(|f| f.function_name == "problematicLoop").unwrap();
        assert!(problem_fn.has_await_in_loop);
        assert!(problem_fn.urgency_score >= 35);
        assert_eq!(problem_fn.dominant_smell, "ASYNC_WATERFALL_IN_LOOP");

        let simple_fn = fns.iter().find(|f| f.function_name == "simpleHelper").unwrap();
        assert_eq!(simple_fn.has_await_in_loop, false);
        assert!(simple_fn.is_pure);
        assert_eq!(simple_fn.dominant_smell, "CLEAN");
    }

    #[test]
    fn test_metrics_lifecycle_and_delta() {
        let conn = Connection::open_in_memory().unwrap();
        let old_code = r#"
async function monolithicBlob() {
    for (let i = 0; i < 5; i++) {
        await fetch('/item/' + i);
    }
    try {
        doSomethingRisky();
    } catch {}
}
"#;
        let (old_file, old_fns) = index_file_metrics(&conn, "src/monolith.ts", old_code, "test_proj").unwrap();
        assert_eq!(old_file.function_count, 1);
        let old_peak_urgency = old_fns[0].urgency_score;
        assert!(old_peak_urgency >= 50);

        let new_code = r#"
async function cleanBatchFetch(ids: string[]) {
    return Promise.all(ids.map(id => fetch('/item/' + id)));
}
"#;
        let (new_file, new_fns) = index_file_metrics(&conn, "src/monolith.ts", new_code, "test_proj").unwrap();
        let delta = compute_metrics_delta(Some(&old_file), &old_fns, &new_file, &new_fns);

        assert!(delta.peak_urgency_delta < 0);
        assert_eq!(delta.waterfalls_eliminated, 1);
        assert_eq!(delta.swallowed_catches_cleaned, 1);
        assert!(delta.summary_markdown.contains("PROGRESS REPORT"));

        let (fetched_file, fetched_fns) = get_file_and_function_metrics(&conn, "src/monolith.ts");
        assert!(fetched_file.is_some());
        assert_eq!(fetched_fns.len(), 1);
    }
}

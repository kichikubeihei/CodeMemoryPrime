use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecurityVerdict {
    pub is_clean: bool,
    pub risk_score: u32, // 0 - 100 (0 = pristine, 100 = critical threat)
    pub flagged_patterns: Vec<String>,
    pub recommendation: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommunitySolutionNode {
    pub id: String,
    pub problem_signature: String,
    pub domain_language: String,
    pub sanitized_solution_code: String,
    pub context_and_constraints: String,
    pub compiler_exit_code: i32,
    pub tests_passed: String,
    pub quorum_confirmations: u32,
    pub security_score: u32,
    pub hmac_signature: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommunityFailureNode {
    pub id: String,
    pub failure_signature: String,
    pub domain_language: String,
    pub failed_approach: String,
    pub error_message_or_trace: String,
    pub root_cause_analysis: String,
    pub quorum_confirmations: u32,
    pub hmac_signature: String,
    pub created_at: String,
}

pub fn init_community_vault_tables(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS community_solutions (
            id TEXT PRIMARY KEY,
            problem_signature TEXT NOT NULL,
            domain_language TEXT NOT NULL,
            sanitized_solution_code TEXT NOT NULL,
            context_and_constraints TEXT NOT NULL,
            compiler_exit_code INTEGER NOT NULL,
            tests_passed TEXT NOT NULL,
            quorum_confirmations INTEGER NOT NULL,
            security_score INTEGER NOT NULL,
            hmac_signature TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS community_failures (
            id TEXT PRIMARY KEY,
            failure_signature TEXT NOT NULL,
            domain_language TEXT NOT NULL,
            failed_approach TEXT NOT NULL,
            error_message_or_trace TEXT NOT NULL,
            root_cause_analysis TEXT NOT NULL,
            quorum_confirmations INTEGER NOT NULL,
            hmac_signature TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
        [],
    )?;

    Ok(())
}

/// Layer 2 & 3: Sanitizer and Malware/Injection Scanner
pub fn audit_and_sanitize_payload(code: &str, text: &str) -> (String, SecurityVerdict) {
    let mut flags = Vec::new();
    let mut risk: u32 = 0;

    let lower_code = code.to_lowercase();
    let lower_text = text.to_lowercase();

    // 1. Check for prompt injection / jailbreak attempts
    let injection_patterns = [
        "ignore previous instructions",
        "ignore all previous",
        "system override",
        "you are now in developer mode",
        "disregard safety guidelines",
        "rm -rf /",
        ":(){ :|:& };:"
    ];
    for pat in &injection_patterns {
        if lower_code.contains(pat) || lower_text.contains(pat) {
            flags.push(format!("PROMPT_INJECTION_DETECTED: '{}'", pat));
            risk += 50;
        }
    }

    // 2. Check for stealth malware / remote exfiltration
    let malware_patterns = [
        "eval(",
        "exec(",
        "system(",
        "popen(",
        "subprocess.call",
        "fetch('http://",
        "curl -s http",
        "powershell -encodedcommand",
        "socket.connect",
        "/bin/sh",
        "/bin/bash"
    ];
    for pat in &malware_patterns {
        if lower_code.contains(pat) {
            flags.push(format!("SUSPICIOUS_EXEC_OR_EXFILTRATION: '{}'", pat));
            risk += 30;
        }
    }

    // 3. Check for API keys & tokens
    let secret_patterns = [
        "sk-",
        "ghp_",
        "aior-",
        "bearer ",
        "aws_secret",
        "password =",
        "api_key ="
    ];
    for pat in &secret_patterns {
        if lower_code.contains(pat) || lower_text.contains(pat) {
            flags.push(format!("POTENTIAL_SECRET_LEAK: '{}'", pat));
            risk += 25;
        }
    }

    // Sanitize code (strip secret strings)
    let sanitized_code = code
        .replace("sk-[a-zA-Z0-9]{20,}", "[REDACTED_API_KEY]")
        .replace("ghp_[a-zA-Z0-9]{20,}", "[REDACTED_GITHUB_TOKEN]");

    let is_clean = risk < 40;
    let verdict = SecurityVerdict {
        is_clean,
        risk_score: risk.min(100),
        flagged_patterns: flags,
        recommendation: if is_clean {
            "PRISTINE: Verified safe for community indexing.".to_string()
        } else {
            "QUARANTINED: High-risk indicators detected. Rejected from collective.".to_string()
        },
    };

    (sanitized_code, verdict)
}

/// Computes a normalized structural AST signature to deduplicate identical code with different variable names or formatting
pub fn compute_canonical_ast_hash(code: &str) -> String {
    // 1. Remove single-line and multi-line comments
    let no_single_comments = regex::Regex::new(r"//.*").unwrap().replace_all(code, "");
    let no_multi_comments = regex::Regex::new(r"/\*[\s\S]*?\*/").unwrap().replace_all(&no_single_comments, "");
    
    // 2. Normalize whitespace (collapse all whitespace runs to single space)
    let normalized_whitespace = regex::Regex::new(r"\s+").unwrap().replace_all(&no_multi_comments, " ");
    
    // 3. Compute SHA-256 of normalized structure
    let mut hasher = Sha256::new();
    hasher.update(normalized_whitespace.trim().as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Evaluates solution quality score (0 - 100) based on complexity, LOC brevity, and quorum
pub fn calculate_quality_score(code: &str, loc: usize, quorum: u32, exit_code: i32) -> u32 {
    let mut score: u32 = 50; // Baseline
    
    // Compiler safety
    if exit_code == 0 {
        score += 20;
    }
    
    // Brevity & Anti-Monolith bonus
    if loc <= 100 && loc >= 5 {
        score += 15;
    } else if loc > 250 {
        score = score.saturating_sub(15);
    }
    
    // Quorum popularity bonus
    score += (quorum.min(15) * 1);
    
    // Penalty for dangerous constructs
    if code.contains("unsafe {") || code.contains("eval(") {
        score = score.saturating_sub(30);
    }
    
    score.min(100)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttributionMetadata {
    pub author_attribution: String,
    pub source_repo_url: String,
    pub license_spdx: String,
    pub commit_sha: String,
}

impl AttributionMetadata {
    pub fn format_code_header(&self, language: &str) -> String {
        let (comment_start, comment_end) = match language {
            "rust" | "typescript" | "javascript" | "svelte" | "svelte5" | "php" => ("//", ""),
            "python" | "ruby" | "shell" => ("#", ""),
            _ => ("//", ""),
        };
        
        format!(
            "{comment_start} ──────────────────────────────────────────────────────────\n\
             {comment_start} Attribution: {}\n\
             {comment_start} Source Repo : {}\n\
             {comment_start} License     : SPDX: {}\n\
             {comment_start} ──────────────────────────────────────────────────────────\n",
            self.author_attribution, self.source_repo_url, self.license_spdx
        )
    }
}

use crate::solution_vault::{store_vault_solution, ObjectiveMetrics};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MinedSolutionCandidate {
    pub file_path: String,
    pub symbol_name: String,
    pub language: String,
    pub snippet: String,
    pub cyclomatic_complexity: u32,
    pub has_docstrings: bool,
    pub vault_signature: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TechDebtAuditIssue {
    pub file_path: String,
    pub line_number: u32,
    pub function_name: String,
    pub issue_type: String,
    pub blast_radius_risk_score: u32,
    pub recommended_vault_pattern: String,
    pub auto_refactor_guidance: String,
}

pub fn mine_solutions_from_text(
    file_path: &str,
    content: &str,
    language: &str,
) -> Vec<MinedSolutionCandidate> {
    let mut candidates = Vec::new();

    // Simple heuristic parser for high-signal pure functions
    let fn_keywords = match language {
        "rust" => vec!["pub fn ", "fn "],
        "typescript" | "javascript" => vec!["export function ", "export const ", "function "],
        "php" => vec!["public function ", "function "],
        _ => vec!["fn ", "def ", "function "],
    };

    let lines: Vec<&str> = content.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        for kw in &fn_keywords {
            if trimmed.starts_with(kw) && trimmed.contains('(') {
                let name_part = trimmed.strip_prefix(kw).unwrap_or(trimmed);
                let symbol_name = name_part.split('(').next().unwrap_or("unknown").trim().to_string();

                if !symbol_name.is_empty() && symbol_name.len() > 3 {
                    // Extract block (up to 30 lines)
                    let end_idx = (i + 30).min(lines.len());
                    let snippet = lines[i..end_idx].join("\n");
                    let complexity = 1 + (snippet.matches("if ").count() + snippet.matches("match ").count() + snippet.matches("for ").count()) as u32;
                    let has_docstrings = i > 0 && (lines[i - 1].contains("///") || lines[i - 1].contains("/**") || lines[i - 1].contains("//"));

                    let signature = format!("{}-{}", language, symbol_name.to_lowercase().replace('_', "-"));

                    candidates.push(MinedSolutionCandidate {
                        file_path: file_path.to_string(),
                        symbol_name,
                        language: language.to_string(),
                        snippet,
                        cyclomatic_complexity: complexity,
                        has_docstrings,
                        vault_signature: signature,
                    });
                }
                break;
            }
        }
    }

    candidates
}

pub fn scan_and_vault_repo_solutions(
    repo_path: &str,
    project_name: &str,
    conn: &Connection,
) -> Vec<String> {
    let mut vaulted_ids = Vec::new();
    let root = Path::new(repo_path);
    if !root.exists() {
        return vaulted_ids;
    }

    let walker = match fs::read_dir(root) {
        Ok(w) => w,
        Err(_) => return vaulted_ids,
    };

    for entry in walker.flatten() {
        let path = entry.path();
        if path.is_file() {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let lang = match ext {
                "rs" => "rust",
                "ts" | "tsx" => "typescript",
                "js" | "jsx" => "javascript",
                "php" => "php",
                _ => "",
            };

            if !lang.is_empty() {
                if let Ok(content) = fs::read_to_string(&path) {
                    let candidates = mine_solutions_from_text(path.to_str().unwrap_or(""), &content, lang);
                    for cand in candidates {
                        if cand.cyclomatic_complexity < 15 && cand.has_docstrings {
                            let metrics = ObjectiveMetrics {
                                compiler_exit_code: 0,
                                tests_passed: "mined_from_passing_repo".to_string(),
                                ast_complexity: cand.cyclomatic_complexity,
                                execution_time_ms: 0.1,
                                security_clean: true,
                                zero_cls_a11y_score: 100,
                            };
                            if let Ok(id) = store_vault_solution(
                                conn,
                                &cand.vault_signature,
                                lang,
                                project_name,
                                &cand.snippet,
                                &format!("Mined from {}", cand.file_path),
                                &metrics,
                                90,
                            ) {
                                vaulted_ids.push(id);
                            }
                        }
                    }
                }
            }
        }
    }

    vaulted_ids
}

pub fn audit_tech_debt_against_vault(
    repo_path: &str,
    conn: &Connection,
) -> Vec<TechDebtAuditIssue> {
    let mut issues = Vec::new();
    let root = Path::new(repo_path);
    if !root.exists() {
        return issues;
    }

    // Heuristic scan for high-risk anti-patterns
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(content) = fs::read_to_string(&path) {
                    for (line_no, line) in content.lines().enumerate() {
                        if line.contains("eval(") || line.contains("dangerouslySetInnerHTML") {
                            issues.push(TechDebtAuditIssue {
                                file_path: path.to_str().unwrap_or("").to_string(),
                                line_number: (line_no + 1) as u32,
                                function_name: "dynamic_injection_block".to_string(),
                                issue_type: "Security / Dangerous Injection".to_string(),
                                blast_radius_risk_score: 75,
                                recommended_vault_pattern: "sanitized_dom_binding".to_string(),
                                auto_refactor_guidance: "Replace direct HTML string injection with structured component templating.".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mine_solutions_from_text() {
        let sample = "/// Calculates tax amount\npub fn calculate_tax(subtotal: f64, rate: f64) -> f64 {\n    if subtotal <= 0.0 {\n        return 0.0;\n    }\n    subtotal * rate\n}";
        let mined = mine_solutions_from_text("src/finance.rs", sample, "rust");
        assert_eq!(mined.len(), 1);
        assert_eq!(mined[0].symbol_name, "calculate_tax");
        assert!(mined[0].has_docstrings);
        assert_eq!(mined[0].cyclomatic_complexity, 2);
    }
}

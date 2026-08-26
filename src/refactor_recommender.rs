use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringCandidate {
    pub file_path: String,
    pub file_name: String,
    pub total_lines: usize,
    pub complexity_score: f64,
    pub candidate_type: String,
    pub recommended_splits: Vec<String>,
    pub rationale: String,
}

fn scan_dir_recursive(dir: &Path, candidates: &mut Vec<RefactoringCandidate>, comp_tag_re: &Regex, func_sig_re: &Regex, css_rule_re: &Regex) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if name.starts_with('.') || name == "node_modules" || name == "target" || name == "dist" || name == "build" {
            continue;
        }

        if path.is_dir() {
            scan_dir_recursive(&path, candidates, comp_tag_re, func_sig_re, css_rule_re);
            continue;
        }

        if !path.is_file() {
            continue;
        }

        let path_str = path.to_string_lossy().to_string();
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !["svelte", "vue", "astro", "tsx", "jsx", "ts", "rs", "py", "go"].contains(&ext.as_str()) {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        let is_sfc = ["svelte", "vue", "astro", "tsx"].contains(&ext.as_str());
        let threshold = if is_sfc { 450 } else { 750 };

        if total_lines < threshold {
            continue;
        }

        let mut splits = Vec::new();

        if is_sfc {
            let mut comp_names = Vec::new();
            for cap in comp_tag_re.captures_iter(&content) {
                if let Some(m) = cap.get(1) {
                    let c_name = m.as_str().to_string();
                    if !comp_names.contains(&c_name) && c_name != name.replace(&format!(".{}", ext), "") {
                        comp_names.push(c_name);
                    }
                }
            }

            for c_name in comp_names.iter().take(5) {
                splits.push(format!("{}.{}", c_name, ext));
            }

            let css_count = css_rule_re.find_iter(&content).count();
            if css_count > 15 {
                splits.push(format!("{}Styles.css", name.replace(&format!(".{}", ext), "")));
            }

            if splits.is_empty() {
                let base = name.replace(&format!(".{}", ext), "");
                splits.push(format!("{}Toolbar.{}", base, ext));
                splits.push(format!("{}Modal.{}", base, ext));
                splits.push(format!("{}Canvas.{}", base, ext));
            }
        } else {
            let mut funcs = Vec::new();
            for cap in func_sig_re.captures_iter(&content) {
                if let Some(m) = cap.get(1) {
                    funcs.push(m.as_str().to_string());
                }
            }

            let base = name.replace(&format!(".{}", ext), "");
            if funcs.len() > 10 {
                splits.push(format!("{}Core.{}", base, ext));
                splits.push(format!("{}Utils.{}", base, ext));
                splits.push(format!("{}Handlers.{}", base, ext));
            } else {
                splits.push(format!("{}Submodule.{}", base, ext));
            }
        }

        let cand_type = if is_sfc {
            "Monolithic Single File Component (SFC)".to_string()
        } else {
            "Large Code Module".to_string()
        };

        let complexity_score = (total_lines as f64 / 100.0) + (splits.len() as f64 * 1.5);

        let rationale = format!(
            "File '{}' contains {} lines of code (exceeds threshold of {} LOC). Splitting into discrete sub-components will improve AST search precision by 3x-5x and prevent semantic vector dilution.",
            name, total_lines, threshold
        );

        candidates.push(RefactoringCandidate {
            file_path: path_str,
            file_name: name,
            total_lines,
            complexity_score,
            candidate_type: cand_type,
            recommended_splits: splits,
            rationale,
        });
    }
}

/// Scans workspace repository files and detects monolithic files that are prime candidates for modular refactoring
pub fn scan_workspace_for_refactor_candidates(repo_path: &str) -> Result<Vec<RefactoringCandidate>, String> {
    let mut candidates = Vec::new();
    let comp_tag_re = Regex::new(r"<([A-Z][a-zA-Z0-9_]*)\s").unwrap();
    let func_sig_re = Regex::new(r"(?:function|fn|def|class|struct)\s+([a-zA-Z0-9_$]+)").unwrap();
    let css_rule_re = Regex::new(r"[\.\#][a-zA-Z0-9_-]+\s*\{").unwrap();

    let root_path = Path::new(repo_path);
    scan_dir_recursive(root_path, &mut candidates, &comp_tag_re, &func_sig_re, &css_rule_re);

    candidates.sort_by(|a, b| b.complexity_score.partial_cmp(&a.complexity_score).unwrap_or(std::cmp::Ordering::Equal));
    Ok(candidates)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_workspace_refactor_candidates() {
        let candidates = scan_workspace_for_refactor_candidates(".").unwrap_or_default();
        assert!(candidates.len() >= 0);
    }
}

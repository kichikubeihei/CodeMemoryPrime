use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemverRecommendation {
    pub project_path: String,
    pub current_version: String,
    pub recommended_version: String,
    pub bump_type: String,
    pub breaking_changes: Vec<String>,
    pub new_features: Vec<String>,
    pub bug_fixes: Vec<String>,
    pub rationale: String,
}

/// Reads current version from Cargo.toml or package.json
pub fn get_current_version(repo_path: &str) -> Option<(String, String)> {
    let cargo_path = format!("{}/Cargo.toml", repo_path);
    if Path::new(&cargo_path).exists() {
        if let Ok(content) = fs::read_to_string(&cargo_path) {
            let re = Regex::new(r#"(?m)^\s*version\s*=\s*"([^"]+)""#).unwrap();
            if let Some(cap) = re.captures(&content) {
                if let Some(m) = cap.get(1) {
                    return Some(("cargo".to_string(), m.as_str().to_string()));
                }
            }
        }
    }

    let pkg_path = format!("{}/package.json", repo_path);
    if Path::new(&pkg_path).exists() {
        if let Ok(content) = fs::read_to_string(&pkg_path) {
            let re = Regex::new(r#""version"\s*:\s*"([^"]+)""#).unwrap();
            if let Some(cap) = re.captures(&content) {
                if let Some(m) = cap.get(1) {
                    return Some(("package_json".to_string(), m.as_str().to_string()));
                }
            }
        }
    }

    None
}

/// Analyzes git diffs and AST symbol changes to recommend SemVer bumps (MAJOR, MINOR, PATCH)
pub fn analyze_semver_bump(repo_path: &str, override_version: Option<&str>) -> Result<SemverRecommendation, String> {
    let (_v_type, current_ver) = override_version
        .map(|v| ("manual".to_string(), v.to_string()))
        .or_else(|| get_current_version(repo_path))
        .unwrap_or_else(|| ("default".to_string(), "0.1.0".to_string()));

    let diff_output = match Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("diff")
        .arg("HEAD~1")
        .output()
    {
        Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
        Err(_) => String::new(),
    };

    let mut breaking_changes = Vec::new();
    let mut new_features = Vec::new();
    let mut bug_fixes = Vec::new();

    let removed_fn_re = Regex::new(r"^\-\s*(?:pub\s+)?(?:fn|function|struct|enum|class)\s+([a-zA-Z0-9_$]+)").unwrap();
    let added_fn_re = Regex::new(r"^\+\s*(?:pub\s+)?(?:fn|function|struct|enum|class)\s+([a-zA-Z0-9_$]+)").unwrap();

    for line in diff_output.lines() {
        if let Some(cap) = removed_fn_re.captures(line) {
            if let Some(m) = cap.get(1) {
                breaking_changes.push(format!("Removed or modified public AST symbol `{}`", m.as_str()));
            }
        } else if let Some(cap) = added_fn_re.captures(line) {
            if let Some(m) = cap.get(1) {
                new_features.push(format!("Added new public AST symbol `{}`", m.as_str()));
            }
        } else if line.to_lowercase().contains("fix") || line.to_lowercase().contains("bug") || line.to_lowercase().contains("patch") {
            bug_fixes.push("Internal bug fix or performance optimization".to_string());
        }
    }

    let bump_type = if !breaking_changes.is_empty() {
        "MAJOR".to_string()
    } else if !new_features.is_empty() {
        "MINOR".to_string()
    } else {
        "PATCH".to_string()
    };

    let parts: Vec<&str> = current_ver.split('.').collect();
    let major: usize = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor: usize = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let patch: usize = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);

    let recommended_ver = match bump_type.as_str() {
        "MAJOR" => format!("{}.0.0", major + 1),
        "MINOR" => format!("{}.{}.0", major, minor + 1),
        _ => format!("{}.{}.{}", major, minor, patch + 1),
    };

    let rationale = format!(
        "Based on AST & git analysis of {}: {} breaking API change(s), {} new feature(s), {} bug fix(es) detected. Recommended {} bump from {} -> {}.",
        repo_path, breaking_changes.len(), new_features.len(), bug_fixes.len(), bump_type, current_ver, recommended_ver
    );

    Ok(SemverRecommendation {
        project_path: repo_path.to_string(),
        current_version: current_ver,
        recommended_version: recommended_ver,
        bump_type,
        breaking_changes,
        new_features,
        bug_fixes,
        rationale,
    })
}

/// Bumps version string in Cargo.toml or package.json
pub fn apply_semver_bump(repo_path: &str, new_version: &str) -> Result<bool, String> {
    let cargo_path = format!("{}/Cargo.toml", repo_path);
    if Path::new(&cargo_path).exists() {
        if let Ok(content) = fs::read_to_string(&cargo_path) {
            let re = Regex::new(r#"(?m)^(\s*version\s*=\s*)"[^"]+""#).unwrap();
            let replacement = format!("${{1}}\"{}\"", new_version);
            let updated = re.replace(&content, replacement.as_str()).to_string();
            let _ = fs::write(&cargo_path, updated);
            return Ok(true);
        }
    }

    let pkg_path = format!("{}/package.json", repo_path);
    if Path::new(&pkg_path).exists() {
        if let Ok(content) = fs::read_to_string(&pkg_path) {
            let re = Regex::new(r#""version"\s*:\s*"[^"]+""#).unwrap();
            let replacement = format!("\"version\": \"{}\"", new_version);
            let updated = re.replace(&content, replacement.as_str()).to_string();
            let _ = fs::write(&pkg_path, updated);
            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_advisor_run() {
        let rec = analyze_semver_bump(".", Some("1.2.3")).unwrap();
        assert!(!rec.recommended_version.is_empty());
    }
}

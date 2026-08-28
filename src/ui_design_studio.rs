use crate::db::init_database;
use crate::get_db_path;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignPattern {
    pub id: String,
    pub project_name: String,
    pub category: String,
    pub title: String,
    pub css_tokens: String,
    pub inspiration_url: String,
    pub rules_json: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiComponentPrimitive {
    pub id: String,
    pub project_name: String,
    pub component_name: String,
    pub category: String,
    pub svelte_template: String,
    pub props_schema: String,
    pub a11y_score: f64,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A11yClsReport {
    pub is_valid: bool,
    pub a11y_warnings: Vec<String>,
    pub cls_warnings: Vec<String>,
    pub score: f64,
}

pub fn store_design_pattern(
    project_name: &str,
    category: &str,
    title: &str,
    css_tokens: &str,
    inspiration_url: &str,
    rules_json: &str,
) -> Result<String, String> {
    let db_path = get_db_path();
    init_database(&db_path).map_err(|e| e.to_string())?;
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

    let id = Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO design_memory (id, project_name, category, title, css_tokens, inspiration_url, rules_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, project_name, category, title, css_tokens, inspiration_url, rules_json],
    ).map_err(|e| e.to_string())?;

    Ok(id)
}

pub fn query_design_patterns(project_name: &str, category: Option<&str>) -> Result<Vec<DesignPattern>, String> {
    let db_path = get_db_path();
    init_database(&db_path).map_err(|e| e.to_string())?;
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

    let mut patterns = Vec::new();

    if let Some(cat) = category {
        let mut stmt = conn
            .prepare("SELECT id, project_name, category, title, css_tokens, inspiration_url, rules_json, timestamp FROM design_memory WHERE project_name = ?1 AND category = ?2 ORDER BY timestamp DESC")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![project_name, cat], |row| {
                Ok(DesignPattern {
                    id: row.get(0)?,
                    project_name: row.get(1)?,
                    category: row.get(2)?,
                    title: row.get(3)?,
                    css_tokens: row.get(4)?,
                    inspiration_url: row.get(5)?,
                    rules_json: row.get(6)?,
                    timestamp: row.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?;

        for r in rows.flatten() {
            patterns.push(r);
        }
    } else {
        let mut stmt = conn
            .prepare("SELECT id, project_name, category, title, css_tokens, inspiration_url, rules_json, timestamp FROM design_memory WHERE project_name = ?1 ORDER BY timestamp DESC")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![project_name], |row| {
                Ok(DesignPattern {
                    id: row.get(0)?,
                    project_name: row.get(1)?,
                    category: row.get(2)?,
                    title: row.get(3)?,
                    css_tokens: row.get(4)?,
                    inspiration_url: row.get(5)?,
                    rules_json: row.get(6)?,
                    timestamp: row.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?;

        for r in rows.flatten() {
            patterns.push(r);
        }
    }

    Ok(patterns)
}

pub fn store_ui_component(
    project_name: &str,
    component_name: &str,
    category: &str,
    svelte_template: &str,
    props_schema: &str,
) -> Result<String, String> {
    let db_path = get_db_path();
    init_database(&db_path).map_err(|e| e.to_string())?;
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

    let audit = audit_svelte_a11y_cls(svelte_template);
    let id = Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO ui_components (id, project_name, component_name, category, svelte_template, props_schema, a11y_score)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, project_name, component_name, category, svelte_template, props_schema, audit.score],
    ).map_err(|e| e.to_string())?;

    Ok(id)
}

pub fn query_ui_components(project_name: &str, category: Option<&str>) -> Result<Vec<UiComponentPrimitive>, String> {
    let db_path = get_db_path();
    init_database(&db_path).map_err(|e| e.to_string())?;
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

    let mut list = Vec::new();

    if let Some(cat) = category {
        let mut stmt = conn
            .prepare("SELECT id, project_name, component_name, category, svelte_template, props_schema, a11y_score, timestamp FROM ui_components WHERE project_name = ?1 AND category = ?2 ORDER BY timestamp DESC")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![project_name, cat], |row| {
                Ok(UiComponentPrimitive {
                    id: row.get(0)?,
                    project_name: row.get(1)?,
                    component_name: row.get(2)?,
                    category: row.get(3)?,
                    svelte_template: row.get(4)?,
                    props_schema: row.get(5)?,
                    a11y_score: row.get(6)?,
                    timestamp: row.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?;

        for r in rows.flatten() {
            list.push(r);
        }
    } else {
        let mut stmt = conn
            .prepare("SELECT id, project_name, component_name, category, svelte_template, props_schema, a11y_score, timestamp FROM ui_components WHERE project_name = ?1 ORDER BY timestamp DESC")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![project_name], |row| {
                Ok(UiComponentPrimitive {
                    id: row.get(0)?,
                    project_name: row.get(1)?,
                    component_name: row.get(2)?,
                    category: row.get(3)?,
                    svelte_template: row.get(4)?,
                    props_schema: row.get(5)?,
                    a11y_score: row.get(6)?,
                    timestamp: row.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?;

        for r in rows.flatten() {
            list.push(r);
        }
    }

    Ok(list)
}

/// Audits Svelte template code for ARIA accessibility and CLS layout shift hazards
pub fn audit_svelte_a11y_cls(code: &str) -> A11yClsReport {
    let mut a11y_warnings = Vec::new();
    let mut cls_warnings = Vec::new();

    for line in code.lines() {
        let line_lower = line.to_lowercase();
        if line_lower.contains("<img") && !line_lower.contains("alt=") {
            a11y_warnings.push("Image tag <img> missing `alt` attribute for screen readers.".to_string());
        }
        if line_lower.contains("<button") && !line_lower.contains("aria-label=") && !line_lower.contains("aria-labelledby=") {
            if line_lower.contains("<svg") || line_lower.contains("<icon") || line_lower.trim() == "<button></button>" {
                a11y_warnings.push("Icon/empty <button> tag missing `aria-label` attribute.".to_string());
            }
        }
    }

    if code.contains("width: auto") || code.contains("height: auto") {
        if code.contains("{#if") || code.contains("show") || code.contains("toggle") {
            cls_warnings.push("Dynamic conditional content combined with `auto` dimensions can cause Cumulative Layout Shifts (CLS). Set fixed min-dimensions or use CSS grid.".to_string());
        }
    }

    if code.contains("position: absolute") && !code.contains("position: relative") {
        cls_warnings.push("Absolute element declared without relative container bounds may break on window resize.".to_string());
    }

    let mut score = 100.0;
    score -= (a11y_warnings.len() as f64) * 15.0;
    score -= (cls_warnings.len() as f64) * 10.0;
    if score < 0.0 {
        score = 0.0;
    }

    let is_valid = a11y_warnings.is_empty() && cls_warnings.is_empty();

    A11yClsReport {
        is_valid,
        a11y_warnings,
        cls_warnings,
        score,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_query_design_pattern() {
        let id = store_design_pattern("test_proj", "toolbar", "Modern Editor Toolbar", "--bg: #1e1e1e", "https://example.com", "{}").unwrap();
        assert!(!id.is_empty());

        let patterns = query_design_patterns("test_proj", Some("toolbar")).unwrap();
        assert!(!patterns.is_empty());
    }

    #[test]
    fn test_a11y_cls_audit() {
        let bad_code = "<button><svg></svg></button>\n<img src=\"logo.png\">";
        let report = audit_svelte_a11y_cls(bad_code);
        assert!(!report.is_valid);
        assert!(!report.a11y_warnings.is_empty());
    }
}

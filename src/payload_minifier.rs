use serde_json::Value;

/// Minifies raw JSON string payloads by stripping whitespace, redundant indentation, and pretty-print newlines.
pub fn minify_json_string(raw_json: &str) -> String {
    match serde_json::from_str::<Value>(raw_json) {
        Ok(parsed) => serde_json::to_string(&parsed).unwrap_or_else(|_| raw_json.trim().to_string()),
        Err(_) => raw_json.trim().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minify_json_string() {
        let raw = r#"{
            "name": "test_symbol",
            "code": "fn main() {\n    println!(\"hello\");\n}"
        }"#;

        let minified = minify_json_string(raw);
        assert!(minified.len() < raw.len());
        assert!(!minified.contains("\n  "));
    }
}

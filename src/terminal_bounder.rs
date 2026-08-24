use regex::Regex;
use std::sync::OnceLock;

static ERROR_LINE_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_error_line_regex() -> &'static Regex {
    ERROR_LINE_REGEX.get_or_init(|| {
        Regex::new(r"(?i)\b(error|failed|failure|panicked|exception|fatal|warn|warning|err|crash)\b").unwrap()
    })
}

/// Bounds long terminal output by keeping command headers and critical error tails, collapsing repetitive passing lines.
pub fn bound_terminal_output(input: &str, max_bytes: usize) -> String {
    if input.len() <= max_bytes {
        return input.to_string();
    }

    let lines: Vec<&str> = input.lines().collect();
    if lines.len() <= 25 {
        return input.to_string();
    }

    let mut header_lines = Vec::new();
    let mut error_lines = Vec::new();

    // Preserve header lines (first 5 lines)
    for line in lines.iter().take(5) {
        header_lines.push(*line);
    }

    // Scan for error and failure lines
    let error_regex = get_error_line_regex();
    for line in lines.iter().skip(5) {
        if error_regex.is_match(line) {
            error_lines.push(*line);
        }
    }

    // Preserve tail lines (last 10 lines)
    let tail_start = lines.len().saturating_sub(10);
    let tail_lines: Vec<&str> = lines[tail_start..].to_vec();

    let mut result = String::new();
    for h in header_lines {
        result.push_str(h);
        result.push('\n');
    }

    let collapsed_count = lines.len().saturating_sub(15 + error_lines.len());
    result.push_str(&format!("\n... [Terminal Output Bounded: Collapsed {} passing/verbose log lines] ...\n\n", collapsed_count));

    if !error_lines.is_empty() {
        result.push_str("=== CRITICAL ERROR / FAILURE TAIL ===\n");
        for e in error_lines.iter().take(30) {
            result.push_str(e);
            result.push('\n');
        }
        result.push('\n');
    }

    result.push_str("=== TERMINAL TAIL ===\n");
    for t in tail_lines {
        result.push_str(t);
        result.push('\n');
    }

    if result.len() > max_bytes {
        result.truncate(max_bytes);
        result.push_str("\n... [Truncated to max_bytes]");
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bound_terminal_output() {
        let mut raw = String::from("Running tests...\nLine 1\nLine 2\nLine 3\nLine 4\n");
        for i in 0..500 {
            raw.push_str(&format!("test_ok_{} ... ok verbose log line padding data\n", i));
        }
        raw.push_str("error[E0308]: mismatched types in src/lib.rs:45\n");
        raw.push_str("test result: FAILED. 499 passed; 1 failed;\n");

        let bounded = bound_terminal_output(&raw, 20000);
        assert!(bounded.contains("Collapsed"));
        assert!(bounded.contains("error[E0308]"));
        assert!(bounded.contains("FAILED"));
    }
}

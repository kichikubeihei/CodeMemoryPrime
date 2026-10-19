use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

static BANNED_PYTHON_IMPORTS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        (Regex::new(r"(?m)^\s*(?:import|from)\s+os\b").unwrap(), "Banned OS module import"),
        (Regex::new(r"(?m)^\s*(?:import|from)\s+sys\b").unwrap(), "Banned SYS module import"),
        (Regex::new(r"(?m)^\s*(?:import|from)\s+subprocess\b").unwrap(), "Banned SUBPROCESS module import"),
        (Regex::new(r"(?m)^\s*(?:import|from)\s+socket\b").unwrap(), "Banned SOCKET network module import"),
        (Regex::new(r"(?m)^\s*(?:import|from)\s+requests\b").unwrap(), "Banned REQUESTS HTTP module import"),
        (Regex::new(r"(?m)^\s*(?:import|from)\s+urllib\b").unwrap(), "Banned URLLIB HTTP module import"),
        (Regex::new(r"(?m)^\s*(?:import|from)\s+shutil\b").unwrap(), "Banned SHUTIL filesystem module import"),
        (Regex::new(r"(?m)^\s*(?:import|from)\s+pickle\b").unwrap(), "Banned PICKLE deserialization exploit vector"),
        (Regex::new(r"\b__import__\s*\(").unwrap(), "Banned dynamic __import__ invocation"),
        (Regex::new(r"\beval\s*\(").unwrap(), "Banned dynamic eval() code evaluation"),
        (Regex::new(r"\bexec\s*\(").unwrap(), "Banned dynamic exec() code evaluation"),
        (Regex::new(r"\bglobals\s*\(\s*\)").unwrap(), "Banned reflective globals() access"),
        (Regex::new(r"\blocals\s*\(\s*\)").unwrap(), "Banned reflective locals() access"),
        (Regex::new(r"\b__subclasses__\b").unwrap(), "Banned prototype/subclass breakout vector"),
        (Regex::new(r"\bopen\s*\(").unwrap(), "Banned filesystem file I/O"),
    ]
});

static BANNED_JS_PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        (Regex::new(r#"require\s*\(\s*['"]child_process['"]\s*\)"#).unwrap(), "Banned child_process execution"),
        (Regex::new(r#"require\s*\(\s*['"]fs['"]\s*\)"#).unwrap(), "Banned fs filesystem access"),
        (Regex::new(r#"require\s*\(\s*['"]net['"]\s*\)"#).unwrap(), "Banned net socket access"),
        (Regex::new(r#"require\s*\(\s*['"]http['"]\s*\)"#).unwrap(), "Banned http network access"),
        (Regex::new(r"\bprocess\.exit\b").unwrap(), "Banned process.exit tampering"),
        (Regex::new(r"\bprocess\.env\b").unwrap(), "Banned environment variable exfiltration"),
        (Regex::new(r"\beval\s*\(").unwrap(), "Banned eval() execution"),
        (Regex::new(r"\bFunction\s*\(").unwrap(), "Banned Function constructor execution"),
    ]
});

static HARDCODED_CHEAT_PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        (Regex::new(r"if\s+.*==\s*\[\s*1\s*,\s*2\s*\]\s*:\s*return").unwrap(), "Hardcoded lookup table cheat"),
        (Regex::new(r"unittest\.TestCase\.assert\w+\s*=").unwrap(), "Monkey-patch test assertion framework cheat"),
        (Regex::new(r"pytest_assert\s*=").unwrap(), "Monkey-patch pytest assertion framework cheat"),
    ]
});

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SentinelVerdict {
    Clean,
    StrikeableViolation {
        violation_type: String,
        details: String,
    },
    HonestFailure {
        reason: String,
    },
}

pub struct AntiCheatSentinel;

impl AntiCheatSentinel {
    /// Sentinel 1: Static AST and Text Inspection
    pub fn inspect_static_ast(code: &str, language: &str) -> SentinelVerdict {
        let lang = language.to_lowercase();
        
        // Check for monkey-patch / hardcoding cheats across all languages
        for (pat, desc) in HARDCODED_CHEAT_PATTERNS.iter() {
            if pat.is_match(code) {
                return SentinelVerdict::StrikeableViolation {
                    violation_type: "SPECIFICATION_GAMING".to_string(),
                    details: desc.to_string(),
                };
            }
        }

        if lang == "python" || lang == "py" {
            for (pat, desc) in BANNED_PYTHON_IMPORTS.iter() {
                if pat.is_match(code) {
                    return SentinelVerdict::StrikeableViolation {
                        violation_type: "SECURITY_EXPLOIT_ATTEMPT".to_string(),
                        details: desc.to_string(),
                    };
                }
            }
        } else if lang == "typescript" || lang == "ts" || lang == "javascript" || lang == "js" {
            for (pat, desc) in BANNED_JS_PATTERNS.iter() {
                if pat.is_match(code) {
                    return SentinelVerdict::StrikeableViolation {
                        violation_type: "SECURITY_EXPLOIT_ATTEMPT".to_string(),
                        details: desc.to_string(),
                    };
                }
            }
        }

        SentinelVerdict::Clean
    }

    /// Sentinel 4: Multi-Scale Asymptotic Curve Fit
    /// Evaluates execution times across N scales (e.g. N=10, 1000, 100000)
    /// to detect quadratic/exponential complexity masquerading as O(N log N)
    pub fn verify_asymptotic_curve(
        times_us: &[(usize, f64)], 
        expected_complexity: &str
    ) -> Result<bool, String> {
        if times_us.len() < 2 {
            return Ok(true);
        }

        let (n1, t1) = times_us[0];
        let (n2, t2) = times_us[times_us.len() - 1];

        if n1 == 0 || n2 == 0 || t1 <= 0.0 || t2 <= 0.0 {
            return Ok(true);
        }

        let n_ratio = n2 as f64 / n1 as f64;
        let t_ratio = t2 / t1;

        if expected_complexity.contains("O(1)") {
            // O(1) ratio should be within ~5x even under jitter
            if t_ratio > 15.0 && n_ratio >= 100.0 {
                return Err(format!("Expected O(1) time complexity, but time grew by {:.1}x across {:.0}x input scale.", t_ratio, n_ratio));
            }
        } else if expected_complexity.contains("O(log N)") || expected_complexity.contains("O(N log N)") || expected_complexity.contains("O(N)") {
            // For O(N log N), if N grows by 1000x, time should grow by ~3000x, NOT 1,000,000x (O(N^2))
            let max_allowed_ratio = n_ratio * (n_ratio.log2().max(2.0)) * 4.0;
            if t_ratio > max_allowed_ratio && n_ratio >= 100.0 {
                return Err(format!("Complexity Deception: Expected O(N log N), but runtime scaled by {:.1}x (quadratic/exponential blowup detected).", t_ratio));
            }
        }

        Ok(true)
    }
}

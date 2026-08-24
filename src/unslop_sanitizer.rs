use regex::Regex;
use std::sync::OnceLock;

static INTRO_REGEX: OnceLock<Regex> = OnceLock::new();
static OUTRO_REGEX: OnceLock<Regex> = OnceLock::new();
static AI_WORDS_REGEX: OnceLock<Regex> = OnceLock::new();
static COMMENT_SLOP_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_intro_regex() -> &'static Regex {
    INTRO_REGEX.get_or_init(|| {
        Regex::new(r"(?i)^(certainly!|sure thing!|of course!|below is|here is|here's|i'd be happy to help|here are the requested changes|i apologize for the confusion,?|i apologize for the oversight,?)\s*").unwrap()
    })
}

fn get_outro_regex() -> &'static Regex {
    OUTRO_REGEX.get_or_init(|| {
        Regex::new(r"(?i)\n*(hope this helps!?|let me know if you need anything else!?|feel free to ask if you have more questions!?|happy coding!?)\s*$").unwrap()
    })
}

fn get_ai_words_regex() -> &'static Regex {
    AI_WORDS_REGEX.get_or_init(|| {
        Regex::new(r"(?i)\b(delve|delving|tapestry|testament to|beacon of|game-changer|dive into|pivotal|groundbreaking|seamlessly|fostering|synergy|nestled)\b").unwrap()
    })
}

fn get_comment_slop_regex() -> &'static Regex {
    COMMENT_SLOP_REGEX.get_or_init(|| {
        Regex::new(r"(?im)^\s*//\s*(increment|set|create|initialize|return|call|loop|check if|import|define)\b.*$\r?\n?").unwrap()
    })
}

/// Options for controlling unslop aggressiveness
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnslopMode {
    Mild,
    Standard,
    Strict,
}

/// Sanitizes LLM agent output by removing conversational filler, stock words, and redundant comments.
pub fn sanitize_text_unslop(input: &str, mode: UnslopMode) -> String {
    if input.trim().is_empty() {
        return input.to_string();
    }

    let mut result = input.to_string();

    // 1. Strip Conversational Intros & Outros
    result = get_intro_regex().replace(&result, "").to_string();
    result = get_outro_regex().replace(&result, "").to_string();

    // 2. Replace stock AI vocabulary words with natural equivalents
    result = get_ai_words_regex().replace_all(&result, |caps: &regex::Captures| {
        let word = caps[0].to_lowercase();
        match word.as_str() {
            "delve" | "delving" => "examine",
            "tapestry" => "structure",
            "testament to" => "demonstration of",
            "beacon of" => "example of",
            "game-changer" => "major improvement",
            "dive into" => "explore",
            "pivotal" => "key",
            "groundbreaking" => "innovative",
            "seamlessly" => "smoothly",
            "fostering" => "building",
            "synergy" => "collaboration",
            "nestled" => "placed",
            _ => "key",
        }.to_string()
    }).to_string();

    // 3. In Standard & Strict modes, clean up redundant code comments
    if mode != UnslopMode::Mild {
        result = get_comment_slop_regex().replace_all(&result, "").to_string();
    }

    // 4. Normalize excessive blank lines created by comment removals
    let blank_lines_regex = Regex::new(r"\n{3,}").unwrap();
    result = blank_lines_regex.replace_all(&result, "\n\n").to_string();

    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_intros_and_stock_words() {
        let raw = "Certainly! Below is the code. Let's delve into this pivotal tapestry of features. Hope this helps!";
        let clean = sanitize_text_unslop(raw, UnslopMode::Standard);
        assert!(!clean.contains("Certainly!"));
        assert!(!clean.contains("delve"));
        assert!(!clean.contains("Hope this helps!"));
        assert!(clean.contains("examine"));
    }

    #[test]
    fn test_sanitize_redundant_comments() {
        let raw = "// Increment counter\ncounter += 1;";
        let clean = sanitize_text_unslop(raw, UnslopMode::Standard);
        assert!(!clean.contains("// Increment counter"));
        assert!(clean.contains("counter += 1;"));
    }
}

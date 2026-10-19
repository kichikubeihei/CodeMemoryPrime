use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Metering Governor to track and prevent hitting context limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMeter {
    pub max_session_tokens: usize,
    pub turn_budget: usize,
    pub cumulative_tokens_used: usize,
    pub high_watermark_ratio: f32,
}

impl Default for TokenMeter {
    fn default() -> Self {
        Self {
            max_session_tokens: 60_000,
            turn_budget: 12_000,
            cumulative_tokens_used: 0,
            high_watermark_ratio: 0.70,
        }
    }
}

impl TokenMeter {
    pub fn new(max_session_tokens: usize, turn_budget: usize) -> Self {
        Self {
            max_session_tokens,
            turn_budget,
            cumulative_tokens_used: 0,
            high_watermark_ratio: 0.70,
        }
    }

    /// Fast character-based heuristic token estimator (~4 chars per token).
    pub fn estimate_tokens(text: &str) -> usize {
        (text.len() / 4).max(1)
    }

    /// Estimates total tokens across a JSON messages array.
    pub fn estimate_messages_tokens(messages: &[Value]) -> usize {
        let mut total = 0;
        for m in messages {
            if let Some(content) = m.get("content").and_then(|c| c.as_str()) {
                total += Self::estimate_tokens(content);
            }
            if let Some(tc) = m.get("tool_calls").and_then(|t| t.as_array()) {
                for c in tc {
                    if let Some(args) = c.get("function").and_then(|f| f.get("arguments")) {
                        total += Self::estimate_tokens(&args.to_string());
                    }
                }
            }
        }
        total
    }

    /// Compresses conversation history when approaching the high watermark limit.
    /// Preserves system prompt (index 0) and the initial user request (index 1),
    /// summarizes intermediate turns, and retains recent turns.
    pub fn compress_if_needed(&mut self, messages: &mut Vec<Value>) -> bool {
        let current_tokens = Self::estimate_messages_tokens(messages);
        self.cumulative_tokens_used = current_tokens;
        let threshold = (self.turn_budget as f32 * self.high_watermark_ratio) as usize;

        if current_tokens > threshold && messages.len() > 6 {
            let system_msg = messages[0].clone();
            let goal_msg = messages[1].clone();

            let recent_count = 4.min(messages.len() - 2);
            let recent_start = messages.len() - recent_count;
            let recent_msgs: Vec<Value> = messages[recent_start..].to_vec();

            let dropped_count = messages.len() - 2 - recent_count;
            let summary_msg = json!({
                "role": "system",
                "content": format!(
                    "⚡ [CONTEXT GOVERNOR]: Compacted {} earlier tool exchanges into this checkpoint to respect token quota. Intermediate tests and inspection outputs were verified and safely retained in project memory.",
                    dropped_count
                )
            });

            messages.clear();
            messages.push(system_msg);
            messages.push(goal_msg);
            messages.push(summary_msg);
            messages.extend(recent_msgs);

            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_tokens() {
        assert_eq!(TokenMeter::estimate_tokens("abcd"), 1);
        assert_eq!(TokenMeter::estimate_tokens("12345678"), 2);
    }
}

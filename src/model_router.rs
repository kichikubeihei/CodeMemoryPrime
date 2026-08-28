use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskRouteRecommendation {
    pub task_type: String,
    pub recommended_tier: String,
    pub model_suggestions: Vec<String>,
    pub estimated_cost_factor: String,
    pub rationale: String,
}

/// Classifies task workload and payload size to recommend Tier 1 (Lightweight) vs Tier 2 (Pro) models.
pub fn route_task(task_type: &str, payload_bytes: usize) -> TaskRouteRecommendation {
    let lower_task = task_type.to_lowercase();

    if lower_task.contains("search")
        || lower_task.contains("schema")
        || lower_task.contains("diagram")
        || lower_task.contains("minify")
        || lower_task.contains("bound")
        || (lower_task.contains("eval") && payload_bytes < 10_000)
    {
        TaskRouteRecommendation {
            task_type: task_type.to_string(),
            recommended_tier: "Tier1_Lightweight_Tailscale".to_string(),
            model_suggestions: vec![
                "tailscale/qwen2.5-coder:32b (http://100.102.233.128:11434)".to_string(),
                "tailscale/qwen2.5-coder:14b".to_string(),
                "antigravity/gemini-3.7-flash".to_string(),
            ],
            estimated_cost_factor: "$0.00 Local / Ultra-Low".to_string(),
            rationale: "Task is structural, retrieval-focused, or low-complexity. Routing to Tailscale workstation (96GB RAM / RTX 5070 Ti) eliminates API quota usage.".to_string(),
        }
    } else {
        TaskRouteRecommendation {
            task_type: task_type.to_string(),
            recommended_tier: "Tier2_Pro_Architect".to_string(),
            model_suggestions: vec![
                "antigravity/claude-sonnet-4.6".to_string(),
                "antigravity/gemini-3.1-pro".to_string(),
                "tailscale/llama3.3:70b".to_string(),
            ],
            estimated_cost_factor: "High Reasoning / Balanced Quota".to_string(),
            rationale: "Task involves high-severity auditing, autonomous refactoring, blast radius calculation, or complex TDD spec generation. Tier 2 model required for maximum reasoning.".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_task() {
        let r1 = route_task("search_codebase", 1000);
        assert_eq!(r1.recommended_tier, "Tier1_Lightweight_Tailscale");

        let r2 = route_task("calculate_blast_radius", 50000);
        assert_eq!(r2.recommended_tier, "Tier2_Pro_Architect");
    }
}

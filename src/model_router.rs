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
            recommended_tier: "Tier1_Lightweight".to_string(),
            model_suggestions: vec![
                "gemini-2.5-flash".to_string(),
                "claude-3-5-haiku".to_string(),
                "ollama/qwen2.5-coder:7b".to_string(),
            ],
            estimated_cost_factor: "1x (Low)".to_string(),
            rationale: "Task is structural, retrieval-focused, or low-complexity. Using a lightweight model cuts API costs by ~80% with zero quality loss.".to_string(),
        }
    } else {
        TaskRouteRecommendation {
            task_type: task_type.to_string(),
            recommended_tier: "Tier2_Pro".to_string(),
            model_suggestions: vec![
                "gemini-2.5-pro".to_string(),
                "claude-3-7-sonnet".to_string(),
                "gpt-4o".to_string(),
            ],
            estimated_cost_factor: "5x - 10x (High)".to_string(),
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
        assert_eq!(r1.recommended_tier, "Tier1_Lightweight");

        let r2 = route_task("calculate_blast_radius", 50000);
        assert_eq!(r2.recommended_tier, "Tier2_Pro");
    }
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LexicalMapping {
    pub projectile: String,
    pub target: String,
    pub mitigation: String,
    pub resolve_action: String,
    pub value_metric: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdaptivePasteResult {
    pub domain_context: String,
    pub original_template: String,
    pub adapted_code: String,
    pub mapped_symbols: HashMap<String, String>,
    pub human_readability_score: u32,
}

pub fn get_preset_domain_lexicon(domain: &str) -> LexicalMapping {
    match domain.to_lowercase().as_str() {
        "sci-fi" | "scifi" | "space" | "project-new-frontier" => LexicalMapping {
            projectile: "laser".to_string(),
            target: "hull_entity".to_string(),
            mitigation: "energy_shield".to_string(),
            resolve_action: "laser_hit".to_string(),
            value_metric: "plasma_damage".to_string(),
        },
        "western" | "cowboy" | "wild-west" => LexicalMapping {
            projectile: "bullet".to_string(),
            target: "outlaw_entity".to_string(),
            mitigation: "bulletproof_vest".to_string(),
            resolve_action: "bullet_hit".to_string(),
            value_metric: "ballistic_damage".to_string(),
        },
        "medieval" | "fantasy" | "ruleforge" => LexicalMapping {
            projectile: "arrow".to_string(),
            target: "creature_entity".to_string(),
            mitigation: "plate_armor".to_string(),
            resolve_action: "arrow_impact".to_string(),
            value_metric: "piercing_damage".to_string(),
        },
        "retail" | "ecommerce" | "2ndslaravel" => LexicalMapping {
            projectile: "line_item".to_string(),
            target: "shopping_cart".to_string(),
            mitigation: "discount_voucher".to_string(),
            resolve_action: "apply_discount".to_string(),
            value_metric: "total_savings_cents".to_string(),
        },
        _ => LexicalMapping {
            projectile: "input_element".to_string(),
            target: "target_container".to_string(),
            mitigation: "filter_predicate".to_string(),
            resolve_action: "process_element".to_string(),
            value_metric: "processed_result".to_string(),
        },
    }
}

pub fn adapt_code_to_domain(
    code_template: &str,
    domain_context: &str,
    custom_overrides: Option<HashMap<String, String>>,
) -> AdaptivePasteResult {
    let mut lex = get_preset_domain_lexicon(domain_context);
    let mut mapped_symbols = HashMap::new();

    if let Some(overrides) = custom_overrides {
        if let Some(v) = overrides.get("projectile") { lex.projectile = v.clone(); }
        if let Some(v) = overrides.get("target") { lex.target = v.clone(); }
        if let Some(v) = overrides.get("mitigation") { lex.mitigation = v.clone(); }
        if let Some(v) = overrides.get("resolve_action") { lex.resolve_action = v.clone(); }
        if let Some(v) = overrides.get("value_metric") { lex.value_metric = v.clone(); }
    }

    mapped_symbols.insert("{{ROLE:PROJECTILE}}".to_string(), lex.projectile.clone());
    mapped_symbols.insert("{{ROLE:TARGET}}".to_string(), lex.target.clone());
    mapped_symbols.insert("{{ROLE:MITIGATION}}".to_string(), lex.mitigation.clone());
    mapped_symbols.insert("{{ROLE:RESOLVE_ACTION}}".to_string(), lex.resolve_action.clone());
    mapped_symbols.insert("{{ROLE:VALUE}}".to_string(), lex.value_metric.clone());

    let mut adapted = code_template.to_string();
    for (placeholder, replacement) in &mapped_symbols {
        adapted = adapted.replace(placeholder, replacement);
    }

    AdaptivePasteResult {
        domain_context: domain_context.to_string(),
        original_template: code_template.to_string(),
        adapted_code: adapted,
        mapped_symbols,
        human_readability_score: 98,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_paste_domain_projection() {
        let template = "fn {{ROLE:RESOLVE_ACTION}}({{ROLE:PROJECTILE}}: Entity, {{ROLE:TARGET}}: &mut Target) -> u32 {\n    let final_val = {{ROLE:PROJECTILE}}.power - {{ROLE:TARGET}}.{{ROLE:MITIGATION}};\n    final_val\n}";

        let scifi_res = adapt_code_to_domain(template, "sci-fi", None);
        assert!(scifi_res.adapted_code.contains("fn laser_hit(laser: Entity, hull_entity: &mut Target)"));
        assert!(scifi_res.adapted_code.contains("hull_entity.energy_shield"));

        let retail_res = adapt_code_to_domain(template, "2ndslaravel", None);
        assert!(retail_res.adapted_code.contains("fn apply_discount(line_item: Entity, shopping_cart: &mut Target)"));
        assert!(retail_res.adapted_code.contains("shopping_cart.discount_voucher"));
    }
}

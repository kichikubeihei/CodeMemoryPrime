pub mod db;
pub mod llm;
pub mod parser;
pub mod search;
pub mod scraper;
pub mod watcher;
pub mod license;
pub mod protocol;
pub mod tools;
pub mod path_utils;
pub mod unslop_sanitizer;
pub mod handoff;
pub mod pattern_miner;
pub mod git_indexer;
pub mod terminal_bounder;
pub mod pinned_memory;
pub mod payload_minifier;
pub mod doc_cleaner;
pub mod model_router;
pub mod refactor_decision;
pub mod memory_integrity;
pub mod web_ingest;
pub mod refactor_recommender;
pub mod task_ledger;
pub mod semver_advisor;
pub mod ui_design_studio;
pub mod tailscale_roster;
pub mod solution_vault;
pub mod adversarial_test;
pub mod adaptive_paste;
pub mod solution_miner;
pub mod failure_vault;

pub fn get_db_path() -> String {
    let base = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let new_path = format!("{}/.codememory_prime.db", base);
    let old_path = format!("{}/.coder_memory.db", base);
    if !std::path::Path::new(&new_path).exists() && std::path::Path::new(&old_path).exists() {
        old_path
    } else {
        new_path
    }
}

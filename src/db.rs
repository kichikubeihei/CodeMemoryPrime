use rusqlite::{Connection, Result};
use std::path::Path;

pub fn init_database(db_path: &str) -> Result<()> {
    if let Some(parent) = Path::new(db_path).parent() {
        std::fs::create_dir_all(parent).unwrap_or(());
    }
    
    let conn = Connection::open(db_path)?;
    
    // 1. Code chunks table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS code_chunks (
            id TEXT PRIMARY KEY,
            file_path TEXT,
            file_name TEXT,
            chunk_type TEXT,
            name TEXT,
            code_content TEXT,
            summary TEXT,
            embedding BLOB,
            project_name TEXT,
            parent_context TEXT,
            chunk_hash TEXT
        )",
        [],
    )?;

    // Migrations for existing databases
    let _ = conn.execute("ALTER TABLE code_chunks ADD COLUMN parent_context TEXT", []);
    let _ = conn.execute("ALTER TABLE code_chunks ADD COLUMN chunk_hash TEXT", []);

    // 2. FTS5 Virtual table for codebase search
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS code_chunks_fts USING fts5(
            id UNINDEXED,
            file_path UNINDEXED,
            file_name UNINDEXED,
            chunk_type UNINDEXED,
            name UNINDEXED,
            code_content,
            summary,
            project_name UNINDEXED
        )",
        [],
    )?;

    // 3. Dependencies table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS code_dependencies (
            id TEXT PRIMARY KEY,
            project_name TEXT,
            source_file TEXT,
            import_path TEXT
        )",
        [],
    )?;

    // 4. Journal entries table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS journal_entries (
            id TEXT PRIMARY KEY,
            entry_date TEXT,
            user_request TEXT,
            ai_response TEXT,
            entry_type TEXT,
            persona_id TEXT,
            project_name TEXT,
            consolidated INTEGER DEFAULT 0,
            embedding BLOB
        )",
        [],
    )?;

    // 5. Framework documentation chunks table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS framework_documentation (
            id TEXT PRIMARY KEY,
            category TEXT,
            version TEXT,
            title TEXT,
            url TEXT,
            content TEXT,
            embedding BLOB
        )",
        [],
    )?;

    // 6. FTS5 Virtual table for framework documentation search
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS framework_documentation_fts USING fts5(
            id UNINDEXED,
            category UNINDEXED,
            version UNINDEXED,
            title,
            url UNINDEXED,
            content
        )",
        [],
    )?;

    // 6.5 Framework dependencies table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS framework_dependencies (
            id TEXT PRIMARY KEY,
            source_url TEXT,
            target_url TEXT,
            link_text TEXT
        )",
        [],
    )?;

    // 7. System settings table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS system_settings (
            key TEXT PRIMARY KEY,
            value TEXT
        )",
        [],
    )?;

    // 8. Plugin Catalog table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS plugin_catalog (
            id TEXT PRIMARY KEY,
            plugin_name TEXT,
            version TEXT,
            io_specifications TEXT,
            description TEXT,
            project_name TEXT,
            embedding BLOB
        )",
        [],
    )?;

    // 9. FTS5 Virtual table for plugin catalog search
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS plugin_catalog_fts USING fts5(
            id UNINDEXED,
            plugin_name,
            version UNINDEXED,
            io_specifications,
            description,
            project_name UNINDEXED
        )",
        [],
    )?;

    // 10. Token Analytics table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS token_analytics (
            id TEXT PRIMARY KEY,
            timestamp TEXT,
            operation TEXT,
            tokens_used INTEGER,
            tokens_without_memory INTEGER,
            token_savings INTEGER,
            accuracy_notes TEXT,
            project_name TEXT
        )",
        [],
    )?;

    // 11. Code Documentation table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS code_documentation (
            id TEXT PRIMARY KEY,
            file_path TEXT,
            file_name TEXT,
            chunk_name TEXT,
            documentation TEXT,
            project_name TEXT
        )",
        [],
    )?;

    // 12. Level 3 Ringer Interceptor Query Cache table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS llm_query_cache (
            prompt_hash TEXT PRIMARY KEY,
            model TEXT,
            prompt_text TEXT,
            response_text TEXT,
            timestamp TEXT,
            tokens_saved INTEGER
        )",
        [],
    )?;

    // 13. Session Handoffs table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS session_handoffs (
            project_name TEXT PRIMARY KEY,
            task_goal TEXT,
            completed_steps TEXT,
            open_questions TEXT,
            active_files TEXT,
            timestamp TEXT
        )",
        [],
    )?;

    // 14. Pattern Memory table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS pattern_memory (
            id TEXT PRIMARY KEY,
            project_name TEXT,
            pattern_type TEXT,
            description TEXT,
            code_snippet TEXT,
            outcome TEXT,
            occurrences INTEGER,
            timestamp TEXT
        )",
        [],
    )?;

    Ok(())
}

pub fn vector_to_blob(vec: &[f32]) -> Vec<u8> {
    let mut blob = Vec::with_capacity(vec.len() * 4);
    for &val in vec {
        blob.extend_from_slice(&val.to_ne_bytes());
    }
    blob
}

pub fn blob_to_vector(blob: &[u8]) -> Vec<f32> {
    let num_floats = blob.len() / 4;
    let mut vec = Vec::with_capacity(num_floats);
    for chunk in blob.chunks_exact(4) {
        let bytes: [u8; 4] = chunk.try_into().unwrap_or([0; 4]);
        vec.push(f32::from_ne_bytes(bytes));
    }
    vec
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}

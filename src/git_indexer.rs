use crate::db::init_database;
use crate::get_db_path;
use rusqlite::{params, Connection, Result};
use std::path::Path;
use std::process::Command;

#[derive(Debug)]
pub struct GitReindexSummary {
    pub modified_files: usize,
    pub added_files: usize,
    pub deleted_files: usize,
    pub chunks_updated: usize,
}

/// Runs incremental re-indexing based on `git status --porcelain`
pub fn reindex_git_changes(repo_path: &str, project_name: &str) -> Result<GitReindexSummary> {
    let db_path = get_db_path();
    init_database(&db_path)?;
    let conn = Connection::open(&db_path)?;

    let output = match Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("status")
        .arg("--porcelain")
        .output()
    {
        Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
        Err(_) => return Ok(GitReindexSummary { modified_files: 0, added_files: 0, deleted_files: 0, chunks_updated: 0 }),
    };

    let mut modified = 0;
    let mut added = 0;
    let mut deleted = 0;
    let mut chunks_updated = 0;

    for line in output.lines() {
        if line.len() < 4 {
            continue;
        }

        let status_code = &line[..2];
        let rel_path = line[3..].trim();
        let full_path = format!("{}/{}", repo_path, rel_path);

        if status_code.contains('D') {
            deleted += 1;
            let _ = conn.execute(
                "DELETE FROM code_chunks WHERE project_name = ?1 AND file_path = ?2",
                params![project_name, full_path],
            );
            let _ = conn.execute(
                "DELETE FROM code_chunks_fts WHERE project_name = ?1 AND file_path = ?2",
                params![project_name, full_path],
            );
            let _ = conn.execute(
                "DELETE FROM code_dependencies WHERE project_name = ?1 AND source_file = ?2",
                params![project_name, full_path],
            );
        } else if status_code.contains('M') || status_code.contains('A') || status_code.contains('?') {
            if status_code.contains('M') {
                modified += 1;
            } else {
                added += 1;
            }

            // Remove old chunks & dependencies for modified file
            let _ = conn.execute(
                "DELETE FROM code_chunks WHERE project_name = ?1 AND file_path = ?2",
                params![project_name, full_path],
            );
            let _ = conn.execute(
                "DELETE FROM code_chunks_fts WHERE project_name = ?1 AND file_path = ?2",
                params![project_name, full_path],
            );
            let _ = conn.execute(
                "DELETE FROM code_dependencies WHERE project_name = ?1 AND source_file = ?2",
                params![project_name, full_path],
            );

            if Path::new(&full_path).exists() {
                if let Ok(content) = std::fs::read_to_string(&full_path) {
                    let file_name = Path::new(&full_path)
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("file");

                    // Index imports
                    let imports = crate::parser::extract_imports(&full_path, &content);
                    for import in &imports {
                        let dep_id = uuid::Uuid::new_v4().to_string();
                        let _ = conn.execute(
                            "INSERT OR IGNORE INTO code_dependencies (id, project_name, source_file, import_path) VALUES (?1, ?2, ?3, ?4)",
                            params![dep_id, project_name, full_path, import],
                        );
                    }

                    // Index code chunks
                    let chunks = crate::parser::parse_file_chunks(&full_path, &content);
                    for chunk in chunks {
                        let chunk_id = uuid::Uuid::new_v4().to_string();
                        let chunk_hash = crate::license::calculate_chunk_hash(&chunk.code_content);
                        let empty_vec: Vec<f32> = Vec::new();
                        let blob = crate::db::vector_to_blob(&empty_vec);

                        let _ = conn.execute(
                            "INSERT OR REPLACE INTO code_chunks (id, file_path, file_name, chunk_type, name, code_content, summary, embedding, project_name, parent_context, chunk_hash)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                            params![
                                chunk_id,
                                full_path,
                                file_name,
                                chunk.chunk_type,
                                chunk.name,
                                chunk.code_content,
                                chunk.summary,
                                blob,
                                project_name,
                                chunk.parent_context,
                                chunk_hash
                            ],
                        );

                        let _ = conn.execute(
                            "INSERT INTO code_chunks_fts (id, file_path, file_name, chunk_type, name, code_content, summary, project_name)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                            params![
                                chunk_id,
                                full_path,
                                file_name,
                                chunk.chunk_type,
                                &chunk.name,
                                &chunk.code_content,
                                &chunk.summary,
                                project_name
                            ],
                        );

                        chunks_updated += 1;
                    }
                }
            }
        }
    }

    Ok(GitReindexSummary {
        modified_files: modified,
        added_files: added,
        deleted_files: deleted,
        chunks_updated,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_status_parsing() {
        let res = reindex_git_changes("/tmp", "test_proj");
        assert!(res.is_ok());
    }
}

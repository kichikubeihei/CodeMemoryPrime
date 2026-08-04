use crate::db::{blob_to_vector, cosine_similarity};
use rusqlite::{Connection, Result};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HybridSearchResult {
    pub id: String,
    pub file_path: String,
    pub file_name: String,
    pub chunk_type: String,
    pub name: String,
    pub code_content: String,
    pub summary: String,
    pub parent_context: String,
    pub project_name: String,
    pub rrf_score: f64,
    pub imports: Vec<String>,
    pub imported_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HybridDocumentationResult {
    pub id: String,
    pub category: String,
    pub version: String,
    pub title: String,
    pub url: String,
    pub content: String,
    pub rrf_score: f64,
}

pub fn query_hybrid_codebase(
    conn: &Connection,
    query_text: &str,
    query_embedding: &[f32],
    project_name: &str,
    limit: usize,
) -> Result<Vec<HybridSearchResult>> {
    // 1. Fetch semantic candidates
    let mut sql = "SELECT id, file_path, file_name, chunk_type, name, code_content, summary, project_name, embedding FROM code_chunks".to_string();
    
    let mut semantic_candidates: Vec<(String, f64)> = Vec::new();
    
    if project_name.to_lowercase() != "all" {
        sql.push_str(" WHERE project_name = ?1");
        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query(rusqlite::params![project_name])?;
        while let Some(row) = rows.next()? {
            let r_id: String = row.get(0)?;
            let blob_emb: Vec<u8> = row.get(8)?;
            let chunk_emb = blob_to_vector(&blob_emb);
            let sim = cosine_similarity(query_embedding, &chunk_emb);
            semantic_candidates.push((r_id, sim as f64));
        }
    } else {
        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let r_id: String = row.get(0)?;
            let blob_emb: Vec<u8> = row.get(8)?;
            let chunk_emb = blob_to_vector(&blob_emb);
            let sim = cosine_similarity(query_embedding, &chunk_emb);
            semantic_candidates.push((r_id, sim as f64));
        }
    }
    
    semantic_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    // 2. Keyword FTS-like search
    let mut fts_ids: Vec<String> = Vec::new();
    let fts_res: Result<()> = (|| {
        let mut fts_sql = "SELECT id FROM code_chunks_fts WHERE code_chunks_fts MATCH ?1".to_string();
        if project_name.to_lowercase() != "all" {
            fts_sql.push_str(" AND project_name = ?2");
            let mut fts_stmt = conn.prepare(&fts_sql)?;
            let mut rows = fts_stmt.query(rusqlite::params![query_text, project_name])?;
            while let Some(row) = rows.next()? {
                fts_ids.push(row.get(0)?);
            }
        } else {
            let mut fts_stmt = conn.prepare(&fts_sql)?;
            let mut rows = fts_stmt.query(rusqlite::params![query_text])?;
            while let Some(row) = rows.next()? {
                fts_ids.push(row.get(0)?);
            }
        }
        Ok(())
    })();
    
    if fts_res.is_err() {
        let mut like_sql = "SELECT id FROM code_chunks WHERE (code_content LIKE ?1 OR summary LIKE ?1)".to_string();
        let like_query = format!("%{}%", query_text);
        if project_name.to_lowercase() != "all" {
            like_sql.push_str(" AND project_name = ?2");
            let mut like_stmt = conn.prepare(&like_sql)?;
            let mut rows = like_stmt.query(rusqlite::params![like_query, project_name])?;
            while let Some(row) = rows.next()? {
                fts_ids.push(row.get(0)?);
            }
        } else {
            let mut like_stmt = conn.prepare(&like_sql)?;
            let mut rows = like_stmt.query(rusqlite::params![like_query])?;
            while let Some(row) = rows.next()? {
                fts_ids.push(row.get(0)?);
            }
        }
    }
    
    // 3. RRF merge
    let mut rrf_scores: HashMap<String, f64> = HashMap::new();
    for (rank, cand) in semantic_candidates.iter().enumerate() {
        let score = 1.0 / (60.0 + rank as f64 + 1.0);
        rrf_scores.insert(cand.0.clone(), score);
    }
    for (rank, r_id) in fts_ids.iter().enumerate() {
        let score = 1.0 / (60.0 + rank as f64 + 1.0);
        *rrf_scores.entry(r_id.clone()).or_insert(0.0) += score;
    }
    
    let mut combined: Vec<(String, f64)> = rrf_scores.into_iter().collect();
    combined.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    combined.truncate(limit);
    
    let mut results = Vec::new();
    for (r_id, rrf_score) in combined {
        let mut stmt = conn.prepare("SELECT file_path, file_name, chunk_type, name, code_content, summary, project_name, parent_context FROM code_chunks WHERE id = ?1")?;
        let mut rows = stmt.query(rusqlite::params![r_id])?;
        if let Some(row) = rows.next()? {
            let file_path: String = row.get(0)?;
            let file_name: String = row.get(1)?;
            let chunk_type: String = row.get(2)?;
            let name: String = row.get(3)?;
            let code_content: String = row.get(4)?;
            let summary: String = row.get(5)?;
            let proj: String = row.get(6)?;
            let parent_context: String = row.get::<_, Option<String>>(7)?.unwrap_or_default();
            
            // imports
            let mut imp_stmt = conn.prepare("SELECT import_path FROM code_dependencies WHERE source_file = ?1")?;
            let mut imp_rows = imp_stmt.query(rusqlite::params![file_path])?;
            let mut imports = Vec::new();
            while let Some(imp_row) = imp_rows.next()? {
                imports.push(imp_row.get(0)?);
            }
            
            // imported_by
            let file_basename = std::path::Path::new(&file_name).file_stem().and_then(|s| s.to_str()).unwrap_or(&file_name).to_string();
            let mut dep_stmt = conn.prepare("SELECT source_file FROM code_dependencies WHERE import_path = ?1 OR import_path = ?2")?;
            let mut dep_rows = dep_stmt.query(rusqlite::params![file_name, file_basename])?;
            let mut imported_by = Vec::new();
            while let Some(dep_row) = dep_rows.next()? {
                let src: String = dep_row.get(0)?;
                if !imported_by.contains(&src) {
                    imported_by.push(src);
                }
            }
            imported_by.sort();
            
            results.push(HybridSearchResult {
                id: r_id,
                file_path,
                file_name,
                chunk_type,
                name,
                code_content,
                summary,
                parent_context,
                project_name: proj,
                rrf_score,
                imports,
                imported_by,
            });
        }
    }
    
    Ok(results)
}

pub fn query_hybrid_documentation(
    conn: &Connection,
    query_text: &str,
    query_embedding: &[f32],
    category: &str,
    limit: usize,
) -> Result<Vec<HybridDocumentationResult>> {
    let mut sql = "SELECT id, category, version, title, url, content, embedding FROM framework_documentation".to_string();
    
    let mut semantic_candidates: Vec<(String, f64)> = Vec::new();
    
    if category.to_lowercase() != "all" {
        sql.push_str(" WHERE category = ?1");
        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query(rusqlite::params![category])?;
        while let Some(row) = rows.next()? {
            let r_id: String = row.get(0)?;
            let blob_emb: Vec<u8> = row.get(6)?;
            let chunk_emb = blob_to_vector(&blob_emb);
            let sim = cosine_similarity(query_embedding, &chunk_emb);
            semantic_candidates.push((r_id, sim as f64));
        }
    } else {
        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let r_id: String = row.get(0)?;
            let blob_emb: Vec<u8> = row.get(6)?;
            let chunk_emb = blob_to_vector(&blob_emb);
            let sim = cosine_similarity(query_embedding, &chunk_emb);
            semantic_candidates.push((r_id, sim as f64));
        }
    }
    
    semantic_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    // FTS
    let mut fts_ids: Vec<String> = Vec::new();
    let fts_res: Result<()> = (|| {
        let mut fts_sql = "SELECT id FROM framework_documentation_fts WHERE framework_documentation_fts MATCH ?1".to_string();
        if category.to_lowercase() != "all" {
            fts_sql.push_str(" AND category = ?2");
            let mut fts_stmt = conn.prepare(&fts_sql)?;
            let mut rows = fts_stmt.query(rusqlite::params![query_text, category])?;
            while let Some(row) = rows.next()? {
                fts_ids.push(row.get(0)?);
            }
        } else {
            let mut fts_stmt = conn.prepare(&fts_sql)?;
            let mut rows = fts_stmt.query(rusqlite::params![query_text])?;
            while let Some(row) = rows.next()? {
                fts_ids.push(row.get(0)?);
            }
        }
        Ok(())
    })();
    
    if fts_res.is_err() {
        let mut like_sql = "SELECT id FROM framework_documentation WHERE (title LIKE ?1 OR content LIKE ?1)".to_string();
        let like_query = format!("%{}%", query_text);
        if category.to_lowercase() != "all" {
            like_sql.push_str(" AND category = ?2");
            let mut like_stmt = conn.prepare(&like_sql)?;
            let mut rows = like_stmt.query(rusqlite::params![like_query, category])?;
            while let Some(row) = rows.next()? {
                fts_ids.push(row.get(0)?);
            }
        } else {
            let mut like_stmt = conn.prepare(&like_sql)?;
            let mut rows = like_stmt.query(rusqlite::params![like_query])?;
            while let Some(row) = rows.next()? {
                fts_ids.push(row.get(0)?);
            }
        }
    }
    
    let mut rrf_scores: HashMap<String, f64> = HashMap::new();
    for (rank, cand) in semantic_candidates.iter().enumerate() {
        let score = 1.0 / (60.0 + rank as f64 + 1.0);
        rrf_scores.insert(cand.0.clone(), score);
    }
    for (rank, r_id) in fts_ids.iter().enumerate() {
        let score = 1.0 / (60.0 + rank as f64 + 1.0);
        *rrf_scores.entry(r_id.clone()).or_insert(0.0) += score;
    }
    
    let mut combined: Vec<(String, f64)> = rrf_scores.into_iter().collect();
    
    if !combined.is_empty() {
        let placeholders: Vec<String> = combined.iter().map(|_| "?".to_string()).collect();
        let query = format!(
            "SELECT fd.id, COUNT(fdep.id) FROM framework_documentation fd JOIN framework_dependencies fdep ON fd.url = fdep.target_url WHERE fd.id IN ({}) GROUP BY fd.id",
            placeholders.join(",")
        );
        let params_vec: Vec<String> = combined.iter().map(|(id, _)| id.clone()).collect();
        if let Ok(mut stmt) = conn.prepare(&query) {
            if let Ok(mut rows) = stmt.query(rusqlite::params_from_iter(params_vec.iter())) {
                let mut boosts: HashMap<String, f64> = HashMap::new();
                while let Ok(Some(row)) = rows.next() {
                    let id: String = row.get(0).unwrap();
                    let count: i64 = row.get(1).unwrap();
                    boosts.insert(id, count as f64);
                }
                for (id, score) in combined.iter_mut() {
                    if let Some(&count) = boosts.get(id) {
                        *score += count * 0.005; // 0.005 boost per incoming link
                    }
                }
            }
        }
    }

    combined.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    combined.truncate(limit);
    
    let mut results = Vec::new();
    for (r_id, rrf_score) in combined {
        let mut stmt = conn.prepare("SELECT category, version, title, url, content FROM framework_documentation WHERE id = ?1")?;
        let mut rows = stmt.query(rusqlite::params![r_id])?;
        if let Some(row) = rows.next()? {
            results.push(HybridDocumentationResult {
                id: r_id,
                category: row.get(0)?,
                version: row.get(1)?,
                title: row.get(2)?,
                url: row.get(3)?,
                content: row.get(4)?,
                rrf_score,
            });
        }
    }
    
    Ok(results)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsolidatedFactResult {
    pub id: String,
    pub project_name: String,
    pub category: String,
    pub fact_type: String,
    pub fact_content: String,
    pub rrf_score: f64,
}

pub fn query_consolidated_facts(
    conn: &Connection,
    _query_text: &str,
    query_embedding: &[f32],
    project_name: &str,
    limit: usize,
) -> Result<Vec<ConsolidatedFactResult>> {
    let mut sql = "SELECT id, project_name, category, fact_type, fact_content, embedding FROM consolidated_facts".to_string();
    let mut candidates: Vec<(String, f64)> = Vec::new();

    if project_name.to_lowercase() != "all" {
        sql.push_str(" WHERE project_name = ?1");
        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query(rusqlite::params![project_name])?;
        while let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            let blob: Vec<u8> = row.get(5)?;
            let emb = blob_to_vector(&blob);
            let sim = cosine_similarity(query_embedding, &emb);
            candidates.push((id, sim as f64));
        }
    } else {
        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            let blob: Vec<u8> = row.get(5)?;
            let emb = blob_to_vector(&blob);
            let sim = cosine_similarity(query_embedding, &emb);
            candidates.push((id, sim as f64));
        }
    }

    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    candidates.truncate(limit);

    let mut results = Vec::new();
    for (r_id, score) in candidates {
        let mut stmt = conn.prepare("SELECT project_name, category, fact_type, fact_content FROM consolidated_facts WHERE id = ?1")?;
        let mut rows = stmt.query(rusqlite::params![r_id])?;
        if let Some(row) = rows.next()? {
            results.push(ConsolidatedFactResult {
                id: r_id,
                project_name: row.get(0)?,
                category: row.get(1)?,
                fact_type: row.get(2)?,
                fact_content: row.get(3)?,
                rrf_score: score,
            });
        }
    }

    Ok(results)
}

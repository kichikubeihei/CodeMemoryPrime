use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

fn default_evidence_tier() -> String {
    "verified".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KnowledgeNode {
    pub id: String,
    pub profile: String,       // "code", "lore", "ttrpg", "agent"
    pub entity_type: String,   // "character", "faction", "spell", "subagent", "function", etc.
    pub name: String,
    pub content: String,
    pub metadata_json: String, // Rich JSON metadata
    pub created_at: String,
    pub updated_at: String,
    #[serde(default = "default_evidence_tier")]
    pub evidence_tier: String, // "verified", "axiom", "inference", "historical", "hypothesis"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KnowledgeEdge {
    pub id: String,
    pub profile: String,
    pub source_id: String,
    pub target_id: String,
    pub relation_type: String, // "allied_with", "enemy_of", "located_in", "depends_on", etc.
    pub intensity: f64,        // Relationship strength / weight (e.g. 1.0)
    pub metadata_json: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlatGraphEdge {
    pub source_id: String,
    pub source_name: String,
    pub target_id: String,
    pub target_name: String,
    pub relation_type: String,
    pub intensity: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KnowledgeSubgraph {
    pub root_id: String,
    pub nodes: Vec<KnowledgeNode>,
    pub edges: Vec<KnowledgeEdge>,
}

pub fn init_knowledge_graph_tables(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS knowledge_nodes (
            id TEXT PRIMARY KEY,
            profile TEXT NOT NULL,
            entity_type TEXT NOT NULL,
            name TEXT NOT NULL,
            content TEXT NOT NULL,
            metadata_json TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            evidence_tier TEXT DEFAULT 'verified'
        )",
        [],
    )?;
    let _ = conn.execute("ALTER TABLE knowledge_nodes ADD COLUMN evidence_tier TEXT DEFAULT 'verified'", []);

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_knodes_profile_type 
         ON knowledge_nodes(profile, entity_type)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_knodes_name 
         ON knowledge_nodes(name)",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS knowledge_edges (
            id TEXT PRIMARY KEY,
            profile TEXT NOT NULL,
            source_id TEXT NOT NULL,
            target_id TEXT NOT NULL,
            relation_type TEXT NOT NULL,
            intensity REAL NOT NULL DEFAULT 1.0,
            metadata_json TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY(source_id) REFERENCES knowledge_nodes(id) ON DELETE CASCADE,
            FOREIGN KEY(target_id) REFERENCES knowledge_nodes(id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_kedges_source_target 
         ON knowledge_edges(source_id, target_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_kedges_profile 
         ON knowledge_edges(profile)",
        [],
    )?;

    Ok(())
}

pub fn insert_knowledge_node(conn: &Connection, node: &KnowledgeNode) -> Result<String> {
    let _ = conn.execute("ALTER TABLE knowledge_nodes ADD COLUMN evidence_tier TEXT DEFAULT 'verified'", []);
    let tier = if node.evidence_tier.is_empty() { "verified" } else { &node.evidence_tier };
    conn.execute(
        "INSERT INTO knowledge_nodes (id, profile, entity_type, name, content, metadata_json, created_at, updated_at, evidence_tier)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(id) DO UPDATE SET
             name=excluded.name,
             content=excluded.content,
             metadata_json=excluded.metadata_json,
             updated_at=excluded.updated_at,
             evidence_tier=excluded.evidence_tier",
        params![
            node.id,
            node.profile,
            node.entity_type,
            node.name,
            node.content,
            node.metadata_json,
            node.created_at,
            node.updated_at,
            tier
        ],
    )?;
    Ok(node.id.clone())
}

pub fn insert_knowledge_edge(conn: &Connection, edge: &KnowledgeEdge) -> Result<String> {
    conn.execute(
        "INSERT INTO knowledge_edges (id, profile, source_id, target_id, relation_type, intensity, metadata_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(id) DO UPDATE SET
             relation_type=excluded.relation_type,
             intensity=excluded.intensity,
             metadata_json=excluded.metadata_json",
        params![
            edge.id,
            edge.profile,
            edge.source_id,
            edge.target_id,
            edge.relation_type,
            edge.intensity,
            edge.metadata_json,
            edge.created_at
        ],
    )?;
    Ok(edge.id.clone())
}

pub fn query_nodes_by_type(conn: &Connection, profile: &str, entity_type: &str) -> Result<Vec<KnowledgeNode>> {
    let _ = conn.execute("ALTER TABLE knowledge_nodes ADD COLUMN evidence_tier TEXT DEFAULT 'verified'", []);
    let mut stmt = conn.prepare(
        "SELECT id, profile, entity_type, name, content, metadata_json, created_at, updated_at, evidence_tier
         FROM knowledge_nodes
         WHERE profile = ?1 AND entity_type = ?2
         ORDER BY name ASC",
    )?;

    let rows = stmt.query_map(params![profile, entity_type], |row| {
        let tier: Option<String> = row.get(8).ok();
        Ok(KnowledgeNode {
            id: row.get(0)?,
            profile: row.get(1)?,
            entity_type: row.get(2)?,
            name: row.get(3)?,
            content: row.get(4)?,
            metadata_json: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
            evidence_tier: tier.unwrap_or_else(|| "verified".to_string()),
        })
    })?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

pub fn query_node_by_id(conn: &Connection, id: &str) -> Result<Option<KnowledgeNode>> {
    let _ = conn.execute("ALTER TABLE knowledge_nodes ADD COLUMN evidence_tier TEXT DEFAULT 'verified'", []);
    let mut stmt = conn.prepare(
        "SELECT id, profile, entity_type, name, content, metadata_json, created_at, updated_at, evidence_tier
         FROM knowledge_nodes
         WHERE id = ?1",
    )?;

    let mut rows = stmt.query_map(params![id], |row| {
        let tier: Option<String> = row.get(8).ok();
        Ok(KnowledgeNode {
            id: row.get(0)?,
            profile: row.get(1)?,
            entity_type: row.get(2)?,
            name: row.get(3)?,
            content: row.get(4)?,
            metadata_json: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
            evidence_tier: tier.unwrap_or_else(|| "verified".to_string()),
        })
    })?;

    if let Some(r) = rows.next() {
        Ok(Some(r?))
    } else {
        Ok(None)
    }
}

/// Instant Flat Graph Edge query for 60 FPS Visualizers (Cytoscape / D3 / Svelte 5 Canvas)
pub fn query_all_edges_flat(conn: &Connection, profile: &str) -> Result<Vec<FlatGraphEdge>> {
    let mut stmt = conn.prepare(
        "SELECT 
            e.source_id, 
            n1.name AS source_name, 
            e.target_id, 
            n2.name AS target_name, 
            e.relation_type, 
            e.intensity
         FROM knowledge_edges e
         JOIN knowledge_nodes n1 ON e.source_id = n1.id
         JOIN knowledge_nodes n2 ON e.target_id = n2.id
         WHERE e.profile = ?1",
    )?;

    let rows = stmt.query_map(params![profile], |row| {
        Ok(FlatGraphEdge {
            source_id: row.get(0)?,
            source_name: row.get(1)?,
            target_id: row.get(2)?,
            target_name: row.get(3)?,
            relation_type: row.get(4)?,
            intensity: row.get(5)?,
        })
    })?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

/// Recursive Common Table Expression (CTE) Subgraph Traversal
pub fn query_subgraph(conn: &Connection, root_id: &str, max_depth: u32) -> Result<KnowledgeSubgraph> {
    let mut stmt = conn.prepare(
        "WITH RECURSIVE traverse(node_id, depth) AS (
            SELECT ?1 AS node_id, 0 AS depth
            UNION
            SELECT e.target_id, t.depth + 1
            FROM knowledge_edges e
            JOIN traverse t ON e.source_id = t.node_id
            WHERE t.depth < ?2
        )
        SELECT DISTINCT n.id, n.profile, n.entity_type, n.name, n.content, n.metadata_json, n.created_at, n.updated_at, n.evidence_tier
        FROM knowledge_nodes n
        JOIN traverse t ON n.id = t.node_id",
    )?;

    let node_rows = stmt.query_map(params![root_id, max_depth], |row| {
        let tier: Option<String> = row.get(8).ok();
        Ok(KnowledgeNode {
            id: row.get(0)?,
            profile: row.get(1)?,
            entity_type: row.get(2)?,
            name: row.get(3)?,
            content: row.get(4)?,
            metadata_json: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
            evidence_tier: tier.unwrap_or_else(|| "verified".to_string()),
        })
    })?;

    let mut nodes = Vec::new();
    let mut node_ids = Vec::new();
    for r in node_rows {
        let n = r?;
        node_ids.push(n.id.clone());
        nodes.push(n);
    }

    // Fetch all connecting edges between these nodes
    let mut edges = Vec::new();
    if !node_ids.is_empty() {
        let placeholders = node_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT id, profile, source_id, target_id, relation_type, intensity, metadata_json, created_at
             FROM knowledge_edges
             WHERE source_id IN ({}) AND target_id IN ({})",
            placeholders, placeholders
        );

        let mut edge_stmt = conn.prepare(&sql)?;
        let mut query_params: Vec<&dyn rusqlite::ToSql> = Vec::new();
        for id in &node_ids {
            query_params.push(id);
        }
        for id in &node_ids {
            query_params.push(id);
        }

        let edge_rows = edge_stmt.query_map(&query_params[..], |row| {
            Ok(KnowledgeEdge {
                id: row.get(0)?,
                profile: row.get(1)?,
                source_id: row.get(2)?,
                target_id: row.get(3)?,
                relation_type: row.get(4)?,
                intensity: row.get(5)?,
                metadata_json: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;

        for r in edge_rows {
            edges.push(r?);
        }
    }

    Ok(KnowledgeSubgraph {
        root_id: root_id.to_string(),
        nodes,
        edges,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_graph_crud_and_subgraph() {
        let conn = Connection::open_in_memory().unwrap();
        init_knowledge_graph_tables(&conn).unwrap();

        // 1. Insert Characters for Lore Titan
        let aragorn = KnowledgeNode {
            id: "char_aragorn".to_string(),
            profile: "lore".to_string(),
            entity_type: "character".to_string(),
            name: "Aragorn II Elessar".to_string(),
            content: "High King of Gondor and Arnor, Chieftain of the Dúnedain.".to_string(),
            metadata_json: r#"{"race":"Human","title":"King of Gondor"}"#.to_string(),
            created_at: "2026-08-31T00:00:00Z".to_string(),
            updated_at: "2026-08-31T00:00:00Z".to_string(),
            evidence_tier: "axiom".to_string(),
        };
        let arwen = KnowledgeNode {
            id: "char_arwen".to_string(),
            profile: "lore".to_string(),
            entity_type: "character".to_string(),
            name: "Arwen Undómiel".to_string(),
            content: "Queen of the Reunited Kingdom, daughter of Elrond.".to_string(),
            metadata_json: r#"{"race":"Half-elven"}"#.to_string(),
            created_at: "2026-08-31T00:00:00Z".to_string(),
            updated_at: "2026-08-31T00:00:00Z".to_string(),
            evidence_tier: "axiom".to_string(),
        };

        insert_knowledge_node(&conn, &aragorn).unwrap();
        insert_knowledge_node(&conn, &arwen).unwrap();

        // 2. Insert Relationship Edge
        let edge = KnowledgeEdge {
            id: "edge_aragorn_arwen".to_string(),
            profile: "lore".to_string(),
            source_id: "char_aragorn".to_string(),
            target_id: "char_arwen".to_string(),
            relation_type: "betrothed_to".to_string(),
            intensity: 1.0,
            metadata_json: r#"{"status":"married"}"#.to_string(),
            created_at: "2026-08-31T00:00:00Z".to_string(),
        };
        insert_knowledge_edge(&conn, &edge).unwrap();

        // 3. Query Nodes by Type
        let chars = query_nodes_by_type(&conn, "lore", "character").unwrap();
        assert_eq!(chars.len(), 2);

        // 4. Query Flat Edges for Visualizer
        let flat_edges = query_all_edges_flat(&conn, "lore").unwrap();
        assert_eq!(flat_edges.len(), 1);
        assert_eq!(flat_edges[0].source_name, "Aragorn II Elessar");
        assert_eq!(flat_edges[0].target_name, "Arwen Undómiel");

        // 5. Query Recursive Subgraph
        let subgraph = query_subgraph(&conn, "char_aragorn", 2).unwrap();
        assert_eq!(subgraph.nodes.len(), 2);
        assert_eq!(subgraph.edges.len(), 1);
    }
}

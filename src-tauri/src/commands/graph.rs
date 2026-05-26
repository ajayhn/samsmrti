use crate::commands::profiles::ActiveProfile;
use crate::commands::review::ReviewCard;
use crate::db::deck_tree::deck_scope_ids;
use crate::db::Database;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

// ── Entity types ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub entity_type: Option<String>,
    pub description: String,
    pub created_at: i64,
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEntityInput {
    pub name: String,
    pub entity_type: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEntityInput {
    pub id: String,
    pub name: Option<String>,
    pub entity_type: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

// ── Relation types ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RelationType {
    pub id: String,
    pub name: String,
    pub inverse_name: Option<String>,
    pub created_at: i64,
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRelationTypeInput {
    pub name: String,
    pub inverse_name: Option<String>,
    pub tags: Option<Vec<String>>,
}

// ── Triple types ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Triple {
    pub id: String,
    pub subject_id: String,
    pub relation_type_id: String,
    pub object_id: String,
    pub created_at: i64,
    pub subject_name: String,
    pub relation_name: String,
    pub object_name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTripleInput {
    pub subject_id: String,
    pub relation_type_id: String,
    pub object_id: String,
}

#[derive(Debug, Deserialize)]
pub struct BatchCreateTriplesInput {
    pub subject_id: String,
    pub relation_type_id: String,
    pub object_ids: Vec<String>,
}

// ── Mindmap types ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct MindMapNode {
    pub id: String,
    pub label: String,
    pub node_type: String, // "root", "relation", "entity"
    pub entity_type: Option<String>,
    pub card_status: String, // "reviewed", "due", "none"
    pub triple_count: i64,
}

#[derive(Debug, Serialize)]
pub struct MindMapEdge {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Serialize)]
pub struct MindMapData {
    pub nodes: Vec<MindMapNode>,
    pub edges: Vec<MindMapEdge>,
}

// ── Card suggestion types ───────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct CardSuggestion {
    pub triple_ids: Vec<String>,
    pub front: String,
    pub back: String,
    pub suggestion_type: String, // "forward" or "reverse"
}

// ── E-R-E review types ─────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct EreReviewEntity {
    pub entity_id: String,
    pub entity_name: String,
    pub due_count: i64,
}

// ── Helper: ensure a tag exists and return its id ───────────────────────────

fn ensure_tag(conn: &rusqlite::Connection, tag_name: &str) -> Result<String, String> {
    let existing: Option<String> = conn
        .query_row("SELECT id FROM tags WHERE name = ?1", [tag_name], |row| {
            row.get(0)
        })
        .ok();
    if let Some(id) = existing {
        return Ok(id);
    }
    let id = format!("tag_{}", uuid::Uuid::new_v4().simple());
    conn.execute(
        "INSERT INTO tags (id, name) VALUES (?1, ?2)",
        (&id, tag_name),
    )
    .map_err(|e| e.to_string())?;
    Ok(id)
}

fn load_entity_tags(conn: &rusqlite::Connection, entity_id: &str) -> Vec<String> {
    conn.prepare("SELECT t.name FROM tags t JOIN entity_tags et ON t.id = et.tag_id WHERE et.entity_id = ?1")
        .and_then(|mut s| {
            s.query_map([entity_id], |row| row.get(0))
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default()
}

fn load_relation_type_tags(conn: &rusqlite::Connection, rt_id: &str) -> Vec<String> {
    conn.prepare("SELECT t.name FROM tags t JOIN relation_type_tags rtt ON t.id = rtt.tag_id WHERE rtt.relation_type_id = ?1")
        .and_then(|mut s| {
            s.query_map([rt_id], |row| row.get(0))
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default()
}

// ═══════════════════════════════════════════════════════════════════════════
// Entity CRUD
// ═══════════════════════════════════════════════════════════════════════════

fn normalize_type(t: &str) -> String {
    let s = t.trim();
    if s.is_empty() {
        return String::new();
    }
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().to_string() + &c.as_str().to_lowercase(),
    }
}

#[tauri::command]
pub fn create_entity(db: State<Database>, input: CreateEntityInput) -> Result<Entity, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp();
    let id = format!("ent_{}", uuid::Uuid::new_v4().simple());
    let entity_type = input.entity_type.as_deref().map(normalize_type).filter(|s| !s.is_empty());

    conn.execute(
        "INSERT INTO entities (id, name, entity_type, description, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        (&id, &input.name, &entity_type, input.description.as_deref().unwrap_or(""), now),
    )
    .map_err(|e| e.to_string())?;

    if let Some(tags) = &input.tags {
        for tag_name in tags {
            let tag_id = ensure_tag(&conn, tag_name)?;
            conn.execute(
                "INSERT OR IGNORE INTO entity_tags (entity_id, tag_id) VALUES (?1, ?2)",
                (&id, &tag_id),
            )
            .map_err(|e| e.to_string())?;
        }
    }

    let tags = load_entity_tags(&conn, &id);

    Ok(Entity {
        id,
        name: input.name,
        entity_type,
        description: input.description.unwrap_or_default(),
        created_at: now,
        tags,
    })
}

#[tauri::command]
pub fn get_entities(
    db: State<Database>,
    search: Option<String>,
    entity_type: Option<String>,
    tag: Option<String>,
) -> Result<Vec<Entity>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut sql = String::from("SELECT e.id, e.name, e.entity_type, e.description, e.created_at FROM entities e ");
    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let mut idx = 1;

    if let Some(ref s) = search {
        if !s.trim().is_empty() {
            conditions.push(format!("e.name LIKE ?{}", idx));
            params.push(Box::new(format!("%{}%", s.trim())));
            idx += 1;
        }
    }

    if let Some(ref et) = entity_type {
        if !et.is_empty() {
            conditions.push(format!("e.entity_type = ?{}", idx));
            params.push(Box::new(et.clone()));
            idx += 1;
        }
    }

    if let Some(ref t) = tag {
        if !t.is_empty() {
            conditions.push(format!(
                "EXISTS (SELECT 1 FROM entity_tags et JOIN tags tg ON tg.id = et.tag_id WHERE et.entity_id = e.id AND tg.name = ?{})",
                idx
            ));
            params.push(Box::new(t.clone()));
            idx += 1;
        }
    }
    let _ = idx;

    if !conditions.is_empty() {
        sql.push_str("WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }
    sql.push_str(" ORDER BY e.name ASC LIMIT 500");

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows: Vec<(String, String, Option<String>, String, i64)> = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut entities = Vec::with_capacity(rows.len());
    for (id, name, etype, desc, created) in rows {
        let tags = load_entity_tags(&conn, &id);
        entities.push(Entity {
            id,
            name,
            entity_type: etype,
            description: desc,
            created_at: created,
            tags,
        });
    }
    Ok(entities)
}

#[tauri::command]
pub fn update_entity(db: State<Database>, input: UpdateEntityInput) -> Result<Entity, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    if let Some(ref name) = input.name {
        conn.execute("UPDATE entities SET name = ?1 WHERE id = ?2", (name, &input.id))
            .map_err(|e| e.to_string())?;
    }
    if let Some(ref et) = input.entity_type {
        let normalized = normalize_type(et);
        let val: Option<&str> = if normalized.is_empty() { None } else { Some(&normalized) };
        conn.execute("UPDATE entities SET entity_type = ?1 WHERE id = ?2", (val, &input.id))
            .map_err(|e| e.to_string())?;
    }
    if let Some(ref desc) = input.description {
        conn.execute("UPDATE entities SET description = ?1 WHERE id = ?2", (desc, &input.id))
            .map_err(|e| e.to_string())?;
    }
    if let Some(ref tags) = input.tags {
        conn.execute("DELETE FROM entity_tags WHERE entity_id = ?1", [&input.id])
            .map_err(|e| e.to_string())?;
        for tag_name in tags {
            let tag_id = ensure_tag(&conn, tag_name)?;
            conn.execute(
                "INSERT OR IGNORE INTO entity_tags (entity_id, tag_id) VALUES (?1, ?2)",
                (&input.id, &tag_id),
            )
            .map_err(|e| e.to_string())?;
        }
    }

    let (name, etype, desc, created): (String, Option<String>, String, i64) = conn
        .query_row(
            "SELECT name, entity_type, description, created_at FROM entities WHERE id = ?1",
            [&input.id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|e| e.to_string())?;

    let tags = load_entity_tags(&conn, &input.id);

    Ok(Entity {
        id: input.id,
        name,
        entity_type: etype,
        description: desc,
        created_at: created,
        tags,
    })
}

#[tauri::command]
pub fn delete_entity(db: State<Database>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM entities WHERE id = ?1", [&id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// Relation Type CRUD
// ═══════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub fn create_relation_type(
    db: State<Database>,
    input: CreateRelationTypeInput,
) -> Result<RelationType, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp();
    let id = format!("rt_{}", uuid::Uuid::new_v4().simple());

    conn.execute(
        "INSERT INTO relation_types (id, name, inverse_name, created_at) VALUES (?1, ?2, ?3, ?4)",
        (&id, &input.name, &input.inverse_name, now),
    )
    .map_err(|e| e.to_string())?;

    if let Some(tags) = &input.tags {
        for tag_name in tags {
            let tag_id = ensure_tag(&conn, tag_name)?;
            conn.execute(
                "INSERT OR IGNORE INTO relation_type_tags (relation_type_id, tag_id) VALUES (?1, ?2)",
                (&id, &tag_id),
            )
            .map_err(|e| e.to_string())?;
        }
    }

    let tags = load_relation_type_tags(&conn, &id);

    Ok(RelationType {
        id,
        name: input.name,
        inverse_name: input.inverse_name,
        created_at: now,
        tags,
    })
}

#[tauri::command]
pub fn get_relation_types(
    db: State<Database>,
    tag: Option<String>,
) -> Result<Vec<RelationType>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut sql = String::from(
        "SELECT rt.id, rt.name, rt.inverse_name, rt.created_at FROM relation_types rt ",
    );

    if let Some(ref t) = tag {
        if !t.is_empty() {
            sql.push_str("WHERE EXISTS (SELECT 1 FROM relation_type_tags rtt JOIN tags tg ON tg.id = rtt.tag_id WHERE rtt.relation_type_id = rt.id AND tg.name = ?1) ");
        }
    }
    sql.push_str("ORDER BY rt.name ASC");

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

    let rows: Vec<(String, String, Option<String>, i64)> = if let Some(ref t) = tag {
        if !t.is_empty() {
            stmt.query_map([t], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
        } else {
            stmt.query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
        }
    } else {
        stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
    };

    let mut rts = Vec::with_capacity(rows.len());
    for (id, name, inv, created) in rows {
        let tags = load_relation_type_tags(&conn, &id);
        rts.push(RelationType {
            id,
            name,
            inverse_name: inv,
            created_at: created,
            tags,
        });
    }
    Ok(rts)
}

#[tauri::command]
pub fn delete_relation_type(db: State<Database>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM relation_types WHERE id = ?1", [&id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// Triple CRUD
// ═══════════════════════════════════════════════════════════════════════════

fn load_triple_row(conn: &rusqlite::Connection, triple_id: &str) -> Result<Triple, String> {
    conn.query_row(
        "SELECT t.id, t.subject_id, t.relation_type_id, t.object_id, t.created_at,
                se.name, rt.name, oe.name
         FROM triples t
         JOIN entities se ON se.id = t.subject_id
         JOIN relation_types rt ON rt.id = t.relation_type_id
         JOIN entities oe ON oe.id = t.object_id
         WHERE t.id = ?1",
        [triple_id],
        |row| {
            Ok(Triple {
                id: row.get(0)?,
                subject_id: row.get(1)?,
                relation_type_id: row.get(2)?,
                object_id: row.get(3)?,
                created_at: row.get(4)?,
                subject_name: row.get(5)?,
                relation_name: row.get(6)?,
                object_name: row.get(7)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_triple(db: State<Database>, input: CreateTripleInput) -> Result<Triple, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp();
    let id = format!("trp_{}", uuid::Uuid::new_v4().simple());

    conn.execute(
        "INSERT INTO triples (id, subject_id, relation_type_id, object_id, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        (&id, &input.subject_id, &input.relation_type_id, &input.object_id, now),
    )
    .map_err(|e| e.to_string())?;

    load_triple_row(&conn, &id)
}

#[tauri::command]
pub fn batch_create_triples(
    db: State<Database>,
    input: BatchCreateTriplesInput,
) -> Result<Vec<Triple>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp();
    let mut triples = Vec::new();

    for object_id in &input.object_ids {
        let id = format!("trp_{}", uuid::Uuid::new_v4().simple());
        let result = conn.execute(
            "INSERT OR IGNORE INTO triples (id, subject_id, relation_type_id, object_id, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            (&id, &input.subject_id, &input.relation_type_id, object_id, now),
        );
        if let Ok(1) = result {
            if let Ok(t) = load_triple_row(&conn, &id) {
                triples.push(t);
            }
        }
    }

    Ok(triples)
}

#[tauri::command]
pub fn get_triples(
    db: State<Database>,
    subject_id: Option<String>,
    object_id: Option<String>,
    relation_type_id: Option<String>,
    entity_id: Option<String>,
) -> Result<Vec<Triple>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut sql = String::from(
        "SELECT t.id, t.subject_id, t.relation_type_id, t.object_id, t.created_at,
                se.name, rt.name, oe.name
         FROM triples t
         JOIN entities se ON se.id = t.subject_id
         JOIN relation_types rt ON rt.id = t.relation_type_id
         JOIN entities oe ON oe.id = t.object_id ",
    );

    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let mut idx = 1;

    if let Some(ref sid) = subject_id {
        conditions.push(format!("t.subject_id = ?{}", idx));
        params.push(Box::new(sid.clone()));
        idx += 1;
    }
    if let Some(ref oid) = object_id {
        conditions.push(format!("t.object_id = ?{}", idx));
        params.push(Box::new(oid.clone()));
        idx += 1;
    }
    if let Some(ref rid) = relation_type_id {
        conditions.push(format!("t.relation_type_id = ?{}", idx));
        params.push(Box::new(rid.clone()));
        idx += 1;
    }
    if let Some(ref eid) = entity_id {
        conditions.push(format!("(t.subject_id = ?{} OR t.object_id = ?{})", idx, idx + 1));
        params.push(Box::new(eid.clone()));
        params.push(Box::new(eid.clone()));
        idx += 2;
    }
    let _ = idx;

    if !conditions.is_empty() {
        sql.push_str("WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }
    sql.push_str(" ORDER BY t.created_at DESC LIMIT 1000");

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok(Triple {
                id: row.get(0)?,
                subject_id: row.get(1)?,
                relation_type_id: row.get(2)?,
                object_id: row.get(3)?,
                created_at: row.get(4)?,
                subject_name: row.get(5)?,
                relation_name: row.get(6)?,
                object_name: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

#[derive(Debug, Deserialize)]
pub struct UpdateTripleInput {
    pub id: String,
    pub subject_id: Option<String>,
    pub relation_type_id: Option<String>,
    pub object_id: Option<String>,
}

#[tauri::command]
pub fn update_triple(db: State<Database>, input: UpdateTripleInput) -> Result<Triple, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    if let Some(ref sid) = input.subject_id {
        conn.execute("UPDATE triples SET subject_id = ?1 WHERE id = ?2", (sid, &input.id))
            .map_err(|e| e.to_string())?;
    }
    if let Some(ref rtid) = input.relation_type_id {
        conn.execute("UPDATE triples SET relation_type_id = ?1 WHERE id = ?2", (rtid, &input.id))
            .map_err(|e| e.to_string())?;
    }
    if let Some(ref oid) = input.object_id {
        conn.execute("UPDATE triples SET object_id = ?1 WHERE id = ?2", (oid, &input.id))
            .map_err(|e| e.to_string())?;
    }

    let row: (String, String, String, String, i64, String, String, String) = conn
        .query_row(
            "SELECT t.id, t.subject_id, t.relation_type_id, t.object_id, t.created_at,
                    se.name, rt.name, oe.name
             FROM triples t
             JOIN entities se ON se.id = t.subject_id
             JOIN relation_types rt ON rt.id = t.relation_type_id
             JOIN entities oe ON oe.id = t.object_id
             WHERE t.id = ?1",
            [&input.id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?)),
        )
        .map_err(|e| e.to_string())?;

    Ok(Triple {
        id: row.0,
        subject_id: row.1,
        relation_type_id: row.2,
        object_id: row.3,
        created_at: row.4,
        subject_name: row.5,
        relation_name: row.6,
        object_name: row.7,
    })
}

#[tauri::command]
pub fn delete_triple(db: State<Database>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM triples WHERE id = ?1", [&id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// Card-Triple linking
// ═══════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub fn link_card_to_triple(
    db: State<Database>,
    card_id: String,
    triple_id: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR IGNORE INTO card_triples (card_id, triple_id) VALUES (?1, ?2)",
        (&card_id, &triple_id),
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn unlink_card_from_triple(
    db: State<Database>,
    card_id: String,
    triple_id: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM card_triples WHERE card_id = ?1 AND triple_id = ?2",
        (&card_id, &triple_id),
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_triples_for_card(db: State<Database>, card_id: String) -> Result<Vec<Triple>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.subject_id, t.relation_type_id, t.object_id, t.created_at,
                    se.name, rt.name, oe.name
             FROM triples t
             JOIN card_triples ct ON ct.triple_id = t.id
             JOIN entities se ON se.id = t.subject_id
             JOIN relation_types rt ON rt.id = t.relation_type_id
             JOIN entities oe ON oe.id = t.object_id
             WHERE ct.card_id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([&card_id], |row| {
            Ok(Triple {
                id: row.get(0)?,
                subject_id: row.get(1)?,
                relation_type_id: row.get(2)?,
                object_id: row.get(3)?,
                created_at: row.get(4)?,
                subject_name: row.get(5)?,
                relation_name: row.get(6)?,
                object_name: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

#[tauri::command]
pub fn get_cards_for_triple(
    db: State<Database>,
    triple_id: String,
) -> Result<Vec<String>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT card_id FROM card_triples WHERE triple_id = ?1")
        .map_err(|e| e.to_string())?;
    let ids = stmt
        .query_map([&triple_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<String>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(ids)
}

// ═══════════════════════════════════════════════════════════════════════════
// Mindmap data
// ═══════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub fn get_mindmap(
    db: State<Database>,
    active: State<'_, Mutex<ActiveProfile>>,
    entity_id: String,
) -> Result<MindMapData, String> {
    let active_guard = active.lock().map_err(|e| e.to_string())?;
    let profile_id = active_guard.id.clone();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp();

    let (root_name, root_type): (String, Option<String>) = conn
        .query_row(
            "SELECT name, entity_type FROM entities WHERE id = ?1",
            [&entity_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    nodes.push(MindMapNode {
        id: entity_id.clone(),
        label: root_name,
        node_type: "root".to_string(),
        entity_type: root_type,
        card_status: "none".to_string(),
        triple_count: 0,
    });

    // Outgoing triples: entity is subject
    let mut out_stmt = conn
        .prepare(
            "SELECT t.relation_type_id, rt.name, t.object_id, e.name, e.entity_type, t.id
             FROM triples t
             JOIN relation_types rt ON rt.id = t.relation_type_id
             JOIN entities e ON e.id = t.object_id
             WHERE t.subject_id = ?1
             ORDER BY rt.name, e.name",
        )
        .map_err(|e| e.to_string())?;

    let out_rows: Vec<(String, String, String, String, Option<String>, String)> = out_stmt
        .query_map([&entity_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // Incoming triples: entity is object
    let mut in_stmt = conn
        .prepare(
            "SELECT t.relation_type_id, rt.name, rt.inverse_name, t.subject_id, e.name, e.entity_type, t.id
             FROM triples t
             JOIN relation_types rt ON rt.id = t.relation_type_id
             JOIN entities e ON e.id = t.subject_id
             WHERE t.object_id = ?1
             ORDER BY rt.name, e.name",
        )
        .map_err(|e| e.to_string())?;

    let in_rows: Vec<(String, String, Option<String>, String, String, Option<String>, String)> = in_stmt
        .query_map([&entity_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // Group outgoing by relation type
    let mut seen_relations = std::collections::HashSet::new();
    let mut seen_entities = std::collections::HashSet::new();

    for (rt_id, rt_name, obj_id, obj_name, obj_type, triple_id) in &out_rows {
        let rel_node_id = format!("rel_out_{}", rt_id);
        if seen_relations.insert(rel_node_id.clone()) {
            let count = out_rows.iter().filter(|r| &r.0 == rt_id).count() as i64;
            nodes.push(MindMapNode {
                id: rel_node_id.clone(),
                label: rt_name.clone(),
                node_type: "relation".to_string(),
                entity_type: None,
                card_status: "none".to_string(),
                triple_count: count,
            });
            edges.push(MindMapEdge {
                source: entity_id.clone(),
                target: rel_node_id.clone(),
            });
        }

        if seen_entities.insert(obj_id.clone()) {
            let card_status = get_card_status_for_triple(&conn, &profile_id, triple_id, now);
            nodes.push(MindMapNode {
                id: obj_id.clone(),
                label: obj_name.clone(),
                node_type: "entity".to_string(),
                entity_type: obj_type.clone(),
                card_status,
                triple_count: 0,
            });
        }
        let rel_node_id = format!("rel_out_{}", rt_id);
        edges.push(MindMapEdge {
            source: rel_node_id,
            target: obj_id.clone(),
        });
    }

    for (rt_id, rt_name, inv_name, subj_id, subj_name, subj_type, triple_id) in &in_rows {
        let label = inv_name.as_deref().unwrap_or(&format!("(inv) {}", rt_name)).to_string();
        let rel_node_id = format!("rel_in_{}", rt_id);
        if seen_relations.insert(rel_node_id.clone()) {
            let count = in_rows.iter().filter(|r| &r.0 == rt_id).count() as i64;
            nodes.push(MindMapNode {
                id: rel_node_id.clone(),
                label,
                node_type: "relation".to_string(),
                entity_type: None,
                card_status: "none".to_string(),
                triple_count: count,
            });
            edges.push(MindMapEdge {
                source: entity_id.clone(),
                target: rel_node_id.clone(),
            });
        }

        if seen_entities.insert(subj_id.clone()) {
            let card_status = get_card_status_for_triple(&conn, &profile_id, triple_id, now);
            nodes.push(MindMapNode {
                id: subj_id.clone(),
                label: subj_name.clone(),
                node_type: "entity".to_string(),
                entity_type: subj_type.clone(),
                card_status,
                triple_count: 0,
            });
        }
        let rel_node_id = format!("rel_in_{}", rt_id);
        edges.push(MindMapEdge {
            source: rel_node_id,
            target: subj_id.clone(),
        });
    }

    Ok(MindMapData { nodes, edges })
}

fn get_card_status_for_triple(
    conn: &rusqlite::Connection,
    profile_id: &str,
    triple_id: &str,
    now: i64,
) -> String {
    let result: Option<(i64, i64)> = conn
        .query_row(
            "SELECT
                COUNT(*),
                SUM(CASE WHEN (cp.buried_until IS NULL OR cp.buried_until <= ?3)
                    AND (cp.state = 'new' OR (cp.state IN ('learning','review','relearning') AND cp.due_at <= ?3)) THEN 1 ELSE 0 END)
             FROM card_triples ct
             JOIN cards c ON c.id = ct.card_id
             JOIN card_progress cp ON cp.card_id = c.id AND cp.profile_id = ?2
             WHERE ct.triple_id = ?1",
            rusqlite::params![triple_id, profile_id, now],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();

    match result {
        Some((total, due)) if total > 0 && due > 0 => "due".to_string(),
        Some((total, _)) if total > 0 => "reviewed".to_string(),
        _ => "none".to_string(),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Card suggestions from triples
// ═══════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub fn suggest_cards_from_triples(
    db: State<Database>,
    entity_id: Option<String>,
) -> Result<Vec<CardSuggestion>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut sql = String::from(
        "SELECT t.id, t.subject_id, t.relation_type_id, t.object_id,
                se.name, rt.name, rt.inverse_name, oe.name
         FROM triples t
         JOIN entities se ON se.id = t.subject_id
         JOIN relation_types rt ON rt.id = t.relation_type_id
         JOIN entities oe ON oe.id = t.object_id
         WHERE NOT EXISTS (SELECT 1 FROM card_triples ct WHERE ct.triple_id = t.id) ",
    );

    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    if let Some(ref eid) = entity_id {
        sql.push_str("AND (t.subject_id = ?1 OR t.object_id = ?1) ");
        params.push(Box::new(eid.clone()));
    }
    sql.push_str("ORDER BY se.name, rt.name LIMIT 200");

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows: Vec<(String, String, String, String, String, String, Option<String>, String)> = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok((
                row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?,
                row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // Group by (subject_id, relation_type_id) for forward suggestions
    let mut forward_groups: std::collections::HashMap<(String, String), Vec<(String, String, String, String)>> =
        std::collections::HashMap::new();

    for (tid, sid, rtid, _oid, sname, rname, _inv, oname) in &rows {
        forward_groups
            .entry((sid.clone(), rtid.clone()))
            .or_default()
            .push((tid.clone(), sname.clone(), rname.clone(), oname.clone()));
    }

    let mut suggestions = Vec::new();

    for ((_sid, _rtid), group) in &forward_groups {
        if group.is_empty() {
            continue;
        }
        let subject = &group[0].1;
        let relation = &group[0].2;
        let objects: Vec<&str> = group.iter().map(|g| g.3.as_str()).collect();
        let triple_ids: Vec<String> = group.iter().map(|g| g.0.clone()).collect();

        suggestions.push(CardSuggestion {
            triple_ids,
            front: format!("What {} does {} have?", relation, subject),
            back: objects.join(", "),
            suggestion_type: "forward".to_string(),
        });
    }

    // Reverse suggestions for relations with inverse_name
    let mut reverse_groups: std::collections::HashMap<(String, String), Vec<(String, String, Option<String>, String)>> =
        std::collections::HashMap::new();

    for (tid, _sid, rtid, oid, sname, _rname, inv, _oname) in &rows {
        if inv.is_some() {
            reverse_groups
                .entry((oid.clone(), rtid.clone()))
                .or_default()
                .push((tid.clone(), sname.clone(), inv.clone(), _oname.clone()));
        }
    }

    for ((_oid, _rtid), group) in &reverse_groups {
        if group.is_empty() {
            continue;
        }
        let inv_name = group[0].2.as_deref().unwrap_or("relates to");
        let object_name = &group[0].3;
        let subjects: Vec<&str> = group.iter().map(|g| g.1.as_str()).collect();
        let triple_ids: Vec<String> = group.iter().map(|g| g.0.clone()).collect();

        suggestions.push(CardSuggestion {
            triple_ids,
            front: format!("{} {} which?", object_name, inv_name),
            back: subjects.join(", "),
            suggestion_type: "reverse".to_string(),
        });
    }

    Ok(suggestions)
}

// ═══════════════════════════════════════════════════════════════════════════
// E-R-E Review
// ═══════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub fn get_ere_due_cards(
    db: State<Database>,
    active: State<'_, Mutex<ActiveProfile>>,
    deck_id: String,
    entity_id: Option<String>,
) -> Result<Vec<ReviewCard>, String> {
    let active_guard = active.lock().map_err(|e| e.to_string())?;
    let profile_id = active_guard.id.clone();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp();

    let mut sql = String::from(
        "SELECT DISTINCT c.id, c.note_id, n.note_type_id, nt.is_cloze, c.template_ordinal,
                ct.front_html, ct.back_html, n.fields_json, cp.state, nt.css
         FROM cards c
         JOIN card_progress cp ON cp.card_id = c.id
         JOIN notes n ON n.id = c.note_id
         JOIN note_types nt ON nt.id = n.note_type_id
         JOIN card_templates ct ON ct.id = (
             SELECT ct2.id FROM card_templates ct2
             WHERE ct2.note_type_id = n.note_type_id
               AND ct2.ordinal = (
                 CASE WHEN nt.is_cloze = 1 THEN 0
                      WHEN c.template_ordinal >= 1000 THEN c.template_ordinal / 1000
                      ELSE c.template_ordinal END
               )
             ORDER BY ct2.id DESC
             LIMIT 1
         )
         JOIN card_triples ctr ON ctr.card_id = c.id ",
    );

    let scope = deck_scope_ids(&conn, &deck_id).map_err(|e| e.to_string())?;
    if scope.is_empty() {
        return Ok(Vec::new());
    }

    let deck_ph: String = (0..scope.len()).map(|i| format!("?{}", i + 1)).collect::<Vec<_>>().join(", ");
    let profile_idx = scope.len() + 1;
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> =
        scope.iter().map(|id| Box::new(id.clone()) as Box<dyn rusqlite::types::ToSql>).collect();
    params.push(Box::new(profile_id.clone()));
    let mut idx = scope.len() + 2;

    sql.push_str(&format!(
        "WHERE cp.profile_id = ?{profile_idx} AND n.deck_id IN ({deck_ph}) "
    ));

    sql.push_str(&format!(
        "AND (cp.buried_until IS NULL OR cp.buried_until <= ?{}) ",
        idx
    ));
    params.push(Box::new(now));
    idx += 1;

    sql.push_str(&format!(
        "AND ((cp.state = 'new') OR (cp.state IN ('learning','review','relearning') AND cp.due_at <= ?{})) ",
        idx
    ));
    params.push(Box::new(now));
    idx += 1;

    if let Some(ref eid) = entity_id {
        sql.push_str(&format!(
            "AND EXISTS (SELECT 1 FROM triples trp WHERE trp.id = ctr.triple_id AND (trp.subject_id = ?{} OR trp.object_id = ?{})) ",
            idx, idx + 1
        ));
        params.push(Box::new(eid.clone()));
        params.push(Box::new(eid.clone()));
        idx += 2;
    }
    let _ = idx;

    sql.push_str("ORDER BY CASE cp.state WHEN 'learning' THEN 0 WHEN 'relearning' THEN 1 WHEN 'review' THEN 2 WHEN 'new' THEN 3 END, CASE WHEN cp.state = 'new' THEN random() ELSE cp.due_at END ASC LIMIT 200");

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let cards = stmt
        .query_map(param_refs.as_slice(), |row| {
            let fields_str: String = row.get(7)?;
            Ok(ReviewCard {
                card_id: row.get(0)?,
                note_id: row.get(1)?,
                note_type_id: row.get(2)?,
                is_cloze: row.get::<_, i64>(3)? != 0,
                template_ordinal: row.get(4)?,
                front_html: row.get(5)?,
                back_html: row.get(6)?,
                fields: serde_json::from_str(&fields_str)
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
                state: row.get(8)?,
                css: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(cards)
}

#[tauri::command]
pub fn get_ere_review_summary(
    db: State<Database>,
    active: State<'_, Mutex<ActiveProfile>>,
    deck_id: String,
) -> Result<Vec<EreReviewEntity>, String> {
    let active_guard = active.lock().map_err(|e| e.to_string())?;
    let profile_id = active_guard.id.clone();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp();
    let scope = deck_scope_ids(&conn, &deck_id).map_err(|e| e.to_string())?;
    if scope.is_empty() {
        return Ok(Vec::new());
    }

    let deck_ph = scope.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let sql = format!(
        "SELECT e.id, e.name, COUNT(DISTINCT c.id) as due_count
         FROM entities e
         JOIN triples t ON (t.subject_id = e.id OR t.object_id = e.id)
         JOIN card_triples ctr ON ctr.triple_id = t.id
         JOIN cards c ON c.id = ctr.card_id
         JOIN card_progress cp ON cp.card_id = c.id AND cp.profile_id = ?
         JOIN notes n ON n.id = c.note_id
         WHERE n.deck_id IN ({deck_ph})
           AND (cp.buried_until IS NULL OR cp.buried_until <= ?)
           AND ((cp.state = 'new') OR (cp.state IN ('learning','review','relearning') AND cp.due_at <= ?))
         GROUP BY e.id
         HAVING due_count > 0
         ORDER BY due_count DESC"
    );

    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> =
        scope.iter().map(|id| Box::new(id.clone()) as Box<dyn rusqlite::types::ToSql>).collect();
    params.insert(0, Box::new(profile_id));
    params.push(Box::new(now));
    params.push(Box::new(now));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok(EreReviewEntity {
                entity_id: row.get(0)?,
                entity_name: row.get(1)?,
                due_count: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

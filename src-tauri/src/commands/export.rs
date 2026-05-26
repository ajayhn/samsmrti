use crate::db::Database;
use flate2::write::GzEncoder;
use flate2::Compression;
use rusqlite::types::Value as SqlValue;
use rusqlite::Row;
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::fs;
use std::io::Write;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct ExportResult {
    pub path: String,
    pub notes_exported: usize,
    pub cards_exported: usize,
}

#[derive(Debug, Serialize)]
pub struct FullExportResult {
    pub path: String,
    pub bytes_written: usize,
    pub decks: usize,
    pub notes: usize,
    pub cards: usize,
    pub entities: usize,
    pub triples: usize,
}

fn rusqlite_value_to_json(v: SqlValue) -> Value {
    match v {
        SqlValue::Null => Value::Null,
        SqlValue::Integer(i) => json!(i),
        SqlValue::Real(f) => json!(f),
        SqlValue::Text(s) => Value::String(s),
        SqlValue::Blob(b) => Value::String(String::from_utf8_lossy(&b).into_owned()),
    }
}

fn row_to_json(row: &Row<'_>, columns: &[String]) -> Result<Value, rusqlite::Error> {
    let mut map = Map::new();
    for (i, name) in columns.iter().enumerate() {
        let v: SqlValue = row.get(i)?;
        map.insert(name.clone(), rusqlite_value_to_json(v));
    }
    Ok(Value::Object(map))
}

fn fetch_rows(conn: &rusqlite::Connection, sql: &str) -> Result<Vec<Value>, String> {
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let columns: Vec<String> = stmt
        .column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();
    let rows = stmt
        .query_map([], |row| row_to_json(row, &columns))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
pub fn export_deck_json(
    db: State<Database>,
    deck_id: String,
    file_path: String,
) -> Result<ExportResult, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let deck_name: String = conn
        .query_row("SELECT name FROM decks WHERE id = ?1", [&deck_id], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let mut notes_stmt = conn
        .prepare("SELECT id, note_type_id, fields_json, created_at FROM notes WHERE deck_id = ?1")
        .map_err(|e| e.to_string())?;

    let notes: Vec<serde_json::Value> = notes_stmt
        .query_map([&deck_id], |row| {
            let fields_str: String = row.get(2)?;
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "note_type_id": row.get::<_, String>(1)?,
                "fields": serde_json::from_str::<serde_json::Value>(&fields_str).unwrap_or_default(),
                "created_at": row.get::<_, i64>(3)?,
            }))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut cards_count = 0usize;
    for note in &notes {
        if let Some(nid) = note["id"].as_str() {
            let count: i64 = conn
                .query_row("SELECT COUNT(*) FROM cards WHERE note_id = ?1", [nid], |row| row.get(0))
                .unwrap_or(0);
            cards_count += count as usize;
        }
    }

    let export_data = serde_json::json!({
        "format": "samsmrti-v1",
        "deck_name": deck_name,
        "deck_id": deck_id,
        "exported_at": chrono::Utc::now().timestamp(),
        "notes_count": notes.len(),
        "notes": notes,
    });

    let json_str = serde_json::to_string_pretty(&export_data).map_err(|e| e.to_string())?;
    let mut file = fs::File::create(&file_path).map_err(|e| e.to_string())?;
    file.write_all(json_str.as_bytes())
        .map_err(|e| e.to_string())?;

    Ok(ExportResult {
        path: file_path,
        notes_exported: notes.len(),
        cards_exported: cards_count,
    })
}

/// Export all collection data as gzipped JSON. Excludes `review_log` and per-card study
/// scheduling fields (state, due dates, reps, etc.).
#[tauri::command]
pub fn export_all_gz(db: State<Database>, file_path: String) -> Result<FullExportResult, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let decks = fetch_rows(&conn, "SELECT * FROM decks ORDER BY created_at")?;
    let note_types = fetch_rows(&conn, "SELECT * FROM note_types ORDER BY created_at")?;
    let fields = fetch_rows(&conn, "SELECT * FROM fields ORDER BY note_type_id, ordinal")?;
    let card_templates =
        fetch_rows(&conn, "SELECT * FROM card_templates ORDER BY note_type_id, ordinal")?;
    let notes = fetch_rows(&conn, "SELECT * FROM notes ORDER BY created_at")?;
    let cards = fetch_rows(
        &conn,
        "SELECT id, note_id, template_ordinal FROM cards ORDER BY note_id, template_ordinal",
    )?;
    let tags = fetch_rows(&conn, "SELECT * FROM tags ORDER BY name")?;
    let note_tags = fetch_rows(&conn, "SELECT * FROM note_tags")?;
    let note_links = fetch_rows(&conn, "SELECT * FROM note_links ORDER BY created_at")?;
    let entities = fetch_rows(&conn, "SELECT * FROM entities ORDER BY created_at")?;
    let relation_types =
        fetch_rows(&conn, "SELECT * FROM relation_types ORDER BY created_at")?;
    let triples = fetch_rows(&conn, "SELECT * FROM triples ORDER BY created_at")?;
    let card_triples = fetch_rows(&conn, "SELECT * FROM card_triples")?;
    let entity_tags = fetch_rows(&conn, "SELECT * FROM entity_tags")?;
    let relation_type_tags = fetch_rows(&conn, "SELECT * FROM relation_type_tags")?;

    let deck_count = decks.len();
    let note_count = notes.len();
    let card_count = cards.len();
    let entity_count = entities.len();
    let triple_count = triples.len();

    let export_data = json!({
        "format": "samsmrti-backup-v1",
        "exported_at": chrono::Utc::now().timestamp(),
        "excludes": ["review_log", "card_scheduling", "notes_fts"],
        "decks": decks,
        "note_types": note_types,
        "fields": fields,
        "card_templates": card_templates,
        "notes": notes,
        "cards": cards,
        "tags": tags,
        "note_tags": note_tags,
        "note_links": note_links,
        "entities": entities,
        "relation_types": relation_types,
        "triples": triples,
        "card_triples": card_triples,
        "entity_tags": entity_tags,
        "relation_type_tags": relation_type_tags,
    });

    let json_bytes = serde_json::to_vec(&export_data).map_err(|e| e.to_string())?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(&json_bytes)
        .map_err(|e| e.to_string())?;
    let compressed = encoder.finish().map_err(|e| e.to_string())?;
    let bytes_written = compressed.len();
    fs::write(&file_path, &compressed).map_err(|e| e.to_string())?;

    Ok(FullExportResult {
        path: file_path,
        bytes_written,
        decks: deck_count,
        notes: note_count,
        cards: card_count,
        entities: entity_count,
        triples: triple_count,
    })
}

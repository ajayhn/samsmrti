use crate::commands::search::ensure_search_index_conn;
use crate::db::card_progress;
use flate2::read::GzDecoder;
use rusqlite::types::Value as SqlValue;
use rusqlite::{Connection, Row};
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::fs;
use std::io::Read;
use std::path::Path;

pub const FORMAT_CONTENT_V1: &str = "samsmrti-content-v1";
pub const FORMAT_LEGACY_BACKUP_V1: &str = "samsmrti-backup-v1";

#[derive(Debug, Serialize)]
pub struct ContentExportSummary {
    pub path: String,
    pub decks: usize,
    pub notes: usize,
    pub cards: usize,
    pub entities: usize,
    pub triples: usize,
}

#[derive(Debug, Serialize, Default)]
pub struct ContentImportSummary {
    pub decks_added: usize,
    pub notes_added: usize,
    pub cards_added: usize,
    pub entities_added: usize,
    pub triples_added: usize,
    pub rows_skipped: usize,
    pub warnings: Vec<String>,
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

fn fetch_rows(conn: &Connection, sql: &str) -> Result<Vec<Value>, String> {
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

pub fn build_content_payload(conn: &Connection) -> Result<Value, String> {
    let decks = fetch_rows(conn, "SELECT * FROM decks ORDER BY created_at")?;
    let note_types = fetch_rows(conn, "SELECT * FROM note_types ORDER BY created_at")?;
    let fields = fetch_rows(conn, "SELECT * FROM fields ORDER BY note_type_id, ordinal")?;
    let card_templates =
        fetch_rows(conn, "SELECT * FROM card_templates ORDER BY note_type_id, ordinal")?;
    let notes = fetch_rows(conn, "SELECT * FROM notes ORDER BY created_at")?;
    let cards = fetch_rows(
        conn,
        "SELECT id, note_id, template_ordinal FROM cards ORDER BY note_id, template_ordinal",
    )?;
    let tags = fetch_rows(conn, "SELECT * FROM tags ORDER BY name")?;
    let note_tags = fetch_rows(conn, "SELECT * FROM note_tags")?;
    let note_links = fetch_rows(conn, "SELECT * FROM note_links ORDER BY created_at")?;
    let entities = fetch_rows(conn, "SELECT * FROM entities ORDER BY created_at")?;
    let relation_types =
        fetch_rows(conn, "SELECT * FROM relation_types ORDER BY created_at")?;
    let triples = fetch_rows(conn, "SELECT * FROM triples ORDER BY created_at")?;
    let card_triples = fetch_rows(conn, "SELECT * FROM card_triples")?;
    let entity_tags = fetch_rows(conn, "SELECT * FROM entity_tags")?;
    let relation_type_tags = fetch_rows(conn, "SELECT * FROM relation_type_tags")?;

    Ok(json!({
        "format": FORMAT_CONTENT_V1,
        "exported_at": chrono::Utc::now().timestamp(),
        "excludes": [
            "review_log",
            "card_progress",
            "profiles",
            "karma_state",
            "karma_daily",
            "app_settings",
            "notes_fts",
            "card_scheduling"
        ],
        "decks": decks,
        "note_types": note_types,
        "fields": fields,
        "card_templates": card_templates,
        "notes": notes,
        "tags": tags,
        "note_tags": note_tags,
        "note_links": note_links,
        "entities": entities,
        "relation_types": relation_types,
        "triples": triples,
        "cards": cards,
        "card_triples": card_triples,
        "entity_tags": entity_tags,
        "relation_type_tags": relation_type_tags,
    }))
}

fn read_content_json_file(path: &Path) -> Result<Value, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let is_gz = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("gz"))
        .unwrap_or(false)
        || bytes.starts_with(&[0x1f, 0x8b]);

    let json_bytes = if is_gz {
        let mut decoder = GzDecoder::new(&bytes[..]);
        let mut out = Vec::new();
        decoder
            .read_to_end(&mut out)
            .map_err(|e| e.to_string())?;
        out
    } else {
        bytes
    };

    serde_json::from_slice(&json_bytes).map_err(|e| e.to_string())
}

fn sort_decks_parent_first(decks: &[Value]) -> Vec<Value> {
    let mut remaining: Vec<Value> = decks.to_vec();
    let mut sorted = Vec::new();
    let mut inserted_ids = std::collections::HashSet::new();

    for _ in 0..decks.len().saturating_add(1) {
        if remaining.is_empty() {
            break;
        }
        let mut next = Vec::new();
        for deck in remaining.drain(..) {
            let parent = deck.get("parent_id").and_then(|v| {
                if v.is_null() {
                    None
                } else {
                    v.as_str().filter(|s| !s.is_empty())
                }
            });
            let can_insert = parent.is_none() || inserted_ids.contains(parent.unwrap());
            if can_insert {
                if let Some(id) = deck.get("id").and_then(|v| v.as_str()) {
                    inserted_ids.insert(id.to_string());
                }
                sorted.push(deck);
            } else {
                next.push(deck);
            }
        }
        if next.is_empty() {
            break;
        }
        remaining = next;
    }
    if !remaining.is_empty() {
        sorted.extend(remaining);
    }
    sorted
}

fn insert_rows(
    conn: &Connection,
    table: &str,
    rows: &[Value],
    summary: &mut ContentImportSummary,
) -> Result<usize, String> {
    let mut added = 0usize;
    for row in rows {
        let obj = row
            .as_object()
            .ok_or_else(|| format!("Invalid row in {table}"))?;
        if obj.is_empty() {
            continue;
        }
        let cols: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
        let placeholders: Vec<String> = (1..=cols.len()).map(|i| format!("?{i}")).collect();
        let sql = format!(
            "INSERT OR IGNORE INTO {table} ({}) VALUES ({})",
            cols.join(", "),
            placeholders.join(", ")
        );
        let params: Vec<SqlValue> = cols
            .iter()
            .map(|c| json_value_to_sql(obj.get(*c).unwrap_or(&Value::Null)))
            .collect();
        match conn.execute(&sql, rusqlite::params_from_iter(params.iter())) {
            Ok(n) if n > 0 => added += 1,
            Ok(_) => summary.rows_skipped += 1,
            Err(e) => summary.warnings.push(format!("{table}: {e}")),
        }
    }
    Ok(added)
}

fn json_value_to_sql(v: &Value) -> SqlValue {
    match v {
        Value::Null => SqlValue::Null,
        Value::Bool(b) => SqlValue::Integer(if *b { 1 } else { 0 }),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                SqlValue::Integer(i)
            } else {
                SqlValue::Real(n.as_f64().unwrap_or(0.0))
            }
        }
        Value::String(s) => SqlValue::Text(s.clone()),
        _ => SqlValue::Text(v.to_string()),
    }
}

fn import_cards(
    conn: &Connection,
    rows: &[Value],
    summary: &mut ContentImportSummary,
) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp();
    for row in rows {
        let obj = row.as_object().ok_or("Invalid card row")?;
        let card_id = obj
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or("Card missing id")?;
        let note_id = obj
            .get("note_id")
            .and_then(|v| v.as_str())
            .ok_or("Card missing note_id")?;
        let ordinal = obj
            .get("template_ordinal")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let n = conn
            .execute(
                "INSERT OR IGNORE INTO cards (
                    id, note_id, template_ordinal, state, difficulty, stability,
                    due_at, last_review_at, reps, lapses, buried_until
                 ) VALUES (?1, ?2, ?3, 'new', 0.0, 0.0, ?4, NULL, 0, 0, NULL)",
                (card_id, note_id, ordinal, now),
            )
            .map_err(|e| e.to_string())?;
        if n > 0 {
            summary.cards_added += 1;
            card_progress::seed_progress_for_all_profiles(conn, card_id, now)
                .map_err(|e| e.to_string())?;
        } else {
            summary.rows_skipped += 1;
        }
    }
    Ok(())
}

pub fn import_content_payload(conn: &Connection, data: &Value) -> Result<ContentImportSummary, String> {
    let format = data
        .get("format")
        .and_then(|v| v.as_str())
        .ok_or("Missing format field")?;
    if format != FORMAT_CONTENT_V1 && format != FORMAT_LEGACY_BACKUP_V1 {
        return Err(format!("Unsupported export format: {format}"));
    }

    let mut summary = ContentImportSummary::default();
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| e.to_string())?;

    let decks = data.get("decks").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let sorted_decks = sort_decks_parent_first(&decks);
    let _ = insert_rows(
        &tx,
        "note_types",
        data.get("note_types")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
        &mut summary,
    )?;
    let _ = insert_rows(
        &tx,
        "fields",
        data.get("fields")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
        &mut summary,
    )?;
    let _ = insert_rows(
        &tx,
        "card_templates",
        data.get("card_templates")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
        &mut summary,
    )?;

    summary.decks_added = insert_rows(&tx, "decks", &sorted_decks, &mut summary)?;

    let _ = insert_rows(
        &tx,
        "tags",
        data.get("tags")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
        &mut summary,
    )?;

    summary.entities_added += insert_rows(
        &tx,
        "entities",
        data.get("entities")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
        &mut summary,
    )?;

    let _ = insert_rows(
        &tx,
        "relation_types",
        data.get("relation_types")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
        &mut summary,
    )?;

    summary.notes_added += insert_rows(
        &tx,
        "notes",
        data.get("notes")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
        &mut summary,
    )?;

    import_cards(
        &tx,
        data.get("cards")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
        &mut summary,
    )?;

    let _ = insert_rows(
        &tx,
        "note_tags",
        data.get("note_tags")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
        &mut summary,
    )?;
    let _ = insert_rows(
        &tx,
        "note_links",
        data.get("note_links")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
        &mut summary,
    )?;
    summary.triples_added += insert_rows(
        &tx,
        "triples",
        data.get("triples")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
        &mut summary,
    )?;
    let _ = insert_rows(
        &tx,
        "card_triples",
        data.get("card_triples")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
        &mut summary,
    )?;
    let _ = insert_rows(&tx, "entity_tags", data.get("entity_tags").and_then(|v| v.as_array()).map(|a| a.as_slice()).unwrap_or(&[]), &mut summary)?;
    let _ = insert_rows(&tx, "relation_type_tags", data.get("relation_type_tags").and_then(|v| v.as_array()).map(|a| a.as_slice()).unwrap_or(&[]), &mut summary)?;

    tx.commit().map_err(|e| e.to_string())?;

    let _ = ensure_search_index_conn(conn);

    Ok(summary)
}

pub fn export_content_json_file(conn: &Connection, file_path: &str) -> Result<ContentExportSummary, String> {
    let payload = build_content_payload(conn)?;
    let json_str = serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?;
    fs::write(file_path, json_str).map_err(|e| e.to_string())?;

    Ok(ContentExportSummary {
        path: file_path.to_string(),
        decks: payload["decks"].as_array().map(|a| a.len()).unwrap_or(0),
        notes: payload["notes"].as_array().map(|a| a.len()).unwrap_or(0),
        cards: payload["cards"].as_array().map(|a| a.len()).unwrap_or(0),
        entities: payload["entities"].as_array().map(|a| a.len()).unwrap_or(0),
        triples: payload["triples"].as_array().map(|a| a.len()).unwrap_or(0),
    })
}

pub fn import_content_file(conn: &Connection, file_path: &str) -> Result<ContentImportSummary, String> {
    let data = read_content_json_file(Path::new(file_path))?;
    import_content_payload(conn, &data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    #[test]
    fn content_export_import_roundtrip() {
        let db = Database::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO decks (id, name, description, new_per_day, max_reviews, created_at, updated_at)
             VALUES ('dk_test', 'Test', '', 20, 200, ?1, ?1)",
            [now],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO notes (id, deck_id, note_type_id, fields_json, created_at, updated_at)
             VALUES ('n1', 'dk_test', 'nt_basic', '{\"Front\":\"Q\",\"Back\":\"A\"}', ?1, ?1)",
            [now],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO cards (id, note_id, template_ordinal, state, due_at)
             VALUES ('c1', 'n1', 0, 'new', ?1)",
            [now],
        )
        .unwrap();
        card_progress::seed_progress_for_all_profiles(&conn, "c1", now).unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("content.json");
        export_content_json_file(&conn, path.to_str().unwrap()).unwrap();

        conn.execute("DELETE FROM card_progress", []).unwrap();
        conn.execute("DELETE FROM cards", []).unwrap();
        conn.execute("DELETE FROM notes", []).unwrap();
        conn.execute("DELETE FROM decks", []).unwrap();

        let summary = import_content_file(&conn, path.to_str().unwrap()).unwrap();
        assert_eq!(summary.decks_added, 1);
        assert_eq!(summary.notes_added, 1);
        assert_eq!(summary.cards_added, 1);

        let progress: i64 = conn
            .query_row("SELECT COUNT(*) FROM card_progress WHERE card_id = 'c1'", [], |r| r.get(0))
            .unwrap();
        assert!(progress > 0);
    }
}

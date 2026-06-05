use crate::commands::search::ensure_search_index_conn;
use crate::db::card_progress;
use crate::db::deck_tree::{deck_scope_ids, rollup_deck_counts};
use flate2::read::GzDecoder;
use rusqlite::types::Value as SqlValue;
use rusqlite::{Connection, Row};
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::Path;

pub const FORMAT_CONTENT_V1: &str = "samsmrti-content-v1";
pub const FORMAT_LEGACY_BACKUP_V1: &str = "samsmrti-backup-v1";
pub const FORMAT_DECK_UNDO_V1: &str = "samsmrti-deck-undo-v1";

#[derive(Debug, Serialize)]
pub struct ContentExportSummary {
    pub path: String,
    pub decks: usize,
    pub notes: usize,
    pub cards: usize,
    pub entities: usize,
    pub triples: usize,
}

#[derive(Debug, Serialize, Clone)]
pub struct ContentDeckPreview {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub note_count: usize,
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

/// `None` or empty selection = all decks.
pub fn resolve_deck_scope(
    conn: &Connection,
    selected_deck_ids: Option<&[String]>,
) -> Result<Option<HashSet<String>>, String> {
    let Some(ids) = selected_deck_ids else {
        return Ok(None);
    };
    if ids.is_empty() {
        return Ok(None);
    }

    let mut scope = HashSet::new();
    for id in ids {
        for deck_id in deck_scope_ids(conn, id).map_err(|e| e.to_string())? {
            add_deck_and_ancestors(conn, &deck_id, &mut scope)?;
        }
    }
    Ok(Some(scope))
}

fn add_deck_and_ancestors(
    conn: &Connection,
    deck_id: &str,
    scope: &mut HashSet<String>,
) -> Result<(), String> {
    let mut current = Some(deck_id.to_string());
    while let Some(did) = current {
        if !scope.insert(did.clone()) {
            break;
        }
        let parent: Option<String> = conn
            .query_row(
                "SELECT parent_id FROM decks WHERE id = ?1",
                [&did],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        current = parent.filter(|p| !p.is_empty());
    }
    Ok(())
}

fn resolve_deck_scope_from_payload(
    data: &Value,
    selected_deck_ids: Option<&[String]>,
) -> Option<HashSet<String>> {
    let ids = selected_deck_ids?;
    if ids.is_empty() {
        return None;
    }

    let decks = data.get("decks")?.as_array()?;

    let mut parent_of: HashMap<String, Option<String>> = HashMap::new();
    let mut children_of: HashMap<String, Vec<String>> = HashMap::new();

    for deck in decks {
        let Some(id) = deck.get("id").and_then(|v| v.as_str()) else {
            continue;
        };
        let id = id.to_string();
        let parent_id = deck.get("parent_id").and_then(|v| {
            if v.is_null() {
                None
            } else {
                v.as_str().map(str::to_string)
            }
        });
        parent_of.insert(id.clone(), parent_id.clone());
        children_of.entry(id.clone()).or_default();
        if let Some(p) = parent_id {
            children_of.entry(p).or_default().push(id);
        }
    }

    let mut scope = HashSet::new();
    for root in ids {
        let mut stack = vec![root.clone()];
        while let Some(id) = stack.pop() {
            if scope.insert(id.clone()) {
                if let Some(kids) = children_of.get(&id) {
                    stack.extend(kids.iter().cloned());
                }
            }
        }
        let mut current = Some(root.as_str());
        while let Some(did) = current {
            scope.insert(did.to_string());
            current = parent_of
                .get(did)
                .and_then(|p| p.as_deref());
        }
    }

    Some(scope)
}

fn row_in_deck_scope(row: &Value, deck_field: &str, scope: &HashSet<String>) -> bool {
    row.get(deck_field)
        .and_then(|v| v.as_str())
        .map(|id| scope.contains(id))
        .unwrap_or(false)
}

fn filter_content_payload(data: Value, scope: &HashSet<String>) -> Value {
    let Some(mut obj) = data.as_object().cloned() else {
        return data;
    };

    let decks: Vec<Value> = obj
        .get("decks")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|d| {
            d.get("id")
                .and_then(|v| v.as_str())
                .map(|id| scope.contains(id))
                .unwrap_or(false)
        })
        .collect();

    let notes: Vec<Value> = obj
        .get("notes")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|n| row_in_deck_scope(n, "deck_id", scope))
        .collect();

    let note_ids: HashSet<String> = notes
        .iter()
        .filter_map(|n| n.get("id").and_then(|v| v.as_str()).map(str::to_string))
        .collect();

    let note_type_ids: HashSet<String> = notes
        .iter()
        .filter_map(|n| {
            n.get("note_type_id")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .collect();

    let cards: Vec<Value> = obj
        .get("cards")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|c| {
            c.get("note_id")
                .and_then(|v| v.as_str())
                .map(|id| note_ids.contains(id))
                .unwrap_or(false)
        })
        .collect();

    let card_ids: HashSet<String> = cards
        .iter()
        .filter_map(|c| c.get("id").and_then(|v| v.as_str()).map(str::to_string))
        .collect();

    let note_tags: Vec<Value> = obj
        .get("note_tags")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|nt| {
            nt.get("note_id")
                .and_then(|v| v.as_str())
                .map(|id| note_ids.contains(id))
                .unwrap_or(false)
        })
        .collect();

    let tag_ids: HashSet<String> = note_tags
        .iter()
        .filter_map(|nt| nt.get("tag_id").and_then(|v| v.as_str()).map(str::to_string))
        .collect();

    let tags: Vec<Value> = obj
        .get("tags")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|t| {
            t.get("id")
                .and_then(|v| v.as_str())
                .map(|id| tag_ids.contains(id))
                .unwrap_or(false)
        })
        .collect();

    let note_links: Vec<Value> = obj
        .get("note_links")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|nl| {
            let src = nl
                .get("source_note_id")
                .and_then(|v| v.as_str())
                .map(|id| note_ids.contains(id))
                .unwrap_or(false);
            let tgt = nl
                .get("target_note_id")
                .and_then(|v| v.as_str())
                .map(|id| note_ids.contains(id))
                .unwrap_or(false);
            src && tgt
        })
        .collect();

    let card_triples: Vec<Value> = obj
        .get("card_triples")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|ct| {
            ct.get("card_id")
                .and_then(|v| v.as_str())
                .map(|id| card_ids.contains(id))
                .unwrap_or(false)
        })
        .collect();

    let triple_ids: HashSet<String> = card_triples
        .iter()
        .filter_map(|ct| ct.get("triple_id").and_then(|v| v.as_str()).map(str::to_string))
        .collect();

    let triples: Vec<Value> = obj
        .get("triples")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|t| {
            t.get("id")
                .and_then(|v| v.as_str())
                .map(|id| triple_ids.contains(id))
                .unwrap_or(false)
        })
        .collect();

    let entity_ids: HashSet<String> = triples
        .iter()
        .flat_map(|t| {
            [
                t.get("subject_id").and_then(|v| v.as_str()),
                t.get("object_id").and_then(|v| v.as_str()),
            ]
        })
        .flatten()
        .map(str::to_string)
        .collect();

    let relation_type_ids: HashSet<String> = triples
        .iter()
        .filter_map(|t| {
            t.get("relation_type_id")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .collect();

    let entities: Vec<Value> = obj
        .get("entities")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|e| {
            e.get("id")
                .and_then(|v| v.as_str())
                .map(|id| entity_ids.contains(id))
                .unwrap_or(false)
        })
        .collect();

    let relation_types: Vec<Value> = obj
        .get("relation_types")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|rt| {
            rt.get("id")
                .and_then(|v| v.as_str())
                .map(|id| relation_type_ids.contains(id))
                .unwrap_or(false)
        })
        .collect();

    let entity_tags: Vec<Value> = obj
        .get("entity_tags")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|et| {
            et.get("entity_id")
                .and_then(|v| v.as_str())
                .map(|id| entity_ids.contains(id))
                .unwrap_or(false)
        })
        .collect();

    let relation_type_tags: Vec<Value> = obj
        .get("relation_type_tags")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|rtt| {
            rtt.get("relation_type_id")
                .and_then(|v| v.as_str())
                .map(|id| relation_type_ids.contains(id))
                .unwrap_or(false)
        })
        .collect();

    let note_types: Vec<Value> = obj
        .get("note_types")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|nt| {
            nt.get("id")
                .and_then(|v| v.as_str())
                .map(|id| note_type_ids.contains(id))
                .unwrap_or(false)
        })
        .collect();

    let fields: Vec<Value> = obj
        .get("fields")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|f| {
            f.get("note_type_id")
                .and_then(|v| v.as_str())
                .map(|id| note_type_ids.contains(id))
                .unwrap_or(false)
        })
        .collect();

    let card_templates: Vec<Value> = obj
        .get("card_templates")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|ct| {
            ct.get("note_type_id")
                .and_then(|v| v.as_str())
                .map(|id| note_type_ids.contains(id))
                .unwrap_or(false)
        })
        .collect();

    obj.insert("decks".into(), Value::Array(decks));
    obj.insert("note_types".into(), Value::Array(note_types));
    obj.insert("fields".into(), Value::Array(fields));
    obj.insert("card_templates".into(), Value::Array(card_templates));
    obj.insert("notes".into(), Value::Array(notes));
    obj.insert("cards".into(), Value::Array(cards));
    obj.insert("tags".into(), Value::Array(tags));
    obj.insert("note_tags".into(), Value::Array(note_tags));
    obj.insert("note_links".into(), Value::Array(note_links));
    obj.insert("entities".into(), Value::Array(entities));
    obj.insert("relation_types".into(), Value::Array(relation_types));
    obj.insert("triples".into(), Value::Array(triples));
    obj.insert("card_triples".into(), Value::Array(card_triples));
    obj.insert("entity_tags".into(), Value::Array(entity_tags));
    obj.insert(
        "relation_type_tags".into(),
        Value::Array(relation_type_tags),
    );

    Value::Object(obj)
}

fn rollup_note_counts(
    deck_ids: &[String],
    parent_of: &HashMap<String, Option<String>>,
    direct: &HashMap<String, usize>,
) -> HashMap<String, usize> {
    let direct_tuples: HashMap<String, (i64, i64, i64)> = direct
        .iter()
        .map(|(id, &n)| (id.clone(), (n as i64, 0, 0)))
        .collect();
    rollup_deck_counts(deck_ids, parent_of, &direct_tuples)
        .into_iter()
        .map(|(id, (n, _, _))| (id, n as usize))
        .collect()
}

pub fn list_decks_from_payload(data: &Value) -> Result<Vec<ContentDeckPreview>, String> {
    let format = data
        .get("format")
        .and_then(|v| v.as_str())
        .ok_or("Missing format field")?;
    if format != FORMAT_CONTENT_V1 && format != FORMAT_LEGACY_BACKUP_V1 {
        return Err(format!("Unsupported export format: {format}"));
    }

    let decks = data
        .get("decks")
        .and_then(|v| v.as_array())
        .ok_or("Missing decks array")?;

    let mut direct_notes: HashMap<String, usize> = HashMap::new();
    if let Some(notes) = data.get("notes").and_then(|v| v.as_array()) {
        for note in notes {
            if let Some(deck_id) = note.get("deck_id").and_then(|v| v.as_str()) {
                *direct_notes.entry(deck_id.to_string()).or_default() += 1;
            }
        }
    }

    let mut deck_ids = Vec::new();
    let mut parent_of: HashMap<String, Option<String>> = HashMap::new();
    for deck in decks {
        let id = deck
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or("Deck missing id")?
            .to_string();
        let parent_id = deck.get("parent_id").and_then(|v| {
            if v.is_null() {
                None
            } else {
                v.as_str().map(str::to_string)
            }
        });
        deck_ids.push(id.clone());
        parent_of.insert(id, parent_id);
    }
    let note_count_by_deck = rollup_note_counts(&deck_ids, &parent_of, &direct_notes);

    let mut out = Vec::new();
    for deck in decks {
        let id = deck
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or("Deck missing id")?
            .to_string();
        let name = deck
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("(unnamed)")
            .to_string();
        let parent_id = parent_of.get(&id).cloned().flatten();
        out.push(ContentDeckPreview {
            note_count: note_count_by_deck.get(&id).copied().unwrap_or(0),
            id,
            name,
            parent_id,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

pub fn build_content_payload(
    conn: &Connection,
    selected_deck_ids: Option<&[String]>,
) -> Result<Value, String> {
    let payload = build_content_payload_unfiltered(conn)?;
    if let Some(scope) = resolve_deck_scope(conn, selected_deck_ids)? {
        Ok(filter_content_payload(payload, &scope))
    } else {
        Ok(payload)
    }
}

fn build_content_payload_unfiltered(conn: &Connection) -> Result<Value, String> {
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

pub fn import_content_payload(
    conn: &Connection,
    data: &Value,
    selected_deck_ids: Option<&[String]>,
) -> Result<ContentImportSummary, String> {
    let format = data
        .get("format")
        .and_then(|v| v.as_str())
        .ok_or("Missing format field")?;
    if format != FORMAT_CONTENT_V1 && format != FORMAT_LEGACY_BACKUP_V1 {
        return Err(format!("Unsupported export format: {format}"));
    }

    let data = if let Some(scope) = resolve_deck_scope_from_payload(&data, selected_deck_ids) {
        filter_content_payload(data.clone(), &scope)
    } else {
        data.clone()
    };

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

fn fetch_rows_where_in(
    conn: &Connection,
    table: &str,
    column: &str,
    ids: &[String],
) -> Result<Vec<Value>, String> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let sql = format!("SELECT * FROM {table} WHERE {column} IN ({placeholders})");
    fetch_rows_with_params(conn, &sql, ids)
}

fn fetch_rows_with_params(
    conn: &Connection,
    sql: &str,
    params: &[String],
) -> Result<Vec<Value>, String> {
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let columns: Vec<String> = stmt
        .column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();
    let sql_params: Vec<SqlValue> = params.iter().map(|s| SqlValue::Text(s.clone())).collect();
    let rows = stmt
        .query_map(rusqlite::params_from_iter(sql_params.iter()), |row| {
            row_to_json(row, &columns)
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn scoped_card_ids_from_content(content: &Value) -> Vec<String> {
    content
        .get("cards")
        .and_then(|v| v.as_array())
        .map(|cards| {
            cards
                .iter()
                .filter_map(|c| c.get("id").and_then(|v| v.as_str()).map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn insert_rows_or_replace(
    conn: &Connection,
    table: &str,
    rows: &[Value],
) -> Result<(), String> {
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
            "INSERT OR REPLACE INTO {table} ({}) VALUES ({})",
            cols.join(", "),
            placeholders.join(", ")
        );
        let params: Vec<SqlValue> = cols
            .iter()
            .map(|c| json_value_to_sql(obj.get(*c).unwrap_or(&Value::Null)))
            .collect();
        conn.execute(&sql, rusqlite::params_from_iter(params.iter()))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Capture a deck subtree (and study state) before deletion so it can be restored.
pub fn build_deck_delete_snapshot(conn: &Connection, root_id: &str) -> Result<Value, String> {
    let mut content = build_content_payload(conn, Some(&[root_id.to_string()]))?;
    let card_ids = scoped_card_ids_from_content(&content);

    if !card_ids.is_empty() {
        content["cards"] = Value::Array(fetch_rows_where_in(conn, "cards", "id", &card_ids)?);
    }

    let card_progress = fetch_rows_where_in(conn, "card_progress", "card_id", &card_ids)?;
    let review_log = fetch_rows_where_in(conn, "review_log", "card_id", &card_ids)?;
    let card_flags = fetch_rows_where_in(conn, "card_flags", "card_id", &card_ids)?;

    Ok(json!({
        "format": FORMAT_DECK_UNDO_V1,
        "root_deck_id": root_id,
        "content": content,
        "card_progress": card_progress,
        "review_log": review_log,
        "card_flags": card_flags,
    }))
}

/// Restore a deck subtree previously captured by [`build_deck_delete_snapshot`].
pub fn restore_deck_delete_snapshot(conn: &Connection, snapshot: &Value) -> Result<(), String> {
    let format = snapshot
        .get("format")
        .and_then(|v| v.as_str())
        .ok_or("Missing format field")?;
    if format != FORMAT_DECK_UNDO_V1 {
        return Err(format!("Unsupported deck undo format: {format}"));
    }

    let content = snapshot
        .get("content")
        .ok_or("Missing content in deck undo snapshot")?;

    let mut summary = ContentImportSummary::default();
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| e.to_string())?;

    let decks = content
        .get("decks")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let sorted_decks = sort_decks_parent_first(&decks);

    let _ = insert_rows(
        &tx,
        "note_types",
        content
            .get("note_types")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
        &mut summary,
    )?;
    let _ = insert_rows(
        &tx,
        "fields",
        content
            .get("fields")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
        &mut summary,
    )?;
    let _ = insert_rows(
        &tx,
        "card_templates",
        content
            .get("card_templates")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
        &mut summary,
    )?;
    let _ = insert_rows(&tx, "decks", &sorted_decks, &mut summary)?;
    let _ = insert_rows(
        &tx,
        "tags",
        content
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
        &mut summary,
    )?;
    let _ = insert_rows(
        &tx,
        "entities",
        content
            .get("entities")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
        &mut summary,
    )?;
    let _ = insert_rows(
        &tx,
        "relation_types",
        content
            .get("relation_types")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
        &mut summary,
    )?;
    let _ = insert_rows(
        &tx,
        "notes",
        content
            .get("notes")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
        &mut summary,
    )?;

    let cards = content
        .get("cards")
        .and_then(|v| v.as_array())
        .map(|a| a.as_slice())
        .unwrap_or(&[]);
    let _ = insert_rows(&tx, "cards", cards, &mut summary)?;

    let _ = insert_rows(
        &tx,
        "note_tags",
        content
            .get("note_tags")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
        &mut summary,
    )?;
    let _ = insert_rows(
        &tx,
        "note_links",
        content
            .get("note_links")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
        &mut summary,
    )?;
    let _ = insert_rows(
        &tx,
        "triples",
        content
            .get("triples")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
        &mut summary,
    )?;
    let _ = insert_rows(
        &tx,
        "card_triples",
        content
            .get("card_triples")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
        &mut summary,
    )?;
    let _ = insert_rows(
        &tx,
        "entity_tags",
        content
            .get("entity_tags")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
        &mut summary,
    )?;
    let _ = insert_rows(
        &tx,
        "relation_type_tags",
        content
            .get("relation_type_tags")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
        &mut summary,
    )?;

    let card_ids = scoped_card_ids_from_content(content);
    if !card_ids.is_empty() {
        let placeholders = card_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let sql = format!("DELETE FROM card_progress WHERE card_id IN ({placeholders})");
        let params: Vec<SqlValue> = card_ids.iter().map(|s| SqlValue::Text(s.clone())).collect();
        tx.execute(&sql, rusqlite::params_from_iter(params.iter()))
            .map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;

    insert_rows_or_replace(
        conn,
        "card_progress",
        snapshot
            .get("card_progress")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
    )?;
    let _ = insert_rows(
        conn,
        "review_log",
        snapshot
            .get("review_log")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
        &mut summary,
    );
    let _ = insert_rows(
        conn,
        "card_flags",
        snapshot
            .get("card_flags")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]),
        &mut summary,
    );

    let _ = ensure_search_index_conn(conn);

    Ok(())
}

pub fn export_content_json_file(
    conn: &Connection,
    file_path: &str,
    selected_deck_ids: Option<&[String]>,
) -> Result<ContentExportSummary, String> {
    let payload = build_content_payload(conn, selected_deck_ids)?;
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

pub fn preview_content_import_file(file_path: &str) -> Result<Vec<ContentDeckPreview>, String> {
    let data = read_content_json_file(Path::new(file_path))?;
    list_decks_from_payload(&data)
}

pub fn import_content_file(
    conn: &Connection,
    file_path: &str,
    selected_deck_ids: Option<&[String]>,
) -> Result<ContentImportSummary, String> {
    let data = read_content_json_file(Path::new(file_path))?;
    import_content_payload(conn, &data, selected_deck_ids)
}

pub fn list_export_decks(conn: &Connection) -> Result<Vec<ContentDeckPreview>, String> {
    let decks = fetch_rows(conn, "SELECT * FROM decks ORDER BY name")?;

    let mut direct_notes: HashMap<String, usize> = HashMap::new();
    let mut stmt = conn
        .prepare("SELECT deck_id, COUNT(*) FROM notes GROUP BY deck_id")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
        })
        .map_err(|e| e.to_string())?;
    for row in rows {
        let (id, count) = row.map_err(|e| e.to_string())?;
        direct_notes.insert(id, count);
    }

    let mut deck_ids = Vec::new();
    let mut parent_of: HashMap<String, Option<String>> = HashMap::new();
    for deck in &decks {
        let obj = deck.as_object().ok_or("Invalid deck row")?;
        let id = obj
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or("Deck missing id")?
            .to_string();
        let parent_id = obj.get("parent_id").and_then(|v| {
            if v.is_null() {
                None
            } else {
                v.as_str().map(str::to_string)
            }
        });
        deck_ids.push(id.clone());
        parent_of.insert(id, parent_id);
    }
    let note_count_by_deck = rollup_note_counts(&deck_ids, &parent_of, &direct_notes);

    let mut out = Vec::new();
    for deck in decks {
        let obj = deck.as_object().ok_or("Invalid deck row")?;
        let id = obj
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or("Deck missing id")?
            .to_string();
        let name = obj
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("(unnamed)")
            .to_string();
        let parent_id = parent_of.get(&id).cloned().flatten();
        out.push(ContentDeckPreview {
            note_count: note_count_by_deck.get(&id).copied().unwrap_or(0),
            id,
            name,
            parent_id,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    #[test]
    fn list_export_decks_rollup_includes_subdecks() {
        let db = Database::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO decks (id, name, description, new_per_day, max_reviews, created_at, updated_at)
             VALUES ('dk_parent', 'Chemistry', '', 20, 200, ?1, ?1)",
            [now],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO decks (id, name, parent_id, description, new_per_day, max_reviews, created_at, updated_at)
             VALUES ('dk_child', 'Polyatomic Ions', 'dk_parent', '', 20, 200, ?1, ?1)",
            [now],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO notes (id, deck_id, note_type_id, fields_json, created_at, updated_at)
             VALUES ('n1', 'dk_child', 'nt_basic', '{}', ?1, ?1)",
            [now],
        )
        .unwrap();

        let decks = list_export_decks(&conn).unwrap();
        let parent = decks.iter().find(|d| d.id == "dk_parent").unwrap();
        let child = decks.iter().find(|d| d.id == "dk_child").unwrap();
        assert_eq!(child.note_count, 1);
        assert_eq!(parent.note_count, 1);
    }

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
        export_content_json_file(&conn, path.to_str().unwrap(), None).unwrap();

        conn.execute("DELETE FROM card_progress", []).unwrap();
        conn.execute("DELETE FROM cards", []).unwrap();
        conn.execute("DELETE FROM notes", []).unwrap();
        conn.execute("DELETE FROM decks", []).unwrap();

        let summary = import_content_file(&conn, path.to_str().unwrap(), None).unwrap();
        assert_eq!(summary.decks_added, 1);
        assert_eq!(summary.notes_added, 1);
        assert_eq!(summary.cards_added, 1);

        let progress: i64 = conn
            .query_row("SELECT COUNT(*) FROM card_progress WHERE card_id = 'c1'", [], |r| r.get(0))
            .unwrap();
        assert!(progress > 0);
    }
}

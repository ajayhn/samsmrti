use crate::commands::karma::{self, KarmaEarnEvent};
use crate::commands::window_profiles::WindowProfiles;
use crate::commands::search::upsert_note_fts_conn;
use crate::db::card_progress;
use crate::db::deck_tree::deck_scope_ids;
use crate::db::{sync_country_note, Database};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use tauri::{State, WebviewWindow};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NoteType {
    pub id: String,
    pub name: String,
    pub css: String,
    pub is_cloze: bool,
    pub created_at: i64,
    pub fields: Vec<Field>,
    pub templates: Vec<CardTemplate>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Field {
    pub id: String,
    pub note_type_id: String,
    pub name: String,
    pub ordinal: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CardTemplate {
    pub id: String,
    pub note_type_id: String,
    pub name: String,
    pub front_html: String,
    pub back_html: String,
    pub ordinal: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Note {
    pub id: String,
    pub deck_id: String,
    pub note_type_id: String,
    pub fields_json: serde_json::Value,
    pub created_at: i64,
    pub updated_at: i64,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Card {
    pub id: String,
    pub note_id: String,
    pub template_ordinal: i64,
    pub state: String,
    pub difficulty: f64,
    pub stability: f64,
    pub due_at: i64,
    pub last_review_at: Option<i64>,
    pub reps: i64,
    pub lapses: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateNoteInput {
    pub deck_id: String,
    pub note_type_id: String,
    pub fields: serde_json::Value,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNoteInput {
    pub id: String,
    pub deck_id: Option<String>,
    pub fields: Option<serde_json::Value>,
    pub tags: Option<Vec<String>>,
}

#[tauri::command]
pub fn get_note_types(db: State<Database>) -> Result<Vec<NoteType>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut nt_stmt = conn
        .prepare("SELECT id, name, css, is_cloze, created_at FROM note_types ORDER BY name")
        .map_err(|e| e.to_string())?;

    let note_types: Vec<(String, String, String, bool, i64)> = nt_stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get::<_, i64>(3)? != 0,
                row.get(4)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for (id, name, css, is_cloze, created_at) in note_types {
        let mut f_stmt = conn
            .prepare(
                "SELECT id, note_type_id, name, ordinal FROM fields WHERE note_type_id = ?1 ORDER BY ordinal",
            )
            .map_err(|e| e.to_string())?;
        let fields: Vec<Field> = f_stmt
            .query_map([&id], |row| {
                Ok(Field {
                    id: row.get(0)?,
                    note_type_id: row.get(1)?,
                    name: row.get(2)?,
                    ordinal: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let mut t_stmt = conn
            .prepare(
                "SELECT id, note_type_id, name, front_html, back_html, ordinal FROM card_templates WHERE note_type_id = ?1 ORDER BY ordinal",
            )
            .map_err(|e| e.to_string())?;
        let templates: Vec<CardTemplate> = t_stmt
            .query_map([&id], |row| {
                Ok(CardTemplate {
                    id: row.get(0)?,
                    note_type_id: row.get(1)?,
                    name: row.get(2)?,
                    front_html: row.get(3)?,
                    back_html: row.get(4)?,
                    ordinal: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        result.push(NoteType {
            id,
            name,
            css,
            is_cloze,
            created_at,
            fields,
            templates,
        });
    }

    Ok(result)
}

fn find_each_field(front_html: &str, back_html: &str) -> Option<String> {
    let re_pattern = "{{each:";
    for html in [front_html, back_html] {
        if let Some(start) = html.find(re_pattern) {
            let after = &html[start + re_pattern.len()..];
            if let Some(end) = after.find("}}") {
                let field_name = after[..end].trim().to_string();
                if !field_name.is_empty() {
                    return Some(field_name);
                }
            }
        }
    }
    None
}

fn count_csv_items(value: &str) -> usize {
    value
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .count()
        .max(1)
}

fn count_cloze_deletions(text: &str) -> usize {
    let mut max_n = 0usize;
    let mut i = 0;
    let bytes = text.as_bytes();
    while i + 3 < bytes.len() {
        if bytes[i] == b'{' && bytes[i + 1] == b'{' && bytes[i + 2] == b'c' {
            let start = i + 3;
            let mut end = start;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
            if end > start && end + 1 < bytes.len() && bytes[end] == b':' && bytes[end + 1] == b':' {
                if let Ok(n) = text[start..end].parse::<usize>() {
                    if n > max_n {
                        max_n = n;
                    }
                }
            }
        }
        i += 1;
    }
    max_n
}

#[derive(Debug, Serialize)]
pub struct CreateNoteResult {
    pub note: Note,
    pub karma: KarmaEarnEvent,
}

#[tauri::command]
pub fn create_note(
    db: State<Database>,
    window: WebviewWindow,
    profiles: State<'_, WindowProfiles>,
    input: CreateNoteInput,
) -> Result<CreateNoteResult, String> {
    let active = profiles.for_window(&window)?;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let note_id = format!("n_{}", uuid::Uuid::new_v4().simple());
    let now = chrono::Utc::now().timestamp();
    let fields_str = serde_json::to_string(&input.fields).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO notes (id, deck_id, note_type_id, fields_json, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (&note_id, &input.deck_id, &input.note_type_id, &fields_str, now, now),
    )
    .map_err(|e| e.to_string())?;

    let is_cloze: bool = conn
        .query_row(
            "SELECT is_cloze FROM note_types WHERE id = ?1",
            [&input.note_type_id],
            |row| Ok(row.get::<_, i64>(0)? != 0),
        )
        .map_err(|e| e.to_string())?;

    if is_cloze {
        let text_content = input
            .fields
            .as_object()
            .and_then(|o| o.values().next())
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let n = count_cloze_deletions(text_content).max(1);
        for i in 0..n {
            let card_id = format!("c_{}", uuid::Uuid::new_v4().simple());
            conn.execute(
                "INSERT INTO cards (id, note_id, template_ordinal, state, due_at) VALUES (?1, ?2, ?3, 'new', ?4)",
                (&card_id, &note_id, i as i64, now),
            )
            .map_err(|e| e.to_string())?;
            card_progress::seed_progress_for_all_profiles(&conn, &card_id, now)
                .map_err(|e| e.to_string())?;
        }
    } else {
        let mut t_stmt = conn
            .prepare("SELECT front_html, back_html, ordinal FROM card_templates WHERE note_type_id = ?1 ORDER BY ordinal")
            .map_err(|e| e.to_string())?;
        let templates: Vec<(String, String, i64)> = t_stmt
            .query_map([&input.note_type_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let fields_obj = input.fields.as_object();
        for (front, back, ordinal) in &templates {
            if let Some(each_field) = find_each_field(front, back) {
                let field_value = fields_obj
                    .and_then(|o| o.get(&each_field))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let item_count = count_csv_items(field_value);
                for item_idx in 0..item_count {
                    let card_id = format!("c_{}", uuid::Uuid::new_v4().simple());
                    let encoded_ordinal = ordinal * 1000 + item_idx as i64;
                    conn.execute(
                        "INSERT INTO cards (id, note_id, template_ordinal, state, due_at) VALUES (?1, ?2, ?3, 'new', ?4)",
                        (&card_id, &note_id, encoded_ordinal, now),
                    )
                    .map_err(|e| e.to_string())?;
                    card_progress::seed_progress_for_all_profiles(&conn, &card_id, now)
                        .map_err(|e| e.to_string())?;
                }
            } else {
                let card_id = format!("c_{}", uuid::Uuid::new_v4().simple());
                conn.execute(
                    "INSERT INTO cards (id, note_id, template_ordinal, state, due_at) VALUES (?1, ?2, ?3, 'new', ?4)",
                    (&card_id, &note_id, ordinal, now),
                )
                .map_err(|e| e.to_string())?;
                card_progress::seed_progress_for_all_profiles(&conn, &card_id, now)
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    let tags = input.tags.unwrap_or_default();
    for tag_name in &tags {
        let tag_id = format!("t_{}", uuid::Uuid::new_v4().simple());
        conn.execute(
            "INSERT OR IGNORE INTO tags (id, name) VALUES (?1, ?2)",
            (&tag_id, tag_name),
        )
        .map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR IGNORE INTO note_tags (note_id, tag_id) SELECT ?1, id FROM tags WHERE name = ?2",
            (&note_id, tag_name),
        )
        .map_err(|e| e.to_string())?;
    }

    if input.note_type_id == "nt_country" {
        sync_country_note(&conn, &note_id).map_err(|e| e.to_string())?;
    }

    upsert_note_fts_conn(&conn, &note_id, &fields_str)?;

    let card_count = karma::count_cards_for_note(&conn, &note_id)?;
    let karma = karma::earn_add_conn(&conn, &active, card_count)?;

    Ok(CreateNoteResult {
        note: Note {
            id: note_id,
            deck_id: input.deck_id,
            note_type_id: input.note_type_id,
            fields_json: input.fields,
            created_at: now,
            updated_at: now,
            tags,
        },
        karma,
    })
}

/// Most-used note type in this deck (includes subdecks). `None` if the deck has no notes.
#[tauri::command]
pub fn get_deck_primary_note_type(
    db: State<Database>,
    deck_id: String,
) -> Result<Option<String>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let scope = deck_scope_ids(&conn, &deck_id).map_err(|e| e.to_string())?;
    if scope.is_empty() {
        return Ok(None);
    }

    let placeholders = scope.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let sql = format!(
        "SELECT note_type_id, COUNT(*) AS cnt FROM notes
         WHERE deck_id IN ({placeholders})
         GROUP BY note_type_id
         ORDER BY cnt DESC
         LIMIT 1"
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        scope.iter().map(|id| id as &dyn rusqlite::types::ToSql).collect();

    let result = stmt
        .query_row(param_refs.as_slice(), |row| row.get::<_, String>(0))
        .optional()
        .map_err(|e| e.to_string())?;

    Ok(result)
}

#[tauri::command]
pub fn get_notes(db: State<Database>, deck_id: String) -> Result<Vec<Note>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let scope = deck_scope_ids(&conn, &deck_id).map_err(|e| e.to_string())?;
    if scope.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders = scope.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let sql = format!(
        "SELECT id, deck_id, note_type_id, fields_json, created_at, updated_at FROM notes WHERE deck_id IN ({placeholders}) ORDER BY created_at DESC"
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        scope.iter().map(|id| id as &dyn rusqlite::types::ToSql).collect();

    let notes = stmt
        .query_map(param_refs.as_slice(), |row| {
            let fields_str: String = row.get(3)?;
            Ok(Note {
                id: row.get(0)?,
                deck_id: row.get(1)?,
                note_type_id: row.get(2)?,
                fields_json: serde_json::from_str(&fields_str).unwrap_or(serde_json::Value::Object(
                    serde_json::Map::new(),
                )),
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                tags: Vec::new(),
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for mut note in notes {
        let mut tag_stmt = conn
            .prepare("SELECT t.name FROM tags t JOIN note_tags nt ON nt.tag_id = t.id WHERE nt.note_id = ?1")
            .map_err(|e| e.to_string())?;
        let tags: Vec<String> = tag_stmt
            .query_map([&note.id], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        note.tags = tags;
        result.push(note);
    }

    Ok(result)
}

#[tauri::command]
pub fn update_note(db: State<Database>, input: UpdateNoteInput) -> Result<Note, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp();

    let current = conn
        .query_row(
            "SELECT deck_id, note_type_id, fields_json, created_at FROM notes WHERE id = ?1",
            [&input.id],
            |row| {
                let fields_str: String = row.get(2)?;
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    fields_str,
                    row.get::<_, i64>(3)?,
                ))
            },
        )
        .map_err(|e| e.to_string())?;

    let deck_id = input.deck_id.unwrap_or(current.0);
    let fields = input.fields.unwrap_or_else(|| {
        serde_json::from_str(&current.2).unwrap_or(serde_json::Value::Object(serde_json::Map::new()))
    });
    let fields_str = serde_json::to_string(&fields).map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE notes SET deck_id=?1, fields_json=?2, updated_at=?3 WHERE id=?4",
        (&deck_id, &fields_str, now, &input.id),
    )
    .map_err(|e| e.to_string())?;

    if let Some(tags) = &input.tags {
        conn.execute("DELETE FROM note_tags WHERE note_id = ?1", [&input.id])
            .map_err(|e| e.to_string())?;
        for tag_name in tags {
            let tag_id = format!("t_{}", uuid::Uuid::new_v4().simple());
            conn.execute(
                "INSERT OR IGNORE INTO tags (id, name) VALUES (?1, ?2)",
                (&tag_id, tag_name),
            )
            .map_err(|e| e.to_string())?;
            conn.execute(
                "INSERT OR IGNORE INTO note_tags (note_id, tag_id) SELECT ?1, id FROM tags WHERE name = ?2",
                (&input.id, tag_name),
            )
            .map_err(|e| e.to_string())?;
        }
    }

    let mut tag_stmt = conn
        .prepare("SELECT t.name FROM tags t JOIN note_tags nt ON nt.tag_id = t.id WHERE nt.note_id = ?1")
        .map_err(|e| e.to_string())?;
    let tags: Vec<String> = tag_stmt
        .query_map([&input.id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    if current.1 == "nt_country" {
        sync_country_note(&conn, &input.id).map_err(|e| e.to_string())?;
    }

    upsert_note_fts_conn(&conn, &input.id, &fields_str)?;

    Ok(Note {
        id: input.id,
        deck_id,
        note_type_id: current.1,
        fields_json: fields,
        created_at: current.3,
        updated_at: now,
        tags,
    })
}

#[tauri::command]
pub fn get_note_tags(db: State<Database>, note_id: String) -> Result<Vec<String>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT t.name FROM tags t
             JOIN note_tags nt ON nt.tag_id = t.id
             WHERE nt.note_id = ?1
             ORDER BY t.name",
        )
        .map_err(|e| e.to_string())?;
    let tags: Vec<String> = stmt
        .query_map([&note_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(tags)
}

#[tauri::command]
pub fn get_card_flag(db: State<Database>, card_id: String) -> Result<bool, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let flagged: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM card_flags WHERE card_id = ?1)",
            [&card_id],
            |row| Ok(row.get::<_, i64>(0)? != 0),
        )
        .map_err(|e| e.to_string())?;
    Ok(flagged)
}

#[tauri::command]
pub fn set_card_flag(db: State<Database>, card_id: String, flagged: bool) -> Result<bool, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    if flagged {
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO card_flags (card_id, flagged_at) VALUES (?1, ?2)
             ON CONFLICT(card_id) DO UPDATE SET flagged_at = excluded.flagged_at",
            (&card_id, now),
        )
        .map_err(|e| e.to_string())?;
    } else {
        conn.execute("DELETE FROM card_flags WHERE card_id = ?1", [&card_id])
            .map_err(|e| e.to_string())?;
    }
    Ok(flagged)
}

#[tauri::command]
pub fn delete_note(db: State<Database>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM notes WHERE id = ?1", [&id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_cards_for_note(
    db: State<Database>,
    window: WebviewWindow,
    profiles: State<'_, WindowProfiles>,
    note_id: String,
) -> Result<Vec<Card>, String> {
    let active = profiles.for_window(&window)?;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.note_id, c.template_ordinal, cp.state, cp.difficulty, cp.stability, cp.due_at, cp.last_review_at, cp.reps, cp.lapses
             FROM cards c
             JOIN card_progress cp ON cp.card_id = c.id AND cp.profile_id = ?2
             WHERE c.note_id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let cards = stmt
        .query_map((&note_id, &active.id), |row| {
            Ok(Card {
                id: row.get(0)?,
                note_id: row.get(1)?,
                template_ordinal: row.get(2)?,
                state: row.get(3)?,
                difficulty: row.get(4)?,
                stability: row.get(5)?,
                due_at: row.get(6)?,
                last_review_at: row.get(7)?,
                reps: row.get(8)?,
                lapses: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(cards)
}

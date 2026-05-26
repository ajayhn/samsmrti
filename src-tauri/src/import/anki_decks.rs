use rusqlite::Connection;
use std::collections::HashMap;

const ANKI_DECK_SEP: char = '\u{001f}';

/// Split Anki deck path (`Parent::Child` or `Parent\x1fChild`).
pub fn split_anki_deck_path(name: &str) -> Vec<String> {
    if name.contains(ANKI_DECK_SEP) {
        return name
            .split(ANKI_DECK_SEP)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }
    if name.contains("::") {
        return name
            .split("::")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }
    vec![name.trim().to_string()]
}

/// Ensure deck hierarchy exists; returns leaf deck id for this path.
pub fn ensure_deck_path(
    conn: &Connection,
    segments: &[String],
    now: i64,
    path_cache: &mut HashMap<String, String>,
) -> Result<String, String> {
    let mut parent_id: Option<String> = None;
    let mut path_key = String::new();
    let mut last_id = String::new();

    for segment in segments {
        if !path_key.is_empty() {
            path_key.push('/');
        }
        path_key.push_str(segment);

        if let Some(id) = path_cache.get(&path_key) {
            last_id = id.clone();
            parent_id = Some(last_id.clone());
            continue;
        }

        let existing: Option<String> = conn
            .query_row(
                "SELECT id FROM decks WHERE name = ?1 AND (
                    (?2 IS NULL AND parent_id IS NULL) OR parent_id = ?2
                 )",
                (segment, &parent_id),
                |row| row.get(0),
            )
            .ok();

        let deck_id = if let Some(id) = existing {
            id
        } else {
            let id = format!("dk_{}", uuid::Uuid::new_v4().simple());
            conn.execute(
                "INSERT INTO decks (id, name, parent_id, description, created_at, updated_at) VALUES (?1, ?2, ?3, '', ?4, ?5)",
                (&id, segment, &parent_id, now, now),
            )
            .map_err(|e| e.to_string())?;
            id
        };

        path_cache.insert(path_key.clone(), deck_id.clone());
        parent_id = Some(deck_id.clone());
        last_id = deck_id;
    }

    Ok(last_id)
}

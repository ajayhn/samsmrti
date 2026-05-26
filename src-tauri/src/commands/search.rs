use crate::commands::profiles::ActiveProfile;
use crate::db::deck_tree::deck_scope_ids;
use crate::db::Database;
use rusqlite::Connection;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub note_id: String,
    pub deck_id: String,
    pub deck_name: String,
    pub note_type_id: String,
    pub note_type_name: String,
    pub fields_json: serde_json::Value,
    pub tags: Vec<String>,
    pub card_count: i64,
    pub created_at: i64,
}

fn extract_text_from_fields(fields_json: &str) -> String {
    let obj: serde_json::Value = serde_json::from_str(fields_json).unwrap_or_default();
    if let Some(map) = obj.as_object() {
        map.values()
            .filter_map(|v| v.as_str())
            .map(|s| strip_html(s))
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        String::new()
    }
}

fn strip_html(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
            result.push(' ');
        } else if !in_tag {
            result.push(c);
        }
    }
    result
}

/// Rebuild the full-text index for all notes (slow on large collections).
pub fn rebuild_search_index_conn(conn: &Connection) -> Result<(), String> {
    let mut stmt = conn
        .prepare("SELECT id, fields_json FROM notes")
        .map_err(|e| e.to_string())?;

    let rows: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM notes_fts", [])
        .map_err(|e| e.to_string())?;
    for (note_id, fields_json) in &rows {
        let text = extract_text_from_fields(fields_json);
        tx.execute(
            "INSERT INTO notes_fts (note_id, content) VALUES (?1, ?2)",
            (note_id, &text),
        )
        .map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

/// Build the index only when notes exist but FTS rows are missing (e.g. after import).
pub fn ensure_search_index_conn(conn: &Connection) -> Result<bool, String> {
    let fts_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM notes_fts", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    let notes_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    if notes_count == 0 || fts_count >= notes_count {
        return Ok(false);
    }
    rebuild_search_index_conn(conn)?;
    Ok(true)
}

pub fn upsert_note_fts_conn(conn: &Connection, note_id: &str, fields_json: &str) -> Result<(), String> {
    conn.execute("DELETE FROM notes_fts WHERE note_id = ?1", [note_id])
        .map_err(|e| e.to_string())?;
    let text = extract_text_from_fields(fields_json);
    conn.execute(
        "INSERT INTO notes_fts (note_id, content) VALUES (?1, ?2)",
        (note_id, &text),
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn attach_tags_to_results(
    conn: &Connection,
    results: &mut [SearchResult],
) -> Result<(), String> {
    if results.is_empty() {
        return Ok(());
    }
    let placeholders = results
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT nt.note_id, t.name FROM note_tags nt
         JOIN tags t ON t.id = nt.tag_id
         WHERE nt.note_id IN ({placeholders})
         ORDER BY t.name"
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let note_ids: Vec<&str> = results.iter().map(|r| r.note_id.as_str()).collect();
    let mut tag_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut rows = stmt
        .query(rusqlite::params_from_iter(note_ids.iter().copied()))
        .map_err(|e| e.to_string())?;
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let note_id: String = row.get(0).map_err(|e| e.to_string())?;
        let tag: String = row.get(1).map_err(|e| e.to_string())?;
        tag_map.entry(note_id).or_default().push(tag);
    }
    for r in results {
        r.tags = tag_map.remove(&r.note_id).unwrap_or_default();
    }
    Ok(())
}

#[tauri::command]
pub fn rebuild_search_index(db: State<Database>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    rebuild_search_index_conn(&conn)
}

#[tauri::command]
pub fn ensure_search_index(db: State<Database>) -> Result<bool, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    ensure_search_index_conn(&conn)
}

#[tauri::command]
pub fn search_notes(
    db: State<Database>,
    query: String,
    deck_id: Option<String>,
    tag: Option<String>,
    note_type_id: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<SearchResult>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let lim = limit.unwrap_or(50);
    let off = offset.unwrap_or(0);

    let trimmed = query.trim();

    if trimmed.is_empty() && deck_id.is_none() && tag.is_none() && note_type_id.is_none() {
        return Ok(vec![]);
    }

    let has_fts = !trimmed.is_empty();

    let mut sql = String::from(
        "SELECT n.id, n.deck_id, d.name, n.note_type_id, nt.name, n.fields_json, n.created_at,
                (SELECT COUNT(*) FROM cards c WHERE c.note_id = n.id) as card_count
         FROM notes n
         JOIN decks d ON d.id = n.deck_id
         JOIN note_types nt ON nt.id = n.note_type_id ",
    );

    if has_fts {
        sql.push_str("JOIN notes_fts fts ON fts.note_id = n.id ");
    }

    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let mut param_idx = 1;

    if has_fts {
        conditions.push(format!("notes_fts MATCH ?{}", param_idx));
        let fts_query = format!("\"{}\"", trimmed.replace('"', "\"\""));
        params.push(Box::new(fts_query));
        param_idx += 1;
    }

    if let Some(ref did) = deck_id {
        let scope = deck_scope_ids(&conn, did).map_err(|e| e.to_string())?;
        if !scope.is_empty() {
            let ph: Vec<String> = (0..scope.len())
                .map(|i| format!("?{}", param_idx + i))
                .collect();
            conditions.push(format!("n.deck_id IN ({})", ph.join(", ")));
            for id in scope {
                params.push(Box::new(id));
                param_idx += 1;
            }
        }
    }

    if let Some(ref tag_name) = tag {
        conditions.push(format!(
            "EXISTS (SELECT 1 FROM note_tags nt2 JOIN tags t ON t.id = nt2.tag_id WHERE nt2.note_id = n.id AND t.name = ?{})",
            param_idx
        ));
        params.push(Box::new(tag_name.clone()));
        param_idx += 1;
    }

    if let Some(ref nt_id) = note_type_id {
        conditions.push(format!("n.note_type_id = ?{}", param_idx));
        params.push(Box::new(nt_id.clone()));
        param_idx += 1;
    }

    if !conditions.is_empty() {
        sql.push_str("WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(&format!(" ORDER BY n.created_at DESC LIMIT ?{} OFFSET ?{}", param_idx, param_idx + 1));
    params.push(Box::new(lim));
    params.push(Box::new(off));

    let params_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

    let results = stmt
        .query_map(params_refs.as_slice(), |row| {
            let fields_str: String = row.get(5)?;
            Ok(SearchResult {
                note_id: row.get(0)?,
                deck_id: row.get(1)?,
                deck_name: row.get(2)?,
                note_type_id: row.get(3)?,
                note_type_name: row.get(4)?,
                fields_json: serde_json::from_str(&fields_str)
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
                tags: vec![],
                card_count: row.get(7)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut results_with_tags = results;
    attach_tags_to_results(&conn, &mut results_with_tags)?;
    Ok(results_with_tags)
}

#[tauri::command]
pub fn get_all_tags(db: State<Database>) -> Result<Vec<(String, String, i64)>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.name, COUNT(nt.note_id) as usage_count
             FROM tags t
             LEFT JOIN note_tags nt ON nt.tag_id = t.id
             GROUP BY t.id
             ORDER BY usage_count DESC",
        )
        .map_err(|e| e.to_string())?;

    let tags = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(tags)
}

#[derive(Debug, Serialize)]
pub struct StatsOverview {
    pub total_cards: i64,
    pub new_cards: i64,
    pub learning_cards: i64,
    pub review_cards: i64,
    pub total_decks: i64,
    pub total_reviews_today: i64,
    pub streak_days: i64,
    pub daily_reviews: Vec<DailyReview>,
}

#[derive(Debug, Serialize)]
pub struct DailyReview {
    pub date: String,
    pub count: i64,
    pub again: i64,
    pub hard: i64,
    pub good: i64,
    pub easy: i64,
}

#[tauri::command]
pub fn get_stats_overview(
    db: State<Database>,
    active: State<'_, Mutex<ActiveProfile>>,
) -> Result<StatsOverview, String> {
    let active_guard = active.lock().map_err(|e| e.to_string())?;
    let profile_id = active_guard.id.clone();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp();
    let today_start = now - (now % 86400);

    let total_cards: i64 = conn
        .query_row("SELECT COUNT(*) FROM cards", [], |row| row.get(0))
        .unwrap_or(0);

    let new_cards: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM card_progress WHERE profile_id = ?1 AND state = 'new'",
            [&profile_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let learning_cards: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM card_progress WHERE profile_id = ?1 AND state IN ('learning', 'relearning')",
            [&profile_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let review_cards: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM card_progress WHERE profile_id = ?1 AND state = 'review'",
            [&profile_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let total_decks: i64 = conn
        .query_row("SELECT COUNT(*) FROM decks", [], |row| row.get(0))
        .unwrap_or(0);

    let total_reviews_today: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM review_log WHERE profile_id = ?1 AND reviewed_at >= ?2",
            (&profile_id, today_start),
            |row| row.get(0),
        )
        .unwrap_or(0);

    let mut daily_stmt = conn
        .prepare(
            "SELECT date(reviewed_at, 'unixepoch') as day, COUNT(*) as cnt,
                    SUM(CASE WHEN rating = 1 THEN 1 ELSE 0 END),
                    SUM(CASE WHEN rating = 2 THEN 1 ELSE 0 END),
                    SUM(CASE WHEN rating = 3 THEN 1 ELSE 0 END),
                    SUM(CASE WHEN rating = 4 THEN 1 ELSE 0 END)
             FROM review_log
             WHERE profile_id = ?1 AND reviewed_at >= ?2
             GROUP BY day
             ORDER BY day DESC",
        )
        .map_err(|e| e.to_string())?;

    let thirty_days_ago = now - (30 * 86400);
    let daily_reviews: Vec<DailyReview> = daily_stmt
        .query_map((&profile_id, thirty_days_ago), |row| {
            Ok(DailyReview {
                date: row.get(0)?,
                count: row.get(1)?,
                again: row.get(2)?,
                hard: row.get(3)?,
                good: row.get(4)?,
                easy: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let streak_days = calculate_streak(&daily_reviews);

    Ok(StatsOverview {
        total_cards,
        new_cards,
        learning_cards,
        review_cards,
        total_decks,
        total_reviews_today,
        streak_days,
        daily_reviews,
    })
}

fn calculate_streak(daily: &[DailyReview]) -> i64 {
    if daily.is_empty() {
        return 0;
    }
    let mut streak = 0i64;
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let mut expected_date = today;

    for day in daily {
        if day.date == expected_date {
            streak += 1;
            let parsed = chrono::NaiveDate::parse_from_str(&day.date, "%Y-%m-%d");
            if let Ok(d) = parsed {
                expected_date = (d - chrono::Duration::days(1)).format("%Y-%m-%d").to_string();
            } else {
                break;
            }
        } else {
            break;
        }
    }

    streak
}


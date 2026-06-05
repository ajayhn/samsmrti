use crate::commands::karma::{self, KarmaEarnEvent};
use crate::commands::window_profiles::WindowProfiles;
use crate::db::deck_tree::deck_scope_ids;
use crate::db::Database;
use serde::{Deserialize, Serialize};
use tauri::{State, WebviewWindow};

#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewCard {
    pub card_id: String,
    pub note_id: String,
    pub note_type_id: String,
    pub is_cloze: bool,
    pub template_ordinal: i64,
    pub front_html: String,
    pub back_html: String,
    pub fields: serde_json::Value,
    pub state: String,
    pub css: String,
}

#[derive(Debug, Deserialize)]
pub struct AnswerInput {
    pub card_id: String,
    pub rating: i32,
    pub elapsed_ms: i64,
}

#[derive(Debug, Serialize)]
pub struct AnswerResult {
    pub card_id: String,
    pub new_state: String,
    pub due_at: i64,
    pub stability: f64,
    pub difficulty: f64,
    pub review_log_id: String,
    pub karma: KarmaEarnEvent,
}

#[derive(Debug, Serialize)]
pub struct ReviewStats {
    pub reviewed_today: i64,
    pub again_count: i64,
    pub hard_count: i64,
    pub good_count: i64,
    pub easy_count: i64,
    pub total_time_ms: i64,
}

const INITIAL_STABILITY: [f64; 4] = [0.4, 0.6, 2.4, 5.8];
const INITIAL_DIFFICULTY: [f64; 4] = [7.0, 6.0, 5.0, 3.0];

fn clamp(val: f64, min: f64, max: f64) -> f64 {
    val.max(min).min(max)
}

fn next_difficulty(d: f64, rating: i32) -> f64 {
    let delta = match rating {
        1 => 0.8,
        2 => 0.2,
        3 => -0.2,
        4 => -0.6,
        _ => 0.0,
    };
    clamp(d + delta, 1.0, 10.0)
}

fn next_stability(s: f64, d: f64, rating: i32) -> f64 {
    if rating == 1 {
        return clamp(s * 0.2, 0.1, 1.0);
    }
    let multiplier = match rating {
        2 => 1.2,
        3 => 2.5,
        4 => 3.5,
        _ => 1.0,
    };
    let difficulty_factor = 1.0 + (10.0 - d) * 0.05;
    s * multiplier * difficulty_factor
}

fn interval_preview(state: &str, difficulty: f64, stability: f64, rating: i32) -> f64 {
    if state == "new" {
        let idx = (rating - 1).max(0).min(3) as usize;
        let s = INITIAL_STABILITY[idx];
        if rating == 1 { 0.007 } else { s }
    } else {
        let new_d = next_difficulty(difficulty, rating);
        let new_s = next_stability(stability, new_d, rating);
        if rating == 1 { 0.007 } else { new_s }
    }
}

fn format_interval(days: f64) -> String {
    if days < 0.042 {
        format!("{}m", (days * 1440.0).round() as i64)
    } else if days < 1.0 {
        format!("{}h", (days * 24.0).round() as i64)
    } else if days < 30.0 {
        format!("{}d", days.round() as i64)
    } else if days < 365.0 {
        format!("{:.1}mo", days / 30.0)
    } else {
        format!("{:.1}y", days / 365.0)
    }
}

#[derive(Debug, Serialize)]
pub struct IntervalPreview {
    pub again: String,
    pub hard: String,
    pub good: String,
    pub easy: String,
}

#[tauri::command]
pub fn get_interval_preview(
    db: State<Database>,
    window: WebviewWindow,
    profiles: State<'_, WindowProfiles>,
    card_id: String,
) -> Result<IntervalPreview, String> {
    let active = profiles.for_window(&window)?;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let (state, difficulty, stability): (String, f64, f64) = conn
        .query_row(
            "SELECT state, difficulty, stability FROM card_progress WHERE profile_id = ?1 AND card_id = ?2",
            (&active.id, &card_id),
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| e.to_string())?;

    Ok(IntervalPreview {
        again: format_interval(interval_preview(&state, difficulty, stability, 1)),
        hard: format_interval(interval_preview(&state, difficulty, stability, 2)),
        good: format_interval(interval_preview(&state, difficulty, stability, 3)),
        easy: format_interval(interval_preview(&state, difficulty, stability, 4)),
    })
}

#[tauri::command]
pub fn get_review_queue(
    db: State<Database>,
    window: WebviewWindow,
    profiles: State<'_, WindowProfiles>,
    deck_id: String,
) -> Result<Vec<ReviewCard>, String> {
    let active = profiles.for_window(&window)?;
    let profile_id = active.id.clone();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp();

    let deck = conn
        .query_row(
            "SELECT new_per_day, max_reviews FROM decks WHERE id = ?1",
            [&deck_id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        )
        .map_err(|e| e.to_string())?;

    let (new_limit, review_limit) = deck;
    let scope = deck_scope_ids(&conn, &deck_id).map_err(|e| e.to_string())?;
    if scope.is_empty() {
        return Ok(Vec::new());
    }

    let deck_placeholders = scope
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");
    let total_limit = new_limit + review_limit;

    let sql = format!(
        "SELECT c.id, c.note_id, n.note_type_id, nt.is_cloze, c.template_ordinal,
                ct.front_html, ct.back_html, n.fields_json, cp.state, nt.css
         FROM cards c
         JOIN card_progress cp ON cp.card_id = c.id AND cp.profile_id = ?
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
         WHERE n.deck_id IN ({deck_placeholders})
           AND (cp.buried_until IS NULL OR cp.buried_until <= ?)
           AND ((cp.state = 'new') OR (cp.state IN ('learning','review','relearning') AND cp.due_at <= ?))
         ORDER BY
           CASE cp.state
             WHEN 'learning' THEN 0
             WHEN 'relearning' THEN 1
             WHEN 'review' THEN 2
             WHEN 'new' THEN 3
           END,
           CASE WHEN cp.state = 'new' THEN random() ELSE cp.due_at END ASC
         LIMIT ?"
    );

    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(profile_id)];
    params.extend(
        scope
            .iter()
            .map(|id| Box::new(id.clone()) as Box<dyn rusqlite::types::ToSql>),
    );
    params.push(Box::new(now));
    params.push(Box::new(now));
    params.push(Box::new(total_limit));

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
pub fn answer_card(
    db: State<Database>,
    window: WebviewWindow,
    profiles: State<'_, WindowProfiles>,
    input: AnswerInput,
) -> Result<AnswerResult, String> {
    let active = profiles.for_window(&window)?;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp();

    let profile_id = active.id.clone();

    let (state, difficulty, stability): (String, f64, f64) = conn
        .query_row(
            "SELECT state, difficulty, stability FROM card_progress WHERE profile_id = ?1 AND card_id = ?2",
            (&profile_id, &input.card_id),
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| e.to_string())?;

    let rating_idx = (input.rating - 1).max(0).min(3) as usize;

    let (new_state, new_difficulty, new_stability, interval_days) = if state == "new" {
        let s = INITIAL_STABILITY[rating_idx];
        let d = INITIAL_DIFFICULTY[rating_idx];
        let new_state = if input.rating == 1 { "learning" } else { "review" };
        let interval = if input.rating == 1 { 0.007 } else { s };
        (new_state.to_string(), d, s, interval)
    } else {
        let new_d = next_difficulty(difficulty, input.rating);
        let new_s = next_stability(stability, new_d, input.rating);
        let new_state = if input.rating == 1 { "relearning" } else { "review" };
        let interval = if input.rating == 1 { 0.007 } else { new_s };
        (new_state.to_string(), new_d, new_s, interval)
    };

    let due_at = now + (interval_days * 86400.0) as i64;

    conn.execute(
        "UPDATE card_progress SET state=?1, difficulty=?2, stability=?3, due_at=?4, last_review_at=?5, reps=reps+1, lapses=lapses+?6 WHERE profile_id=?7 AND card_id=?8",
        (
            &new_state,
            new_difficulty,
            new_stability,
            due_at,
            now,
            if input.rating == 1 { 1 } else { 0 },
            &profile_id,
            &input.card_id,
        ),
    )
    .map_err(|e| e.to_string())?;

    let log_id = format!("rl_{}", uuid::Uuid::new_v4().simple());
    conn.execute(
        "INSERT INTO review_log (id, card_id, profile_id, reviewed_at, rating, elapsed_ms, scheduled_days, state_before, state_after) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        (
            &log_id,
            &input.card_id,
            &profile_id,
            now,
            input.rating,
            input.elapsed_ms,
            interval_days,
            &state,
            &new_state,
        ),
    )
    .map_err(|e| e.to_string())?;

    let karma = karma::earn_review_conn(&conn, &active, input.elapsed_ms)?;

    Ok(AnswerResult {
        card_id: input.card_id,
        new_state,
        due_at,
        stability: new_stability,
        difficulty: new_difficulty,
        review_log_id: log_id,
        karma,
    })
}

#[derive(Debug, Serialize)]
pub struct UndoReviewResult {
    pub karma: KarmaEarnEvent,
}

#[tauri::command]
pub fn undo_review(
    db: State<Database>,
    window: WebviewWindow,
    profiles: State<'_, WindowProfiles>,
    review_log_id: String,
) -> Result<UndoReviewResult, String> {
    let active = profiles.for_window(&window)?;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let profile_id = active.id.clone();

    let (card_id, rating, state_before): (String, i32, String) = conn
        .query_row(
            "SELECT card_id, rating, state_before FROM review_log WHERE id = ?1 AND profile_id = ?2",
            (&review_log_id, &profile_id),
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| e.to_string())?;

    let prev_log: Option<(f64, f64)> = conn
        .query_row(
            "SELECT scheduled_days, scheduled_days FROM review_log WHERE card_id = ?1 AND profile_id = ?2 AND id != ?3 ORDER BY reviewed_at DESC LIMIT 1",
            (&card_id, &profile_id, &review_log_id),
            |row| Ok((row.get::<_, f64>(0)?, row.get::<_, f64>(1)?)),
        )
        .ok();

    let (prev_diff, prev_stab) = prev_log.unwrap_or((0.0, 0.0));

    let lapse_delta = if rating == 1 { -1 } else { 0 };

    conn.execute(
        "UPDATE card_progress SET state=?1, difficulty=?2, stability=?3, due_at=0, reps=MAX(reps-1,0), lapses=MAX(lapses+?4,0) WHERE profile_id=?5 AND card_id=?6",
        (
            &state_before,
            prev_diff,
            prev_stab,
            lapse_delta,
            &profile_id,
            &card_id,
        ),
    )
    .map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM review_log WHERE id = ?1", [&review_log_id])
        .map_err(|e| e.to_string())?;

    let karma = karma::revert_review_conn(&conn, &active)?;

    Ok(UndoReviewResult { karma })
}

#[tauri::command]
pub fn get_review_stats(
    db: State<Database>,
    window: WebviewWindow,
    profiles: State<'_, WindowProfiles>,
    deck_id: String,
) -> Result<ReviewStats, String> {
    let active = profiles.for_window(&window)?;
    let profile_id = active.id.clone();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp();
    let today_start = now - (now % 86400);
    let scope = deck_scope_ids(&conn, &deck_id).map_err(|e| e.to_string())?;
    if scope.is_empty() {
        return Ok(ReviewStats {
            reviewed_today: 0,
            again_count: 0,
            hard_count: 0,
            good_count: 0,
            easy_count: 0,
            total_time_ms: 0,
        });
    }

    let deck_placeholders = scope.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let sql = format!(
        "SELECT rl.rating, rl.elapsed_ms
         FROM review_log rl
         JOIN cards c ON c.id = rl.card_id
         JOIN notes n ON n.id = c.note_id
         WHERE n.deck_id IN ({deck_placeholders}) AND rl.profile_id = ? AND rl.reviewed_at >= ?"
    );

    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> =
        scope.iter().map(|id| Box::new(id.clone()) as Box<dyn rusqlite::types::ToSql>).collect();
    params.push(Box::new(profile_id));
    params.push(Box::new(today_start));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows: Vec<(i32, i64)> = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut stats = ReviewStats {
        reviewed_today: rows.len() as i64,
        again_count: 0,
        hard_count: 0,
        good_count: 0,
        easy_count: 0,
        total_time_ms: 0,
    };

    for (rating, elapsed) in &rows {
        stats.total_time_ms += elapsed;
        match rating {
            1 => stats.again_count += 1,
            2 => stats.hard_count += 1,
            3 => stats.good_count += 1,
            4 => stats.easy_count += 1,
            _ => {}
        }
    }

    Ok(stats)
}

fn end_of_today(now: i64) -> i64 {
    let day_start = now - (now % 86400);
    day_start + 86400
}

#[derive(Debug, Serialize)]
pub struct BuriedCard {
    pub card_id: String,
    pub note_id: String,
    pub deck_id: String,
    pub deck_name: String,
    pub front_html: String,
    pub fields: serde_json::Value,
    pub state: String,
    pub buried_until: i64,
}

#[tauri::command]
pub fn bury_card(
    db: State<Database>,
    window: WebviewWindow,
    profiles: State<'_, WindowProfiles>,
    card_id: String,
) -> Result<i64, String> {
    let active = profiles.for_window(&window)?;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp();
    let buried_until = end_of_today(now);

    let updated = conn
        .execute(
            "UPDATE card_progress SET buried_until = ?1 WHERE profile_id = ?2 AND card_id = ?3",
            (buried_until, &active.id, &card_id),
        )
        .map_err(|e| e.to_string())?;

    if updated == 0 {
        return Err("Card not found".to_string());
    }

    Ok(buried_until)
}

#[tauri::command]
pub fn unbury_card(
    db: State<Database>,
    window: WebviewWindow,
    profiles: State<'_, WindowProfiles>,
    card_id: String,
) -> Result<(), String> {
    let active = profiles.for_window(&window)?;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE card_progress SET buried_until = NULL WHERE profile_id = ?1 AND card_id = ?2",
        (&active.id, &card_id),
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_buried_cards(
    db: State<Database>,
    window: WebviewWindow,
    profiles: State<'_, WindowProfiles>,
    query: Option<String>,
    deck_id: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<BuriedCard>, String> {
    let active = profiles.for_window(&window)?;
    let profile_id = active.id.clone();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp();
    let fetch_limit = limit.unwrap_or(100).min(500);
    let search = query.unwrap_or_default().trim().to_lowercase();

    let base_sql = "SELECT c.id, c.note_id, n.deck_id, d.name, ct.front_html, n.fields_json, cp.state, cp.buried_until
         FROM cards c
         JOIN card_progress cp ON cp.card_id = c.id AND cp.profile_id = ?2
         JOIN notes n ON n.id = c.note_id
         JOIN decks d ON d.id = n.deck_id
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
         WHERE cp.buried_until IS NOT NULL AND cp.buried_until > ?1";

    let map_row = |row: &rusqlite::Row<'_>| -> rusqlite::Result<BuriedCard> {
        let fields_str: String = row.get(5)?;
        Ok(BuriedCard {
            card_id: row.get(0)?,
            note_id: row.get(1)?,
            deck_id: row.get(2)?,
            deck_name: row.get(3)?,
            front_html: row.get(4)?,
            fields: serde_json::from_str(&fields_str)
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
            state: row.get(6)?,
            buried_until: row.get(7)?,
        })
    };

    let mut cards: Vec<BuriedCard> = if let Some(ref did) = deck_id {
        let sql = format!("{base_sql} AND n.deck_id = ?3 ORDER BY cp.buried_until ASC, d.name, c.id LIMIT ?4");
        conn.prepare(&sql)
            .map_err(|e| e.to_string())?
            .query_map(rusqlite::params![now, profile_id, did, fetch_limit], map_row)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    } else {
        let sql = format!("{base_sql} ORDER BY cp.buried_until ASC, d.name, c.id LIMIT ?3");
        conn.prepare(&sql)
            .map_err(|e| e.to_string())?
            .query_map(rusqlite::params![now, profile_id, fetch_limit], map_row)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };

    if !search.is_empty() {
        cards.retain(|c| {
            let fields_text = c
                .fields
                .as_object()
                .map(|obj| {
                    obj.values()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .unwrap_or_default()
                .to_lowercase();
            c.deck_name.to_lowercase().contains(&search)
                || c.front_html.to_lowercase().contains(&search)
                || fields_text.contains(&search)
        });
    }

    Ok(cards)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReviewLogSnapshot {
    pub id: String,
    pub reviewed_at: i64,
    pub rating: i32,
    pub elapsed_ms: i64,
    pub scheduled_days: f64,
    pub state_before: String,
    pub state_after: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeletedCardSnapshot {
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
    pub buried_until: Option<i64>,
    pub triple_ids: Vec<String>,
    pub review_logs: Vec<ReviewLogSnapshot>,
}

#[tauri::command]
pub fn delete_card(
    db: State<Database>,
    window: WebviewWindow,
    profiles: State<'_, WindowProfiles>,
    card_id: String,
) -> Result<DeletedCardSnapshot, String> {
    let active = profiles.for_window(&window)?;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut snapshot: DeletedCardSnapshot = conn
        .query_row(
            "SELECT c.id, c.note_id, c.template_ordinal, cp.state, cp.difficulty, cp.stability, cp.due_at, cp.last_review_at, cp.reps, cp.lapses, cp.buried_until
             FROM cards c
             JOIN card_progress cp ON cp.card_id = c.id AND cp.profile_id = ?2
             WHERE c.id = ?1",
            (&card_id, &active.id),
            |row| {
                Ok(DeletedCardSnapshot {
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
                    buried_until: row.get(10)?,
                    triple_ids: Vec::new(),
                    review_logs: Vec::new(),
                })
            },
        )
        .map_err(|e| e.to_string())?;

    snapshot.triple_ids = conn
        .prepare("SELECT triple_id FROM card_triples WHERE card_id = ?1")
        .map_err(|e| e.to_string())?
        .query_map([&card_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    snapshot.review_logs = conn
        .prepare(
            "SELECT id, reviewed_at, rating, elapsed_ms, scheduled_days, state_before, state_after
             FROM review_log WHERE card_id = ?1 ORDER BY reviewed_at",
        )
        .map_err(|e| e.to_string())?
        .query_map([&card_id], |row| {
            Ok(ReviewLogSnapshot {
                id: row.get(0)?,
                reviewed_at: row.get(1)?,
                rating: row.get(2)?,
                elapsed_ms: row.get(3)?,
                scheduled_days: row.get(4)?,
                state_before: row.get(5)?,
                state_after: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM cards WHERE id = ?1", [&card_id])
        .map_err(|e| e.to_string())?;

    Ok(snapshot)
}

#[tauri::command]
pub fn restore_card(db: State<Database>, snapshot: DeletedCardSnapshot) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM cards WHERE id = ?1",
            [&snapshot.id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if exists {
        return Err("Card already exists".to_string());
    }

    conn.execute(
        "INSERT INTO cards (id, note_id, template_ordinal, state, difficulty, stability, due_at, last_review_at, reps, lapses, buried_until)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![
            &snapshot.id,
            &snapshot.note_id,
            snapshot.template_ordinal,
            &snapshot.state,
            snapshot.difficulty,
            snapshot.stability,
            snapshot.due_at,
            snapshot.last_review_at,
            snapshot.reps,
            snapshot.lapses,
            snapshot.buried_until,
        ],
    )
    .map_err(|e| e.to_string())?;

    let profile_ids: Vec<String> = conn
        .prepare("SELECT id FROM profiles")
        .map_err(|e| e.to_string())?
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for pid in &profile_ids {
        conn.execute(
            "INSERT INTO card_progress (profile_id, card_id, state, difficulty, stability, due_at, last_review_at, reps, lapses, buried_until)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                pid,
                &snapshot.id,
                &snapshot.state,
                snapshot.difficulty,
                snapshot.stability,
                snapshot.due_at,
                snapshot.last_review_at,
                snapshot.reps,
                snapshot.lapses,
                snapshot.buried_until,
            ],
        )
        .map_err(|e| e.to_string())?;
    }

    for log in &snapshot.review_logs {
        conn.execute(
            "INSERT INTO review_log (id, card_id, profile_id, reviewed_at, rating, elapsed_ms, scheduled_days, state_before, state_after)
             VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7, ?8)",
            (
                &log.id,
                &snapshot.id,
                log.reviewed_at,
                log.rating,
                log.elapsed_ms,
                log.scheduled_days,
                &log.state_before,
                &log.state_after,
            ),
        )
        .map_err(|e| e.to_string())?;
    }

    for triple_id in &snapshot.triple_ids {
        conn.execute(
            "INSERT OR IGNORE INTO card_triples (card_id, triple_id) VALUES (?1, ?2)",
            (&snapshot.id, triple_id),
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

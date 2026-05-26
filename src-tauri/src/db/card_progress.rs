use rusqlite::Connection;

/// One-time: copy shared `cards` scheduling into per-profile rows for every profile.
pub fn migrate_card_progress(conn: &Connection) -> Result<(), rusqlite::Error> {
    let progress_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM card_progress", [], |row| row.get(0))?;
    if progress_count > 0 {
        return Ok(());
    }

    let card_count: i64 = conn.query_row("SELECT COUNT(*) FROM cards", [], |row| row.get(0))?;
    if card_count == 0 {
        return Ok(());
    }

    conn.execute(
        "INSERT OR IGNORE INTO card_progress (
            profile_id, card_id, state, difficulty, stability, due_at,
            last_review_at, reps, lapses, buried_until
         )
         SELECT p.id, c.id, c.state, c.difficulty, c.stability, c.due_at,
                c.last_review_at, c.reps, c.lapses, c.buried_until
         FROM profiles p
         CROSS JOIN cards c",
        [],
    )?;
    Ok(())
}

/// New card: every profile starts with independent `new` scheduling.
pub fn seed_progress_for_all_profiles(
    conn: &Connection,
    card_id: &str,
    due_at: i64,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR IGNORE INTO card_progress (
            profile_id, card_id, state, difficulty, stability, due_at,
            last_review_at, reps, lapses, buried_until
         )
         SELECT id, ?1, 'new', 0.0, 0.0, ?2, NULL, 0, 0, NULL FROM profiles",
        (card_id, due_at),
    )?;
    Ok(())
}

/// New profile: every existing card is `new` for them.
pub fn seed_all_cards_for_profile(
    conn: &Connection,
    profile_id: &str,
    due_at: i64,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR IGNORE INTO card_progress (
            profile_id, card_id, state, difficulty, stability, due_at,
            last_review_at, reps, lapses, buried_until
         )
         SELECT ?1, c.id, 'new', 0.0, 0.0, ?2, NULL, 0, 0, NULL FROM cards c",
        (profile_id, due_at),
    )?;
    Ok(())
}

pub fn migrate_review_log_profile_id(conn: &Connection) -> Result<(), rusqlite::Error> {
    let has_column: bool = conn
        .prepare("PRAGMA table_info(review_log)")?
        .query_map([], |row| {
            let name: String = row.get(1)?;
            Ok(name == "profile_id")
        })?
        .filter_map(|r| r.ok())
        .any(|b| b);

    if !has_column {
        conn.execute(
            "ALTER TABLE review_log ADD COLUMN profile_id TEXT REFERENCES profiles(id)",
            [],
        )?;
    }

    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_review_log_profile ON review_log(profile_id, reviewed_at);",
    )?;
    Ok(())
}

/// Apply migrations safe for databases created before karma/profiles/card_progress.
pub fn apply_schema_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    migrate_review_log_profile_id(conn)?;
    migrate_card_progress(conn)?;
    Ok(())
}

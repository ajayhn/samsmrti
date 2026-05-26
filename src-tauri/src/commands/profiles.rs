use crate::db::card_progress;
use crate::db::Database;
use rusqlite::Connection;
use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

pub const ADMIN_PROFILE_ID: &str = "profile_admin";
const ACTIVE_PROFILE_KEY: &str = "active_profile_id";

#[derive(Debug, Clone, Serialize)]
pub struct Profile {
    pub id: String,
    pub display_name: String,
    pub is_admin: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct ActiveProfile {
    pub id: String,
    pub is_admin: bool,
}

pub fn load_active_profile_from_db(conn: &Connection) -> Result<ActiveProfile, String> {
    let id: String = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = ?1",
            [ACTIVE_PROFILE_KEY],
            |row| row.get(0),
        )
        .map_err(|_| "No active profile configured".to_string())?;

    let is_admin: bool = conn
        .query_row(
            "SELECT is_admin FROM profiles WHERE id = ?1",
            [&id],
            |row| Ok(row.get::<_, i64>(0)? != 0),
        )
        .map_err(|e| e.to_string())?;

    Ok(ActiveProfile { id, is_admin })
}

pub fn persist_active_profile(conn: &Connection, profile_id: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        (ACTIVE_PROFILE_KEY, profile_id),
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn row_to_profile(row: &rusqlite::Row<'_>) -> rusqlite::Result<Profile> {
    Ok(Profile {
        id: row.get(0)?,
        display_name: row.get(1)?,
        is_admin: row.get::<_, i64>(2)? != 0,
        created_at: row.get(3)?,
    })
}

#[tauri::command]
pub fn list_profiles(db: State<Database>) -> Result<Vec<Profile>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, display_name, is_admin, created_at FROM profiles ORDER BY is_admin DESC, display_name ASC",
        )
        .map_err(|e| e.to_string())?;
    let profiles = stmt
        .query_map([], row_to_profile)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(profiles)
}

#[tauri::command]
pub fn get_active_profile(
    db: State<Database>,
    active: State<'_, Mutex<ActiveProfile>>,
) -> Result<Profile, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let active_guard = active.lock().map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT id, display_name, is_admin, created_at FROM profiles WHERE id = ?1",
        [&active_guard.id],
        row_to_profile,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_active_profile(
    db: State<Database>,
    active: State<'_, Mutex<ActiveProfile>>,
    profile_id: String,
) -> Result<Profile, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let profile = conn
        .query_row(
            "SELECT id, display_name, is_admin, created_at FROM profiles WHERE id = ?1",
            [&profile_id],
            row_to_profile,
        )
        .map_err(|_| "Profile not found".to_string())?;

    persist_active_profile(&conn, &profile_id)?;

    let mut active_guard = active.lock().map_err(|e| e.to_string())?;
    active_guard.id = profile.id.clone();
    active_guard.is_admin = profile.is_admin;

    Ok(profile)
}

#[tauri::command]
pub fn create_profile(
    db: State<Database>,
    display_name: String,
) -> Result<Profile, String> {
    let name = display_name.trim();
    if name.is_empty() {
        return Err("Profile name cannot be empty".to_string());
    }

    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let id = format!("profile_{}", uuid::Uuid::new_v4().simple());
    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT INTO profiles (id, display_name, is_admin, created_at) VALUES (?1, ?2, 0, ?3)",
        (&id, name, now),
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO karma_state (profile_id, balance_cents, last_streak_bonus_at) VALUES (?1, 0, 0)",
        [&id],
    )
    .map_err(|e| e.to_string())?;

    card_progress::seed_all_cards_for_profile(&conn, &id, now)
        .map_err(|e| e.to_string())?;

    Ok(Profile {
        id,
        display_name: name.to_string(),
        is_admin: false,
        created_at: now,
    })
}

#[tauri::command]
pub fn delete_profile(
    db: State<Database>,
    active: State<'_, Mutex<ActiveProfile>>,
    profile_id: String,
) -> Result<(), String> {
    if profile_id == ADMIN_PROFILE_ID {
        return Err("Cannot delete the Admin profile".to_string());
    }

    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM profiles", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    if total <= 1 {
        return Err("Cannot delete the last profile".to_string());
    }

    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM profiles WHERE id = ?1",
            [&profile_id],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|e| e.to_string())?
        > 0;
    if !exists {
        return Err("Profile not found".to_string());
    }

    let active_guard = active.lock().map_err(|e| e.to_string())?;
    let deleting_active = active_guard.id == profile_id;
    drop(active_guard);

    conn.execute("DELETE FROM profiles WHERE id = ?1", [&profile_id])
        .map_err(|e| e.to_string())?;

    if deleting_active {
        let fallback_id: String = conn
            .query_row(
                "SELECT id FROM profiles WHERE is_admin = 0 ORDER BY created_at ASC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .or_else(|_| {
                conn.query_row(
                    "SELECT id FROM profiles WHERE id = ?1",
                    [ADMIN_PROFILE_ID],
                    |row| row.get(0),
                )
            })
            .map_err(|e| e.to_string())?;

        persist_active_profile(&conn, &fallback_id)?;
        let mut active_guard = active.lock().map_err(|e| e.to_string())?;
        let is_admin: bool = conn
            .query_row(
                "SELECT is_admin FROM profiles WHERE id = ?1",
                [&fallback_id],
                |row| Ok(row.get::<_, i64>(0)? != 0),
            )
            .map_err(|e| e.to_string())?;
        active_guard.id = fallback_id;
        active_guard.is_admin = is_admin;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    #[test]
    fn seed_includes_admin_and_default() {
        let db = Database::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let admin: (String, i64) = conn
            .query_row(
                "SELECT display_name, is_admin FROM profiles WHERE id = ?1",
                [ADMIN_PROFILE_ID],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(admin.0, "Admin");
        assert_eq!(admin.1, 1);

        let user_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM profiles WHERE is_admin = 0",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(user_count >= 1);
    }
}

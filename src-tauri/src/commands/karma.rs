use crate::commands::profiles::ActiveProfile;
use crate::db::Database;
use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

pub const REVIEW_CENTS_FULL: i64 = 10;
pub const REVIEW_CENTS_MID: i64 = 5;
pub const REVIEW_CENTS_LOW: i64 = 2;
pub const ADD_CENTS_PER_CARD: i64 = 20;
pub const DAILY_QUALIFY_BONUS_CENTS: i64 = 50;
pub const STREAK_MILESTONE_BONUS_CENTS: i64 = 500;
pub const ACTIVE_SECONDS_QUALIFY: i64 = 600;
pub const EFFECTIVE_ACTIONS_QUALIFY: i64 = 15;

#[derive(Debug, Clone, Serialize)]
pub struct KarmaEarnEvent {
    pub earned_cents: i64,
    pub balance_cents: i64,
    pub streak_days: i64,
    pub bonus_awarded_cents: i64,
    pub qualified_today: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DailyQualified {
    pub day: String,
    pub qualified: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct KarmaOverview {
    pub balance_cents: i64,
    pub streak_days: i64,
    pub qualified_today: bool,
    pub today_active_seconds: i64,
    pub today_effective_actions: i64,
    pub daily_qualified: Vec<DailyQualified>,
    pub profile_id: String,
    pub is_admin: bool,
}

fn today_utc() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}

fn admin_no_earn(active: &ActiveProfile) -> Option<KarmaEarnEvent> {
    if active.is_admin {
        Some(KarmaEarnEvent {
            earned_cents: 0,
            balance_cents: 0,
            streak_days: 0,
            bonus_awarded_cents: 0,
            qualified_today: false,
        })
    } else {
        None
    }
}

fn ensure_karma_state(conn: &Connection, profile_id: &str) -> Result<(), String> {
    conn.execute(
        "INSERT OR IGNORE INTO karma_state (profile_id, balance_cents, last_streak_bonus_at) VALUES (?1, 0, 0)",
        [profile_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn get_or_create_daily(
    conn: &Connection,
    profile_id: &str,
    day: &str,
) -> Result<(i64, i64, i64, i64, i64, i64), String> {
    conn.execute(
        "INSERT OR IGNORE INTO karma_daily (profile_id, day, active_seconds, review_count, add_count, qualified, earned_cents)
         VALUES (?1, ?2, 0, 0, 0, 0, 0)",
        (profile_id, day),
    )
    .map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT active_seconds, review_count, add_count, qualified, earned_cents,
                (SELECT balance_cents FROM karma_state WHERE profile_id = ?1)
         FROM karma_daily WHERE profile_id = ?1 AND day = ?2",
        (profile_id, day),
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        },
    )
    .map_err(|e| e.to_string())
}

fn effective_actions(review_count: i64, add_count: i64) -> i64 {
    review_count + add_count * 2
}

fn is_qualified(active_seconds: i64, review_count: i64, add_count: i64) -> bool {
    active_seconds >= ACTIVE_SECONDS_QUALIFY
        || effective_actions(review_count, add_count) >= EFFECTIVE_ACTIONS_QUALIFY
}

fn review_earn_cents(review_count_before: i64) -> i64 {
    let n = review_count_before + 1;
    if n <= 50 {
        REVIEW_CENTS_FULL
    } else if n <= 100 {
        REVIEW_CENTS_MID
    } else {
        REVIEW_CENTS_LOW
    }
}

fn calculate_streak(conn: &Connection, profile_id: &str) -> Result<i64, String> {
    let today = today_utc();
    let mut streak = 0i64;
    let mut expected = today;

    loop {
        let qualified: Option<i64> = conn
            .query_row(
                "SELECT qualified FROM karma_daily WHERE profile_id = ?1 AND day = ?2",
                (profile_id, &expected),
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;

        match qualified {
            Some(1) => {
                streak += 1;
                if let Ok(d) = chrono::NaiveDate::parse_from_str(&expected, "%Y-%m-%d") {
                    expected = (d - chrono::Duration::days(1))
                        .format("%Y-%m-%d")
                        .to_string();
                } else {
                    break;
                }
            }
            _ => break,
        }
    }

    Ok(streak)
}

fn apply_streak_bonus(
    conn: &Connection,
    profile_id: &str,
    streak: i64,
) -> Result<i64, String> {
    if streak == 0 || streak % 7 != 0 {
        return Ok(0);
    }

    let last_bonus: i64 = conn
        .query_row(
            "SELECT last_streak_bonus_at FROM karma_state WHERE profile_id = ?1",
            [profile_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if last_bonus >= streak {
        return Ok(0);
    }

    conn.execute(
        "UPDATE karma_state SET balance_cents = balance_cents + ?1, last_streak_bonus_at = ?2 WHERE profile_id = ?3",
        (STREAK_MILESTONE_BONUS_CENTS, streak, profile_id),
    )
    .map_err(|e| e.to_string())?;

    let day = today_utc();
    conn.execute(
        "UPDATE karma_daily SET earned_cents = earned_cents + ?1 WHERE profile_id = ?2 AND day = ?3",
        (STREAK_MILESTONE_BONUS_CENTS, profile_id, day),
    )
    .map_err(|e| e.to_string())?;

    Ok(STREAK_MILESTONE_BONUS_CENTS)
}

fn maybe_daily_qualify_bonus(
    conn: &Connection,
    profile_id: &str,
    day: &str,
    was_qualified: i64,
    active_seconds: i64,
    review_count: i64,
    add_count: i64,
) -> Result<i64, String> {
    if was_qualified != 0 {
        return Ok(0);
    }
    if !is_qualified(active_seconds, review_count, add_count) {
        return Ok(0);
    }

    conn.execute(
        "UPDATE karma_daily SET qualified = 1 WHERE profile_id = ?1 AND day = ?2",
        (profile_id, day),
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE karma_state SET balance_cents = balance_cents + ?1 WHERE profile_id = ?2",
        (DAILY_QUALIFY_BONUS_CENTS, profile_id),
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE karma_daily SET earned_cents = earned_cents + ?1 WHERE profile_id = ?2 AND day = ?3",
        (DAILY_QUALIFY_BONUS_CENTS, profile_id, day),
    )
    .map_err(|e| e.to_string())?;

    Ok(DAILY_QUALIFY_BONUS_CENTS)
}

fn finish_earn(
    conn: &Connection,
    profile_id: &str,
    base_earned: i64,
) -> Result<KarmaEarnEvent, String> {
    let day = today_utc();
    let (active_seconds, review_count, add_count, qualified, _, balance) =
        get_or_create_daily(conn, profile_id, &day)?;

    let qualify_bonus = maybe_daily_qualify_bonus(
        conn,
        profile_id,
        &day,
        qualified,
        active_seconds,
        review_count,
        add_count,
    )?;

    let qualified_now = is_qualified(active_seconds, review_count, add_count)
        || qualified != 0
        || qualify_bonus > 0;

    let streak = calculate_streak(conn, profile_id)?;
    let streak_bonus = apply_streak_bonus(conn, profile_id, streak)?;

    let balance_cents: i64 = conn
        .query_row(
            "SELECT balance_cents FROM karma_state WHERE profile_id = ?1",
            [profile_id],
            |row| row.get(0),
        )
        .unwrap_or(balance);

    let total_earned = base_earned + qualify_bonus + streak_bonus;

    Ok(KarmaEarnEvent {
        earned_cents: total_earned,
        balance_cents,
        streak_days: streak,
        bonus_awarded_cents: streak_bonus,
        qualified_today: qualified_now,
    })
}

pub fn earn_review_conn(
    conn: &Connection,
    active: &ActiveProfile,
    elapsed_ms: i64,
) -> Result<KarmaEarnEvent, String> {
    if let Some(ev) = admin_no_earn(active) {
        return Ok(ev);
    }

    let profile_id = &active.id;
    ensure_karma_state(conn, profile_id)?;
    let day = today_utc();

    let (_active_seconds, review_count, _add_count, _, _, _) =
        get_or_create_daily(conn, profile_id, &day)?;

    let earn = review_earn_cents(review_count);

    conn.execute(
        "UPDATE karma_daily SET review_count = review_count + 1,
         active_seconds = active_seconds + ?1,
         earned_cents = earned_cents + ?2
         WHERE profile_id = ?3 AND day = ?4",
        (
            (elapsed_ms / 2000).min(300),
            earn,
            profile_id,
            day,
        ),
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE karma_state SET balance_cents = balance_cents + ?1 WHERE profile_id = ?2",
        (earn, profile_id),
    )
    .map_err(|e| e.to_string())?;

    finish_earn(conn, profile_id, earn)
}

pub fn revert_review_conn(conn: &Connection, active: &ActiveProfile) -> Result<KarmaEarnEvent, String> {
    if let Some(ev) = admin_no_earn(active) {
        return Ok(ev);
    }

    let profile_id = &active.id;
    let day = today_utc();

    let review_count: i64 = conn
        .query_row(
            "SELECT review_count FROM karma_daily WHERE profile_id = ?1 AND day = ?2",
            (profile_id, &day),
            |row| row.get(0),
        )
        .unwrap_or(0);

    if review_count <= 0 {
        return finish_earn(conn, profile_id, 0);
    }

    let refund = review_earn_cents(review_count - 1);

    conn.execute(
        "UPDATE karma_daily SET review_count = MAX(review_count - 1, 0),
         earned_cents = MAX(earned_cents - ?1, 0)
         WHERE profile_id = ?2 AND day = ?3",
        (refund, profile_id, day),
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE karma_state SET balance_cents = MAX(balance_cents - ?1, 0) WHERE profile_id = ?2",
        (refund, profile_id),
    )
    .map_err(|e| e.to_string())?;

    finish_earn(conn, profile_id, -refund)
}

pub fn earn_add_conn(
    conn: &Connection,
    active: &ActiveProfile,
    card_count: i64,
) -> Result<KarmaEarnEvent, String> {
    if let Some(ev) = admin_no_earn(active) {
        return Ok(ev);
    }

    if card_count <= 0 {
        return finish_earn(conn, &active.id, 0);
    }

    let profile_id = &active.id;
    ensure_karma_state(conn, profile_id)?;
    let day = today_utc();
    get_or_create_daily(conn, profile_id, &day)?;

    let earn = ADD_CENTS_PER_CARD * card_count;

    conn.execute(
        "UPDATE karma_daily SET add_count = add_count + ?1, earned_cents = earned_cents + ?2
         WHERE profile_id = ?3 AND day = ?4",
        (card_count, earn, profile_id, day),
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE karma_state SET balance_cents = balance_cents + ?1 WHERE profile_id = ?2",
        (earn, profile_id),
    )
    .map_err(|e| e.to_string())?;

    finish_earn(conn, profile_id, earn)
}

pub fn count_cards_for_note(conn: &Connection, note_id: &str) -> Result<i64, String> {
    conn.query_row(
        "SELECT COUNT(*) FROM cards WHERE note_id = ?1",
        [note_id],
        |row| row.get(0),
    )
    .map_err(|e| e.to_string())
}

pub fn build_overview_conn(conn: &Connection, active: &ActiveProfile) -> Result<KarmaOverview, String> {
    if active.is_admin {
        return Ok(KarmaOverview {
            balance_cents: 0,
            streak_days: 0,
            qualified_today: false,
            today_active_seconds: 0,
            today_effective_actions: 0,
            daily_qualified: vec![],
            profile_id: active.id.clone(),
            is_admin: true,
        });
    }

    let profile_id = &active.id;
    ensure_karma_state(conn, profile_id)?;
    let day = today_utc();
    let (active_seconds, review_count, add_count, qualified, _, balance) =
        get_or_create_daily(conn, profile_id, &day)?;

    let streak = calculate_streak(conn, profile_id)?;

    let thirty_days_ago = (chrono::Utc::now() - chrono::Duration::days(30))
        .format("%Y-%m-%d")
        .to_string();

    let mut stmt = conn
        .prepare(
            "SELECT day, qualified FROM karma_daily
             WHERE profile_id = ?1 AND day >= ?2
             ORDER BY day DESC",
        )
        .map_err(|e| e.to_string())?;

    let daily_qualified: Vec<DailyQualified> = stmt
        .query_map((profile_id, thirty_days_ago), |row| {
            Ok(DailyQualified {
                day: row.get(0)?,
                qualified: row.get::<_, i64>(1)? != 0,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(KarmaOverview {
        balance_cents: balance,
        streak_days: streak,
        qualified_today: qualified != 0 || is_qualified(active_seconds, review_count, add_count),
        today_active_seconds: active_seconds,
        today_effective_actions: effective_actions(review_count, add_count),
        daily_qualified,
        profile_id: active.id.clone(),
        is_admin: false,
    })
}

#[tauri::command]
pub fn get_karma_overview(
    db: State<Database>,
    active: State<'_, Mutex<ActiveProfile>>,
) -> Result<KarmaOverview, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let active_guard = active.lock().map_err(|e| e.to_string())?;
    build_overview_conn(&conn, &active_guard)
}

#[tauri::command]
pub fn record_activity(
    db: State<Database>,
    active: State<'_, Mutex<ActiveProfile>>,
    seconds: i64,
) -> Result<KarmaOverview, String> {
    let active_guard = active.lock().map_err(|e| e.to_string())?;
    if active_guard.is_admin || seconds <= 0 {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        return build_overview_conn(&conn, &active_guard);
    }

    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let profile_id = active_guard.id.clone();
    let day = today_utc();
    ensure_karma_state(&conn, &profile_id)?;
    get_or_create_daily(&conn, &profile_id, &day)?;

    conn.execute(
        "UPDATE karma_daily SET active_seconds = active_seconds + ?1 WHERE profile_id = ?2 AND day = ?3",
        (seconds, &profile_id, &day),
    )
    .map_err(|e| e.to_string())?;

    let (active_seconds, review_count, add_count, qualified, _, _) =
        get_or_create_daily(&conn, &profile_id, &day)?;
    let _ = maybe_daily_qualify_bonus(
        &conn,
        &profile_id,
        &day,
        qualified,
        active_seconds,
        review_count,
        add_count,
    )?;

    build_overview_conn(&conn, &active_guard)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::profiles::ADMIN_PROFILE_ID;
    use crate::db::Database;

    fn test_profile(id: &str, admin: bool) -> ActiveProfile {
        ActiveProfile {
            id: id.to_string(),
            is_admin: admin,
        }
    }

    fn setup_user(conn: &Connection, id: &str) {
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO profiles (id, display_name, is_admin, created_at) VALUES (?1, ?2, 0, ?3)",
            (id, id, now),
        )
        .unwrap();
        conn.execute(
            "INSERT INTO karma_state (profile_id, balance_cents, last_streak_bonus_at) VALUES (?1, 0, 0)",
            [id],
        )
        .unwrap();
    }

    #[test]
    fn admin_earns_nothing() {
        let db = Database::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let admin = test_profile(ADMIN_PROFILE_ID, true);
        let ev = earn_review_conn(&conn, &admin, 5000).unwrap();
        assert_eq!(ev.earned_cents, 0);
        assert_eq!(ev.balance_cents, 0);
    }

    #[test]
    fn review_earn_per_profile() {
        let db = Database::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        setup_user(&conn, "user_a");
        setup_user(&conn, "user_b");

        let a = test_profile("user_a", false);
        let b = test_profile("user_b", false);

        let ev_a = earn_review_conn(&conn, &a, 1000).unwrap();
        assert_eq!(ev_a.earned_cents, 10);
        assert_eq!(ev_a.balance_cents, 10);

        let ev_b = earn_review_conn(&conn, &b, 1000).unwrap();
        assert_eq!(ev_b.balance_cents, 10);

        let overview_a = build_overview_conn(&conn, &a).unwrap();
        let overview_b = build_overview_conn(&conn, &b).unwrap();
        assert_eq!(overview_a.balance_cents, 10);
        assert_eq!(overview_b.balance_cents, 10);
    }

    #[test]
    fn diminishing_review_tiers() {
        assert_eq!(review_earn_cents(0), 10);
        assert_eq!(review_earn_cents(49), 10);
        assert_eq!(review_earn_cents(50), 5);
        assert_eq!(review_earn_cents(99), 5);
        assert_eq!(review_earn_cents(100), 2);
    }

    #[test]
    fn qualification_or_threshold() {
        assert!(!is_qualified(599, 0, 0));
        assert!(is_qualified(600, 0, 0));
        assert!(is_qualified(0, 10, 3)); // 10 + 6 = 16 effective
        assert!(!is_qualified(0, 5, 2)); // 5 + 4 = 9
    }

    #[test]
    fn streak_bonus_at_seven_days() {
        let db = Database::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        setup_user(&conn, "streak_user");
        for i in 0..7 {
            let d = (chrono::Utc::now() - chrono::Duration::days(i))
                .format("%Y-%m-%d")
                .to_string();
            conn.execute(
                "INSERT INTO karma_daily (profile_id, day, active_seconds, review_count, add_count, qualified, earned_cents)
                 VALUES (?1, ?2, 600, 0, 0, 1, 0)",
                ("streak_user", d),
            )
            .unwrap();
        }

        let streak = calculate_streak(&conn, "streak_user").unwrap();
        assert_eq!(streak, 7);

        let bonus = apply_streak_bonus(&conn, "streak_user", streak).unwrap();
        assert_eq!(bonus, STREAK_MILESTONE_BONUS_CENTS);

        let bonus_again = apply_streak_bonus(&conn, "streak_user", streak).unwrap();
        assert_eq!(bonus_again, 0);
    }
}

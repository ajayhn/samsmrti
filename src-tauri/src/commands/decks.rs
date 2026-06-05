use crate::commands::window_profiles::WindowProfiles;
use crate::db::deck_tree::{deck_scope_ids, direct_deck_counts, is_ancestor_of, rollup_deck_counts};
use crate::db::Database;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{State, WebviewWindow};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Deck {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub description: String,
    pub new_per_day: i64,
    pub max_reviews: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeckWithCounts {
    #[serde(flatten)]
    pub deck: Deck,
    pub total_cards: i64,
    pub due_cards: i64,
    pub new_cards: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateDeckInput {
    pub name: String,
    pub parent_id: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDeckInput {
    pub id: String,
    pub name: Option<String>,
    pub parent_id: Option<Option<String>>,
    pub description: Option<String>,
    pub new_per_day: Option<i64>,
    pub max_reviews: Option<i64>,
}

fn validate_parent(
    conn: &rusqlite::Connection,
    deck_id: &str,
    parent_id: &Option<String>,
) -> Result<(), String> {
    let Some(parent) = parent_id else {
        return Ok(());
    };
    if parent == deck_id {
        return Err("A deck cannot be its own parent.".to_string());
    }
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM decks WHERE id = ?1",
            [parent],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if !exists {
        return Err("Parent deck not found.".to_string());
    }
    if is_ancestor_of(conn, deck_id, parent).map_err(|e| e.to_string())? {
        return Err("Cannot move a deck under one of its subdecks.".to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn get_decks(
    db: State<Database>,
    window: WebviewWindow,
    profiles: State<'_, WindowProfiles>,
) -> Result<Vec<DeckWithCounts>, String> {
    let active = profiles.for_window(&window)?;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp();

    let mut stmt = conn
        .prepare(
            "SELECT id, name, parent_id, description, new_per_day, max_reviews, created_at, updated_at
             FROM decks ORDER BY name",
        )
        .map_err(|e| e.to_string())?;

    let rows: Vec<Deck> = stmt
        .query_map([], |row| {
            Ok(Deck {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                description: row.get(3)?,
                new_per_day: row.get(4)?,
                max_reviews: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let deck_ids: Vec<String> = rows.iter().map(|d| d.id.clone()).collect();
    let parent_of: HashMap<String, Option<String>> = rows
        .iter()
        .map(|d| (d.id.clone(), d.parent_id.clone()))
        .collect();
    let direct =
        direct_deck_counts(&conn, &active.id, now).map_err(|e| e.to_string())?;
    let rolled = rollup_deck_counts(&deck_ids, &parent_of, &direct);

    let decks = rows
        .into_iter()
        .map(|deck| {
            let (total_cards, due_cards, new_cards) =
                rolled.get(&deck.id).copied().unwrap_or((0, 0, 0));
            DeckWithCounts {
                deck,
                total_cards,
                due_cards,
                new_cards,
            }
        })
        .collect();

    Ok(decks)
}

#[tauri::command]
pub fn create_deck(db: State<Database>, input: CreateDeckInput) -> Result<Deck, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let id = format!("dk_{}", uuid::Uuid::new_v4().simple());
    let now = chrono::Utc::now().timestamp();
    let desc = input.description.unwrap_or_default();

    if let Some(ref parent) = input.parent_id {
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM decks WHERE id = ?1",
                [parent],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        if !exists {
            return Err("Parent deck not found.".to_string());
        }
    }

    conn.execute(
        "INSERT INTO decks (id, name, parent_id, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (&id, &input.name, &input.parent_id, &desc, now, now),
    )
    .map_err(|e| e.to_string())?;

    Ok(Deck {
        id,
        name: input.name,
        parent_id: input.parent_id,
        description: desc,
        new_per_day: 20,
        max_reviews: 200,
        created_at: now,
        updated_at: now,
    })
}

#[tauri::command]
pub fn update_deck(db: State<Database>, input: UpdateDeckInput) -> Result<Deck, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp();

    let current: Deck = conn
        .query_row(
            "SELECT id, name, parent_id, description, new_per_day, max_reviews, created_at, updated_at FROM decks WHERE id = ?1",
            [&input.id],
            |row| {
                Ok(Deck {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    parent_id: row.get(2)?,
                    description: row.get(3)?,
                    new_per_day: row.get(4)?,
                    max_reviews: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    let name = input.name.unwrap_or(current.name);
    let parent_id = input.parent_id.unwrap_or(current.parent_id);
    let description = input.description.unwrap_or(current.description);
    let new_per_day = input.new_per_day.unwrap_or(current.new_per_day);
    let max_reviews = input.max_reviews.unwrap_or(current.max_reviews);

    validate_parent(&conn, &input.id, &parent_id)?;

    conn.execute(
        "UPDATE decks SET name=?1, parent_id=?2, description=?3, new_per_day=?4, max_reviews=?5, updated_at=?6 WHERE id=?7",
        (&name, &parent_id, &description, new_per_day, max_reviews, now, &input.id),
    )
    .map_err(|e| e.to_string())?;

    Ok(Deck {
        id: input.id,
        name,
        parent_id,
        description,
        new_per_day,
        max_reviews,
        created_at: current.created_at,
        updated_at: now,
    })
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeletedDeckSnapshot {
    pub root_deck_id: String,
    pub data: serde_json::Value,
}

#[tauri::command]
pub fn delete_deck(db: State<Database>, id: String) -> Result<DeletedDeckSnapshot, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let snapshot_data =
        crate::backup::content::build_deck_delete_snapshot(&conn, &id).map_err(|e| e.to_string())?;
    let scope = deck_scope_ids(&conn, &id).map_err(|e| e.to_string())?;

    // Delete deepest subdecks first so notes/cards cascade cleanly.
    let mut by_depth: Vec<(usize, String)> = Vec::new();
    for deck_id in &scope {
        let depth: i64 = conn
            .query_row(
                "WITH RECURSIVE chain AS (
                    SELECT id, parent_id, 0 AS depth FROM decks WHERE id = ?1
                    UNION ALL
                    SELECT d.id, d.parent_id, chain.depth + 1
                    FROM decks d
                    INNER JOIN chain ON d.id = chain.parent_id
                 )
                 SELECT MAX(depth) FROM chain",
                [deck_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        by_depth.push((depth as usize, deck_id.clone()));
    }
    by_depth.sort_by(|a, b| b.0.cmp(&a.0));

    for (_, deck_id) in by_depth {
        conn.execute("DELETE FROM decks WHERE id = ?1", [&deck_id])
            .map_err(|e| e.to_string())?;
    }
    Ok(DeletedDeckSnapshot {
        root_deck_id: id,
        data: snapshot_data,
    })
}

#[tauri::command]
pub fn restore_deleted_deck(
    db: State<Database>,
    snapshot: DeletedDeckSnapshot,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::backup::content::restore_deck_delete_snapshot(&conn, &snapshot.data)
        .map_err(|e| e.to_string())
}

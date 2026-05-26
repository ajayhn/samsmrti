//! Idempotent seed: Chemistry → Polyatomic Ions deck with forward + reverse cards.

use crate::commands::search::upsert_note_fts_conn;
use crate::db::card_progress;
use rusqlite::Connection;

pub const CHEMISTRY_DECK_ID: &str = "dk_chemistry";
pub const POLYATOMIC_DECK_ID: &str = "dk_chemistry_polyatomic";

/// (ion name, formula with charge)
const IONS: &[(&str, &str)] = &[
    ("Acetate", "C₂H₃O₂⁻"),
    ("Ammonium", "NH₄⁺"),
    ("Bicarbonate (hydrogen carbonate)", "HCO₃⁻"),
    ("Bisulfate (hydrogen sulfate)", "HSO₄⁻"),
    ("Bisulfite (hydrogen sulfite)", "HSO₃⁻"),
    ("Bromate", "BrO₃⁻"),
    ("Carbonate", "CO₃²⁻"),
    ("Chlorate", "ClO₃⁻"),
    ("Chlorite", "ClO₂⁻"),
    ("Chromate", "CrO₄²⁻"),
    ("Cyanate", "OCN⁻"),
    ("Cyanide", "CN⁻"),
    ("Dichromate", "Cr₂O₇²⁻"),
    ("Dihydrogen phosphate", "H₂PO₄⁻"),
    ("Hydrogen phosphate", "HPO₄²⁻"),
    ("Hydroxide", "OH⁻"),
    ("Hypochlorite", "ClO⁻"),
    ("Iodate", "IO₃⁻"),
    ("Nitrate", "NO₃⁻"),
    ("Nitrite", "NO₂⁻"),
    ("Oxalate", "C₂O₄²⁻"),
    ("Permanganate", "MnO₄⁻"),
    ("Perchlorate", "ClO₄⁻"),
    ("Peroxide", "O₂²⁻"),
    ("Phosphate", "PO₄³⁻"),
    ("Phosphite", "PO₃³⁻"),
    ("Sulfate", "SO₄²⁻"),
    ("Sulfite", "SO₃²⁻"),
    ("Thiocyanate", "SCN⁻"),
    ("Thiosulfate", "S₂O₃²⁻"),
];

pub fn seed_chemistry_polyatomic_ions(conn: &Connection) -> Result<usize, rusqlite::Error> {
    let existing: i64 = conn.query_row(
        "SELECT COUNT(*) FROM notes WHERE deck_id = ?1",
        [POLYATOMIC_DECK_ID],
        |row| row.get(0),
    )?;
    if existing > 0 {
        return Ok(0);
    }

    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT OR IGNORE INTO decks (id, name, parent_id, description, new_per_day, max_reviews, created_at, updated_at)
         VALUES (?1, ?2, NULL, ?3, 20, 200, ?4, ?4)",
        (
            CHEMISTRY_DECK_ID,
            "Chemistry",
            "General chemistry topics",
            now,
        ),
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO decks (id, name, parent_id, description, new_per_day, max_reviews, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 20, 200, ?5, ?5)",
        (
            POLYATOMIC_DECK_ID,
            "Polyatomic Ions",
            CHEMISTRY_DECK_ID,
            "Common polyatomic ions — name ↔ formula (with charge)",
            now,
        ),
    )?;

    let mut added = 0usize;
    for (i, (name, formula)) in IONS.iter().enumerate() {
        let note_id = format!("n_pio_{i:03}");
        let fields = serde_json::json!({
            "Front": format!("{name}"),
            "Back": format!("{formula}"),
        })
        .to_string();

        conn.execute(
            "INSERT INTO notes (id, deck_id, note_type_id, fields_json, created_at, updated_at)
             VALUES (?1, ?2, 'nt_basic_rev', ?3, ?4, ?4)",
            (&note_id, POLYATOMIC_DECK_ID, &fields, now),
        )?;

        // Basic (and reversed): Card 1 name→formula, Card 2 formula→name
        for ordinal in 0..2 {
            let card_id = format!("c_pio_{i:03}_{ordinal}");
            conn.execute(
                "INSERT INTO cards (id, note_id, template_ordinal, state, due_at)
                 VALUES (?1, ?2, ?3, 'new', ?4)",
                (&card_id, &note_id, ordinal, now),
            )?;
            card_progress::seed_progress_for_all_profiles(conn, &card_id, now)?;
        }

        let _ = upsert_note_fts_conn(conn, &note_id, &fields);
        added += 1;
    }

    Ok(added)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    #[test]
    fn seeds_polyatomic_deck_with_reversed_cards() {
        let db = Database::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let added = seed_chemistry_polyatomic_ions(&conn).unwrap();
        assert_eq!(added, IONS.len());

        let parent: String = conn
            .query_row(
                "SELECT name FROM decks WHERE id = ?1",
                [CHEMISTRY_DECK_ID],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(parent, "Chemistry");

        let child_parent: String = conn
            .query_row(
                "SELECT parent_id FROM decks WHERE id = ?1",
                [POLYATOMIC_DECK_ID],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(child_parent, CHEMISTRY_DECK_ID);

        let note_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM notes WHERE deck_id = ?1",
                [POLYATOMIC_DECK_ID],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(note_count, IONS.len() as i64);

        let card_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM cards c JOIN notes n ON n.id = c.note_id WHERE n.deck_id = ?1",
                [POLYATOMIC_DECK_ID],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(card_count, (IONS.len() * 2) as i64);

        assert_eq!(seed_chemistry_polyatomic_ions(&conn).unwrap(), 0);
    }
}

mod countries_graph;
pub mod card_progress;
pub mod deck_tree;

use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub use countries_graph::sync_country_note;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn wal_checkpoint(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        Ok(())
    }

    /// Release the on-disk DB file so it can be replaced (opens an in-memory connection).
    pub fn release_db_file(&self) -> Result<(), rusqlite::Error> {
        let mut conn = self.conn.lock().unwrap();
        *conn = Connection::open_in_memory()?;
        Ok(())
    }

    /// Reopen the database at `db_path` after a full restore.
    pub fn reopen(&self, db_path: &Path) -> Result<(), rusqlite::Error> {
        let mut conn = self.conn.lock().unwrap();
        *conn = Connection::open(db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Ok(())
    }

    #[cfg(test)]
    pub fn in_memory() -> Result<Self, rusqlite::Error> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        let schema = include_str!("schema.sql");
        conn.execute_batch(schema)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.seed_profiles()?;
        db.seed_default_note_types()?;
        {
            let conn = db.conn.lock().unwrap();
            card_progress::apply_schema_migrations(&conn)?;
        }
        Ok(db)
    }

    pub fn new(app_data_dir: &PathBuf) -> Result<Self, rusqlite::Error> {
        fs::create_dir_all(app_data_dir).ok();
        let db_path = app_data_dir.join("samsmrti.db");
        let conn = Connection::open(db_path)?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        let schema = include_str!("schema.sql");
        conn.execute_batch(schema)?;

        let _ = conn.execute(
            "ALTER TABLE cards ADD COLUMN buried_until INTEGER",
            [],
        );

        let db = Self {
            conn: Mutex::new(conn),
        };
        db.seed_profiles()?;
        db.seed_default_note_types()?;
        {
            let conn = db.conn.lock().unwrap();
            card_progress::apply_schema_migrations(&conn)?;
            let now = chrono::Utc::now().timestamp();
            db.dedupe_card_templates(&conn)?;
            db.dedupe_note_type_fields(&conn)?;
            db.seed_country_note_type(&conn, now)?;
            db.seed_senator_note_type(&conn, now)?;
            let _ = countries_graph::sync_all_country_notes(&conn);
            let _ = crate::commands::search::ensure_search_index_conn(&conn);
            let _ = crate::seed::polyatomic_ions::seed_chemistry_polyatomic_ions(&conn);
        }
        Ok(db)
    }

    /// Remove duplicate fields (same note_type + name) when seed re-ran after a note-type edit.
    fn seed_profiles(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT OR IGNORE INTO profiles (id, display_name, is_admin, created_at) VALUES (?1, ?2, ?3, ?4)",
            ("profile_admin", "Admin", 1, now),
        )?;

        let non_admin_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM profiles WHERE is_admin = 0",
            [],
            |row| row.get(0),
        )?;

        if non_admin_count == 0 {
            conn.execute(
                "INSERT OR IGNORE INTO profiles (id, display_name, is_admin, created_at) VALUES (?1, ?2, ?3, ?4)",
                ("profile_default", "Default", 0, now),
            )?;
            conn.execute(
                "INSERT OR IGNORE INTO karma_state (profile_id, balance_cents, last_streak_bonus_at) VALUES (?1, 0, 0)",
                ["profile_default"],
            )?;
        }

        conn.execute(
            "INSERT OR IGNORE INTO karma_state (profile_id, balance_cents, last_streak_bonus_at) VALUES (?1, 0, 0)",
            ["profile_admin"],
        )?;

        let active_key = "active_profile_id";
        let existing: Option<String> = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = ?1",
                [active_key],
                |row| row.get(0),
            )
            .ok();

        if existing.is_none() {
            let default_id: String = conn
                .query_row(
                    "SELECT id FROM profiles WHERE is_admin = 0 ORDER BY created_at ASC LIMIT 1",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or_else(|_| "profile_admin".to_string());
            conn.execute(
                "INSERT INTO app_settings (key, value) VALUES (?1, ?2)",
                (active_key, default_id.as_str()),
            )?;
        }

        Ok(())
    }

    fn dedupe_note_type_fields(&self, conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        conn.execute_batch(
            "DELETE FROM fields
             WHERE id LIKE 'f_ctry_%'
               AND EXISTS (
                 SELECT 1 FROM fields f2
                 WHERE f2.note_type_id = fields.note_type_id
                   AND f2.name = fields.name
                   AND f2.id != fields.id
               );
             DELETE FROM fields
             WHERE rowid NOT IN (
               SELECT MIN(rowid) FROM fields GROUP BY note_type_id, name
             );",
        )?;
        Ok(())
    }

    /// Remove duplicate templates (same note_type + ordinal) left by seed + edit overlap.
    fn dedupe_card_templates(&self, conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        conn.execute_batch(
            "DELETE FROM card_templates
             WHERE id LIKE 'ct_ctry_%'
               AND note_type_id = 'nt_country'
               AND ordinal IN (
                 SELECT ordinal FROM card_templates
                 WHERE note_type_id = 'nt_country' AND id NOT LIKE 'ct_ctry_%'
               );
             DELETE FROM card_templates
             WHERE rowid NOT IN (
               SELECT MAX(rowid) FROM card_templates GROUP BY note_type_id, ordinal
             );",
        )?;
        Ok(())
    }

    fn seed_default_note_types(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();

        let count: i64 =
            conn.query_row("SELECT COUNT(*) FROM note_types", [], |row| row.get(0))?;

        if count > 0 {
            return Ok(());
        }

        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO note_types (id, name, css, is_cloze, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            ("nt_basic", "Basic", "", 0, now),
        )?;
        conn.execute(
            "INSERT INTO fields (id, note_type_id, name, ordinal) VALUES (?1, ?2, ?3, ?4)",
            ("f_basic_front", "nt_basic", "Front", 0),
        )?;
        conn.execute(
            "INSERT INTO fields (id, note_type_id, name, ordinal) VALUES (?1, ?2, ?3, ?4)",
            ("f_basic_back", "nt_basic", "Back", 1),
        )?;
        conn.execute(
            "INSERT INTO card_templates (id, note_type_id, name, front_html, back_html, ordinal) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            ("ct_basic", "nt_basic", "Card 1", "{{Front}}", "{{FrontSide}}<hr id=\"answer\">{{Back}}", 0),
        )?;

        conn.execute(
            "INSERT INTO note_types (id, name, css, is_cloze, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            ("nt_basic_rev", "Basic (and reversed)", "", 0, now),
        )?;
        conn.execute(
            "INSERT INTO fields (id, note_type_id, name, ordinal) VALUES (?1, ?2, ?3, ?4)",
            ("f_rev_front", "nt_basic_rev", "Front", 0),
        )?;
        conn.execute(
            "INSERT INTO fields (id, note_type_id, name, ordinal) VALUES (?1, ?2, ?3, ?4)",
            ("f_rev_back", "nt_basic_rev", "Back", 1),
        )?;
        conn.execute(
            "INSERT INTO card_templates (id, note_type_id, name, front_html, back_html, ordinal) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            ("ct_rev_1", "nt_basic_rev", "Card 1", "{{Front}}", "{{FrontSide}}<hr id=\"answer\">{{Back}}", 0),
        )?;
        conn.execute(
            "INSERT INTO card_templates (id, note_type_id, name, front_html, back_html, ordinal) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            ("ct_rev_2", "nt_basic_rev", "Card 2", "{{Back}}", "{{FrontSide}}<hr id=\"answer\">{{Front}}", 1),
        )?;

        conn.execute(
            "INSERT INTO note_types (id, name, css, is_cloze, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            ("nt_cloze", "Cloze", "", 1, now),
        )?;
        conn.execute(
            "INSERT INTO fields (id, note_type_id, name, ordinal) VALUES (?1, ?2, ?3, ?4)",
            ("f_cloze_text", "nt_cloze", "Text", 0),
        )?;
        conn.execute(
            "INSERT INTO fields (id, note_type_id, name, ordinal) VALUES (?1, ?2, ?3, ?4)",
            ("f_cloze_extra", "nt_cloze", "Extra", 1),
        )?;
        conn.execute(
            "INSERT INTO card_templates (id, note_type_id, name, front_html, back_html, ordinal) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            ("ct_cloze", "nt_cloze", "Cloze", "{{cloze:Text}}", "{{cloze:Text}}<br>{{Extra}}", 0),
        )?;

        self.seed_country_note_type(&conn, now)?;

        Ok(())
    }

    fn seed_senator_note_type(&self, conn: &rusqlite::Connection, now: i64) -> Result<(), rusqlite::Error> {
        const SENATOR_CSS: &str = r#"
.senator-front img, .senator-photo-back img { max-width: 180px; height: auto; display: block; margin: 0 auto 8px; }
.senator-name { text-align: center; margin-bottom: 8px; }
.senator-prompt { text-align: center; color: #666; }
.senator-answer { font-size: 1.1em; }
"#;
        const FRONT_PREFIX: &str = r#"<div class="senator-front">{{Photo}}<p class="senator-name"><strong>{{Name}}</strong></p><p class="senator-prompt">"#;
        const FRONT_SUFFIX: &str = "</p></div>";
        conn.execute(
            "INSERT OR IGNORE INTO note_types (id, name, css, is_cloze, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            ("nt_senator", "US Senator", SENATOR_CSS, 0, now),
        )?;

        let field_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM fields WHERE note_type_id = 'nt_senator'",
            [],
            |row| row.get(0),
        )?;
        if field_count == 0 {
            for (id, name, ord) in [
                ("f_sen_name", "Name", 0),
                ("f_sen_state", "State", 1),
                ("f_sen_party", "Party", 2),
                ("f_sen_since", "Since", 3),
                ("f_sen_background", "Background", 4),
                ("f_sen_trivia", "Trivia", 5),
                ("f_sen_personal", "PersonalFact", 6),
                ("f_sen_photo", "Photo", 7),
            ] {
                conn.execute(
                    "INSERT INTO fields (id, note_type_id, name, ordinal) VALUES (?1, ?2, ?3, ?4)",
                    (id, "nt_senator", name, ord),
                )?;
            }
        }

        let tmpl_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM card_templates WHERE note_type_id = 'nt_senator'",
            [],
            |row| row.get(0),
        )?;
        if tmpl_count == 0 {
            let front_state = format!(
                "{FRONT_PREFIX}Which state does this senator represent?{FRONT_SUFFIX}"
            );
            let front_party =
                format!("{FRONT_PREFIX}What party is this senator?{FRONT_SUFFIX}");
            conn.execute(
                "INSERT OR IGNORE INTO card_templates (id, note_type_id, name, front_html, back_html, ordinal) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                (
                    "ct_sen_state",
                    "nt_senator",
                    "State",
                    front_state,
                    Self::senator_back_html_state(),
                    0,
                ),
            )?;
            conn.execute(
                "INSERT OR IGNORE INTO card_templates (id, note_type_id, name, front_html, back_html, ordinal) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                (
                    "ct_sen_party",
                    "nt_senator",
                    "Party",
                    front_party,
                    Self::senator_back_html_party(),
                    1,
                ),
            )?;
        }

        self.patch_senator_templates(conn)?;
        Ok(())
    }

    /// Back templates must use literal `{{Field}}` — do not build via `format!` (Rust eats braces).
    fn senator_back_html_state() -> String {
        [
            r#"<p class="senator-answer"><strong>{{State}}</strong></p>"#,
            r#"<hr><p><strong>In Senate since</strong> {{Since}}</p><div class="senator-photo-back">{{Photo}}</div><p><strong>Background</strong><br>{{Background}}</p><p><strong>Trivia</strong><br>{{Trivia}}</p><p><strong>Surprising personal fact</strong><br>{{PersonalFact}}</p>"#,
        ]
        .concat()
    }

    fn senator_back_html_party() -> String {
        [
            r#"<p class="senator-answer"><strong>{{Party}}</strong></p>"#,
            r#"<hr><p><strong>In Senate since</strong> {{Since}}</p><div class="senator-photo-back">{{Photo}}</div><p><strong>Background</strong><br>{{Background}}</p><p><strong>Trivia</strong><br>{{Trivia}}</p><p><strong>Surprising personal fact</strong><br>{{PersonalFact}}</p>"#,
        ]
        .concat()
    }

    fn patch_senator_templates(&self, conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        conn.execute(
            "UPDATE card_templates SET back_html = ?1
             WHERE note_type_id = 'nt_senator' AND ordinal = 0",
            [Self::senator_back_html_state()],
        )?;
        conn.execute(
            "UPDATE card_templates SET back_html = ?1
             WHERE note_type_id = 'nt_senator' AND ordinal = 1",
            [Self::senator_back_html_party()],
        )?;
        Ok(())
    }

    fn seed_country_note_type(&self, conn: &rusqlite::Connection, now: i64) -> Result<(), rusqlite::Error> {
        conn.execute(
            "INSERT OR IGNORE INTO note_types (id, name, css, is_cloze, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            ("nt_country", "Country Details", "", 0, now),
        )?;

        let field_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM fields WHERE note_type_id = 'nt_country'",
            [],
            |row| row.get(0),
        )?;
        if field_count == 0 {
            for (id, name, ord) in [
                ("f_ctry_country", "Country", 0),
                ("f_ctry_capital", "Capital", 1),
                ("f_ctry_rivers", "Rivers", 2),
                ("f_ctry_languages", "Languages", 3),
                ("f_ctry_continent", "Continent", 4),
                ("f_ctry_mountains", "Mountains", 5),
                ("f_ctry_cities", "Cities", 6),
                ("f_ctry_unis", "Universities", 7),
                ("f_ctry_currency", "Currency", 8),
                ("f_ctry_flag", "Flag", 9),
            ] {
                conn.execute(
                    "INSERT INTO fields (id, note_type_id, name, ordinal) VALUES (?1, ?2, ?3, ?4)",
                    (id, "nt_country", name, ord),
                )?;
            }
        }

        let tmpl_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM card_templates WHERE note_type_id = 'nt_country'",
            [],
            |row| row.get(0),
        )?;
        if tmpl_count == 0 {
            conn.execute(
                "INSERT OR IGNORE INTO card_templates (id, note_type_id, name, front_html, back_html, ordinal) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                ("ct_ctry_cap", "nt_country", "Capital", "What is the capital of <b>{{Country}}</b>?", "{{Capital}}", 0),
            )?;
            conn.execute(
                "INSERT OR IGNORE INTO card_templates (id, note_type_id, name, front_html, back_html, ordinal) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                ("ct_ctry_river", "nt_country", "River → Country",
                 "River <b>{{each:Rivers}}</b> flows through which country{{hint_suffix}}?",
                 "{{Country}}", 1),
            )?;
            conn.execute(
                "INSERT OR IGNORE INTO card_templates (id, note_type_id, name, front_html, back_html, ordinal) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                ("ct_ctry_lang", "nt_country", "Language → Country",
                 "<b>{{each:Languages}}</b> is an official language of which country{{hint_suffix}}?",
                 "{{Country}}", 2),
            )?;
            conn.execute(
                "INSERT OR IGNORE INTO card_templates (id, note_type_id, name, front_html, back_html, ordinal) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                ("ct_ctry_mount", "nt_country", "Mountain → Country",
                 "Mountain <b>{{each:Mountains}}</b> is in which country{{hint_suffix}}?",
                 "{{Country}}", 3),
            )?;
            conn.execute(
                "INSERT OR IGNORE INTO card_templates (id, note_type_id, name, front_html, back_html, ordinal) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                ("ct_ctry_city", "nt_country", "City → Country",
                 "<b>{{each:Cities}}</b> is a city in which country{{hint_suffix}}?",
                 "{{Country}}", 4),
            )?;
            conn.execute(
                "INSERT OR IGNORE INTO card_templates (id, note_type_id, name, front_html, back_html, ordinal) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                ("ct_ctry_uni", "nt_country", "University → Country",
                 "<b>{{each:Universities}}</b> is a university in which country{{hint_suffix}}?",
                 "{{Country}}", 5),
            )?;
            conn.execute(
                "INSERT OR IGNORE INTO card_templates (id, note_type_id, name, front_html, back_html, ordinal) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                ("ct_ctry_curr", "nt_country", "Currency",
                 "What is the currency of <b>{{Country}}</b>?",
                 "{{Currency}}", 6),
            )?;
            conn.execute(
                "INSERT OR IGNORE INTO card_templates (id, note_type_id, name, front_html, back_html, ordinal) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                ("ct_ctry_flag", "nt_country", "Flag",
                 "Which country's flag is this?<br>{{Flag}}",
                 "{{Country}}", 7),
            )?;
            conn.execute(
                "INSERT OR IGNORE INTO card_templates (id, note_type_id, name, front_html, back_html, ordinal) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                ("ct_ctry_cont", "nt_country", "Continent",
                 "<b>{{Country}}</b> is on which continent?",
                 "{{Continent}}", 8),
            )?;
        }

        self.patch_country_reverse_templates(conn)?;
        Ok(())
    }

    /// Smart disambiguation hints: distinctive prompts omit the suffix; ambiguous ones get strong clues.
    /// Patches by ordinal — dedupe may have removed `ct_ctry_*` template ids.
    fn patch_country_reverse_templates(
        &self,
        conn: &rusqlite::Connection,
    ) -> Result<(), rusqlite::Error> {
        let updates: &[(i32, &str)] = &[
            (
                1,
                "River <b>{{each:Rivers}}</b> flows through which country{{hint_suffix}}?",
            ),
            (
                2,
                "<b>{{each:Languages}}</b> is an official language of which country{{hint_suffix}}?",
            ),
            (
                3,
                "Mountain <b>{{each:Mountains}}</b> is in which country{{hint_suffix}}?",
            ),
            (
                4,
                "<b>{{each:Cities}}</b> is a city in which country{{hint_suffix}}?",
            ),
            (
                5,
                "<b>{{each:Universities}}</b> is a university in which country{{hint_suffix}}?",
            ),
        ];
        for (ordinal, front) in updates {
            conn.execute(
                "UPDATE card_templates SET front_html = ?2
                 WHERE note_type_id = 'nt_country' AND ordinal = ?1",
                (*ordinal, *front),
            )?;
        }
        Ok(())
    }

    pub fn seed_example_decks(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();

        let deck_count: i64 = conn.query_row("SELECT COUNT(*) FROM decks", [], |row| row.get(0))?;
        if deck_count > 0 {
            return Ok(());
        }

        let now = chrono::Utc::now().timestamp();

        let examples: &[(&str, &str, &str, &[(&str, &str)])] = &[
            ("dk_ex_science", "Science", "Basic scientific concepts", &[
                ("What is the powerhouse of the cell?", "The mitochondria"),
                ("What is the chemical formula for water?", "H₂O"),
                ("What is the speed of light?", "Approximately 299,792,458 meters per second"),
                ("What is Newton's first law?", "An object at rest stays at rest, and an object in motion stays in motion, unless acted upon by an external force"),
                ("What planet is closest to the sun?", "Mercury"),
                ("What gas do plants absorb from the atmosphere?", "Carbon dioxide (CO₂)"),
                ("What is the boiling point of water at sea level?", "100°C (212°F)"),
                ("What is DNA?", "Deoxyribonucleic acid — the molecule that carries genetic instructions for life"),
            ]),
            ("dk_ex_math", "Mathematics", "Fundamental math concepts", &[
                ("What is the Pythagorean theorem?", "a² + b² = c² (for a right triangle)"),
                ("What is the value of π (pi) to 5 decimal places?", "3.14159"),
                ("What is the derivative of x²?", "2x"),
                ("What is the integral of 2x dx?", "x² + C"),
                ("What is the quadratic formula?", "x = (-b ± √(b²-4ac)) / 2a"),
                ("What is 0! (zero factorial)?", "1"),
                ("What is the sum of angles in a triangle?", "180 degrees"),
                ("What is Euler's identity?", "e^(iπ) + 1 = 0"),
            ]),
            ("dk_ex_history", "History", "Key historical events and figures", &[
                ("In what year did World War II end?", "1945"),
                ("Who was the first President of the United States?", "George Washington"),
                ("What was the Renaissance?", "A cultural movement from the 14th to 17th century, originating in Italy, emphasizing art, science, and humanism"),
                ("When did the Berlin Wall fall?", "November 9, 1989"),
                ("Who discovered America in 1492?", "Christopher Columbus (though indigenous peoples had lived there for millennia)"),
                ("What was the Industrial Revolution?", "The transition from agrarian to industrial economies, beginning in Britain around 1760"),
                ("When was the French Revolution?", "1789–1799"),
                ("Who wrote the Declaration of Independence?", "Thomas Jefferson (primary author)"),
            ]),
            ("dk_ex_geography", "Geography", "World geography basics", &[
                ("What is the largest ocean?", "The Pacific Ocean"),
                ("What is the longest river in the world?", "The Nile River (approximately 6,650 km)"),
                ("What is the largest continent by area?", "Asia"),
                ("What is the highest mountain in the world?", "Mount Everest (8,849 meters)"),
                ("What is the capital of Japan?", "Tokyo"),
                ("What is the largest desert in the world?", "Antarctica (technically a cold desert), or the Sahara (largest hot desert)"),
                ("How many continents are there?", "Seven: Asia, Africa, North America, South America, Antarctica, Europe, and Australia/Oceania"),
                ("What country has the largest population?", "India (as of 2023, surpassing China)"),
            ]),
        ];

        for (deck_id, name, desc, cards) in examples {
            conn.execute(
                "INSERT INTO decks (id, name, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                (deck_id, name, desc, now, now),
            )?;

            for (i, (front, back)) in cards.iter().enumerate() {
                let note_id = format!("n_ex_{}_{}", deck_id, i);
                let fields = serde_json::json!({"Front": front, "Back": back}).to_string();
                conn.execute(
                    "INSERT INTO notes (id, deck_id, note_type_id, fields_json, created_at, updated_at) VALUES (?1, ?2, 'nt_basic', ?3, ?4, ?5)",
                    (&note_id, deck_id, &fields, now, now),
                )?;

                let card_id = format!("c_ex_{}_{}", deck_id, i);
                conn.execute(
                    "INSERT INTO cards (id, note_id, template_ordinal, state, due_at) VALUES (?1, ?2, 0, 'new', ?3)",
                    (&card_id, &note_id, now),
                )?;
                card_progress::seed_progress_for_all_profiles(&conn, &card_id, now)?;
            }
        }

        Ok(())
    }
}

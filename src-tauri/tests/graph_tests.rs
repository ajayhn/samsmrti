use rusqlite::params;

const TEST_PROFILE: &str = "profile_test";

fn setup_db() -> rusqlite::Connection {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    let schema = include_str!("../src/db/schema.sql");
    conn.execute_batch(schema).unwrap();

    let now = chrono::Utc::now().timestamp();

    // Seed note types
    conn.execute(
        "INSERT INTO note_types (id, name, css, is_cloze, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        ("nt_basic", "Basic", "", 0, now),
    ).unwrap();
    conn.execute(
        "INSERT INTO fields (id, note_type_id, name, ordinal) VALUES (?1, ?2, ?3, ?4)",
        ("f_basic_front", "nt_basic", "Front", 0),
    ).unwrap();
    conn.execute(
        "INSERT INTO fields (id, note_type_id, name, ordinal) VALUES (?1, ?2, ?3, ?4)",
        ("f_basic_back", "nt_basic", "Back", 1),
    ).unwrap();
    conn.execute(
        "INSERT INTO card_templates (id, note_type_id, name, front_html, back_html, ordinal) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        ("ct_basic", "nt_basic", "Card 1", "{{Front}}", "{{FrontSide}}<hr>{{Back}}", 0),
    ).unwrap();

    // Seed a deck
    conn.execute(
        "INSERT INTO decks (id, name, description, new_per_day, max_reviews, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        ("dk_geo", "Geography", "World geography", 20, 200, now, now),
    ).unwrap();

    conn.execute(
        "INSERT OR IGNORE INTO profiles (id, display_name, is_admin, created_at) VALUES (?1, ?2, 0, ?3)",
        (TEST_PROFILE, "Test", now),
    )
    .unwrap();

    conn
}

fn add_card(conn: &rusqlite::Connection, note_id: &str, card_id: &str, deck_id: &str, front: &str, back: &str) {
    let now = chrono::Utc::now().timestamp();
    let fields = serde_json::json!({"Front": front, "Back": back}).to_string();
    conn.execute(
        "INSERT INTO notes (id, deck_id, note_type_id, fields_json, created_at, updated_at) VALUES (?1, ?2, 'nt_basic', ?3, ?4, ?5)",
        (note_id, deck_id, &fields, now, now),
    ).unwrap();
    conn.execute(
        "INSERT INTO cards (id, note_id, template_ordinal, state, due_at) VALUES (?1, ?2, 0, 'new', ?3)",
        (card_id, note_id, now),
    ).unwrap();
    samsmrti_lib::db::card_progress::seed_progress_for_all_profiles(&conn, card_id, now).unwrap();
}

// ═══════════════════════════════════════════════════════════════════════════
// Review queue tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_review_queue_returns_new_cards() {
    let conn = setup_db();
    add_card(&conn, "n1", "c1", "dk_geo", "Capital of France?", "Paris");
    add_card(&conn, "n2", "c2", "dk_geo", "Capital of Japan?", "Tokyo");
    add_card(&conn, "n3", "c3", "dk_geo", "Largest ocean?", "Pacific");

    let now = chrono::Utc::now().timestamp();
    let mut stmt = conn.prepare(
        "SELECT c.id, cp.state
         FROM cards c
         JOIN card_progress cp ON cp.card_id = c.id AND cp.profile_id = ?3
         JOIN notes n ON n.id = c.note_id
         WHERE n.deck_id = ?1
           AND ((cp.state = 'new') OR (cp.state IN ('learning','review','relearning') AND cp.due_at <= ?2))
         ORDER BY CASE cp.state WHEN 'learning' THEN 0 WHEN 'relearning' THEN 1 WHEN 'review' THEN 2 WHEN 'new' THEN 3 END, cp.due_at ASC
         LIMIT 220",
    ).unwrap();

    let results: Vec<(String, String)> = stmt
        .query_map(params!["dk_geo", now, TEST_PROFILE], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|(_, state)| state == "new"));
}

#[test]
fn test_review_queue_excludes_other_deck() {
    let conn = setup_db();
    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT INTO decks (id, name, description, new_per_day, max_reviews, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        ("dk_sci", "Science", "", 20, 200, now, now),
    ).unwrap();

    add_card(&conn, "n1", "c1", "dk_geo", "Capital?", "Paris");
    add_card(&conn, "n2", "c2", "dk_sci", "Powerhouse?", "Mitochondria");

    let mut stmt = conn.prepare(
        "SELECT c.id FROM cards c
         JOIN card_progress cp ON cp.card_id = c.id AND cp.profile_id = ?2
         JOIN notes n ON n.id = c.note_id
         WHERE n.deck_id = ?1 AND cp.state = 'new' LIMIT 220",
    ).unwrap();

    let results: Vec<String> = stmt
        .query_map(params!["dk_geo", TEST_PROFILE], |row| row.get(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0], "c1");
}

#[test]
fn test_due_cards_counted_correctly() {
    let conn = setup_db();
    let now = chrono::Utc::now().timestamp();
    add_card(&conn, "n1", "c1", "dk_geo", "Q1", "A1");
    add_card(&conn, "n2", "c2", "dk_geo", "Q2", "A2");

    // Move one card to reviewed state with past due date
    conn.execute(
        "UPDATE card_progress SET state = 'review', due_at = ?1 WHERE profile_id = ?2 AND card_id = 'c1'",
        params![now - 3600, TEST_PROFILE],
    ).unwrap();

    let counts = samsmrti_lib::db::deck_tree::direct_deck_counts(&conn, TEST_PROFILE, now).unwrap();
    let (total, due, new_count) = counts.get("dk_geo").copied().unwrap_or((0, 0, 0));

    assert_eq!(total, 2);
    assert_eq!(due, 1);
    assert_eq!(new_count, 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// Entity CRUD tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_entity_crud() {
    let conn = setup_db();
    let now = chrono::Utc::now().timestamp();

    // Create
    conn.execute(
        "INSERT INTO entities (id, name, entity_type, description, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        ("ent_1", "India", "Country", "Republic of India", now),
    ).unwrap();

    // Read
    let (name, etype): (String, Option<String>) = conn.query_row(
        "SELECT name, entity_type FROM entities WHERE id = 'ent_1'",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).unwrap();
    assert_eq!(name, "India");
    assert_eq!(etype.unwrap(), "Country");

    // Update
    conn.execute("UPDATE entities SET name = 'Bharat' WHERE id = 'ent_1'", []).unwrap();
    let updated_name: String = conn.query_row(
        "SELECT name FROM entities WHERE id = 'ent_1'", [], |row| row.get(0),
    ).unwrap();
    assert_eq!(updated_name, "Bharat");

    // Delete
    conn.execute("DELETE FROM entities WHERE id = 'ent_1'", []).unwrap();
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM entities", [], |row| row.get(0)).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_entity_search_by_name() {
    let conn = setup_db();
    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT INTO entities (id, name, entity_type, description, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        ("ent_1", "India", "Country", "", now),
    ).unwrap();
    conn.execute(
        "INSERT INTO entities (id, name, entity_type, description, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        ("ent_2", "Indiana Jones", "Character", "", now),
    ).unwrap();
    conn.execute(
        "INSERT INTO entities (id, name, entity_type, description, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        ("ent_3", "Japan", "Country", "", now),
    ).unwrap();

    let mut stmt = conn.prepare(
        "SELECT id FROM entities WHERE name LIKE ?1 ORDER BY name",
    ).unwrap();

    let results: Vec<String> = stmt
        .query_map(["%Ind%"], |row| row.get(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(results.len(), 2);
    assert!(results.contains(&"ent_1".to_string()));
    assert!(results.contains(&"ent_2".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════════
// Relation type tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_relation_type_with_inverse() {
    let conn = setup_db();
    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT INTO relation_types (id, name, inverse_name, created_at) VALUES (?1, ?2, ?3, ?4)",
        ("rt_1", "Has-River", "Flows-Through", now),
    ).unwrap();

    let (name, inverse): (String, Option<String>) = conn.query_row(
        "SELECT name, inverse_name FROM relation_types WHERE id = 'rt_1'",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).unwrap();

    assert_eq!(name, "Has-River");
    assert_eq!(inverse.unwrap(), "Flows-Through");
}

// ═══════════════════════════════════════════════════════════════════════════
// Triple tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_triple_creation_and_uniqueness() {
    let conn = setup_db();
    let now = chrono::Utc::now().timestamp();

    conn.execute("INSERT INTO entities (id, name, created_at) VALUES ('e1', 'India', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO entities (id, name, created_at) VALUES ('e2', 'Ganges', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO relation_types (id, name, created_at) VALUES ('rt1', 'Has-River', ?1)", [now]).unwrap();

    conn.execute(
        "INSERT INTO triples (id, subject_id, relation_type_id, object_id, created_at) VALUES ('t1', 'e1', 'rt1', 'e2', ?1)",
        [now],
    ).unwrap();

    // Duplicate should fail
    let result = conn.execute(
        "INSERT INTO triples (id, subject_id, relation_type_id, object_id, created_at) VALUES ('t2', 'e1', 'rt1', 'e2', ?1)",
        [now],
    );
    assert!(result.is_err());
}

#[test]
fn test_triple_cascade_on_entity_delete() {
    let conn = setup_db();
    let now = chrono::Utc::now().timestamp();

    conn.execute("INSERT INTO entities (id, name, created_at) VALUES ('e1', 'India', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO entities (id, name, created_at) VALUES ('e2', 'Ganges', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO relation_types (id, name, created_at) VALUES ('rt1', 'Has-River', ?1)", [now]).unwrap();
    conn.execute(
        "INSERT INTO triples (id, subject_id, relation_type_id, object_id, created_at) VALUES ('t1', 'e1', 'rt1', 'e2', ?1)",
        [now],
    ).unwrap();

    // Deleting entity should cascade-delete the triple
    conn.execute("DELETE FROM entities WHERE id = 'e1'", []).unwrap();
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM triples", [], |row| row.get(0)).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_triples_query_by_entity() {
    let conn = setup_db();
    let now = chrono::Utc::now().timestamp();

    conn.execute("INSERT INTO entities (id, name, created_at) VALUES ('e1', 'India', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO entities (id, name, created_at) VALUES ('e2', 'Ganges', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO entities (id, name, created_at) VALUES ('e3', 'Yamuna', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO entities (id, name, created_at) VALUES ('e4', 'Japan', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO relation_types (id, name, created_at) VALUES ('rt1', 'Has-River', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO relation_types (id, name, created_at) VALUES ('rt2', 'Borders', ?1)", [now]).unwrap();

    conn.execute("INSERT INTO triples (id, subject_id, relation_type_id, object_id, created_at) VALUES ('t1', 'e1', 'rt1', 'e2', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO triples (id, subject_id, relation_type_id, object_id, created_at) VALUES ('t2', 'e1', 'rt1', 'e3', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO triples (id, subject_id, relation_type_id, object_id, created_at) VALUES ('t3', 'e4', 'rt2', 'e1', ?1)", [now]).unwrap();

    // Query all triples involving India (as subject or object)
    let mut stmt = conn.prepare(
        "SELECT t.id FROM triples t WHERE t.subject_id = ?1 OR t.object_id = ?1",
    ).unwrap();
    let results: Vec<String> = stmt
        .query_map(["e1"], |row| row.get(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(results.len(), 3);
}

// ═══════════════════════════════════════════════════════════════════════════
// Card-triple linking tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_card_triple_linking() {
    let conn = setup_db();
    let now = chrono::Utc::now().timestamp();

    add_card(&conn, "n1", "c1", "dk_geo", "Rivers of India?", "Ganges, Yamuna");

    conn.execute("INSERT INTO entities (id, name, created_at) VALUES ('e1', 'India', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO entities (id, name, created_at) VALUES ('e2', 'Ganges', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO relation_types (id, name, created_at) VALUES ('rt1', 'Has-River', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO triples (id, subject_id, relation_type_id, object_id, created_at) VALUES ('t1', 'e1', 'rt1', 'e2', ?1)", [now]).unwrap();

    // Link card to triple
    conn.execute("INSERT INTO card_triples (card_id, triple_id) VALUES ('c1', 't1')", []).unwrap();

    // Verify link exists
    let linked: Vec<String> = conn
        .prepare("SELECT triple_id FROM card_triples WHERE card_id = 'c1'")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(linked, vec!["t1"]);

    // Verify card cascade: delete card should remove link
    conn.execute("DELETE FROM cards WHERE id = 'c1'", []).unwrap();
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM card_triples", [], |row| row.get(0)).unwrap();
    assert_eq!(count, 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// Mindmap data tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_mindmap_data_query() {
    let conn = setup_db();
    let now = chrono::Utc::now().timestamp();

    conn.execute("INSERT INTO entities (id, name, entity_type, created_at) VALUES ('e1', 'India', 'Country', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO entities (id, name, entity_type, created_at) VALUES ('e2', 'Ganges', 'River', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO entities (id, name, entity_type, created_at) VALUES ('e3', 'Delhi', 'City', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO relation_types (id, name, created_at) VALUES ('rt1', 'Has-River', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO relation_types (id, name, created_at) VALUES ('rt2', 'Capital', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO triples (id, subject_id, relation_type_id, object_id, created_at) VALUES ('t1', 'e1', 'rt1', 'e2', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO triples (id, subject_id, relation_type_id, object_id, created_at) VALUES ('t2', 'e1', 'rt2', 'e3', ?1)", [now]).unwrap();

    // Outgoing from India
    let mut out_stmt = conn.prepare(
        "SELECT rt.name, e.name FROM triples t
         JOIN relation_types rt ON rt.id = t.relation_type_id
         JOIN entities e ON e.id = t.object_id
         WHERE t.subject_id = 'e1'
         ORDER BY rt.name",
    ).unwrap();
    let outgoing: Vec<(String, String)> = out_stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(outgoing.len(), 2);
    assert_eq!(outgoing[0], ("Capital".to_string(), "Delhi".to_string()));
    assert_eq!(outgoing[1], ("Has-River".to_string(), "Ganges".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════════
// E-R-E review tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_ere_due_cards_query() {
    let conn = setup_db();
    let now = chrono::Utc::now().timestamp();

    add_card(&conn, "n1", "c1", "dk_geo", "Rivers?", "Ganges");
    add_card(&conn, "n2", "c2", "dk_geo", "Capital?", "Delhi");
    add_card(&conn, "n3", "c3", "dk_geo", "Unlinked", "Card");

    conn.execute("INSERT INTO entities (id, name, created_at) VALUES ('e1', 'India', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO entities (id, name, created_at) VALUES ('e2', 'Ganges', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO relation_types (id, name, created_at) VALUES ('rt1', 'Has-River', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO triples (id, subject_id, relation_type_id, object_id, created_at) VALUES ('t1', 'e1', 'rt1', 'e2', ?1)", [now]).unwrap();

    // Link only c1 to a triple
    conn.execute("INSERT INTO card_triples (card_id, triple_id) VALUES ('c1', 't1')", []).unwrap();

    // Query ERE due cards
    let mut stmt = conn.prepare(
        "SELECT DISTINCT c.id
         FROM cards c
         JOIN card_progress cp ON cp.card_id = c.id AND cp.profile_id = ?3
         JOIN notes n ON n.id = c.note_id
         JOIN card_triples ctr ON ctr.card_id = c.id
         WHERE n.deck_id = ?1
           AND ((cp.state = 'new') OR (cp.state IN ('learning','review','relearning') AND cp.due_at <= ?2))",
    ).unwrap();

    let due: Vec<String> = stmt
        .query_map(params!["dk_geo", now, TEST_PROFILE], |row| row.get(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(due.len(), 1);
    assert_eq!(due[0], "c1");
}

#[test]
fn test_ere_review_summary() {
    let conn = setup_db();
    let now = chrono::Utc::now().timestamp();

    add_card(&conn, "n1", "c1", "dk_geo", "Q1", "A1");
    add_card(&conn, "n2", "c2", "dk_geo", "Q2", "A2");

    conn.execute("INSERT INTO entities (id, name, created_at) VALUES ('e1', 'India', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO entities (id, name, created_at) VALUES ('e2', 'Ganges', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO relation_types (id, name, created_at) VALUES ('rt1', 'Has-River', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO triples (id, subject_id, relation_type_id, object_id, created_at) VALUES ('t1', 'e1', 'rt1', 'e2', ?1)", [now]).unwrap();

    conn.execute("INSERT INTO card_triples (card_id, triple_id) VALUES ('c1', 't1')", []).unwrap();
    conn.execute("INSERT INTO card_triples (card_id, triple_id) VALUES ('c2', 't1')", []).unwrap();

    // Summary: India should have 2 due (both cards linked via triple involving India)
    let mut stmt = conn.prepare(
        "SELECT e.id, e.name, COUNT(DISTINCT c.id) as due_count
         FROM entities e
         JOIN triples t ON (t.subject_id = e.id OR t.object_id = e.id)
         JOIN card_triples ctr ON ctr.triple_id = t.id
         JOIN cards c ON c.id = ctr.card_id
         JOIN card_progress cp ON cp.card_id = c.id AND cp.profile_id = ?3
         JOIN notes n ON n.id = c.note_id
         WHERE n.deck_id = ?1
           AND ((cp.state = 'new') OR (cp.state IN ('learning','review','relearning') AND cp.due_at <= ?2))
         GROUP BY e.id
         HAVING due_count > 0
         ORDER BY due_count DESC",
    ).unwrap();

    let summary: Vec<(String, String, i64)> = stmt
        .query_map(params!["dk_geo", now, TEST_PROFILE], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(summary.len(), 2); // India and Ganges both touched by the triple
    let india = summary.iter().find(|(id, _, _)| id == "e1").unwrap();
    assert_eq!(india.2, 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// Entity tags tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_entity_tags() {
    let conn = setup_db();
    let now = chrono::Utc::now().timestamp();

    conn.execute("INSERT INTO entities (id, name, created_at) VALUES ('e1', 'India', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO tags (id, name) VALUES ('t1', 'geography')", []).unwrap();
    conn.execute("INSERT INTO tags (id, name) VALUES ('t2', 'asia')", []).unwrap();
    conn.execute("INSERT INTO entity_tags (entity_id, tag_id) VALUES ('e1', 't1')", []).unwrap();
    conn.execute("INSERT INTO entity_tags (entity_id, tag_id) VALUES ('e1', 't2')", []).unwrap();

    // Query entity's tags
    let mut stmt = conn.prepare(
        "SELECT tg.name FROM tags tg JOIN entity_tags et ON tg.id = et.tag_id WHERE et.entity_id = 'e1' ORDER BY tg.name",
    ).unwrap();
    let tags: Vec<String> = stmt.query_map([], |row| row.get(0)).unwrap()
        .collect::<Result<Vec<_>, _>>().unwrap();

    assert_eq!(tags, vec!["asia", "geography"]);

    // Filter entities by tag
    let mut stmt2 = conn.prepare(
        "SELECT e.id FROM entities e WHERE EXISTS (SELECT 1 FROM entity_tags et JOIN tags tg ON tg.id = et.tag_id WHERE et.entity_id = e.id AND tg.name = 'asia')",
    ).unwrap();
    let filtered: Vec<String> = stmt2.query_map([], |row| row.get(0)).unwrap()
        .collect::<Result<Vec<_>, _>>().unwrap();

    assert_eq!(filtered, vec!["e1"]);
}

// ═══════════════════════════════════════════════════════════════════════════
// Card suggestion tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_unlinked_triples_for_suggestions() {
    let conn = setup_db();
    let now = chrono::Utc::now().timestamp();

    add_card(&conn, "n1", "c1", "dk_geo", "Q", "A");

    conn.execute("INSERT INTO entities (id, name, created_at) VALUES ('e1', 'India', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO entities (id, name, created_at) VALUES ('e2', 'Ganges', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO entities (id, name, created_at) VALUES ('e3', 'Yamuna', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO relation_types (id, name, inverse_name, created_at) VALUES ('rt1', 'Has-River', 'Flows-Through', ?1)", [now]).unwrap();

    conn.execute("INSERT INTO triples (id, subject_id, relation_type_id, object_id, created_at) VALUES ('t1', 'e1', 'rt1', 'e2', ?1)", [now]).unwrap();
    conn.execute("INSERT INTO triples (id, subject_id, relation_type_id, object_id, created_at) VALUES ('t2', 'e1', 'rt1', 'e3', ?1)", [now]).unwrap();

    // Link c1 to t1 only - t2 remains unlinked
    conn.execute("INSERT INTO card_triples (card_id, triple_id) VALUES ('c1', 't1')", []).unwrap();

    // Query unlinked triples
    let mut stmt = conn.prepare(
        "SELECT t.id FROM triples t WHERE NOT EXISTS (SELECT 1 FROM card_triples ct WHERE ct.triple_id = t.id)",
    ).unwrap();
    let unlinked: Vec<String> = stmt.query_map([], |row| row.get(0)).unwrap()
        .collect::<Result<Vec<_>, _>>().unwrap();

    assert_eq!(unlinked.len(), 1);
    assert_eq!(unlinked[0], "t2");
}

// ═══════════════════════════════════════════════════════════════════════════
// Schema integrity tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_full_schema_creates_all_tables() {
    let conn = setup_db();

    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let expected = vec![
        "card_templates", "card_triples", "cards", "decks",
        "entities", "entity_tags", "fields", "note_links",
        "note_tags", "note_types", "notes", "notes_fts",
        "relation_type_tags", "relation_types", "review_log",
        "tags", "triples",
    ];

    for t in &expected {
        assert!(tables.contains(&t.to_string()), "Missing table: {}", t);
    }
}

#[test]
fn test_country_note_syncs_graph_triples_and_card_links() {
    let conn = setup_db();
    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT INTO note_types (id, name, css, is_cloze, created_at) VALUES ('nt_country', 'Country Details', '', 0, ?1)",
        [now],
    )
    .unwrap();

    let fields = serde_json::json!({
        "Country": "France",
        "Capital": "Paris",
        "Rivers": "Seine, Loire",
        "Languages": "French",
        "Continent": "Europe",
        "Mountains": "Mont Blanc",
        "Cities": "Lyon",
        "Universities": "Sorbonne University",
        "Currency": "Euro (EUR)",
        "Flag": "🇫🇷"
    })
    .to_string();

    conn.execute(
        "INSERT INTO notes (id, deck_id, note_type_id, fields_json, created_at, updated_at) VALUES ('n_fr', 'dk_geo', 'nt_country', ?1, ?2, ?2)",
        (&fields, now),
    )
    .unwrap();

    conn.execute(
        "INSERT INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_fr_cap', 'n_fr', 0, 'new', ?1)",
        [now],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_fr_river0', 'n_fr', 1000, 'new', ?1)",
        [now],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO cards (id, note_id, template_ordinal, state, due_at) VALUES ('c_fr_river1', 'n_fr', 1001, 'new', ?1)",
        [now],
    )
    .unwrap();

    samsmrti_lib::db::sync_country_note(&conn, "n_fr").unwrap();

    let triple_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM triples", [], |row| row.get(0))
        .unwrap();
    assert!(triple_count >= 7, "expected multiple triples, got {triple_count}");

    let france_id: String = conn
        .query_row(
            "SELECT id FROM entities WHERE name = 'France' AND entity_type = 'Country'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    let capital_link: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM card_triples ct
             JOIN triples t ON t.id = ct.triple_id
             JOIN entities e ON e.id = t.object_id
             WHERE ct.card_id = 'c_fr_cap' AND t.subject_id = ?1 AND e.name = 'Paris'",
            [&france_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(capital_link, 1);

    let river_links: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM card_triples WHERE card_id IN ('c_fr_river0', 'c_fr_river1')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(river_links, 2);
}

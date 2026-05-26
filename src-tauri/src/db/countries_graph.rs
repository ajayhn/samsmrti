//! Build knowledge-map entities, triples, and card links for Country Details notes.

use rusqlite::{Connection, OptionalExtension};
use std::collections::HashMap;

const NOTE_TYPE_COUNTRY: &str = "nt_country";

struct CountryFields {
    country: String,
    capital: String,
    rivers: Vec<String>,
    languages: Vec<String>,
    continent: String,
    mountains: Vec<String>,
    cities: Vec<String>,
    universities: Vec<String>,
    currency: String,
}

fn split_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn field_str(fields: &serde_json::Value, key: &str) -> String {
    fields
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string()
}

fn parse_country_fields(fields: &serde_json::Value) -> Option<CountryFields> {
    let country = field_str(fields, "Country");
    if country.is_empty() {
        return None;
    }
    Some(CountryFields {
        country,
        capital: field_str(fields, "Capital"),
        rivers: split_csv(&field_str(fields, "Rivers")),
        languages: split_csv(&field_str(fields, "Languages")),
        continent: field_str(fields, "Continent"),
        mountains: split_csv(&field_str(fields, "Mountains")),
        cities: split_csv(&field_str(fields, "Cities")),
        universities: split_csv(&field_str(fields, "Universities")),
        currency: field_str(fields, "Currency"),
    })
}

fn normalize_entity_type(t: &str) -> String {
    let s = t.trim();
    if s.is_empty() {
        return String::new();
    }
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().to_string() + &c.as_str().to_lowercase(),
    }
}

fn ensure_entity(
    conn: &Connection,
    name: &str,
    entity_type: &str,
    now: i64,
) -> Result<String, rusqlite::Error> {
    let et = normalize_entity_type(entity_type);
    let existing: Option<String> = conn
        .query_row(
            "SELECT id FROM entities WHERE name = ?1 AND entity_type = ?2",
            (name, &et),
            |row| row.get(0),
        )
        .optional()?;
    if let Some(id) = existing {
        return Ok(id);
    }
    let id = format!("ent_{}", uuid::Uuid::new_v4().simple());
    conn.execute(
        "INSERT INTO entities (id, name, entity_type, description, created_at) VALUES (?1, ?2, ?3, '', ?4)",
        (&id, name, &et, now),
    )?;
    Ok(id)
}

fn ensure_relation_type(
    conn: &Connection,
    id: &str,
    name: &str,
    inverse: Option<&str>,
    now: i64,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR IGNORE INTO relation_types (id, name, inverse_name, created_at) VALUES (?1, ?2, ?3, ?4)",
        (id, name, inverse, now),
    )?;
    Ok(())
}

fn ensure_country_relation_types(conn: &Connection, now: i64) -> Result<(), rusqlite::Error> {
    ensure_relation_type(conn, "rt_ctry_capital", "Capital", Some("Capital-Of"), now)?;
    ensure_relation_type(conn, "rt_ctry_river", "Has-River", Some("Flows-Through"), now)?;
    ensure_relation_type(
        conn,
        "rt_ctry_language",
        "Has-Language",
        Some("Official-Language-Of"),
        now,
    )?;
    ensure_relation_type(conn, "rt_ctry_mountain", "Has-Mountain", Some("Mountain-In"), now)?;
    ensure_relation_type(conn, "rt_ctry_city", "Has-City", Some("City-In"), now)?;
    ensure_relation_type(
        conn,
        "rt_ctry_university",
        "Has-University",
        Some("University-In"),
        now,
    )?;
    ensure_relation_type(
        conn,
        "rt_ctry_currency",
        "Has-Currency",
        Some("Currency-Of"),
        now,
    )?;
    ensure_relation_type(
        conn,
        "rt_ctry_continent",
        "On-Continent",
        Some("Contains-Country"),
        now,
    )?;
    Ok(())
}

fn ensure_triple(
    conn: &Connection,
    subject_id: &str,
    relation_type_id: &str,
    object_id: &str,
    now: i64,
) -> Result<String, rusqlite::Error> {
    let existing: Option<String> = conn
        .query_row(
            "SELECT id FROM triples WHERE subject_id = ?1 AND relation_type_id = ?2 AND object_id = ?3",
            (subject_id, relation_type_id, object_id),
            |row| row.get(0),
        )
        .optional()?;

    if let Some(id) = existing {
        return Ok(id);
    }

    let id = format!("trp_{}", uuid::Uuid::new_v4().simple());
    conn.execute(
        "INSERT INTO triples (id, subject_id, relation_type_id, object_id, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        (&id, subject_id, relation_type_id, object_id, now),
    )?;
    Ok(id)
}

fn link_card_to_triple(conn: &Connection, card_id: &str, triple_id: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR IGNORE INTO card_triples (card_id, triple_id) VALUES (?1, ?2)",
        (card_id, triple_id),
    )?;
    Ok(())
}

fn base_template_ordinal(template_ordinal: i64) -> i64 {
    if template_ordinal >= 1000 {
        template_ordinal / 1000
    } else {
        template_ordinal
    }
}

fn item_index(template_ordinal: i64) -> usize {
    if template_ordinal >= 1000 {
        (template_ordinal % 1000) as usize
    } else {
        0
    }
}

/// Build triples for one country note and link each card to its matching triple.
pub fn sync_country_note(conn: &Connection, note_id: &str) -> Result<(), rusqlite::Error> {
    let now = chrono::Utc::now().timestamp();
    ensure_country_relation_types(conn, now)?;

    let (note_type_id, fields_str): (String, String) = conn.query_row(
        "SELECT note_type_id, fields_json FROM notes WHERE id = ?1",
        [note_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    if note_type_id != NOTE_TYPE_COUNTRY {
        return Ok(());
    }

    let fields: serde_json::Value = serde_json::from_str(&fields_str).unwrap_or_default();
    let Some(cf) = parse_country_fields(&fields) else {
        return Ok(());
    };

    let country_id = ensure_entity(conn, &cf.country, "Country", now)?;

    let mut triple_by_slot: HashMap<(i64, usize), String> = HashMap::new();

    if !cf.capital.is_empty() {
        let capital_id = ensure_entity(conn, &cf.capital, "City", now)?;
        let tid = ensure_triple(conn, &country_id, "rt_ctry_capital", &capital_id, now)?;
        triple_by_slot.insert((0, 0), tid);
    }

    for (i, river) in cf.rivers.iter().enumerate() {
        let river_id = ensure_entity(conn, river, "River", now)?;
        let tid = ensure_triple(conn, &country_id, "rt_ctry_river", &river_id, now)?;
        triple_by_slot.insert((1, i), tid);
    }

    for (i, lang) in cf.languages.iter().enumerate() {
        let lang_id = ensure_entity(conn, lang, "Language", now)?;
        let tid = ensure_triple(conn, &country_id, "rt_ctry_language", &lang_id, now)?;
        triple_by_slot.insert((2, i), tid);
    }

    for (i, mountain) in cf.mountains.iter().enumerate() {
        let mount_id = ensure_entity(conn, mountain, "Mountain", now)?;
        let tid = ensure_triple(conn, &country_id, "rt_ctry_mountain", &mount_id, now)?;
        triple_by_slot.insert((3, i), tid);
    }

    for (i, city) in cf.cities.iter().enumerate() {
        let city_id = ensure_entity(conn, city, "City", now)?;
        let tid = ensure_triple(conn, &country_id, "rt_ctry_city", &city_id, now)?;
        triple_by_slot.insert((4, i), tid);
    }

    for (i, uni) in cf.universities.iter().enumerate() {
        let uni_id = ensure_entity(conn, uni, "University", now)?;
        let tid = ensure_triple(conn, &country_id, "rt_ctry_university", &uni_id, now)?;
        triple_by_slot.insert((5, i), tid);
    }

    if !cf.currency.is_empty() {
        let currency_id = ensure_entity(conn, &cf.currency, "Currency", now)?;
        let tid = ensure_triple(conn, &country_id, "rt_ctry_currency", &currency_id, now)?;
        triple_by_slot.insert((6, 0), tid);
    }

    if !cf.continent.is_empty() {
        let continent_id = ensure_entity(conn, &cf.continent, "Continent", now)?;
        let tid = ensure_triple(conn, &country_id, "rt_ctry_continent", &continent_id, now)?;
        triple_by_slot.insert((8, 0), tid.clone());
        // Flag card (template 7) shares the continent triple as geographic context
        triple_by_slot.insert((7, 0), tid);
    }

    let mut card_stmt = conn.prepare(
        "SELECT id, template_ordinal FROM cards WHERE note_id = ?1",
    )?;
    let cards = card_stmt.query_map([note_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })?;

    for card in cards {
        let (card_id, template_ordinal) = card?;
        let base = base_template_ordinal(template_ordinal);
        let idx = item_index(template_ordinal);
        if let Some(triple_id) = triple_by_slot.get(&(base, idx)) {
            link_card_to_triple(conn, &card_id, triple_id)?;
        }
    }

    Ok(())
}

/// Sync graph links for every Country Details note (e.g. Countries deck on startup).
pub fn sync_all_country_notes(conn: &Connection) -> Result<(), rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id FROM notes WHERE note_type_id = ?1",
    )?;
    let note_ids: Vec<String> = stmt
        .query_map([NOTE_TYPE_COUNTRY], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    for note_id in note_ids {
        sync_country_note(conn, &note_id)?;
    }
    Ok(())
}

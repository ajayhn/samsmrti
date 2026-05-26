use crate::commands::search::upsert_note_fts_conn;
use crate::import::ImportResult;
use rusqlite::Connection;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const DECK_ID: &str = "dk_senate";
const DECK_NAME: &str = "US Senate (119th)";
const NOTE_TYPE_ID: &str = "nt_senator";

#[derive(Debug, Deserialize)]
struct SenatorJson {
    name: String,
    state: String,
    party: String,
    since: i64,
    #[serde(default)]
    background: String,
    #[serde(default, rename = "funFact")]
    fun_fact: String,
    #[serde(default, rename = "personalFact")]
    personal_fact: String,
}

pub struct SenatorRecord {
    pub name: String,
    pub state: String,
    pub party: String,
    pub since: String,
    pub background: String,
    pub trivia: String,
    pub personal_fact: String,
    pub photo: String,
}

fn load_senators_json(path: &Path) -> Result<Vec<SenatorJson>, String> {
    let data = fs::read_to_string(path).map_err(|e| format!("Read {}: {e}", path.display()))?;
    serde_json::from_str(&data).map_err(|e| format!("Parse JSON: {e}"))
}

fn json_to_record(j: SenatorJson) -> SenatorRecord {
    SenatorRecord {
        name: j.name,
        state: j.state,
        party: normalize_party(&j.party),
        since: j.since.to_string(),
        background: j.background,
        trivia: j.fun_fact,
        personal_fact: j.personal_fact,
        photo: String::new(),
    }
}

fn normalize_party(code: &str) -> String {
    match code.trim() {
        "R" | "Republican" => "Republican".to_string(),
        "D" | "Democrat" | "Democratic" => "Democrat".to_string(),
        "I" | "Independent" => "Independent".to_string(),
        other => other.to_string(),
    }
}

fn senator_slug(name: &str) -> String {
    let mut s = String::new();
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            s.push(c.to_ascii_lowercase());
        } else if (c == ' ' || c == '-' || c == '\'') && !s.ends_with('_') && !s.is_empty() {
            s.push('_');
        }
    }
    s.trim_matches('_').to_string()
}

fn photo_html(url: &str, alt: &str) -> String {
    let esc_alt = alt.replace('&', "&amp;").replace('"', "&quot;");
    let esc_url = url.replace('&', "&amp;");
    format!(r#"<img src="{esc_url}" alt="{esc_alt}">"#)
}

/// Decode quoted-printable content from MHTML saves (enough for senator grid parsing).
pub fn decode_quoted_printable(input: &str) -> String {
    let mut raw = String::new();
    let mut line_buf = String::new();
    for line in input.lines() {
        let trimmed = line.trim_end();
        if trimmed.ends_with('=') && trimmed.len() > 1 {
            line_buf.push_str(&trimmed[..trimmed.len() - 1]);
        } else {
            line_buf.push_str(trimmed);
            raw.push_str(&line_buf);
            raw.push('\n');
            line_buf.clear();
        }
    }
    if !line_buf.is_empty() {
        raw.push_str(&line_buf);
    }

    let mut out = String::with_capacity(raw.len());
    let bytes = raw.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'=' && i + 2 < bytes.len() {
            if let Ok(hex) = u8::from_str_radix(
                std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""),
                16,
            ) {
                out.push(hex as char);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

/// Extract senator name → photo HTML from saved senators_standalone.html grid.
pub fn photos_from_html(html_path: &Path) -> Result<HashMap<String, String>, String> {
    let raw = fs::read_to_string(html_path).map_err(|e| format!("Read HTML: {e}"))?;
    let html = decode_quoted_printable(&raw);
    let mut photos = HashMap::new();

    for chunk in html.split("class=\"card\"").skip(1) {
        let name = extract_between(chunk, "class=\"card-name\">", "</div>");
        let Some(name) = name.map(str::trim).filter(|s| !s.is_empty()) else {
            continue;
        };
        if let Some(img_tag) = extract_img_tag(chunk) {
            if let (Some(src), alt) = parse_img_src_alt(&img_tag) {
                let alt = alt.unwrap_or(name);
                photos.insert(name.to_string(), photo_html(src, alt));
            }
        }
    }

    Ok(photos)
}

fn extract_between<'a>(hay: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let i = hay.find(start)? + start.len();
    let rest = &hay[i..];
    let j = rest.find(end)?;
    Some(&rest[..j])
}

fn extract_img_tag(chunk: &str) -> Option<String> {
    let i = chunk.find("<img")?;
    let rest = &chunk[i..];
    let j = rest.find('>')? + 1;
    Some(rest[..j].to_string())
}

fn parse_img_src_alt(tag: &str) -> (Option<&str>, Option<&str>) {
    let src = extract_attr(tag, "src");
    let alt = extract_attr(tag, "alt");
    (src, alt)
}

fn extract_attr<'a>(tag: &'a str, attr: &str) -> Option<&'a str> {
    let needle = format!("{attr}=\"");
    let i = tag.find(&needle)? + needle.len();
    let rest = &tag[i..];
    let j = rest.find('"')?;
    Some(&rest[..j])
}

pub fn merge_photos(records: &mut [SenatorRecord], photos: HashMap<String, String>) {
    for rec in records.iter_mut() {
        if let Some(html) = photos.get(&rec.name) {
            rec.photo = html.clone();
        }
    }
}

pub fn import_senators_conn(
    conn: &Connection,
    records: &[SenatorRecord],
) -> Result<ImportResult, String> {
    let mut result = ImportResult::default();
    let now = chrono::Utc::now().timestamp();

    let deck_exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM decks WHERE id = ?1",
            [DECK_ID],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if deck_exists == 0 {
        conn.execute(
            "INSERT INTO decks (id, name, parent_id, description, new_per_day, max_reviews, created_at, updated_at) VALUES (?1, ?2, NULL, ?3, 20, 200, ?4, ?4)",
            (
                DECK_ID,
                DECK_NAME,
                "119th United States Senate — imported from senators.json",
                now,
            ),
        )
        .map_err(|e| e.to_string())?;
        result.decks_imported = 1;
    }

    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    for rec in records {
        let slug = senator_slug(&rec.name);
        if slug.is_empty() {
            result.warnings.push(format!("Skip empty slug for name {:?}", rec.name));
            continue;
        }
        let note_id = format!("n_sen_{slug}");
        let fields = serde_json::json!({
            "Name": rec.name,
            "State": rec.state,
            "Party": rec.party,
            "Since": rec.since,
            "Background": rec.background,
            "Trivia": rec.trivia,
            "PersonalFact": rec.personal_fact,
            "Photo": rec.photo,
        });
        let fields_str = fields.to_string();

        let inserted = tx
            .execute(
                "INSERT OR IGNORE INTO notes (id, deck_id, note_type_id, fields_json, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
                (&note_id, DECK_ID, NOTE_TYPE_ID, &fields_str, now),
            )
            .map_err(|e| e.to_string())?;
        if inserted == 0 {
            result.warnings.push(format!("Note already exists: {}", rec.name));
            continue;
        }
        result.notes_imported += 1;

        for ordinal in 0..2 {
            let card_id = format!("c_sen_{slug}_{ordinal}");
            tx.execute(
                "INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES (?1, ?2, ?3, 'new', ?4)",
                (&card_id, &note_id, ordinal, now),
            )
            .map_err(|e| e.to_string())?;
            result.cards_imported += 1;
        }

        upsert_note_fts_conn(&tx, &note_id, &fields_str)?;
    }

    tx.commit().map_err(|e| e.to_string())?;

    if records.len() != 100 {
        result.warnings.push(format!(
            "Expected 100 senators, got {}",
            records.len()
        ));
    }

    let without_photo = records.iter().filter(|r| r.photo.is_empty()).count();
    if without_photo > 0 {
        result.warnings.push(format!(
            "{without_photo} senators have no photo (pass senators_standalone.html to merge)"
        ));
    }

    Ok(result)
}

pub fn import_senators_paths(
    conn: &Connection,
    json_path: &Path,
    html_path: Option<&Path>,
) -> Result<ImportResult, String> {
    let json_rows = load_senators_json(json_path)?;
    let mut records: Vec<SenatorRecord> = json_rows.into_iter().map(json_to_record).collect();

    if let Some(html) = html_path {
        if html.exists() {
            let photos = photos_from_html(html)?;
            merge_photos(&mut records, photos);
        } else {
            return Err(format!("HTML not found: {}", html.display()));
        }
    }

    import_senators_conn(conn, &records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_party_codes() {
        assert_eq!(normalize_party("R"), "Republican");
        assert_eq!(normalize_party("D"), "Democrat");
        assert_eq!(normalize_party("I"), "Independent");
    }

    #[test]
    fn slug_names() {
        assert_eq!(senator_slug("Mark Kelly"), "mark_kelly");
        assert_eq!(senator_slug("Tommy Tuberville"), "tommy_tuberville");
    }
}

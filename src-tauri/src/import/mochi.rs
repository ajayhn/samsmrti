use super::ImportResult;
use rusqlite::Connection;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;

pub fn import_mochi(file_path: &str, db_conn: &Connection, media_dir: &Path) -> Result<ImportResult, String> {
    let file = fs::File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid zip: {}", e))?;

    let mut result = ImportResult::default();
    let now = chrono::Utc::now().timestamp();

    let mut data_content = String::new();
    let mut is_json = false;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();

        if name == "data.json" {
            entry.read_to_string(&mut data_content).map_err(|e| e.to_string())?;
            is_json = true;
        } else if name == "data.edn" {
            entry.read_to_string(&mut data_content).map_err(|e| e.to_string())?;
            is_json = false;
        } else if !name.starts_with("__MACOSX") && !name.ends_with('/') {
            fs::create_dir_all(media_dir).ok();
            let dest = media_dir.join(&name);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent).ok();
            }
            let mut out = fs::File::create(&dest).map_err(|e| e.to_string())?;
            std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
            result.media_imported += 1;
        }
    }

    if data_content.is_empty() {
        return Err("No data.json or data.edn found in .mochi file".to_string());
    }

    let data: serde_json::Value = if is_json {
        serde_json::from_str(&data_content).map_err(|e| format!("Invalid JSON: {}", e))?
    } else {
        parse_edn_to_json(&data_content)?
    };

    import_mochi_data(&data, db_conn, now, &mut result)?;

    Ok(result)
}

fn parse_edn_to_json(edn_str: &str) -> Result<serde_json::Value, String> {
    // Simple EDN-to-JSON conversion:
    // Replace EDN keywords (:key) with JSON strings, adjust syntax
    let mut result = String::with_capacity(edn_str.len());
    let chars: Vec<char> = edn_str.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            ':' if i == 0 || !chars[i-1].is_alphanumeric() => {
                // EDN keyword -> JSON string
                let start = i + 1;
                let mut end = start;
                while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '-' || chars[end] == '_' || chars[end] == '/' || chars[end] == '.' || chars[end] == '?') {
                    end += 1;
                }
                let keyword: String = chars[start..end].iter().collect();
                result.push('"');
                result.push_str(&keyword);
                result.push('"');
                i = end;
            }
            '{' => { result.push('{'); i += 1; }
            '}' => { result.push('}'); i += 1; }
            '[' => { result.push('['); i += 1; }
            ']' => { result.push(']'); i += 1; }
            '#' if i + 1 < chars.len() && chars[i + 1] == '{' => {
                // EDN set #{} -> JSON array []
                result.push('[');
                i += 2;
            }
            '"' => {
                // String literal
                result.push('"');
                i += 1;
                while i < chars.len() {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        result.push(chars[i]);
                        result.push(chars[i + 1]);
                        i += 2;
                    } else if chars[i] == '"' {
                        result.push('"');
                        i += 1;
                        break;
                    } else {
                        result.push(chars[i]);
                        i += 1;
                    }
                }
            }
            c if c.is_whitespace() || c == ',' => {
                // Add commas between JSON elements where needed
                let trimmed = result.trim_end();
                let last = trimmed.chars().last();
                let needs_comma = match last {
                    Some('}') | Some(']') | Some('"') => true,
                    Some(ch) => ch.is_alphanumeric(),
                    None => false,
                };
                // Look ahead for next non-whitespace
                let mut j = i + 1;
                while j < chars.len() && (chars[j].is_whitespace() || chars[j] == ',') {
                    j += 1;
                }
                if needs_comma && j < chars.len() && !matches!(chars[j], '}' | ']') {
                    result.push(',');
                }
                result.push(' ');
                i = j;
            }
            c => {
                result.push(c);
                i += 1;
            }
        }
    }

    serde_json::from_str(&result).map_err(|e| format!("Failed to parse converted EDN: {} (converted: {})", e, &result[..result.len().min(200)]))
}

fn import_mochi_data(
    data: &serde_json::Value,
    db_conn: &Connection,
    now: i64,
    result: &mut ImportResult,
) -> Result<(), String> {
    let decks = data.get("decks").and_then(|d| d.as_array());
    let all_decks = decks.cloned().unwrap_or_default();
    let mut deck_id_map: HashMap<String, String> = HashMap::new();

    for deck in &all_decks {
        let mochi_id = deck.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let name = deck.get("name").and_then(|v| v.as_str()).unwrap_or("Imported Deck");
        let parent_mochi_id = deck.get("parent-id").and_then(|v| v.as_str());

        let deck_id = if mochi_id.is_empty() {
            format!("dk_{}", uuid::Uuid::new_v4().simple())
        } else {
            format!("dk_mochi_{}", mochi_id)
        };

        let parent_id = parent_mochi_id.and_then(|pid| deck_id_map.get(pid)).cloned();

        db_conn
            .execute(
                "INSERT OR IGNORE INTO decks (id, name, parent_id, description, created_at, updated_at) VALUES (?1, ?2, ?3, '', ?4, ?5)",
                (&deck_id, name, &parent_id, now, now),
            )
            .map_err(|e| e.to_string())?;

        if !mochi_id.is_empty() {
            deck_id_map.insert(mochi_id.to_string(), deck_id.clone());
        }
        result.decks_imported += 1;

        let templates = deck.get("templates").and_then(|t| t.as_array());
        if let Some(templates) = templates {
            for template in templates {
                import_mochi_template(template, db_conn, now)?;
            }
        }

        let cards = deck.get("cards").and_then(|c| c.as_array());
        if let Some(cards) = cards {
            for card in cards {
                import_mochi_card(card, &deck_id, db_conn, now, result)?;
            }
        }
    }

    let top_level_cards = data.get("cards").and_then(|c| c.as_array());
    if let Some(cards) = top_level_cards {
        for card in cards {
            let card_deck_mochi_id = card.get("deck-id").and_then(|v| v.as_str()).unwrap_or("");
            let deck_id = deck_id_map.get(card_deck_mochi_id).cloned().unwrap_or_else(|| {
                deck_id_map.values().next().cloned().unwrap_or_default()
            });
            if !deck_id.is_empty() {
                import_mochi_card(card, &deck_id, db_conn, now, result)?;
            }
        }
    }

    Ok(())
}

fn import_mochi_template(
    template: &serde_json::Value,
    db_conn: &Connection,
    now: i64,
) -> Result<(), String> {
    let tmpl_id = template.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let name = template.get("name").and_then(|v| v.as_str()).unwrap_or("Card");
    let content = template.get("content").and_then(|v| v.as_str()).unwrap_or("");

    let nt_id = if tmpl_id.is_empty() {
        format!("nt_mochi_{}", uuid::Uuid::new_v4().simple())
    } else {
        format!("nt_mochi_{}", tmpl_id)
    };

    let exists: bool = db_conn
        .query_row("SELECT COUNT(*) FROM note_types WHERE id = ?1", [&nt_id], |row| {
            Ok(row.get::<_, i64>(0)? > 0)
        })
        .unwrap_or(false);

    if exists {
        return Ok(());
    }

    let is_cloze = content.contains("{{c") || name.to_lowercase().contains("cloze");

    db_conn
        .execute(
            "INSERT INTO note_types (id, name, css, is_cloze, created_at) VALUES (?1, ?2, '', ?3, ?4)",
            (&nt_id, name, is_cloze as i64, now),
        )
        .map_err(|e| e.to_string())?;

    let fields_data = template.get("fields");
    if let Some(fields_obj) = fields_data.and_then(|f| f.as_object()) {
        let mut field_list: Vec<(&String, &serde_json::Value)> = fields_obj.iter().collect();
        field_list.sort_by(|a, b| {
            let pos_a = a.1.get("pos").and_then(|v| v.as_str()).unwrap_or("z");
            let pos_b = b.1.get("pos").and_then(|v| v.as_str()).unwrap_or("z");
            pos_a.cmp(pos_b)
        });

        for (i, (_fid, field)) in field_list.iter().enumerate() {
            let fname = field.get("name").and_then(|v| v.as_str()).unwrap_or("Field");
            let field_id = format!("f_mochi_{}_{}", tmpl_id, i);
            db_conn
                .execute(
                    "INSERT INTO fields (id, note_type_id, name, ordinal) VALUES (?1, ?2, ?3, ?4)",
                    (&field_id, &nt_id, fname, i as i64),
                )
                .map_err(|e| e.to_string())?;
        }
    } else {
        let field_id = format!("f_mochi_{}_0", tmpl_id);
        db_conn
            .execute(
                "INSERT INTO fields (id, note_type_id, name, ordinal) VALUES (?1, ?2, 'Content', 0)",
                (&field_id, &nt_id),
            )
            .map_err(|e| e.to_string())?;
    }

    let ct_id = format!("ct_mochi_{}", tmpl_id);
    db_conn
        .execute(
            "INSERT INTO card_templates (id, note_type_id, name, front_html, back_html, ordinal) VALUES (?1, ?2, ?3, '{{Content}}', '{{Content}}', 0)",
            (&ct_id, &nt_id, name),
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn import_mochi_card(
    card: &serde_json::Value,
    deck_id: &str,
    db_conn: &Connection,
    now: i64,
    result: &mut ImportResult,
) -> Result<(), String> {
    let mochi_id = card.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let content = card.get("content").and_then(|v| v.as_str());
    let name = card.get("name").and_then(|v| v.as_str());

    let note_id = if mochi_id.is_empty() {
        format!("n_{}", uuid::Uuid::new_v4().simple())
    } else {
        format!("n_mochi_{}", mochi_id)
    };

    let nt_id = find_note_type_for_deck(deck_id, db_conn)
        .unwrap_or_else(|| "nt_basic".to_string());

    let mut fields_json = serde_json::Map::new();

    if let Some(fields_data) = card.get("fields").and_then(|f| f.as_object()) {
        for (key, val) in fields_data {
            let val_str = val.as_str().unwrap_or("").to_string();
            fields_json.insert(key.clone(), serde_json::Value::String(val_str));
        }
    } else if let Some(c) = content {
        fields_json.insert("Content".to_string(), serde_json::Value::String(c.to_string()));
    } else if let Some(n) = name {
        fields_json.insert("Front".to_string(), serde_json::Value::String(n.to_string()));
    }

    let fields_str = serde_json::to_string(&fields_json).unwrap_or_default();

    db_conn
        .execute(
            "INSERT OR IGNORE INTO notes (id, deck_id, note_type_id, fields_json, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (&note_id, deck_id, &nt_id, &fields_str, now, now),
        )
        .map_err(|e| e.to_string())?;
    result.notes_imported += 1;

    let card_id = format!("c_mochi_{}", if mochi_id.is_empty() {
        uuid::Uuid::new_v4().simple().to_string()
    } else {
        mochi_id.to_string()
    });

    db_conn
        .execute(
            "INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES (?1, ?2, 0, 'new', ?3)",
            (&card_id, &note_id, now),
        )
        .map_err(|e| e.to_string())?;
    result.cards_imported += 1;

    Ok(())
}

fn find_note_type_for_deck(deck_id: &str, db_conn: &Connection) -> Option<String> {
    db_conn
        .query_row(
            "SELECT note_type_id FROM notes WHERE deck_id = ?1 LIMIT 1",
            [deck_id],
            |row| row.get(0),
        )
        .ok()
}

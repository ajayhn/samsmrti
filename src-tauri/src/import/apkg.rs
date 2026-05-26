use super::anki_decks::{ensure_deck_path, split_anki_deck_path};
use super::ImportResult;
use rusqlite::Connection;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Import from a live Anki profile `collection.anki2` (close Anki first).
pub fn import_anki_collection(
    collection_path: &str,
    db_conn: &Connection,
    media_dir: &Path,
) -> Result<ImportResult, String> {
    let temp_dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    let dest = temp_dir.path().join("collection.anki2");
    fs::copy(collection_path, &dest).map_err(|e| format!("Failed to copy collection: {}", e))?;

    let anki_conn = Connection::open(&dest).map_err(|e| e.to_string())?;
    let mut result = ImportResult::default();
    result.warnings.push(
        "Imported from live Anki profile. Close Anki before importing. Media may be incomplete — use .apkg export for full media.".into(),
    );

    import_from_anki_conn(&anki_conn, db_conn, media_dir, None, &mut result)?;
    Ok(result)
}

pub fn import_apkg(file_path: &str, db_conn: &Connection, media_dir: &Path) -> Result<ImportResult, String> {
    let file = fs::File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid zip: {}", e))?;

    let mut result = ImportResult::default();
    let temp_dir = tempfile::tempdir().map_err(|e| e.to_string())?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();
        let out_path = temp_dir.path().join(&name);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        let mut out_file = fs::File::create(&out_path).map_err(|e| e.to_string())?;
        std::io::copy(&mut entry, &mut out_file).map_err(|e| e.to_string())?;
    }

    let anki_db_path = if temp_dir.path().join("collection.anki21").exists() {
        temp_dir.path().join("collection.anki21")
    } else if temp_dir.path().join("collection.anki2").exists() {
        temp_dir.path().join("collection.anki2")
    } else {
        return Err("No collection database found in .apkg file".to_string());
    };

    let anki_conn = Connection::open(&anki_db_path).map_err(|e| e.to_string())?;
    let media_map = load_media_map(temp_dir.path());
    import_from_anki_conn(&anki_conn, db_conn, media_dir, Some((temp_dir.path(), &media_map)), &mut result)?;
    Ok(result)
}

fn import_from_anki_conn(
    anki_conn: &Connection,
    db_conn: &Connection,
    media_dir: &Path,
    media_source: Option<(&Path, &HashMap<String, String>)>,
    result: &mut ImportResult,
) -> Result<(), String> {
    if let Some((temp_dir, media_map)) = media_source {
        import_media(media_map, temp_dir, media_dir, result);
    }

    let now = chrono::Utc::now().timestamp();
    let note_type_map = import_note_types(anki_conn, db_conn, now, result)?;
    let deck_map = import_decks(anki_conn, db_conn, now, result)?;
    let empty_media = HashMap::new();
    let media_map = media_source.map(|(_, m)| m).unwrap_or(&empty_media);
    import_notes_and_cards(
        anki_conn,
        db_conn,
        &note_type_map,
        &deck_map,
        media_map,
        media_dir,
        now,
        result,
    )?;
    Ok(())
}

fn load_media_map(temp_dir: &Path) -> HashMap<String, String> {
    let media_path = temp_dir.join("media");
    if !media_path.exists() {
        return HashMap::new();
    }
    match fs::read_to_string(&media_path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => HashMap::new(),
    }
}

fn import_media(
    media_map: &HashMap<String, String>,
    temp_dir: &Path,
    media_dir: &Path,
    result: &mut ImportResult,
) {
    fs::create_dir_all(media_dir).ok();
    for (index, filename) in media_map {
        let src = temp_dir.join(index);
        if src.exists() {
            let dest = media_dir.join(filename);
            if fs::copy(&src, &dest).is_ok() {
                result.media_imported += 1;
            }
        }
    }
}

fn import_note_types(
    anki_conn: &Connection,
    db_conn: &Connection,
    now: i64,
    result: &mut ImportResult,
) -> Result<HashMap<i64, String>, String> {
    let mut map = HashMap::new();

    let has_notetypes = anki_conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='notetypes'")
        .and_then(|mut s| s.query_row([], |_| Ok(true)))
        .unwrap_or(false);

    if has_notetypes {
        let mut stmt = anki_conn
            .prepare("SELECT id, name, config FROM notetypes")
            .map_err(|e| e.to_string())?;

        let rows: Vec<(i64, String, Vec<u8>)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get::<_, Vec<u8>>(2).unwrap_or_default())))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        for (anki_id, name, _config) in rows {
            let nt_id = format!("nt_imp_{}", anki_id);

            let exists: bool = db_conn
                .query_row("SELECT COUNT(*) FROM note_types WHERE id = ?1", [&nt_id], |row| {
                    Ok(row.get::<_, i64>(0)? > 0)
                })
                .unwrap_or(false);

            if !exists {
                let is_cloze = name.to_lowercase().contains("cloze");
                db_conn
                    .execute(
                        "INSERT INTO note_types (id, name, css, is_cloze, created_at) VALUES (?1, ?2, '', ?3, ?4)",
                        (&nt_id, &name, is_cloze as i64, now),
                    )
                    .map_err(|e| e.to_string())?;

                import_fields_for_notetype(anki_conn, db_conn, anki_id, &nt_id)?;
                import_templates_for_notetype(anki_conn, db_conn, anki_id, &nt_id, is_cloze)?;
            }

            map.insert(anki_id, nt_id);
        }
    } else {
        let mut stmt = anki_conn
            .prepare("SELECT models FROM col")
            .map_err(|e| e.to_string())?;

        let models_json: String = stmt
            .query_row([], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        let models: serde_json::Value = serde_json::from_str(&models_json).unwrap_or_default();

        if let Some(obj) = models.as_object() {
            for (anki_id_str, model) in obj {
                let anki_id: i64 = anki_id_str.parse().unwrap_or(0);
                let name = model["name"].as_str().unwrap_or("Imported");
                let nt_id = format!("nt_imp_{}", anki_id);

                let exists: bool = db_conn
                    .query_row("SELECT COUNT(*) FROM note_types WHERE id = ?1", [&nt_id], |row| {
                        Ok(row.get::<_, i64>(0)? > 0)
                    })
                    .unwrap_or(false);

                if !exists {
                    let is_cloze = model["type"].as_i64().unwrap_or(0) == 1;
                    db_conn
                        .execute(
                            "INSERT INTO note_types (id, name, css, is_cloze, created_at) VALUES (?1, ?2, '', ?3, ?4)",
                            (&nt_id, name, is_cloze as i64, now),
                        )
                        .map_err(|e| e.to_string())?;

                    if let Some(flds) = model["flds"].as_array() {
                        for (i, fld) in flds.iter().enumerate() {
                            let fname = fld["name"].as_str().unwrap_or("Field");
                            let fid = format!("f_imp_{}_{}", anki_id, i);
                            db_conn
                                .execute(
                                    "INSERT INTO fields (id, note_type_id, name, ordinal) VALUES (?1, ?2, ?3, ?4)",
                                    (&fid, &nt_id, fname, i as i64),
                                )
                                .map_err(|e| e.to_string())?;
                        }
                    }

                    if let Some(tmpls) = model["tmpls"].as_array() {
                        for (i, tmpl) in tmpls.iter().enumerate() {
                            let tname = tmpl["name"].as_str().unwrap_or("Card");
                            let front = tmpl["qfmt"].as_str().unwrap_or("{{Front}}");
                            let back = tmpl["afmt"].as_str().unwrap_or("{{Back}}");
                            let tid = format!("ct_imp_{}_{}", anki_id, i);
                            db_conn
                                .execute(
                                    "INSERT INTO card_templates (id, note_type_id, name, front_html, back_html, ordinal) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                                    (&tid, &nt_id, tname, front, back, i as i64),
                                )
                                .map_err(|e| e.to_string())?;
                        }
                    }
                }

                map.insert(anki_id, nt_id);
            }
        }
    }

    Ok(map)
}

fn import_fields_for_notetype(
    anki_conn: &Connection,
    db_conn: &Connection,
    anki_nt_id: i64,
    nt_id: &str,
) -> Result<(), String> {
    let has_fields_table = anki_conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='fields'")
        .and_then(|mut s| s.query_row([], |_| Ok(true)))
        .unwrap_or(false);

    if has_fields_table {
        let mut stmt = anki_conn
            .prepare("SELECT name, ord FROM fields WHERE ntid = ?1 ORDER BY ord")
            .map_err(|e| e.to_string())?;

        let fields: Vec<(String, i64)> = stmt
            .query_map([anki_nt_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        for (name, ord) in fields {
            let fid = format!("f_imp_{}_{}", anki_nt_id, ord);
            db_conn
                .execute(
                    "INSERT INTO fields (id, note_type_id, name, ordinal) VALUES (?1, ?2, ?3, ?4)",
                    (&fid, nt_id, &name, ord),
                )
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

fn import_templates_for_notetype(
    anki_conn: &Connection,
    db_conn: &Connection,
    anki_nt_id: i64,
    nt_id: &str,
    is_cloze: bool,
) -> Result<(), String> {
    let has_templates_table = anki_conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='templates'")
        .and_then(|mut s| s.query_row([], |_| Ok(true)))
        .unwrap_or(false);

    if has_templates_table {
        let mut stmt = anki_conn
            .prepare("SELECT name, ord, config FROM templates WHERE ntid = ?1 ORDER BY ord")
            .map_err(|e| e.to_string())?;

        let templates: Vec<(String, i64, Vec<u8>)> = stmt
            .query_map([anki_nt_id], |row| Ok((row.get(0)?, row.get(1)?, row.get::<_, Vec<u8>>(2).unwrap_or_default())))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        if templates.is_empty() {
            let tid = format!("ct_imp_{}_0", anki_nt_id);
            let (front, back) = if is_cloze {
                ("{{cloze:Text}}", "{{cloze:Text}}")
            } else {
                ("{{Front}}", "{{FrontSide}}<hr>{{Back}}")
            };
            db_conn
                .execute(
                    "INSERT INTO card_templates (id, note_type_id, name, front_html, back_html, ordinal) VALUES (?1, ?2, 'Card 1', ?3, ?4, 0)",
                    (&tid, nt_id, front, back),
                )
                .map_err(|e| e.to_string())?;
        } else {
            for (name, ord, _config) in templates {
                let tid = format!("ct_imp_{}_{}", anki_nt_id, ord);
                let (front, back) = if is_cloze {
                    ("{{cloze:Text}}".to_string(), "{{cloze:Text}}".to_string())
                } else {
                    ("{{Front}}".to_string(), "{{FrontSide}}<hr>{{Back}}".to_string())
                };
                db_conn
                    .execute(
                        "INSERT INTO card_templates (id, note_type_id, name, front_html, back_html, ordinal) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        (&tid, nt_id, &name, &front, &back, ord),
                    )
                    .map_err(|e| e.to_string())?;
            }
        }
    } else {
        let tid = format!("ct_imp_{}_0", anki_nt_id);
        let (front, back) = if is_cloze {
            ("{{cloze:Text}}", "{{cloze:Text}}")
        } else {
            ("{{Front}}", "{{FrontSide}}<hr>{{Back}}")
        };
        db_conn
            .execute(
                "INSERT INTO card_templates (id, note_type_id, name, front_html, back_html, ordinal) VALUES (?1, ?2, 'Card 1', ?3, ?4, 0)",
                (&tid, nt_id, front, back),
            )
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn import_decks(
    anki_conn: &Connection,
    db_conn: &Connection,
    now: i64,
    result: &mut ImportResult,
) -> Result<HashMap<i64, String>, String> {
    let mut map = HashMap::new();

    let has_decks_table = anki_conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='decks'")
        .and_then(|mut s| s.query_row([], |_| Ok(true)))
        .unwrap_or(false);

    if has_decks_table {
        let mut stmt = anki_conn
            .prepare("SELECT id, name FROM decks")
            .map_err(|e| e.to_string())?;

        let decks: Vec<(i64, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let mut path_cache: HashMap<String, String> = HashMap::new();
        for (anki_id, name) in decks {
            let segments: Vec<String> = split_anki_deck_path(&name);
            let deck_id = ensure_deck_path(db_conn, &segments, now, &mut path_cache)?;
            map.insert(anki_id, deck_id);
            result.decks_imported += 1;
        }
    } else {
        let decks_json: String = anki_conn
            .query_row("SELECT decks FROM col", [], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        let decks: serde_json::Value = serde_json::from_str(&decks_json).unwrap_or_default();
        let mut path_cache: HashMap<String, String> = HashMap::new();

        if let Some(obj) = decks.as_object() {
            for (anki_id_str, deck) in obj {
                let anki_id: i64 = anki_id_str.parse().unwrap_or(0);
                let name = deck["name"].as_str().unwrap_or("Imported Deck");
                let segments: Vec<String> = split_anki_deck_path(name);
                let deck_id = ensure_deck_path(db_conn, &segments, now, &mut path_cache)?;
                map.insert(anki_id, deck_id);
                result.decks_imported += 1;
            }
        }
    }

    Ok(map)
}

fn import_notes_and_cards(
    anki_conn: &Connection,
    db_conn: &Connection,
    note_type_map: &HashMap<i64, String>,
    deck_map: &HashMap<i64, String>,
    _media_map: &HashMap<String, String>,
    _media_dir: &Path,
    now: i64,
    result: &mut ImportResult,
) -> Result<(), String> {
    let mut note_stmt = anki_conn
        .prepare("SELECT id, mid, flds, tags FROM notes")
        .map_err(|e| e.to_string())?;

    let notes: Vec<(i64, i64, String, String)> = note_stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get::<_, String>(3).unwrap_or_default(),
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut card_stmt = anki_conn
        .prepare("SELECT id, nid, did, ord, type, queue FROM cards")
        .map_err(|e| e.to_string())?;

    let cards: Vec<(i64, i64, i64, i64, i64, i64)> = card_stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let cards_by_note: HashMap<i64, Vec<&(i64, i64, i64, i64, i64, i64)>> = {
        let mut m: HashMap<i64, Vec<&(i64, i64, i64, i64, i64, i64)>> = HashMap::new();
        for card in &cards {
            m.entry(card.1).or_default().push(card);
        }
        m
    };

    for (anki_note_id, mid, flds, tags_str) in &notes {
        let nt_id = match note_type_map.get(mid) {
            Some(id) => id,
            None => {
                result.warnings.push(format!("Skipped note {}: unknown note type {}", anki_note_id, mid));
                continue;
            }
        };

        let note_cards = cards_by_note.get(anki_note_id);
        let deck_id = note_cards
            .and_then(|c| c.first())
            .and_then(|c| deck_map.get(&c.2))
            .cloned()
            .unwrap_or_else(|| deck_map.values().next().cloned().unwrap_or_default());

        if deck_id.is_empty() {
            result.warnings.push(format!("Skipped note {}: no deck found", anki_note_id));
            continue;
        }

        let field_names: Vec<String> = db_conn
            .prepare("SELECT name FROM fields WHERE note_type_id = ?1 ORDER BY ordinal")
            .and_then(|mut s| {
                s.query_map([nt_id], |row| row.get(0))
                    .map(|rows| rows.collect::<Result<Vec<String>, _>>())
            })
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;

        let field_values: Vec<&str> = flds.split('\x1f').collect();
        let mut fields_json = serde_json::Map::new();
        for (i, name) in field_names.iter().enumerate() {
            let val = field_values.get(i).unwrap_or(&"");
            fields_json.insert(name.clone(), serde_json::Value::String(val.to_string()));
        }
        let fields_str = serde_json::to_string(&fields_json).unwrap_or_default();

        let note_id = format!("n_imp_{}", anki_note_id);
        db_conn
            .execute(
                "INSERT OR IGNORE INTO notes (id, deck_id, note_type_id, fields_json, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                (&note_id, &deck_id, nt_id, &fields_str, now, now),
            )
            .map_err(|e| e.to_string())?;
        result.notes_imported += 1;

        let tags: Vec<&str> = tags_str.split_whitespace().filter(|t| !t.is_empty()).collect();
        for tag_name in &tags {
            let tag_id = format!("t_imp_{}", uuid::Uuid::new_v4().simple());
            db_conn.execute("INSERT OR IGNORE INTO tags (id, name) VALUES (?1, ?2)", (&tag_id, tag_name)).ok();
            db_conn.execute(
                "INSERT OR IGNORE INTO note_tags (note_id, tag_id) SELECT ?1, id FROM tags WHERE name = ?2",
                (&note_id, tag_name),
            ).ok();
        }

        if let Some(note_cards) = note_cards {
            for card_tuple in note_cards {
                let card_id = format!("c_imp_{}", card_tuple.0);
                let state = match card_tuple.4 {
                    0 => "new",
                    1 => "learning",
                    2 => "review",
                    3 => "relearning",
                    _ => "new",
                };
                db_conn
                    .execute(
                        "INSERT OR IGNORE INTO cards (id, note_id, template_ordinal, state, due_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                        (&card_id, &note_id, card_tuple.3, state, now),
                    )
                    .map_err(|e| e.to_string())?;
                crate::db::card_progress::seed_progress_for_all_profiles(db_conn, &card_id, now)
                    .map_err(|e| e.to_string())?;
                result.cards_imported += 1;
            }
        }
    }

    Ok(())
}


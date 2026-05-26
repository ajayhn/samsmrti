use crate::db::Database;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

#[derive(Debug, Serialize, Clone)]
pub struct NoteTypeDeckRef {
    pub deck_id: String,
    pub deck_name: String,
    pub note_count: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct NoteTypeUsageSummary {
    pub note_type_id: String,
    pub note_count: i64,
    pub card_count: i64,
    pub deck_count: i64,
    pub top_decks: Vec<NoteTypeDeckRef>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNoteTypeInput {
    pub name: String,
    pub is_cloze: bool,
    pub css: Option<String>,
    pub fields: Vec<String>,
    pub templates: Vec<TemplateInput>,
}

#[derive(Debug, Deserialize)]
pub struct TemplateInput {
    pub name: String,
    pub front_html: String,
    pub back_html: String,
}

#[tauri::command]
pub fn create_note_type(
    db: State<Database>,
    input: CreateNoteTypeInput,
) -> Result<String, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp();
    let nt_id = format!("nt_{}", uuid::Uuid::new_v4().simple());

    conn.execute(
        "INSERT INTO note_types (id, name, css, is_cloze, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        (&nt_id, &input.name, input.css.as_deref().unwrap_or(""), input.is_cloze as i64, now),
    )
    .map_err(|e| e.to_string())?;

    for (i, field_name) in input.fields.iter().enumerate() {
        let field_id = format!("f_{}", uuid::Uuid::new_v4().simple());
        conn.execute(
            "INSERT INTO fields (id, note_type_id, name, ordinal) VALUES (?1, ?2, ?3, ?4)",
            (&field_id, &nt_id, field_name, i as i64),
        )
        .map_err(|e| e.to_string())?;
    }

    for (i, tmpl) in input.templates.iter().enumerate() {
        let tmpl_id = format!("ct_{}", uuid::Uuid::new_v4().simple());
        conn.execute(
            "INSERT INTO card_templates (id, note_type_id, name, front_html, back_html, ordinal) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (&tmpl_id, &nt_id, &tmpl.name, &tmpl.front_html, &tmpl.back_html, i as i64),
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(nt_id)
}

#[derive(Debug, Deserialize)]
pub struct UpdateNoteTypeInput {
    pub id: String,
    pub name: Option<String>,
    pub css: Option<String>,
    pub fields: Option<Vec<String>>,
    pub templates: Option<Vec<TemplateInput>>,
}

#[tauri::command]
pub fn update_note_type(
    db: State<Database>,
    input: UpdateNoteTypeInput,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    if let Some(ref name) = input.name {
        conn.execute(
            "UPDATE note_types SET name = ?1 WHERE id = ?2",
            (name, &input.id),
        )
        .map_err(|e| e.to_string())?;
    }

    if let Some(ref css) = input.css {
        conn.execute(
            "UPDATE note_types SET css = ?1 WHERE id = ?2",
            (css, &input.id),
        )
        .map_err(|e| e.to_string())?;
    }

    if let Some(ref fields) = input.fields {
        conn.execute("DELETE FROM fields WHERE note_type_id = ?1", [&input.id])
            .map_err(|e| e.to_string())?;
        for (i, field_name) in fields.iter().enumerate() {
            let field_id = format!("f_{}", uuid::Uuid::new_v4().simple());
            conn.execute(
                "INSERT INTO fields (id, note_type_id, name, ordinal) VALUES (?1, ?2, ?3, ?4)",
                (&field_id, &input.id, field_name, i as i64),
            )
            .map_err(|e| e.to_string())?;
        }
    }

    if let Some(ref templates) = input.templates {
        conn.execute(
            "DELETE FROM card_templates WHERE note_type_id = ?1",
            [&input.id],
        )
        .map_err(|e| e.to_string())?;
        for (i, tmpl) in templates.iter().enumerate() {
            let tmpl_id = format!("ct_{}", uuid::Uuid::new_v4().simple());
            conn.execute(
                "INSERT INTO card_templates (id, note_type_id, name, front_html, back_html, ordinal) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                (&tmpl_id, &input.id, &tmpl.name, &tmpl.front_html, &tmpl.back_html, i as i64),
            )
            .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
pub fn get_note_type_usage(db: State<Database>) -> Result<Vec<NoteTypeUsageSummary>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut agg: HashMap<String, (i64, i64, i64)> = HashMap::new();
    let mut agg_stmt = conn
        .prepare(
            "SELECT n.note_type_id,
                    COUNT(DISTINCT n.id),
                    COUNT(DISTINCT c.id),
                    COUNT(DISTINCT n.deck_id)
             FROM notes n
             LEFT JOIN cards c ON c.note_id = n.id
             GROUP BY n.note_type_id",
        )
        .map_err(|e| e.to_string())?;
    let agg_rows = agg_stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    for row in agg_rows {
        agg.insert(row.0, (row.1, row.2, row.3));
    }

    let mut decks_by_nt: HashMap<String, Vec<NoteTypeDeckRef>> = HashMap::new();
    let mut deck_stmt = conn
        .prepare(
            "SELECT n.note_type_id, n.deck_id, d.name, COUNT(DISTINCT n.id) as note_count
             FROM notes n
             JOIN decks d ON d.id = n.deck_id
             GROUP BY n.note_type_id, n.deck_id
             ORDER BY n.note_type_id, note_count DESC",
        )
        .map_err(|e| e.to_string())?;
    let deck_rows = deck_stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    for (nt_id, deck_id, deck_name, note_count) in deck_rows {
        let entry = decks_by_nt.entry(nt_id).or_default();
        if entry.len() < 5 {
            entry.push(NoteTypeDeckRef {
                deck_id,
                deck_name,
                note_count,
            });
        }
    }

    let all_nt_ids: Vec<String> = conn
        .prepare("SELECT id FROM note_types ORDER BY name")
        .map_err(|e| e.to_string())?
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut out = Vec::with_capacity(all_nt_ids.len());
    for nt_id in all_nt_ids {
        let (note_count, card_count, deck_count) =
            agg.get(&nt_id).copied().unwrap_or((0, 0, 0));
        out.push(NoteTypeUsageSummary {
            note_type_id: nt_id.clone(),
            note_count,
            card_count,
            deck_count,
            top_decks: decks_by_nt.remove(&nt_id).unwrap_or_default(),
        });
    }

    Ok(out)
}

#[tauri::command]
pub fn delete_note_type(db: State<Database>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM notes WHERE note_type_id = ?1",
            [&id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if count > 0 {
        return Err(format!(
            "Cannot delete note type: {} notes still use it",
            count
        ));
    }

    conn.execute("DELETE FROM note_types WHERE id = ?1", [&id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

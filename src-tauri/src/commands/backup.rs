use crate::backup::{
    build_content_payload, import_content_payload, list_decks_from_payload, list_export_decks,
    ContentDeckPreview,
};
use rusqlite::Connection;
use serde::Serialize;

// ---------------------------------------------------------------------------
// Core: content export/import (deck-scoped JSON, the manual-sync format --
// see Phase 3 of streamed-yawning-coral.md) is platform-agnostic. The full
// binary backup (media zip via tempfile) stays native-only -- see
// backup::full and commands::backup::native below.
// ---------------------------------------------------------------------------

pub fn list_content_export_decks_core(conn: &Connection) -> Result<Vec<ContentDeckPreview>, String> {
    list_export_decks(conn)
}

/// Result of building the content-export JSON payload in memory. Native
/// writes `content` straight to a file; wasm hands it back for the browser
/// to save (download blob / File System Access API).
#[derive(Debug, Serialize)]
pub struct ContentExportResult {
    pub content: String,
    pub decks: usize,
    pub notes: usize,
    pub cards: usize,
    pub entities: usize,
    pub triples: usize,
}

pub fn export_content_json_core(
    conn: &Connection,
    selected_deck_ids: Option<&[String]>,
) -> Result<ContentExportResult, String> {
    let payload = build_content_payload(conn, selected_deck_ids)?;
    let content = serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?;
    Ok(ContentExportResult {
        content,
        decks: payload["decks"].as_array().map(|a| a.len()).unwrap_or(0),
        notes: payload["notes"].as_array().map(|a| a.len()).unwrap_or(0),
        cards: payload["cards"].as_array().map(|a| a.len()).unwrap_or(0),
        entities: payload["entities"].as_array().map(|a| a.len()).unwrap_or(0),
        triples: payload["triples"].as_array().map(|a| a.len()).unwrap_or(0),
    })
}

pub fn preview_content_import_json_core(
    content_json: &str,
) -> Result<Vec<ContentDeckPreview>, String> {
    let data: serde_json::Value = serde_json::from_str(content_json).map_err(|e| e.to_string())?;
    list_decks_from_payload(&data)
}

pub fn import_content_json_core(
    conn: &Connection,
    content_json: &str,
    selected_deck_ids: Option<&[String]>,
) -> Result<crate::backup::ContentImportSummary, String> {
    let data: serde_json::Value = serde_json::from_str(content_json).map_err(|e| e.to_string())?;
    import_content_payload(conn, &data, selected_deck_ids)
}

// ---------------------------------------------------------------------------
// Native (Tauri) command wrappers.
// ---------------------------------------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::*;
    use crate::backup::{
        export_content_json_file, export_full_backup_file, import_content_file,
        preview_content_import_file, restore_full_backup_file, ContentExportSummary,
        ContentImportSummary, FullBackupExportSummary, FullBackupRestoreSummary,
    };
    use crate::commands::profiles::load_active_profile_from_db;
    use crate::commands::window_profiles::WindowProfiles;
    use crate::db::Database;
    use tauri::{AppHandle, Manager, State, WebviewWindow};

    #[tauri::command]
    pub fn list_content_export_decks(db: State<Database>) -> Result<Vec<ContentDeckPreview>, String> {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        list_content_export_decks_core(&conn)
    }

    #[tauri::command]
    pub fn preview_content_import(file_path: String) -> Result<Vec<ContentDeckPreview>, String> {
        preview_content_import_file(&file_path)
    }

    #[tauri::command]
    pub fn export_content_json(
        db: State<Database>,
        file_path: String,
        deck_ids: Option<Vec<String>>,
    ) -> Result<ContentExportSummary, String> {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let selected = deck_ids.as_deref();
        export_content_json_file(&conn, &file_path, selected)
    }

    #[tauri::command]
    pub fn import_content_json(
        db: State<Database>,
        file_path: String,
        deck_ids: Option<Vec<String>>,
    ) -> Result<ContentImportSummary, String> {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let selected = deck_ids.as_deref();
        import_content_file(&conn, &file_path, selected)
    }

    #[tauri::command]
    pub fn export_full_backup(
        db: State<Database>,
        app: AppHandle,
        file_path: String,
    ) -> Result<FullBackupExportSummary, String> {
        let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
        export_full_backup_file(&db, &app_data_dir, &file_path)
    }

    #[tauri::command]
    pub fn restore_full_backup(
        db: State<Database>,
        app: AppHandle,
        _window: WebviewWindow,
        profiles: State<'_, WindowProfiles>,
        file_path: String,
    ) -> Result<FullBackupRestoreSummary, String> {
        let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
        let summary = restore_full_backup_file(&db, &app_data_dir, &file_path)?;
        let profile = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            load_active_profile_from_db(&conn)?
        };
        profiles.set_all(profile);
        Ok(summary)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

// ---------------------------------------------------------------------------
// wasm-bindgen exports. No file paths on this target -- content moves as
// plain strings; the browser handles the actual file picker/download.
// Full binary backup (media zip) isn't ported -- see backup/full.rs and the
// Phase 0 spike notes on tempfile/zip's wasm32 compatibility.
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::*;
    use crate::db::wasm_singleton::with_db;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(js_name = listContentExportDecks)]
    pub fn list_content_export_decks() -> Result<String, JsValue> {
        with_db(|db| {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            let decks = list_content_export_decks_core(&conn)?;
            serde_json::to_string(&decks).map_err(|e| e.to_string())
        })
        .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen(js_name = previewContentImport)]
    pub fn preview_content_import(content_json: String) -> Result<String, JsValue> {
        let decks = preview_content_import_json_core(&content_json).map_err(|e| JsValue::from_str(&e))?;
        serde_json::to_string(&decks)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen(js_name = exportContentJson)]
    pub fn export_content_json(deck_ids_json: Option<String>) -> Result<String, JsValue> {
        let selected: Option<Vec<String>> = deck_ids_json
            .map(|s| serde_json::from_str(&s))
            .transpose()
            .map_err(|e: serde_json::Error| JsValue::from_str(&e.to_string()))?;
        with_db(|db| {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            let result = export_content_json_core(&conn, selected.as_deref())?;
            serde_json::to_string(&result).map_err(|e| e.to_string())
        })
        .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen(js_name = importContentJson)]
    pub fn import_content_json(
        content_json: String,
        deck_ids_json: Option<String>,
    ) -> Result<String, JsValue> {
        let selected: Option<Vec<String>> = deck_ids_json
            .map(|s| serde_json::from_str(&s))
            .transpose()
            .map_err(|e: serde_json::Error| JsValue::from_str(&e.to_string()))?;
        with_db(|db| {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            let summary = import_content_json_core(&conn, &content_json, selected.as_deref())?;
            serde_json::to_string(&summary).map_err(|e| e.to_string())
        })
        .map_err(|e| JsValue::from_str(&e))
    }
}

#[cfg(target_arch = "wasm32")]
#[allow(unused_imports)] // #[wasm_bindgen] fns here are called from JS, not Rust
pub use wasm::*;

use crate::backup::{
    export_content_json_file, export_full_backup_file, import_content_file, list_export_decks,
    preview_content_import_file, restore_full_backup_file, ContentDeckPreview, ContentExportSummary,
    ContentImportSummary, FullBackupExportSummary, FullBackupRestoreSummary,
};
use crate::commands::profiles::load_active_profile_from_db;
use crate::commands::window_profiles::WindowProfiles;
use crate::db::Database;
use tauri::{AppHandle, Manager, State, WebviewWindow};

#[tauri::command]
pub fn list_content_export_decks(db: State<Database>) -> Result<Vec<ContentDeckPreview>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    list_export_decks(&conn)
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
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
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
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let summary = restore_full_backup_file(&db, &app_data_dir, &file_path)?;
    let profile = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        load_active_profile_from_db(&conn)?
    };
    profiles.set_all(profile);
    Ok(summary)
}

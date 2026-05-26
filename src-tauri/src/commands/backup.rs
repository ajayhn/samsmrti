use crate::backup::{
    export_content_json_file, export_full_backup_file, import_content_file,
    restore_full_backup_file, ContentExportSummary, ContentImportSummary, FullBackupExportSummary,
    FullBackupRestoreSummary,
};
use crate::commands::profiles::{load_active_profile_from_db, ActiveProfile};
use crate::db::Database;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub fn export_content_json(
    db: State<Database>,
    file_path: String,
) -> Result<ContentExportSummary, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    export_content_json_file(&conn, &file_path)
}

#[tauri::command]
pub fn import_content_json(
    db: State<Database>,
    file_path: String,
) -> Result<ContentImportSummary, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    import_content_file(&conn, &file_path)
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
    active: State<'_, Mutex<ActiveProfile>>,
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
    let mut active_guard = active.lock().map_err(|e| e.to_string())?;
    *active_guard = profile;
    Ok(summary)
}

use crate::commands::search::rebuild_search_index_conn;
use crate::db::Database;
use crate::import::{apkg, mochi, ImportResult};
use tauri::{Manager, State};

#[tauri::command]
pub fn import_file(
    db: State<Database>,
    app: tauri::AppHandle,
    file_path: String,
) -> Result<ImportResult, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let media_dir = app
        .path()
        .app_data_dir()
        .map(|p| p.join("media"))
        .map_err(|e| e.to_string())?;

    let path_lower = file_path.to_lowercase();

    let result = if path_lower.ends_with(".apkg") {
        apkg::import_apkg(&file_path, &conn, &media_dir)
    } else if path_lower.ends_with(".anki2") || path_lower.ends_with(".anki21") {
        apkg::import_anki_collection(&file_path, &conn, &media_dir)
    } else if path_lower.ends_with(".mochi") {
        mochi::import_mochi(&file_path, &conn, &media_dir)
    } else {
        return Err(
            "Unsupported file format. Use .apkg, .anki2 (Anki collection), or .mochi.".to_string(),
        );
    }?;

    rebuild_search_index_conn(&conn)?;
    Ok(result)
}

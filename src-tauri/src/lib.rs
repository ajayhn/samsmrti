mod backup;
mod commands;
pub mod db;
pub mod import;
pub mod seed;

use commands::profiles::load_active_profile_from_db;
use db::Database;
use import::apkg;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{Emitter, Manager};

/// Import quizbowl packet PNGs as cloze notes (deck `ssnct`).
pub fn import_quizbowl_file(
    app_data_dir: &Path,
    packet_dir: &Path,
) -> Result<import::quizbowl::ImportReport, String> {
    let db = Database::new(&app_data_dir.to_path_buf()).map_err(|e| e.to_string())?;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    import::quizbowl::import_quizbowl_dir(&conn, packet_dir)
}

/// Append quizbowl packet PNGs to deck `ssnct` without clearing existing notes.
pub fn import_quizbowl_file_append(
    app_data_dir: &Path,
    packet_dir: &Path,
) -> Result<import::quizbowl::ImportReport, String> {
    let db = Database::new(&app_data_dir.to_path_buf()).map_err(|e| e.to_string())?;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    import::quizbowl::import_quizbowl_dir_with_options(&conn, packet_dir, false)
}

/// Import a single quizbowl PNG (append to deck `ssnct`, no wipe).
pub fn import_quizbowl_png_file(
    app_data_dir: &Path,
    png_path: &Path,
) -> Result<import::quizbowl::ImportReport, String> {
    let db = Database::new(&app_data_dir.to_path_buf()).map_err(|e| e.to_string())?;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    import::quizbowl::import_quizbowl_png(&conn, png_path)
}

/// Import US senators from JSON (+ optional HTML for photos).
pub fn import_senators_file(
    app_data_dir: &Path,
    json_path: &Path,
    html_path: Option<&Path>,
) -> Result<import::ImportResult, String> {
    let db = Database::new(&app_data_dir.to_path_buf()).map_err(|e| e.to_string())?;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    import::senators::import_senators_paths(&conn, json_path, html_path)
}

/// Import an Anki collection file into the app database (for CLI / automation).
pub fn import_anki_collection_file(
    app_data_dir: &Path,
    collection_path: &Path,
) -> Result<import::ImportResult, String> {
    fs::create_dir_all(app_data_dir.join("media")).map_err(|e| e.to_string())?;
    let db = Database::new(&app_data_dir.to_path_buf()).map_err(|e| e.to_string())?;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let media_dir = app_data_dir.join("media");
    apkg::import_anki_collection(
        collection_path.to_str().ok_or("Invalid collection path")?,
        &conn,
        &media_dir,
    )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");

            let database =
                Database::new(&app_data_dir).expect("Failed to initialize database");

            database.seed_example_decks().expect("Failed to seed example decks");

            let active_profile = {
                let conn = database.conn.lock().expect("db lock");
                load_active_profile_from_db(&conn).expect("Failed to load active profile")
            };
            app.manage(database);
            app.manage(Mutex::new(active_profile));

            let export_content = MenuItem::with_id(
                app,
                "export_content",
                "Export Content…",
                true,
                None::<&str>,
            )?;
            let import_content = MenuItem::with_id(
                app,
                "import_content",
                "Import Content…",
                true,
                None::<&str>,
            )?;
            let backup_sep = PredefinedMenuItem::separator(app)?;
            let export_full = MenuItem::with_id(
                app,
                "export_full_backup",
                "Backup (Full)…",
                true,
                None::<&str>,
            )?;
            let restore_full = MenuItem::with_id(
                app,
                "restore_full_backup",
                "Restore (Full)…",
                true,
                None::<&str>,
            )?;
            let file_menu = Submenu::with_items(
                app,
                "File",
                true,
                &[
                    &export_content,
                    &import_content,
                    &backup_sep,
                    &export_full,
                    &restore_full,
                ],
            )?;

            let edit_menu = Submenu::with_items(
                app,
                "Edit",
                true,
                &[
                    &PredefinedMenuItem::undo(app, None)?,
                    &PredefinedMenuItem::redo(app, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::cut(app, None)?,
                    &PredefinedMenuItem::copy(app, None)?,
                    &PredefinedMenuItem::paste(app, None)?,
                    &PredefinedMenuItem::select_all(app, None)?,
                ],
            )?;

            let app_name = app.package_info().name.clone();
            let quit_label = format!("Quit {app_name}");
            let quit_item = PredefinedMenuItem::quit(app, Some(quit_label.as_str()))?;
            let app_menu = Submenu::with_items(app, &app_name, true, &[&quit_item])?;

            let menu = Menu::with_items(app, &[&app_menu, &file_menu, &edit_menu])?;
            app.set_menu(menu)?;

            let handle = app.handle().clone();
            app.on_menu_event(move |_app, event| {
                if event.id() == export_content.id() {
                    let _ = handle.emit("menu-export-content", ());
                } else if event.id() == import_content.id() {
                    let _ = handle.emit("menu-import-content", ());
                } else if event.id() == export_full.id() {
                    let _ = handle.emit("menu-export-full-backup", ());
                } else if event.id() == restore_full.id() {
                    let _ = handle.emit("menu-restore-full-backup", ());
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::decks::get_decks,
            commands::decks::create_deck,
            commands::decks::update_deck,
            commands::decks::delete_deck,
            commands::notes::get_note_types,
            commands::notes::create_note,
            commands::notes::get_notes,
            commands::notes::get_note_tags,
            commands::notes::update_note,
            commands::notes::delete_note,
            commands::notes::get_cards_for_note,
            commands::review::get_review_queue,
            commands::review::answer_card,
            commands::review::undo_review,
            commands::review::get_interval_preview,
            commands::review::get_review_stats,
            commands::review::bury_card,
            commands::review::unbury_card,
            commands::review::get_buried_cards,
            commands::review::delete_card,
            commands::review::restore_card,
            commands::note_types::create_note_type,
            commands::note_types::update_note_type,
            commands::note_types::delete_note_type,
            commands::note_types::get_note_type_usage,
            commands::import::import_file,
            commands::search::search_notes,
            commands::search::rebuild_search_index,
            commands::search::ensure_search_index,
            commands::search::get_all_tags,
            commands::search::get_stats_overview,
            commands::export::export_deck_json,
            commands::export::export_all_gz,
            commands::backup::export_content_json,
            commands::backup::import_content_json,
            commands::backup::export_full_backup,
            commands::backup::restore_full_backup,
            commands::graph::create_entity,
            commands::graph::get_entities,
            commands::graph::update_entity,
            commands::graph::delete_entity,
            commands::graph::create_relation_type,
            commands::graph::get_relation_types,
            commands::graph::delete_relation_type,
            commands::graph::create_triple,
            commands::graph::batch_create_triples,
            commands::graph::get_triples,
            commands::graph::update_triple,
            commands::graph::delete_triple,
            commands::graph::link_card_to_triple,
            commands::graph::unlink_card_from_triple,
            commands::graph::get_triples_for_card,
            commands::graph::get_cards_for_triple,
            commands::graph::get_mindmap,
            commands::graph::suggest_cards_from_triples,
            commands::graph::get_ere_due_cards,
            commands::graph::get_ere_review_summary,
            commands::profiles::list_profiles,
            commands::profiles::get_active_profile,
            commands::profiles::set_active_profile,
            commands::profiles::create_profile,
            commands::profiles::delete_profile,
            commands::karma::get_karma_overview,
            commands::karma::record_activity,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

mod backup;
mod commands;
pub mod db;
pub mod import;
pub mod seed;

use commands::profiles::load_active_profile_from_db;
use commands::window_profiles::{compute_window_titles, open_new_window, sync_window_titles_and_menu, WindowProfiles};
use db::Database;
use import::apkg;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{Emitter, Manager};

const MENU_NEW_WINDOW: &str = "new_window";
const MENU_EXPORT_CONTENT: &str = "export_content";
const MENU_IMPORT_CONTENT: &str = "import_content";
const MENU_EXPORT_FULL: &str = "export_full_backup";
const MENU_RESTORE_FULL: &str = "restore_full_backup";
const MENU_UNDO: &str = "undo";
const MENU_FOCUS_PREFIX: &str = "focus_window::";

fn build_menu(app: &tauri::AppHandle) -> Result<Menu<tauri::Wry>, tauri::Error> {
    let new_window = MenuItem::with_id(app, MENU_NEW_WINDOW, "New Window", true, None::<&str>)?;
    let export_content = MenuItem::with_id(
        app,
        MENU_EXPORT_CONTENT,
        "Export Content…",
        true,
        None::<&str>,
    )?;
    let import_content = MenuItem::with_id(
        app,
        MENU_IMPORT_CONTENT,
        "Import Content…",
        true,
        None::<&str>,
    )?;
    let export_full = MenuItem::with_id(
        app,
        MENU_EXPORT_FULL,
        "Backup (Full)…",
        true,
        None::<&str>,
    )?;
    let restore_full = MenuItem::with_id(
        app,
        MENU_RESTORE_FULL,
        "Restore (Full)…",
        true,
        None::<&str>,
    )?;
    let file_menu = Submenu::with_items(
        app,
        "File",
        true,
        &[
            &new_window,
            &PredefinedMenuItem::separator(app)?,
            &export_content,
            &import_content,
            &PredefinedMenuItem::separator(app)?,
            &export_full,
            &restore_full,
        ],
    )?;

    let undo_item = MenuItem::with_id(app, MENU_UNDO, "Undo", true, Some("CmdOrCtrl+Z"))?;
    let edit_menu = Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &undo_item,
            &PredefinedMenuItem::redo(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )?;

    let mut window_items: Vec<Box<dyn tauri::menu::IsMenuItem<tauri::Wry>>> = vec![
        Box::new(PredefinedMenuItem::minimize(app, None)?),
        Box::new(PredefinedMenuItem::maximize(app, None)?),
        Box::new(PredefinedMenuItem::close_window(app, None)?),
        Box::new(PredefinedMenuItem::separator(app)?),
        Box::new(PredefinedMenuItem::bring_all_to_front(app, None)?),
    ];
    let mut windows: Vec<_> = app.webview_windows().into_iter().collect();
    windows.sort_by(|a, b| a.0.cmp(&b.0));
    let window_titles = app
        .try_state::<Database>()
        .zip(app.try_state::<WindowProfiles>())
        .map(|(db, profiles)| compute_window_titles(app, &db, &profiles))
        .unwrap_or_default();
    if !windows.is_empty() {
        window_items.push(Box::new(PredefinedMenuItem::separator(app)?));
        for (label, window) in windows {
            let title = window_titles
                .get(&label)
                .cloned()
                .unwrap_or_else(|| window.title().unwrap_or_else(|_| label.clone()));
            let item = MenuItem::with_id(
                app,
                format!("{MENU_FOCUS_PREFIX}{label}"),
                title,
                true,
                None::<&str>,
            )?;
            window_items.push(Box::new(item));
        }
    }
    let window_item_refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> =
        window_items.iter().map(|i| i.as_ref()).collect();
    let window_menu = Submenu::with_items(app, "Window", true, &window_item_refs)?;

    let app_name = app.package_info().name.clone();
    let quit_label = format!("Quit {app_name}");
    let quit_item = PredefinedMenuItem::quit(app, Some(quit_label.as_str()))?;
    let app_menu = Submenu::with_items(app, &app_name, true, &[&quit_item])?;

    Menu::with_items(app, &[&app_menu, &file_menu, &edit_menu, &window_menu])
}

fn refresh_menu(app: &tauri::AppHandle) {
    if let Ok(menu) = build_menu(app) {
        let _ = app.set_menu(menu);
    }
}

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

            let window_profiles = WindowProfiles(Mutex::new(HashMap::new()));
            window_profiles.register("main", active_profile);
            app.manage(window_profiles);

            {
                sync_window_titles_and_menu(&app.handle());
            }

            let handle = app.handle().clone();
            app.on_menu_event(move |app_handle, event| {
                let event_id = event.id().0.as_str();
                if event_id == MENU_NEW_WINDOW {
                    let db = app_handle.state::<Database>();
                    let profiles = app_handle.state::<WindowProfiles>();
                    if let Err(e) = open_new_window(app_handle, &db, &profiles) {
                        eprintln!("Failed to open new window: {e}");
                    }
                } else if let Some(label) = event_id.strip_prefix(MENU_FOCUS_PREFIX) {
                    if let Some(window) = app_handle.get_webview_window(label) {
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                } else if event_id == MENU_UNDO {
                    let _ = handle.emit("menu-undo", ());
                } else if event_id == MENU_EXPORT_CONTENT {
                    let _ = handle.emit("menu-export-content", ());
                } else if event_id == MENU_IMPORT_CONTENT {
                    let _ = handle.emit("menu-import-content", ());
                } else if event_id == MENU_EXPORT_FULL {
                    let _ = handle.emit("menu-export-full-backup", ());
                } else if event_id == MENU_RESTORE_FULL {
                    let _ = handle.emit("menu-restore-full-backup", ());
                }
            });

            // Ensure dock/window icon in dev (embedded at compile time via include_image).
            let icon = tauri::include_image!("icons/128x128.png");
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_icon(icon);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::decks::get_decks,
            commands::decks::create_deck,
            commands::decks::update_deck,
            commands::decks::delete_deck,
            commands::decks::restore_deleted_deck,
            commands::notes::get_note_types,
            commands::notes::create_note,
            commands::notes::get_deck_primary_note_type,
            commands::notes::get_notes,
            commands::notes::get_note_tags,
            commands::notes::get_card_flag,
            commands::notes::set_card_flag,
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
            commands::backup::list_content_export_decks,
            commands::backup::preview_content_import,
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

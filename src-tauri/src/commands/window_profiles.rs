use super::profiles::{load_active_profile_from_db, profile_display_name, ActiveProfile};
use crate::db::Database;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

/// Profile display names per window label. Duplicate names get "(2)", "(3)", …
pub fn compute_window_titles(
    app: &AppHandle,
    db: &Database,
    profiles: &WindowProfiles,
) -> HashMap<String, String> {
    let Ok(conn) = db.conn.lock() else {
        return HashMap::new();
    };

    let mut entries: Vec<(String, String)> = Vec::new();
    let mut windows: Vec<_> = app.webview_windows().into_iter().collect();
    windows.sort_by(|a, b| a.0.cmp(&b.0));

    for (label, _) in windows {
        let Ok(active) = profiles.for_label(&label) else {
            continue;
        };
        let Ok(name) = profile_display_name(&conn, &active.id) else {
            continue;
        };
        entries.push((label, name));
    }
    drop(conn);

    let mut name_counts: HashMap<String, usize> = HashMap::new();
    for (_, name) in &entries {
        *name_counts.entry(name.clone()).or_insert(0) += 1;
    }

    let mut name_index: HashMap<String, usize> = HashMap::new();
    let mut titles = HashMap::new();
    for (label, name) in entries {
        let title = if name_counts.get(&name).copied().unwrap_or(0) <= 1 {
            name.clone()
        } else {
            let idx = name_index.entry(name.clone()).or_insert(0);
            *idx += 1;
            if *idx == 1 {
                name.clone()
            } else {
                format!("{name} ({idx})")
            }
        };
        titles.insert(label, title);
    }
    titles
}

/// Set each window's native title from its active profile.
pub fn apply_window_titles(
    app: &AppHandle,
    db: &Database,
    profiles: &WindowProfiles,
) -> Result<(), String> {
    for (label, title) in compute_window_titles(app, db, profiles) {
        if let Some(window) = app.get_webview_window(&label) {
            window.set_title(&title).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub fn sync_window_titles_and_menu(app: &AppHandle) {
    let db = app.state::<Database>();
    let profiles = app.state::<WindowProfiles>();
    let _ = apply_window_titles(app, &db, &profiles);
    crate::refresh_menu(app);
}

/// Like [`sync_window_titles_and_menu`] but safe to call from IPC worker threads.
pub fn sync_window_titles_and_menu_from_command(app: &AppHandle) {
    let app = app.clone();
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let app_for_main = app.clone();
    if app
        .run_on_main_thread(move || {
            sync_window_titles_and_menu(&app_for_main);
            let _ = tx.send(());
        })
        .is_ok()
    {
        let _ = rx.recv_timeout(std::time::Duration::from_secs(2));
    } else {
        sync_window_titles_and_menu(&app);
    }
}

pub struct WindowProfiles(pub Mutex<HashMap<String, ActiveProfile>>);

impl WindowProfiles {
    pub fn register(&self, label: &str, profile: ActiveProfile) {
        if let Ok(mut map) = self.0.lock() {
            map.insert(label.to_string(), profile);
        }
    }

    pub fn for_window(&self, window: &WebviewWindow) -> Result<ActiveProfile, String> {
        self.for_label(window.label())
    }

    pub fn for_label(&self, label: &str) -> Result<ActiveProfile, String> {
        self.0
            .lock()
            .map_err(|e| e.to_string())?
            .get(label)
            .cloned()
            .ok_or_else(|| format!("No profile session for window \"{label}\""))
    }

    pub fn set_for_window(&self, window: &WebviewWindow, profile: ActiveProfile) {
        self.set_for_label(window.label(), profile);
    }

    pub fn set_for_label(&self, label: &str, profile: ActiveProfile) {
        if let Ok(mut map) = self.0.lock() {
            map.insert(label.to_string(), profile);
        }
    }

    pub fn set_all(&self, profile: ActiveProfile) {
        if let Ok(mut map) = self.0.lock() {
            for entry in map.values_mut() {
                *entry = profile.clone();
            }
        }
    }

    pub fn replace_profile_id(&self, old_id: &str, new_profile: ActiveProfile) {
        if let Ok(mut map) = self.0.lock() {
            for entry in map.values_mut() {
                if entry.id == old_id {
                    *entry = new_profile.clone();
                }
            }
        }
    }

    pub fn unregister(&self, label: &str) {
        if let Ok(mut map) = self.0.lock() {
            map.remove(label);
        }
    }
}

fn webview_url(app: &AppHandle) -> Result<WebviewUrl, String> {
    if cfg!(debug_assertions) {
        let dev_url = app
            .config()
            .build
            .dev_url
            .clone()
            .ok_or_else(|| "devUrl is not configured".to_string())?;
        Ok(WebviewUrl::External(dev_url))
    } else {
        Ok(WebviewUrl::App("index.html".into()))
    }
}

pub fn open_new_window(
    app: &AppHandle,
    db: &State<Database>,
    profiles: &State<WindowProfiles>,
) -> Result<(), String> {
    let label = format!("samsmrti-{}", uuid::Uuid::new_v4().simple());
    let initial = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        load_active_profile_from_db(&conn)?
    };

    let _window = WebviewWindowBuilder::new(app, &label, webview_url(app)?)
        .title("Samsmrti")
        .inner_size(1200.0, 800.0)
        .min_inner_size(900.0, 600.0)
        .resizable(true)
        .build()
        .map_err(|e| e.to_string())?;

    profiles.register(&label, initial);
    sync_window_titles_and_menu(app);

    Ok(())
}

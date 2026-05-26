//! Import US senators from senators.json into Samsmrti.
//! Usage: cargo run --bin import_senators -- [senators.json] [app-data-dir] [senators.html]

use std::path::PathBuf;
use std::process;

fn main() {
    let mut args = std::env::args().skip(1);
    let json_path = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data/senators.json"));

    let app_data = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            dirs::home_dir()
                .expect("home dir")
                .join("Library/Application Support/com.samsmrti.desktop")
        });

    let html_path = args.next().map(PathBuf::from).or_else(|| {
        let default = PathBuf::from("data/senators_standalone.html");
        if default.exists() {
            Some(default)
        } else {
            None
        }
    });

    if !json_path.exists() {
        eprintln!("JSON not found: {}", json_path.display());
        process::exit(1);
    }

    eprintln!("Importing senators from {} into {} …", json_path.display(), app_data.display());
    if let Some(ref html) = html_path {
        eprintln!("Merging photos from {} …", html.display());
    } else {
        eprintln!("No HTML file — photos will be empty.");
    }
    eprintln!("(Quit Samsmrti before importing.)");

    match samsmrti_lib::import_senators_file(&app_data, &json_path, html_path.as_deref()) {
        Ok(r) => {
            eprintln!(
                "Done: {} decks, {} notes, {} cards.",
                r.decks_imported, r.notes_imported, r.cards_imported
            );
            for w in &r.warnings {
                eprintln!("  warning: {w}");
            }
        }
        Err(e) => {
            eprintln!("Import failed: {e}");
            process::exit(1);
        }
    }
}

//! One-off CLI: import an Anki collection.anki2 into Samsmrti.
//! Usage: cargo run --bin import_anki -- <path-to-collection.anki2> [app-data-dir]

use std::path::PathBuf;
use std::process;

fn main() {
    let collection = match std::env::args().nth(1) {
        Some(p) => PathBuf::from(p),
        None => {
            eprintln!(
                "Usage: import_anki <collection.anki2> [app-data-dir]\n\
                 Default app-data-dir: ~/Library/Application Support/com.samsmrti.desktop"
            );
            process::exit(1);
        }
    };

    let app_data = std::env::args().nth(2).map(PathBuf::from).unwrap_or_else(|| {
        dirs::home_dir()
            .expect("home dir")
            .join("Library/Application Support/com.samsmrti.desktop")
    });

    if !collection.exists() {
        eprintln!("Collection not found: {}", collection.display());
        process::exit(1);
    }

    eprintln!("Importing {} into {} …", collection.display(), app_data.display());
    eprintln!("(Quit Anki and Samsmrti before importing.)");

    match samsmrti_lib::import_anki_collection_file(&app_data, &collection) {
        Ok(r) => {
            eprintln!(
                "Done: {} decks, {} notes, {} cards, {} media files.",
                r.decks_imported, r.notes_imported, r.cards_imported, r.media_imported
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

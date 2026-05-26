//! Import SSNCT quizbowl packet screenshots as cloze notes.
//! Usage:
//!   cargo run --bin import_quizbowl -- [packet_dir] [app-data-dir]
//!   cargo run --bin import_quizbowl -- --file path/to/page.png [app-data-dir]

use std::path::PathBuf;
use std::process;

fn default_app_data() -> PathBuf {
    dirs::home_dir()
        .expect("home dir")
        .join("Library/Application Support/com.samsmrti.desktop")
}

fn print_report(
    report: &samsmrti_lib::import::quizbowl::ImportReport,
    report_hint: &str,
) {
    let r = &report.summary;
    eprintln!(
        "Done: {} decks, {} notes, {} cards.",
        r.decks_imported, r.notes_imported, r.cards_imported
    );
    let tossups = report
        .entries
        .iter()
        .filter(|e| e.kind == samsmrti_lib::import::quizbowl::QuestionKind::Tossup)
        .count();
    let bonuses = report
        .entries
        .iter()
        .filter(|e| e.kind == samsmrti_lib::import::quizbowl::QuestionKind::Bonus)
        .count();
    eprintln!("  {tossups} tossups, {bonuses} bonuses in report.");
    for entry in &report.entries {
        eprintln!(
            "  - {} ({:?}, {} clozes)",
            entry.file, entry.kind, entry.cloze_count
        );
    }
    for w in &r.warnings {
        eprintln!("  warning: {w}");
    }
    eprintln!("Report: {report_hint}");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let append_mode = args.iter().any(|a| a == "--append");
    let file_idx = args.iter().position(|a| a == "--file");

    eprintln!("(Quit Samsmrti before importing. Requires `tesseract` on PATH.)");

    if let Some(idx) = file_idx {
        let png_path = args
            .get(idx + 1)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("data/multipage.png"));
        let app_data = args
            .get(idx + 2)
            .map(PathBuf::from)
            .unwrap_or_else(default_app_data);

        if !png_path.exists() {
            eprintln!("PNG not found: {}", png_path.display());
            process::exit(1);
        }

        eprintln!(
            "Importing {} into {} (append mode) …",
            png_path.display(),
            app_data.display()
        );

        match samsmrti_lib::import_quizbowl_png_file(&app_data, &png_path) {
            Ok(report) => print_report(&report, "stdout"),
            Err(e) => {
                eprintln!("Import failed: {e}");
                process::exit(1);
            }
        }
        return;
    }

    let positional: Vec<&String> = args
        .iter()
        .skip(1)
        .filter(|a| *a != "--append" && *a != "--file")
        .collect();

    let packet_dir = positional
        .first()
        .map(|s| PathBuf::from(*s))
        .unwrap_or_else(|| PathBuf::from("data/ssnct-2024-pkt-1"));
    let app_data = positional
        .get(1)
        .map(|s| PathBuf::from(*s))
        .unwrap_or_else(default_app_data);

    if !packet_dir.exists() {
        eprintln!("Packet dir not found: {}", packet_dir.display());
        process::exit(1);
    }

    eprintln!(
        "Importing quizbowl clozes from {} into {} {}…",
        packet_dir.display(),
        app_data.display(),
        if append_mode { "(append mode)" } else { "(replace deck notes)" }
    );

    let import_result = if append_mode {
        samsmrti_lib::import_quizbowl_file_append(&app_data, &packet_dir)
    } else {
        samsmrti_lib::import_quizbowl_file(&app_data, &packet_dir)
    };

    match import_result {
        Ok(report) => {
            print_report(
                &report,
                &packet_dir.join("import-report.json").display().to_string(),
            );
        }
        Err(e) => {
            eprintln!("Import failed: {e}");
            process::exit(1);
        }
    }
}

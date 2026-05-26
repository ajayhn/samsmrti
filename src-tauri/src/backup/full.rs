use crate::db::Database;
use rusqlite::Connection;
use serde::Serialize;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::read::ZipArchive;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

pub const FORMAT_FULL_V1: &str = "samsmrti-full-v1";
const MANIFEST_NAME: &str = "manifest.json";
const DB_NAME: &str = "samsmrti.db";
const MEDIA_DIR: &str = "media";

#[derive(Debug, Serialize)]
pub struct FullBackupExportSummary {
    pub path: String,
    pub bytes_written: usize,
    pub media_files: usize,
}

#[derive(Debug, Serialize)]
pub struct FullBackupRestoreSummary {
    pub decks: usize,
    pub notes: usize,
    pub cards: usize,
    pub profiles: usize,
    pub media_files_restored: usize,
    pub previous_db_backup: Option<String>,
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<usize, String> {
    if !src.exists() {
        return Ok(0);
    }
    fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    let mut count = 0usize;
    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let dest = dst.join(entry.file_name());
        if path.is_dir() {
            count += copy_dir_recursive(&path, &dest)?;
        } else {
            fs::copy(&path, &dest).map_err(|e| e.to_string())?;
            count += 1;
        }
    }
    Ok(count)
}

fn add_path_to_zip(
    zip: &mut ZipWriter<std::fs::File>,
    base: &Path,
    path: &Path,
    options: SimpleFileOptions,
) -> Result<(), String> {
    if path.is_dir() {
        for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            add_path_to_zip(zip, base, &entry.path(), options)?;
        }
        return Ok(());
    }
    let rel = path
        .strip_prefix(base)
        .map_err(|e| e.to_string())?
        .to_string_lossy()
        .replace('\\', "/");
    zip.start_file(rel, options)
        .map_err(|e| e.to_string())?;
    let mut f = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).map_err(|e| e.to_string())?;
    zip.write_all(&buf).map_err(|e| e.to_string())?;
    Ok(())
}

fn remove_dir_all(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let p = entry.path();
        if p.is_dir() {
            remove_dir_all(&p)?;
        } else {
            fs::remove_file(&p).map_err(|e| e.to_string())?;
        }
    }
    fs::remove_dir(path).map_err(|e| e.to_string())
}

pub fn export_full_backup_file(
    db: &Database,
    app_data_dir: &Path,
    file_path: &str,
) -> Result<FullBackupExportSummary, String> {
    db.wal_checkpoint().map_err(|e| e.to_string())?;

    let db_path = app_data_dir.join(DB_NAME);
    if !db_path.exists() {
        return Err("Database file not found".into());
    }

    let file = fs::File::create(file_path).map_err(|e| e.to_string())?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated);

    let manifest = serde_json::json!({
        "format": FORMAT_FULL_V1,
        "exported_at": chrono::Utc::now().timestamp(),
        "app_version": env!("CARGO_PKG_VERSION"),
    });
    zip.start_file(MANIFEST_NAME, options)
        .map_err(|e| e.to_string())?;
    zip.write_all(manifest.to_string().as_bytes())
        .map_err(|e| e.to_string())?;

    let db_in_zip = format!("{DB_NAME}");
    zip.start_file(&db_in_zip, options)
        .map_err(|e| e.to_string())?;
    let mut db_bytes = fs::read(&db_path).map_err(|e| e.to_string())?;
    zip.write_all(&mut db_bytes)
        .map_err(|e| e.to_string())?;

    let media_src = app_data_dir.join(MEDIA_DIR);
    let mut media_count = 0usize;
    if media_src.exists() {
        for entry in fs::read_dir(&media_src).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.is_file() {
                media_count += 1;
            }
            add_path_to_zip(
                &mut zip,
                app_data_dir,
                &path,
                options,
            )?;
        }
    }

    let finished = zip.finish().map_err(|e| e.to_string())?;
    let bytes_written = finished.metadata().map(|m| m.len() as usize).unwrap_or(0);

    Ok(FullBackupExportSummary {
        path: file_path.to_string(),
        bytes_written,
        media_files: media_count,
    })
}

fn count_table(conn: &Connection, sql: &str) -> Result<usize, String> {
    let n: i64 = conn
        .query_row(sql, [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    Ok(n as usize)
}

pub fn restore_full_backup_file(
    db: &Database,
    app_data_dir: &PathBuf,
    file_path: &str,
) -> Result<FullBackupRestoreSummary, String> {
    let file = fs::File::open(file_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

    let mut manifest_format = None;
    let mut db_data: Option<Vec<u8>> = None;

    let temp_extract = tempfile::tempdir().map_err(|e| e.to_string())?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();
        let out_path = temp_extract.path().join(&name);
        if name.ends_with('/') {
            fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
            continue;
        }
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf).map_err(|e| e.to_string())?;
        fs::write(&out_path, &buf).map_err(|e| e.to_string())?;

        if name == MANIFEST_NAME {
            let manifest: serde_json::Value =
                serde_json::from_slice(&buf).map_err(|e| e.to_string())?;
            manifest_format = manifest
                .get("format")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
        } else if name == DB_NAME || name.ends_with(DB_NAME) {
            db_data = Some(buf);
        }
    }

    let format = manifest_format.ok_or("Backup missing manifest.json")?;
    if format != FORMAT_FULL_V1 {
        return Err(format!("Unsupported backup format: {format}"));
    }
    let db_data = db_data.ok_or("Backup missing samsmrti.db")?;

    fs::create_dir_all(app_data_dir).map_err(|e| e.to_string())?;
    let db_path = app_data_dir.join(DB_NAME);

    let previous_backup = {
        let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let backup_path = app_data_dir.join(format!("{DB_NAME}.pre-restore-{stamp}"));
        if db_path.exists() {
            db.release_db_file().map_err(|e| e.to_string())?;
            fs::copy(&db_path, &backup_path).map_err(|e| e.to_string())?;
            Some(backup_path.to_string_lossy().into_owned())
        } else {
            db.release_db_file().map_err(|e| e.to_string())?;
            None
        }
    };

    fs::write(&db_path, &db_data).map_err(|e| e.to_string())?;

    let media_dst = app_data_dir.join(MEDIA_DIR);
    if media_dst.exists() {
        remove_dir_all(&media_dst)?;
    }
    fs::create_dir_all(&media_dst).map_err(|e| e.to_string())?;

    let media_src = temp_extract.path().join(MEDIA_DIR);
    let media_files_restored = if media_src.exists() {
        copy_dir_recursive(&media_src, &media_dst)?
    } else {
        0
    };

    db.reopen(&db_path).map_err(|e| e.to_string())?;
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        crate::db::card_progress::apply_schema_migrations(&conn)
            .map_err(|e| e.to_string())?;
        let _ = crate::commands::search::ensure_search_index_conn(&conn);
    }

    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    Ok(FullBackupRestoreSummary {
        decks: count_table(&conn, "SELECT COUNT(*) FROM decks")?,
        notes: count_table(&conn, "SELECT COUNT(*) FROM notes")?,
        cards: count_table(&conn, "SELECT COUNT(*) FROM cards")?,
        profiles: count_table(&conn, "SELECT COUNT(*) FROM profiles")?,
        media_files_restored,
        previous_db_backup: previous_backup,
    })
}

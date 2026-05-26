use crate::commands::search::upsert_note_fts_conn;
use crate::import::ImportResult;
use rusqlite::Connection;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const DECK_ID: &str = "dk_ssnct";
const DECK_NAME: &str = "ssnct";
const NOTE_TYPE_ID: &str = "nt_cloze";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum QuestionKind {
    Tossup,
    Bonus,
}

#[derive(Debug, Clone)]
pub struct ParsedTossup {
    pub lead_in: String,
    pub post_power: String,
    pub answer: String,
}

#[derive(Debug, Clone)]
pub struct BonusPart {
    pub label: char,
    pub stem: String,
    pub answer: String,
}

#[derive(Debug, Clone)]
pub struct ParsedBonus {
    pub intro: String,
    pub parts: Vec<BonusPart>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportReportEntry {
    pub file: String,
    pub kind: QuestionKind,
    pub tags: Vec<String>,
    pub cloze_count: usize,
    pub warnings: Vec<String>,
    pub ocr_preview: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportReport {
    pub entries: Vec<ImportReportEntry>,
    pub summary: ImportResult,
}

pub fn ocr_png(path: &Path) -> Result<String, String> {
    let output = Command::new("tesseract")
        .arg(path)
        .arg("stdout")
        .output()
        .map_err(|e| {
            format!(
                "Failed to run tesseract: {e}. Install with: brew install tesseract"
            )
        })?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("tesseract failed on {}: {err}", path.display()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn normalize_ocr(text: &str) -> String {
    let mut s = text.replace("\r\n", "\n").replace('\r', "\n");
    // Common OCR glitches
    s = s.replace("calledimpulse", "called impulse");
    s = s.replace("1s ", "is ");
    s = s.replace("inanovel", "in a novel");
    s = s.replace("bio-geography", "biogeography");
    s = s.replace("ark’-ih", "ark-ih");
    s = s.replace("“", "\"").replace("”", "\"").replace("’", "'");
    s = s.replace("—", "—");
    s
}

pub fn classify(text: &str) -> QuestionKind {
    let lower = text.to_lowercase();
    if lower.contains("for 10 points each") {
        return QuestionKind::Bonus;
    }
    let has_a = find_part_marker(text, 'A').is_some();
    let has_b = find_part_marker(text, 'B').is_some();
    let answer_count = lower.matches("answer:").count();
    if has_a && has_b && answer_count >= 2 {
        return QuestionKind::Bonus;
    }
    QuestionKind::Tossup
}

fn strip_answer_prefix(s: &str) -> String {
    let t = s.trim();
    let lower = t.to_lowercase();
    if let Some(rest) = lower.strip_prefix("answer:") {
        t[t.len() - rest.len()..].trim().to_string()
    } else {
        t.to_string()
    }
}

fn find_answer_start(text: &str, from: usize) -> Option<usize> {
    let lower = text.to_lowercase();
    let hay = &lower[from..];
    let idx = hay.find("answer:")?;
    Some(from + idx)
}

pub fn parse_tossup(text: &str) -> Result<ParsedTossup, String> {
    let star = text.find("(*)").ok_or("tossup missing (*) power mark")?;
    let lead_in = text[..star].trim().to_string();
    let after = &text[star + 3..];
    let ans_start = find_answer_start(after, 0).ok_or("tossup missing answer:")?;
    let post_power = after[..ans_start].trim().to_string();
    if post_power.is_empty() {
        return Err("empty post-power tossup text".into());
    }
    let answer = strip_answer_prefix(&after[ans_start..]);
    if answer.is_empty() {
        return Err("empty tossup answer".into());
    }
    Ok(ParsedTossup {
        lead_in,
        post_power,
        answer,
    })
}

fn find_part_marker(text: &str, label: char) -> Option<usize> {
    let patterns = [
        format!("\n{label}."),
        format!("\n{label} "),
        format!("\r{label}."),
    ];
    let mut best: Option<usize> = None;
    for pat in &patterns {
        if let Some(i) = text.find(pat) {
            best = Some(best.map_or(i, |b| b.min(i)));
        }
    }
    if label == 'A' {
        if let Some(i) = text.find("A.") {
            if i < 3 || !text[..i].contains('\n') {
                best = Some(best.map_or(i, |b| b.min(i)));
            }
        }
    }
    best
}

pub fn parse_bonus(text: &str) -> Result<ParsedBonus, String> {
    let a_pos = find_part_marker(text, 'A').ok_or("bonus missing part A.")?;
    let intro = text[..a_pos].trim().to_string();

    let mut parts = Vec::new();
    for label in ['A', 'B', 'C'] {
        let Some(start) = find_part_marker(text, label) else {
            if label == 'C' && parts.len() >= 2 {
                break;
            }
            return Err(format!("bonus missing part {label}."));
        };
        let stem_start = if let Some(dot) = text[start..].find('.') {
            start + dot + 1
        } else {
            start + 2
        };

        let end = match label {
            'A' => find_part_marker(text, 'B'),
            'B' => find_part_marker(text, 'C'),
            'C' | _ => None,
        }
        .unwrap_or(text.len());

        let chunk = &text[stem_start..end];
        let ans_pos = find_answer_start(chunk, 0)
            .ok_or_else(|| format!("bonus part {label} missing answer:"))?;
        let stem = chunk[..ans_pos].trim().to_string();
        let answer = strip_answer_prefix(&chunk[ans_pos..]);
        if answer.is_empty() {
            return Err(format!("bonus part {label} empty answer"));
        }
        parts.push(BonusPart { label, stem, answer });
    }

    if parts.len() < 2 {
        return Err("bonus needs at least parts A and B".into());
    }

    Ok(ParsedBonus { intro, parts })
}

fn escape_cloze_content(s: &str) -> String {
    s.replace("}}", "}")
}

fn wrap_cloze(n: usize, content: &str) -> String {
    let inner = escape_cloze_content(content);
    format!("{{{{c{n}::{inner}}}}}")
}

pub fn build_tossup_cloze_text(p: &ParsedTossup) -> String {
    let mut out = String::new();
    if !p.lead_in.is_empty() {
        out.push_str(&p.lead_in);
        out.push_str("\n\n");
    }
    out.push_str(&wrap_cloze(1, &p.post_power));
    out.push_str("\n\nAnswer: ");
    out.push_str(&wrap_cloze(2, &p.answer));
    out
}

pub fn build_bonus_part_text(intro: &str, part: &BonusPart, prior: &[(&BonusPart, &str)]) -> String {
    let mut out = String::new();
    if !intro.is_empty() {
        out.push_str(intro.trim());
        out.push_str("\n\n");
    }
    for (p, revealed) in prior {
        out.push_str(&format!("{}. {}\n\nAnswer: {}\n\n", p.label, p.stem.trim(), revealed));
    }
    out.push_str(&format!("{}. {}\n\nAnswer: ", part.label, part.stem.trim()));
    out.push_str(&wrap_cloze(1, &part.answer));
    out
}

pub fn build_bonus_cloze_text(p: &ParsedBonus) -> String {
    let mut out = String::new();
    if !p.intro.is_empty() {
        out.push_str(p.intro.trim());
        out.push_str("\n\n");
    }
    for (i, part) in p.parts.iter().enumerate() {
        if i > 0 {
            out.push_str("\n\n");
        }
        out.push_str(&format!("{}. {}\n\nAnswer: ", part.label, part.stem.trim()));
        out.push_str(&wrap_cloze(i + 1, &part.answer));
    }
    out
}

fn count_cloze_deletions(text: &str) -> usize {
    let mut max_n = 0usize;
    let mut i = 0;
    let bytes = text.as_bytes();
    while i + 3 < bytes.len() {
        if bytes[i] == b'{' && bytes[i + 1] == b'{' && bytes[i + 2] == b'c' {
            let start = i + 3;
            let mut end = start;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
            if end > start && end + 1 < bytes.len() && bytes[end] == b':' && bytes[end + 1] == b':' {
                if let Ok(n) = text[start..end].parse::<usize>() {
                    max_n = max_n.max(n);
                }
            }
        }
        i += 1;
    }
    max_n
}

fn tags_for(kind: QuestionKind) -> &'static [&'static str] {
    match kind {
        QuestionKind::Tossup => &["quizbowl", "tossup", "ssnct"],
        QuestionKind::Bonus => &["quizbowl", "bonus", "ssnct"],
    }
}

fn file_slug(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .chars()
        .take(80)
        .collect()
}

fn dir_slug(dir: &Path) -> String {
    file_slug(dir)
}

fn packet_label(dir: &Path) -> String {
    let name = dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("packet");
    if let Some(num) = name.rsplit('-').next() {
        if num.chars().all(|c| c.is_ascii_digit()) {
            return format!("SSNCT 2024 Pkt {num}");
        }
    }
    name.to_string()
}

fn is_tournament_header_line(line: &str) -> bool {
    let t = line.trim();
    if t.is_empty() {
        return false;
    }
    let lower = t.to_lowercase();
    lower.contains("national championship tournament")
}

fn is_packet_id_line(line: &str) -> bool {
    let t = line.trim();
    // OCR may read packet IDs as `<603709>`, `«603709`, or `‹603709`.
    t.starts_with('<') || t.starts_with('«') || t.starts_with('‹')
}

/// Line starts a numbered tossup/bonus stem, e.g. `1. This author...`
fn is_numbered_question_line(line: &str) -> bool {
    let t = line.trim();
    let bytes = t.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == 0 || i > 3 {
        return false;
    }
    if i >= bytes.len() || bytes[i] != b'.' {
        return false;
    }
    i += 1;
    i < bytes.len() && bytes[i].is_ascii_whitespace()
}

fn segment_has_answer(segment: &str) -> bool {
    segment.to_lowercase().contains("answer:")
}

/// Split OCR from a multi-question page on packet ID lines like `<603709>`,
/// and on numbered question lines once the previous question has an answer.
pub fn split_multipage_questions(text: &str) -> Vec<String> {
    let mut segments: Vec<String> = Vec::new();
    let mut current = String::new();

    for line in text.lines() {
        if is_packet_id_line(line) {
            if !current.trim().is_empty() {
                segments.push(current.trim().to_string());
                current.clear();
            }
            continue;
        }
        if is_tournament_header_line(line) {
            continue;
        }
        if is_numbered_question_line(line)
            && !current.trim().is_empty()
            && segment_has_answer(&current)
        {
            segments.push(current.trim().to_string());
            current.clear();
        }
        if !line.trim().is_empty() || !current.is_empty() {
            current.push_str(line);
            current.push('\n');
        }
    }

    if !current.trim().is_empty() {
        segments.push(current.trim().to_string());
    }

    segments.retain(|s| s.len() > 30);
    segments
}

fn ensure_ssnct_deck(conn: &Connection, now: i64, result: &mut ImportResult) -> Result<(), String> {
    let deck_exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM decks WHERE id = ?1",
            [DECK_ID],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if deck_exists == 0 {
        conn.execute(
            "INSERT INTO decks (id, name, parent_id, description, new_per_day, max_reviews, created_at, updated_at) VALUES (?1, ?2, NULL, ?3, 20, 200, ?4, ?4)",
            (
                DECK_ID,
                DECK_NAME,
                "SSNCT quizbowl tossups and bonuses",
                now,
            ),
        )
        .map_err(|e| e.to_string())?;
        result.decks_imported = 1;
    }
    Ok(())
}

fn import_text_segment(
    tx: &Connection,
    text: &str,
    source_label: &str,
    slug: &str,
    packet_label: &str,
    now: i64,
    result: &mut ImportResult,
    entries: &mut Vec<ImportReportEntry>,
) -> Result<(), String> {
    let mut warnings = Vec::new();
    let preview: String = text.chars().take(200).collect();
    let kind = classify(text);

    if kind == QuestionKind::Bonus {
        match parse_bonus(text) {
            Ok(p) => {
                if p.parts.len() != 3 {
                    warnings.push(format!("expected 3 parts, got {}", p.parts.len()));
                }
                let body = build_bonus_cloze_text(&p);
                let cloze_count = count_cloze_deletions(&body);
                if cloze_count == 0 {
                    warnings.push("no cloze deletions found".into());
                    result
                        .warnings
                        .push(format!("{source_label}: no clozes"));
                    return Ok(());
                }
                let tag_list = tags_for(QuestionKind::Bonus);
                let note_id = format!("n_qb_{slug}");
                let card_id = format!("c_qb_{slug}");
                let extra = format!("Bonus · {packet_label} · {source_label}");
                delete_note_if_exists(tx, &note_id)?;
                insert_cloze_note(
                    tx,
                    &note_id,
                    &body,
                    &extra,
                    tag_list,
                    &card_id,
                    now,
                    result,
                )?;
                entries.push(ImportReportEntry {
                    file: source_label.to_string(),
                    kind: QuestionKind::Bonus,
                    tags: tag_list.iter().map(|s| s.to_string()).collect(),
                    cloze_count,
                    warnings,
                    ocr_preview: preview,
                });
            }
            Err(e) => {
                result.warnings.push(format!("{source_label}: {e}"));
            }
        }
        return Ok(());
    }

    match parse_tossup(text) {
        Ok(p) => {
            let body = build_tossup_cloze_text(&p);
            let cloze_count = count_cloze_deletions(&body);
            if cloze_count == 0 {
                result
                    .warnings
                    .push(format!("{source_label}: no clozes"));
                return Ok(());
            }
            let tag_list = tags_for(QuestionKind::Tossup);
            let note_id = format!("n_qb_{slug}");
            let extra = format!("Tossup · {packet_label} · {source_label}");
            delete_note_if_exists(tx, &note_id)?;
            insert_cloze_note(
                tx,
                &note_id,
                &body,
                &extra,
                tag_list,
                &format!("c_qb_{slug}"),
                now,
                result,
            )?;
            entries.push(ImportReportEntry {
                file: source_label.to_string(),
                kind: QuestionKind::Tossup,
                tags: tag_list.iter().map(|s| s.to_string()).collect(),
                cloze_count,
                warnings: Vec::new(),
                ocr_preview: preview,
            });
        }
        Err(e) => {
            result.warnings.push(format!("{source_label}: {e}"));
        }
    }
    Ok(())
}

fn delete_note_if_exists(tx: &Connection, note_id: &str) -> Result<(), String> {
    tx.execute("DELETE FROM cards WHERE note_id = ?1", [note_id])
        .map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM notes WHERE id = ?1", [note_id])
        .map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM notes_fts WHERE note_id = ?1", [note_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn import_png_segments(
    tx: &Connection,
    segments: &[String],
    source_label: &str,
    slug_base: &str,
    packet_label: &str,
    now: i64,
    result: &mut ImportResult,
    entries: &mut Vec<ImportReportEntry>,
) -> Result<(), String> {
    if segments.is_empty() {
        return Ok(());
    }
    // Drop a prior single-note import when this page splits into multiple questions.
    if segments.len() > 1 {
        delete_note_if_exists(tx, &format!("n_qb_{slug_base}"))?;
    }
    if segments.len() == 1 {
        import_text_segment(
            tx,
            &segments[0],
            source_label,
            slug_base,
            packet_label,
            now,
            result,
            entries,
        )?;
    } else {
        for (i, segment) in segments.iter().enumerate() {
            let label = format!("{source_label} #{i}");
            let slug = format!("{slug_base}_{i}");
            import_text_segment(
                tx,
                segment,
                &label,
                &slug,
                packet_label,
                now,
                result,
                entries,
            )?;
        }
    }
    Ok(())
}

/// Import a single PNG (possibly containing multiple questions) into deck `ssnct` without clearing existing notes.
pub fn import_quizbowl_png(conn: &Connection, path: &Path) -> Result<ImportReport, String> {
    let mut result = ImportResult::default();
    let mut entries = Vec::new();
    let now = chrono::Utc::now().timestamp();

    ensure_ssnct_deck(conn, now, &mut result)?;

    let fname = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown.png")
        .to_string();
    let slug_base = file_slug(path);
    let label = path
        .parent()
        .map(packet_label)
        .unwrap_or_else(|| "Quizbowl".to_string());

    let raw_ocr = ocr_png(path)?;
    let text = normalize_ocr(&raw_ocr);
    let segments = split_multipage_questions(&text);

    if segments.is_empty() {
        return Err(format!("{fname}: no questions found after OCR"));
    }

    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    import_png_segments(
        &tx,
        &segments,
        &fname,
        &slug_base,
        &label,
        now,
        &mut result,
        &mut entries,
    )?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(ImportReport {
        entries,
        summary: result,
    })
}

fn attach_tags(conn: &Connection, note_id: &str, tag_names: &[&str]) -> Result<(), String> {
    for name in tag_names {
        let tag_id = format!("t_qb_{}", uuid::Uuid::new_v4().simple());
        conn.execute(
            "INSERT OR IGNORE INTO tags (id, name) VALUES (?1, ?2)",
            (&tag_id, name),
        )
        .map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR IGNORE INTO note_tags (note_id, tag_id) SELECT ?1, id FROM tags WHERE name = ?2",
            (note_id, name),
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn clear_deck_notes(conn: &Connection, deck_id: &str) -> Result<(), String> {
    conn.execute(
        "DELETE FROM cards WHERE note_id IN (SELECT id FROM notes WHERE deck_id = ?1)",
        [deck_id],
    )
    .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM notes WHERE deck_id = ?1", [deck_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn insert_cloze_note(
    tx: &Connection,
    note_id: &str,
    text_field: &str,
    extra: &str,
    tag_list: &[&str],
    card_id: &str,
    now: i64,
    result: &mut ImportResult,
) -> Result<(), String> {
    let fields = serde_json::json!({
        "Text": text_field,
        "Extra": extra,
    });
    let fields_str = fields.to_string();

    tx.execute(
        "INSERT INTO notes (id, deck_id, note_type_id, fields_json, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
        (note_id, DECK_ID, NOTE_TYPE_ID, &fields_str, now),
    )
    .map_err(|e| e.to_string())?;
    result.notes_imported += 1;

    attach_tags(tx, note_id, tag_list)?;

    tx.execute(
        "INSERT INTO cards (id, note_id, template_ordinal, state, due_at) VALUES (?1, ?2, 0, 'new', ?3)",
        (card_id, note_id, now),
    )
    .map_err(|e| e.to_string())?;
    crate::db::card_progress::seed_progress_for_all_profiles(tx, card_id, now)
        .map_err(|e| e.to_string())?;
    result.cards_imported += 1;

    upsert_note_fts_conn(tx, note_id, &fields_str)?;
    Ok(())
}

pub fn import_quizbowl_dir(conn: &Connection, dir: &Path) -> Result<ImportReport, String> {
    import_quizbowl_dir_with_options(conn, dir, true)
}

/// Import all PNGs in a packet directory. When `clear_deck` is false, existing notes are kept.
pub fn import_quizbowl_dir_with_options(
    conn: &Connection,
    dir: &Path,
    clear_deck: bool,
) -> Result<ImportReport, String> {
    let mut result = ImportResult::default();
    let mut entries = Vec::new();
    let now = chrono::Utc::now().timestamp();
    let pkt_label = packet_label(dir);
    let packet_slug = dir_slug(dir);

    ensure_ssnct_deck(conn, now, &mut result)?;
    if clear_deck {
        clear_deck_notes(conn, DECK_ID)?;
    }

    let mut pngs: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("Read dir {}: {e}", dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "png"))
        .collect();
    pngs.sort();

    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    for path in &pngs {
        let fname = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown.png")
            .to_string();

        let raw_ocr = match ocr_png(path) {
            Ok(t) => t,
            Err(e) => {
                entries.push(ImportReportEntry {
                    file: fname.clone(),
                    kind: QuestionKind::Tossup,
                    tags: vec![],
                    cloze_count: 0,
                    warnings: vec![e.clone()],
                    ocr_preview: String::new(),
                });
                result.warnings.push(format!("{fname}: {e}"));
                continue;
            }
        };

        let text = normalize_ocr(&raw_ocr);
        let slug_base = format!("{}_{}", packet_slug, file_slug(path));
        let segments = split_multipage_questions(&text);

        if segments.is_empty() {
            result.warnings.push(format!("{fname}: no questions found"));
            continue;
        }

        import_png_segments(
            &tx,
            &segments,
            &fname,
            &slug_base,
            &pkt_label,
            now,
            &mut result,
            &mut entries,
        )?;
    }

    tx.commit().map_err(|e| e.to_string())?;

    let report_path = dir.join("import-report.json");
    let report = ImportReport {
        entries: entries.clone(),
        summary: result.clone(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&report) {
        let _ = fs::write(&report_path, json);
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOSSUP_SAMPLE: &str = r#"1. A "generalized" form of this quantity is the partial derivative of the Lagrangian with respect
to generalized velocity. Planck's constant over this quantity gives the de Broglie (*) wavelength. The average force multiplied by the time the force acts is the change in this quantity, which is
called impulse. This classically conserved quantity is denoted by p. For 10 points—what quantity classically
equals mass times velocity?

answer: (linear) momentum (or momenta)"#;

    #[test]
    fn classify_tossup() {
        assert_eq!(classify(TOSSUP_SAMPLE), QuestionKind::Tossup);
    }

    #[test]
    fn parse_tossup_sample() {
        let p = parse_tossup(TOSSUP_SAMPLE).unwrap();
        assert!(p.lead_in.contains("Lagrangian"));
        assert!(p.post_power.contains("wavelength"));
        assert!(p.answer.contains("momentum"));
    }

    #[test]
    fn tossup_cloze_markup() {
        let p = parse_tossup(TOSSUP_SAMPLE).unwrap();
        let text = build_tossup_cloze_text(&p);
        assert!(text.contains("{{c1::"));
        assert!(text.contains("{{c2::"));
        assert_eq!(count_cloze_deletions(&text), 2);
    }

    #[test]
    fn wrap_cloze_format() {
        assert_eq!(wrap_cloze(1, "foo"), "{{c1::foo}}");
    }

    #[test]
    fn bonus_cloze_three_parts() {
        let p = ParsedBonus {
            intro: "For 10 points each—".into(),
            parts: vec![
                BonusPart {
                    label: 'A',
                    stem: "Name this continent.".into(),
                    answer: "Africa".into(),
                },
                BonusPart {
                    label: 'B',
                    stem: "Name this region.".into(),
                    answer: "Sahel".into(),
                },
                BonusPart {
                    label: 'C',
                    stem: "Name this country.".into(),
                    answer: "Sudan".into(),
                },
            ],
        };
        let text = build_bonus_cloze_text(&p);
        assert_eq!(count_cloze_deletions(&text), 3);
        assert!(text.contains("A. Name this continent.\n\nAnswer: {{c1::Africa}}"));
        assert!(text.contains("\n\nB. Name this region.\n\nAnswer: {{c2::Sahel}}"));
    }

    #[test]
    fn split_multipage_without_packet_ids() {
        const SAMPLE: &str = r#"1. Edge weights cannot be these numbers in (*) Dijkstra's algorithm. For 10 points—what
numbers that are less than zero?

answer: negative numbers

1. In 2023 scientists found that this region had reversed its "super-rotation". For 10 points each—

A. What solid region is the densest part of the Earth?

answer: inner core

B. Like the outer core, Earth's inner core contains large quantities of this element.

answer: iron

C. Seismologist Inge Lehmann discovered the inner core via reflection of p-waves into these regions.

answer: shadow zones

2. A party now known as "Renaissance" defeated the far-right Marine le Pen in—for 10
points—what country that re-elected Emmanuel Macron?

answer: France
"#;
        let segments = split_multipage_questions(SAMPLE);
        assert_eq!(segments.len(), 3);
        assert_eq!(classify(&segments[0]), QuestionKind::Tossup);
        assert_eq!(classify(&segments[1]), QuestionKind::Bonus);
        assert_eq!(classify(&segments[2]), QuestionKind::Tossup);
    }

    #[test]
    fn split_multipage_guillemet_packet_ids() {
        const SAMPLE: &str = r#"19. For 10 points each—

A. Name this country?

answer: Ecuador

«603039

20. This author wrote Tam o' Shanter. (*) bridge after catcalling a witch. For 10 points—what Scotsman?

answer: Robert Burns

«606604

20. This case concerned Plessy. For 10 points each—

A. Name this 1896 case?

answer: Plessy v. Ferguson

B. Plessy challenged the act in this city?

answer: New Orleans

C. The lone dissent was from this justice?

answer: John Marshall Harlan
"#;
        let segments = split_multipage_questions(SAMPLE);
        assert_eq!(segments.len(), 3);
        assert_eq!(classify(&segments[0]), QuestionKind::Bonus);
        assert_eq!(classify(&segments[1]), QuestionKind::Tossup);
        assert_eq!(classify(&segments[2]), QuestionKind::Bonus);
    }

    #[test]
    fn split_multipage_sample() {
        const SAMPLE: &str = r#"1. Salts of this anion carry out a redox reaction. (*) nitrile group. For 10 points—name this anion.

answer: cyanide

<603709>

1. This leader suppressed the Jeju uprising. For 10 points each—

A. Name this dictator?

answer: Syngman Rhee

B. Capital city?

answer: Busan

C. Exile capital?

answer: Seoul

<602126>

2. A pope with this name chose to resign. (*) shifting equinox. For 10 points—give this papal name.

answer: Gregory
"#;
        let segments = split_multipage_questions(SAMPLE);
        assert_eq!(segments.len(), 3);
        assert_eq!(classify(&segments[0]), QuestionKind::Tossup);
        assert_eq!(classify(&segments[1]), QuestionKind::Bonus);
        assert_eq!(classify(&segments[2]), QuestionKind::Tossup);
    }

    #[test]
    fn bonus_part_text_single_cloze() {
        let intro = "For 10 points each—";
        let a = BonusPart {
            label: 'A',
            stem: "Name this state.".into(),
            answer: "Uttar Pradesh".into(),
        };
        let b = BonusPart {
            label: 'B',
            stem: "Name this river.".into(),
            answer: "Ganges".into(),
        };
        let text_a = build_bonus_part_text(intro, &a, &[]);
        assert_eq!(count_cloze_deletions(&text_a), 1);
        assert!(text_a.contains("{{c1::Uttar Pradesh}}"));

        let text_b = build_bonus_part_text(intro, &b, &[(&a, "Uttar Pradesh")]);
        assert_eq!(count_cloze_deletions(&text_b), 1);
        assert!(text_b.contains("{{c1::Ganges}}"));
        assert!(text_b.contains("Uttar Pradesh"));
    }
}

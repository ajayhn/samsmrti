pub mod anki_decks;
pub mod apkg;
pub mod mochi;
pub mod quizbowl;
pub mod senators;

use serde::Serialize;

#[derive(Debug, Serialize, Default, Clone)]
pub struct ImportResult {
    pub decks_imported: usize,
    pub notes_imported: usize,
    pub cards_imported: usize,
    pub media_imported: usize,
    pub warnings: Vec<String>,
}

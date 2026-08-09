//! wasm-bindgen entry points for the browser core. Each command module's
//! `#[cfg(target_arch = "wasm32")]` wasm exports (see e.g. commands/decks.rs)
//! are re-exported here so JS sees one flat module, mirroring the shape of
//! src/lib/tauri.ts's `api` object today.
//!
//! Storage is in-memory only until the OPFS+dedicated-Worker wiring lands
//! (Phase 1 continuation) -- see streamed-yawning-coral.md.

use wasm_bindgen::prelude::*;

#[allow(unused_imports)] // #[wasm_bindgen] fns here are called from JS, not Rust
pub use crate::commands::decks::*;
#[allow(unused_imports)]
pub use crate::commands::karma::*;
#[allow(unused_imports)]
pub use crate::commands::profiles::*;
#[allow(unused_imports)]
pub use crate::commands::review::*;

#[wasm_bindgen]
pub fn init() -> Result<(), JsValue> {
    crate::db::wasm_singleton::init_in_memory().map_err(|e| JsValue::from_str(&e))
}

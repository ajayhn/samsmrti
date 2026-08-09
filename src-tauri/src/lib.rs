mod backup;
mod commands;
pub mod db;
pub mod seed;

#[cfg(not(target_arch = "wasm32"))]
pub mod import;

// Native desktop app: Tauri window/menu chrome, invoke-handler registration,
// and the CLI import binaries all live here.
#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
pub use native::{
    import_anki_collection_file, import_quizbowl_file, import_quizbowl_file_append,
    import_quizbowl_png_file, import_senators_file, run,
};
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use native::refresh_menu;

// Browser core: wasm-bindgen exports consumed by the PWA (and eventually the
// desktop webview once it migrates onto this same engine). See
// /Users/hampapurajay/.claude/plans/streamed-yawning-coral.md Phase 1.
#[cfg(target_arch = "wasm32")]
mod wasm_api;

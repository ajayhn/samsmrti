//! wasm-bindgen entry points for the browser core. Each command module's
//! `#[cfg(target_arch = "wasm32")]` wasm exports (see e.g. commands/decks.rs)
//! are re-exported here so JS sees one flat module, mirroring the shape of
//! src/lib/tauri.ts's `api` object today.

use wasm_bindgen::prelude::*;

#[allow(unused_imports)] // #[wasm_bindgen] fns here are called from JS, not Rust
pub use crate::commands::backup::*;
#[allow(unused_imports)]
pub use crate::commands::decks::*;
#[allow(unused_imports)]
pub use crate::commands::karma::*;
#[allow(unused_imports)]
pub use crate::commands::profiles::*;
#[allow(unused_imports)]
pub use crate::commands::review::*;
#[allow(unused_imports)]
pub use crate::commands::search::*;

/// In-memory only, does not survive a reload. Useful for quick checks/tests;
/// the PWA itself should call `initOpfs` instead.
#[wasm_bindgen(js_name = initInMemory)]
pub fn init_in_memory() -> Result<(), JsValue> {
    crate::db::wasm_singleton::init_in_memory().map_err(|e| JsValue::from_str(&e))
}

/// OPFS-persisted storage. Must be called from within a dedicated Worker --
/// the underlying FileSystemSyncAccessHandle API used by the sahpool VFS
/// isn't available on the main thread in any current browser.
#[wasm_bindgen(js_name = initOpfs)]
pub async fn init_opfs() -> Result<(), JsValue> {
    crate::db::wasm_singleton::init_opfs()
        .await
        .map_err(|e| JsValue::from_str(&e))
}

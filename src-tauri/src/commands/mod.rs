pub mod decks;
pub mod karma;
pub mod profiles;
pub mod review;

// Not yet ported to the platform-agnostic core/wasm pattern (see Phase 1 of
// /Users/hampapurajay/.claude/plans/streamed-yawning-coral.md) -- native/Tauri only for now.
#[cfg(not(target_arch = "wasm32"))]
pub mod backup;
#[cfg(not(target_arch = "wasm32"))]
pub mod export;
#[cfg(not(target_arch = "wasm32"))]
pub mod graph;
#[cfg(not(target_arch = "wasm32"))]
pub mod import;
#[cfg(not(target_arch = "wasm32"))]
pub mod note_types;
#[cfg(not(target_arch = "wasm32"))]
pub mod notes;
#[cfg(not(target_arch = "wasm32"))]
pub mod window_profiles;
#[cfg(not(target_arch = "wasm32"))]
pub mod search;

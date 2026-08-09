fn main() {
    println!("cargo:rerun-if-changed=icons/icon.png");
    println!("cargo:rerun-if-changed=icons/icon.icns");
    println!("cargo:rerun-if-changed=icons/icon.ico");

    // The wasm32 lib target never calls tauri::generate_context!(), and
    // tauri-build's codegen assumes it's driven by the `tauri` CLI wrapper
    // (panics with "missing `cargo:dev` instruction" otherwise) -- skip it
    // for that target rather than fighting an irrelevant native-only step.
    let target = std::env::var("TARGET").unwrap_or_default();
    if !target.starts_with("wasm32") {
        tauri_build::build()
    }
}

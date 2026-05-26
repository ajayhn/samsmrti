# Development

Guide for building, testing, and packaging Samsmrti for distribution.

## Prerequisites

- **Node.js** (LTS recommended) and npm
- **Rust** toolchain (`rustup`, stable)
- **macOS:** Xcode Command Line Tools (for linking and app bundles)
- **Windows / Linux:** see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

## Local development

```bash
npm install
npm run tauri dev
```

This starts the Vite dev server and the Tauri shell with hot reload for the frontend.

### Frontend only

```bash
npm run dev          # Vite on http://localhost:1420
npm run build        # tsc + production Vite build
npm run lint         # ESLint
```

### Rust / backend

```bash
cd src-tauri
cargo build
cargo test
```

CLI import tools (optional):

```bash
cargo run --bin import_anki -- <path>
cargo run --bin import_quizbowl -- <packet-dir>
cargo run --bin import_senators -- <json-path> [html-path]
```

## Project layout

| Path | Purpose |
|------|---------|
| `src/` | React UI, Zustand stores, Tauri API wrappers |
| `src-tauri/src/` | Rust commands, DB, import/export, backup |
| `src-tauri/src/db/schema.sql` | SQLite schema |
| `src-tauri/src/backup/` | Content JSON + full zip backup/restore |
| `src-tauri/tauri.conf.json` | App id, window, build hooks |

App identifier: `com.samsmrti.desktop` (see `tauri.conf.json`).

## Bundling for distribution

To produce an **installable release** (not just `target/debug/samsmrti`):

```bash
npm install
npm run tauri build
```

`tauri build` runs `npm run build` first (`beforeBuildCommand` in `tauri.conf.json`), then compiles the Rust crate in release mode and creates platform bundles.

### Output location

Artifacts are under:

```text
src-tauri/target/release/bundle/
```

| Platform | Typical outputs |
|----------|-----------------|
| **macOS** | `macos/Samsmrti.app`, optionally `dmg/*.dmg` |
| **Windows** | `nsis/*.exe` or `msi/*.msi` |
| **Linux** | `deb/`, `appimage/`, etc. (depends on Tauri bundle config) |

Ship the `.app`, `.dmg`, `.msi`, or `.deb` from that folder to users.

### What to give end users

| Scenario | What to distribute |
|----------|-------------------|
| **Fresh install** | The bundled app only (`.app` / installer). They get empty library + seeded example decks on first launch. |
| **Move their existing library** | Bundled app **plus** a **full backup** from File → **Backup (Full)…** (`.samsmrti-backup`), or a zip of the app data directory (see below). |

App data directory (for manual copy or support):

| Platform | Path |
|----------|------|
| macOS | `~/Library/Application Support/com.samsmrti.desktop/` |
| Linux | `~/.local/share/com.samsmrti.desktop/` (or XDG data dir) |
| Windows | `%APPDATA%\com.samsmrti.desktop\` |

Contains `samsmrti.db` and `media/`. Prefer in-app **Backup (Full)** / **Restore (Full)** over hand-copying when possible.

### If the build fails on the frontend

`npm run tauri build` depends on `npm run build` (`tsc -b && vite build`). Fix TypeScript errors in `src/` before bundling; the Rust side can compile while the frontend still fails.

### Version and product name

Set in `src-tauri/tauri.conf.json` (`version`, `productName`) and `src-tauri/Cargo.toml` (`version`) before release builds.

## Related docs

- **[QUICKSTART.md](./QUICKSTART.md)** — user-facing: profiles, export/import, features
- **[README.md](./README.md)** — project overview

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

**File → New Window** opens another Samsmrti window. Each window keeps its own UI state (review session, browse filters, etc.) and its own **active profile**—switching profile in one window does not change the other. All windows share the same database (decks, notes, cards).

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

Set in `src-tauri/tauri.conf.json` (`version`, `productName`) and `src-tauri/Cargo.toml` (`version`) before release builds. Keep these in sync (e.g. `0.1.0`).

## Releasing to GitHub (macOS)

Published builds are **the app only** — no `samsmrti.db`, media, or user library. First launch creates a fresh library under the app data directory (see above).

CI workflow: [`.github/workflows/release-macos.yml`](.github/workflows/release-macos.yml) on `macos-latest`. It runs `npm run tauri build`, then uploads:

| Artifact | Description |
|----------|-------------|
| `Samsmrti_<version>_aarch64.dmg` | Apple Silicon installer (drag to Applications) |
| `Samsmrti-<version>-macos.zip` | Zipped `Samsmrti.app` |

Both appear on the [GitHub Releases](https://github.com/ajayhn/samsmrti/releases) page for the tag, and as a workflow artifact named `samsmrti-macos`.

### Standard release (recommended)

1. **Bump the version** in both files:
   - `src-tauri/tauri.conf.json` → `"version"`
   - `src-tauri/Cargo.toml` → `version` under `[package]`

2. **Commit and push** to `main`:
   ```bash
   git add src-tauri/tauri.conf.json src-tauri/Cargo.toml
   git commit -m "Bump version to 0.2.0."
   git push origin main
   ```

3. **Create and push an annotated tag** (must match `v*` — e.g. `v0.2.0`):
   ```bash
   git tag -a v0.2.0 -m "Samsmrti v0.2.0"
   git push origin v0.2.0
   ```

4. **Watch the workflow** on GitHub: **Actions → Release macOS**. When it succeeds, open **Releases → v0.2.0** and download the `.dmg` or `.zip`.

To **re-run a failed release** for the same version, delete the remote tag, fix the issue on `main`, move the tag to the new commit, and force-push the tag:

```bash
git push origin :refs/tags/v0.2.0    # delete remote tag
git tag -fa v0.2.0 -m "Samsmrti v0.2.0"
git push origin v0.2.0 -f
```

Delete the draft/failed GitHub Release first if GitHub rejects duplicate uploads.

### Manual release (no tag)

On GitHub: **Actions → Release macOS → Run workflow** (branch `main`). This builds and uploads the **workflow artifact** only; it does **not** create a GitHub Release unless you also push a `v*` tag.

### Local build (same binaries, manual upload)

```bash
npm install
npm run tauri build
```

Outputs:

```text
src-tauri/target/release/bundle/macos/Samsmrti.app
src-tauri/target/release/bundle/dmg/Samsmrti_<version>_aarch64.dmg
```

Optional zip for sharing:

```bash
VERSION=$(node -p "require('./src-tauri/tauri.conf.json').version")
mkdir -p dist-release
ditto -c -k --sequesterRsrc --keepParent \
  src-tauri/target/release/bundle/macos/Samsmrti.app \
  "dist-release/Samsmrti-${VERSION}-macos.zip"
cp src-tauri/target/release/bundle/dmg/*.dmg dist-release/
```

`dist-release/` is gitignored — do not commit binaries. Upload manually via **Releases → Draft a new release** if not using the tag workflow.

With [GitHub CLI](https://cli.github.com/) installed:

```bash
gh release create v0.2.0 dist-release/* \
  --title "Samsmrti v0.2.0 (macOS)" \
  --notes "macOS app only; no user database included."
```

### App icon

Desktop icons live in `src-tauri/icons/` (`.icns`, `.ico`, PNG sizes). The web favicon is `public/icon.svg` (with `public/favicon.png` for Apple touch).

To regenerate from a new **square** PNG (1024×1024 recommended):

```bash
npm run tauri icon path/to/icon.png -o src-tauri/icons
```

Dock icons look best with ~20% transparent margin around the artwork (same visual weight as Cursor). When updating the master PNG, scale the graphic to **80%** and center it on a square canvas before running `tauri icon`.

After changing icons you must **recompile the Rust app** (icons are baked in at build time). Quit Samsmrti, then:

```bash
npm run tauri dev
```

A plain quit/reopen without recompiling will keep the old dock icon. For a packaged `.app`, run `npm run tauri build` and open the bundle under `src-tauri/target/release/bundle/`.

## Related docs

- **[QUICKSTART.md](./QUICKSTART.md)** — user-facing: profiles, export/import, features
- **[README.md](./README.md)** — project overview

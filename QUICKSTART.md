# Samsmrti — Quickstart

Samsmrti is a desktop spaced-repetition app (Tauri + React + SQLite) with decks, a knowledge graph, multi-profile study on one machine, and optional Karma gamification.

## Run the app

```bash
npm install
npm run tauri dev
```

To package the app for distribution, see **[DEVELOPMENT.md](./DEVELOPMENT.md#bundling-for-distribution)**.

## First steps

1. Pick a **profile** in the sidebar (honor system — switch when someone else uses the machine).
2. Open a deck → **Study Now** → rate cards with **1–4** (Again / Hard / Good / Easy).
3. **Add Cards** to create notes; use **Browse Cards** to search.
4. Open **Help** in the app for the full in-app guide.

Example decks (Science, Math, History, Geography) and **Chemistry → Polyatomic Ions** are seeded on first launch.

## Profiles & Karma

- **Admin** profile: can study everything but earns **no Karma**.
- Other profiles: separate **card scheduling** (`card_progress` per profile) and **Karma** ($0.10/review, $0.20/add, streak bonuses).
- Manage profiles under **Settings → Profiles**.

## Data: two export modes

| Mode | Menu / Settings | File | Includes | Use case |
|------|-----------------|------|----------|----------|
| **Content export/import** | File → Export Content / Import Content | `.json` (pretty JSON) | Decks, note types, notes, cards (as *new*), tags, knowledge graph | Share collection with someone who starts **fresh** — no review history, profiles, or karma |
| **Full backup / restore** | File → Backup (Full) / Restore (Full) | `.samsmrti-backup` (zip) | SQLite DB + `media/` folder — scheduling, review log, profiles, karma, settings | **Move to another computer** or disaster recovery |

### Content export (`samsmrti-content-v1`)

- **Export:** File → **Export Content…** or Settings → **Export content (.json)**.
- **Import:** File → **Import Content…** or Settings → **Import content**.
- Merges by ID: existing rows are kept; only new IDs are added.
- Imported cards are **`new`** for every profile (progress is seeded automatically).
- Does **not** include images in the file — copy the app **media** folder separately if notes reference local images (see below).
- Legacy **`.json.gz`** exports (`samsmrti-backup-v1`) can be imported the same way.

### Full backup (`samsmrti-full-v1`)

- **Backup:** File → **Backup (Full)…** — writes a zip containing `samsmrti.db`, `media/`, and `manifest.json`.
- **Restore:** File → **Restore (Full)…** — **replaces** the local database and media. Your current DB is copied to `samsmrti.db.pre-restore-<timestamp>` in the app data folder first.
- After restore, reload the UI (decks/profiles refresh automatically).

### Single-deck JSON

Settings → **Export Deck** saves one deck (notes only, `samsmrti-v1` format) for small shares.

## App data locations

| Platform | Path |
|----------|------|
| macOS | `~/Library/Application Support/com.samsmrti.desktop/` |
| Linux | `~/.local/share/com.samsmrti.desktop/` (or XDG data dir) |
| Windows | `%APPDATA%\com.samsmrti.desktop\` |

Contains:

- `samsmrti.db` — all structured data
- `media/` — images and other imported media files

To **manually** move the app with full state: quit Samsmrti, zip the whole folder above, copy to the same path on another machine (same OS profile), or use **Backup (Full)** / **Restore (Full)** inside the app.

## Import from other apps

**Sidebar → Import Deck:**

- **Anki** `.apkg` or live `collection.anki2` (quit Anki first for the collection file)
- **Mochi** `.mochi`

Imports deck hierarchy, notes, cards, tags, and media. Scheduling is written to the database; new profiles still get their own `card_progress` rows when they study.

## Main features (short)

| Feature | Where |
|---------|--------|
| Spaced repetition (FSRS-style) | Study session, per-profile queues |
| Subdecks | Deck settings → parent deck; parent **Study** includes children |
| Cloze & reversed cards | Note types; Polyatomic Ions uses Basic (and reversed) |
| Knowledge graph (entities, triples) | **Knowledge Map**; ERE review by entity |
| Search & tags | **Browse Cards** |
| Bury / undo / edit in review | Keyboard shortcuts in Help |
| Stats & Karma | **Stats** page |

## Keyboard shortcuts (review)

| Key | Action |
|-----|--------|
| Space | Show answer |
| 1–4 | Rate card |
| 9 | Bury |
| U / Ctrl+Z | Undo |
| E | Edit note |
| Esc | End session |

## See also

- **[DEVELOPMENT.md](./DEVELOPMENT.md)** — dev setup, tests, bundling for distribution
- In-app **Help** (`/help`) — detailed topics including import, graph, and review behavior

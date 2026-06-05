CREATE TABLE IF NOT EXISTS decks (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    parent_id   TEXT REFERENCES decks(id) ON DELETE SET NULL,
    description TEXT NOT NULL DEFAULT '',
    new_per_day INTEGER NOT NULL DEFAULT 20,
    max_reviews INTEGER NOT NULL DEFAULT 200,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS note_types (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    css        TEXT NOT NULL DEFAULT '',
    is_cloze   INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS fields (
    id           TEXT PRIMARY KEY,
    note_type_id TEXT NOT NULL REFERENCES note_types(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    ordinal      INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS card_templates (
    id           TEXT PRIMARY KEY,
    note_type_id TEXT NOT NULL REFERENCES note_types(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    front_html   TEXT NOT NULL,
    back_html    TEXT NOT NULL,
    ordinal      INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS notes (
    id           TEXT PRIMARY KEY,
    deck_id      TEXT NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
    note_type_id TEXT NOT NULL REFERENCES note_types(id),
    fields_json  TEXT NOT NULL DEFAULT '{}',
    created_at   INTEGER NOT NULL,
    updated_at   INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS cards (
    id               TEXT PRIMARY KEY,
    note_id          TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    template_ordinal INTEGER NOT NULL DEFAULT 0,
    state            TEXT NOT NULL DEFAULT 'new',
    difficulty       REAL NOT NULL DEFAULT 0.0,
    stability        REAL NOT NULL DEFAULT 0.0,
    due_at           INTEGER NOT NULL DEFAULT 0,
    last_review_at   INTEGER,
    reps             INTEGER NOT NULL DEFAULT 0,
    lapses           INTEGER NOT NULL DEFAULT 0,
    buried_until     INTEGER
);

CREATE TABLE IF NOT EXISTS review_log (
    id             TEXT PRIMARY KEY,
    card_id        TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
    profile_id     TEXT,
    reviewed_at    INTEGER NOT NULL,
    rating         INTEGER NOT NULL,
    elapsed_ms     INTEGER NOT NULL DEFAULT 0,
    scheduled_days REAL NOT NULL DEFAULT 0.0,
    state_before   TEXT NOT NULL DEFAULT '',
    state_after    TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS tags (
    id   TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS note_tags (
    note_id TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    tag_id  TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (note_id, tag_id)
);

CREATE TABLE IF NOT EXISTS card_flags (
    card_id     TEXT PRIMARY KEY REFERENCES cards(id) ON DELETE CASCADE,
    flagged_at  INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS note_links (
    id             TEXT PRIMARY KEY,
    source_note_id TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    target_note_id TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    created_at     INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_notes_deck ON notes(deck_id);
CREATE INDEX IF NOT EXISTS idx_cards_note ON cards(note_id);
CREATE INDEX IF NOT EXISTS idx_cards_due ON cards(state, due_at);
CREATE INDEX IF NOT EXISTS idx_review_log_card ON review_log(card_id);
CREATE INDEX IF NOT EXISTS idx_note_links_source ON note_links(source_note_id);
CREATE INDEX IF NOT EXISTS idx_note_links_target ON note_links(target_note_id);
CREATE INDEX IF NOT EXISTS idx_decks_parent ON decks(parent_id);

CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
    note_id UNINDEXED,
    content,
    content_rowid='rowid'
);

-- Entity-Relation Graph tables
CREATE TABLE IF NOT EXISTS entities (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    entity_type TEXT,
    description TEXT NOT NULL DEFAULT '',
    created_at  INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_entities_type_name ON entities(entity_type, name);

CREATE TABLE IF NOT EXISTS relation_types (
    id           TEXT PRIMARY KEY,
    name         TEXT NOT NULL,
    inverse_name TEXT,
    created_at   INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS triples (
    id               TEXT PRIMARY KEY,
    subject_id       TEXT NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    relation_type_id TEXT NOT NULL REFERENCES relation_types(id) ON DELETE CASCADE,
    object_id        TEXT NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    created_at       INTEGER NOT NULL,
    UNIQUE(subject_id, relation_type_id, object_id)
);
CREATE INDEX IF NOT EXISTS idx_triples_subject ON triples(subject_id);
CREATE INDEX IF NOT EXISTS idx_triples_object ON triples(object_id);
CREATE INDEX IF NOT EXISTS idx_triples_relation ON triples(relation_type_id);

CREATE TABLE IF NOT EXISTS card_triples (
    card_id   TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
    triple_id TEXT NOT NULL REFERENCES triples(id) ON DELETE CASCADE,
    PRIMARY KEY (card_id, triple_id)
);
CREATE INDEX IF NOT EXISTS idx_card_triples_triple ON card_triples(triple_id);

-- Profiles and Karma (honor-system multi-user on one machine)
CREATE TABLE IF NOT EXISTS profiles (
    id           TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    is_admin     INTEGER NOT NULL DEFAULT 0,
    created_at   INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS card_progress (
    profile_id       TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    card_id          TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
    state            TEXT NOT NULL DEFAULT 'new',
    difficulty       REAL NOT NULL DEFAULT 0.0,
    stability        REAL NOT NULL DEFAULT 0.0,
    due_at           INTEGER NOT NULL DEFAULT 0,
    last_review_at   INTEGER,
    reps             INTEGER NOT NULL DEFAULT 0,
    lapses           INTEGER NOT NULL DEFAULT 0,
    buried_until     INTEGER,
    PRIMARY KEY (profile_id, card_id)
);

CREATE TABLE IF NOT EXISTS app_settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS karma_state (
    profile_id            TEXT PRIMARY KEY REFERENCES profiles(id) ON DELETE CASCADE,
    balance_cents         INTEGER NOT NULL DEFAULT 0,
    last_streak_bonus_at  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS karma_daily (
    profile_id     TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    day            TEXT NOT NULL,
    active_seconds INTEGER NOT NULL DEFAULT 0,
    review_count   INTEGER NOT NULL DEFAULT 0,
    add_count      INTEGER NOT NULL DEFAULT 0,
    qualified      INTEGER NOT NULL DEFAULT 0,
    earned_cents   INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (profile_id, day)
);

CREATE INDEX IF NOT EXISTS idx_karma_daily_profile_day ON karma_daily(profile_id, day);
CREATE INDEX IF NOT EXISTS idx_card_progress_due ON card_progress(profile_id, state, due_at);

CREATE TABLE IF NOT EXISTS entity_tags (
    entity_id TEXT NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    tag_id    TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (entity_id, tag_id)
);

CREATE TABLE IF NOT EXISTS relation_type_tags (
    relation_type_id TEXT NOT NULL REFERENCES relation_types(id) ON DELETE CASCADE,
    tag_id           TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (relation_type_id, tag_id)
);

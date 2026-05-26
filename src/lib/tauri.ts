import { invoke } from "@tauri-apps/api/core";

export interface Deck {
  id: string;
  name: string;
  parent_id: string | null;
  description: string;
  new_per_day: number;
  max_reviews: number;
  created_at: number;
  updated_at: number;
}

export interface DeckWithCounts extends Deck {
  total_cards: number;
  due_cards: number;
  new_cards: number;
}

export interface NoteType {
  id: string;
  name: string;
  css: string;
  is_cloze: boolean;
  created_at: number;
  fields: Field[];
  templates: CardTemplate[];
}

export interface NoteTypeDeckRef {
  deck_id: string;
  deck_name: string;
  note_count: number;
}

export interface NoteTypeUsageSummary {
  note_type_id: string;
  note_count: number;
  card_count: number;
  deck_count: number;
  top_decks: NoteTypeDeckRef[];
}

export interface Field {
  id: string;
  note_type_id: string;
  name: string;
  ordinal: number;
}

export interface CardTemplate {
  id: string;
  note_type_id: string;
  name: string;
  front_html: string;
  back_html: string;
  ordinal: number;
}

export interface Note {
  id: string;
  deck_id: string;
  note_type_id: string;
  fields_json: Record<string, string>;
  created_at: number;
  updated_at: number;
  tags: string[];
}

export interface Card {
  id: string;
  note_id: string;
  template_ordinal: number;
  state: string;
  difficulty: number;
  stability: number;
  due_at: number;
  last_review_at: number | null;
  reps: number;
  lapses: number;
}

export interface ReviewCard {
  card_id: string;
  note_id: string;
  note_type_id: string;
  is_cloze: boolean;
  template_ordinal: number;
  front_html: string;
  back_html: string;
  fields: Record<string, string>;
  state: string;
  css: string;
}

export interface KarmaEarnEvent {
  earned_cents: number;
  balance_cents: number;
  streak_days: number;
  bonus_awarded_cents: number;
  qualified_today: boolean;
}

export interface KarmaOverview {
  balance_cents: number;
  streak_days: number;
  qualified_today: boolean;
  today_active_seconds: number;
  today_effective_actions: number;
  daily_qualified: { day: string; qualified: boolean }[];
  profile_id: string;
  is_admin: boolean;
}

export interface Profile {
  id: string;
  display_name: string;
  is_admin: boolean;
  created_at: number;
}

export interface AnswerResult {
  card_id: string;
  new_state: string;
  due_at: number;
  stability: number;
  difficulty: number;
  review_log_id: string;
  karma: KarmaEarnEvent;
}

export interface CreateNoteResult {
  note: Note;
  karma: KarmaEarnEvent;
}

export interface UndoReviewResult {
  karma: KarmaEarnEvent;
}

export interface IntervalPreview {
  again: string;
  hard: string;
  good: string;
  easy: string;
}

export interface ReviewStats {
  reviewed_today: number;
  again_count: number;
  hard_count: number;
  good_count: number;
  easy_count: number;
  total_time_ms: number;
}

export interface DeletedCardSnapshot {
  id: string;
  note_id: string;
  template_ordinal: number;
  state: string;
  difficulty: number;
  stability: number;
  due_at: number;
  last_review_at: number | null;
  reps: number;
  lapses: number;
  buried_until: number | null;
  triple_ids: string[];
  review_logs: {
    id: string;
    reviewed_at: number;
    rating: number;
    elapsed_ms: number;
    scheduled_days: number;
    state_before: string;
    state_after: string;
  }[];
}

export interface BuriedCard {
  card_id: string;
  note_id: string;
  deck_id: string;
  deck_name: string;
  front_html: string;
  fields: Record<string, string>;
  state: string;
  buried_until: number;
}

export interface CreateNoteTypeInput {
  name: string;
  is_cloze: boolean;
  css?: string;
  fields: string[];
  templates: { name: string; front_html: string; back_html: string }[];
}

// ── Entity-Relation Graph types ───────────────────────────────────────────

export interface Entity {
  id: string;
  name: string;
  entity_type: string | null;
  description: string;
  created_at: number;
  tags: string[];
}

export interface RelationType {
  id: string;
  name: string;
  inverse_name: string | null;
  created_at: number;
  tags: string[];
}

export interface Triple {
  id: string;
  subject_id: string;
  relation_type_id: string;
  object_id: string;
  created_at: number;
  subject_name: string;
  relation_name: string;
  object_name: string;
}

export interface MindMapNode {
  id: string;
  label: string;
  node_type: string;
  entity_type: string | null;
  card_status: string;
  triple_count: number;
}

export interface MindMapEdge {
  source: string;
  target: string;
}

export interface MindMapData {
  nodes: MindMapNode[];
  edges: MindMapEdge[];
}

export interface CardSuggestion {
  triple_ids: string[];
  front: string;
  back: string;
  suggestion_type: string;
}

export interface EreReviewEntity {
  entity_id: string;
  entity_name: string;
  due_count: number;
}

export const api = {
  getDecks: () => invoke<DeckWithCounts[]>("get_decks"),

  createDeck: (input: {
    name: string;
    parent_id?: string | null;
    description?: string;
  }) => invoke<Deck>("create_deck", { input }),

  updateDeck: (input: {
    id: string;
    name?: string;
    parent_id?: string | null;
    description?: string;
    new_per_day?: number;
    max_reviews?: number;
  }) => invoke<Deck>("update_deck", { input }),

  deleteDeck: (id: string) => invoke<void>("delete_deck", { id }),

  getNoteTypes: () => invoke<NoteType[]>("get_note_types"),

  createNote: (input: {
    deck_id: string;
    note_type_id: string;
    fields: Record<string, string>;
    tags?: string[];
  }) => invoke<CreateNoteResult>("create_note", { input }),

  getNotes: (deckId: string) => invoke<Note[]>("get_notes", { deckId }),

  getNoteTags: (noteId: string) => invoke<string[]>("get_note_tags", { noteId }),

  updateNote: (input: {
    id: string;
    deck_id?: string;
    fields?: Record<string, string>;
    tags?: string[];
  }) => invoke<Note>("update_note", { input }),

  deleteNote: (id: string) => invoke<void>("delete_note", { id }),

  getCardsForNote: (noteId: string) =>
    invoke<Card[]>("get_cards_for_note", { noteId }),

  getReviewQueue: (deckId: string) =>
    invoke<ReviewCard[]>("get_review_queue", { deckId }),

  answerCard: (input: {
    card_id: string;
    rating: number;
    elapsed_ms: number;
  }) => invoke<AnswerResult>("answer_card", { input }),

  undoReview: (reviewLogId: string) =>
    invoke<UndoReviewResult>("undo_review", { reviewLogId }),

  getIntervalPreview: (cardId: string) =>
    invoke<IntervalPreview>("get_interval_preview", { cardId }),

  getReviewStats: (deckId: string) =>
    invoke<ReviewStats>("get_review_stats", { deckId }),

  buryCard: (cardId: string) =>
    invoke<number>("bury_card", { cardId }),

  unburyCard: (cardId: string) =>
    invoke<void>("unbury_card", { cardId }),

  getBuriedCards: (query?: string, deckId?: string, limit?: number) =>
    invoke<BuriedCard[]>("get_buried_cards", { query, deckId, limit }),

  deleteCard: (cardId: string) =>
    invoke<DeletedCardSnapshot>("delete_card", { cardId }),

  restoreCard: (snapshot: DeletedCardSnapshot) =>
    invoke<void>("restore_card", { snapshot }),

  searchNotes: (
    query: string,
    deckId?: string,
    tag?: string,
    noteTypeId?: string,
    limit?: number,
    offset?: number
  ) =>
    invoke<{
      note_id: string;
      deck_id: string;
      deck_name: string;
      note_type_id: string;
      note_type_name: string;
      fields_json: Record<string, string>;
      tags: string[];
      card_count: number;
      created_at: number;
    }[]>("search_notes", {
      query,
      deckId,
      tag,
      noteTypeId,
      limit,
      offset,
    }),

  getNoteTypeUsage: () =>
    invoke<NoteTypeUsageSummary[]>("get_note_type_usage"),

  rebuildSearchIndex: () => invoke<void>("rebuild_search_index"),

  /** Builds FTS index only if missing (fast when already indexed). */
  ensureSearchIndex: () => invoke<boolean>("ensure_search_index"),

  getAllTags: () =>
    invoke<[string, string, number][]>("get_all_tags"),

  getStatsOverview: () =>
    invoke<{
      total_cards: number;
      new_cards: number;
      learning_cards: number;
      review_cards: number;
      total_decks: number;
      total_reviews_today: number;
      streak_days: number;
      daily_reviews: {
        date: string;
        count: number;
        again: number;
        hard: number;
        good: number;
        easy: number;
      }[];
    }>("get_stats_overview"),

  // ── Entity CRUD ──────────────────────────────────────────────────────────

  createEntity: (input: { name: string; entity_type?: string; description?: string; tags?: string[] }) =>
    invoke<Entity>("create_entity", { input }),

  getEntities: (search?: string, entityType?: string, tag?: string) =>
    invoke<Entity[]>("get_entities", { search, entityType, tag }),

  updateEntity: (input: { id: string; name?: string; entity_type?: string; description?: string; tags?: string[] }) =>
    invoke<Entity>("update_entity", { input }),

  deleteEntity: (id: string) => invoke<void>("delete_entity", { id }),

  // ── Relation Type CRUD ─────────────────────────────────────────────────

  createRelationType: (input: { name: string; inverse_name?: string; tags?: string[] }) =>
    invoke<RelationType>("create_relation_type", { input }),

  getRelationTypes: (tag?: string) =>
    invoke<RelationType[]>("get_relation_types", { tag }),

  deleteRelationType: (id: string) => invoke<void>("delete_relation_type", { id }),

  // ── Triple CRUD ────────────────────────────────────────────────────────

  createTriple: (input: { subject_id: string; relation_type_id: string; object_id: string }) =>
    invoke<Triple>("create_triple", { input }),

  batchCreateTriples: (input: { subject_id: string; relation_type_id: string; object_ids: string[] }) =>
    invoke<Triple[]>("batch_create_triples", { input }),

  getTriples: (subjectId?: string, objectId?: string, relationTypeId?: string, entityId?: string) =>
    invoke<Triple[]>("get_triples", { subjectId, objectId, relationTypeId, entityId }),

  updateTriple: (input: { id: string; subject_id?: string; relation_type_id?: string; object_id?: string }) =>
    invoke<Triple>("update_triple", { input }),

  deleteTriple: (id: string) => invoke<void>("delete_triple", { id }),

  // ── Card-Triple linking ────────────────────────────────────────────────

  linkCardToTriple: (cardId: string, tripleId: string) =>
    invoke<void>("link_card_to_triple", { cardId, tripleId }),

  unlinkCardFromTriple: (cardId: string, tripleId: string) =>
    invoke<void>("unlink_card_from_triple", { cardId, tripleId }),

  getTriplesForCard: (cardId: string) =>
    invoke<Triple[]>("get_triples_for_card", { cardId }),

  getCardsForTriple: (tripleId: string) =>
    invoke<string[]>("get_cards_for_triple", { tripleId }),

  // ── Mindmap ────────────────────────────────────────────────────────────

  getMindmap: (entityId: string) =>
    invoke<MindMapData>("get_mindmap", { entityId }),

  // ── Card suggestions ──────────────────────────────────────────────────

  suggestCardsFromTriples: (entityId?: string) =>
    invoke<CardSuggestion[]>("suggest_cards_from_triples", { entityId }),

  // ── E-R-E Review ──────────────────────────────────────────────────────

  getEreDueCards: (deckId: string, entityId?: string) =>
    invoke<ReviewCard[]>("get_ere_due_cards", { deckId, entityId }),

  getEreReviewSummary: (deckId: string) =>
    invoke<EreReviewEntity[]>("get_ere_review_summary", { deckId }),

  importFile: (filePath: string) =>
    invoke<{
      decks_imported: number;
      notes_imported: number;
      cards_imported: number;
      media_imported: number;
      warnings: string[];
    }>("import_file", { filePath }),

  exportDeckJson: (deckId: string, filePath: string) =>
    invoke<{ path: string; notes_exported: number; cards_exported: number }>(
      "export_deck_json",
      { deckId, filePath }
    ),

  exportAllGz: (filePath: string) =>
    invoke<{
      path: string;
      bytes_written: number;
      decks: number;
      notes: number;
      cards: number;
      entities: number;
      triples: number;
    }>("export_all_gz", { filePath }),

  exportContentJson: (filePath: string) =>
    invoke<{
      path: string;
      decks: number;
      notes: number;
      cards: number;
      entities: number;
      triples: number;
    }>("export_content_json", { filePath }),

  importContentJson: (filePath: string) =>
    invoke<{
      decks_added: number;
      notes_added: number;
      cards_added: number;
      entities_added: number;
      triples_added: number;
      rows_skipped: number;
      warnings: string[];
    }>("import_content_json", { filePath }),

  exportFullBackup: (filePath: string) =>
    invoke<{
      path: string;
      bytes_written: number;
      media_files: number;
    }>("export_full_backup", { filePath }),

  restoreFullBackup: (filePath: string) =>
    invoke<{
      decks: number;
      notes: number;
      cards: number;
      profiles: number;
      media_files_restored: number;
      previous_db_backup: string | null;
    }>("restore_full_backup", { filePath }),

  createNoteType: (input: CreateNoteTypeInput) =>
    invoke<string>("create_note_type", { input }),

  updateNoteType: (input: {
    id: string;
    name?: string;
    css?: string;
    fields?: string[];
    templates?: { name: string; front_html: string; back_html: string }[];
  }) => invoke<void>("update_note_type", { input }),

  deleteNoteType: (id: string) =>
    invoke<void>("delete_note_type", { id }),

  listProfiles: () => invoke<Profile[]>("list_profiles"),

  getActiveProfile: () => invoke<Profile>("get_active_profile"),

  setActiveProfile: (profileId: string) =>
    invoke<Profile>("set_active_profile", { profileId }),

  createProfile: (displayName: string) =>
    invoke<Profile>("create_profile", { displayName }),

  deleteProfile: (profileId: string) =>
    invoke<void>("delete_profile", { profileId }),

  getKarmaOverview: () => invoke<KarmaOverview>("get_karma_overview"),

  recordActivity: (seconds: number) =>
    invoke<KarmaOverview>("record_activity", { seconds }),
};

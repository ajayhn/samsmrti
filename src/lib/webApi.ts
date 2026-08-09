// PWA implementation of the same `api` interface src/lib/tauri.ts exposes on
// desktop, backed by the wasm core running in a dedicated Worker instead of
// Tauri's invoke(). Review-only scope: covers decks, the review loop, karma,
// profiles, stats, and content-level manual sync -- not note/note-type/graph
// authoring or .apkg/Mochi import (desktop-only, see streamed-yawning-coral.md).
import { callDb } from "./dbWorkerClient";
import type {
  AnswerResult,
  BuriedCard,
  Card,
  ContentDeckOption,
  Deck,
  DeckWithCounts,
  DeletedCardSnapshot,
  DeletedDeckSnapshot,
  IntervalPreview,
  KarmaOverview,
  Profile,
  ReviewCard,
  ReviewStats,
  UndoReviewResult,
} from "./tauri";

export type {
  AnswerResult,
  BuriedCard,
  Card,
  ContentDeckOption,
  Deck,
  DeckWithCounts,
  DeletedCardSnapshot,
  DeletedDeckSnapshot,
  IntervalPreview,
  KarmaOverview,
  Profile,
  ReviewCard,
  ReviewStats,
  UndoReviewResult,
};

export interface ContentExportResult {
  content: string;
  decks: number;
  notes: number;
  cards: number;
  entities: number;
  triples: number;
}

export interface ContentImportSummary {
  decks_added: number;
  notes_added: number;
  cards_added: number;
  entities_added: number;
  triples_added: number;
  rows_skipped: number;
  warnings: string[];
}

// Named `api` (not `webApi`) so it can be aliased in for "../lib/tauri" /
// "../../lib/tauri" imports (see vite.web.config.ts) without touching every
// store/component that already imports { api } from "../lib/tauri".
export const api = {
  getDecks: () => callDb<string>("getDecks").then((s) => JSON.parse(s) as DeckWithCounts[]),

  createDeck: (input: { name: string; parent_id?: string | null; description?: string }) =>
    callDb<string>("createDeck", [JSON.stringify(input)]).then((s) => JSON.parse(s) as Deck),

  updateDeck: (input: {
    id: string;
    name?: string;
    parent_id?: string | null;
    description?: string;
    new_per_day?: number;
    max_reviews?: number;
  }) => callDb<string>("updateDeck", [JSON.stringify(input)]).then((s) => JSON.parse(s) as Deck),

  deleteDeck: (id: string) =>
    callDb<string>("deleteDeck", [id]).then((s) => JSON.parse(s) as DeletedDeckSnapshot),

  restoreDeletedDeck: (snapshot: DeletedDeckSnapshot) =>
    callDb<void>("restoreDeletedDeck", [JSON.stringify(snapshot)]),

  getReviewQueue: (deckId: string) =>
    callDb<string>("getReviewQueue", [deckId]).then((s) => JSON.parse(s) as ReviewCard[]),

  answerCard: (input: { card_id: string; rating: number; elapsed_ms: number }) =>
    callDb<string>("answerCard", [JSON.stringify(input)]).then((s) => JSON.parse(s) as AnswerResult),

  undoReview: (reviewLogId: string) =>
    callDb<string>("undoReview", [reviewLogId]).then((s) => JSON.parse(s) as UndoReviewResult),

  getIntervalPreview: (cardId: string) =>
    callDb<string>("getIntervalPreview", [cardId]).then((s) => JSON.parse(s) as IntervalPreview),

  getReviewStats: (deckId: string) =>
    callDb<string>("getReviewStats", [deckId]).then((s) => JSON.parse(s) as ReviewStats),

  buryCard: (cardId: string) => callDb<number>("buryCard", [cardId]),

  unburyCard: (cardId: string) => callDb<void>("unburyCard", [cardId]),

  getBuriedCards: (query?: string, deckId?: string, limit?: number) =>
    callDb<string>("getBuriedCards", [query ?? null, deckId ?? null, limit ?? null]).then(
      (s) => JSON.parse(s) as BuriedCard[]
    ),

  deleteCard: (cardId: string) =>
    callDb<string>("deleteCard", [cardId]).then((s) => JSON.parse(s) as DeletedCardSnapshot),

  restoreCard: (snapshot: DeletedCardSnapshot) =>
    callDb<void>("restoreCard", [JSON.stringify(snapshot)]),

  listProfiles: () => callDb<string>("listProfiles").then((s) => JSON.parse(s) as Profile[]),

  getActiveProfile: () => callDb<string>("getActiveProfile").then((s) => JSON.parse(s) as Profile),

  setActiveProfile: (profileId: string) =>
    callDb<string>("setActiveProfile", [profileId]).then((s) => JSON.parse(s) as Profile),

  createProfile: (displayName: string) =>
    callDb<string>("createProfile", [displayName]).then((s) => JSON.parse(s) as Profile),

  deleteProfile: (profileId: string) => callDb<void>("deleteProfile", [profileId]),

  getKarmaOverview: () => callDb<string>("getKarmaOverview").then((s) => JSON.parse(s) as KarmaOverview),

  recordActivity: (seconds: number) =>
    callDb<string>("recordActivity", [seconds]).then((s) => JSON.parse(s) as KarmaOverview),

  getStatsOverview: () =>
    callDb<string>("getStatsOverview").then(
      (s) =>
        JSON.parse(s) as {
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
        }
    ),

  // Content-level manual sync. Unlike native (file paths via Tauri's dialog
  // plugin), the browser has no filesystem: content moves as plain strings,
  // and the caller is responsible for the actual file picker / download.
  listContentExportDecks: () =>
    callDb<string>("listContentExportDecks").then((s) => JSON.parse(s) as ContentDeckOption[]),

  previewContentImport: (contentJson: string) =>
    callDb<string>("previewContentImport", [contentJson]).then((s) => JSON.parse(s) as ContentDeckOption[]),

  exportContentJson: (deckIds?: string[]) =>
    callDb<string>("exportContentJson", [deckIds ? JSON.stringify(deckIds) : null]).then(
      (s) => JSON.parse(s) as ContentExportResult
    ),

  importContentJson: (contentJson: string, deckIds?: string[]) =>
    callDb<string>("importContentJson", [contentJson, deckIds ? JSON.stringify(deckIds) : null]).then(
      (s) => JSON.parse(s) as ContentImportSummary
    ),
};

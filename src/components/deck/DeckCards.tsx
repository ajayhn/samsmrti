import { useCallback, useEffect, useMemo, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { api, type NoteType } from "../../lib/tauri";
import { useDeckStore } from "../../stores/deckStore";
import {
  NoteResultList,
  notePreviewText,
  type NoteSearchResult,
} from "../search/NoteResultList";

type SortMode = "added" | "alpha";

type SortState = {
  mode: SortMode;
  ascending: boolean;
};

const SORT_STORAGE_KEY = "deck-cards-sort-v2";

function loadSortState(): SortState {
  try {
    const raw = localStorage.getItem(SORT_STORAGE_KEY);
    if (!raw) return { mode: "added", ascending: true };
    const parsed = JSON.parse(raw) as SortState;
    if (parsed.mode !== "added" && parsed.mode !== "alpha") {
      return { mode: "added", ascending: true };
    }
    return { mode: parsed.mode, ascending: parsed.ascending !== false };
  } catch {
    return { mode: "added", ascending: true };
  }
}

function saveSortState(state: SortState) {
  localStorage.setItem(SORT_STORAGE_KEY, JSON.stringify(state));
}

export function DeckCards() {
  const { deckId } = useParams<{ deckId: string }>();
  const navigate = useNavigate();
  const { decks, fetchDecks, selectDeck } = useDeckStore();
  const deck = decks.find((d) => d.id === deckId);

  const [results, setResults] = useState<NoteSearchResult[]>([]);
  const [noteTypes, setNoteTypes] = useState<NoteType[]>([]);
  const [loading, setLoading] = useState(true);
  const [expanded, setExpanded] = useState<string | null>(null);
  const [sort, setSort] = useState<SortState>(loadSortState);
  const [busyNoteId, setBusyNoteId] = useState<string | null>(null);

  const loadNotes = useCallback(async () => {
    if (!deckId) return;
    setLoading(true);
    try {
      const [notes, types] = await Promise.all([
        api.searchNotes("", deckId, undefined, undefined, 10000),
        api.getNoteTypes(),
      ]);
      setResults(notes);
      setNoteTypes(types);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  }, [deckId]);

  useEffect(() => {
    fetchDecks();
  }, [fetchDecks]);

  useEffect(() => {
    if (deckId) selectDeck(deckId);
  }, [deckId, selectDeck]);

  useEffect(() => {
    void loadNotes();
  }, [loadNotes]);

  const sortedResults = useMemo(() => {
    const copy = [...results];
    const dir = sort.ascending ? 1 : -1;
    if (sort.mode === "added") {
      copy.sort((a, b) => (a.created_at - b.created_at) * dir);
    } else {
      copy.sort(
        (a, b) =>
          notePreviewText(a.fields_json).localeCompare(
            notePreviewText(b.fields_json),
            undefined,
            { sensitivity: "base" }
          ) * dir
      );
    }
    return copy;
  }, [results, sort]);

  const updateSort = (next: SortState) => {
    setSort(next);
    saveSortState(next);
  };

  const handleDeleteNote = async (noteId: string) => {
    const note = results.find((r) => r.note_id === noteId);
    const preview = note ? notePreviewText(note.fields_json).slice(0, 80) : "this note";
    if (
      !confirm(
        `Delete this note and all ${note?.card_count ?? ""} card(s)?\n\n"${preview}"`
      )
    ) {
      return;
    }
    setBusyNoteId(noteId);
    try {
      await api.deleteNote(noteId);
      setResults((prev) => prev.filter((r) => r.note_id !== noteId));
      if (expanded === noteId) setExpanded(null);
      await fetchDecks();
    } catch (e) {
      console.error(e);
      alert("Failed to delete note.");
    } finally {
      setBusyNoteId(null);
    }
  };

  const handleBuryNote = async (noteId: string) => {
    setBusyNoteId(noteId);
    try {
      const cards = await api.getCardsForNote(noteId);
      await Promise.all(cards.map((c) => api.buryCard(c.id)));
      setResults((prev) => prev.filter((r) => r.note_id !== noteId));
      if (expanded === noteId) setExpanded(null);
      await fetchDecks();
    } catch (e) {
      console.error(e);
      alert("Failed to bury cards.");
    } finally {
      setBusyNoteId(null);
    }
  };

  if (!deckId) {
    return null;
  }

  return (
    <div className="h-full flex flex-col p-6">
      <div className="flex items-start justify-between gap-4 mb-4">
        <div>
          <button
            type="button"
            onClick={() => navigate("/")}
            className="inline-flex items-center gap-1 text-sm text-primary-500 hover:underline font-medium cursor-pointer mb-2"
          >
            <svg
              className="w-4 h-4 rotate-180"
              viewBox="0 0 20 20"
              fill="currentColor"
              aria-hidden
            >
              <path
                fillRule="evenodd"
                d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z"
                clipRule="evenodd"
              />
            </svg>
            {deck?.name ?? "Deck"}
          </button>
          <h2 className="text-xl font-bold text-text">Cards</h2>
          <p className="text-sm text-text-muted mt-0.5">
            {loading
              ? "Loading…"
              : `${sortedResults.length} note${sortedResults.length !== 1 ? "s" : ""} in this deck and subdecks`}
          </p>
        </div>

        <div className="flex items-center gap-1.5 shrink-0">
          <div className="flex rounded-lg border border-border overflow-hidden text-sm">
            <button
              type="button"
              onClick={() => updateSort({ mode: "alpha", ascending: true })}
              className={`px-3 py-1.5 transition-colors cursor-pointer ${
                sort.mode === "alpha"
                  ? "bg-primary-100 text-primary-700 dark:bg-primary-900/30 dark:text-primary-300"
                  : "text-text-secondary hover:bg-surface-hover"
              }`}
            >
              A–Z
            </button>
            <button
              type="button"
              onClick={() =>
                updateSort({
                  mode: "added",
                  ascending: sort.mode === "added" ? sort.ascending : true,
                })
              }
              className={`px-3 py-1.5 transition-colors cursor-pointer border-l border-border ${
                sort.mode === "added"
                  ? "bg-primary-100 text-primary-700 dark:bg-primary-900/30 dark:text-primary-300"
                  : "text-text-secondary hover:bg-surface-hover"
              }`}
            >
              Add order
            </button>
          </div>

          {sort.mode === "added" && (
            <div className="flex rounded-lg border border-border overflow-hidden text-sm">
              <button
                type="button"
                onClick={() => updateSort({ mode: "added", ascending: true })}
                title="Oldest first"
                className={`px-2.5 py-1.5 transition-colors cursor-pointer ${
                  sort.ascending
                    ? "bg-primary-100 text-primary-700 dark:bg-primary-900/30 dark:text-primary-300"
                    : "text-text-secondary hover:bg-surface-hover"
                }`}
              >
                ↑
              </button>
              <button
                type="button"
                onClick={() => updateSort({ mode: "added", ascending: false })}
                title="Newest first"
                className={`px-2.5 py-1.5 transition-colors cursor-pointer border-l border-border ${
                  !sort.ascending
                    ? "bg-primary-100 text-primary-700 dark:bg-primary-900/30 dark:text-primary-300"
                    : "text-text-secondary hover:bg-surface-hover"
                }`}
              >
                ↓
              </button>
            </div>
          )}
        </div>
      </div>

      <div className="flex-1 overflow-y-auto min-h-0">
        <NoteResultList
          results={sortedResults}
          loading={loading}
          emptyMessage="No cards in this deck yet."
          expanded={expanded}
          setExpanded={setExpanded}
          noteTypes={noteTypes}
          showDeckName
          compact
          onDeleteNote={handleDeleteNote}
          onBuryNote={handleBuryNote}
          busyNoteId={busyNoteId}
        />
      </div>
    </div>
  );
}

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useNavigate, useParams, useSearchParams } from "react-router-dom";
import {
  clearDeckBrowseBookmark,
  loadDeckBrowseBookmark,
  saveDeckBrowseBookmark,
} from "../../lib/deckBrowseBookmark";
import { resolveActiveDeckId } from "../../lib/deckNavigation";
import { isTypingTarget } from "../../lib/isTypingTarget";
import { api, type NoteType } from "../../lib/tauri";
import { NoteTagsPanel } from "../notes/NoteTagsPanel";
import { useDeckStore } from "../../stores/deckStore";
import { useProfileStore } from "../../stores/profileStore";
import {
  NoteResultList,
  notePreviewText,
  type NoteSearchResult,
} from "../search/NoteResultList";
import { StudyCardPreview } from "../search/StudyCardPreview";

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
  const { deckId: routeDeckId } = useParams<{ deckId: string }>();
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();
  const tagFilter = searchParams.get("tag") ?? "";
  const { decks, fetchDecks, selectDeck, selectedDeckId } = useDeckStore();
  const activeProfileId = useProfileStore((s) => s.active?.id);
  const activeDeckId = resolveActiveDeckId(routeDeckId, selectedDeckId);
  const deck = decks.find((d) => d.id === activeDeckId);

  const [results, setResults] = useState<NoteSearchResult[]>([]);
  const [bookmarkNoteId, setBookmarkNoteId] = useState<string | null>(null);
  const listPaneRef = useRef<HTMLDivElement>(null);
  const previewPaneRef = useRef<HTMLDivElement>(null);
  const tagInputRef = useRef<HTMLInputElement | null>(null);
  const [noteTypes, setNoteTypes] = useState<NoteType[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedNoteId, setSelectedNoteId] = useState<string | null>(null);
  const [sort, setSort] = useState<SortState>(loadSortState);
  const [busyNoteId, setBusyNoteId] = useState<string | null>(null);
  const [noteFlagHints, setNoteFlagHints] = useState<Record<string, boolean>>({});

  useEffect(() => {
    fetchDecks();
  }, [fetchDecks]);

  useEffect(() => {
    if (routeDeckId) selectDeck(routeDeckId);
  }, [routeDeckId, selectDeck]);

  useEffect(() => {
    if (!selectedDeckId || selectedDeckId === routeDeckId) return;
    const next = tagFilter
      ? `/deck/${selectedDeckId}/cards?tag=${encodeURIComponent(tagFilter)}`
      : `/deck/${selectedDeckId}/cards`;
    navigate(next, { replace: true });
  }, [selectedDeckId, routeDeckId, navigate, tagFilter]);

  useEffect(() => {
    if (!activeDeckId) return;
    let cancelled = false;
    setLoading(true);
    setSelectedNoteId(null);
    setNoteFlagHints({});
    Promise.all([
      api.searchNotes(
        "",
        activeDeckId,
        tagFilter || undefined,
        undefined,
        10000
      ),
      api.getNoteTypes(),
    ])
      .then(([notes, types]) => {
        if (cancelled) return;
        setResults(notes);
        setNoteTypes(types);
      })
      .catch((e) => {
        if (!cancelled) console.error(e);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [activeDeckId, tagFilter]);

  useEffect(() => {
    if (!activeDeckId || !activeProfileId) {
      setBookmarkNoteId(null);
      return;
    }
    setBookmarkNoteId(
      loadDeckBrowseBookmark(activeProfileId, activeDeckId, tagFilter)
    );
  }, [activeDeckId, activeProfileId, tagFilter]);

  const handleSetBookmark = useCallback(
    (noteId: string | null) => {
      setBookmarkNoteId(noteId);
      if (!activeProfileId || !activeDeckId) return;
      if (noteId) {
        saveDeckBrowseBookmark(
          activeProfileId,
          activeDeckId,
          tagFilter,
          noteId
        );
      } else {
        clearDeckBrowseBookmark(activeProfileId, activeDeckId, tagFilter);
      }
    },
    [activeProfileId, activeDeckId, tagFilter]
  );

  const clearTagFilter = () => {
    const next = new URLSearchParams(searchParams);
    next.delete("tag");
    setSearchParams(next, { replace: true });
  };

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

  useEffect(() => {
    if (loading || !bookmarkNoteId) return;
    if (!sortedResults.some((r) => r.note_id === bookmarkNoteId)) return;
    const id = requestAnimationFrame(() => {
      document
        .querySelector(`[data-note-id="${bookmarkNoteId}"]`)
        ?.scrollIntoView({ block: "nearest" });
    });
    return () => cancelAnimationFrame(id);
  }, [loading, bookmarkNoteId, sortedResults]);

  const selectedNote = useMemo(
    () => sortedResults.find((r) => r.note_id === selectedNoteId) ?? null,
    [sortedResults, selectedNoteId]
  );

  useEffect(() => {
    if (
      selectedNoteId &&
      !sortedResults.some((r) => r.note_id === selectedNoteId)
    ) {
      setSelectedNoteId(null);
    }
  }, [sortedResults, selectedNoteId]);

  const focusCardList = useCallback(() => {
    if (!selectedNoteId || !listPaneRef.current) return;
    const row = listPaneRef.current.querySelector<HTMLButtonElement>(
      `[data-note-id="${selectedNoteId}"] [data-note-row-select]`
    );
    row?.focus();
  }, [selectedNoteId]);

  const focusTags = useCallback(() => {
    const attempt = () => tagInputRef.current?.focus();
    attempt();
    requestAnimationFrame(attempt);
    window.setTimeout(attempt, 50);
    window.setTimeout(attempt, 200);
  }, []);

  const focusCardText = useCallback(() => {
    previewPaneRef.current
      ?.querySelector<HTMLElement>("[data-study-preview-content]")
      ?.focus();
  }, []);

  const isFocusInPreviewTyping = useCallback((target: EventTarget | null) => {
    if (!(target instanceof Element) || !previewPaneRef.current?.contains(target)) {
      return false;
    }
    if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement) {
      return true;
    }
    return target instanceof HTMLElement && target.isContentEditable;
  }, []);

  /** List pane has focus, or a card is selected and focus is not in the preview editor. */
  const isListNavigationContext = useCallback(
    (target: EventTarget | null) => {
      if (!selectedNoteId) return false;
      if (isFocusInPreviewTyping(target)) return false;
      if (!(target instanceof Element)) return false;
      if (listPaneRef.current?.contains(target)) return true;
      return !previewPaneRef.current?.contains(target);
    },
    [selectedNoteId, isFocusInPreviewTyping]
  );

  const handleSelectNote = useCallback((id: string | null) => {
    setSelectedNoteId(id);
    if (!id) return;
    requestAnimationFrame(() => {
      listPaneRef.current
        ?.querySelector<HTMLButtonElement>(
          `[data-note-id="${id}"] [data-note-row-select]`
        )
        ?.focus();
    });
  }, []);

  const goToBookmark = useCallback(() => {
    if (!bookmarkNoteId) return;
    handleSelectNote(bookmarkNoteId);
  }, [bookmarkNoteId, handleSelectNote]);

  useEffect(() => {
    if (!selectedNoteId) return;
    document
      .querySelector(`[data-note-id="${selectedNoteId}"]`)
      ?.scrollIntoView({ block: "nearest" });
  }, [selectedNoteId]);

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      const target = e.target;

      if (isListNavigationContext(target)) {
        if (e.key === "ArrowRight") {
          e.preventDefault();
          focusTags();
          return;
        }
        if (e.key === "ArrowDown") {
          e.preventDefault();
          focusCardText();
          return;
        }
        if (e.key === "ArrowUp") {
          e.preventDefault();
          const currentIdx = sortedResults.findIndex(
            (r) => r.note_id === selectedNoteId
          );
          const nextIdx = Math.max(0, currentIdx - 1);
          const nextId = sortedResults[nextIdx]!.note_id;
          handleSelectNote(nextId);
          return;
        }
      }

      if (
        target instanceof HTMLElement &&
        target.matches("[data-study-preview-content]") &&
        e.key === "ArrowUp"
      ) {
        e.preventDefault();
        focusTags();
        return;
      }

      if (e.key === "ArrowLeft" && previewPaneRef.current) {
        if (
          target instanceof Node &&
          previewPaneRef.current.contains(target) &&
          !(target instanceof HTMLInputElement) &&
          !(target instanceof HTMLTextAreaElement) &&
          !(target instanceof HTMLElement && target.isContentEditable)
        ) {
          e.preventDefault();
          focusCardList();
          return;
        }
      }
      if (isTypingTarget(e.target)) return;
      if (e.key === "ArrowDown") {
        e.preventDefault();
        const nextIdx = selectedNoteId
          ? Math.min(
              sortedResults.length - 1,
              sortedResults.findIndex((r) => r.note_id === selectedNoteId) + 1
            )
          : 0;
        handleSelectNote(sortedResults[nextIdx]?.note_id ?? null);
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        const currentIdx = selectedNoteId
          ? sortedResults.findIndex((r) => r.note_id === selectedNoteId)
          : sortedResults.length;
        const nextIdx = Math.max(0, currentIdx - 1);
        handleSelectNote(sortedResults[nextIdx]?.note_id ?? null);
      } else if (
        e.key === "*" &&
        selectedNoteId &&
        !e.ctrlKey &&
        !e.metaKey &&
        !e.altKey
      ) {
        e.preventDefault();
        handleSetBookmark(
          bookmarkNoteId === selectedNoteId ? null : selectedNoteId
        );
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [
    selectedNoteId,
    bookmarkNoteId,
    handleSetBookmark,
    handleSelectNote,
    focusCardList,
    focusTags,
    focusCardText,
    isListNavigationContext,
    sortedResults,
  ]);

  const handleTagsUpdated = useCallback((noteId: string, tags: string[]) => {
    setResults((prev) =>
      prev.map((r) => (r.note_id === noteId ? { ...r, tags } : r))
    );
  }, []);

  const handleNoteUpdated = useCallback(
    (noteId: string, fields: Record<string, string>, tags: string[]) => {
      setResults((prev) =>
        prev.map((r) =>
          r.note_id === noteId ? { ...r, fields_json: fields, tags } : r
        )
      );
    },
    []
  );

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
      if (selectedNoteId === noteId) setSelectedNoteId(null);
      if (bookmarkNoteId === noteId) handleSetBookmark(null);
      await fetchDecks();
    } catch (e) {
      console.error(e);
      alert("Failed to delete note.");
    } finally {
      setBusyNoteId(null);
    }
  };

  const handleToggleFlagNote = async (noteId: string) => {
    setBusyNoteId(noteId);
    try {
      const cards = await api.getCardsForNote(noteId);
      const flags = await Promise.all(cards.map((c) => api.getCardFlag(c.id)));
      const anyFlagged = flags.some(Boolean);
      const next = cards.length > 0 && !anyFlagged;
      await Promise.all(cards.map((c) => api.setCardFlag(c.id, next)));
      setNoteFlagHints((prev) => ({ ...prev, [noteId]: next }));
    } catch (e) {
      console.error(e);
      alert("Failed to update flag.");
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
      if (selectedNoteId === noteId) setSelectedNoteId(null);
      await fetchDecks();
    } catch (e) {
      console.error(e);
      alert("Failed to bury cards.");
    } finally {
      setBusyNoteId(null);
    }
  };

  if (!activeDeckId) {
    return null;
  }

  return (
    <div className="h-full flex flex-col p-6">
      <div className="flex items-start justify-between gap-4 mb-4 shrink-0">
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
          {tagFilter && (
            <div className="flex items-center gap-2 mt-2 text-sm">
              <span className="text-text-secondary">Tag</span>
              <span className="font-medium text-primary-600 dark:text-primary-400">
                {tagFilter}
              </span>
              <button
                type="button"
                onClick={clearTagFilter}
                className="text-xs text-text-muted hover:text-text cursor-pointer"
              >
                Clear
              </button>
            </div>
          )}
          {bookmarkNoteId &&
            sortedResults.some((r) => r.note_id === bookmarkNoteId) && (
              <button
                type="button"
                onClick={goToBookmark}
                className="mt-2 inline-flex items-center gap-1 text-xs text-amber-700 dark:text-amber-300 hover:underline cursor-pointer"
                title="Jump to your resume bookmark in the list"
              >
                <svg
                  className="w-3.5 h-3.5"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                  aria-hidden
                >
                  <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
                </svg>
                Resume from bookmark
              </button>
            )}
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

      <div className="flex-1 flex gap-4 min-h-0">
        <div
          ref={listPaneRef}
          className="w-[min(42%,22rem)] shrink-0 flex flex-col min-h-0 border border-border rounded-xl bg-surface-alt overflow-hidden"
        >
          <div className="flex-1 overflow-y-auto min-h-0 p-2">
            <NoteResultList
              results={sortedResults}
              loading={loading}
              emptyMessage="No cards in this deck yet."
              selectedNoteId={selectedNoteId}
              onSelectNote={handleSelectNote}
              noteTypes={noteTypes}
              showDeckName
              compact
              onDeleteNote={handleDeleteNote}
              onBuryNote={handleBuryNote}
              onToggleFlagNote={handleToggleFlagNote}
              noteFlagHints={noteFlagHints}
              busyNoteId={busyNoteId}
              bookmarkNoteId={bookmarkNoteId}
              onSetBookmark={handleSetBookmark}
            />
          </div>
        </div>

        <div
          ref={previewPaneRef}
          className="flex-1 min-w-0 flex flex-col min-h-0 border border-border rounded-xl bg-surface overflow-hidden"
        >
          {selectedNote ? (
            <div className="flex-1 overflow-y-auto p-4 space-y-4">
              <NoteTagsPanel
                noteId={selectedNote.note_id}
                onArrowLeftToList={focusCardList}
                onArrowDownToContent={focusCardText}
                tagInputRef={tagInputRef}
                onTagsUpdated={(tags) =>
                  handleTagsUpdated(selectedNote.note_id, tags)
                }
              />
              <StudyCardPreview
                key={selectedNote.note_id}
                noteId={selectedNote.note_id}
                noteTypeId={selectedNote.note_type_id}
                fields={selectedNote.fields_json}
                noteTypes={noteTypes}
                enableEditShortcut
                onFlagStateChange={(anyFlagged) =>
                  setNoteFlagHints((prev) => ({
                    ...prev,
                    [selectedNote.note_id]: anyFlagged,
                  }))
                }
                onNoteUpdated={(fields, tags) =>
                  handleNoteUpdated(selectedNote.note_id, fields, tags)
                }
              />
            </div>
          ) : (
            <div className="flex-1 flex items-center justify-center p-8 text-sm text-text-muted text-center">
              {loading
                ? "Loading cards…"
                : sortedResults.length === 0
                  ? "No cards in this deck."
                  : "Select a card on the left. In the list: → tags, ↓ card text, ↑ previous card. ← returns to the list. E to edit."}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

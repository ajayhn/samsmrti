import { useState, useEffect, useCallback, useMemo } from "react";
import { useSearchParams, useNavigate } from "react-router-dom";
import { filterInputProps } from "../../lib/filterInput";
import { flattenDecksForPicker } from "../../lib/deckTree";
import { api, type NoteType } from "../../lib/tauri";
import { useDeckStore } from "../../stores/deckStore";
import { useTagListStore } from "../../stores/tagListStore";
import { BuriedCardsPanel } from "./BuriedCardsPanel";
import { SearchableFilterCombobox } from "./SearchableFilterCombobox";
import {
  NoteResultList,
  type NoteSearchResult,
} from "./NoteResultList";

type BrowserTab = "search" | "tags" | "buried";

export function CardBrowser() {
  const [tab, setTab] = useState<BrowserTab>("search");
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<NoteSearchResult[]>([]);
  const [selectedDeck, setSelectedDeck] = useState<string>("");
  const [selectedTag, setSelectedTag] = useState<string>("");
  const [selectedNoteTypeId, setSelectedNoteTypeId] = useState<string>("");
  const [filterNoteTypeName, setFilterNoteTypeName] = useState<string>("");
  const [searchParams, setSearchParams] = useSearchParams();
  const navigate = useNavigate();
  const [tags, setTags] = useState<[string, string, number][]>([]);
  const [tagFilter, setTagFilter] = useState("");
  const [loading, setLoading] = useState(false);
  const [indexing, setIndexing] = useState(false);
  const [expanded, setExpanded] = useState<string | null>(null);
  const [noteTypes, setNoteTypes] = useState<NoteType[]>([]);
  const { decks } = useDeckStore();
  const tagListRevision = useTagListStore((s) => s.revision);

  useEffect(() => {
    api.getAllTags().then(setTags).catch(() => {});
  }, [tagListRevision]);

  useEffect(() => {
    api.getNoteTypes().then(setNoteTypes).catch(() => {});
    setIndexing(true);
    api
      .ensureSearchIndex()
      .catch(() => {})
      .finally(() => setIndexing(false));
  }, []);

  useEffect(() => {
    const ntId = searchParams.get("noteTypeId");
    const ntName = searchParams.get("noteTypeName");
    const deckId = searchParams.get("deckId");
    const tag = searchParams.get("tag");
    if (ntId) {
      setSelectedNoteTypeId(ntId);
      setFilterNoteTypeName(ntName ?? "");
      setTab("search");
    }
    if (deckId) setSelectedDeck(deckId);
    if (tag) {
      setSelectedTag(tag);
      setTab("tags");
    }
  }, [searchParams]);

  const doSearch = useCallback(async () => {
    setLoading(true);
    try {
      const res = await api.searchNotes(
        query,
        selectedDeck || undefined,
        selectedTag || undefined,
        selectedNoteTypeId || undefined,
        100
      );
      setResults(res);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  }, [query, selectedDeck, selectedTag, selectedNoteTypeId]);

  useEffect(() => {
    if (tab !== "search" && tab !== "tags") return;
    const hasFilters =
      query.trim() || selectedDeck || selectedTag || selectedNoteTypeId;
    const delay = hasFilters && !query.trim() && selectedTag ? 0 : 300;
    const timeout = setTimeout(() => {
      if (hasFilters) {
        doSearch();
      } else {
        setResults([]);
      }
    }, delay);
    return () => clearTimeout(timeout);
  }, [query, selectedDeck, selectedTag, selectedNoteTypeId, doSearch, tab]);

  const clearBrowseFilters = () => {
    setSelectedNoteTypeId("");
    setFilterNoteTypeName("");
    setSelectedDeck("");
    setSelectedTag("");
    setQuery("");
    setResults([]);
    setSearchParams({});
  };

  const filteredTags = useMemo(() => {
    const q = tagFilter.trim().toLowerCase();
    if (!q) return tags;
    return tags.filter(([, name]) => name.toLowerCase().includes(q));
  }, [tags, tagFilter]);

  const deckFilterOptions = useMemo(
    () =>
      flattenDecksForPicker(decks).map((d) => ({
        value: d.id,
        label: d.name,
        depth: d.depth,
      })),
    [decks]
  );

  const tagFilterOptions = useMemo(
    () =>
      [...tags]
        .sort((a, b) => a[1].localeCompare(b[1]))
        .map(([, name, count]) => ({
          value: name,
          label: name,
          hint: String(count),
        })),
    [tags]
  );

  const selectTag = (name: string) => {
    setSelectedTag(name);
    setTab("tags");
    setQuery("");
    setSelectedDeck("");
    setSearchParams((prev) => {
      const next = new URLSearchParams(prev);
      next.set("tag", name);
      return next;
    });
  };

  const clearTagFilter = () => {
    setSelectedTag("");
    setResults([]);
    setSearchParams((prev) => {
      const next = new URLSearchParams(prev);
      next.delete("tag");
      return next;
    });
  };

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

  return (
    <div className="h-full flex flex-col p-6">
      <div className="flex items-center justify-between mb-4">
        <div>
          <h2 className="text-xl font-bold text-text">Card Browser</h2>
          {indexing && (
            <p className="text-xs text-text-muted mt-0.5">
              Preparing search index (first time only)…
            </p>
          )}
        </div>
        <div className="flex rounded-lg border border-border overflow-hidden text-sm">
          <button
            type="button"
            onClick={() => setTab("search")}
            className={`px-4 py-1.5 transition-colors cursor-pointer ${
              tab === "search"
                ? "bg-primary-100 text-primary-700 dark:bg-primary-900/30 dark:text-primary-300"
                : "text-text-secondary hover:bg-surface-hover"
            }`}
          >
            Search
          </button>
          <button
            type="button"
            onClick={() => {
              setTab("tags");
              setQuery("");
            }}
            className={`px-4 py-1.5 transition-colors cursor-pointer ${
              tab === "tags"
                ? "bg-primary-100 text-primary-700 dark:bg-primary-900/30 dark:text-primary-300"
                : "text-text-secondary hover:bg-surface-hover"
            }`}
          >
            Tags
            {tags.length > 0 && (
              <span className="ml-1.5 text-xs opacity-75">{tags.length}</span>
            )}
          </button>
          <button
            type="button"
            onClick={() => setTab("buried")}
            className={`px-4 py-1.5 transition-colors cursor-pointer ${
              tab === "buried"
                ? "bg-primary-100 text-primary-700 dark:bg-primary-900/30 dark:text-primary-300"
                : "text-text-secondary hover:bg-surface-hover"
            }`}
          >
            Buried
          </button>
        </div>
      </div>

      {tab === "buried" ? (
        <BuriedCardsPanel />
      ) : tab === "tags" ? (
        <div className="flex-1 flex gap-4 min-h-0">
          <div className="w-72 shrink-0 flex flex-col border border-border rounded-xl bg-surface-alt overflow-hidden">
            <div className="p-2 border-b border-border">
              <input
                type="text"
                value={tagFilter}
                onChange={(e) => setTagFilter(e.target.value)}
                placeholder="Filter tags…"
                className="w-full px-2.5 py-1.5 text-sm bg-surface border border-border rounded-lg focus:outline-none focus:border-primary-500"
                name="samsmrti-browse-tag-filter"
                {...filterInputProps}
              />
            </div>
            <div className="flex-1 overflow-y-auto p-2 space-y-0.5">
              {filteredTags.length === 0 ? (
                <p className="text-xs text-text-muted text-center py-4">
                  {tags.length === 0
                    ? "No tags yet. Tags are imported from Anki and set when creating notes."
                    : "No tags match"}
                </p>
              ) : (
                filteredTags.map(([, name, count]) => (
                  <button
                    key={name}
                    type="button"
                    onClick={() => selectTag(name)}
                    className={`w-full text-left px-2.5 py-1.5 rounded-lg text-sm transition-colors cursor-pointer flex items-center justify-between gap-2 ${
                      selectedTag === name
                        ? "bg-primary-100 text-primary-700 dark:bg-primary-900/30 dark:text-primary-300"
                        : "text-text-secondary hover:bg-surface-hover"
                    }`}
                  >
                    <span className="truncate">{name}</span>
                    <span className="text-xs text-text-muted shrink-0 tabular-nums">
                      {count}
                    </span>
                  </button>
                ))
              )}
            </div>
          </div>

          <div className="flex-1 flex flex-col min-w-0">
            {selectedTag ? (
              <>
                <div className="flex items-center gap-2 mb-3">
                  <span className="text-sm text-text-secondary">Notes tagged</span>
                  <span className="text-sm font-medium text-primary-600 dark:text-primary-400">
                    {selectedTag}
                  </span>
                  <button
                    type="button"
                    onClick={clearTagFilter}
                    className="text-xs text-text-muted hover:text-text ml-auto cursor-pointer"
                  >
                    Clear
                  </button>
                </div>
                <div className="flex-1 overflow-y-auto">
                  <NoteResultList
                    results={results}
                    loading={loading}
                    emptyMessage="No notes with this tag."
                    expanded={expanded}
                    setExpanded={setExpanded}
                    noteTypes={noteTypes}
                    onTagsUpdated={handleTagsUpdated}
                    onNoteUpdated={handleNoteUpdated}
                  />
                </div>
              </>
            ) : (
              <div className="flex-1 flex items-center justify-center text-sm text-text-muted text-center px-8">
                Select a tag on the left to browse {tags.length > 0 ? `${tags.length} imported ` : ""}
                notes. Tags came from your Anki import (space-separated on each note).
              </div>
            )}
          </div>
        </div>
      ) : (
        <>
      {(selectedNoteTypeId || selectedDeck || selectedTag) && (
        <div className="flex flex-wrap items-center gap-2 mb-3 text-sm">
          {selectedNoteTypeId && (
            <button
              type="button"
              onClick={() => navigate("/note-types")}
              className="inline-flex items-center gap-1 text-primary-500 hover:underline font-medium cursor-pointer shrink-0"
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
              Note types
            </button>
          )}
          <span className="text-text-muted">Showing:</span>
          {selectedNoteTypeId && (
            <span className="px-2 py-0.5 rounded-full bg-primary-100 text-primary-700 dark:bg-primary-900/30 dark:text-primary-300 text-xs">
              {filterNoteTypeName || "Note type"}
            </span>
          )}
          {selectedDeck && (
            <span className="px-2 py-0.5 rounded-full bg-surface-alt border border-border text-xs text-text-secondary">
              {decks.find((d) => d.id === selectedDeck)?.name ?? "Deck"}
            </span>
          )}
          {selectedTag && (
            <span className="px-2 py-0.5 rounded-full bg-surface-alt border border-border text-xs text-text-secondary">
              {selectedTag}
            </span>
          )}
          <button
            type="button"
            onClick={clearBrowseFilters}
            className="text-xs text-primary-500 hover:underline cursor-pointer"
          >
            Clear filters
          </button>
        </div>
      )}
      <div className="flex gap-3 mb-4">
            <input
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search cards..."
              className="flex-1 px-4 py-2.5 bg-surface-alt border border-border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
              name="samsmrti-card-search"
              {...filterInputProps}
            />
            <SearchableFilterCombobox
              options={deckFilterOptions}
              value={selectedDeck}
              onChange={setSelectedDeck}
              placeholder="All Decks"
              clearLabel="All Decks"
              className="w-[200px] shrink-0"
              inputName="samsmrti-browse-deck-filter"
              ariaLabel="Filter by deck"
            />
            <SearchableFilterCombobox
              options={tagFilterOptions}
              value={selectedTag}
              onChange={setSelectedTag}
              placeholder="All Tags"
              clearLabel="All Tags"
              className="w-[220px] shrink-0"
              inputName="samsmrti-browse-tag-combobox"
              ariaLabel="Filter by tag"
            />
          </div>

          <div className="flex-1 overflow-y-auto min-h-0">
            <NoteResultList
              results={results}
              loading={loading}
              emptyMessage={
                query || selectedDeck || selectedTag || selectedNoteTypeId
                  ? "No results found."
                  : "Type to search, pick a deck or tag, or browse from Note Types."
              }
              expanded={expanded}
              setExpanded={setExpanded}
              noteTypes={noteTypes}
              onTagsUpdated={handleTagsUpdated}
              onNoteUpdated={handleNoteUpdated}
            />
          </div>
        </>
      )}
    </div>
  );
}

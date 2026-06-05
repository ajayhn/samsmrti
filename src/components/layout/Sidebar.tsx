import { useEffect, useState, useCallback, useMemo, useRef } from "react";
import { useNavigate, useLocation } from "react-router-dom";
import { filterInputProps } from "../../lib/filterInput";
import { useDeckStore } from "../../stores/deckStore";
import { useTagListStore } from "../../stores/tagListStore";
import { ImportDialog } from "../import/ImportDialog";
import { ProfileSwitcher } from "../profile/ProfileSwitcher";
import { api, type DeckWithCounts } from "../../lib/tauri";
import { navigateAfterDeckSelect } from "../../lib/deckNavigation";
import { navigateToTagFilter } from "../../lib/tagNavigation";
import {
  deckMatchesQuery,
  filterDeckTree,
  loadCollapsedDeckIds,
  saveCollapsedDeckIds,
} from "../../lib/deckTree";

function Kbd({ children }: { children: string }) {
  return (
    <kbd className="ml-auto text-[10px] font-semibold uppercase text-text-muted/60 bg-surface rounded px-1 py-0.5 border border-border">
      {children}
    </kbd>
  );
}

function DeckRowMenu({
  deck,
  onDelete,
}: {
  deck: DeckWithCounts;
  onDelete: (deckId: string) => void;
}) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const close = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", close);
    return () => document.removeEventListener("mousedown", close);
  }, [open]);

  return (
    <div className="relative shrink-0" ref={ref}>
      <button
        type="button"
        onClick={(e) => {
          e.stopPropagation();
          setOpen((v) => !v);
        }}
        className="p-1.5 rounded-md text-text-muted hover:text-text hover:bg-surface-hover cursor-pointer"
        aria-label={`Actions for ${deck.name}`}
        aria-expanded={open}
        aria-haspopup="menu"
      >
        <span className="text-base leading-none" aria-hidden>
          ⋯
        </span>
      </button>
      {open && (
        <div
          role="menu"
          className="absolute right-0 top-full z-30 mt-0.5 min-w-[7rem] py-1 rounded-lg border border-border bg-surface shadow-lg"
        >
          <button
            type="button"
            role="menuitem"
            onClick={(e) => {
              e.stopPropagation();
              setOpen(false);
              onDelete(deck.id);
            }}
            className="w-full text-left px-3 py-1.5 text-sm text-danger hover:bg-red-50 dark:hover:bg-red-900/20 cursor-pointer"
          >
            Delete
          </button>
        </div>
      )}
    </div>
  );
}

function ChevronIcon({ expanded }: { expanded: boolean }) {
  return (
    <svg
      className={`w-3.5 h-3.5 shrink-0 transition-transform ${expanded ? "rotate-90" : ""}`}
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
  );
}

function DeckTreeItem({
  deck,
  allDecks,
  depth = 0,
  searchQuery,
  visibleIds,
  forceExpandedIds,
  collapsedIds,
  onToggleCollapse,
  onDeleteDeck,
  onSelectDeck,
}: {
  deck: DeckWithCounts;
  allDecks: DeckWithCounts[];
  depth?: number;
  searchQuery: string;
  visibleIds: Set<string>;
  forceExpandedIds: Set<string>;
  collapsedIds: Set<string>;
  onToggleCollapse: (id: string) => void;
  onDeleteDeck: (deckId: string) => void;
  onSelectDeck: (deckId: string) => void;
}) {
  const { selectedDeckId } = useDeckStore();
  const children = allDecks.filter((d) => d.parent_id === deck.id);
  const visibleChildren = children.filter((c) => visibleIds.has(c.id));
  const hasChildren = visibleChildren.length > 0;
  const isSelected = selectedDeckId === deck.id;
  const isExpanded =
    hasChildren &&
    (forceExpandedIds.has(deck.id) || !collapsedIds.has(deck.id));
  const highlight = deckMatchesQuery(deck, searchQuery);

  if (!visibleIds.has(deck.id)) return null;

  return (
    <div>
      <div
        className={`flex items-center gap-0.5 rounded-lg transition-colors ${
          isSelected
            ? "bg-primary-100 dark:bg-primary-900/30"
            : highlight
              ? "bg-amber-50 dark:bg-amber-900/20"
              : ""
        }`}
        style={{ paddingLeft: `${4 + depth * 14}px` }}
      >
        {hasChildren ? (
          <button
            type="button"
            onClick={(e) => {
              e.stopPropagation();
              onToggleCollapse(deck.id);
            }}
            className="p-1.5 rounded-md text-text-muted hover:text-text hover:bg-surface-hover shrink-0 cursor-pointer"
            aria-label={isExpanded ? "Collapse subdecks" : "Expand subdecks"}
            aria-expanded={isExpanded}
          >
            <ChevronIcon expanded={isExpanded} />
          </button>
        ) : (
          <span className="w-[26px] shrink-0" aria-hidden />
        )}

        <button
          type="button"
          onClick={() => onSelectDeck(deck.id)}
          className={`flex-1 min-w-0 flex items-center justify-between py-2 pr-2 rounded-lg text-sm transition-colors cursor-pointer text-left ${
            isSelected
              ? "text-primary-700 dark:text-primary-300 font-medium"
              : "text-text-secondary hover:bg-surface-hover"
          }`}
        >
          <span className="truncate">{deck.name}</span>
          <span className="flex gap-2 text-xs shrink-0 ml-2">
            {deck.new_cards > 0 && (
              <span className="text-primary-500 font-semibold">
                {deck.new_cards}
              </span>
            )}
            {deck.due_cards > 0 && (
              <span className="text-warning font-semibold">
                {deck.due_cards}
              </span>
            )}
          </span>
        </button>

        <DeckRowMenu deck={deck} onDelete={onDeleteDeck} />
      </div>

      {hasChildren && isExpanded && (
        <div>
          {visibleChildren.map((child) => (
            <DeckTreeItem
              key={child.id}
              deck={child}
              allDecks={allDecks}
              depth={depth + 1}
              searchQuery={searchQuery}
              visibleIds={visibleIds}
              forceExpandedIds={forceExpandedIds}
              collapsedIds={collapsedIds}
              onToggleCollapse={onToggleCollapse}
              onDeleteDeck={onDeleteDeck}
              onSelectDeck={onSelectDeck}
            />
          ))}
        </div>
      )}
    </div>
  );
}

export function Sidebar() {
  const { decks, fetchDecks, loading, deleteDeck, selectDeck } = useDeckStore();
  const navigate = useNavigate();
  const location = useLocation();
  const [showImport, setShowImport] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [tagSearchQuery, setTagSearchQuery] = useState("");
  const [tagHighlightIndex, setTagHighlightIndex] = useState(0);
  const tagSearchInputRef = useRef<HTMLInputElement>(null);
  const [tags, setTags] = useState<[string, string, number][]>([]);
  const tagListRevision = useTagListStore((s) => s.revision);
  const [collapsedIds, setCollapsedIds] = useState<Set<string>>(loadCollapsedDeckIds);

  const { visibleIds, forceExpandedIds, hasQuery } = useMemo(
    () => filterDeckTree(decks, searchQuery),
    [decks, searchQuery]
  );

  const rootDecks = useMemo(
    () => decks.filter((d) => !d.parent_id && visibleIds.has(d.id)),
    [decks, visibleIds]
  );

  const parentIdsWithChildren = useMemo(() => {
    const ids = new Set<string>();
    for (const d of decks) {
      if (d.parent_id) ids.add(d.parent_id);
    }
    return ids;
  }, [decks]);

  useEffect(() => {
    fetchDecks();
  }, [fetchDecks]);

  useEffect(() => {
    api.getAllTags().then(setTags).catch(() => {});
  }, [showImport, tagListRevision]);

  const filteredTags = useMemo(() => {
    const q = tagSearchQuery.trim().toLowerCase();
    if (!q) return [];
    return tags.filter(([, name]) => name.toLowerCase().includes(q));
  }, [tags, tagSearchQuery]);

  const hasTagQuery = tagSearchQuery.trim().length > 0;

  useEffect(() => {
    setTagHighlightIndex(0);
  }, [filteredTags.length, tagSearchQuery]);

  const openTagFilter = useCallback(
    (name: string) => {
      navigateToTagFilter(navigate, name, location.pathname);
      setTagSearchQuery("");
      setTagHighlightIndex(0);
      tagSearchInputRef.current?.blur();
    },
    [navigate, location.pathname]
  );

  const handleTagSearchKeyDown = (
    e: React.KeyboardEvent<HTMLInputElement>
  ) => {
    if (!hasTagQuery || filteredTags.length === 0) {
      if (e.key === "Escape") {
        setTagSearchQuery("");
      }
      return;
    }
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        setTagHighlightIndex((i) =>
          i + 1 >= filteredTags.length ? 0 : i + 1
        );
        break;
      case "ArrowUp":
        e.preventDefault();
        setTagHighlightIndex((i) =>
          i <= 0 ? filteredTags.length - 1 : i - 1
        );
        break;
      case "Enter": {
        e.preventDefault();
        const [, name] = filteredTags[tagHighlightIndex] ?? [];
        if (name) openTagFilter(name);
        break;
      }
      case "Escape":
        e.preventDefault();
        setTagSearchQuery("");
        break;
    }
  };

  const toggleCollapse = useCallback((id: string) => {
    setCollapsedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      saveCollapsedDeckIds(next);
      return next;
    });
  }, []);

  const collapseAll = useCallback(() => {
    const next = new Set(parentIdsWithChildren);
    setCollapsedIds(next);
    saveCollapsedDeckIds(next);
  }, [parentIdsWithChildren]);

  const expandAll = useCallback(() => {
    setCollapsedIds(new Set());
    saveCollapsedDeckIds(new Set());
  }, []);

  const handleSelectDeck = useCallback(
    (id: string) => {
      selectDeck(id);
      navigateAfterDeckSelect(
        navigate,
        id,
        location.pathname,
        location.search
      );
    },
    [selectDeck, navigate, location.pathname, location.search]
  );

  const handleDeleteDeck = useCallback(
    async (id: string) => {
      const deck = decks.find((d) => d.id === id);
      if (!deck) return;
      const childCount = decks.filter((d) => d.parent_id === id).length;
      const msg =
        childCount > 0
          ? `Delete "${deck.name}" and all ${childCount} subdeck(s) and their cards?`
          : `Delete "${deck.name}" and all its cards?`;
      if (!confirm(msg)) return;
      await deleteDeck(id);
    },
    [decks, deleteDeck]
  );

  const isInputFocused = () => {
    const tag = document.activeElement?.tagName;
    return (
      tag === "INPUT" ||
      tag === "TEXTAREA" ||
      tag === "SELECT" ||
      (document.activeElement as HTMLElement)?.isContentEditable
    );
  };

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (isInputFocused()) return;
      if (location.pathname.startsWith("/review/")) return;

      if (e.ctrlKey || e.metaKey || e.altKey) return;

      switch (e.key) {
        case "b":
        case "B":
          e.preventDefault();
          navigate("/browse");
          break;
        case "g":
        case "G":
          e.preventDefault();
          navigate("/graph");
          break;
        case "i":
        case "I":
          e.preventDefault();
          setShowImport(true);
          break;
        case "?":
          e.preventDefault();
          navigate("/help");
          break;
      }
    },
    [navigate, location.pathname]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  return (
    <>
      <aside className="w-64 h-full border-r border-border bg-surface-alt flex flex-col shrink-0">
        <div className="p-4 border-b border-border">
          <h1 className="text-lg font-bold tracking-tight text-text">
            Samsmrti
          </h1>
        </div>

        <div className="px-2 pt-2 pb-1 space-y-1.5 border-b border-border">
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search decks…"
            className="w-full px-2.5 py-1.5 text-sm bg-surface border border-border rounded-lg text-text placeholder:text-text-muted focus:outline-none focus:border-primary-500"
            aria-label="Search decks by name"
            name="samsmrti-deck-filter"
            {...filterInputProps}
          />
          <input
            ref={tagSearchInputRef}
            type="text"
            value={tagSearchQuery}
            onChange={(e) => setTagSearchQuery(e.target.value)}
            onKeyDown={handleTagSearchKeyDown}
            placeholder="Search tags…"
            className="w-full px-2.5 py-1.5 text-sm bg-surface border border-border rounded-lg text-text placeholder:text-text-muted focus:outline-none focus:border-primary-500"
            aria-label="Search tags by name"
            name="samsmrti-tag-filter"
            role="combobox"
            aria-expanded={hasTagQuery}
            aria-controls="sidebar-tag-listbox"
            aria-autocomplete="list"
            aria-activedescendant={
              hasTagQuery && filteredTags.length > 0
                ? `sidebar-tag-opt-${tagHighlightIndex}`
                : undefined
            }
            {...filterInputProps}
          />
          {hasTagQuery && (
            <div
              id="sidebar-tag-listbox"
              className="max-h-40 overflow-y-auto rounded-lg border border-border bg-surface"
              role="listbox"
              aria-label="Matching tags"
            >
              {filteredTags.length === 0 ? (
                <p className="px-2.5 py-2 text-xs text-text-muted text-center">
                  No tags match
                </p>
              ) : (
                filteredTags.map(([, name, count], index) => (
                  <button
                    key={name}
                    id={`sidebar-tag-opt-${index}`}
                    type="button"
                    role="option"
                    aria-selected={index === tagHighlightIndex}
                    onMouseEnter={() => setTagHighlightIndex(index)}
                    onClick={() => openTagFilter(name)}
                    className={`w-full text-left px-2.5 py-1.5 text-sm transition-colors cursor-pointer flex items-center justify-between gap-2 ${
                      index === tagHighlightIndex
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
          )}
          {parentIdsWithChildren.size > 0 && (
            <div className="flex gap-2 px-0.5">
              <button
                type="button"
                onClick={expandAll}
                className="text-[11px] text-text-muted hover:text-primary-600 transition-colors cursor-pointer"
              >
                Expand all
              </button>
              <span className="text-text-muted/40">·</span>
              <button
                type="button"
                onClick={collapseAll}
                className="text-[11px] text-text-muted hover:text-primary-600 transition-colors cursor-pointer"
              >
                Collapse all
              </button>
            </div>
          )}
        </div>

        <nav className="flex-1 overflow-y-auto p-2 space-y-0.5 min-h-0">
          {loading && decks.length === 0 ? (
            <div className="p-4 text-center text-text-muted text-sm">
              Loading...
            </div>
          ) : decks.length === 0 ? (
            <div className="p-4 text-center text-text-muted text-sm">
              No decks yet
            </div>
          ) : rootDecks.length === 0 ? (
            <div className="p-4 text-center text-text-muted text-sm">
              No decks match &ldquo;{searchQuery.trim()}&rdquo;
            </div>
          ) : (
            rootDecks.map((deck) => (
              <DeckTreeItem
                key={deck.id}
                deck={deck}
                allDecks={decks}
                searchQuery={searchQuery}
                visibleIds={visibleIds}
                forceExpandedIds={forceExpandedIds}
                collapsedIds={collapsedIds}
                onToggleCollapse={toggleCollapse}
                onDeleteDeck={handleDeleteDeck}
                onSelectDeck={handleSelectDeck}
              />
            ))
          )}
        </nav>

        <div className="p-3 border-t border-border space-y-1">
          <button
            onClick={() => navigate("/browse")}
            className="w-full px-3 py-2 text-sm text-text-secondary hover:bg-surface-hover rounded-lg transition-colors text-left cursor-pointer flex items-center"
          >
            Browse Cards
            <Kbd>B</Kbd>
          </button>
          <button
            onClick={() => navigate("/stats")}
            className="w-full px-3 py-2 text-sm text-text-secondary hover:bg-surface-hover rounded-lg transition-colors text-left cursor-pointer flex items-center"
          >
            Statistics
          </button>
          <button
            onClick={() => navigate("/graph")}
            className="w-full px-3 py-2 text-sm text-text-secondary hover:bg-surface-hover rounded-lg transition-colors text-left cursor-pointer flex items-center"
          >
            Knowledge Map
            <Kbd>G</Kbd>
          </button>
          <button
            onClick={() => setShowImport(true)}
            className="w-full px-3 py-2 text-sm text-text-secondary hover:bg-surface-hover rounded-lg transition-colors text-left cursor-pointer flex items-center"
          >
            Import Deck
            <Kbd>I</Kbd>
          </button>
          <button
            onClick={() => navigate("/note-types")}
            className="w-full px-3 py-2 text-sm text-text-secondary hover:bg-surface-hover rounded-lg transition-colors text-left cursor-pointer flex items-center"
          >
            Note Types
          </button>
          <ProfileSwitcher />
          <button
            onClick={() => navigate("/settings")}
            className="w-full px-3 py-2 text-sm text-text-secondary hover:bg-surface-hover rounded-lg transition-colors text-left cursor-pointer flex items-center"
          >
            Settings
          </button>
          <button
            onClick={() => navigate("/help")}
            className="w-full px-3 py-2 text-sm text-text-secondary hover:bg-surface-hover rounded-lg transition-colors text-left cursor-pointer flex items-center"
          >
            Help
            <Kbd>?</Kbd>
          </button>
          <div className="flex items-center gap-2 text-xs text-text-muted mt-2">
            <span className="inline-block w-2 h-2 rounded-full bg-primary-500" />
            New
            <span className="inline-block w-2 h-2 rounded-full bg-warning" />
            Due
            {hasQuery && (
              <>
                <span className="ml-auto tabular-nums">
                  {visibleIds.size} shown
                </span>
              </>
            )}
          </div>
        </div>
      </aside>

      {showImport && <ImportDialog onClose={() => setShowImport(false)} />}
    </>
  );
}

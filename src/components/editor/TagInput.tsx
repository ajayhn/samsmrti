import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { filterInputProps } from "../../lib/filterInput";
import { getRecentTags, recordRecentTag } from "../../lib/recentTags";
import { api } from "../../lib/tauri";

interface Props {
  value: string[];
  onChange: (tags: string[]) => void;
  /** Optional ref to the tag text input (for programmatic focus). */
  inputRef?: React.RefObject<HTMLInputElement | null>;
  /** When set, ← from the tag field (empty or at start) moves focus to the card list. */
  onArrowLeftToList?: () => void;
  /** When set, ↓ from the tag field (suggestions closed) moves focus to card text. */
  onArrowDownToContent?: () => void;
}

function tagDepth(name: string): number {
  const parts = name.split("::").filter(Boolean);
  return Math.max(0, parts.length - 1);
}

function tagLeaf(name: string): string {
  const parts = name.split("::").filter(Boolean);
  return parts[parts.length - 1] ?? name;
}

function assignInputRef(
  el: HTMLInputElement | null,
  local: React.RefObject<HTMLInputElement | null>,
  external?: React.RefObject<HTMLInputElement | null>
) {
  local.current = el;
  if (external) {
    (external as React.MutableRefObject<HTMLInputElement | null>).current = el;
  }
}

export function TagInput({
  value,
  onChange,
  inputRef: externalInputRef,
  onArrowLeftToList,
  onArrowDownToContent,
}: Props) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [allTags, setAllTags] = useState<[string, string, number][]>([]);
  const [recentTags, setRecentTags] = useState<string[]>(() => getRecentTags());
  const [highlightIndex, setHighlightIndex] = useState(0);
  const containerRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLUListElement>(null);

  const refreshTags = useCallback(() => {
    api.getAllTags().then(setAllTags).catch(() => {});
  }, []);

  useEffect(() => {
    refreshTags();
  }, [refreshTags]);

  const selectedSet = useMemo(() => new Set(value), [value]);

  const usageByName = useMemo(() => {
    const m = new Map<string, number>();
    for (const [, name, count] of allTags) {
      m.set(name, count);
    }
    return m;
  }, [allTags]);

  const allNames = useMemo(() => allTags.map(([, name]) => name), [allTags]);

  const tagMatchesQuery = useCallback((name: string, q: string) => {
    const lower = name.toLowerCase();
    if (lower.includes(q)) return true;
    return name.split("::").some((part) => part.toLowerCase().includes(q));
  }, []);

  const rankSuggestions = useCallback(
    (names: string[], q: string) => {
      const recentIndex = new Map(
        recentTags.map((n, i) => [n.toLowerCase(), i])
      );
      return [...names].sort((a, b) => {
        const ra = recentIndex.get(a.toLowerCase());
        const rb = recentIndex.get(b.toLowerCase());
        if (ra !== undefined && rb !== undefined) return ra - rb;
        if (ra !== undefined) return -1;
        if (rb !== undefined) return 1;

        if (q) {
          const aLower = a.toLowerCase();
          const bLower = b.toLowerCase();
          const aLeaf = tagLeaf(a).toLowerCase();
          const bLeaf = tagLeaf(b).toLowerCase();
          const aPrefix =
            aLower.startsWith(q) || aLeaf.startsWith(q);
          const bPrefix =
            bLower.startsWith(q) || bLeaf.startsWith(q);
          if (aPrefix && !bPrefix) return -1;
          if (!aPrefix && bPrefix) return 1;
        }

        const ua = usageByName.get(a) ?? 0;
        const ub = usageByName.get(b) ?? 0;
        if (ub !== ua) return ub - ua;
        return a.localeCompare(b);
      });
    },
    [recentTags, usageByName]
  );

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    const pool = allNames.filter((name) => !selectedSet.has(name));

    if (!q) {
      const recentAvailable = recentTags.filter(
        (t) => !selectedSet.has(t) && pool.includes(t)
      );
      const rest = pool.filter((n) => !recentAvailable.includes(n));
      const sortedRest = rankSuggestions(rest, "");
      return [...recentAvailable, ...sortedRest].slice(0, 40);
    }

    const matches = pool.filter((name) => tagMatchesQuery(name, q));
    return rankSuggestions(matches, q).slice(0, 40);
  }, [
    allNames,
    query,
    selectedSet,
    recentTags,
    tagMatchesQuery,
    rankSuggestions,
  ]);

  const addTag = useCallback(
    (name: string) => {
      const trimmed = name.trim();
      if (!trimmed || selectedSet.has(trimmed)) return;
      recordRecentTag(trimmed);
      setRecentTags(getRecentTags());
      onChange([...value, trimmed]);
      setQuery("");
      setOpen(false);
      void api.getAllTags().then(setAllTags);
    },
    [onChange, selectedSet, value]
  );

  const removeTag = useCallback(
    (name: string) => {
      onChange(value.filter((t) => t !== name));
    },
    [onChange, value]
  );

  const commitQuery = useCallback(() => {
    const trimmed = query.trim();
    if (!trimmed) return;
    const exact = allNames.find(
      (n) => n.toLowerCase() === trimmed.toLowerCase() && !selectedSet.has(n)
    );
    if (exact) {
      addTag(exact);
      return;
    }
    if (filtered.length > 0) {
      addTag(filtered[0]);
      return;
    }
    addTag(trimmed);
  }, [addTag, filtered, query, selectedSet, allNames]);

  useEffect(() => {
    const onDocClick = (e: MouseEvent) => {
      if (!containerRef.current?.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", onDocClick);
    return () => document.removeEventListener("mousedown", onDocClick);
  }, []);

  const trimmedQuery = query.trim();
  const showCreate =
    trimmedQuery.length > 0 &&
    !selectedSet.has(trimmedQuery) &&
    !allNames.some((n) => n.toLowerCase() === trimmedQuery.toLowerCase());

  const optionCount = filtered.length + (showCreate ? 1 : 0);
  const listOpen = open && optionCount > 0;

  useEffect(() => {
    setHighlightIndex(0);
  }, [query, filtered.length, showCreate]);

  useEffect(() => {
    if (!listOpen) return;
    const el = listRef.current?.querySelector(
      `[data-tag-option-index="${highlightIndex}"]`
    );
    el?.scrollIntoView({ block: "nearest" });
  }, [highlightIndex, listOpen]);

  const selectHighlighted = useCallback(() => {
    if (highlightIndex < filtered.length) {
      addTag(filtered[highlightIndex]!);
      return;
    }
    if (showCreate && highlightIndex === filtered.length) {
      addTag(trimmedQuery);
      return;
    }
    commitQuery();
  }, [
    addTag,
    commitQuery,
    filtered,
    highlightIndex,
    showCreate,
    trimmedQuery,
  ]);

  return (
    <div ref={containerRef} className="relative">
      <div
        className="flex flex-wrap gap-1.5 min-h-[42px] px-3 py-2 bg-surface-alt border border-border rounded-xl focus-within:ring-2 focus-within:ring-primary-500"
        onClick={() => inputRef.current?.focus()}
      >
        {value.map((name) => (
          <span
            key={name}
            className="inline-flex items-center gap-1 px-2 py-0.5 rounded-lg bg-surface border border-border text-xs text-text"
            title={name}
          >
            <span className="max-w-[200px] truncate">{name}</span>
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                removeTag(name);
              }}
              className="text-text-muted hover:text-text cursor-pointer leading-none"
              aria-label={`Remove tag ${name}`}
            >
              ×
            </button>
          </span>
        ))}
        <input
          ref={(el) => assignInputRef(el, inputRef, externalInputRef)}
          type="text"
          value={query}
          onChange={(e) => {
            setQuery(e.target.value);
            setOpen(true);
          }}
          onFocus={() => {
            refreshTags();
            setRecentTags(getRecentTags());
            setOpen(true);
          }}
          onKeyDown={(e) => {
            if (e.key === "Escape") {
              e.preventDefault();
              setOpen(false);
              setQuery("");
              setHighlightIndex(0);
            } else if (e.key === "ArrowDown") {
              if (!listOpen && onArrowDownToContent) {
                e.preventDefault();
                e.stopPropagation();
                inputRef.current?.blur();
                onArrowDownToContent();
                return;
              }
              if (!listOpen) return;
              e.preventDefault();
              e.stopPropagation();
              setHighlightIndex((i) =>
                i + 1 >= optionCount ? 0 : i + 1
              );
            } else if (e.key === "ArrowUp") {
              if (!listOpen) return;
              e.preventDefault();
              e.stopPropagation();
              setHighlightIndex((i) =>
                i <= 0 ? optionCount - 1 : i - 1
              );
            } else if (
              e.key === "ArrowLeft" &&
              onArrowLeftToList &&
              !listOpen
            ) {
              const input = inputRef.current;
              const atStart =
                input &&
                input.selectionStart === 0 &&
                input.selectionEnd === 0;
              if (!query.trim() || atStart) {
                e.preventDefault();
                e.stopPropagation();
                input?.blur();
                onArrowLeftToList();
              }
            } else if (e.key === "Enter") {
              e.preventDefault();
              if (listOpen) {
                selectHighlighted();
              } else {
                commitQuery();
              }
            } else if (e.key === "," && query.trim()) {
              e.preventDefault();
              if (listOpen) {
                selectHighlighted();
              } else {
                commitQuery();
              }
            } else if (
              e.key === "Backspace" &&
              !query &&
              value.length > 0
            ) {
              removeTag(value[value.length - 1]);
            }
          }}
          placeholder={
            value.length === 0
              ? "Search or add tags (e.g. science::biology)"
              : "Add another tag..."
          }
          className="flex-1 min-w-[120px] bg-transparent text-sm focus:outline-none py-0.5"
          role="combobox"
          aria-expanded={listOpen}
          aria-controls="samsmrti-tag-listbox"
          aria-autocomplete="list"
          aria-activedescendant={
            listOpen ? `samsmrti-tag-opt-${highlightIndex}` : undefined
          }
          name="samsmrti-tag-combobox"
          {...filterInputProps}
        />
      </div>
      {listOpen && (
        <ul
          id="samsmrti-tag-listbox"
          ref={listRef}
          className="absolute z-20 mt-1 w-full max-h-56 overflow-y-auto bg-surface border border-border rounded-xl shadow-lg py-1"
          role="listbox"
        >
          {filtered.map((name, index) => {
            const depth = tagDepth(name);
            const leaf = tagLeaf(name);
            const hasParent = depth > 0;
            const highlighted = index === highlightIndex;
            return (
              <li key={name}>
                <button
                  id={`samsmrti-tag-opt-${index}`}
                  type="button"
                  data-tag-option-index={index}
                  onMouseEnter={() => setHighlightIndex(index)}
                  onClick={() => addTag(name)}
                  className={`w-full text-left px-3 py-2 text-sm cursor-pointer text-text flex items-baseline gap-2 min-w-0 ${
                    highlighted
                      ? "bg-primary-100 dark:bg-primary-900/30"
                      : "hover:bg-surface-alt"
                  }`}
                  role="option"
                  aria-selected={highlighted}
                  style={{ paddingLeft: `${12 + depth * 14}px` }}
                >
                  <span className="font-medium truncate">{leaf}</span>
                  {hasParent && (
                    <span className="text-xs text-text-muted truncate shrink">
                      {name}
                    </span>
                  )}
                </button>
              </li>
            );
          })}
          {showCreate && (
            <li>
              <button
                type="button"
                onClick={() => addTag(trimmedQuery)}
                className="w-full text-left px-4 py-2 text-sm hover:bg-surface-alt cursor-pointer text-primary-600 border-t border-border"
                role="option"
              >
                Create tag &ldquo;{trimmedQuery}&rdquo;
              </button>
            </li>
          )}
        </ul>
      )}
      <p className="mt-1.5 text-xs text-text-muted">
        ↑↓ to move in suggestions, Enter to add. Recently used tags are listed
        first. Hierarchy uses{" "}
        <code className="text-text-secondary">parent::child</code> in the name.
      </p>
    </div>
  );
}

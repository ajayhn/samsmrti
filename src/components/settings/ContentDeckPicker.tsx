import { useEffect, useMemo, useState } from "react";
import type { ContentDeckOption } from "../../lib/tauri";

interface Props {
  title: string;
  description: string;
  decks: ContentDeckOption[];
  confirmLabel: string;
  onConfirm: (deckIds: string[]) => void;
  onCancel: () => void;
}

function depthFor(deck: ContentDeckOption, byId: Map<string, ContentDeckOption>): number {
  let depth = 0;
  let parentId = deck.parent_id;
  const seen = new Set<string>([deck.id]);
  while (parentId) {
    if (seen.has(parentId)) break;
    seen.add(parentId);
    depth += 1;
    parentId = byId.get(parentId)?.parent_id ?? null;
  }
  return depth;
}

export function ContentDeckPicker({
  title,
  description,
  decks,
  confirmLabel,
  onConfirm,
  onCancel,
}: Props) {
  const [selected, setSelected] = useState<Set<string>>(() => new Set(decks.map((d) => d.id)));

  useEffect(() => {
    setSelected(new Set(decks.map((d) => d.id)));
  }, [decks]);

  const sorted = useMemo(() => {
    const byId = new Map(decks.map((d) => [d.id, d]));
    return [...decks].sort((a, b) => {
      const da = depthFor(a, byId);
      const db = depthFor(b, byId);
      if (da !== db) return da - db;
      return a.name.localeCompare(b.name);
    });
  }, [decks]);

  const byId = useMemo(() => new Map(decks.map((d) => [d.id, d])), [decks]);

  const allSelected = selected.size === decks.length && decks.length > 0;
  const noneSelected = selected.size === 0;

  const toggle = (id: string) => {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const selectAll = () => setSelected(new Set(decks.map((d) => d.id)));
  const clearAll = () => setSelected(new Set());

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4">
      <div className="bg-surface border border-border rounded-2xl shadow-xl w-full max-w-md max-h-[85vh] flex flex-col">
        <div className="p-4 border-b border-border">
          <h3 className="text-lg font-semibold text-text">{title}</h3>
          <p className="text-xs text-text-muted mt-1">{description}</p>
        </div>

        <div className="flex items-center gap-2 px-4 py-2 border-b border-border text-xs">
          <button
            type="button"
            onClick={selectAll}
            disabled={allSelected}
            className="text-primary-600 hover:underline disabled:opacity-40 cursor-pointer"
          >
            Select all
          </button>
          <span className="text-text-muted">·</span>
          <button
            type="button"
            onClick={clearAll}
            disabled={noneSelected}
            className="text-primary-600 hover:underline disabled:opacity-40 cursor-pointer"
          >
            Clear
          </button>
          <span className="ml-auto text-text-muted">
            {selected.size} / {decks.length}
          </span>
        </div>

        <ul className="flex-1 overflow-y-auto p-2 space-y-0.5 min-h-[120px]">
          {sorted.length === 0 ? (
            <li className="text-sm text-text-muted px-3 py-4">No decks in file.</li>
          ) : (
            sorted.map((deck) => {
              const depth = depthFor(deck, byId);
              return (
                <li key={deck.id}>
                  <label
                    className="flex items-center gap-2 px-2 py-2 rounded-lg hover:bg-surface-alt cursor-pointer"
                    style={{ paddingLeft: `${8 + depth * 16}px` }}
                  >
                    <input
                      type="checkbox"
                      checked={selected.has(deck.id)}
                      onChange={() => toggle(deck.id)}
                      className="rounded border-border"
                    />
                    <span className="text-sm text-text flex-1 truncate">{deck.name}</span>
                    <span className="text-xs text-text-muted shrink-0" title="Includes notes in subdecks">
                      {deck.note_count} note{deck.note_count !== 1 ? "s" : ""}
                    </span>
                  </label>
                </li>
              );
            })
          )}
        </ul>

        <div className="p-4 border-t border-border flex gap-2 justify-end">
          <button
            type="button"
            onClick={onCancel}
            className="px-4 py-2 text-sm text-text-secondary hover:text-text cursor-pointer"
          >
            Cancel
          </button>
          <button
            type="button"
            disabled={noneSelected}
            onClick={() => onConfirm([...selected])}
            className="px-4 py-2 bg-primary-600 text-white text-sm font-medium rounded-lg hover:bg-primary-700 disabled:opacity-50 cursor-pointer"
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}

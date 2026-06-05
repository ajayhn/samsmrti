import { useState, useEffect } from "react";
import { useDeckStore } from "../../stores/deckStore";
import { filterInputProps } from "../../lib/filterInput";
import type { DeckWithCounts } from "../../lib/tauri";

interface DeckSettingsProps {
  deck: DeckWithCounts;
  onClose: () => void;
}

function isDescendantOf(
  decks: DeckWithCounts[],
  ancestorId: string,
  deckId: string
): boolean {
  let current = decks.find((d) => d.id === deckId);
  while (current?.parent_id) {
    if (current.parent_id === ancestorId) return true;
    current = decks.find((d) => d.id === current!.parent_id);
  }
  return false;
}

export function DeckSettings({ deck, onClose }: DeckSettingsProps) {
  const { decks, updateDeck } = useDeckStore();
  const [name, setName] = useState(deck.name);
  const [description, setDescription] = useState(deck.description);
  const [parentId, setParentId] = useState<string>(deck.parent_id ?? "");
  const [newPerDay, setNewPerDay] = useState(deck.new_per_day);
  const [maxReviews, setMaxReviews] = useState(deck.max_reviews);
  const [saving, setSaving] = useState(false);

  const parentOptions = decks.filter(
    (d) =>
      d.id !== deck.id &&
      !isDescendantOf(decks, deck.id, d.id)
  );

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  const handleSave = async () => {
    setSaving(true);
    try {
      await updateDeck(deck.id, {
        name,
        description,
        parent_id: parentId || null,
        new_per_day: newPerDay,
        max_reviews: maxReviews,
      });
      onClose();
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/40 flex items-center justify-center z-50 p-4">
      <div className="bg-surface rounded-2xl border border-border shadow-xl w-full max-w-md p-6 space-y-5">
        <h3 className="text-lg font-bold text-text">Deck Settings</h3>

        <div>
          <label className="block text-sm font-medium text-text-secondary mb-1">
            Name
          </label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            className="w-full px-3 py-2 bg-surface-alt border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
            name="samsmrti-deck-name"
            {...filterInputProps}
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-text-secondary mb-1">
            Parent deck
          </label>
          <select
            value={parentId}
            onChange={(e) => setParentId(e.target.value)}
            className="w-full px-3 py-2 bg-surface-alt border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
          >
            <option value="">(Top level)</option>
            {parentOptions.map((d) => (
              <option key={d.id} value={d.id}>
                {d.name}
              </option>
            ))}
          </select>
          <p className="text-xs text-text-muted mt-1">
            Subdecks appear nested in the sidebar. Studying this deck includes all subdecks.
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium text-text-secondary mb-1">
            Description
          </label>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            rows={2}
            className="w-full px-3 py-2 bg-surface-alt border border-border rounded-lg text-sm resize-none focus:outline-none focus:ring-2 focus:ring-primary-500"
          />
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-sm font-medium text-text-secondary mb-1">
              New cards / day
            </label>
            <input
              type="number"
              value={newPerDay}
              onChange={(e) => setNewPerDay(parseInt(e.target.value) || 0)}
              min={0}
              max={9999}
              className="w-full px-3 py-2 bg-surface-alt border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-text-secondary mb-1">
              Max reviews / day
            </label>
            <input
              type="number"
              value={maxReviews}
              onChange={(e) => setMaxReviews(parseInt(e.target.value) || 0)}
              min={0}
              max={9999}
              className="w-full px-3 py-2 bg-surface-alt border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
            />
          </div>
        </div>

        <div className="flex gap-3 pt-2">
          <button
            onClick={handleSave}
            disabled={saving || !name.trim()}
            className="flex-1 px-4 py-2.5 bg-primary-600 text-white rounded-xl text-sm font-medium hover:bg-primary-700 disabled:opacity-50 transition-colors"
          >
            {saving ? "Saving..." : "Save"}
          </button>
          <button
            onClick={onClose}
            className="px-4 py-2.5 text-text-secondary hover:bg-surface-hover rounded-xl text-sm transition-colors"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}

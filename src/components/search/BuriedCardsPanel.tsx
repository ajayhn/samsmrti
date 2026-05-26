import { useState, useEffect, useCallback } from "react";
import { api, type BuriedCard } from "../../lib/tauri";
import { useDeckStore } from "../../stores/deckStore";
import { renderTemplate } from "../../lib/cloze";

function formatUnburyTime(ts: number): string {
  const date = new Date(ts * 1000);
  return date.toLocaleString(undefined, {
    weekday: "short",
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  });
}

export function BuriedCardsPanel() {
  const [query, setQuery] = useState("");
  const [selectedDeck, setSelectedDeck] = useState("");
  const [cards, setCards] = useState<BuriedCard[]>([]);
  const [loading, setLoading] = useState(false);
  const { decks, fetchDecks } = useDeckStore();

  const loadBuried = useCallback(async () => {
    setLoading(true);
    try {
      const res = await api.getBuriedCards(
        query.trim() || undefined,
        selectedDeck || undefined,
        200
      );
      setCards(res);
    } catch (e) {
      console.error(e);
      setCards([]);
    } finally {
      setLoading(false);
    }
  }, [query, selectedDeck]);

  useEffect(() => {
    fetchDecks();
  }, [fetchDecks]);

  useEffect(() => {
    const timeout = setTimeout(loadBuried, 300);
    return () => clearTimeout(timeout);
  }, [loadBuried]);

  const handleUnbury = async (cardId: string) => {
    await api.unburyCard(cardId);
    await loadBuried();
    fetchDecks();
  };

  const getPreview = (card: BuriedCard) => {
    const rendered = renderTemplate(
      card.front_html,
      card.fields,
      false,
      0,
      false
    );
    const stripped = rendered.replace(/<[^>]*>/g, "").trim();
    return stripped.length > 120 ? stripped.slice(0, 120) + "..." : stripped;
  };

  return (
    <div className="flex flex-col flex-1 min-h-0">
      <div className="flex gap-3 mb-4">
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search buried cards..."
          className="flex-1 px-4 py-2.5 bg-surface-alt border border-border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
        />
        <select
          value={selectedDeck}
          onChange={(e) => setSelectedDeck(e.target.value)}
          className="px-3 py-2.5 bg-surface-alt border border-border rounded-xl text-sm"
        >
          <option value="">All Decks</option>
          {decks.map((d) => (
            <option key={d.id} value={d.id}>
              {d.name}
            </option>
          ))}
        </select>
      </div>

      <div className="flex-1 overflow-y-auto space-y-2">
        {loading && (
          <div className="text-center text-text-muted text-sm py-4">
            Loading buried cards...
          </div>
        )}

        {!loading && cards.length === 0 && (
          <div className="text-center text-text-muted text-sm py-8">
            No buried cards. Press <kbd className="px-1 py-0.5 rounded border border-border text-xs">9</kbd> during review to bury a card until tomorrow.
          </div>
        )}

        {cards.map((c) => (
          <div
            key={c.card_id}
            className="bg-surface-alt border border-border rounded-xl px-4 py-3 flex items-center justify-between gap-4"
          >
            <div className="min-w-0 flex-1">
              <p className="text-sm font-medium text-text truncate">
                {getPreview(c)}
              </p>
              <div className="flex items-center gap-2 mt-1 flex-wrap">
                <span className="text-xs text-text-muted">{c.deck_name}</span>
                <span className="text-xs text-text-muted">{c.state}</span>
                <span className="text-xs text-amber-600 dark:text-amber-400">
                  Unburies {formatUnburyTime(c.buried_until)}
                </span>
              </div>
            </div>
            <button
              onClick={() => handleUnbury(c.card_id)}
              className="shrink-0 px-3 py-1.5 text-sm font-medium text-primary-600 hover:text-primary-700 border border-primary-200 dark:border-primary-800 rounded-lg hover:bg-primary-50 dark:hover:bg-primary-900/20 transition-colors cursor-pointer"
            >
              Unbury
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}

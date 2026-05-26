import { useState, useEffect } from "react";
import { api, type CardSuggestion } from "../../lib/tauri";
import { useDeckStore } from "../../stores/deckStore";

interface Props {
  entityId?: string | null;
  onClose: () => void;
}

export function SuggestCards({ entityId, onClose }: Props) {
  const [suggestions, setSuggestions] = useState<CardSuggestion[]>([]);
  const [loading, setLoading] = useState(true);
  const [acceptedIds, setAcceptedIds] = useState<Set<number>>(new Set());
  const [dismissedIds, setDismissedIds] = useState<Set<number>>(new Set());
  const { selectedDeckId, fetchDecks } = useDeckStore();

  useEffect(() => {
    loadSuggestions();
  }, [entityId]);

  const loadSuggestions = async () => {
    setLoading(true);
    try {
      const results = await api.suggestCardsFromTriples(entityId || undefined);
      setSuggestions(results);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  const handleAccept = async (idx: number) => {
    const suggestion = suggestions[idx];
    if (!selectedDeckId || !suggestion) return;

    try {
      const { note, karma } = await api.createNote({
        deck_id: selectedDeckId,
        note_type_id: "nt_basic",
        fields: {
          Front: suggestion.front,
          Back: suggestion.back,
        },
        tags: ["auto-suggested"],
      });
      const { useKarmaStore } = await import("../../stores/karmaStore");
      useKarmaStore.getState().applyEarn(karma);

      const cards = await api.getCardsForNote(note.id);
      if (cards.length > 0) {
        for (const tripleId of suggestion.triple_ids) {
          await api.linkCardToTriple(cards[0].id, tripleId);
        }
      }

      setAcceptedIds((prev) => new Set([...prev, idx]));
      await fetchDecks();
    } catch (e) {
      console.error(e);
    }
  };

  const handleDismiss = (idx: number) => {
    setDismissedIds((prev) => new Set([...prev, idx]));
  };

  const visibleSuggestions = suggestions
    .map((s, i) => ({ ...s, idx: i }))
    .filter(({ idx }) => !acceptedIds.has(idx) && !dismissedIds.has(idx));

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-surface rounded-2xl shadow-2xl w-full max-w-2xl max-h-[80vh] flex flex-col">
        <div className="flex items-center justify-between px-6 py-4 border-b border-border">
          <div>
            <h3 className="text-lg font-bold text-text">Card Suggestions</h3>
            <p className="text-sm text-text-muted mt-0.5">
              {entityId ? "Suggestions for this entity" : "Cards from unlinked triples"}
            </p>
          </div>
          <button
            onClick={onClose}
            className="text-text-muted hover:text-text text-xl px-2 cursor-pointer"
          >
            &times;
          </button>
        </div>

        <div className="flex-1 overflow-y-auto p-4 space-y-3">
          {loading && (
            <div className="text-center text-text-muted py-8 text-sm">
              Loading suggestions...
            </div>
          )}

          {!loading && visibleSuggestions.length === 0 && (
            <div className="text-center text-text-muted py-8">
              <p className="text-sm">No suggestions available.</p>
              <p className="text-xs mt-1">All triples already have linked cards.</p>
            </div>
          )}

          {visibleSuggestions.map(({ idx, front, back, suggestion_type, triple_ids }) => (
            <div
              key={idx}
              className="border border-border rounded-xl p-4 bg-surface-alt"
            >
              <div className="flex items-start justify-between gap-4">
                <div className="flex-1 min-w-0">
                  <span
                    className={`inline-block text-[10px] uppercase font-semibold px-2 py-0.5 rounded mb-2 ${
                      suggestion_type === "forward"
                        ? "bg-primary-100 dark:bg-primary-900/30 text-primary-600"
                        : "bg-amber-100 dark:bg-amber-900/30 text-amber-600"
                    }`}
                  >
                    {suggestion_type}
                  </span>

                  <div className="grid grid-cols-2 gap-3 mt-2">
                    <div>
                      <p className="text-[10px] font-medium text-text-muted uppercase tracking-wider mb-1">
                        Front
                      </p>
                      <p className="text-sm text-text">{front}</p>
                    </div>
                    <div>
                      <p className="text-[10px] font-medium text-text-muted uppercase tracking-wider mb-1">
                        Back
                      </p>
                      <p className="text-sm text-text">{back}</p>
                    </div>
                  </div>

                  <p className="text-[10px] text-text-muted mt-2">
                    {triple_ids.length} triple{triple_ids.length !== 1 ? "s" : ""}
                  </p>
                </div>

                <div className="flex flex-col gap-2 shrink-0">
                  <button
                    onClick={() => handleAccept(idx)}
                    disabled={!selectedDeckId}
                    className="px-3 py-1.5 bg-primary-600 text-white text-xs font-medium rounded-lg hover:bg-primary-700 disabled:opacity-40 transition-colors cursor-pointer"
                  >
                    Accept
                  </button>
                  <button
                    onClick={() => handleDismiss(idx)}
                    className="px-3 py-1.5 bg-surface border border-border text-text-secondary text-xs font-medium rounded-lg hover:bg-surface-hover transition-colors cursor-pointer"
                  >
                    Dismiss
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>

        <div className="px-6 py-3 border-t border-border text-xs text-text-muted flex justify-between items-center">
          <span>
            {acceptedIds.size} accepted, {dismissedIds.size} dismissed of{" "}
            {suggestions.length} total
          </span>
          <button
            onClick={onClose}
            className="px-4 py-1.5 text-sm text-text-secondary hover:text-text transition-colors cursor-pointer"
          >
            Done
          </button>
        </div>
      </div>
    </div>
  );
}

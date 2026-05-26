import { useState, useEffect, useCallback } from "react";
import {
  api,
  type ReviewCard,
  type EreReviewEntity,
  type IntervalPreview,
  type AnswerResult,
} from "../../lib/tauri";
import { useDeckStore } from "../../stores/deckStore";
import { useActivityTracker } from "../../hooks/useActivityTracker";
import { renderTemplate } from "../../lib/cloze";
import { isNativeEditShortcut } from "../../lib/keyboard";

interface Props {
  deckId: string;
  onClose: () => void;
}

const RATING_LABELS = ["Again", "Hard", "Good", "Easy"] as const;
const RATING_COLORS = [
  "bg-danger hover:bg-red-600",
  "bg-orange-500 hover:bg-orange-600",
  "bg-success hover:bg-green-600",
  "bg-primary-500 hover:bg-primary-600",
];

export function EreReview({ deckId, onClose }: Props) {
  const [entities, setEntities] = useState<EreReviewEntity[]>([]);
  const [loading, setLoading] = useState(true);
  const [activeEntityId, setActiveEntityId] = useState<string | null>(null);
  useActivityTracker(activeEntityId != null);
  const [queue, setQueue] = useState<ReviewCard[]>([]);
  const [currentIndex, setCurrentIndex] = useState(0);
  const [isFlipped, setIsFlipped] = useState(false);
  const [intervalPreview, setIntervalPreview] = useState<IntervalPreview | null>(null);
  const [results, setResults] = useState<AnswerResult[]>([]);
  const [startTime, setStartTime] = useState(Date.now());
  const { fetchDecks } = useDeckStore();

  const loadSummary = useCallback(async () => {
    setLoading(true);
    try {
      const summary = await api.getEreReviewSummary(deckId);
      setEntities(summary);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  }, [deckId]);

  useEffect(() => {
    loadSummary();
  }, [loadSummary]);

  const startEntityReview = async (entityId: string) => {
    try {
      const cards = await api.getEreDueCards(deckId, entityId);
      if (cards.length === 0) return;
      setActiveEntityId(entityId);
      setQueue(cards);
      setCurrentIndex(0);
      setIsFlipped(false);
      setResults([]);
      setStartTime(Date.now());
      setIntervalPreview(null);
    } catch (e) {
      console.error(e);
    }
  };

  const currentCard = queue[currentIndex] ?? null;
  const remaining = queue.length - currentIndex;

  const flipCard = async () => {
    setIsFlipped(true);
    if (currentCard) {
      try {
        const preview = await api.getIntervalPreview(currentCard.card_id);
        setIntervalPreview(preview);
      } catch {
        // ignore
      }
    }
  };

  const answerCard = async (rating: number) => {
    if (!currentCard) return;
    const elapsed = Date.now() - startTime;
    const result = await api.answerCard({
      card_id: currentCard.card_id,
      rating,
      elapsed_ms: elapsed,
    });
    const { useKarmaStore } = await import("../../stores/karmaStore");
    useKarmaStore.getState().applyEarn(result.karma);
    setResults((prev) => [...prev, result]);
    setCurrentIndex((i) => i + 1);
    setIsFlipped(false);
    setIntervalPreview(null);
    setStartTime(Date.now());
    fetchDecks();
  };

  const undoLast = async () => {
    if (results.length === 0) return;
    const lastResult = results[results.length - 1];
    const undo = await api.undoReview(lastResult.review_log_id);
    const { useKarmaStore } = await import("../../stores/karmaStore");
    useKarmaStore.getState().applyEarn(undo.karma);
    setResults((prev) => prev.slice(0, -1));
    setCurrentIndex((i) => i - 1);
    setIsFlipped(false);
    setIntervalPreview(null);
    fetchDecks();
  };

  const finishEntity = () => {
    setActiveEntityId(null);
    setQueue([]);
    setCurrentIndex(0);
    setResults([]);
    loadSummary();
  };

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (isNativeEditShortcut(e)) return;
      if (!activeEntityId) {
        if (e.key === "Escape") onClose();
        return;
      }

      const entityDone = currentIndex >= queue.length;

      if (entityDone) {
        if (e.key === "Escape" || e.key === "Enter") finishEntity();
        return;
      }

      if (e.key === " " && !isFlipped) {
        e.preventDefault();
        flipCard();
      } else if (isFlipped && ["1", "2", "3", "4"].includes(e.key)) {
        answerCard(parseInt(e.key));
      } else if (e.key === "z" && (e.ctrlKey || e.metaKey) && results.length > 0) {
        e.preventDefault();
        undoLast();
      } else if (e.key === "Escape") {
        finishEntity();
      }
    },
    [activeEntityId, currentIndex, queue.length, isFlipped, results.length]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  // Entity done screen
  if (activeEntityId && currentIndex >= queue.length) {
    const entityName = entities.find((e) => e.entity_id === activeEntityId)?.entity_name ?? "Entity";
    return (
      <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
        <div className="bg-surface rounded-2xl shadow-2xl w-full max-w-lg p-8 text-center">
          <h3 className="text-xl font-bold text-text mb-2">
            {entityName} - Complete
          </h3>
          <p className="text-text-secondary mb-6">
            Reviewed {results.length} card{results.length !== 1 ? "s" : ""}.
          </p>
          <button
            onClick={finishEntity}
            className="px-6 py-3 bg-primary-600 text-white rounded-xl font-medium hover:bg-primary-700 transition-colors cursor-pointer"
          >
            Back to Entity List
          </button>
        </div>
      </div>
    );
  }

  // Active review for an entity
  if (activeEntityId && currentCard) {
    const frontContent = renderTemplate(
      currentCard.front_html,
      currentCard.fields,
      currentCard.is_cloze,
      currentCard.template_ordinal,
      false
    );
    const backContent = renderTemplate(
      currentCard.back_html,
      currentCard.fields,
      currentCard.is_cloze,
      currentCard.template_ordinal,
      true,
      currentCard.front_html
    );

    const entityName = entities.find((e) => e.entity_id === activeEntityId)?.entity_name ?? "Entity";
    const intervals = intervalPreview
      ? [intervalPreview.again, intervalPreview.hard, intervalPreview.good, intervalPreview.easy]
      : null;

    return (
      <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
        <div className="bg-surface rounded-2xl shadow-2xl w-full max-w-2xl max-h-[90vh] flex flex-col">
          {/* Header */}
          <div className="flex items-center justify-between px-6 py-3 border-b border-border">
            <div className="flex items-center gap-3">
              <button
                onClick={finishEntity}
                className="text-sm text-text-secondary hover:text-text transition-colors cursor-pointer"
              >
                Back to List
              </button>
              {results.length > 0 && (
                <button
                  onClick={undoLast}
                  className="text-sm text-primary-500 hover:text-primary-600 transition-colors cursor-pointer"
                >
                  Undo
                </button>
              )}
            </div>
            <span className="text-sm text-text-muted">
              {entityName} &middot; {remaining} remaining
            </span>
          </div>

          {/* Progress */}
          <div className="h-1 bg-surface-alt">
            <div
              className="h-full bg-primary-500 transition-all duration-300"
              style={{
                width: `${((currentIndex + 1) / queue.length) * 100}%`,
              }}
            />
          </div>

          {/* Card */}
          <div className="flex-1 flex items-center justify-center p-8 overflow-y-auto">
            <div className="w-full max-w-xl">
              <style>{currentCard.css}</style>
              <div className="bg-surface-alt rounded-2xl border border-border p-8 min-h-[200px] flex flex-col items-center justify-center">
                <div
                  className="text-lg text-center leading-relaxed w-full prose prose-stone dark:prose-invert max-w-none"
                  dangerouslySetInnerHTML={{
                    __html: isFlipped ? backContent : frontContent,
                  }}
                />
              </div>

              <div className="mt-5">
                {!isFlipped ? (
                  <button
                    onClick={flipCard}
                    className="w-full py-4 bg-surface-alt border border-border rounded-xl text-text font-medium hover:bg-surface-hover transition-colors cursor-pointer"
                  >
                    Show Answer
                    <span className="text-text-muted text-sm ml-2">(Space)</span>
                  </button>
                ) : (
                  <div className="grid grid-cols-4 gap-3">
                    {RATING_LABELS.map((label, i) => (
                      <button
                        key={label}
                        onClick={() => answerCard(i + 1)}
                        className={`py-3 rounded-xl text-white font-medium transition-colors cursor-pointer ${RATING_COLORS[i]}`}
                      >
                        {label}
                        {intervals && (
                          <span className="block text-xs opacity-75 mt-0.5">
                            {intervals[i]}
                          </span>
                        )}
                      </button>
                    ))}
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>
    );
  }

  // Entity list view
  const totalDue = entities.reduce((sum, e) => sum + e.due_count, 0);

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-surface rounded-2xl shadow-2xl w-full max-w-lg max-h-[80vh] flex flex-col">
        <div className="flex items-center justify-between px-6 py-4 border-b border-border">
          <div>
            <h3 className="text-lg font-bold text-text">Review by Entity</h3>
            <p className="text-sm text-text-muted mt-0.5">
              {totalDue} card{totalDue !== 1 ? "s" : ""} due across{" "}
              {entities.length} entit{entities.length !== 1 ? "ies" : "y"}
            </p>
          </div>
          <button
            onClick={onClose}
            className="text-text-muted hover:text-text text-xl px-2 cursor-pointer"
          >
            &times;
          </button>
        </div>

        <div className="flex-1 overflow-y-auto p-4">
          {loading && (
            <div className="text-center text-text-muted py-8 text-sm">Loading...</div>
          )}

          {!loading && entities.length === 0 && (
            <div className="text-center text-text-muted py-8 text-sm">
              No E-R-E cards are due right now.
            </div>
          )}

          <div className="space-y-2">
            {entities.map((entity) => (
              <button
                key={entity.entity_id}
                onClick={() => startEntityReview(entity.entity_id)}
                className="w-full flex items-center justify-between px-4 py-3 bg-surface-alt border border-border rounded-xl hover:bg-surface-hover transition-colors cursor-pointer"
              >
                <span className="font-medium text-text">{entity.entity_name}</span>
                <span className="text-sm text-warning font-semibold">
                  {entity.due_count} due
                </span>
              </button>
            ))}
          </div>
        </div>

        <div className="px-6 py-3 border-t border-border">
          <button
            onClick={onClose}
            className="w-full py-2 text-sm text-text-secondary hover:text-text transition-colors cursor-pointer"
          >
            Close (Esc)
          </button>
        </div>
      </div>
    </div>
  );
}

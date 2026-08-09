import { useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { KarmaDisplay } from "../components/karma/KarmaDisplay";
import { ProfileOnboarding } from "../components/profile/ProfileOnboarding";
import { useActivityTracker } from "../hooks/useActivityTracker";
import { useReviewStore } from "../stores/reviewStore";
import { useDeckStore } from "../stores/deckStore";
import { renderStudyContent } from "../lib/cloze";

const RATING_LABELS: Record<number, string> = {
  1: "Again",
  2: "Hard",
  3: "Good",
  4: "Easy",
};

export function ReviewSessionPage() {
  const { deckId } = useParams<{ deckId: string }>();
  const navigate = useNavigate();
  const {
    queue,
    currentIndex,
    isFlipped,
    sessionActive,
    loading,
    intervalPreview,
    startSession,
    flipCard,
    answerCard,
    endSession,
    currentCard,
    remaining,
    canUndo,
    undoLast,
  } = useReviewStore();
  const fetchDecks = useDeckStore((s) => s.fetchDecks);

  useActivityTracker(sessionActive);

  useEffect(() => {
    if (deckId) startSession(deckId);
    return () => endSession();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [deckId]);

  const card = currentCard();

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full text-text-muted">
        Loading...
      </div>
    );
  }

  if (!sessionActive || !card) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-4 p-6 text-center">
        <p className="text-lg text-text">All caught up for now.</p>
        <button
          onClick={() => {
            fetchDecks();
            navigate("/");
          }}
          className="px-6 py-3 bg-primary-600 text-white rounded-xl font-medium"
        >
          Back to Decks
        </button>
      </div>
    );
  }

  const cardContent = renderStudyContent(
    card.fields,
    card.front_html,
    card.back_html,
    card.is_cloze,
    card.template_ordinal,
    isFlipped
  );

  const intervals = intervalPreview
    ? [intervalPreview.again, intervalPreview.hard, intervalPreview.good, intervalPreview.easy]
    : null;

  return (
    <div className="flex flex-col h-full">
      <div className="h-1 bg-surface-alt">
        <div
          className="h-full bg-primary-500 transition-all duration-300"
          style={{ width: `${((currentIndex + 1) / queue.length) * 100}%` }}
        />
      </div>

      <div className="flex items-center justify-between px-4 py-3 border-b border-border">
        <div className="flex items-center gap-3">
          <button
            onClick={() => {
              endSession();
              navigate("/");
            }}
            className="text-sm text-text-secondary"
          >
            End
          </button>
          {canUndo() && (
            <button onClick={() => undoLast()} className="text-sm text-primary-500">
              Undo
            </button>
          )}
        </div>
        <div className="flex items-center gap-3">
          <KarmaDisplay />
          <span className="text-sm text-text-muted">{remaining()} left</span>
        </div>
      </div>
      <ProfileOnboarding />

      <div className="flex-1 flex items-center justify-center p-4">
        <div className="w-full max-w-2xl">
          <style>{card.css}</style>
          <div className="bg-surface-alt rounded-2xl border border-border shadow-sm p-8 min-h-[280px] flex items-center justify-center">
            <div
              className="text-lg text-center leading-relaxed w-full prose prose-stone dark:prose-invert max-w-none select-text whitespace-pre-wrap"
              dangerouslySetInnerHTML={{ __html: cardContent }}
            />
          </div>

          <div className="mt-6">
            {!isFlipped ? (
              <button
                onClick={() => flipCard()}
                className="w-full py-4 bg-primary-600 text-white rounded-xl font-medium text-lg"
              >
                Show Answer
              </button>
            ) : (
              <div className="grid grid-cols-4 gap-2">
                {[1, 2, 3, 4].map((rating) => (
                  <button
                    key={rating}
                    onClick={() => answerCard(rating)}
                    className={`py-4 rounded-xl font-medium text-white ${
                      rating === 1
                        ? "bg-red-500"
                        : rating === 2
                          ? "bg-orange-500"
                          : rating === 3
                            ? "bg-green-500"
                            : "bg-blue-500"
                    }`}
                  >
                    <div>{RATING_LABELS[rating]}</div>
                    {intervals && (
                      <div className="text-xs opacity-80">{intervals[rating - 1]}</div>
                    )}
                  </button>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}


import { useEffect, useCallback, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { KarmaDisplay } from "../karma/KarmaDisplay";
import { ProfileOnboarding } from "../profile/ProfileOnboarding";
import { useActivityTracker } from "../../hooks/useActivityTracker";
import { useKarmaStore } from "../../stores/karmaStore";
import { useProfileStore } from "../../stores/profileStore";
import { useReviewStore } from "../../stores/reviewStore";
import {
  countClozeDeletions,
  isProgressiveQuizbowlCard,
  progressiveRevealLabel,
  renderStudyContent,
} from "../../lib/cloze";
import { useDeckStore } from "../../stores/deckStore";
import { api, type ReviewStats } from "../../lib/tauri";
import { NoteTagsPanel } from "../notes/NoteTagsPanel";
import { ReviewNoteEditor } from "./ReviewNoteEditor";
import { isNativeEditShortcut } from "../../lib/keyboard";
import { isTypingTarget } from "../../lib/isTypingTarget";

const RATING_LABELS = ["Again", "Hard", "Good", "Easy"] as const;
const RATING_KEYS = ["1", "2", "3", "4"];
const RATING_COLORS = [
  "bg-danger hover:bg-red-600",
  "bg-orange-500 hover:bg-orange-600",
  "bg-success hover:bg-green-600",
  "bg-primary-500 hover:bg-primary-600",
];

export function ReviewSession() {
  const { deckId } = useParams<{ deckId: string }>();
  const navigate = useNavigate();
  const fetchProfiles = useProfileStore((s) => s.fetchProfiles);
  const fetchKarma = useKarmaStore((s) => s.fetchKarma);
  useActivityTracker(true);

  useEffect(() => {
    fetchProfiles().then(() => fetchKarma());
  }, [fetchProfiles, fetchKarma]);

  const store = useReviewStore();
  const {
    sessionActive,
    isFlipped,
    loading,
    startSession,
    flipCard,
    answerCard,
    buryCard,
    deleteCard,
    undoLast,
    endSession,
    updateNoteFields,
    currentCard,
    remaining,
    queue,
    currentIndex,
    canUndo,
    intervalPreview,
    actions,
  } = store;
  const { fetchDecks } = useDeckStore();
  const [stats, setStats] = useState<ReviewStats | null>(null);
  const [editOpen, setEditOpen] = useState(false);
  const [revealStep, setRevealStep] = useState(0);
  const [isFlagged, setIsFlagged] = useState(false);
  const [flagging, setFlagging] = useState(false);

  const card = currentCard();
  const sessionDone = !sessionActive || !card;
  const progressiveQuizbowl =
    card != null && isProgressiveQuizbowlCard(card.is_cloze, card.fields);
  const progressiveTotal = progressiveQuizbowl
    ? countClozeDeletions(card.fields.Text ?? "")
    : 0;

  useEffect(() => {
    setRevealStep(0);
  }, [card?.card_id]);

  useEffect(() => {
    if (!card?.card_id) {
      setIsFlagged(false);
      return;
    }
    api
      .getCardFlag(card.card_id)
      .then(setIsFlagged)
      .catch(() => setIsFlagged(false));
  }, [card?.card_id]);

  const handleReveal = useCallback(() => {
    if (!card || isFlipped) return;
    if (progressiveQuizbowl && revealStep < progressiveTotal) {
      const next = revealStep + 1;
      setRevealStep(next);
      if (next >= progressiveTotal) {
        flipCard();
      }
      return;
    }
    flipCard();
  }, [
    card,
    isFlipped,
    progressiveQuizbowl,
    revealStep,
    progressiveTotal,
    flipCard,
  ]);

  const reviewedCount = actions.filter((a) => a.type === "answer").length;

  useEffect(() => {
    if (deckId) {
      startSession(deckId);
    }
    return () => endSession();
  }, [deckId, startSession, endSession]);

  useEffect(() => {
    if (!sessionActive && queue.length > 0 && deckId) {
      api.getReviewStats(deckId).then(setStats).catch(() => {});
    }
  }, [sessionActive, queue.length, deckId]);

  const handleAnswer = useCallback(
    async (rating: number) => {
      await answerCard(rating);
      fetchDecks();
    },
    [answerCard, fetchDecks]
  );

  const handleBury = useCallback(async () => {
    await buryCard();
    fetchDecks();
  }, [buryCard, fetchDecks]);

  const handleDelete = useCallback(async () => {
    await deleteCard();
    fetchDecks();
  }, [deleteCard, fetchDecks]);

  const handleFlag = useCallback(async () => {
    if (!card || flagging) return;
    const next = !isFlagged;
    setFlagging(true);
    try {
      await api.setCardFlag(card.card_id, next);
      setIsFlagged(next);
    } finally {
      setFlagging(false);
    }
  }, [card, flagging, isFlagged]);

  const handleUndo = useCallback(async () => {
    await undoLast();
    fetchDecks();
  }, [undoLast, fetchDecks]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (loading) return;
      if (isNativeEditShortcut(e)) return;

      if (editOpen) {
        if (e.key === "Escape") {
          e.preventDefault();
          setEditOpen(false);
        }
        return;
      }

      if (sessionDone) {
        if (e.key === "Escape") {
          fetchDecks();
          navigate("/");
        }
        return;
      }

      if (
        e.key.toLowerCase() === "e" &&
        !e.ctrlKey &&
        !e.metaKey &&
        !e.altKey &&
        !isTypingTarget(e.target)
      ) {
        e.preventDefault();
        setEditOpen(true);
        return;
      }

      if (e.key === " " && !isFlipped) {
        e.preventDefault();
        handleReveal();
      } else if (isFlipped && RATING_KEYS.includes(e.key)) {
        e.preventDefault();
        handleAnswer(parseInt(e.key, 10));
      } else if (e.key === "9") {
        e.preventDefault();
        handleBury();
      } else if (
        e.key.toLowerCase() === "f" &&
        !e.ctrlKey &&
        !e.metaKey &&
        !e.altKey &&
        !isTypingTarget(e.target)
      ) {
        e.preventDefault();
        handleFlag();
      } else if (e.key.toLowerCase() === "d") {
        e.preventDefault();
        handleDelete();
      } else if (e.key === "u" && !e.ctrlKey && !e.metaKey && canUndo()) {
        e.preventDefault();
        handleUndo();
      } else if (e.key === "z" && (e.ctrlKey || e.metaKey) && canUndo()) {
        e.preventDefault();
        handleUndo();
      } else if (e.key === "Escape") {
        endSession();
        navigate("/");
      }
    },
    [
      loading,
      sessionDone,
      isFlipped,
      handleReveal,
      handleAnswer,
      handleBury,
      handleFlag,
      handleDelete,
      endSession,
      navigate,
      canUndo,
      handleUndo,
      fetchDecks,
      editOpen,
    ]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full text-text-muted">
        Loading review session...
      </div>
    );
  }

  if (sessionDone) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-6 p-8">
        <div className="text-center">
          <h2 className="text-2xl font-bold text-text mb-2">
            Session Complete
          </h2>
          <p className="text-text-secondary">
            You reviewed {reviewedCount} card{reviewedCount !== 1 ? "s" : ""}.
          </p>
        </div>

        {stats && stats.reviewed_today > 0 && (
          <div className="w-full max-w-sm space-y-3">
            <h3 className="text-sm font-semibold text-text-secondary text-center">
              Today's Summary
            </h3>
            <div className="grid grid-cols-4 gap-2 text-center">
              <div className="bg-red-50 dark:bg-red-900/20 rounded-xl p-3">
                <p className="text-xl font-bold text-danger">
                  {stats.again_count}
                </p>
                <p className="text-xs text-text-muted">Again</p>
              </div>
              <div className="bg-orange-50 dark:bg-orange-900/20 rounded-xl p-3">
                <p className="text-xl font-bold text-orange-500">
                  {stats.hard_count}
                </p>
                <p className="text-xs text-text-muted">Hard</p>
              </div>
              <div className="bg-green-50 dark:bg-green-900/20 rounded-xl p-3">
                <p className="text-xl font-bold text-success">
                  {stats.good_count}
                </p>
                <p className="text-xs text-text-muted">Good</p>
              </div>
              <div className="bg-primary-50 dark:bg-primary-900/20 rounded-xl p-3">
                <p className="text-xl font-bold text-primary-500">
                  {stats.easy_count}
                </p>
                <p className="text-xs text-text-muted">Easy</p>
              </div>
            </div>
            <p className="text-xs text-text-muted text-center">
              Total time: {Math.round(stats.total_time_ms / 1000 / 60)}m{" "}
              {Math.round((stats.total_time_ms / 1000) % 60)}s
            </p>
          </div>
        )}

        <button
          onClick={() => {
            fetchDecks();
            navigate("/");
          }}
          className="px-6 py-3 bg-primary-600 text-white rounded-xl font-medium hover:bg-primary-700 transition-colors"
        >
          Back to Decks
          <kbd className="ml-1.5 inline-flex items-center justify-center min-w-[1.25rem] h-5 px-1 text-[10px] font-semibold rounded bg-white/20 border border-white/20">
            Esc
          </kbd>
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
    isFlipped,
    revealStep
  );

  const revealLabel =
    progressiveQuizbowl && revealStep < progressiveTotal
      ? progressiveRevealLabel(revealStep, card.fields)
      : "Show Answer";

  const intervals = intervalPreview
    ? [intervalPreview.again, intervalPreview.hard, intervalPreview.good, intervalPreview.easy]
    : null;

  return (
    <div className="flex flex-col h-full">
      <div className="h-1 bg-surface-alt">
        <div
          className="h-full bg-primary-500 transition-all duration-300"
          style={{
            width: `${((currentIndex + 1) / queue.length) * 100}%`,
          }}
        />
      </div>

      <div className="flex items-center justify-between px-6 py-3 border-b border-border">
        <div className="flex items-center gap-3">
          <button
            onClick={() => {
              endSession();
              navigate("/");
            }}
            className="text-sm text-text-secondary hover:text-text transition-colors"
          >
            End Session
          </button>
          {canUndo() && (
            <button
              onClick={handleUndo}
              className="text-sm text-primary-500 hover:text-primary-600 transition-colors"
              title="Undo last action (U)"
            >
              Undo
              <kbd className="ml-1 text-[10px] font-semibold text-text-muted">U</kbd>
            </button>
          )}
          <button
            onClick={() => setEditOpen(true)}
            className="text-sm text-text-secondary hover:text-text transition-colors"
            title="Edit note (E)"
          >
            Edit
            <kbd className="ml-1 text-[10px] font-semibold text-text-muted">E</kbd>
          </button>
        </div>
        <div className="flex items-center gap-4">
          <KarmaDisplay />
          <span className="text-sm text-text-muted">
            {remaining()} remaining
          </span>
        </div>
      </div>
      <ProfileOnboarding />

      <div className="flex-1 flex items-center justify-center p-8">
        <div className="w-full max-w-2xl">
          <style>{card.css}</style>

          <div className="bg-surface-alt rounded-2xl border border-border shadow-sm p-8 min-h-[300px] flex flex-col items-center justify-center">
            <div
              className="text-lg text-center leading-relaxed w-full prose prose-stone dark:prose-invert max-w-none select-text whitespace-pre-wrap"
              dangerouslySetInnerHTML={{
                __html: cardContent,
              }}
            />
          </div>

          <div className="flex justify-center mt-3 gap-2">
            <span
              className={`text-xs px-2.5 py-1 rounded-full font-medium ${
                card.state === "new"
                  ? "bg-primary-100 text-primary-700 dark:bg-primary-900/30 dark:text-primary-300"
                  : card.state === "learning" || card.state === "relearning"
                    ? "bg-orange-100 text-orange-700 dark:bg-orange-900/30 dark:text-orange-300"
                    : "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-300"
              }`}
            >
              {card.state}
            </span>
            {isFlagged && (
              <span className="text-xs px-2.5 py-1 rounded-full font-medium bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-200">
                flag
              </span>
            )}
          </div>

          <div className="mt-4 w-full max-w-xl mx-auto text-left">
            <NoteTagsPanel noteId={card.note_id} compact />
          </div>

          <div className="mt-5 space-y-3">
            {!isFlipped ? (
              <button
                onClick={handleReveal}
                className="w-full py-4 bg-surface-alt border border-border rounded-xl text-text font-medium hover:bg-surface-hover transition-colors cursor-pointer"
              >
                {revealLabel}
                <span className="text-text-muted text-sm ml-2">(Space)</span>
              </button>
            ) : (
              <div className="grid grid-cols-4 gap-3">
                {RATING_LABELS.map((label, i) => (
                  <button
                    key={label}
                    onClick={() => handleAnswer(i + 1)}
                    className={`py-3 rounded-xl text-white font-medium transition-colors cursor-pointer ${RATING_COLORS[i]}`}
                  >
                    {label}({i + 1})
                    {intervals && (
                      <span className="block text-xs opacity-75 mt-0.5">
                        {intervals[i]}
                      </span>
                    )}
                  </button>
                ))}
              </div>
            )}

            <div className="grid grid-cols-3 gap-3">
              <button
                onClick={handleBury}
                className="py-2.5 text-sm text-text-secondary border border-border rounded-xl hover:bg-surface-hover transition-colors cursor-pointer"
                title="Hide until tomorrow (9)"
              >
                Bury
                <kbd className="ml-1.5 text-[10px] font-semibold text-text-muted">9</kbd>
              </button>
              <button
                onClick={handleFlag}
                disabled={flagging}
                className={`py-2.5 text-sm border rounded-xl transition-colors cursor-pointer disabled:cursor-default disabled:opacity-60 ${
                  isFlagged
                    ? "text-amber-800 dark:text-amber-200 border-amber-300 dark:border-amber-700 bg-amber-50 dark:bg-amber-900/20 hover:bg-amber-100 dark:hover:bg-amber-900/30"
                    : "text-text-secondary border-border hover:bg-surface-hover"
                }`}
                title={isFlagged ? "Remove flag from this card (F)" : "Flag this card (F)"}
              >
                {isFlagged ? "Unflag" : "Flag"}
                <kbd className="ml-1.5 text-[10px] font-semibold text-text-muted">F</kbd>
              </button>
              <button
                onClick={handleDelete}
                className="py-2.5 text-sm text-danger border border-red-200 dark:border-red-900/50 rounded-xl hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors cursor-pointer"
                title="Remove this card permanently (D). Undo with U."
              >
                Delete
                <kbd className="ml-1.5 text-[10px] font-semibold text-text-muted">D</kbd>
              </button>
            </div>
          </div>
        </div>
      </div>

      <ReviewNoteEditor
        card={card}
        open={editOpen}
        onClose={() => setEditOpen(false)}
        onSaved={(fields) => updateNoteFields(card.note_id, fields)}
      />
    </div>
  );
}

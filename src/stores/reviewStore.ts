import { create } from "zustand";
import {
  api,
  type ReviewCard,
  type AnswerResult,
  type IntervalPreview,
  type DeletedCardSnapshot,
} from "../lib/tauri";

type SessionAction =
  | { type: "answer"; result: AnswerResult }
  | { type: "bury"; card: ReviewCard }
  | { type: "delete"; card: ReviewCard; snapshot: DeletedCardSnapshot };

interface ReviewState {
  queue: ReviewCard[];
  currentIndex: number;
  isFlipped: boolean;
  sessionActive: boolean;
  loading: boolean;
  startTime: number;
  actions: SessionAction[];
  intervalPreview: IntervalPreview | null;
  deckId: string | null;

  startSession: (deckId: string) => Promise<void>;
  flipCard: () => void;
  answerCard: (rating: number) => Promise<void>;
  buryCard: () => Promise<void>;
  deleteCard: () => Promise<void>;
  undoLast: () => Promise<void>;
  endSession: () => void;
  updateNoteFields: (noteId: string, fields: Record<string, string>) => void;
  currentCard: () => ReviewCard | null;
  remaining: () => number;
  canUndo: () => boolean;
}

function removeCurrentFromQueue(queue: ReviewCard[], currentIndex: number) {
  const newQueue = [
    ...queue.slice(0, currentIndex),
    ...queue.slice(currentIndex + 1),
  ];
  return {
    queue: newQueue,
    sessionActive: currentIndex < newQueue.length,
  };
}

export const useReviewStore = create<ReviewState>((set, get) => ({
  queue: [],
  currentIndex: 0,
  isFlipped: false,
  sessionActive: false,
  loading: false,
  startTime: 0,
  actions: [],
  intervalPreview: null,
  deckId: null,

  startSession: async (deckId: string) => {
    set({ loading: true, deckId });
    try {
      const queue = await api.getReviewQueue(deckId);
      set({
        queue,
        currentIndex: 0,
        isFlipped: false,
        sessionActive: true,
        loading: false,
        startTime: Date.now(),
        actions: [],
        intervalPreview: null,
      });
    } catch (e) {
      set({ loading: false });
      throw e;
    }
  },

  flipCard: async () => {
    set({ isFlipped: true });
    const card = get().currentCard();
    if (card) {
      try {
        const preview = await api.getIntervalPreview(card.card_id);
        set({ intervalPreview: preview });
      } catch {
        // Non-critical, ignore
      }
    }
  },

  answerCard: async (rating: number) => {
    const { queue, currentIndex, startTime } = get();
    const card = queue[currentIndex];
    if (!card) return;

    const elapsed = Date.now() - startTime;
    const result = await api.answerCard({
      card_id: card.card_id,
      rating,
      elapsed_ms: elapsed,
    });

    const { useKarmaStore } = await import("./karmaStore");
    useKarmaStore.getState().applyEarn(result.karma);

    const nextIndex = currentIndex + 1;
    const sessionActive = nextIndex < queue.length;

    set({
      actions: [...get().actions, { type: "answer", result }],
      currentIndex: nextIndex,
      isFlipped: false,
      sessionActive,
      startTime: Date.now(),
      intervalPreview: null,
    });
  },

  buryCard: async () => {
    const { queue, currentIndex } = get();
    const card = queue[currentIndex];
    if (!card) return;

    await api.buryCard(card.card_id);
    const { queue: newQueue, sessionActive } = removeCurrentFromQueue(
      queue,
      currentIndex
    );

    set({
      actions: [...get().actions, { type: "bury", card }],
      queue: newQueue,
      isFlipped: false,
      sessionActive,
      intervalPreview: null,
    });
  },

  deleteCard: async () => {
    const { queue, currentIndex } = get();
    const card = queue[currentIndex];
    if (!card) return;

    const snapshot = await api.deleteCard(card.card_id);
    const { queue: newQueue, sessionActive } = removeCurrentFromQueue(
      queue,
      currentIndex
    );

    set({
      actions: [...get().actions, { type: "delete", card, snapshot }],
      queue: newQueue,
      isFlipped: false,
      sessionActive,
      intervalPreview: null,
    });
  },

  undoLast: async () => {
    const { actions, currentIndex, queue } = get();
    if (actions.length === 0) return;

    const last = actions[actions.length - 1];
    const remainingActions = actions.slice(0, -1);

    if (last.type === "answer") {
      const undo = await api.undoReview(last.result.review_log_id);
      const { useKarmaStore } = await import("./karmaStore");
      useKarmaStore.getState().applyEarn(undo.karma);
      set({
        actions: remainingActions,
        currentIndex: currentIndex - 1,
        isFlipped: false,
        sessionActive: true,
        intervalPreview: null,
      });
      return;
    }

    if (last.type === "bury") {
      await api.unburyCard(last.card.card_id);
    } else {
      await api.restoreCard(last.snapshot);
    }

    const newQueue = [...queue];
    newQueue.splice(currentIndex, 0, last.card);
    set({
      actions: remainingActions,
      queue: newQueue,
      isFlipped: false,
      sessionActive: true,
      intervalPreview: null,
    });
  },

  endSession: () =>
    set({
      queue: [],
      currentIndex: 0,
      isFlipped: false,
      sessionActive: false,
      actions: [],
      intervalPreview: null,
      deckId: null,
    }),

  updateNoteFields: (noteId, fields) => {
    set((state) => ({
      queue: state.queue.map((c) =>
        c.note_id === noteId ? { ...c, fields } : c
      ),
    }));
  },

  currentCard: () => {
    const { queue, currentIndex } = get();
    return queue[currentIndex] ?? null;
  },

  remaining: () => {
    const { queue, currentIndex } = get();
    return queue.length - currentIndex;
  },

  canUndo: () => get().actions.length > 0,
}));

import { create } from "zustand";
import { api, type DeletedDeckSnapshot } from "../lib/tauri";
import { useDeckStore } from "./deckStore";

type DeckDeleteAction = {
  snapshot: DeletedDeckSnapshot;
  previousSelectedDeckId: string | null;
};

interface UndoState {
  deckDeletes: DeckDeleteAction[];
  pushDeckDelete: (
    snapshot: DeletedDeckSnapshot,
    previousSelectedDeckId: string | null
  ) => void;
  undoLast: () => Promise<boolean>;
  canUndo: () => boolean;
  clear: () => void;
}

export const useUndoStore = create<UndoState>((set, get) => ({
  deckDeletes: [],

  pushDeckDelete: (snapshot, previousSelectedDeckId) => {
    set((state) => ({
      deckDeletes: [
        ...state.deckDeletes,
        { snapshot, previousSelectedDeckId },
      ].slice(-20),
    }));
  },

  undoLast: async () => {
    const actions = get().deckDeletes;
    if (actions.length === 0) return false;

    const last = actions[actions.length - 1];
    await api.restoreDeletedDeck(last.snapshot);
    set({ deckDeletes: actions.slice(0, -1) });

    await useDeckStore.getState().fetchDecks();
    const restoredId = last.snapshot.root_deck_id;
    const decks = useDeckStore.getState().decks;
    const stillExists = decks.some((d) => d.id === last.previousSelectedDeckId);
    useDeckStore
      .getState()
      .selectDeck(
        stillExists && last.previousSelectedDeckId
          ? last.previousSelectedDeckId
          : restoredId
      );
    return true;
  },

  canUndo: () => get().deckDeletes.length > 0,

  clear: () => set({ deckDeletes: [] }),
}));

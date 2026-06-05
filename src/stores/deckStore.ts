import { create } from "zustand";
import { api, type DeckWithCounts, type Deck, type DeletedDeckSnapshot } from "../lib/tauri";
import { useUndoStore } from "./undoStore";

interface DeckState {
  decks: DeckWithCounts[];
  selectedDeckId: string | null;
  loading: boolean;
  error: string | null;

  fetchDecks: () => Promise<void>;
  createDeck: (
    name: string,
    parentId?: string | null,
    description?: string
  ) => Promise<Deck>;
  updateDeck: (
    id: string,
    updates: Partial<Omit<Deck, "id" | "created_at" | "updated_at">>
  ) => Promise<void>;
  deleteDeck: (id: string) => Promise<DeletedDeckSnapshot>;
  selectDeck: (id: string | null) => void;
}

export const useDeckStore = create<DeckState>((set, get) => ({
  decks: [],
  selectedDeckId: null,
  loading: false,
  error: null,

  fetchDecks: async () => {
    set({ loading: true, error: null });
    try {
      const decks = await api.getDecks();
      set({ decks, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  createDeck: async (name, parentId, description) => {
    const deck = await api.createDeck({
      name,
      parent_id: parentId,
      description,
    });
    await get().fetchDecks();
    return deck;
  },

  updateDeck: async (id, updates) => {
    await api.updateDeck({ id, ...updates });
    await get().fetchDecks();
  },

  deleteDeck: async (id) => {
    const previousSelectedDeckId = get().selectedDeckId;
    const snapshot = await api.deleteDeck(id);
    useUndoStore.getState().pushDeckDelete(snapshot, previousSelectedDeckId);
    if (previousSelectedDeckId === id) {
      set({ selectedDeckId: null });
    }
    await get().fetchDecks();
    return snapshot;
  },

  selectDeck: (id) => set({ selectedDeckId: id }),
}));

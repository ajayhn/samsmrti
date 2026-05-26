import { create } from "zustand";
import { api, type DeckWithCounts, type Deck } from "../lib/tauri";

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
  deleteDeck: (id: string) => Promise<void>;
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
    await api.deleteDeck(id);
    if (get().selectedDeckId === id) {
      set({ selectedDeckId: null });
    }
    await get().fetchDecks();
  },

  selectDeck: (id) => set({ selectedDeckId: id }),
}));

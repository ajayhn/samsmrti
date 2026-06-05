import { create } from "zustand";
import type { ContentDeckOption } from "../lib/tauri";

export type ContentTransferPickerMode = "export" | "import";

interface PickerState {
  mode: ContentTransferPickerMode;
  decks: ContentDeckOption[];
  filePath?: string;
  resolve: (deckIds: string[] | null) => void;
}

interface ContentTransferState {
  picker: PickerState | null;
  openPicker: (picker: PickerState) => void;
  closePicker: () => void;
}

export const useContentTransferStore = create<ContentTransferState>((set) => ({
  picker: null,
  openPicker: (picker) => set({ picker }),
  closePicker: () => set({ picker: null }),
}));

export function pickDecks(
  mode: ContentTransferPickerMode,
  decks: ContentDeckOption[],
  filePath?: string
): Promise<string[] | null> {
  return new Promise((resolve) => {
    useContentTransferStore.getState().openPicker({
      mode,
      decks,
      filePath,
      resolve,
    });
  });
}

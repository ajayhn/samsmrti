import { create } from "zustand";

/** Bump when tags are created or removed so sidebars/comboboxes refetch. */
export const useTagListStore = create<{
  revision: number;
  notifyTagsChanged: () => void;
}>((set) => ({
  revision: 0,
  notifyTagsChanged: () => set((s) => ({ revision: s.revision + 1 })),
}));

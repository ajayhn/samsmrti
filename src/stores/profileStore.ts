import { create } from "zustand";
import { api, type Profile } from "../lib/tauri";

interface ProfileState {
  profiles: Profile[];
  active: Profile | null;
  loading: boolean;
  showOnboarding: boolean;
  fetchProfiles: () => Promise<void>;
  switchProfile: (profileId: string) => Promise<void>;
  createProfile: (displayName: string) => Promise<Profile>;
  deleteProfile: (profileId: string) => Promise<void>;
}

export const useProfileStore = create<ProfileState>((set, get) => ({
  profiles: [],
  active: null,
  loading: true,
  showOnboarding: false,

  fetchProfiles: async () => {
    set({ loading: true });
    try {
      const [profiles, active] = await Promise.all([
        api.listProfiles(),
        api.getActiveProfile(),
      ]);
      const nonAdmin = profiles.filter((p) => !p.is_admin);
      set({
        profiles,
        active,
        showOnboarding: nonAdmin.length === 0,
        loading: false,
      });
    } catch (e) {
      console.error(e);
      set({ loading: false });
    }
  },

  switchProfile: async (profileId) => {
    const active = await api.setActiveProfile(profileId);
    set({ active });
    const { useKarmaStore } = await import("./karmaStore");
    await useKarmaStore.getState().fetchKarma();
    const { useDeckStore } = await import("./deckStore");
    await useDeckStore.getState().fetchDecks();
  },

  createProfile: async (displayName) => {
    const profile = await api.createProfile(displayName);
    await get().fetchProfiles();
    await get().switchProfile(profile.id);
    set({ showOnboarding: false });
    return profile;
  },

  deleteProfile: async (profileId) => {
    await api.deleteProfile(profileId);
    await get().fetchProfiles();
  },
}));

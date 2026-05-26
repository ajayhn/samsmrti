import { create } from "zustand";
import { api, type KarmaEarnEvent, type KarmaOverview } from "../lib/tauri";

interface KarmaState {
  overview: KarmaOverview | null;
  displayCents: number;
  floatDelta: number | null;
  loading: boolean;
  fetchKarma: () => Promise<void>;
  setOverview: (overview: KarmaOverview) => void;
  applyEarn: (event: KarmaEarnEvent) => void;
  clearFloat: () => void;
}

export function formatKarmaDollars(cents: number): string {
  return `$${(cents / 100).toFixed(2)}`;
}

export const useKarmaStore = create<KarmaState>((set, get) => ({
  overview: null,
  displayCents: 0,
  floatDelta: null,
  loading: true,

  fetchKarma: async () => {
    set({ loading: true });
    try {
      const overview = await api.getKarmaOverview();
      set({
        overview,
        displayCents: overview.balance_cents,
        loading: false,
      });
    } catch (e) {
      console.error(e);
      set({ loading: false });
    }
  },

  setOverview: (overview) => {
    set({
      overview,
      displayCents: overview.balance_cents,
      loading: false,
    });
  },

  applyEarn: (event) => {
    const prev = get().overview;
    const overview: KarmaOverview = prev
      ? {
          ...prev,
          balance_cents: event.balance_cents,
          streak_days: event.streak_days,
          qualified_today: event.qualified_today,
        }
      : {
          balance_cents: event.balance_cents,
          streak_days: event.streak_days,
          qualified_today: event.qualified_today,
          today_active_seconds: 0,
          today_effective_actions: 0,
          daily_qualified: [],
          profile_id: "",
          is_admin: false,
        };

    set({
      overview,
      floatDelta: event.earned_cents !== 0 ? event.earned_cents : null,
    });

    const start = get().displayCents;
    const end = event.balance_cents;
    const duration = 400;
    const t0 = performance.now();

    const tick = (now: number) => {
      const t = Math.min(1, (now - t0) / duration);
      const eased = 1 - (1 - t) ** 3;
      set({ displayCents: Math.round(start + (end - start) * eased) });
      if (t < 1) {
        requestAnimationFrame(tick);
      } else {
        set({ displayCents: end });
      }
    };
    requestAnimationFrame(tick);

    if (event.bonus_awarded_cents > 0) {
      setTimeout(() => {
        set({ floatDelta: event.bonus_awarded_cents });
        setTimeout(() => get().clearFloat(), 2000);
      }, 500);
    } else if (event.earned_cents !== 0) {
      setTimeout(() => get().clearFloat(), 1200);
    }
  },

  clearFloat: () => set({ floatDelta: null }),
}));

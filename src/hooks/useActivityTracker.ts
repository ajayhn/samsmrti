import { useEffect, useRef } from "react";
import { api } from "../lib/tauri";
import { useKarmaStore } from "../stores/karmaStore";
import { useProfileStore } from "../stores/profileStore";

const FLUSH_INTERVAL_MS = 30_000;
const IDLE_MS = 60_000;

/** Tracks focused, active time and flushes to karma (skipped for Admin). */
export function useActivityTracker(enabled: boolean) {
  const active = useProfileStore((s) => s.active);
  const pendingRef = useRef(0);
  const lastActiveRef = useRef(Date.now());

  useEffect(() => {
    if (!enabled || active?.is_admin) return;

    const markActive = () => {
      lastActiveRef.current = Date.now();
    };

    const flush = () => {
      const secs = pendingRef.current;
      pendingRef.current = 0;
      if (secs > 0) {
        api
          .recordActivity(secs)
          .then((overview) => useKarmaStore.getState().setOverview(overview))
          .catch(console.error);
      }
    };

    const tick = window.setInterval(() => {
      if (document.hidden) return;
      const idle = Date.now() - lastActiveRef.current > IDLE_MS;
      if (!idle) {
        pendingRef.current += 1;
      }
      if (pendingRef.current >= 30) {
        flush();
      }
    }, 1000);

    const flushInterval = window.setInterval(flush, FLUSH_INTERVAL_MS);

    window.addEventListener("mousemove", markActive);
    window.addEventListener("keydown", markActive);
    window.addEventListener("click", markActive);
    window.addEventListener("scroll", markActive, true);

    return () => {
      clearInterval(tick);
      clearInterval(flushInterval);
      flush();
      window.removeEventListener("mousemove", markActive);
      window.removeEventListener("keydown", markActive);
      window.removeEventListener("click", markActive);
      window.removeEventListener("scroll", markActive, true);
    };
  }, [enabled, active?.id, active?.is_admin]);
}

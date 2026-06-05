import { useEffect, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { Outlet } from "react-router-dom";
import { KarmaDisplay } from "../karma/KarmaDisplay";
import { ProfileOnboarding } from "../profile/ProfileOnboarding";
import { useDeckStore } from "../../stores/deckStore";
import { useKarmaStore } from "../../stores/karmaStore";
import { useProfileStore } from "../../stores/profileStore";
import { useUndoStore } from "../../stores/undoStore";
import { ContentTransferHost } from "../settings/ContentTransferHost";
import { Sidebar } from "./Sidebar";

export function AppLayout() {
  const fetchProfiles = useProfileStore((s) => s.fetchProfiles);
  const fetchKarma = useKarmaStore((s) => s.fetchKarma);
  const fetchDecks = useDeckStore((s) => s.fetchDecks);

  const tryUndoDeckDelete = useCallback(async () => {
    if (!useUndoStore.getState().canUndo()) return;
    await useUndoStore.getState().undoLast();
  }, []);

  useEffect(() => {
    fetchProfiles().then(() => {
      fetchKarma();
      fetchDecks();
    });
  }, [fetchProfiles, fetchKarma, fetchDecks]);

  useEffect(() => {
    const unsub = listen("menu-undo", () => {
      void tryUndoDeckDelete();
    });
    return () => {
      unsub.then((fn) => fn());
    };
  }, [tryUndoDeckDelete]);

  useEffect(() => {
    const isInputFocused = () => {
      const tag = document.activeElement?.tagName;
      return (
        tag === "INPUT" ||
        tag === "TEXTAREA" ||
        tag === "SELECT" ||
        (document.activeElement as HTMLElement)?.isContentEditable
      );
    };

    const onKeyDown = (e: KeyboardEvent) => {
      if (!e.metaKey && !e.ctrlKey) return;
      if (e.altKey || e.shiftKey) return;
      if (e.key.toLowerCase() !== "z") return;
      if (isInputFocused()) return;
      if (!useUndoStore.getState().canUndo()) return;
      e.preventDefault();
      void tryUndoDeckDelete();
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [tryUndoDeckDelete]);

  return (
    <div className="flex h-screen overflow-hidden">
      <Sidebar />
      <div className="flex-1 flex flex-col min-w-0">
        <header className="shrink-0 flex items-center justify-end px-4 py-2 border-b border-border bg-surface">
          <KarmaDisplay />
        </header>
        <main className="flex-1 overflow-y-auto">
          <Outlet />
        </main>
      </div>
      <ProfileOnboarding />
      <ContentTransferHost />
    </div>
  );
}

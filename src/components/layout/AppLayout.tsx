import { useEffect } from "react";
import { Outlet } from "react-router-dom";
import { KarmaDisplay } from "../karma/KarmaDisplay";
import { ProfileOnboarding } from "../profile/ProfileOnboarding";
import { useDeckStore } from "../../stores/deckStore";
import { useKarmaStore } from "../../stores/karmaStore";
import { useProfileStore } from "../../stores/profileStore";
import { Sidebar } from "./Sidebar";

export function AppLayout() {
  const fetchProfiles = useProfileStore((s) => s.fetchProfiles);
  const fetchKarma = useKarmaStore((s) => s.fetchKarma);
  const fetchDecks = useDeckStore((s) => s.fetchDecks);

  useEffect(() => {
    fetchProfiles().then(() => {
      fetchKarma();
      fetchDecks();
    });
  }, [fetchProfiles, fetchKarma, fetchDecks]);

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
    </div>
  );
}

import { useEffect } from "react";
import { HashRouter, Routes, Route } from "react-router-dom";
import { useProfileStore } from "./stores/profileStore";
import { useKarmaStore } from "./stores/karmaStore";
import { DeckListPage } from "./webapp/DeckListPage";
import { ReviewSessionPage } from "./webapp/ReviewSessionPage";
import { SyncPage } from "./webapp/SyncPage";

export default function AppWeb() {
  const fetchProfiles = useProfileStore((s) => s.fetchProfiles);
  const fetchKarma = useKarmaStore((s) => s.fetchKarma);

  useEffect(() => {
    fetchProfiles().then(() => fetchKarma());
  }, [fetchProfiles, fetchKarma]);

  return (
    <HashRouter>
      <div className="h-screen w-screen bg-surface text-text overflow-hidden">
        <Routes>
          <Route path="/" element={<DeckListPage />} />
          <Route path="/review/:deckId" element={<ReviewSessionPage />} />
          <Route path="/sync" element={<SyncPage />} />
        </Routes>
      </div>
    </HashRouter>
  );
}

import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { AppLayout } from "./components/layout/AppLayout";
import {
  exportContentJson,
  exportFullBackup,
  importContentJson,
  restoreFullBackup,
} from "./lib/dataTransfer";
import { DeckHome } from "./components/deck/DeckHome";
import { DeckCards } from "./components/deck/DeckCards";
import { AddCard } from "./components/editor/AddCard";
import { NoteTypeManager } from "./components/editor/NoteTypeManager";
import { ReviewSession } from "./components/review/ReviewSession";
import { CardBrowser } from "./components/search/CardBrowser";
import { StatsDashboard } from "./components/stats/StatsDashboard";
import { EntityGraph } from "./components/graph/EntityGraph";
import { Settings } from "./components/settings/Settings";
import { UserGuide } from "./components/help/UserGuide";

export default function App() {
  useEffect(() => {
    const unsubs = [
      listen("menu-export-content", () => exportContentJson()),
      listen("menu-import-content", () => importContentJson()),
      listen("menu-export-full-backup", () => exportFullBackup()),
      listen("menu-restore-full-backup", () => restoreFullBackup()),
    ];
    return () => {
      unsubs.forEach((p) => p.then((fn) => fn()));
    };
  }, []);

  // Fallback when the WebView does not wire Cmd+C (selection → clipboard).
  useEffect(() => {
    const onCopy = () => {
      const text = window.getSelection()?.toString();
      if (!text) return;
      navigator.clipboard.writeText(text).catch(() => {});
    };
    document.addEventListener("copy", onCopy);
    return () => document.removeEventListener("copy", onCopy);
  }, []);

  return (
    <BrowserRouter>
      <Routes>
        <Route element={<AppLayout />}>
          <Route path="/" element={<DeckHome />} />
          <Route path="/deck/:deckId/cards" element={<DeckCards />} />
          <Route path="/add/:deckId" element={<AddCard />} />
          <Route path="/note-types" element={<NoteTypeManager />} />
          <Route path="/browse" element={<CardBrowser />} />
          <Route path="/stats" element={<StatsDashboard />} />
          <Route path="/graph" element={<EntityGraph />} />
          <Route path="/settings" element={<Settings />} />
          <Route path="/help" element={<UserGuide />} />
        </Route>
        <Route path="/review/:deckId" element={<ReviewSession />} />
      </Routes>
    </BrowserRouter>
  );
}

import { useState, useEffect } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { api } from "../../lib/tauri";
import {
  exportContentJson,
  exportFullBackup,
  importContentJson,
  restoreFullBackup,
} from "../../lib/dataTransfer";
import { useDeckStore } from "../../stores/deckStore";
import { useProfileStore } from "../../stores/profileStore";

export function Settings() {
  const { profiles, active, createProfile, deleteProfile, switchProfile, fetchProfiles } =
    useProfileStore();
  const [newProfileName, setNewProfileName] = useState("");
  const [darkMode, setDarkMode] = useState(() => {
    return document.documentElement.classList.contains("dark");
  });
  const [exporting, setExporting] = useState(false);
  const [exportMsg, setExportMsg] = useState<string | null>(null);
  const [selectedExportDeck, setSelectedExportDeck] = useState("");
  const { decks } = useDeckStore();

  useEffect(() => {
    if (darkMode) {
      document.documentElement.classList.add("dark");
      localStorage.setItem("samsmrti-theme", "dark");
    } else {
      document.documentElement.classList.remove("dark");
      localStorage.setItem("samsmrti-theme", "light");
    }
  }, [darkMode]);

  useEffect(() => {
    const saved = localStorage.getItem("samsmrti-theme");
    if (saved === "dark") {
      setDarkMode(true);
      document.documentElement.classList.add("dark");
    }
  }, []);

  useEffect(() => {
    fetchProfiles();
  }, [fetchProfiles]);

  const handleExport = async () => {
    if (!selectedExportDeck) return;
    const filePath = await save({
      defaultPath: "deck-export.json",
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (!filePath) return;

    setExporting(true);
    setExportMsg(null);
    try {
      const result = await api.exportDeckJson(selectedExportDeck, filePath);
      setExportMsg(
        `Exported ${result.notes_exported} notes and ${result.cards_exported} cards.`
      );
    } catch (e) {
      setExportMsg(`Export failed: ${e}`);
    } finally {
      setExporting(false);
    }
  };

  return (
    <div className="h-full overflow-y-auto p-6 max-w-xl mx-auto space-y-8">
      <h2 className="text-xl font-bold text-text">Settings</h2>

      {/* Profiles */}
      <section className="space-y-4">
        <h3 className="text-sm font-semibold text-text-secondary uppercase tracking-wider">
          Profiles
        </h3>
        <div className="bg-surface-alt border border-border rounded-xl p-4 space-y-3">
          <p className="text-xs text-text-muted">
            Honor system: switch profile when someone else studies on this
            computer. The Admin profile never earns Karma.
          </p>
          <ul className="space-y-2">
            {profiles.map((p) => (
              <li
                key={p.id}
                className="flex items-center justify-between gap-2 text-sm"
              >
                <button
                  type="button"
                  onClick={() => switchProfile(p.id)}
                  className={`text-left cursor-pointer hover:underline ${
                    active?.id === p.id ? "text-primary-500 font-medium" : "text-text"
                  }`}
                >
                  {p.display_name}
                  {p.is_admin && (
                    <span className="text-text-muted ml-1">(no karma)</span>
                  )}
                </button>
                {!p.is_admin && (
                  <button
                    type="button"
                    onClick={() => deleteProfile(p.id)}
                    className="text-xs text-danger hover:underline cursor-pointer"
                  >
                    Delete
                  </button>
                )}
              </li>
            ))}
          </ul>
          <div className="flex gap-2">
            <input
              type="text"
              value={newProfileName}
              onChange={(e) => setNewProfileName(e.target.value)}
              placeholder="New profile name"
              className="flex-1 px-3 py-2 text-sm rounded-lg border border-border bg-surface"
            />
            <button
              type="button"
              onClick={async () => {
                const name = newProfileName.trim();
                if (!name) return;
                await createProfile(name);
                setNewProfileName("");
              }}
              className="px-3 py-2 text-sm bg-primary-500 text-white rounded-lg cursor-pointer"
            >
              Add
            </button>
          </div>
        </div>
      </section>

      {/* Appearance */}
      <section className="space-y-4">
        <h3 className="text-sm font-semibold text-text-secondary uppercase tracking-wider">
          Appearance
        </h3>
        <div className="flex items-center justify-between bg-surface-alt border border-border rounded-xl p-4">
          <div>
            <p className="text-sm font-medium text-text">Dark Mode</p>
            <p className="text-xs text-text-muted">
              Switch between light and dark themes
            </p>
          </div>
          <button
            onClick={() => setDarkMode(!darkMode)}
            className={`relative w-12 h-7 rounded-full transition-colors ${
              darkMode ? "bg-primary-500" : "bg-gray-300"
            }`}
          >
            <span
              className={`absolute top-0.5 left-0.5 w-6 h-6 rounded-full bg-white shadow-sm transition-transform ${
                darkMode ? "translate-x-5" : "translate-x-0"
              }`}
            />
          </button>
        </div>
      </section>

      {/* Export */}
      <section className="space-y-4">
        <h3 className="text-sm font-semibold text-text-secondary uppercase tracking-wider">
          Data
        </h3>
        <div className="bg-surface-alt border border-border rounded-xl p-4 space-y-3">
          <p className="text-sm font-medium text-text">Export Deck</p>
          <p className="text-xs text-text-muted">
            Export a deck as JSON for backup or sharing.
          </p>
          <div className="flex gap-3">
            <select
              value={selectedExportDeck}
              onChange={(e) => setSelectedExportDeck(e.target.value)}
              className="flex-1 px-3 py-2 bg-surface border border-border rounded-lg text-sm"
            >
              <option value="">Select a deck</option>
              {decks.map((d) => (
                <option key={d.id} value={d.id}>
                  {d.name}
                </option>
              ))}
            </select>
            <button
              onClick={handleExport}
              disabled={!selectedExportDeck || exporting}
              className="px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700 disabled:opacity-50 transition-colors"
            >
              {exporting ? "Exporting..." : "Export"}
            </button>
          </div>
          {exportMsg && (
            <p className="text-xs text-text-secondary">{exportMsg}</p>
          )}
        </div>
        <div className="bg-surface-alt border border-border rounded-xl p-4 space-y-3">
          <p className="text-sm font-medium text-text">Export / import content</p>
          <p className="text-xs text-text-muted">
            JSON collection for sharing decks with someone who starts fresh (no review
            history, profiles, or karma). Also under <strong>File → Export Content</strong>{" "}
            and <strong>Import Content</strong>.
          </p>
          <div className="flex flex-wrap gap-2">
            <button
              type="button"
              onClick={() => exportContentJson()}
              className="px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700 transition-colors"
            >
              Export content (.json)
            </button>
            <button
              type="button"
              onClick={() => importContentJson()}
              className="px-4 py-2 bg-surface border border-border rounded-lg text-sm font-medium hover:bg-surface-alt transition-colors"
            >
              Import content
            </button>
          </div>
        </div>
        <div className="bg-surface-alt border border-border rounded-xl p-4 space-y-3">
          <p className="text-sm font-medium text-text">Full backup / restore</p>
          <p className="text-xs text-text-muted">
            Binary backup for moving to another computer: database, media, per-profile
            scheduling, review log, and karma. Replaces local data on restore.{" "}
            <strong>File → Backup (Full)</strong> / <strong>Restore (Full)</strong>.
          </p>
          <div className="flex flex-wrap gap-2">
            <button
              type="button"
              onClick={() => exportFullBackup()}
              className="px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700 transition-colors"
            >
              Backup (full)
            </button>
            <button
              type="button"
              onClick={() => restoreFullBackup()}
              className="px-4 py-2 bg-surface border border-border rounded-lg text-sm font-medium hover:bg-surface-alt transition-colors"
            >
              Restore (full)
            </button>
          </div>
        </div>
      </section>

      {/* About */}
      <section className="space-y-4">
        <h3 className="text-sm font-semibold text-text-secondary uppercase tracking-wider">
          About
        </h3>
        <div className="bg-surface-alt border border-border rounded-xl p-4">
          <p className="text-sm font-medium text-text">Samsmrti</p>
          <p className="text-xs text-text-muted mt-1">
            A modern spaced repetition app built with Tauri, React, and SQLite.
          </p>
          <p className="text-xs text-text-muted mt-1">Version 0.1.0</p>
        </div>
      </section>
    </div>
  );
}

import { useEffect, useRef, useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { api, type ContentDeckOption } from "../lib/webApi";
import { useDeckStore } from "../stores/deckStore";

function downloadJson(filename: string, content: string) {
  const blob = new Blob([content], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
}

function ExportPanel() {
  const [decks, setDecks] = useState<ContentDeckOption[]>([]);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState("");

  useEffect(() => {
    api.listContentExportDecks().then(setDecks).catch((e) => setStatus(String(e)));
  }, []);

  const toggle = (id: string) => {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const handleExport = async () => {
    setBusy(true);
    setStatus("");
    try {
      const deckIds = selected.size > 0 ? Array.from(selected) : undefined;
      const result = await api.exportContentJson(deckIds);
      const stamp = new Date().toISOString().slice(0, 10);
      downloadJson(`samsmrti-export-${stamp}.json`, result.content);
      const plural = (n: number, word: string) => `${n} ${word}${n === 1 ? "" : "s"}`;
      setStatus(
        `Exported ${plural(result.decks, "deck")}, ${plural(result.notes, "note")}, ${plural(result.cards, "card")}.`
      );
    } catch (e) {
      setStatus(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="space-y-3">
      <p className="text-sm text-text-secondary">
        Pick decks to export, or none to export everything. Save the downloaded file somewhere
        you can get to from your other device (AirDrop, iCloud Drive, email to yourself).
      </p>
      <div className="space-y-1.5 max-h-64 overflow-y-auto">
        {decks.map((deck) => (
          <label
            key={deck.id}
            className="flex items-center gap-2 px-3 py-2 rounded-lg bg-surface-alt border border-border cursor-pointer"
          >
            <input
              type="checkbox"
              checked={selected.has(deck.id)}
              onChange={() => toggle(deck.id)}
            />
            <span className="text-text text-sm">{deck.name}</span>
            <span className="text-text-muted text-xs ml-auto">
              {deck.note_count} note{deck.note_count === 1 ? "" : "s"}
            </span>
          </label>
        ))}
        {decks.length === 0 && (
          <p className="text-text-muted text-sm">No decks to export yet.</p>
        )}
      </div>
      <button
        onClick={handleExport}
        disabled={busy}
        className="w-full py-3 bg-primary-600 text-white rounded-xl font-medium disabled:opacity-50"
      >
        {busy ? "Exporting…" : selected.size > 0 ? `Export ${selected.size} selected` : "Export all decks"}
      </button>
      {status && <p className="text-sm text-text-secondary">{status}</p>}
    </div>
  );
}

function ImportPanel() {
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [fileContent, setFileContent] = useState<string | null>(null);
  const [preview, setPreview] = useState<ContentDeckOption[] | null>(null);
  const [busy, setBusy] = useState(false);
  const [urlBusy, setUrlBusy] = useState(false);
  const [urlInput, setUrlInput] = useState("");
  const [status, setStatus] = useState("");
  const fetchDecks = useDeckStore((s) => s.fetchDecks);
  const [searchParams, setSearchParams] = useSearchParams();

  const previewFromText = async (text: string) => {
    const decks = await api.previewContentImport(text);
    setFileContent(text);
    setPreview(decks);
  };

  const handleFile = async (file: File) => {
    setStatus("");
    setPreview(null);
    setFileContent(null);
    try {
      const text = await file.text();
      await previewFromText(text);
    } catch (e) {
      setStatus(String(e));
    }
  };

  const handleFetchUrl = async (url: string) => {
    setStatus("");
    setPreview(null);
    setFileContent(null);
    setUrlBusy(true);
    try {
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error(`Server returned ${response.status} ${response.statusText}`);
      }
      const text = await response.text();
      await previewFromText(text);
    } catch (e) {
      setStatus(
        `Couldn't fetch that link. If it's hosted elsewhere, its server needs to allow ` +
          `cross-origin requests from this app (CORS). (${String(e)})`
      );
    } finally {
      setUrlBusy(false);
    }
  };

  // Deep link: .../#/sync?import=<url> pre-fills and previews automatically,
  // but still requires an explicit tap on "Import" to actually commit it.
  useEffect(() => {
    const linked = searchParams.get("import");
    if (linked) {
      setUrlInput(linked);
      handleFetchUrl(linked);
      const next = new URLSearchParams(searchParams);
      next.delete("import");
      setSearchParams(next, { replace: true });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const handleImport = async () => {
    if (!fileContent) return;
    setBusy(true);
    setStatus("");
    try {
      const summary = await api.importContentJson(fileContent);
      const plural = (n: number, word: string) => `${n} ${word}${n === 1 ? "" : "s"}`;
      setStatus(
        `Imported ${plural(summary.decks_added, "deck")}, ${plural(summary.notes_added, "note")}, ${plural(summary.cards_added, "card")}.` +
          (summary.warnings.length > 0 ? ` Warnings: ${summary.warnings.join(", ")}` : "")
      );
      setPreview(null);
      setFileContent(null);
      setUrlInput("");
      if (fileInputRef.current) fileInputRef.current.value = "";
      await fetchDecks();
    } catch (e) {
      setStatus(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="space-y-3">
      <p className="text-sm text-text-secondary">
        Import a deck exported from your desktop app. Reviewing here starts fresh -- your
        desktop progress isn't affected, and progress made here isn't sent back automatically.
      </p>
      <input
        ref={fileInputRef}
        type="file"
        accept="application/json,.json"
        onChange={(e) => {
          const file = e.target.files?.[0];
          if (file) handleFile(file);
        }}
        className="block w-full text-sm text-text-secondary file:mr-3 file:py-2 file:px-3 file:rounded-lg file:border-0 file:bg-primary-600 file:text-white"
      />
      <div className="flex items-center gap-2 text-xs text-text-muted">
        <div className="flex-1 border-t border-border" />
        or
        <div className="flex-1 border-t border-border" />
      </div>
      <div className="flex gap-2">
        <input
          type="url"
          inputMode="url"
          placeholder="Paste a deck link (https://...)"
          value={urlInput}
          onChange={(e) => setUrlInput(e.target.value)}
          className="flex-1 min-w-0 px-3 py-2 rounded-lg bg-surface-alt border border-border text-sm text-text"
        />
        <button
          onClick={() => urlInput && handleFetchUrl(urlInput)}
          disabled={urlBusy || !urlInput}
          className="px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium disabled:opacity-50 shrink-0"
        >
          {urlBusy ? "Fetching…" : "Fetch"}
        </button>
      </div>
      {preview && (
        <div className="space-y-1.5">
          <p className="text-sm text-text">This file contains:</p>
          {preview.map((deck) => (
            <div
              key={deck.id}
              className="px-3 py-2 rounded-lg bg-surface-alt border border-border text-sm text-text"
            >
              {deck.name} <span className="text-text-muted">· {deck.note_count} notes</span>
            </div>
          ))}
          <button
            onClick={handleImport}
            disabled={busy}
            className="w-full py-3 bg-primary-600 text-white rounded-xl font-medium disabled:opacity-50"
          >
            {busy ? "Importing…" : "Import"}
          </button>
        </div>
      )}
      {status && <p className="text-sm text-text-secondary">{status}</p>}
    </div>
  );
}

export function SyncPage() {
  const navigate = useNavigate();
  const [tab, setTab] = useState<"import" | "export">("import");

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center gap-3 px-4 py-3 border-b border-border">
        <button onClick={() => navigate("/")} className="text-sm text-text-secondary">
          Back
        </button>
        <h1 className="text-lg font-semibold text-text">Sync decks</h1>
      </div>

      <div className="flex border-b border-border">
        <button
          onClick={() => setTab("import")}
          className={`flex-1 py-3 text-sm font-medium ${
            tab === "import" ? "text-primary-500 border-b-2 border-primary-500" : "text-text-muted"
          }`}
        >
          Import
        </button>
        <button
          onClick={() => setTab("export")}
          className={`flex-1 py-3 text-sm font-medium ${
            tab === "export" ? "text-primary-500 border-b-2 border-primary-500" : "text-text-muted"
          }`}
        >
          Export
        </button>
      </div>

      <div className="flex-1 overflow-y-auto p-4">
        {tab === "import" ? <ImportPanel /> : <ExportPanel />}
      </div>
    </div>
  );
}

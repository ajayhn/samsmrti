import { useState, useEffect } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { api } from "../../lib/tauri";
import { useDeckStore } from "../../stores/deckStore";

interface ImportResult {
  decks_imported: number;
  notes_imported: number;
  cards_imported: number;
  media_imported: number;
  warnings: string[];
}

interface ImportDialogProps {
  onClose: () => void;
}

export function ImportDialog({ onClose }: ImportDialogProps) {
  const [filePath, setFilePath] = useState<string | null>(null);
  const [importing, setImporting] = useState(false);
  const [result, setResult] = useState<ImportResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const { fetchDecks } = useDeckStore();

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  const handleSelectFile = async () => {
    const selected = await open({
      multiple: false,
          filters: [
        {
          name: "Flashcard Files",
          extensions: ["apkg", "anki2", "anki21", "mochi"],
        },
      ],
    });
    if (selected) {
      setFilePath(selected as string);
      setError(null);
      setResult(null);
    }
  };

  const handleImport = async () => {
    if (!filePath) return;
    setImporting(true);
    setError(null);
    try {
      const res = await api.importFile(filePath);
      setResult(res);
      await fetchDecks();
    } catch (e) {
      setError(String(e));
    } finally {
      setImporting(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/40 flex items-center justify-center z-50 p-4">
      <div className="bg-surface rounded-2xl border border-border shadow-xl w-full max-w-md p-6 space-y-5">
        <h3 className="text-lg font-bold text-text">Import Deck</h3>

        <p className="text-sm text-text-secondary">
          Import from Anki (.apkg or collection.anki2) or Mochi (.mochi). For your full Anki library, quit Anki and select{" "}
          <code className="text-xs bg-surface-alt px-1 rounded">collection.anki2</code> from your profile folder.
        </p>

        {!result ? (
          <>
            <div
              onClick={handleSelectFile}
              className="border-2 border-dashed border-border rounded-xl p-8 text-center cursor-pointer hover:border-primary-400 hover:bg-surface-alt transition-colors"
            >
              {filePath ? (
                <div>
                  <p className="text-sm font-medium text-text truncate">
                    {filePath.split("/").pop() || filePath}
                  </p>
                  <p className="text-xs text-text-muted mt-1">
                    Click to change file
                  </p>
                </div>
              ) : (
                <div>
                  <p className="text-sm text-text-secondary">
                    Click to select a file
                  </p>
                  <p className="text-xs text-text-muted mt-1">
                    Supports .apkg and .mochi formats
                  </p>
                </div>
              )}
            </div>

            {error && (
              <div className="px-4 py-3 bg-red-50 dark:bg-red-900/20 rounded-xl text-sm text-danger">
                {error}
              </div>
            )}

            <div className="flex gap-3">
              <button
                onClick={handleImport}
                disabled={!filePath || importing}
                className="flex-1 px-4 py-2.5 bg-primary-600 text-white rounded-xl text-sm font-medium hover:bg-primary-700 disabled:opacity-50 transition-colors"
              >
                {importing ? "Importing..." : "Import"}
              </button>
              <button
                onClick={onClose}
                className="px-4 py-2.5 text-text-secondary hover:bg-surface-hover rounded-xl text-sm transition-colors"
              >
                Cancel
              </button>
            </div>
          </>
        ) : (
          <>
            <div className="space-y-3">
              <div className="grid grid-cols-2 gap-3 text-center">
                <div className="bg-surface-alt rounded-xl p-4 border border-border">
                  <p className="text-2xl font-bold text-primary-500">
                    {result.decks_imported}
                  </p>
                  <p className="text-xs text-text-muted">Decks</p>
                </div>
                <div className="bg-surface-alt rounded-xl p-4 border border-border">
                  <p className="text-2xl font-bold text-success">
                    {result.cards_imported}
                  </p>
                  <p className="text-xs text-text-muted">Cards</p>
                </div>
                <div className="bg-surface-alt rounded-xl p-4 border border-border">
                  <p className="text-2xl font-bold text-text">
                    {result.notes_imported}
                  </p>
                  <p className="text-xs text-text-muted">Notes</p>
                </div>
                <div className="bg-surface-alt rounded-xl p-4 border border-border">
                  <p className="text-2xl font-bold text-text">
                    {result.media_imported}
                  </p>
                  <p className="text-xs text-text-muted">Media</p>
                </div>
              </div>

              {result.warnings.length > 0 && (
                <div className="bg-orange-50 dark:bg-orange-900/20 rounded-xl p-3">
                  <p className="text-xs font-medium text-orange-700 dark:text-orange-300 mb-1">
                    Warnings ({result.warnings.length})
                  </p>
                  <div className="max-h-32 overflow-y-auto space-y-1">
                    {result.warnings.slice(0, 10).map((w, i) => (
                      <p key={i} className="text-xs text-orange-600 dark:text-orange-400">
                        {w}
                      </p>
                    ))}
                    {result.warnings.length > 10 && (
                      <p className="text-xs text-orange-500">
                        ...and {result.warnings.length - 10} more
                      </p>
                    )}
                  </div>
                </div>
              )}
            </div>

            <button
              onClick={onClose}
              className="w-full px-4 py-2.5 bg-primary-600 text-white rounded-xl text-sm font-medium hover:bg-primary-700 transition-colors"
            >
              Done
            </button>
          </>
        )}
      </div>
    </div>
  );
}

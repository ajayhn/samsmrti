import { useState, useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { useDeckStore } from "../../stores/deckStore";
import { isNativeEditShortcut } from "../../lib/keyboard";
import { DeckSettings } from "./DeckSettings";

function Kbd({ children }: { children: string }) {
  return (
    <kbd className="ml-1.5 inline-flex items-center justify-center min-w-[1.25rem] h-5 px-1 text-[10px] font-semibold uppercase rounded bg-white/20 border border-white/20">
      {children}
    </kbd>
  );
}

function KbdDark({ children }: { children: string }) {
  return (
    <kbd className="ml-1.5 inline-flex items-center justify-center min-w-[1.25rem] h-5 px-1 text-[10px] font-semibold uppercase rounded bg-black/10 dark:bg-white/10 border border-black/10 dark:border-white/10 text-text-muted">
      {children}
    </kbd>
  );
}

export function DeckHome() {
  const { decks, selectedDeckId, createDeck, deleteDeck } =
    useDeckStore();
  const selectedDeck = decks.find((d) => d.id === selectedDeckId);
  const navigate = useNavigate();

  const [showCreate, setShowCreate] = useState(false);
  const [createParentId, setCreateParentId] = useState<string | null>(null);
  const [newName, setNewName] = useState("");
  const [creating, setCreating] = useState(false);
  const [showSettings, setShowSettings] = useState(false);

  const openCreate = (parentId: string | null = null) => {
    setCreateParentId(parentId);
    setShowCreate(true);
  };

  const handleCreate = async () => {
    if (!newName.trim()) return;
    setCreating(true);
    try {
      const deck = await createDeck(newName.trim(), createParentId ?? undefined);
      setNewName("");
      setShowCreate(false);
      setCreateParentId(null);
      useDeckStore.getState().selectDeck(deck.id);
    } finally {
      setCreating(false);
    }
  };

  const subdeckCount = selectedDeck
    ? decks.filter((d) => d.parent_id === selectedDeck.id).length
    : 0;

  const handleDelete = async (id: string) => {
    const deck = decks.find((d) => d.id === id);
    if (!deck) return;
    const childCount = decks.filter((d) => d.parent_id === id).length;
    const msg =
      childCount > 0
        ? `Delete "${deck.name}" and all ${childCount} subdeck(s) and their cards?`
        : `Delete "${deck.name}" and all its cards?`;
    if (!confirm(msg)) return;
    await deleteDeck(id);
  };

  const isInputFocused = () => {
    const tag = document.activeElement?.tagName;
    return tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT" ||
           (document.activeElement as HTMLElement)?.isContentEditable;
  };

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (isNativeEditShortcut(e)) return;
      if (isInputFocused()) return;
      if (showSettings || showCreate) return;

      if (!selectedDeck) {
        if (e.key === "n" || e.key === "N") {
          e.preventDefault();
          openCreate();
        }
        return;
      }

      if (e.key === "s" || e.key === "S") {
        if (selectedDeck.due_cards > 0 || selectedDeck.new_cards > 0) {
          e.preventDefault();
          navigate(`/review/${selectedDeck.id}`);
        }
      } else if ((e.key === "a" || e.key === "A") && !e.metaKey && !e.ctrlKey) {
        e.preventDefault();
        navigate(`/add/${selectedDeck.id}`);
      } else if ((e.key === "v" || e.key === "V") && !e.metaKey && !e.ctrlKey) {
        e.preventDefault();
        navigate(`/deck/${selectedDeck.id}/cards`);
      } else if (e.key === "n" || e.key === "N") {
        e.preventDefault();
        openCreate();
      }
    },
    [selectedDeck, navigate, showSettings, showCreate]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  if (!selectedDeck) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-6 p-8">
        <div className="text-center max-w-md">
          <h2 className="text-2xl font-bold text-text mb-2">
            Welcome to Samsmrti
          </h2>
          <p className="text-text-secondary">
            Select a deck from the sidebar to start studying, or create a new
            one.
          </p>
        </div>

        {!showCreate ? (
          <button
            onClick={() => openCreate()}
            className="px-6 py-3 bg-primary-600 text-white rounded-xl font-medium hover:bg-primary-700 transition-colors shadow-sm"
          >
            Create Your First Deck
            <Kbd>N</Kbd>
          </button>
        ) : (
          <div className="flex gap-2 w-full max-w-sm">
            <input
              type="text"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleCreate();
                if (e.key === "Escape") { setShowCreate(false); setNewName(""); }
              }}
              placeholder={createParentId ? "Subdeck name..." : "Deck name..."}
              autoFocus
              className="flex-1 px-4 py-2.5 bg-surface border border-border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent"
            />
            <button
              onClick={handleCreate}
              disabled={creating || !newName.trim()}
              className="px-5 py-2.5 bg-primary-600 text-white rounded-xl text-sm font-medium hover:bg-primary-700 disabled:opacity-50 transition-colors"
            >
              Create
            </button>
            <button
              onClick={() => {
                setShowCreate(false);
                setCreateParentId(null);
                setNewName("");
              }}
              className="px-3 py-2.5 text-text-secondary hover:bg-surface-hover rounded-xl text-sm transition-colors"
            >
              Cancel
            </button>
          </div>
        )}

        {decks.length > 0 && (
          <div className="w-full max-w-lg mt-4">
            <h3 className="text-sm font-medium text-text-secondary mb-3">
              Your Decks
            </h3>
            <div className="space-y-2">
              {decks
                .filter((d) => !d.parent_id)
                .map((deck) => (
                  <div
                    key={deck.id}
                    onClick={() => useDeckStore.getState().selectDeck(deck.id)}
                    className="flex items-center justify-between p-4 bg-surface-alt rounded-xl border border-border hover:border-primary-300 cursor-pointer transition-colors"
                  >
                    <div>
                      <p className="font-medium text-text">{deck.name}</p>
                      <p className="text-xs text-text-muted mt-0.5">
                        {deck.total_cards} cards
                      </p>
                    </div>
                    <div className="flex gap-3 text-sm">
                      <span className="text-primary-500 font-semibold">
                        {deck.new_cards} new
                      </span>
                      <span className="text-warning font-semibold">
                        {deck.due_cards} due
                      </span>
                    </div>
                  </div>
                ))}
            </div>
          </div>
        )}
      </div>
    );
  }

  return (
    <div className="p-8 max-w-3xl mx-auto">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h2 className="text-2xl font-bold text-text">{selectedDeck.name}</h2>
          <p className="text-text-secondary mt-1">
            {selectedDeck.description ||
              (subdeckCount > 0
                ? `Includes ${subdeckCount} subdeck${subdeckCount !== 1 ? "s" : ""} — counts include all subdecks`
                : "Counts include all subdecks when nested")}
          </p>
        </div>
        <div className="flex items-center gap-3">
          <button
            onClick={() => setShowSettings(true)}
            className="text-sm text-text-secondary hover:text-text transition-colors cursor-pointer"
          >
            Settings
          </button>
          <button
            onClick={() => handleDelete(selectedDeck.id)}
            className="text-xs text-danger hover:underline cursor-pointer"
          >
            Delete
          </button>
        </div>
      </div>

      <div className="grid grid-cols-3 gap-4 mb-8">
        <div className="bg-surface-alt rounded-xl p-5 border border-border text-center">
          <p className="text-3xl font-bold text-primary-500">
            {selectedDeck.new_cards}
          </p>
          <p className="text-sm text-text-secondary mt-1">New</p>
        </div>
        <div className="bg-surface-alt rounded-xl p-5 border border-border text-center">
          <p className="text-3xl font-bold text-warning">
            {selectedDeck.due_cards}
          </p>
          <p className="text-sm text-text-secondary mt-1">Due</p>
        </div>
        <div className="bg-surface-alt rounded-xl p-5 border border-border text-center">
          <p className="text-3xl font-bold text-text">
            {selectedDeck.total_cards}
          </p>
          <p className="text-sm text-text-secondary mt-1">Total</p>
        </div>
      </div>

      <div className="flex flex-wrap gap-3">
        {(selectedDeck.due_cards > 0 || selectedDeck.new_cards > 0) && (
          <button
            onClick={() => navigate(`/review/${selectedDeck.id}`)}
            className="flex-1 px-6 py-3.5 bg-primary-600 text-white rounded-xl font-medium hover:bg-primary-700 transition-colors shadow-sm text-center"
          >
            Study Now
            <Kbd>S</Kbd>
          </button>
        )}
        <button
          onClick={() => navigate(`/deck/${selectedDeck.id}/cards`)}
          className="flex-1 min-w-[140px] px-6 py-3.5 bg-surface-alt border border-border text-text rounded-xl font-medium hover:bg-surface-hover transition-colors text-center"
        >
          View Cards
          <KbdDark>V</KbdDark>
        </button>
        <button
          onClick={() => navigate(`/add/${selectedDeck.id}`)}
          className="flex-1 min-w-[140px] px-6 py-3.5 bg-surface-alt border border-border text-text rounded-xl font-medium hover:bg-surface-hover transition-colors text-center"
        >
          Add Cards
          <KbdDark>A</KbdDark>
        </button>
        <button
          onClick={() => openCreate(selectedDeck.id)}
          className="px-6 py-3.5 bg-surface-alt border border-border text-text-secondary rounded-xl font-medium hover:bg-surface-hover transition-colors text-center"
        >
          Create Subdeck
        </button>
      </div>

      {showCreate && (
        <div className="mt-6 flex gap-2 max-w-md">
          <input
            type="text"
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") handleCreate();
              if (e.key === "Escape") {
                setShowCreate(false);
                setCreateParentId(null);
                setNewName("");
              }
            }}
            placeholder="Subdeck name..."
            autoFocus
            className="flex-1 px-4 py-2.5 bg-surface-alt border border-border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
          />
          <button
            onClick={handleCreate}
            disabled={creating || !newName.trim()}
            className="px-5 py-2.5 bg-primary-600 text-white rounded-xl text-sm font-medium hover:bg-primary-700 disabled:opacity-50"
          >
            Create
          </button>
          <button
            onClick={() => {
              setShowCreate(false);
              setCreateParentId(null);
              setNewName("");
            }}
            className="px-3 py-2.5 text-text-secondary hover:bg-surface-hover rounded-xl text-sm"
          >
            Cancel
          </button>
        </div>
      )}

      {showSettings && (
        <DeckSettings
          deck={selectedDeck}
          onClose={() => setShowSettings(false)}
        />
      )}
    </div>
  );
}

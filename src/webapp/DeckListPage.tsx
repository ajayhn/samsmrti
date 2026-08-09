import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { useDeckStore } from "../stores/deckStore";
import { useProfileStore } from "../stores/profileStore";
import { KarmaDisplay } from "../components/karma/KarmaDisplay";
import { ProfileOnboarding } from "../components/profile/ProfileOnboarding";

export function DeckListPage() {
  const navigate = useNavigate();
  const { decks, loading, fetchDecks } = useDeckStore();
  const { active } = useProfileStore();

  useEffect(() => {
    fetchDecks();
  }, [fetchDecks]);

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-4 py-3 border-b border-border">
        <h1 className="text-lg font-semibold text-text">Samsmrti</h1>
        <div className="flex items-center gap-3">
          <KarmaDisplay />
          {active && (
            <span className="text-sm text-text-muted">{active.display_name}</span>
          )}
          <button
            onClick={() => navigate("/sync")}
            className="text-sm text-primary-500 font-medium"
          >
            Sync
          </button>
        </div>
      </div>
      <ProfileOnboarding />

      <div className="flex-1 overflow-y-auto p-4">
        {loading && decks.length === 0 && (
          <p className="text-text-muted text-center mt-8">Loading decks…</p>
        )}
        {!loading && decks.length === 0 && (
          <div className="text-center mt-8 space-y-3">
            <p className="text-text-muted">No decks yet.</p>
            <button
              onClick={() => navigate("/sync")}
              className="px-5 py-2.5 bg-primary-600 text-white rounded-xl font-medium"
            >
              Import a deck
            </button>
          </div>
        )}
        <div className="space-y-2">
          {decks.map((deck) => (
            <button
              key={deck.id}
              onClick={() => navigate(`/review/${deck.id}`)}
              className="w-full flex items-center justify-between px-4 py-4 bg-surface-alt rounded-xl border border-border text-left"
            >
              <div>
                <div className="font-medium text-text">{deck.name}</div>
                <div className="text-xs text-text-muted mt-0.5">
                  {deck.total_cards} cards
                </div>
              </div>
              <div className="flex items-center gap-2">
                {deck.new_cards > 0 && (
                  <span className="text-xs px-2 py-1 rounded-full bg-primary-100 text-primary-700 dark:bg-primary-900/30 dark:text-primary-300 font-medium">
                    {deck.new_cards} new
                  </span>
                )}
                {deck.due_cards > 0 && (
                  <span className="text-xs px-2 py-1 rounded-full bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-300 font-medium">
                    {deck.due_cards} due
                  </span>
                )}
              </div>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}

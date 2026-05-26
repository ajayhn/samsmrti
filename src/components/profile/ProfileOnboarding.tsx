import { useState } from "react";
import { useProfileStore } from "../../stores/profileStore";

export function ProfileOnboarding() {
  const show = useProfileStore((s) => s.showOnboarding);
  const createProfile = useProfileStore((s) => s.createProfile);
  const [name, setName] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  if (!show) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const trimmed = name.trim();
    if (!trimmed) {
      setError("Enter your name");
      return;
    }
    setBusy(true);
    setError("");
    try {
      await createProfile(trimmed);
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/50 p-4">
      <div className="bg-surface border border-border rounded-2xl shadow-xl max-w-md w-full p-6 space-y-4">
        <h2 className="text-lg font-bold text-text">Who&apos;s studying?</h2>
        <p className="text-sm text-text-secondary">
          Create a profile so your Karma points and streak stay yours. When
          someone else uses Samsmrti on this computer, switch profiles first —
          honor system, no password.
        </p>
        <form onSubmit={handleSubmit} className="space-y-3">
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Your name"
            className="w-full px-3 py-2 rounded-lg border border-border bg-surface-alt text-text"
            autoFocus
            maxLength={64}
          />
          {error && (
            <p className="text-sm text-red-500">{error}</p>
          )}
          <button
            type="submit"
            disabled={busy}
            className="w-full py-2 rounded-lg bg-primary-500 text-white font-medium hover:opacity-90 disabled:opacity-50 cursor-pointer"
          >
            {busy ? "Creating…" : "Create profile"}
          </button>
        </form>
      </div>
    </div>
  );
}

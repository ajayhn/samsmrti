import { useState } from "react";
import { useProfileStore } from "../../stores/profileStore";

export function ProfileSwitcher() {
  const profiles = useProfileStore((s) => s.profiles);
  const active = useProfileStore((s) => s.active);
  const switchProfile = useProfileStore((s) => s.switchProfile);
  const createProfile = useProfileStore((s) => s.createProfile);
  const [open, setOpen] = useState(false);
  const [creating, setCreating] = useState(false);
  const [newName, setNewName] = useState("");

  if (!active) return null;

  const initial = active.display_name.charAt(0).toUpperCase();

  const handleCreate = async () => {
    const trimmed = newName.trim();
    if (!trimmed) return;
    try {
      await createProfile(trimmed);
      setNewName("");
      setCreating(false);
      setOpen(false);
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <div className="relative px-2 mb-2">
      <button
        type="button"
        onClick={() => setOpen(!open)}
        className="w-full flex items-center gap-2 px-3 py-2 rounded-lg hover:bg-surface-hover text-left text-sm cursor-pointer"
        title="Switch profile (honor system)"
      >
        <span
          className={`w-7 h-7 rounded-full flex items-center justify-center text-xs font-bold shrink-0 ${
            active.is_admin
              ? "bg-surface-alt text-text-muted"
              : "bg-primary-500/20 text-primary-500"
          }`}
        >
          {initial}
        </span>
        <span className="truncate text-text flex-1">{active.display_name}</span>
        <span className="text-text-muted text-xs">▾</span>
      </button>

      {open && (
        <>
          <div
            className="fixed inset-0 z-40"
            onClick={() => setOpen(false)}
          />
          <div className="absolute left-2 right-2 bottom-full mb-1 z-50 bg-surface border border-border rounded-xl shadow-lg py-1 max-h-48 overflow-y-auto">
            {profiles.map((p) => (
              <button
                key={p.id}
                type="button"
                onClick={() => {
                  switchProfile(p.id);
                  setOpen(false);
                }}
                className={`w-full px-3 py-2 text-sm text-left hover:bg-surface-hover cursor-pointer ${
                  p.id === active.id ? "text-primary-500 font-medium" : "text-text"
                }`}
              >
                {p.display_name}
                {p.is_admin && (
                  <span className="text-text-muted ml-1">(no karma)</span>
                )}
              </button>
            ))}
            {creating ? (
              <div className="px-2 py-2 flex gap-1">
                <input
                  type="text"
                  value={newName}
                  onChange={(e) => setNewName(e.target.value)}
                  placeholder="Name"
                  className="flex-1 px-2 py-1 text-sm rounded border border-border bg-surface-alt"
                  onKeyDown={(e) => e.key === "Enter" && handleCreate()}
                />
                <button
                  type="button"
                  onClick={handleCreate}
                  className="px-2 py-1 text-xs bg-primary-500 text-white rounded cursor-pointer"
                >
                  Add
                </button>
              </div>
            ) : (
              <button
                type="button"
                onClick={() => setCreating(true)}
                className="w-full px-3 py-2 text-sm text-primary-500 text-left hover:bg-surface-hover cursor-pointer"
              >
                + New profile
              </button>
            )}
          </div>
        </>
      )}
    </div>
  );
}

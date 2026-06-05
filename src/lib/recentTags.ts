const STORAGE_KEY = "samsmrti-recent-tags";
const MAX_RECENT = 40;

/** Tags you added recently in this app (browser localStorage), most recent first. */
export function getRecentTags(): string[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter((t): t is string => typeof t === "string" && t.trim().length > 0);
  } catch {
    return [];
  }
}

export function recordRecentTag(name: string): void {
  const trimmed = name.trim();
  if (!trimmed) return;
  const lower = trimmed.toLowerCase();
  const prev = getRecentTags().filter((t) => t.toLowerCase() !== lower);
  const next = [trimmed, ...prev].slice(0, MAX_RECENT);
  localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
}

const STORAGE_KEY = "deck-browse-bookmark-v1";

type BookmarkStore = Record<string, string>;

function storageKey(profileId: string, deckId: string, tagFilter: string) {
  return `${profileId}|${deckId}|${tagFilter}`;
}

function readStore(): BookmarkStore {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return {};
    return JSON.parse(raw) as BookmarkStore;
  } catch {
    return {};
  }
}

function writeStore(store: BookmarkStore) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(store));
}

export function loadDeckBrowseBookmark(
  profileId: string,
  deckId: string,
  tagFilter: string
): string | null {
  const key = storageKey(profileId, deckId, tagFilter);
  return readStore()[key] ?? null;
}

export function saveDeckBrowseBookmark(
  profileId: string,
  deckId: string,
  tagFilter: string,
  noteId: string
) {
  const store = readStore();
  store[storageKey(profileId, deckId, tagFilter)] = noteId;
  writeStore(store);
}

export function clearDeckBrowseBookmark(
  profileId: string,
  deckId: string,
  tagFilter: string
) {
  const store = readStore();
  delete store[storageKey(profileId, deckId, tagFilter)];
  writeStore(store);
}

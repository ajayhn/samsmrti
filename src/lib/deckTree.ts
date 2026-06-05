import type { DeckWithCounts } from "./tauri";

const COLLAPSED_STORAGE_KEY = "samsmrti-deck-collapsed";

export function loadCollapsedDeckIds(): Set<string> {
  try {
    const raw = localStorage.getItem(COLLAPSED_STORAGE_KEY);
    if (!raw) return new Set();
    const parsed = JSON.parse(raw) as string[];
    return new Set(Array.isArray(parsed) ? parsed : []);
  } catch {
    return new Set();
  }
}

export function saveCollapsedDeckIds(ids: Set<string>): void {
  localStorage.setItem(COLLAPSED_STORAGE_KEY, JSON.stringify([...ids]));
}

export interface DeckTreeFilterResult {
  visibleIds: Set<string>;
  /** Ancestors of search hits — keep expanded while filtering */
  forceExpandedIds: Set<string>;
  hasQuery: boolean;
}

export function filterDeckTree(
  decks: DeckWithCounts[],
  query: string
): DeckTreeFilterResult {
  const allIds = new Set(decks.map((d) => d.id));
  const q = query.trim().toLowerCase();

  if (!q) {
    return { visibleIds: allIds, forceExpandedIds: new Set(), hasQuery: false };
  }

  const byId = new Map(decks.map((d) => [d.id, d]));
  const visibleIds = new Set<string>();
  const forceExpandedIds = new Set<string>();

  const addAncestors = (deckId: string) => {
    let pid: string | null | undefined = byId.get(deckId)?.parent_id;
    while (pid) {
      visibleIds.add(pid);
      forceExpandedIds.add(pid);
      pid = byId.get(pid)?.parent_id;
    }
  };

  const addDescendants = (deckId: string) => {
    for (const child of decks.filter((d) => d.parent_id === deckId)) {
      visibleIds.add(child.id);
      addDescendants(child.id);
    }
  };

  for (const deck of decks) {
    if (!deck.name.toLowerCase().includes(q)) continue;
    visibleIds.add(deck.id);
    addAncestors(deck.id);
    addDescendants(deck.id);
  }

  return { visibleIds, forceExpandedIds, hasQuery: true };
}

export function deckMatchesQuery(deck: DeckWithCounts, query: string): boolean {
  const q = query.trim().toLowerCase();
  if (!q) return false;
  return deck.name.toLowerCase().includes(q);
}

export interface FlatDeckOption {
  id: string;
  name: string;
  depth: number;
}

/** Deck tree order for pickers (parents before children). */
export function flattenDecksForPicker(decks: DeckWithCounts[]): FlatDeckOption[] {
  const byParent = new Map<string | null, DeckWithCounts[]>();
  for (const deck of decks) {
    const key = deck.parent_id;
    if (!byParent.has(key)) byParent.set(key, []);
    byParent.get(key)!.push(deck);
  }
  for (const group of byParent.values()) {
    group.sort((a, b) => a.name.localeCompare(b.name));
  }

  const out: FlatDeckOption[] = [];
  const walk = (parentId: string | null, depth: number) => {
    for (const deck of byParent.get(parentId) ?? []) {
      out.push({ id: deck.id, name: deck.name, depth });
      walk(deck.id, depth + 1);
    }
  };
  walk(null, 0);
  return out;
}

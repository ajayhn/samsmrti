import type { NavigateFunction } from "react-router-dom";

/** Navigate after picking a deck in the sidebar, preserving the current screen when possible. */
export function navigateAfterDeckSelect(
  navigate: NavigateFunction,
  deckId: string,
  pathname: string,
  search: string
) {
  if (/^\/deck\/[^/]+\/cards\/?$/.test(pathname)) {
    navigate(`/deck/${deckId}/cards`);
    return;
  }
  if (/^\/add\/[^/]+\/?$/.test(pathname)) {
    navigate(`/add/${deckId}`);
    return;
  }
  if (pathname === "/browse" || pathname.startsWith("/browse")) {
    const params = new URLSearchParams(search);
    params.set("deckId", deckId);
    navigate({ pathname: "/browse", search: params.toString() });
    return;
  }
  if (pathname.startsWith("/review/")) {
    return;
  }
  navigate(`/deck/${deckId}/cards`);
}

/** Deck id to load in the main pane: sidebar selection wins until the URL catches up. */
export function resolveActiveDeckId(
  routeDeckId: string | undefined,
  selectedDeckId: string | null
): string | undefined {
  if (selectedDeckId && selectedDeckId !== routeDeckId) {
    return selectedDeckId;
  }
  return routeDeckId ?? selectedDeckId ?? undefined;
}

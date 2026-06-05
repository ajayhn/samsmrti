import type { NavigateFunction } from "react-router-dom";

/** Open notes filtered by tag — View Cards when on a deck, otherwise Card Browser. */
export function navigateToTagFilter(
  navigate: NavigateFunction,
  tagName: string,
  pathname: string
) {
  const encoded = encodeURIComponent(tagName);
  const deckCards = pathname.match(/^\/deck\/([^/]+)\/cards\/?$/);
  if (deckCards) {
    navigate(`/deck/${deckCards[1]}/cards?tag=${encoded}`);
    return;
  }
  navigate({ pathname: "/browse", search: `?tag=${encoded}` });
}

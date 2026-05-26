import {
  combineHintPhrases,
  formatHintPhrase,
  isDistinctivePrompt,
  promptKindFromOrdinal,
  scoreHintCandidate,
  type CountryPromptKind,
  type HintCandidateKind,
  type ScoredHintCandidate,
} from "./countryGeo";

const CLOZE_RE = /\{\{c(\d+)::([^}]*?)(?:::([^}]*?))?\}\}/g;

export function countClozeDeletions(text: string): number {
  let maxN = 0;
  for (const match of text.matchAll(CLOZE_RE)) {
    const n = parseInt(match[1], 10);
    if (!Number.isNaN(n)) maxN = Math.max(maxN, n);
  }
  return maxN;
}

/** Quizbowl tossup/bonus: reveal clozes one at a time on a single card. */
export function isProgressiveQuizbowlCard(
  isCloze: boolean,
  fields: Record<string, string>
): boolean {
  if (!isCloze) return false;
  const extra = fields.Extra ?? "";
  if (!extra.startsWith("Bonus") && !extra.startsWith("Tossup")) return false;
  return countClozeDeletions(fields.Text ?? "") > 1;
}

/** @deprecated Use isProgressiveQuizbowlCard */
export function isProgressiveBonusCard(
  isCloze: boolean,
  fields: Record<string, string>
): boolean {
  return isProgressiveQuizbowlCard(isCloze, fields);
}

export function progressiveQuizbowlKind(
  fields: Record<string, string>
): "bonus" | "tossup" | null {
  const extra = fields.Extra ?? "";
  if (extra.startsWith("Bonus")) return "bonus";
  if (extra.startsWith("Tossup")) return "tossup";
  return null;
}

const PART_LABELS = ["A", "B", "C", "D", "E"] as const;

export function progressiveRevealLabel(
  step: number,
  fields: Record<string, string>
): string {
  const kind = progressiveQuizbowlKind(fields);
  if (kind === "tossup") {
    if (step === 0) return "Reveal post-power";
    if (step === 1) return "Reveal answer";
    return "Show Answer";
  }
  if (kind === "bonus") {
    return `Reveal part ${PART_LABELS[step] ?? step + 1}`;
  }
  return "Show Answer";
}

export function renderClozeProgressive(
  text: string,
  revealedThrough: number,
  showAll: boolean
): string {
  return text.replace(CLOZE_RE, (_match, nStr, answer) => {
    const n = parseInt(nStr, 10);
    if (showAll || n <= revealedThrough) {
      return `<span class="cloze-answer">${answer}</span>`;
    }
    return `<span class="cloze-blank">[...]</span>`;
  });
}

export function renderStudyContent(
  fields: Record<string, string>,
  frontHtml: string,
  backHtml: string,
  isCloze: boolean,
  templateOrdinal: number,
  showAnswer: boolean,
  progressiveRevealStep = 0
): string {
  if (isProgressiveQuizbowlCard(isCloze, fields)) {
    const total = countClozeDeletions(fields.Text ?? "");
    let html = renderClozeProgressive(
      fields.Text ?? "",
      showAnswer ? total : progressiveRevealStep,
      showAnswer
    );
    if (showAnswer && fields.Extra) {
      html += `<br><br>${fields.Extra}`;
    }
    return html;
  }
  return renderTemplate(
    showAnswer ? backHtml : frontHtml,
    fields,
    isCloze,
    templateOrdinal,
    showAnswer,
    frontHtml
  );
}

export function renderCloze(
  text: string,
  cardOrdinal: number,
  showAnswer: boolean
): string {
  const targetN = cardOrdinal + 1;

  return text.replace(CLOZE_RE, (_match, nStr, answer, hint) => {
    const n = parseInt(nStr, 10);

    if (n === targetN) {
      if (showAnswer) {
        return `<span class="cloze-answer">${answer}</span>`;
      }
      const hintText = hint || "...";
      return `<span class="cloze-blank">[${hintText}]</span>`;
    }

    if (showAnswer) {
      return `<span class="cloze-inactive">${answer}</span>`;
    }
    return `<span class="cloze-inactive">[...]</span>`;
  });
}

function splitCsv(value: string): string[] {
  return value.split(",").map((s) => s.trim()).filter(Boolean);
}

function norm(s: string): string {
  return s.trim().toLowerCase();
}

function seedIndex(seed: string, length: number): number {
  if (length <= 0) return 0;
  let h = 0;
  for (let i = 0; i < seed.length; i++) {
    h = (h * 31 + seed.charCodeAt(i)) >>> 0;
  }
  return h % length;
}

function collectHintCandidates(
  fields: Record<string, string>,
  exclude: Set<string>,
  promptKind: CountryPromptKind
): ScoredHintCandidate[] {
  const candidates: ScoredHintCandidate[] = [];

  const addItems = (
    items: string[],
    kind: HintCandidateKind
  ) => {
    for (const item of items) {
      if (exclude.has(norm(item))) continue;
      candidates.push({
        phrase: formatHintPhrase(item, kind),
        score: scoreHintCandidate(item, kind, promptKind),
        kind,
      });
    }
  };

  addItems(splitCsv(fields.Mountains || ""), "mountain");
  addItems(splitCsv(fields.Universities || ""), "university");

  const cities = splitCsv(fields.Cities || "");
  addItems(cities.slice(1), "city");

  const languages = splitCsv(fields.Languages || "");
  if (languages.length > 1) {
    addItems(languages, "language");
  }

  addItems(splitCsv(fields.Rivers || ""), "river");

  if (candidates.length === 0 && cities.length > 0) {
    const fallback = cities.find((c) => !exclude.has(norm(c))) || cities[0];
    if (fallback) {
      candidates.push({
        phrase: formatHintPhrase(fallback, "city"),
        score: scoreHintCandidate(fallback, "city", promptKind),
        kind: "city",
      });
    }
  }

  return candidates;
}

/**
 * Suffix for “→ Country” cards: empty when the prompt item alone is distinctive
 * (e.g. Shanghai → China), otherwise “ that is also home to …” with the strongest
 * disambiguating facts from the same note (e.g. Volga → Elbrus + Saint Petersburg).
 */
export function buildCountryHintSuffix(
  fields: Record<string, string>,
  promptItem: string,
  promptKind: CountryPromptKind,
  seed: string
): string {
  if (promptItem && isDistinctivePrompt(promptItem, promptKind, fields)) {
    return "";
  }

  const exclude = new Set<string>();
  if (promptItem) exclude.add(norm(promptItem));
  if (fields.Country) exclude.add(norm(fields.Country));
  if (fields.Capital) exclude.add(norm(fields.Capital));

  const candidates = collectHintCandidates(fields, exclude, promptKind);
  const needsStrongHint = Boolean(promptItem);

  const phrase =
    candidates.length === 0
      ? needsStrongHint
        ? "distinctive geographic features from the same country"
        : ""
      : pickHintPhrase(candidates, seed, needsStrongHint);

  if (!phrase) return "";
  return ` that is also home to ${phrase}`;
}

function pickHintPhrase(
  candidates: ScoredHintCandidate[],
  seed: string,
  needsStrongHint: boolean
): string {
  const sorted = [...candidates].sort((a, b) => b.score - a.score);
  const maxScore = sorted[0]?.score ?? 0;
  const STRONG = 7;
  const MIN_COMBINED = 6;

  if (needsStrongHint && maxScore < STRONG) {
    const strong = sorted.filter((c) => c.score >= MIN_COMBINED);
    const pool = strong.length >= 2 ? strong.slice(0, 2) : sorted.slice(0, 2);
    return combineHintPhrases(pool.map((c) => c.phrase));
  }

  const tier = sorted.filter((c) => c.score >= maxScore - 1);
  return tier[seedIndex(seed, tier.length)]?.phrase ?? sorted[0].phrase;
}

export function renderTemplate(
  template: string,
  fields: Record<string, string>,
  isCloze: boolean,
  cardOrdinal: number,
  showAnswer: boolean,
  frontSideHtml?: string
): string {
  let result = template;

  const itemIndex = cardOrdinal >= 1000 ? cardOrdinal % 1000 : cardOrdinal;
  const hasEach = /\{\{each:(\w+)\}\}/.test(result);
  let currentItem = "";

  if (hasEach) {
    result = result.replace(/\{\{each:(\w+)\}\}/g, (_match, fieldName) => {
      const value = fields[fieldName] || "";
      const items = splitCsv(value);
      currentItem = items[itemIndex] || items[0] || value;
      return currentItem;
    });
    result = result.replace(/\{\{item\}\}/gi, () => currentItem);
  }

  const needsCountryHints =
    /\{\{hint_suffix\}\}/i.test(result) ||
    /\{\{hint_phrase\}\}/i.test(result) ||
    /\s+that is also home to\s+\{\{hint_phrase\}\}/i.test(result);

  if (needsCountryHints) {
    const promptKind = promptKindFromOrdinal(cardOrdinal);
    const seed = `${fields.Country ?? ""}|${currentItem}|${cardOrdinal}`;
    const suffix =
      promptKind != null
        ? buildCountryHintSuffix(fields, currentItem, promptKind, seed)
        : "";

    result = result.replace(/\{\{hint_suffix\}\}/gi, () => suffix);
    // Legacy templates (dedupe kept a non-ct_ctry_* row with hint_phrase)
    result = result.replace(
      /\s+that is also home to\s+\{\{hint_phrase\}\}/gi,
      () => suffix
    );
    result = result.replace(/\{\{hint_phrase\}\}/gi, () => {
      if (!suffix) return "";
      const prefix = " that is also home to ";
      return suffix.startsWith(prefix) ? suffix.slice(prefix.length) : suffix;
    });
  }

  if (isCloze) {
    result = result.replace(/\{\{cloze:(\w+)\}\}/g, (_match, fieldName) => {
      const value = fields[fieldName] || "";
      return renderCloze(value, cardOrdinal, showAnswer);
    });
  }

  result = result.replace(/\{\{FrontSide\}\}/g, () => {
    const sideTemplate = frontSideHtml ?? fields.Front ?? "";
    return renderTemplate(
      sideTemplate,
      fields,
      isCloze,
      cardOrdinal,
      false,
      frontSideHtml
    );
  });

  result = result.replace(/\{\{(\w+)\}\}/g, (_match, fieldName) => {
    return fields[fieldName] || "";
  });

  return result;
}

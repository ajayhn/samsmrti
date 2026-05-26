export type CountryPromptKind =
  | "river"
  | "language"
  | "mountain"
  | "city"
  | "university";

export function promptKindFromOrdinal(
  cardOrdinal: number
): CountryPromptKind | null {
  if (cardOrdinal < 1000) return null;
  const tmpl = Math.floor(cardOrdinal / 1000);
  switch (tmpl) {
    case 1:
      return "river";
    case 2:
      return "language";
    case 3:
      return "mountain";
    case 4:
      return "city";
    case 5:
      return "university";
    default:
      return null;
  }
}

function norm(s: string): string {
  return s.trim().toLowerCase();
}

/** Rivers that span or evoke multiple countries — need a strong secondary clue. */
const AMBIGUOUS_RIVERS = new Set([
  "volga",
  "nile",
  "danube",
  "rhine",
  "amazon",
  "mississippi",
  "colorado",
  "ganges",
  "paraná",
  "parana",
  "orange",
  "murray",
  "loire",
  "seine",
  "elbe",
  "mekong",
  "yangtze",
  "yellow",
  "pearl",
  "rio grande",
  "limpopo",
  "po",
  "tiber",
]);

const AMBIGUOUS_LANGUAGES = new Set([
  "english",
  "spanish",
  "arabic",
  "french",
  "german",
  "portuguese",
]);

/** How strongly a name points to one country (0–10). */
const UNIQUE_SCORES: Record<string, number> = {
  // Cities
  shanghai: 10,
  tokyo: 10,
  osaka: 9,
  kyoto: 9,
  mumbai: 10,
  bangalore: 9,
  chennai: 9,
  kolkata: 9,
  cairo: 10,
  alexandria: 9,
  luxor: 9,
  giza: 9,
  sydney: 10,
  melbourne: 10,
  brisbane: 9,
  perth: 8,
  "são paulo": 10,
  "sao paulo": 10,
  "rio de janeiro": 10,
  brasília: 9,
  brasilia: 9,
  salvador: 8,
  seoul: 10,
  busan: 9,
  nakdong: 9,
  geum: 8,
  yeongsan: 8,
  incheon: 8,
  "new york": 10,
  "los angeles": 10,
  chicago: 9,
  houston: 8,
  phoenix: 8,
  "mexico city": 10,
  guadalajara: 9,
  monterrey: 8,
  cancún: 9,
  cancun: 9,
  "cape town": 10,
  johannesburg: 10,
  durban: 9,
  "saint petersburg": 10,
  novosibirsk: 10,
  yekaterinburg: 9,
  kazan: 8,
  edinburgh: 9,
  glasgow: 9,
  manchester: 8,
  nice: 8,
  florence: 9,
  naples: 8,
  munich: 9,
  hamburg: 8,
  cologne: 8,
  // Mountains
  "mount fuji": 10,
  fuji: 10,
  denali: 10,
  "mount everest": 9,
  everest: 9,
  k2: 9,
  "mount elbrus": 10,
  elbrus: 10,
  "table mountain": 10,
  hallasan: 10,
  "pico de orizaba": 9,
  popocatépetl: 9,
  popocatepetl: 9,
  "ben nevis": 9,
  snowdon: 8,
  "zugspitze": 9,
  "mont blanc": 7,
  kosciuszko: 10,
  "mount kosciuszko": 10,
  kangchenjunga: 9,
  "nanda devi": 8,
  "pico da neblina": 9,
  mafadi: 9,
  "mount catherine": 9,
  "mount sinai": 9,
  matterhorn: 9,
  jirisan: 8,
  seoraksan: 8,
  "klyuchevskaya sopka": 9,
  // Universities (substring match applied separately)
  "harvard university": 10,
  "massachusetts institute of technology": 10,
  "stanford university": 10,
  "tsinghua university": 10,
  "peking university": 10,
  "university of oxford": 10,
  "university of cambridge": 10,
  "lomonosov moscow state university": 10,
  "seoul national university": 10,
  "university of tokyo": 10,
  "university of melbourne": 9,
  "university of sydney": 9,
  "sorbonne university": 9,
  "école polytechnique": 9,
  "ecole polytechnique": 9,
};

/** Peaks listed on multiple country notes — still need a disambiguating hint. */
const SHARED_MOUNTAINS = new Set(["mont blanc", "monte rosa", "matterhorn"]);

function splitCsv(value: string): string[] {
  return value.split(",").map((s) => s.trim()).filter(Boolean);
}

function lookupUniqueness(name: string): number {
  const key = norm(name);
  if (UNIQUE_SCORES[key] !== undefined) return UNIQUE_SCORES[key];
  const stripped = key.replace(/^mount\s+/, "");
  if (UNIQUE_SCORES[stripped] !== undefined) return UNIQUE_SCORES[stripped];
  for (const [pattern, score] of Object.entries(UNIQUE_SCORES)) {
    if (key.includes(pattern) || pattern.includes(key)) return score;
  }
  return 0;
}

export function isDistinctivePrompt(
  item: string,
  kind: CountryPromptKind,
  fields?: Record<string, string>
): boolean {
  const key = norm(item);
  if (!key) return false;

  switch (kind) {
    case "river":
      return !AMBIGUOUS_RIVERS.has(key);
    case "language":
      return !AMBIGUOUS_LANGUAGES.has(key);
    case "mountain": {
      if (lookupUniqueness(item) >= 8) return true;
      const mountains = splitCsv(fields?.Mountains || "");
      if (
        mountains.length > 0 &&
        norm(mountains[0]) === key &&
        !SHARED_MOUNTAINS.has(key)
      ) {
        return true;
      }
      return false;
    }
    case "city":
    case "university":
      return lookupUniqueness(item) >= 8;
    default:
      return false;
  }
}

export type HintCandidateKind = "mountain" | "university" | "city" | "language" | "river";

export interface ScoredHintCandidate {
  phrase: string;
  score: number;
  kind: HintCandidateKind;
}

function baseKindScore(kind: HintCandidateKind): number {
  switch (kind) {
    case "mountain":
      return 4;
    case "university":
      return 3;
    case "city":
      return 2;
    case "language":
      return 1;
    case "river":
      return 0;
  }
}

export function scoreHintCandidate(
  name: string,
  kind: HintCandidateKind,
  promptKind: CountryPromptKind
): number {
  let score = baseKindScore(kind) + lookupUniqueness(name);

  if (kind === "river" && AMBIGUOUS_RIVERS.has(norm(name))) {
    score -= 6;
  }
  if (kind === "language" && AMBIGUOUS_LANGUAGES.has(norm(name))) {
    score -= 4;
  }
  // Don't pair a weak river hint with an ambiguous river prompt
  if (promptKind === "river" && kind === "river") {
    score -= 3;
  }

  return score;
}

export function formatHintPhrase(name: string, kind: HintCandidateKind): string {
  switch (kind) {
    case "mountain":
      return `the mountain ${name}`;
    case "university":
      return `the university ${name}`;
    case "city":
      return `the city of ${name}`;
    case "language":
      return `where ${name} is an official language`;
    case "river":
      return `the river ${name}`;
  }
}

export function combineHintPhrases(phrases: string[]): string {
  if (phrases.length === 0) return "";
  if (phrases.length === 1) return phrases[0];
  if (phrases.length === 2) return `${phrases[0]} and ${phrases[1]}`;
  return `${phrases.slice(0, -1).join(", ")}, and ${phrases[phrases.length - 1]}`;
}

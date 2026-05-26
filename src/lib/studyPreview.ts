import type { CardTemplate, NoteType } from "./tauri";
import {
  countClozeDeletions,
  isProgressiveQuizbowlCard,
  renderStudyContent,
} from "./cloze";

export function templateForCard(
  noteType: NoteType,
  templateOrdinal: number
): CardTemplate | undefined {
  if (noteType.is_cloze) {
    return noteType.templates.find((t) => t.ordinal === 0) ?? noteType.templates[0];
  }
  const ord =
    templateOrdinal >= 1000
      ? Math.floor(templateOrdinal / 1000)
      : templateOrdinal;
  return (
    noteType.templates.find((t) => t.ordinal === ord) ?? noteType.templates[0]
  );
}

export function studyOrdinalsForNote(
  noteType: NoteType,
  fields: Record<string, string>,
  cardCount: number
): number[] {
  if (isProgressiveQuizbowlCard(noteType.is_cloze, fields)) {
    return [0];
  }
  if (noteType.is_cloze) {
    const text = fields.Text ?? Object.values(fields)[0] ?? "";
    const clozes = countClozeDeletions(text);
    const n = Math.max(clozes, cardCount, 1);
    return Array.from({ length: n }, (_, i) => i);
  }
  return Array.from({ length: Math.max(cardCount, 1) }, (_, i) => i);
}

export function renderStudyFrontHtml(
  noteType: NoteType,
  fields: Record<string, string>,
  templateOrdinal: number
): string {
  const template = templateForCard(noteType, templateOrdinal);
  if (!template) return "";
  return renderStudyContent(
    fields,
    template.front_html,
    template.back_html,
    noteType.is_cloze,
    templateOrdinal,
    false,
    0
  );
}

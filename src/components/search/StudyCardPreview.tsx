import { useEffect, useState } from "react";
import { api, type Card, type NoteType } from "../../lib/tauri";
import {
  countClozeDeletions,
  isProgressiveQuizbowlCard,
  progressiveQuizbowlKind,
  renderStudyContent,
} from "../../lib/cloze";
import { templateForCard } from "../../lib/studyPreview";

const PART_LABELS = ["A", "B", "C", "D", "E"] as const;

function cardLabel(
  noteType: NoteType,
  card: Card,
  index: number,
  total: number,
  fields: Record<string, string>
) {
  const kind = progressiveQuizbowlKind(fields);
  if (isProgressiveQuizbowlCard(noteType.is_cloze, fields)) {
    if (kind === "tossup") {
      return "Tossup · Space to reveal post-power → answer";
    }
    return "Bonus · Space to reveal A → B → C";
  }
  if (noteType.is_cloze) {
    return total > 1 ? `Cloze c${card.template_ordinal + 1}` : "Cloze";
  }
  const template = templateForCard(noteType, card.template_ordinal);
  if (template?.name && total > 1) {
    return template.name;
  }
  return total > 1 ? `Card ${index + 1}` : "Card";
}

function progressiveStepLabel(kind: "bonus" | "tossup", stepIndex: number): string {
  if (kind === "tossup") {
    return stepIndex === 0 ? "Post-power" : "Answer";
  }
  return `Part ${PART_LABELS[stepIndex] ?? stepIndex + 1}`;
}

function CardPreviewPanel({
  noteType,
  fields,
  card,
  index,
  total,
}: {
  noteType: NoteType;
  fields: Record<string, string>;
  card: Card;
  index: number;
  total: number;
}) {
  const [side, setSide] = useState<"front" | "back">("front");
  const [revealStep, setRevealStep] = useState(0);
  const template = templateForCard(noteType, card.template_ordinal);
  const progressive = isProgressiveQuizbowlCard(noteType.is_cloze, fields);
  const quizbowlKind = progressiveQuizbowlKind(fields);
  const clozeTotal = progressive ? countClozeDeletions(fields.Text ?? "") : 0;
  const showAnswer = side === "back";

  const content = template
    ? renderStudyContent(
        fields,
        template.front_html,
        template.back_html,
        noteType.is_cloze,
        card.template_ordinal,
        showAnswer,
        revealStep
      )
    : "";

  return (
    <div className="rounded-xl border border-border bg-surface-alt overflow-hidden">
      <div className="flex items-center justify-between gap-2 px-3 py-2 border-b border-border bg-surface">
        <span className="text-xs font-medium text-text-secondary truncate">
          {cardLabel(noteType, card, index, total, fields)}
        </span>
        <div className="flex rounded-lg border border-border overflow-hidden text-xs shrink-0">
          <button
            type="button"
            onClick={() => setSide("front")}
            className={`px-2.5 py-1 transition-colors cursor-pointer ${
              side === "front"
                ? "bg-primary-100 text-primary-700 dark:bg-primary-900/30 dark:text-primary-300"
                : "text-text-secondary hover:bg-surface-hover"
            }`}
          >
            Front
          </button>
          <button
            type="button"
            onClick={() => setSide("back")}
            className={`px-2.5 py-1 transition-colors cursor-pointer ${
              side === "back"
                ? "bg-primary-100 text-primary-700 dark:bg-primary-900/30 dark:text-primary-300"
                : "text-text-secondary hover:bg-surface-hover"
            }`}
          >
            Back
          </button>
        </div>
      </div>

      {progressive && quizbowlKind && side === "front" && clozeTotal > 0 && (
        <div className="flex flex-wrap items-center gap-1.5 px-3 py-2 border-b border-border bg-surface">
          <span className="text-[11px] text-text-muted mr-1">Simulate study:</span>
          <button
            type="button"
            onClick={() => setRevealStep(0)}
            className={`text-[11px] px-2 py-0.5 rounded-full border cursor-pointer ${
              revealStep === 0
                ? "border-primary-400 bg-primary-50 text-primary-700 dark:bg-primary-900/30 dark:text-primary-300"
                : "border-border text-text-secondary hover:bg-surface-hover"
            }`}
          >
            Start
          </button>
          {Array.from({ length: clozeTotal }, (_, i) => (
            <button
              key={i}
              type="button"
              onClick={() => setRevealStep(i + 1)}
              className={`text-[11px] px-2 py-0.5 rounded-full border cursor-pointer ${
                revealStep === i + 1
                  ? "border-primary-400 bg-primary-50 text-primary-700 dark:bg-primary-900/30 dark:text-primary-300"
                  : "border-border text-text-secondary hover:bg-surface-hover"
              }`}
            >
              {progressiveStepLabel(quizbowlKind, i)}
            </button>
          ))}
        </div>
      )}

      <div
        className="p-4 text-sm text-text leading-relaxed whitespace-pre-wrap prose prose-sm prose-stone dark:prose-invert max-w-none"
        dangerouslySetInnerHTML={{ __html: content }}
      />
    </div>
  );
}

export function StudyCardPreview({
  noteId,
  noteTypeId,
  fields,
  noteTypes,
}: {
  noteId: string;
  noteTypeId: string;
  fields: Record<string, string>;
  noteTypes: NoteType[];
}) {
  const [cards, setCards] = useState<Card[] | null>(null);
  const noteType = noteTypes.find((nt) => nt.id === noteTypeId);

  useEffect(() => {
    setCards(null);
    api
      .getCardsForNote(noteId)
      .then((c) => setCards([...c].sort((a, b) => a.template_ordinal - b.template_ordinal)))
      .catch(() => setCards([]));
  }, [noteId]);

  if (!noteType) {
    return (
      <p className="text-xs text-text-muted">Note type not found for preview.</p>
    );
  }

  if (cards === null) {
    return (
      <p className="text-xs text-text-muted py-2">Loading study preview…</p>
    );
  }

  if (cards.length === 0) {
    return (
      <p className="text-xs text-text-muted">No study cards for this note.</p>
    );
  }

  return (
    <div className="space-y-3">
      <style>{noteType.css}</style>
      <p className="text-xs font-medium text-text-secondary uppercase tracking-wide">
        Study preview
      </p>
      {cards.map((card, index) => (
        <CardPreviewPanel
          key={card.id}
          noteType={noteType}
          fields={fields}
          card={card}
          index={index}
          total={cards.length}
        />
      ))}
    </div>
  );
}

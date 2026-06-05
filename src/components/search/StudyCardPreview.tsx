import { useCallback, useEffect, useState } from "react";
import { isTypingTarget } from "../../lib/isTypingTarget";
import { api, type Card, type NoteType } from "../../lib/tauri";
import { NoteEditorDialog } from "../notes/NoteEditorDialog";
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
  isFlagged,
  flagging,
  onFlag,
  onEdit,
  primaryContent = false,
}: {
  noteType: NoteType;
  fields: Record<string, string>;
  card: Card;
  index: number;
  total: number;
  isFlagged: boolean;
  flagging: boolean;
  onFlag: () => void;
  onEdit: () => void;
  /** First card: keyboard focus target for note text in View Cards. */
  primaryContent?: boolean;
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
          <button
            type="button"
            onClick={onFlag}
            disabled={flagging}
            className={`px-2.5 py-1 transition-colors cursor-pointer border-l border-border disabled:cursor-default disabled:opacity-60 ${
              isFlagged
                ? "bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-200 hover:bg-amber-200/80 dark:hover:bg-amber-900/50"
                : "text-text-secondary hover:bg-surface-hover"
            }`}
            title={isFlagged ? "Remove flag from this card" : "Flag this card"}
          >
            {isFlagged ? "Unflag" : "Flag"}
          </button>
          <button
            type="button"
            onClick={onEdit}
            className="px-2.5 py-1 transition-colors cursor-pointer border-l border-border text-text-secondary hover:bg-surface-hover"
            title="Edit note fields (E)"
          >
            Edit
            <kbd className="ml-1 text-[9px] font-semibold text-text-muted/80">E</kbd>
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
        {...(primaryContent
          ? {
              "data-study-preview-content": true,
              tabIndex: -1,
              className:
                "p-4 text-sm text-text leading-relaxed whitespace-pre-wrap prose prose-sm prose-stone dark:prose-invert max-w-none outline-none focus:ring-2 focus:ring-primary-500/40 rounded-b-xl",
            }
          : {
              className:
                "p-4 text-sm text-text leading-relaxed whitespace-pre-wrap prose prose-sm prose-stone dark:prose-invert max-w-none",
            })}
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
  onFlagStateChange,
  onNoteUpdated,
  enableEditShortcut = false,
}: {
  noteId: string;
  noteTypeId: string;
  fields: Record<string, string>;
  noteTypes: NoteType[];
  /** Called when per-card flag state is loaded or toggled (`true` if any card is flagged). */
  onFlagStateChange?: (anyFlagged: boolean) => void;
  onNoteUpdated?: (
    fields: Record<string, string>,
    tags: string[]
  ) => void;
  /** Listen for E to open the editor (e.g. View Cards). */
  enableEditShortcut?: boolean;
}) {
  const [cards, setCards] = useState<Card[] | null>(null);
  const [flaggedCardIds, setFlaggedCardIds] = useState<Record<string, boolean>>({});
  const [flagging, setFlagging] = useState(false);
  const [flaggingCardId, setFlaggingCardId] = useState<string | null>(null);
  const [localFields, setLocalFields] = useState(fields);
  const [editOpen, setEditOpen] = useState(false);
  const noteType = noteTypes.find((nt) => nt.id === noteTypeId);

  useEffect(() => {
    setLocalFields(fields);
  }, [noteId, fields]);

  const handleToggleFlag = useCallback(async (cardId: string) => {
    if (flagging) return;
    const next = !flaggedCardIds[cardId];
    setFlagging(true);
    setFlaggingCardId(cardId);
    try {
      await api.setCardFlag(cardId, next);
      setFlaggedCardIds((prev) => ({ ...prev, [cardId]: next }));
    } finally {
      setFlagging(false);
      setFlaggingCardId(null);
    }
  }, [flaggedCardIds, flagging]);

  const openEdit = useCallback(() => setEditOpen(true), []);

  useEffect(() => {
    if (!enableEditShortcut) return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (editOpen) {
        if (e.key === "Escape") {
          e.preventDefault();
          setEditOpen(false);
        }
        return;
      }
      if (
        e.key.toLowerCase() === "e" &&
        !e.ctrlKey &&
        !e.metaKey &&
        !e.altKey &&
        !isTypingTarget(e.target)
      ) {
        e.preventDefault();
        openEdit();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [enableEditShortcut, editOpen, openEdit]);

  useEffect(() => {
    if (!onFlagStateChange || cards === null) return;
    onFlagStateChange(Object.values(flaggedCardIds).some(Boolean));
  }, [flaggedCardIds, cards, onFlagStateChange]);

  useEffect(() => {
    setCards(null);
    api
      .getCardsForNote(noteId)
      .then(async (c) => {
        const sorted = [...c].sort((a, b) => a.template_ordinal - b.template_ordinal);
        setCards(sorted);
        const flags = await Promise.all(
          sorted.map(async (card) => [card.id, await api.getCardFlag(card.id)] as const)
        );
        setFlaggedCardIds(Object.fromEntries(flags));
      })
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
          fields={localFields}
          card={card}
          index={index}
          total={cards.length}
          isFlagged={Boolean(flaggedCardIds[card.id])}
          flagging={flagging && flaggingCardId === card.id}
          onFlag={() => handleToggleFlag(card.id)}
          onEdit={openEdit}
          primaryContent={index === 0}
        />
      ))}

      <NoteEditorDialog
        note={{
          note_id: noteId,
          note_type_id: noteTypeId,
          fields: localFields,
        }}
        open={editOpen}
        onClose={() => setEditOpen(false)}
        onSaved={(newFields, tags) => {
          setLocalFields(newFields);
          onNoteUpdated?.(newFields, tags);
        }}
      />
    </div>
  );
}

import type { NoteType } from "../../lib/tauri";
import { NoteTagsPanel } from "../notes/NoteTagsPanel";
import { StudyCardPreview } from "./StudyCardPreview";

export interface NoteSearchResult {
  note_id: string;
  deck_id: string;
  deck_name: string;
  note_type_id: string;
  note_type_name: string;
  fields_json: Record<string, string>;
  tags: string[];
  card_count: number;
  created_at: number;
}

export function notePreviewText(fields: Record<string, string>): string {
  const values = Object.values(fields);
  const first = values[0] || "";
  const stripped = first.replace(/<[^>]*>/g, "").trim();
  return stripped.length > 100 ? stripped.slice(0, 100) + "..." : stripped;
}

/** Single-line summary for compact deck card rows. */
export function notePreviewOneLine(
  fields: Record<string, string>,
  maxLen = 140
): string {
  const values = Object.values(fields);
  const first = values[0] || "";
  const line = first
    .replace(/<[^>]*>/g, "")
    .replace(/\{\{c\d+::([^}]*?)(?:::([^}]*?))?\}\}/g, "[...]")
    .replace(/\s+/g, " ")
    .trim();
  return line.length > maxLen ? `${line.slice(0, maxLen)}…` : line;
}

export function NoteResultList({
  results,
  loading,
  emptyMessage,
  expanded,
  setExpanded,
  selectedNoteId,
  onSelectNote,
  noteTypes,
  showDeckName = true,
  compact = false,
  onDeleteNote,
  onBuryNote,
  onToggleFlagNote,
  onNoteFlagStateChange,
  onTagsUpdated,
  onNoteUpdated,
  noteFlagHints,
  busyNoteId,
  bookmarkNoteId,
  onSetBookmark,
}: {
  results: NoteSearchResult[];
  loading: boolean;
  emptyMessage: string;
  /** Inline expand mode (browser, etc.). */
  expanded?: string | null;
  setExpanded?: (id: string | null) => void;
  /** Master-detail mode: select a row; preview lives elsewhere. */
  selectedNoteId?: string | null;
  onSelectNote?: (id: string | null) => void;
  noteTypes: NoteType[];
  showDeckName?: boolean;
  compact?: boolean;
  onDeleteNote?: (noteId: string) => void | Promise<void>;
  onBuryNote?: (noteId: string) => void | Promise<void>;
  onToggleFlagNote?: (noteId: string) => void | Promise<void>;
  onNoteFlagStateChange?: (noteId: string, anyFlagged: boolean) => void;
  onTagsUpdated?: (noteId: string, tags: string[]) => void;
  onNoteUpdated?: (
    noteId: string,
    fields: Record<string, string>,
    tags: string[]
  ) => void;
  /** Per-note: true if any study card is flagged (drives row Flag/Unflag label). */
  noteFlagHints?: Record<string, boolean>;
  busyNoteId?: string | null;
  /** Resume bookmark: one starred note per deck list (browse / View Cards). */
  bookmarkNoteId?: string | null;
  onSetBookmark?: (noteId: string | null) => void;
}) {
  const selectionMode = onSelectNote != null;
  if (loading) {
    return (
      <div className="text-center text-text-muted text-sm py-4">Loading...</div>
    );
  }

  if (results.length === 0) {
    return (
      <div className="text-center text-text-muted text-sm py-8">{emptyMessage}</div>
    );
  }

  const showActions = Boolean(onDeleteNote || onBuryNote || onToggleFlagNote);

  return (
    <div className={compact ? "space-y-0.5" : "space-y-2"}>
      {results.map((r) => {
        const isBusy = busyNoteId === r.note_id;
        const isActive = selectionMode
          ? selectedNoteId === r.note_id
          : expanded === r.note_id;
        const anyFlagged = Boolean(noteFlagHints?.[r.note_id]);
        const isBookmark = bookmarkNoteId === r.note_id;
        const summary = compact
          ? notePreviewOneLine(r.fields_json)
          : notePreviewText(r.fields_json);

        return (
          <div
            key={r.note_id}
            data-note-id={r.note_id}
            className={`border overflow-hidden ${
              compact ? "rounded-lg" : "rounded-xl"
            } ${
              isActive
                ? "bg-primary-50 border-primary-300 dark:bg-primary-900/20 dark:border-primary-700"
                : isBookmark
                  ? "bg-amber-50/80 border-amber-200 dark:bg-amber-900/15 dark:border-amber-800/60"
                  : "bg-surface-alt border-border"
            }`}
          >
            <div className={`flex items-center gap-2 ${compact ? "min-h-9" : ""}`}>
              {onSetBookmark && (
                <button
                  type="button"
                  onClick={(e) => {
                    e.stopPropagation();
                    onSetBookmark(isBookmark ? null : r.note_id);
                  }}
                  className={`shrink-0 p-1.5 rounded-md transition-colors cursor-pointer ${
                    compact ? "ml-1" : "ml-2"
                  } ${
                    isBookmark
                      ? "text-amber-500 hover:text-amber-600"
                      : "text-text-muted/50 hover:text-amber-500"
                  }`}
                  title={
                    isBookmark
                      ? "Clear resume bookmark"
                      : "Set resume bookmark (start here next time)"
                  }
                  aria-label={
                    isBookmark ? "Clear resume bookmark" : "Set resume bookmark"
                  }
                >
                  <svg
                    className="w-4 h-4"
                    viewBox="0 0 20 20"
                    fill={isBookmark ? "currentColor" : "none"}
                    stroke="currentColor"
                    strokeWidth={isBookmark ? 0 : 1.5}
                    aria-hidden
                  >
                    <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
                  </svg>
                </button>
              )}
              <button
                type="button"
                data-note-row-select
                onClick={() => {
                  if (selectionMode) {
                    onSelectNote(isActive ? null : r.note_id);
                  } else if (setExpanded) {
                    setExpanded(expanded === r.note_id ? null : r.note_id);
                  }
                }}
                className={`flex-1 min-w-0 text-left transition-colors cursor-pointer ${
                  compact ? "px-3 py-2 flex items-center" : "px-4 py-3"
                } ${isActive ? "" : "hover:bg-surface-hover"}`}
              >
                {compact ? (
                  <span className="text-sm text-text truncate">{summary}</span>
                ) : (
                  <>
                    <p className="text-sm font-medium text-text truncate">
                      {summary}
                    </p>
                    <div className="flex items-center gap-2 mt-2 flex-wrap">
                      {showDeckName && (
                        <span className="text-xs text-text-muted">{r.deck_name}</span>
                      )}
                      <span className="text-xs text-text-muted">{r.note_type_name}</span>
                      <span className="text-xs text-text-muted">
                        {r.card_count} card{r.card_count !== 1 ? "s" : ""}
                      </span>
                      {r.tags.slice(0, 3).map((t) => (
                        <span
                          key={t}
                          className="text-xs px-2 py-0.5 bg-primary-100 text-primary-700 dark:bg-primary-900/30 dark:text-primary-300 rounded-full"
                        >
                          {t}
                        </span>
                      ))}
                      {r.tags.length > 3 && (
                        <span className="text-xs text-text-muted">
                          +{r.tags.length - 3} tags
                        </span>
                      )}
                    </div>
                  </>
                )}
              </button>

              {showActions && (
                <div
                  className={`flex shrink-0 gap-1 ${
                    compact ? "pr-2" : "flex-col gap-1.5 p-3"
                  }`}
                >
                  {onToggleFlagNote && (
                    <button
                      type="button"
                      disabled={isBusy}
                      onClick={(e) => {
                        e.stopPropagation();
                        void onToggleFlagNote(r.note_id);
                      }}
                      title={
                        anyFlagged
                          ? "Remove flag from all cards for this note"
                          : "Flag all cards for this note"
                      }
                      className={`text-xs border rounded-md transition-colors cursor-pointer disabled:opacity-50 ${
                        compact ? "px-2 py-0.5" : "px-2.5 py-1 rounded-lg"
                      } ${
                        anyFlagged
                          ? "text-amber-800 dark:text-amber-200 border-amber-300 dark:border-amber-700 bg-amber-50 dark:bg-amber-900/20 hover:bg-amber-100 dark:hover:bg-amber-900/30"
                          : "text-text-secondary border-border hover:bg-surface-hover"
                      }`}
                    >
                      {anyFlagged ? "Unflag" : "Flag"}
                    </button>
                  )}
                  {onBuryNote && (
                    <button
                      type="button"
                      disabled={isBusy}
                      onClick={(e) => {
                        e.stopPropagation();
                        void onBuryNote(r.note_id);
                      }}
                      className={`text-xs border rounded-md hover:bg-surface-hover transition-colors cursor-pointer disabled:opacity-50 ${
                        compact
                          ? "px-2 py-0.5 text-text-secondary border-border"
                          : "px-2.5 py-1 text-text-secondary border-border rounded-lg"
                      }`}
                    >
                      Bury
                    </button>
                  )}
                  {onDeleteNote && (
                    <button
                      type="button"
                      disabled={isBusy}
                      onClick={(e) => {
                        e.stopPropagation();
                        void onDeleteNote(r.note_id);
                      }}
                      className={`text-xs border rounded-md transition-colors cursor-pointer disabled:opacity-50 ${
                        compact
                          ? "px-2 py-0.5 text-danger border-red-200 dark:border-red-900/50 hover:bg-red-50 dark:hover:bg-red-900/20"
                          : "px-2.5 py-1 text-danger border-red-200 dark:border-red-900/50 rounded-lg hover:bg-red-50 dark:hover:bg-red-900/20"
                      }`}
                    >
                      Delete
                    </button>
                  )}
                </div>
              )}
            </div>

            {!selectionMode && expanded === r.note_id && setExpanded && (
              <div className="px-4 py-3 border-t border-border bg-surface space-y-4">
                <NoteTagsPanel
                  noteId={r.note_id}
                  onTagsUpdated={
                    onTagsUpdated
                      ? (tags) => onTagsUpdated(r.note_id, tags)
                      : undefined
                  }
                />
                <StudyCardPreview
                  noteId={r.note_id}
                  noteTypeId={r.note_type_id}
                  fields={r.fields_json}
                  noteTypes={noteTypes}
                  onFlagStateChange={
                    onNoteFlagStateChange
                      ? (anyFlagged) => onNoteFlagStateChange(r.note_id, anyFlagged)
                      : undefined
                  }
                  onNoteUpdated={
                    onNoteUpdated
                      ? (fields, tags) => onNoteUpdated(r.note_id, fields, tags)
                      : undefined
                  }
                />
                <details className="group">
                  <summary className="text-xs font-medium text-text-muted cursor-pointer hover:text-text-secondary">
                    Raw fields
                  </summary>
                  <div className="mt-2 space-y-2">
                    {Object.entries(r.fields_json).map(([key, val]) => (
                      <div key={key}>
                        <p className="text-xs font-medium text-text-secondary">{key}</p>
                        <div
                          className="text-sm text-text mt-0.5 prose prose-sm max-w-none whitespace-pre-wrap"
                          dangerouslySetInnerHTML={{ __html: val }}
                        />
                      </div>
                    ))}
                  </div>
                </details>
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}

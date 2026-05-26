import type { NoteType } from "../../lib/tauri";
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
  noteTypes,
  showDeckName = true,
  compact = false,
  onDeleteNote,
  onBuryNote,
  busyNoteId,
}: {
  results: NoteSearchResult[];
  loading: boolean;
  emptyMessage: string;
  expanded: string | null;
  setExpanded: (id: string | null) => void;
  noteTypes: NoteType[];
  showDeckName?: boolean;
  compact?: boolean;
  onDeleteNote?: (noteId: string) => void | Promise<void>;
  onBuryNote?: (noteId: string) => void | Promise<void>;
  busyNoteId?: string | null;
}) {
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

  const showActions = Boolean(onDeleteNote || onBuryNote);

  return (
    <div className={compact ? "space-y-0.5" : "space-y-2"}>
      {results.map((r) => {
        const isBusy = busyNoteId === r.note_id;
        const summary = compact
          ? notePreviewOneLine(r.fields_json)
          : notePreviewText(r.fields_json);

        return (
          <div
            key={r.note_id}
            className={`bg-surface-alt border border-border overflow-hidden ${
              compact ? "rounded-lg" : "rounded-xl"
            }`}
          >
            <div className={`flex items-center gap-2 ${compact ? "min-h-9" : ""}`}>
              <button
                type="button"
                onClick={() =>
                  setExpanded(expanded === r.note_id ? null : r.note_id)
                }
                className={`flex-1 min-w-0 text-left hover:bg-surface-hover transition-colors cursor-pointer ${
                  compact ? "px-3 py-2 flex items-center" : "px-4 py-3"
                }`}
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

            {expanded === r.note_id && (
              <div className="px-4 py-3 border-t border-border bg-surface space-y-4">
                {r.tags.length > 0 && (
                  <div className="flex flex-wrap gap-1">
                    {r.tags.map((t) => (
                      <span
                        key={t}
                        className="text-xs px-2 py-0.5 bg-primary-100 text-primary-700 dark:bg-primary-900/30 dark:text-primary-300 rounded-full"
                      >
                        {t}
                      </span>
                    ))}
                  </div>
                )}
                <StudyCardPreview
                  noteId={r.note_id}
                  noteTypeId={r.note_type_id}
                  fields={r.fields_json}
                  noteTypes={noteTypes}
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

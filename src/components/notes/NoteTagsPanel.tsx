import { useCallback, useEffect, useRef, useState } from "react";
import { TagInput } from "../editor/TagInput";
import { api } from "../../lib/tauri";
import { useTagListStore } from "../../stores/tagListStore";

export function NoteTagsPanel({
  noteId,
  compact = false,
  onTagsUpdated,
  onArrowLeftToList,
  onArrowDownToContent,
  tagInputRef,
}: {
  noteId: string;
  compact?: boolean;
  onTagsUpdated?: (tags: string[]) => void;
  onArrowLeftToList?: () => void;
  onArrowDownToContent?: () => void;
  tagInputRef?: React.RefObject<HTMLInputElement | null>;
}) {
  const [tags, setTags] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const saveGen = useRef(0);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    api
      .getNoteTags(noteId)
      .then((t) => {
        if (!cancelled) setTags(t);
      })
      .catch(() => {
        if (!cancelled) setError("Could not load tags");
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [noteId]);

  const persistTags = useCallback(
    async (next: string[]) => {
      const gen = ++saveGen.current;
      setSaving(true);
      setError(null);
      try {
        const note = await api.updateNote({ id: noteId, tags: next });
        if (gen !== saveGen.current) return;
        setTags(note.tags);
        useTagListStore.getState().notifyTagsChanged();
        onTagsUpdated?.(note.tags);
      } catch (e) {
        if (gen !== saveGen.current) return;
        setError(e instanceof Error ? e.message : "Failed to save tags");
      } finally {
        if (gen === saveGen.current) setSaving(false);
      }
    },
    [noteId, onTagsUpdated]
  );

  const handleChange = useCallback(
    (next: string[]) => {
      setTags(next);
      void persistTags(next);
    },
    [persistTags]
  );

  return (
    <div className={compact ? "space-y-1.5" : "space-y-2"}>
      <div className="flex items-center gap-2">
        <span
          className={`font-medium text-text-secondary uppercase tracking-wide ${
            compact ? "text-[10px]" : "text-xs"
          }`}
        >
          Tags
        </span>
        {saving && (
          <span className="text-[10px] text-text-muted">Saving…</span>
        )}
      </div>
      {loading ? (
        <p className="text-xs text-text-muted">Loading tags…</p>
      ) : (
        <TagInput
          value={tags}
          onChange={handleChange}
          inputRef={tagInputRef}
          onArrowLeftToList={onArrowLeftToList}
          onArrowDownToContent={onArrowDownToContent}
        />
      )}
      {error && <p className="text-xs text-danger">{error}</p>}
    </div>
  );
}

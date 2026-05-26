import { useEffect, useState } from "react";
import { api, type ReviewCard } from "../../lib/tauri";

interface ReviewNoteEditorProps {
  card: ReviewCard;
  open: boolean;
  onClose: () => void;
  onSaved: (fields: Record<string, string>) => void;
}

export function ReviewNoteEditor({
  card,
  open,
  onClose,
  onSaved,
}: ReviewNoteEditorProps) {
  const [fields, setFields] = useState<Record<string, string>>({});
  const [fieldNames, setFieldNames] = useState<string[]>([]);
  const [tagsText, setTagsText] = useState("");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    setFields({ ...card.fields });
    setError(null);
    api.getNoteTypes().then((types) => {
      const nt = types.find((t) => t.id === card.note_type_id);
      setFieldNames(nt?.fields.map((f) => f.name) ?? Object.keys(card.fields));
    });
    api.getNoteTags(card.note_id).then((tags) => {
      setTagsText(tags.join(", "));
    });
  }, [open, card.note_id, card.note_type_id]);

  if (!open) return null;

  const handleSave = async () => {
    setSaving(true);
    setError(null);
    try {
      const tags = tagsText
        .split(",")
        .map((t) => t.trim())
        .filter(Boolean);
      await api.updateNote({ id: card.note_id, fields, tags });
      onSaved(fields);
      onClose();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to save note");
    } finally {
      setSaving(false);
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50"
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div
        className="w-full max-w-lg max-h-[85vh] overflow-y-auto bg-surface rounded-2xl border border-border shadow-xl"
        role="dialog"
        aria-labelledby="review-edit-title"
      >
        <div className="sticky top-0 flex items-center justify-between px-5 py-4 border-b border-border bg-surface">
          <h2 id="review-edit-title" className="text-lg font-semibold text-text">
            Edit note
          </h2>
          <button
            type="button"
            onClick={onClose}
            className="text-sm text-text-secondary hover:text-text"
          >
            Cancel
            <kbd className="ml-1 text-[10px] font-semibold text-text-muted">Esc</kbd>
          </button>
        </div>

        <div className="p-5 space-y-4">
          <label className="block">
            <span className="text-sm font-medium text-text-secondary">Tags</span>
            <input
              type="text"
              value={tagsText}
              onChange={(e) => setTagsText(e.target.value)}
              placeholder="Comma-separated (e.g. anatomy::heart, exam)"
              className="mt-1 w-full px-3 py-2 text-sm bg-surface-alt border border-border rounded-lg text-text focus:outline-none focus:border-primary-500"
            />
          </label>
          {fieldNames.map((name) => (
            <label key={name} className="block">
              <span className="text-sm font-medium text-text-secondary">{name}</span>
              <textarea
                value={fields[name] ?? ""}
                onChange={(e) =>
                  setFields((prev) => ({ ...prev, [name]: e.target.value }))
                }
                rows={name === "Universities" || name === "Cities" ? 3 : 2}
                className="mt-1 w-full px-3 py-2 text-sm bg-surface-alt border border-border rounded-lg text-text focus:outline-none focus:border-primary-500 resize-y"
              />
            </label>
          ))}
          {error && <p className="text-sm text-danger">{error}</p>}
        </div>

        <div className="sticky bottom-0 flex gap-3 px-5 py-4 border-t border-border bg-surface">
          <button
            type="button"
            onClick={onClose}
            className="flex-1 py-2.5 text-sm border border-border rounded-xl hover:bg-surface-hover"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={handleSave}
            disabled={saving}
            className="flex-1 py-2.5 text-sm bg-primary-600 text-white rounded-xl hover:bg-primary-700 disabled:opacity-50"
          >
            {saving ? "Saving…" : "Save"}
          </button>
        </div>
      </div>
    </div>
  );
}

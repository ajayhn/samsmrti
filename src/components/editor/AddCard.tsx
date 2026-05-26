import { useState, useEffect, useCallback } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { api, type NoteType, type Triple } from "../../lib/tauri";
import { useDeckStore } from "../../stores/deckStore";
import { useActivityTracker } from "../../hooks/useActivityTracker";
import { RichEditor } from "./RichEditor";
import { renderTemplate } from "../../lib/cloze";

export function AddCard() {
  useActivityTracker(true);
  const { deckId } = useParams<{ deckId: string }>();
  const navigate = useNavigate();
  const { fetchDecks } = useDeckStore();

  const [noteTypes, setNoteTypes] = useState<NoteType[]>([]);
  const [selectedType, setSelectedType] = useState<NoteType | null>(null);
  const [fields, setFields] = useState<Record<string, string>>({});
  const [fieldTexts, setFieldTexts] = useState<Record<string, string>>({});
  const [activeField, setActiveField] = useState<string>("");
  const [tags, setTags] = useState("");
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [showPreview, setShowPreview] = useState(false);
  const [showTripleLink, setShowTripleLink] = useState(false);
  const [tripleSearch, setTripleSearch] = useState("");
  const [availableTriples, setAvailableTriples] = useState<Triple[]>([]);
  const [linkedTripleIds, setLinkedTripleIds] = useState<Set<string>>(new Set());

  useEffect(() => {
    api.getNoteTypes().then((types) => {
      setNoteTypes(types);
      if (types.length > 0) {
        selectNoteType(types[0]);
      }
    });
  }, []);

  const searchTriples = useCallback(async (q: string) => {
    try {
      const triples = await api.getTriples();
      const filtered = q.trim()
        ? triples.filter(
            (t) =>
              t.subject_name.toLowerCase().includes(q.toLowerCase()) ||
              t.relation_name.toLowerCase().includes(q.toLowerCase()) ||
              t.object_name.toLowerCase().includes(q.toLowerCase())
          )
        : triples;
      setAvailableTriples(filtered.slice(0, 50));
    } catch {
      setAvailableTriples([]);
    }
  }, []);

  useEffect(() => {
    if (showTripleLink) {
      searchTriples(tripleSearch);
    }
  }, [showTripleLink, tripleSearch, searchTriples]);

  const toggleTripleLink = (tripleId: string) => {
    setLinkedTripleIds((prev) => {
      const next = new Set(prev);
      if (next.has(tripleId)) {
        next.delete(tripleId);
      } else {
        next.add(tripleId);
      }
      return next;
    });
  };

  const selectNoteType = (nt: NoteType) => {
    setSelectedType(nt);
    const initial: Record<string, string> = {};
    const initialTexts: Record<string, string> = {};
    nt.fields.forEach((f) => {
      initial[f.name] = "";
      initialTexts[f.name] = "";
    });
    setFields(initial);
    setFieldTexts(initialTexts);
    setActiveField(nt.fields[0]?.name || "");
  };

  const handleTypeChange = (typeId: string) => {
    const nt = noteTypes.find((t) => t.id === typeId);
    if (nt) selectNoteType(nt);
  };

  const handleFieldChange = (fieldName: string, html: string, text: string) => {
    setFields((prev) => ({ ...prev, [fieldName]: html }));
    setFieldTexts((prev) => ({ ...prev, [fieldName]: text }));
  };

  const handleSave = async () => {
    if (!deckId || !selectedType) return;
    const hasContent = Object.values(fieldTexts).some((v) => v.trim());
    if (!hasContent) return;

    setSaving(true);
    try {
      const fieldsToSend = selectedType.is_cloze ? fieldTexts : fields;
      const { note, karma } = await api.createNote({
        deck_id: deckId,
        note_type_id: selectedType.id,
        fields: fieldsToSend,
        tags: tags
          .split(",")
          .map((t) => t.trim())
          .filter(Boolean),
      });
      const { useKarmaStore } = await import("../../stores/karmaStore");
      useKarmaStore.getState().applyEarn(karma);

      if (linkedTripleIds.size > 0) {
        const cards = await api.getCardsForNote(note.id);
        if (cards.length > 0) {
          for (const tripleId of linkedTripleIds) {
            await api.linkCardToTriple(cards[0].id, tripleId);
          }
        }
      }

      await fetchDecks();

      const initial: Record<string, string> = {};
      const initialTexts: Record<string, string> = {};
      selectedType.fields.forEach((f) => {
        initial[f.name] = "";
        initialTexts[f.name] = "";
      });
      setFields(initial);
      setFieldTexts(initialTexts);
      setTags("");
      setLinkedTripleIds(new Set());
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } finally {
      setSaving(false);
    }
  };

  if (!selectedType) {
    return (
      <div className="flex items-center justify-center h-full text-text-muted">
        Loading...
      </div>
    );
  }

  const previewFields = selectedType.is_cloze ? fieldTexts : fields;
  const frontPreview =
    selectedType.templates[0]
      ? renderTemplate(
          selectedType.templates[0].front_html,
          previewFields,
          selectedType.is_cloze,
          0,
          false
        )
      : "";
  const backPreview =
    selectedType.templates[0]
      ? renderTemplate(
          selectedType.templates[0].back_html,
          previewFields,
          selectedType.is_cloze,
          0,
          true,
          selectedType.templates[0].front_html
        )
      : "";

  return (
    <div className="p-8 max-w-4xl mx-auto">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-xl font-bold text-text">Add Cards</h2>
        <button
          onClick={() => navigate("/")}
          className="text-sm text-text-secondary hover:text-text transition-colors"
        >
          Back to Deck
        </button>
      </div>

      {/* Note type selector */}
      <div className="mb-5">
        <label className="block text-sm font-medium text-text-secondary mb-2">
          Note Type
        </label>
        <select
          value={selectedType.id}
          onChange={(e) => handleTypeChange(e.target.value)}
          className="w-full px-4 py-2.5 bg-surface-alt border border-border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
        >
          {noteTypes.map((nt) => (
            <option key={nt.id} value={nt.id}>
              {nt.name}
            </option>
          ))}
        </select>
      </div>

      {/* Cloze hint */}
      {selectedType.is_cloze && (
        <div className="mb-4 px-4 py-3 bg-primary-50 dark:bg-primary-900/20 rounded-xl text-sm text-primary-700 dark:text-primary-300">
          Select text and press{" "}
          <kbd className="bg-primary-100 dark:bg-primary-900/40 px-1.5 py-0.5 rounded font-mono text-xs">
            Ctrl+Shift+C
          </kbd>{" "}
          to create a cloze deletion, or click the{" "}
          <span className="font-mono">[...]</span> button.
        </div>
      )}

      {/* Field tabs */}
      <div className="flex border-b border-border mb-0">
        {selectedType.fields.map((field) => (
          <button
            key={field.id}
            onClick={() => setActiveField(field.name)}
            className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors cursor-pointer ${
              activeField === field.name
                ? "border-primary-500 text-primary-600 dark:text-primary-400"
                : "border-transparent text-text-secondary hover:text-text"
            }`}
          >
            {field.name}
          </button>
        ))}
        <div className="flex-1" />
        <button
          onClick={() => setShowPreview(!showPreview)}
          className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors cursor-pointer ${
            showPreview
              ? "border-primary-500 text-primary-600"
              : "border-transparent text-text-secondary hover:text-text"
          }`}
        >
          Preview
        </button>
      </div>

      {/* Editor or preview */}
      <div className="mb-6">
        {showPreview ? (
          <div className="grid grid-cols-2 gap-4 mt-4">
            <div className="bg-surface-alt rounded-xl border border-border p-6">
              <p className="text-xs font-medium text-text-muted mb-3 uppercase tracking-wider">
                Front
              </p>
              <div
                className="prose prose-sm prose-stone dark:prose-invert max-w-none"
                dangerouslySetInnerHTML={{ __html: frontPreview || "<em class='text-text-muted'>Empty</em>" }}
              />
            </div>
            <div className="bg-surface-alt rounded-xl border border-border p-6">
              <p className="text-xs font-medium text-text-muted mb-3 uppercase tracking-wider">
                Back
              </p>
              <div
                className="prose prose-sm prose-stone dark:prose-invert max-w-none"
                dangerouslySetInnerHTML={{ __html: backPreview || "<em class='text-text-muted'>Empty</em>" }}
              />
            </div>
          </div>
        ) : (
          selectedType.fields.map((field) => (
            <div
              key={field.id}
              className={activeField === field.name ? "block mt-2" : "hidden"}
            >
              <RichEditor
                content={fields[field.name] || ""}
                onChange={(html, text) =>
                  handleFieldChange(field.name, html, text)
                }
                placeholder={
                  selectedType.is_cloze && field.ordinal === 0
                    ? "The mitochondria is the powerhouse of the cell (select text and Ctrl+Shift+C for cloze)"
                    : `Enter ${field.name.toLowerCase()}...`
                }
                isCloze={selectedType.is_cloze && field.ordinal === 0}
              />
            </div>
          ))
        )}
      </div>

      {/* Tags */}
      <div className="mb-6">
        <label className="block text-sm font-medium text-text-secondary mb-1.5">
          Tags
        </label>
        <input
          type="text"
          value={tags}
          onChange={(e) => setTags(e.target.value)}
          placeholder="Comma-separated tags (e.g., science::biology, exam)"
          className="w-full px-4 py-2.5 bg-surface-alt border border-border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
        />
      </div>

      {/* Link to Knowledge Graph */}
      <div className="mb-6">
        <button
          onClick={() => setShowTripleLink(!showTripleLink)}
          className="flex items-center gap-2 text-sm font-medium text-text-secondary hover:text-text transition-colors cursor-pointer"
        >
          <span className={`transition-transform ${showTripleLink ? "rotate-90" : ""}`}>&#9654;</span>
          Link to Knowledge Graph
          {linkedTripleIds.size > 0 && (
            <span className="text-xs bg-primary-100 dark:bg-primary-900/30 text-primary-600 px-2 py-0.5 rounded-full">
              {linkedTripleIds.size}
            </span>
          )}
        </button>

        {showTripleLink && (
          <div className="mt-3 border border-border rounded-xl p-3 bg-surface-alt">
            <input
              value={tripleSearch}
              onChange={(e) => setTripleSearch(e.target.value)}
              placeholder="Search triples..."
              className="w-full px-3 py-2 bg-surface border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500 mb-2"
            />
            <div className="max-h-40 overflow-y-auto space-y-1">
              {availableTriples.length === 0 && (
                <p className="text-xs text-text-muted text-center py-2">
                  No triples found. Create them on the Knowledge Map page.
                </p>
              )}
              {availableTriples.map((triple) => (
                <label
                  key={triple.id}
                  className="flex items-center gap-2 px-2 py-1.5 rounded hover:bg-surface-hover cursor-pointer text-sm"
                >
                  <input
                    type="checkbox"
                    checked={linkedTripleIds.has(triple.id)}
                    onChange={() => toggleTripleLink(triple.id)}
                    className="rounded border-border"
                  />
                  <span className="text-text">
                    {triple.subject_name}{" "}
                    <span className="text-text-muted">{triple.relation_name}</span>{" "}
                    {triple.object_name}
                  </span>
                </label>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Actions */}
      <div className="flex gap-3">
        <button
          onClick={handleSave}
          disabled={saving}
          className="flex-1 px-6 py-3 bg-primary-600 text-white rounded-xl font-medium hover:bg-primary-700 disabled:opacity-50 transition-colors"
        >
          {saving ? "Saving..." : saved ? "Saved!" : "Add Card"}
        </button>
      </div>
    </div>
  );
}

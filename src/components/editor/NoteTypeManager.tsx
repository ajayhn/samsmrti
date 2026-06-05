import { useState, useEffect, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { filterInputProps } from "../../lib/filterInput";
import { api, type NoteType, type NoteTypeUsageSummary } from "../../lib/tauri";

function formatCount(n: number): string {
  if (n >= 1_000_000) {
    const v = n / 1_000_000;
    return `${v % 1 === 0 ? v.toFixed(0) : v.toFixed(1)}M`;
  }
  if (n >= 1000) {
    const v = n / 1000;
    return `${v % 1 === 0 ? v.toFixed(0) : v.toFixed(1)}k`;
  }
  return String(n);
}

function NoteTypeUsageBar({
  usage,
  onBrowseAll,
  onBrowseDeck,
}: {
  usage: NoteTypeUsageSummary | undefined;
  onBrowseAll: () => void;
  onBrowseDeck: (deckId: string) => void;
}) {
  const [showDecks, setShowDecks] = useState(false);

  if (!usage || usage.note_count === 0) {
    return (
      <p className="text-xs text-text-muted mt-1">Not used by any notes yet</p>
    );
  }

  return (
    <div className="mt-1.5 space-y-1" onClick={(e) => e.stopPropagation()}>
      <div className="flex flex-wrap items-center gap-x-2 gap-y-0.5 text-xs text-text-muted">
        <span className="tabular-nums">
          {usage.deck_count} deck{usage.deck_count !== 1 ? "s" : ""}
          {" · "}
          {formatCount(usage.note_count)} notes
          {" · "}
          {formatCount(usage.card_count)} cards
        </span>
        <button
          type="button"
          onClick={onBrowseAll}
          className="text-primary-500 hover:underline font-medium cursor-pointer"
        >
          Browse
        </button>
        {usage.deck_count > 0 && (
          <button
            type="button"
            onClick={() => setShowDecks((v) => !v)}
            className="text-text-muted hover:text-text cursor-pointer"
          >
            {showDecks ? "Hide decks" : "Decks ▾"}
          </button>
        )}
      </div>
      {showDecks && usage.top_decks.length > 0 && (
        <div className="flex flex-wrap gap-1">
          {usage.top_decks.map((d) => (
            <button
              key={d.deck_id}
              type="button"
              onClick={() => onBrowseDeck(d.deck_id)}
              title={`${d.note_count} notes`}
              className="px-2 py-0.5 rounded-md bg-surface border border-border text-[11px] text-text-secondary hover:border-primary-400 hover:text-primary-600 cursor-pointer truncate max-w-[140px]"
            >
              {d.deck_name}
              <span className="text-text-muted ml-1 tabular-nums">{formatCount(d.note_count)}</span>
            </button>
          ))}
          {usage.deck_count > usage.top_decks.length && (
            <button
              type="button"
              onClick={onBrowseAll}
              className="px-2 py-0.5 text-[11px] text-primary-500 hover:underline cursor-pointer"
            >
              +{usage.deck_count - usage.top_decks.length} more
            </button>
          )}
        </div>
      )}
    </div>
  );
}

interface TemplateForm {
  name: string;
  front_html: string;
  back_html: string;
}

function NoteTypeForm({
  title,
  name,
  setName,
  isCloze,
  setIsCloze,
  fields,
  setFields,
  templates,
  setTemplates,
  saving,
  onSave,
  onCancel,
  saveLabel,
  showClozeToggle,
}: {
  title: string;
  name: string;
  setName: (v: string) => void;
  isCloze: boolean;
  setIsCloze?: (v: boolean) => void;
  fields: string[];
  setFields: (v: string[]) => void;
  templates: TemplateForm[];
  setTemplates: (v: TemplateForm[]) => void;
  saving: boolean;
  onSave: () => void;
  onCancel: () => void;
  saveLabel: string;
  showClozeToggle: boolean;
}) {
  const addField = () => setFields([...fields, `Field ${fields.length + 1}`]);
  const removeField = (i: number) => setFields(fields.filter((_, idx) => idx !== i));
  const updateField = (i: number, val: string) => {
    const next = [...fields];
    next[i] = val;
    setFields(next);
  };

  const addTemplate = () =>
    setTemplates([...templates, { name: `Card ${templates.length + 1}`, front_html: "", back_html: "" }]);
  const removeTemplate = (i: number) => setTemplates(templates.filter((_, idx) => idx !== i));
  const updateTemplate = (i: number, key: keyof TemplateForm, val: string) => {
    const next = [...templates];
    next[i] = { ...next[i], [key]: val };
    setTemplates(next);
  };

  return (
    <div className="bg-surface-alt rounded-xl border border-border p-6 space-y-5">
      <h3 className="font-semibold text-text">{title}</h3>

      <div>
        <label className="block text-sm font-medium text-text-secondary mb-1">Name</label>
        <input
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="e.g., Vocabulary"
          className="w-full px-3 py-2 bg-surface border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
        />
      </div>

      {showClozeToggle && setIsCloze && (
        <label className="flex items-center gap-2 text-sm text-text-secondary cursor-pointer">
          <input
            type="checkbox"
            checked={isCloze}
            onChange={(e) => setIsCloze(e.target.checked)}
            className="rounded"
          />
          Cloze note type
        </label>
      )}

      <div>
        <label className="block text-sm font-medium text-text-secondary mb-2">Fields</label>
        <div className="space-y-2">
          {fields.map((f, i) => (
            <div key={i} className="flex gap-2">
              <input
                type="text"
                value={f}
                onChange={(e) => updateField(i, e.target.value)}
                className="flex-1 px-3 py-2 bg-surface border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
              />
              {fields.length > 1 && (
                <button onClick={() => removeField(i)} className="px-2 text-danger text-sm hover:underline cursor-pointer">
                  Remove
                </button>
              )}
            </div>
          ))}
        </div>
        <button
          onClick={addField}
          className="mt-3 w-full py-2 bg-surface border border-dashed border-primary-400 rounded-lg text-sm text-primary-500 font-medium hover:bg-primary-50 dark:hover:bg-primary-900/20 transition-colors cursor-pointer"
        >
          + Add field
        </button>
      </div>

      <div>
        <label className="block text-sm font-medium text-text-secondary mb-2">Card Templates</label>
        <p className="text-xs text-text-muted mb-3">
          Use {"{{FieldName}}"} to reference fields. Use {"{{FrontSide}}"} on the back to include the front.
          {" "}Use {"{{each:FieldName}}"} for one card per comma-separated item.
          {" "}Use {"{{hint_suffix}}"} on “→ Country” cards: omitted for distinctive prompts (e.g. Shanghai), added for ambiguous ones (e.g. Volga).
          {isCloze && <> Use {"{{cloze:FieldName}}"} for cloze rendering.</>}
        </p>
        <div className="space-y-4">
          {templates.map((t, i) => (
            <div key={i} className="p-4 bg-surface rounded-lg border border-border space-y-3">
              <div className="flex items-center justify-between">
                <input
                  type="text"
                  value={t.name}
                  onChange={(e) => updateTemplate(i, "name", e.target.value)}
                  className="px-2 py-1 bg-transparent border-b border-border text-sm font-medium focus:outline-none focus:border-primary-500"
                />
                {templates.length > 1 && (
                  <button onClick={() => removeTemplate(i)} className="text-xs text-danger hover:underline cursor-pointer">
                    Remove
                  </button>
                )}
              </div>
              <div>
                <label className="text-xs text-text-muted">Front template</label>
                <textarea
                  value={t.front_html}
                  onChange={(e) => updateTemplate(i, "front_html", e.target.value)}
                  rows={2}
                  className="w-full mt-1 px-3 py-2 bg-surface-alt border border-border rounded-lg text-sm font-mono resize-none focus:outline-none focus:ring-2 focus:ring-primary-500"
                />
              </div>
              <div>
                <label className="text-xs text-text-muted">Back template</label>
                <textarea
                  value={t.back_html}
                  onChange={(e) => updateTemplate(i, "back_html", e.target.value)}
                  rows={2}
                  className="w-full mt-1 px-3 py-2 bg-surface-alt border border-border rounded-lg text-sm font-mono resize-none focus:outline-none focus:ring-2 focus:ring-primary-500"
                />
              </div>
            </div>
          ))}
        </div>
        <button
          onClick={addTemplate}
          className="mt-3 w-full py-2 bg-surface border border-dashed border-primary-400 rounded-lg text-sm text-primary-500 font-medium hover:bg-primary-50 dark:hover:bg-primary-900/20 transition-colors cursor-pointer"
        >
          + Add template
        </button>
      </div>

      <div className="flex gap-3 pt-2">
        <button
          onClick={onSave}
          disabled={saving || !name.trim()}
          className="px-6 py-2.5 bg-primary-600 text-white rounded-xl text-sm font-medium hover:bg-primary-700 disabled:opacity-50 transition-colors cursor-pointer"
        >
          {saving ? "Saving..." : saveLabel}
        </button>
        <button
          onClick={onCancel}
          className="px-4 py-2.5 text-text-secondary hover:bg-surface-hover rounded-xl text-sm transition-colors cursor-pointer"
        >
          Cancel
        </button>
      </div>
    </div>
  );
}

export function NoteTypeManager() {
  const navigate = useNavigate();
  const [noteTypes, setNoteTypes] = useState<NoteType[]>([]);
  const [usageById, setUsageById] = useState<Record<string, NoteTypeUsageSummary>>({});
  const [showCreate, setShowCreate] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);

  const [name, setName] = useState("");
  const [isCloze, setIsCloze] = useState(false);
  const [fields, setFields] = useState(["Front", "Back"]);
  const [templates, setTemplates] = useState<TemplateForm[]>([
    { name: "Card 1", front_html: "{{Front}}", back_html: "{{FrontSide}}<hr>{{Back}}" },
  ]);
  const [saving, setSaving] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");

  const filteredNoteTypes = useMemo(() => {
    const q = searchQuery.trim().toLowerCase();
    if (!q) return noteTypes;
    return noteTypes.filter((nt) => {
      if (editingId === nt.id) return true;
      if (nt.name.toLowerCase().includes(q)) return true;
      if (nt.fields.some((f) => f.name.toLowerCase().includes(q))) return true;
      if (nt.templates.some((t) => t.name.toLowerCase().includes(q))) return true;
      if (nt.is_cloze && "cloze".includes(q)) return true;
      return false;
    });
  }, [noteTypes, searchQuery, editingId]);

  const reload = async () => {
    const [types, usage] = await Promise.all([
      api.getNoteTypes(),
      api.getNoteTypeUsage(),
    ]);
    setNoteTypes(types);
    setUsageById(Object.fromEntries(usage.map((u) => [u.note_type_id, u])));
  };

  useEffect(() => {
    reload();
  }, []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (editingId) {
          setEditingId(null);
        } else if (showCreate) {
          setShowCreate(false);
          resetForm();
        } else {
          navigate("/");
        }
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [showCreate, editingId, navigate]);

  const resetForm = () => {
    setName("");
    setIsCloze(false);
    setFields(["Front", "Back"]);
    setTemplates([{ name: "Card 1", front_html: "{{Front}}", back_html: "{{FrontSide}}<hr>{{Back}}" }]);
  };

  const startEdit = (nt: NoteType) => {
    setEditingId(nt.id);
    setShowCreate(false);
    setName(nt.name);
    setIsCloze(nt.is_cloze);
    setFields(nt.fields.map((f) => f.name));
    setTemplates(
      nt.templates.map((t) => ({
        name: t.name,
        front_html: t.front_html,
        back_html: t.back_html,
      }))
    );
  };

  const handleCreate = async () => {
    if (!name.trim() || fields.length === 0 || templates.length === 0) return;
    setSaving(true);
    try {
      await api.createNoteType({
        name: name.trim(),
        is_cloze: isCloze,
        fields: fields.filter((f) => f.trim()),
        templates,
      });
      await reload();
      setShowCreate(false);
      resetForm();
    } finally {
      setSaving(false);
    }
  };

  const handleUpdate = async () => {
    if (!editingId || !name.trim()) return;
    setSaving(true);
    try {
      await api.updateNoteType({
        id: editingId,
        name: name.trim(),
        fields: fields.filter((f) => f.trim()),
        templates,
      });
      await reload();
      setEditingId(null);
      resetForm();
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await api.deleteNoteType(id);
      setNoteTypes(noteTypes.filter((nt) => nt.id !== id));
    } catch (e) {
      alert(String(e));
    }
  };

  const isBuiltin = (id: string) =>
    id.startsWith("nt_basic") || id === "nt_cloze";

  const browseNoteType = (nt: NoteType, deckId?: string) => {
    const params = new URLSearchParams({
      noteTypeId: nt.id,
      noteTypeName: nt.name,
    });
    if (deckId) params.set("deckId", deckId);
    navigate(`/browse?${params.toString()}`);
  };

  return (
    <div className="p-8 max-w-3xl mx-auto">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-xl font-bold text-text">Note Types</h2>
        <button
          onClick={() => navigate("/")}
          className="text-sm text-text-secondary hover:text-text transition-colors cursor-pointer"
        >
          Back
        </button>
      </div>

      {noteTypes.length > 0 && (
        <div className="mb-4">
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search note types by name, field, or template…"
            className="w-full px-3 py-2.5 text-sm bg-surface-alt border border-border rounded-xl text-text placeholder:text-text-muted focus:outline-none focus:ring-2 focus:ring-primary-500"
            aria-label="Search note types"
            name="samsmrti-note-type-search"
            {...filterInputProps}
          />
          {searchQuery.trim() && (
            <p className="text-xs text-text-muted mt-1.5">
              {filteredNoteTypes.length} of {noteTypes.length} note type
              {noteTypes.length !== 1 ? "s" : ""}
            </p>
          )}
        </div>
      )}

      <div className="space-y-3 mb-8">
        {filteredNoteTypes.length === 0 && searchQuery.trim() ? (
          <p className="text-sm text-text-muted text-center py-8">
            No note types match &ldquo;{searchQuery.trim()}&rdquo;
          </p>
        ) : null}
        {filteredNoteTypes.map((nt) =>
          editingId === nt.id ? (
            <NoteTypeForm
              key={nt.id}
              title={`Edit: ${nt.name}`}
              name={name}
              setName={setName}
              isCloze={isCloze}
              fields={fields}
              setFields={setFields}
              templates={templates}
              setTemplates={setTemplates}
              saving={saving}
              onSave={handleUpdate}
              onCancel={() => { setEditingId(null); resetForm(); }}
              saveLabel="Save Changes"
              showClozeToggle={false}
            />
          ) : (
            <div
              key={nt.id}
              className="p-4 bg-surface-alt rounded-xl border border-border hover:border-primary-400/50 transition-colors"
            >
              <div className="flex items-start justify-between gap-3">
                <button
                  type="button"
                  className="min-w-0 flex-1 text-left cursor-pointer"
                  onClick={() => startEdit(nt)}
                >
                  <p className="font-medium text-text">{nt.name}</p>
                  <p className="text-xs text-text-muted mt-0.5">
                    {nt.fields.map((f) => f.name).join(", ")}
                    {nt.is_cloze && " (Cloze)"}
                    {" · "}
                    {nt.templates.length} template{nt.templates.length !== 1 ? "s" : ""}
                  </p>
                  <NoteTypeUsageBar
                    usage={usageById[nt.id]}
                    onBrowseAll={() => browseNoteType(nt)}
                    onBrowseDeck={(deckId) => browseNoteType(nt, deckId)}
                  />
                </button>
                <div className="flex shrink-0 items-center gap-2">
                  <button
                    type="button"
                    onClick={() => startEdit(nt)}
                    className="text-xs text-text-secondary hover:text-text cursor-pointer"
                  >
                    Edit
                  </button>
                  {!isBuiltin(nt.id) && (
                    <button
                      type="button"
                      onClick={() => handleDelete(nt.id)}
                      className="text-xs text-danger hover:underline cursor-pointer"
                    >
                      Delete
                    </button>
                  )}
                </div>
              </div>
            </div>
          )
        )}
      </div>

      {!showCreate && !editingId ? (
        <button
          onClick={() => { resetForm(); setShowCreate(true); }}
          className="w-full py-3 bg-surface-alt border border-dashed border-border rounded-xl text-text-secondary hover:bg-surface-hover transition-colors text-sm font-medium cursor-pointer"
        >
          + Create Custom Note Type
        </button>
      ) : showCreate ? (
        <NoteTypeForm
          title="New Note Type"
          name={name}
          setName={setName}
          isCloze={isCloze}
          setIsCloze={setIsCloze}
          fields={fields}
          setFields={setFields}
          templates={templates}
          setTemplates={setTemplates}
          saving={saving}
          onSave={handleCreate}
          onCancel={() => { setShowCreate(false); resetForm(); }}
          saveLabel="Create Note Type"
          showClozeToggle={true}
        />
      ) : null}
    </div>
  );
}

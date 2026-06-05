import { useState, useRef, useEffect, useCallback } from "react";
import { filterInputProps } from "../../lib/filterInput";
import { api, type Entity, type RelationType } from "../../lib/tauri";

interface Props {
  onTripleCreated: () => void;
  focusedEntityId?: string | null;
}

interface AutocompleteOption {
  id: string;
  label: string;
  isNew?: boolean;
}

function useAutocomplete(
  fetchFn: (query: string) => Promise<AutocompleteOption[]>,
  onSelect: (option: AutocompleteOption) => void,
  clearAfterSelect = false
) {
  const [query, setQuery] = useState("");
  const [options, setOptions] = useState<AutocompleteOption[]>([]);
  const [open, setOpen] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const timerRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const inputRef = useRef<HTMLInputElement>(null);

  const search = useCallback(
    (q: string) => {
      if (timerRef.current) clearTimeout(timerRef.current);
      timerRef.current = setTimeout(async () => {
        const results = await fetchFn(q);
        if (q.trim() && !results.some((r) => r.label.toLowerCase() === q.trim().toLowerCase())) {
          results.push({ id: "__new__", label: q.trim(), isNew: true });
        }
        setOptions(results);
        setSelectedIndex(0);
        setOpen(results.length > 0);
      }, 150);
    },
    [fetchFn]
  );

  const handleChange = (value: string) => {
    setQuery(value);
    if (value.trim()) {
      search(value);
    } else {
      setOptions([]);
      setOpen(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (!open) return;
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIndex((i) => Math.min(i + 1, options.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIndex((i) => Math.max(i - 1, 0));
    } else if (e.key === "Enter" && options.length > 0) {
      e.preventDefault();
      const selected = options[selectedIndex];
      if (selected) {
        onSelect(selected);
        setQuery(clearAfterSelect ? "" : selected.label);
        setOpen(false);
      }
    } else if (e.key === "Escape") {
      setOpen(false);
    }
  };

  const selectOption = (opt: AutocompleteOption) => {
    onSelect(opt);
    setQuery(clearAfterSelect ? "" : opt.label);
    setOpen(false);
  };

  return { query, setQuery, options, open, setOpen, selectedIndex, handleChange, handleKeyDown, selectOption, inputRef };
}

interface ObjectEntry {
  id: string;
  label: string;
}

interface PendingNewEntity {
  name: string;
  target: "subject" | "object";
}

export function TripleEntry({ onTripleCreated, focusedEntityId }: Props) {
  const [subjectId, setSubjectId] = useState<string | null>(null);
  const [relationId, setRelationId] = useState<string | null>(null);
  const [objects, setObjects] = useState<ObjectEntry[]>([]);
  const [saving, setSaving] = useState(false);
  const [pendingNew, setPendingNew] = useState<PendingNewEntity | null>(null);
  const [pendingType, setPendingType] = useState("");

  const objectInputRef = useRef<HTMLInputElement>(null);

  const fetchEntities = useCallback(async (q: string): Promise<AutocompleteOption[]> => {
    if (!q.trim()) return [];
    const entities = await api.getEntities(q);
    return entities.map((e: Entity) => ({
      id: e.id,
      label: e.entity_type ? `${e.name} (${e.entity_type})` : e.name,
    }));
  }, []);

  const fetchRelations = useCallback(async (q: string): Promise<AutocompleteOption[]> => {
    const types = await api.getRelationTypes();
    const filtered = q.trim()
      ? types.filter((r: RelationType) => r.name.toLowerCase().includes(q.toLowerCase()))
      : types;
    return filtered.map((r: RelationType) => ({
      id: r.id,
      label: r.inverse_name ? `${r.name} / ${r.inverse_name}` : r.name,
    }));
  }, []);

  const subject = useAutocomplete(fetchEntities, async (opt) => {
    if (opt.isNew) {
      setPendingNew({ name: opt.label, target: "subject" });
      setPendingType("");
    } else {
      setSubjectId(opt.id);
    }
  });

  const relation = useAutocomplete(fetchRelations, async (opt) => {
    if (opt.isNew) {
      const rt = await api.createRelationType({ name: opt.label });
      setRelationId(rt.id);
    } else {
      setRelationId(opt.id);
    }
  });

  const objectAc = useAutocomplete(fetchEntities, async (opt) => {
    if (opt.isNew) {
      setPendingNew({ name: opt.label, target: "object" });
      setPendingType("");
    } else {
      setObjects((prev) => [...prev, { id: opt.id, label: opt.label }]);
    }
  }, true);

  const confirmNewEntity = async (typeOverride?: string) => {
    if (!pendingNew) return;
    const target = pendingNew.target;
    const entityType = typeOverride !== undefined ? typeOverride : pendingType.trim();
    const entity = await api.createEntity({
      name: pendingNew.name,
      entity_type: entityType || undefined,
    });
    const displayLabel = entityType
      ? `${pendingNew.name} (${entityType})`
      : pendingNew.name;

    if (target === "subject") {
      setSubjectId(entity.id);
      subject.setQuery(displayLabel);
    } else {
      setObjects((prev) => [...prev, { id: entity.id, label: displayLabel }]);
    }
    setPendingNew(null);
    setPendingType("");
    setTimeout(() => {
      if (target === "object") {
        objectInputRef.current?.focus();
      }
    }, 50);
  };

  const cancelNewEntity = () => {
    const target = pendingNew?.target;
    if (target === "subject") {
      subject.setQuery("");
    }
    setPendingNew(null);
    setPendingType("");
    setTimeout(() => {
      if (target === "object") {
        objectInputRef.current?.focus();
      }
    }, 50);
  };

  useEffect(() => {
    if (focusedEntityId) {
      (async () => {
        const entities = await api.getEntities();
        const ent = entities.find((e: Entity) => e.id === focusedEntityId);
        if (ent) {
          setSubjectId(ent.id);
          subject.setQuery(ent.entity_type ? `${ent.name} (${ent.entity_type})` : ent.name);
        }
      })();
    }
  }, [focusedEntityId]);

  const handleSubmit = async () => {
    if (!subjectId || !relationId || objects.length === 0) return;
    setSaving(true);
    try {
      const objectIds = objects.map((o) => o.id);
      if (objectIds.length === 1) {
        await api.createTriple({
          subject_id: subjectId,
          relation_type_id: relationId,
          object_id: objectIds[0],
        });
      } else {
        await api.batchCreateTriples({
          subject_id: subjectId,
          relation_type_id: relationId,
          object_ids: objectIds,
        });
      }
      subject.setQuery("");
      relation.setQuery("");
      objectAc.setQuery("");
      setSubjectId(null);
      setRelationId(null);
      setObjects([]);
      onTripleCreated();
      subject.inputRef.current?.focus();
    } finally {
      setSaving(false);
    }
  };

  const removeObject = (idx: number) => {
    setObjects((prev) => prev.filter((_, i) => i !== idx));
  };

  const handleKeyDownGlobal = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !subject.open && !relation.open && !objectAc.open) {
      e.preventDefault();
      handleSubmit();
    }
  };

  return (
    <div
      className="flex items-center gap-2 px-4 py-3 bg-surface-alt border-t border-border"
      onKeyDown={handleKeyDownGlobal}
    >
      {/* Subject */}
      <div className="relative flex-1 min-w-0">
        <input
          ref={subject.inputRef}
          value={subject.query}
          onChange={(e) => subject.handleChange(e.target.value)}
          onKeyDown={subject.handleKeyDown}
          onFocus={() => subject.query && subject.handleChange(subject.query)}
          onBlur={() => setTimeout(() => subject.setOpen(false), 200)}
          placeholder="Subject entity..."
          className="w-full px-3 py-2 bg-surface border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
          {...filterInputProps}
        />
        {subject.open && (
          <div className="absolute bottom-full mb-1 left-0 right-0 bg-surface border border-border rounded-lg shadow-lg max-h-48 overflow-y-auto z-50">
            {subject.options.map((opt, i) => (
              <button
                key={opt.id}
                onMouseDown={() => subject.selectOption(opt)}
                className={`w-full text-left px-3 py-2 text-sm cursor-pointer ${
                  i === subject.selectedIndex ? "bg-primary-100 dark:bg-primary-900/30" : "hover:bg-surface-hover"
                } ${opt.isNew ? "text-primary-600 italic" : "text-text"}`}
              >
                {opt.isNew ? `+ Create "${opt.label}"` : opt.label}
              </button>
            ))}
          </div>
        )}
      </div>

      {/* Relation */}
      <div className="relative flex-1 min-w-0">
        <input
          ref={relation.inputRef}
          value={relation.query}
          onChange={(e) => relation.handleChange(e.target.value)}
          onKeyDown={relation.handleKeyDown}
          onFocus={() => relation.handleChange(relation.query)}
          onBlur={() => setTimeout(() => relation.setOpen(false), 200)}
          placeholder="Relation..."
          className="w-full px-3 py-2 bg-surface border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
          {...filterInputProps}
        />
        {relation.open && (
          <div className="absolute bottom-full mb-1 left-0 right-0 bg-surface border border-border rounded-lg shadow-lg max-h-48 overflow-y-auto z-50">
            {relation.options.map((opt, i) => (
              <button
                key={opt.id}
                onMouseDown={() => relation.selectOption(opt)}
                className={`w-full text-left px-3 py-2 text-sm cursor-pointer ${
                  i === relation.selectedIndex ? "bg-primary-100 dark:bg-primary-900/30" : "hover:bg-surface-hover"
                } ${opt.isNew ? "text-primary-600 italic" : "text-text"}`}
              >
                {opt.isNew ? `+ Create "${opt.label}"` : opt.label}
              </button>
            ))}
          </div>
        )}
      </div>

      {/* Objects */}
      <div className="relative flex-[2] min-w-0">
        <div className="flex flex-wrap items-center gap-1 px-3 py-1.5 bg-surface border border-border rounded-lg min-h-[38px]">
          {objects.map((obj, i) => (
            <span
              key={i}
              className="inline-flex items-center gap-1 bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-300 text-xs px-2 py-0.5 rounded"
            >
              {obj.label}
              <button
                onClick={() => removeObject(i)}
                className="hover:text-red-500 cursor-pointer"
              >
                &times;
              </button>
            </span>
          ))}
          <input
            ref={objectInputRef}
            value={objectAc.query}
            onChange={(e) => objectAc.handleChange(e.target.value)}
            onKeyDown={objectAc.handleKeyDown}
            onFocus={() => objectAc.query && objectAc.handleChange(objectAc.query)}
            onBlur={() => setTimeout(() => objectAc.setOpen(false), 200)}
            placeholder={objects.length > 0 ? "Add more..." : "Object entities..."}
            className="flex-1 min-w-[100px] py-0.5 bg-transparent text-sm focus:outline-none"
            {...filterInputProps}
          />
        </div>
        {objectAc.open && (
          <div className="absolute bottom-full mb-1 left-0 right-0 bg-surface border border-border rounded-lg shadow-lg max-h-48 overflow-y-auto z-50">
            {objectAc.options.map((opt, i) => (
              <button
                key={opt.id}
                onMouseDown={() => objectAc.selectOption(opt)}
                className={`w-full text-left px-3 py-2 text-sm cursor-pointer ${
                  i === objectAc.selectedIndex ? "bg-primary-100 dark:bg-primary-900/30" : "hover:bg-surface-hover"
                } ${opt.isNew ? "text-primary-600 italic" : "text-text"}`}
              >
                {opt.isNew ? `+ Create "${opt.label}"` : opt.label}
              </button>
            ))}
          </div>
        )}
      </div>

      <button
        onClick={handleSubmit}
        disabled={saving || !subjectId || !relationId || objects.length === 0}
        className={`px-4 py-2 rounded-lg text-sm font-medium transition-all shrink-0 ${
          saving || !subjectId || !relationId || objects.length === 0
            ? "bg-gray-600/40 text-gray-400 cursor-not-allowed"
            : "bg-primary-600 text-white hover:bg-primary-500 cursor-pointer shadow-md shadow-primary-600/30"
        }`}
      >
        {saving ? "..." : "Add"}
      </button>

      {pendingNew && (
        <div className="fixed inset-0 bg-black/40 flex items-center justify-center z-50">
          <div className="bg-surface rounded-xl shadow-2xl w-full max-w-xs p-5">
            <p className="text-sm font-medium text-text mb-1">
              New entity: <span className="text-primary-600">{pendingNew.name}</span>
            </p>
            <p className="text-xs text-text-muted mb-3">
              Optionally specify its type, then press Enter or click Create.
            </p>
            <input
              value={pendingType}
              onChange={(e) => setPendingType(e.target.value)}
              placeholder="Type (e.g. Country, City, River...)"
              className="w-full px-3 py-2 bg-surface-alt border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
              autoFocus
              onKeyDown={(e) => {
                if (e.key === "Enter") confirmNewEntity();
                if (e.key === "Escape") cancelNewEntity();
              }}
            />
            <div className="flex gap-2 mt-3">
              <button
                onClick={() => confirmNewEntity()}
                className="flex-1 px-3 py-1.5 bg-primary-600 text-white text-xs font-medium rounded-lg hover:bg-primary-700 transition-colors cursor-pointer"
              >
                Create
              </button>
              <button
                onClick={() => confirmNewEntity("")}
                className="px-3 py-1.5 text-xs text-text-secondary hover:text-text transition-colors cursor-pointer"
              >
                No type
              </button>
              <button
                onClick={cancelNewEntity}
                className="px-3 py-1.5 text-xs text-text-muted hover:text-danger transition-colors cursor-pointer"
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

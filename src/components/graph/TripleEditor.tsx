import { useState, useEffect, useCallback } from "react";
import { api, type Triple, type Entity, type RelationType, type MindMapNode } from "../../lib/tauri";

interface Props {
  rootEntityId: string;
  focusedNode: MindMapNode;
  onClose: () => void;
  onChanged: () => void;
}

export function TripleEditor({ rootEntityId, focusedNode, onClose, onChanged }: Props) {
  const [triples, setTriples] = useState<Triple[]>([]);
  const [loading, setLoading] = useState(true);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [entities, setEntities] = useState<Entity[]>([]);
  const [relationTypes, setRelationTypes] = useState<RelationType[]>([]);
  const [editSubject, setEditSubject] = useState("");
  const [editRelation, setEditRelation] = useState("");
  const [editObject, setEditObject] = useState("");

  const loadTriples = useCallback(async () => {
    setLoading(true);
    try {
      let results: Triple[];
      if (focusedNode.node_type === "relation") {
        const allForRoot = await api.getTriples(undefined, undefined, undefined, rootEntityId);
        const relLabel = focusedNode.label.replace(/ \(\d+\)$/, "");
        results = allForRoot.filter((t) => t.relation_name === relLabel);
      } else {
        const entityId = focusedNode.node_type === "root" ? rootEntityId : focusedNode.id;
        const asSubject = await api.getTriples(entityId);
        const asObject = await api.getTriples(undefined, entityId);
        const seen = new Set<string>();
        results = [];
        for (const t of [...asSubject, ...asObject]) {
          if (!seen.has(t.id)) {
            seen.add(t.id);
            results.push(t);
          }
        }
      }
      setTriples(results);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  }, [rootEntityId, focusedNode]);

  useEffect(() => {
    loadTriples();
  }, [loadTriples]);

  const loadPickerData = useCallback(async () => {
    try {
      const [ents, rels] = await Promise.all([api.getEntities(), api.getRelationTypes()]);
      setEntities(ents);
      setRelationTypes(rels);
    } catch (e) {
      console.error(e);
    }
  }, []);

  const startEdit = (triple: Triple) => {
    setEditingId(triple.id);
    setEditSubject(triple.subject_id);
    setEditRelation(triple.relation_type_id);
    setEditObject(triple.object_id);
    loadPickerData();
  };

  const saveEdit = async () => {
    if (!editingId) return;
    try {
      await api.updateTriple({
        id: editingId,
        subject_id: editSubject,
        relation_type_id: editRelation,
        object_id: editObject,
      });
      setEditingId(null);
      await loadTriples();
      onChanged();
    } catch (e) {
      console.error(e);
    }
  };

  const handleDelete = async (tripleId: string) => {
    try {
      await api.deleteTriple(tripleId);
      await loadTriples();
      onChanged();
    } catch (e) {
      console.error(e);
    }
  };

  const title =
    focusedNode.node_type === "relation"
      ? `Triples: ${focusedNode.label}`
      : `Triples for: ${focusedNode.label}`;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div
        className="bg-surface rounded-2xl shadow-2xl w-full max-w-lg max-h-[80vh] flex flex-col"
        onKeyDown={(e) => { if (e.key === "Escape") onClose(); }}
      >
        <div className="flex items-center justify-between px-6 py-4 border-b border-border">
          <h3 className="text-lg font-bold text-text">{title}</h3>
          <button
            onClick={onClose}
            className="text-text-muted hover:text-text text-lg cursor-pointer"
          >
            &times;
          </button>
        </div>

        <div className="flex-1 overflow-y-auto px-6 py-4 space-y-2">
          {loading && (
            <p className="text-text-muted text-sm text-center py-4">Loading...</p>
          )}
          {!loading && triples.length === 0 && (
            <p className="text-text-muted text-sm text-center py-4">No triples found.</p>
          )}
          {triples.map((triple) => (
            <div
              key={triple.id}
              className="border border-border rounded-lg p-3 bg-surface-alt"
            >
              {editingId === triple.id ? (
                <div className="space-y-2">
                  <div className="grid grid-cols-3 gap-2">
                    <div>
                      <label className="block text-[10px] font-medium text-text-muted mb-0.5">Subject</label>
                      <select
                        value={editSubject}
                        onChange={(e) => setEditSubject(e.target.value)}
                        className="w-full px-2 py-1.5 bg-surface border border-border rounded text-xs focus:outline-none focus:ring-2 focus:ring-primary-500"
                      >
                        {entities.map((ent) => (
                          <option key={ent.id} value={ent.id}>
                            {ent.name}{ent.entity_type ? ` (${ent.entity_type})` : ""}
                          </option>
                        ))}
                      </select>
                    </div>
                    <div>
                      <label className="block text-[10px] font-medium text-text-muted mb-0.5">Relation</label>
                      <select
                        value={editRelation}
                        onChange={(e) => setEditRelation(e.target.value)}
                        className="w-full px-2 py-1.5 bg-surface border border-border rounded text-xs focus:outline-none focus:ring-2 focus:ring-primary-500"
                      >
                        {relationTypes.map((rt) => (
                          <option key={rt.id} value={rt.id}>
                            {rt.name}
                          </option>
                        ))}
                      </select>
                    </div>
                    <div>
                      <label className="block text-[10px] font-medium text-text-muted mb-0.5">Object</label>
                      <select
                        value={editObject}
                        onChange={(e) => setEditObject(e.target.value)}
                        className="w-full px-2 py-1.5 bg-surface border border-border rounded text-xs focus:outline-none focus:ring-2 focus:ring-primary-500"
                      >
                        {entities.map((ent) => (
                          <option key={ent.id} value={ent.id}>
                            {ent.name}{ent.entity_type ? ` (${ent.entity_type})` : ""}
                          </option>
                        ))}
                      </select>
                    </div>
                  </div>
                  <div className="flex gap-2 justify-end">
                    <button
                      onClick={saveEdit}
                      className="px-3 py-1 bg-primary-600 text-white text-xs rounded hover:bg-primary-700 cursor-pointer"
                    >
                      Save
                    </button>
                    <button
                      onClick={() => setEditingId(null)}
                      className="px-3 py-1 text-xs text-text-secondary hover:text-text cursor-pointer"
                    >
                      Cancel
                    </button>
                  </div>
                </div>
              ) : (
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-1.5 text-sm min-w-0">
                    <span className="font-medium text-text truncate">{triple.subject_name}</span>
                    <span className="text-primary-500 italic shrink-0">{triple.relation_name}</span>
                    <span className="text-text-secondary truncate">{triple.object_name}</span>
                  </div>
                  <div className="flex items-center gap-1 shrink-0 ml-2">
                    <button
                      onClick={() => startEdit(triple)}
                      className="text-text-muted hover:text-primary-500 text-xs px-1.5 py-0.5 rounded hover:bg-surface-hover cursor-pointer"
                      title="Edit triple"
                    >
                      Edit
                    </button>
                    <button
                      onClick={() => handleDelete(triple.id)}
                      className="text-text-muted hover:text-danger text-xs px-1.5 py-0.5 rounded hover:bg-surface-hover cursor-pointer"
                      title="Delete triple"
                    >
                      Delete
                    </button>
                  </div>
                </div>
              )}
            </div>
          ))}
        </div>

        <div className="px-6 py-3 border-t border-border flex justify-end">
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm text-text-secondary hover:text-text cursor-pointer"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
}

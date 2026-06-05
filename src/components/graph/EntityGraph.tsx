import { useState, useEffect, useCallback, useRef } from "react";
import { filterInputProps } from "../../lib/filterInput";
import { api, type Entity, type MindMapData, type MindMapNode } from "../../lib/tauri";
import { useDeckStore } from "../../stores/deckStore";
import { isNativeEditShortcut } from "../../lib/keyboard";
import { MindMap } from "./MindMap";
import { TripleEntry } from "./TripleEntry";
import { SuggestCards } from "./SuggestCards";
import { EreReview } from "./EreReview";
import { TripleEditor } from "./TripleEditor";

export function EntityGraph() {
  const [entities, setEntities] = useState<Entity[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const [typeFilter, setTypeFilter] = useState("");
  const [selectedEntityId, setSelectedEntityId] = useState<string | null>(null);
  const [mindmapData, setMindmapData] = useState<MindMapData | null>(null);
  const [loading, setLoading] = useState(false);
  const [dueCount, setDueCount] = useState(0);
  const [showSuggest, setShowSuggest] = useState(false);
  const [showEreReview, setShowEreReview] = useState(false);
  const [showNewEntity, setShowNewEntity] = useState(false);
  const [newEntityName, setNewEntityName] = useState("");
  const [newEntityType, setNewEntityType] = useState("");
  const [editingEntity, setEditingEntity] = useState<Entity | null>(null);
  const [tripleEditorNode, setTripleEditorNode] = useState<MindMapNode | null>(null);
  const [editName, setEditName] = useState("");
  const [editType, setEditType] = useState("");
  const [editDesc, setEditDesc] = useState("");
  const { selectedDeckId } = useDeckStore();
  const tripleEntryRef = useRef<HTMLDivElement>(null);

  const loadEntities = useCallback(async () => {
    try {
      const results = await api.getEntities(
        searchQuery || undefined,
        typeFilter || undefined
      );
      setEntities(results);
    } catch (e) {
      console.error(e);
    }
  }, [searchQuery, typeFilter]);

  useEffect(() => {
    loadEntities();
  }, [loadEntities]);

  const loadMindmap = useCallback(async (entityId: string) => {
    setLoading(true);
    try {
      const data = await api.getMindmap(entityId);
      setMindmapData(data);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  }, []);

  const loadDueCount = useCallback(async () => {
    if (!selectedDeckId) {
      setDueCount(0);
      return;
    }
    try {
      const cards = await api.getEreDueCards(
        selectedDeckId,
        selectedEntityId || undefined
      );
      setDueCount(cards.length);
    } catch {
      setDueCount(0);
    }
  }, [selectedDeckId, selectedEntityId]);

  useEffect(() => {
    loadDueCount();
  }, [loadDueCount]);

  const selectEntity = (entityId: string) => {
    setSelectedEntityId(entityId);
    loadMindmap(entityId);
  };

  const handleEntityClickInMindmap = (entityId: string) => {
    setSelectedEntityId(entityId);
    loadMindmap(entityId);
  };

  const handleNodeDoubleClick = (node: MindMapNode) => {
    setTripleEditorNode(node);
  };

  const handleTripleCreated = () => {
    loadEntities();
    if (selectedEntityId) {
      loadMindmap(selectedEntityId);
    }
    loadDueCount();
  };

  const handleCreateEntity = async () => {
    if (!newEntityName.trim()) return;
    try {
      const entity = await api.createEntity({
        name: newEntityName.trim(),
        entity_type: newEntityType.trim() || undefined,
      });
      setNewEntityName("");
      setNewEntityType("");
      setShowNewEntity(false);
      loadEntities();
      selectEntity(entity.id);
    } catch (e) {
      console.error(e);
    }
  };

  const startEditEntity = (entity: Entity) => {
    setEditingEntity(entity);
    setEditName(entity.name);
    setEditType(entity.entity_type || "");
    setEditDesc(entity.description || "");
  };

  const handleSaveEntity = async () => {
    if (!editingEntity || !editName.trim()) return;
    try {
      await api.updateEntity({
        id: editingEntity.id,
        name: editName.trim(),
        entity_type: editType.trim() || undefined,
        description: editDesc.trim() || undefined,
      });
      setEditingEntity(null);
      loadEntities();
      if (selectedEntityId === editingEntity.id) {
        loadMindmap(editingEntity.id);
      }
    } catch (e) {
      console.error(e);
    }
  };

  const handleDeleteEntity = async (id: string) => {
    try {
      await api.deleteEntity(id);
      if (selectedEntityId === id) {
        setSelectedEntityId(null);
        setMindmapData(null);
      }
      loadEntities();
    } catch (e) {
      console.error(e);
    }
  };

  const entityTypes = [...new Set(entities.map((e) => e.entity_type).filter(Boolean))] as string[];

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (isNativeEditShortcut(e)) return;
      const tag = document.activeElement?.tagName;
      const isInput = tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT" ||
        (document.activeElement as HTMLElement)?.isContentEditable;
      if (isInput) return;

      if (e.key === "t" || e.key === "T") {
        e.preventDefault();
        tripleEntryRef.current?.querySelector("input")?.focus();
      } else if (e.key === "r" || e.key === "R") {
        if (dueCount > 0 && selectedDeckId) {
          e.preventDefault();
          setShowEreReview(true);
        }
      }
    },
    [dueCount, selectedDeckId]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  return (
    <div className="h-full flex flex-col">
      {/* Toolbar */}
      <div className="flex items-center justify-between px-6 py-3 border-b border-border bg-surface">
        <h2 className="text-xl font-bold text-text">Knowledge Map</h2>
        <div className="flex items-center gap-3">
          {dueCount > 0 && selectedDeckId && (
            <button
              onClick={() => setShowEreReview(true)}
              className="px-3 py-1.5 bg-warning/10 border border-warning/30 text-warning text-sm font-medium rounded-lg hover:bg-warning/20 transition-colors cursor-pointer"
            >
              Due: {dueCount}
              <kbd className="ml-1.5 text-[10px] font-semibold bg-warning/20 px-1 py-0.5 rounded">
                R
              </kbd>
            </button>
          )}
          <button
            onClick={() => setShowSuggest(true)}
            className="px-3 py-1.5 bg-surface-alt border border-border text-text-secondary text-sm rounded-lg hover:bg-surface-hover transition-colors cursor-pointer"
          >
            Suggest Cards
          </button>
        </div>
      </div>

      {/* Main content */}
      <div className="flex-1 flex min-h-0">
        {/* Entity sidebar */}
        <div className="w-64 border-r border-border bg-surface-alt flex flex-col shrink-0">
          <div className="p-3 space-y-2 border-b border-border">
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search entities..."
              className="w-full px-3 py-2 bg-surface border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
              name="samsmrti-entity-search"
              {...filterInputProps}
            />
            {entityTypes.length > 0 && (
              <select
                value={typeFilter}
                onChange={(e) => setTypeFilter(e.target.value)}
                className="w-full px-3 py-1.5 bg-surface border border-border rounded-lg text-xs focus:outline-none focus:ring-2 focus:ring-primary-500"
              >
                <option value="">All types</option>
                {entityTypes.map((t) => (
                  <option key={t} value={t}>
                    {t}
                  </option>
                ))}
              </select>
            )}
          </div>

          <div className="flex-1 overflow-y-auto p-2 space-y-0.5">
            {entities.length === 0 && (
              <p className="text-xs text-text-muted text-center py-4">
                No entities yet. Create one below or use the triple entry bar.
              </p>
            )}
            {entities.map((entity) => (
              <div key={entity.id} className="group flex items-center">
                <button
                  onClick={() => selectEntity(entity.id)}
                  className={`flex-1 text-left px-3 py-2 rounded-lg text-sm transition-colors cursor-pointer ${
                    selectedEntityId === entity.id
                      ? "bg-primary-100 text-primary-700 dark:bg-primary-900/30 dark:text-primary-300"
                      : "text-text hover:bg-surface-hover"
                  }`}
                >
                  <span className="font-medium truncate block">{entity.name}</span>
                  {entity.entity_type && (
                    <span className="text-[10px] text-text-muted">{entity.entity_type}</span>
                  )}
                </button>
                <button
                  onClick={() => startEditEntity(entity)}
                  className="text-text-muted hover:text-primary-500 text-[10px] px-1 cursor-pointer"
                  title="Edit entity"
                >
                  &#9998;
                </button>
                <button
                  onClick={() => handleDeleteEntity(entity.id)}
                  className="opacity-0 group-hover:opacity-100 text-text-muted hover:text-danger text-xs px-1 transition-opacity cursor-pointer"
                  title="Delete entity"
                >
                  &times;
                </button>
              </div>
            ))}
          </div>

          <div className="p-3 border-t border-border">
            {showNewEntity ? (
              <div className="space-y-2">
                <input
                  type="text"
                  value={newEntityName}
                  onChange={(e) => setNewEntityName(e.target.value)}
                  placeholder="Entity name"
                  className="w-full px-3 py-2 bg-surface border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
                  autoFocus
                  onKeyDown={(e) => {
                    if (e.key === "Enter") handleCreateEntity();
                    if (e.key === "Escape") setShowNewEntity(false);
                  }}
                  {...filterInputProps}
                />
                <input
                  type="text"
                  value={newEntityType}
                  onChange={(e) => setNewEntityType(e.target.value)}
                  placeholder="Type (optional)"
                  className="w-full px-3 py-1.5 bg-surface border border-border rounded-lg text-xs focus:outline-none focus:ring-2 focus:ring-primary-500"
                  onKeyDown={(e) => {
                    if (e.key === "Enter") handleCreateEntity();
                    if (e.key === "Escape") setShowNewEntity(false);
                  }}
                  {...filterInputProps}
                />
                <div className="flex gap-2">
                  <button
                    onClick={handleCreateEntity}
                    className="flex-1 px-3 py-1.5 bg-primary-600 text-white text-xs font-medium rounded-lg hover:bg-primary-700 transition-colors cursor-pointer"
                  >
                    Create
                  </button>
                  <button
                    onClick={() => setShowNewEntity(false)}
                    className="px-3 py-1.5 text-xs text-text-secondary hover:text-text transition-colors cursor-pointer"
                  >
                    Cancel
                  </button>
                </div>
              </div>
            ) : (
              <button
                onClick={() => setShowNewEntity(true)}
                className="w-full px-3 py-2 text-sm text-primary-600 hover:bg-primary-50 dark:hover:bg-primary-900/20 rounded-lg transition-colors cursor-pointer"
              >
                + New Entity
              </button>
            )}
          </div>
        </div>

        {/* Mindmap area */}
        <div className="flex-1 bg-surface-alt relative">
          {loading && (
            <div className="absolute inset-0 flex items-center justify-center bg-surface-alt/80 z-10">
              <span className="text-text-muted text-sm">Loading mindmap...</span>
            </div>
          )}
          <MindMap data={mindmapData} onEntityClick={handleEntityClickInMindmap} onNodeDoubleClick={handleNodeDoubleClick} />

          {/* Legend */}
          {mindmapData && mindmapData.nodes.length > 0 && (
            <div className="absolute bottom-3 left-3 flex gap-4 text-xs text-text-muted bg-surface/80 backdrop-blur-sm rounded-lg px-3 py-2 border border-border">
              <span className="flex items-center gap-1">
                <span className="w-2 h-2 rounded-full bg-primary-500" /> Has cards
              </span>
              <span className="flex items-center gap-1">
                <span className="w-2 h-2 rounded-full bg-warning" /> Due
              </span>
              <span className="flex items-center gap-1">
                <span className="w-2 h-2 rounded-full bg-gray-400" /> No cards
              </span>
              <span className="text-text-muted">Click to explore, double-click to edit triples, scroll to zoom</span>
            </div>
          )}
        </div>
      </div>

      {/* Triple entry bar */}
      <div ref={tripleEntryRef}>
        <TripleEntry
          onTripleCreated={handleTripleCreated}
          focusedEntityId={selectedEntityId}
        />
      </div>

      {/* Modals */}
      {showSuggest && (
        <SuggestCards
          entityId={selectedEntityId}
          onClose={() => {
            setShowSuggest(false);
            loadDueCount();
            if (selectedEntityId) loadMindmap(selectedEntityId);
          }}
        />
      )}

      {editingEntity && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-surface rounded-2xl shadow-2xl w-full max-w-sm p-6">
            <h3 className="text-lg font-bold text-text mb-4">Edit Entity</h3>
            <div className="space-y-3">
              <div>
                <label className="block text-xs font-medium text-text-secondary mb-1">Name</label>
                <input
                  type="text"
                  value={editName}
                  onChange={(e) => setEditName(e.target.value)}
                  className="w-full px-3 py-2 bg-surface-alt border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
                  autoFocus
                  onKeyDown={(e) => {
                    if (e.key === "Enter") handleSaveEntity();
                    if (e.key === "Escape") setEditingEntity(null);
                  }}
                  {...filterInputProps}
                />
              </div>
              <div>
                <label className="block text-xs font-medium text-text-secondary mb-1">Type</label>
                <input
                  type="text"
                  value={editType}
                  onChange={(e) => setEditType(e.target.value)}
                  placeholder="e.g. Country, River, Person..."
                  className="w-full px-3 py-2 bg-surface-alt border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
                  onKeyDown={(e) => {
                    if (e.key === "Enter") handleSaveEntity();
                    if (e.key === "Escape") setEditingEntity(null);
                  }}
                  {...filterInputProps}
                />
              </div>
              <div>
                <label className="block text-xs font-medium text-text-secondary mb-1">Description</label>
                <textarea
                  value={editDesc}
                  onChange={(e) => setEditDesc(e.target.value)}
                  placeholder="Optional description..."
                  rows={2}
                  className="w-full px-3 py-2 bg-surface-alt border border-border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500 resize-none"
                  onKeyDown={(e) => {
                    if (e.key === "Escape") setEditingEntity(null);
                  }}
                />
              </div>
            </div>
            <div className="flex gap-2 mt-4">
              <button
                onClick={handleSaveEntity}
                disabled={!editName.trim()}
                className="flex-1 px-4 py-2 bg-primary-600 text-white text-sm font-medium rounded-lg hover:bg-primary-700 disabled:opacity-40 transition-colors cursor-pointer"
              >
                Save
              </button>
              <button
                onClick={() => setEditingEntity(null)}
                className="px-4 py-2 text-sm text-text-secondary hover:text-text transition-colors cursor-pointer"
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}

      {showEreReview && selectedDeckId && (
        <EreReview
          deckId={selectedDeckId}
          onClose={() => {
            setShowEreReview(false);
            loadDueCount();
            if (selectedEntityId) loadMindmap(selectedEntityId);
          }}
        />
      )}

      {tripleEditorNode && selectedEntityId && (
        <TripleEditor
          rootEntityId={selectedEntityId}
          focusedNode={tripleEditorNode}
          onClose={() => setTripleEditorNode(null)}
          onChanged={() => {
            loadEntities();
            if (selectedEntityId) loadMindmap(selectedEntityId);
            loadDueCount();
          }}
        />
      )}
    </div>
  );
}

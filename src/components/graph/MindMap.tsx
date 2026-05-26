import { useRef, useEffect, useCallback } from "react";
import type { MindMapData, MindMapNode, MindMapEdge } from "../../lib/tauri";

interface LayoutNode extends MindMapNode {
  x: number;
  y: number;
  radius: number;
}

interface Props {
  data: MindMapData | null;
  onEntityClick: (entityId: string) => void;
  onNodeDoubleClick?: (node: MindMapNode) => void;
}

const COLORS = {
  root: "#6366f1",
  relation: "#8b5cf6",
  reviewed: "#6366f1",
  due: "#f59e0b",
  none: "#9ca3af",
};

function layoutRadialTree(data: MindMapData, cx: number, cy: number): { nodes: LayoutNode[]; edges: MindMapEdge[] } {
  if (!data || data.nodes.length === 0) return { nodes: [], edges: data?.edges ?? [] };

  const root = data.nodes.find((n) => n.node_type === "root");
  if (!root) return { nodes: [], edges: data.edges };

  const relationNodes = data.nodes.filter((n) => n.node_type === "relation");
  const entityNodes = data.nodes.filter((n) => n.node_type === "entity");

  const layoutNodes: LayoutNode[] = [];

  layoutNodes.push({ ...root, x: cx, y: cy, radius: 28 });

  const ringRadius1 = Math.max(120, relationNodes.length * 30);
  relationNodes.forEach((rn, i) => {
    const angle = (i / Math.max(relationNodes.length, 1)) * Math.PI * 2 - Math.PI / 2;
    layoutNodes.push({
      ...rn,
      x: cx + Math.cos(angle) * ringRadius1,
      y: cy + Math.sin(angle) * ringRadius1,
      radius: 18,
    });
  });

  const childMap = new Map<string, string[]>();
  for (const edge of data.edges) {
    const srcIsRelation = relationNodes.some((r) => r.id === edge.source);
    if (srcIsRelation) {
      const list = childMap.get(edge.source) || [];
      list.push(edge.target);
      childMap.set(edge.source, list);
    }
  }

  const ringRadius2 = ringRadius1 + Math.max(100, entityNodes.length * 15);
  const placedEntities = new Set<string>();

  for (let ri = 0; ri < relationNodes.length; ri++) {
    const relNode = relationNodes[ri];
    const children = childMap.get(relNode.id) || [];
    const relAngle = (ri / Math.max(relationNodes.length, 1)) * Math.PI * 2 - Math.PI / 2;

    const spread = children.length > 1 ? Math.PI / Math.max(relationNodes.length, 2) : 0;
    children.forEach((childId, ci) => {
      if (placedEntities.has(childId)) return;
      placedEntities.add(childId);

      const childEntity = entityNodes.find((e) => e.id === childId);
      if (!childEntity) return;

      const childAngle = children.length === 1
        ? relAngle
        : relAngle - spread / 2 + (ci / (children.length - 1)) * spread;

      layoutNodes.push({
        ...childEntity,
        x: cx + Math.cos(childAngle) * ringRadius2,
        y: cy + Math.sin(childAngle) * ringRadius2,
        radius: 14,
      });
    });
  }

  // Place any orphan entities that weren't positioned
  entityNodes
    .filter((e) => !placedEntities.has(e.id))
    .forEach((e, i) => {
      const angle = (i / 8) * Math.PI * 2;
      layoutNodes.push({
        ...e,
        x: cx + Math.cos(angle) * (ringRadius2 + 60),
        y: cy + Math.sin(angle) * (ringRadius2 + 60),
        radius: 14,
      });
    });

  return { nodes: layoutNodes, edges: data.edges };
}

function getNodeColor(node: LayoutNode): string {
  if (node.node_type === "root") return COLORS.root;
  if (node.node_type === "relation") return COLORS.relation;
  if (node.card_status === "due") return COLORS.due;
  if (node.card_status === "reviewed") return COLORS.reviewed;
  return COLORS.none;
}

export function MindMap({ data, onEntityClick, onNodeDoubleClick }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const layoutRef = useRef<{ nodes: LayoutNode[]; edges: MindMapEdge[] }>({ nodes: [], edges: [] });
  const panRef = useRef({ x: 0, y: 0 });
  const dragRef = useRef<{ dragging: boolean; startX: number; startY: number; startPanX: number; startPanY: number }>({
    dragging: false,
    startX: 0,
    startY: 0,
    startPanX: 0,
    startPanY: 0,
  });
  const hoverRef = useRef<LayoutNode | null>(null);
  const scaleRef = useRef(1);

  const draw = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, canvas.width / dpr, canvas.height / dpr);

    const { nodes, edges } = layoutRef.current;
    if (nodes.length === 0) return;

    const scale = scaleRef.current;
    const panX = panRef.current.x;
    const panY = panRef.current.y;

    ctx.save();
    ctx.translate(panX, panY);
    ctx.scale(scale, scale);

    const nodeMap = new Map(nodes.map((n) => [n.id, n]));

    // Draw edges as bezier curves
    for (const edge of edges) {
      const src = nodeMap.get(edge.source);
      const tgt = nodeMap.get(edge.target);
      if (!src || !tgt) continue;

      ctx.beginPath();
      const mx = (src.x + tgt.x) / 2;
      const my = (src.y + tgt.y) / 2;
      const cx1 = mx + (src.y - tgt.y) * 0.15;
      const cy1 = my + (tgt.x - src.x) * 0.15;
      ctx.moveTo(src.x, src.y);
      ctx.quadraticCurveTo(cx1, cy1, tgt.x, tgt.y);
      ctx.strokeStyle = "#d1d5db";
      ctx.lineWidth = 1.5;
      ctx.globalAlpha = 0.5;
      ctx.stroke();
      ctx.globalAlpha = 1;
    }

    // Draw nodes
    for (const node of nodes) {
      const color = getNodeColor(node);
      const isHovered = hoverRef.current?.id === node.id;

      ctx.beginPath();
      ctx.arc(node.x, node.y, node.radius + (isHovered ? 3 : 0), 0, Math.PI * 2);
      ctx.fillStyle = color;
      ctx.fill();
      ctx.strokeStyle = "#ffffff";
      ctx.lineWidth = 2;
      ctx.stroke();

      if (isHovered) {
        ctx.beginPath();
        ctx.arc(node.x, node.y, node.radius + 6, 0, Math.PI * 2);
        ctx.strokeStyle = color;
        ctx.lineWidth = 2;
        ctx.globalAlpha = 0.3;
        ctx.stroke();
        ctx.globalAlpha = 1;
      }

      // Label
      ctx.fillStyle = "#e5e7eb";
      ctx.textAlign = "center";

      if (node.node_type === "root") {
        ctx.font = "bold 13px system-ui, sans-serif";
        ctx.fillStyle = "#f3f4f6";
        ctx.fillText(node.label.slice(0, 30), node.x, node.y + node.radius + 18);
      } else if (node.node_type === "relation") {
        ctx.font = "italic 11px system-ui, sans-serif";
        ctx.fillStyle = "#c4b5fd";
        const label = node.triple_count > 0 ? `${node.label} (${node.triple_count})` : node.label;
        ctx.fillText(label.slice(0, 30), node.x, node.y + node.radius + 16);
      } else {
        ctx.font = "12px system-ui, sans-serif";
        ctx.fillText(node.label.slice(0, 25), node.x, node.y + node.radius + 16);
        if (node.entity_type) {
          ctx.font = "10px system-ui, sans-serif";
          ctx.fillStyle = "#d1d5db";
          ctx.fillText(node.entity_type, node.x, node.y + node.radius + 28);
        }
      }
    }

    // Hover tooltip
    if (hoverRef.current && hoverRef.current.node_type !== "relation") {
      const h = hoverRef.current;
      const tooltipText = h.entity_type ? `${h.entity_type} • ${h.card_status}` : h.card_status;
      ctx.font = "11px system-ui, sans-serif";
      const tw = ctx.measureText(tooltipText).width + 16;
      const tx = h.x - tw / 2;
      const ty = h.y - h.radius - 28;
      ctx.fillStyle = "rgba(0,0,0,0.8)";
      ctx.beginPath();
      ctx.roundRect(tx, ty, tw, 22, 4);
      ctx.fill();
      ctx.fillStyle = "#fff";
      ctx.textAlign = "center";
      ctx.fillText(tooltipText, h.x, ty + 15);
    }

    ctx.restore();
  }, []);

  useEffect(() => {
    if (!data) return;
    const canvas = canvasRef.current;
    if (!canvas) return;
    const parent = canvas.parentElement;
    if (!parent) return;

    const dpr = window.devicePixelRatio || 1;
    const w = parent.clientWidth;
    const h = parent.clientHeight;
    canvas.width = w * dpr;
    canvas.height = h * dpr;
    canvas.style.width = `${w}px`;
    canvas.style.height = `${h}px`;

    const cx = w / 2;
    const cy = h / 2;
    layoutRef.current = layoutRadialTree(data, cx, cy);
    panRef.current = { x: 0, y: 0 };
    scaleRef.current = 1;
    draw();
  }, [data, draw]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const resize = () => {
      const parent = canvas.parentElement;
      if (!parent) return;
      const dpr = window.devicePixelRatio || 1;
      const w = parent.clientWidth;
      const h = parent.clientHeight;
      canvas.width = w * dpr;
      canvas.height = h * dpr;
      canvas.style.width = `${w}px`;
      canvas.style.height = `${h}px`;

      if (data) {
        layoutRef.current = layoutRadialTree(data, w / 2, h / 2);
      }
      draw();
    };

    window.addEventListener("resize", resize);
    return () => window.removeEventListener("resize", resize);
  }, [data, draw]);

  const screenToWorld = useCallback((sx: number, sy: number): [number, number] => {
    const scale = scaleRef.current;
    const px = panRef.current.x;
    const py = panRef.current.y;
    return [(sx - px) / scale, (sy - py) / scale];
  }, []);

  const hitTest = useCallback((sx: number, sy: number): LayoutNode | null => {
    const [wx, wy] = screenToWorld(sx, sy);
    for (const node of layoutRef.current.nodes) {
      const dx = wx - node.x;
      const dy = wy - node.y;
      if (dx * dx + dy * dy <= (node.radius + 6) * (node.radius + 6)) {
        return node;
      }
    }
    return null;
  }, [screenToWorld]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const onMouseDown = (e: MouseEvent) => {
      const rect = canvas.getBoundingClientRect();
      dragRef.current = {
        dragging: true,
        startX: e.clientX - rect.left,
        startY: e.clientY - rect.top,
        startPanX: panRef.current.x,
        startPanY: panRef.current.y,
      };
    };

    const onMouseMove = (e: MouseEvent) => {
      const rect = canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;

      if (dragRef.current.dragging) {
        panRef.current.x = dragRef.current.startPanX + (x - dragRef.current.startX);
        panRef.current.y = dragRef.current.startPanY + (y - dragRef.current.startY);
        draw();
        return;
      }

      const node = hitTest(x, y);
      if (node !== hoverRef.current) {
        hoverRef.current = node;
        canvas.style.cursor = node ? "pointer" : "grab";
        draw();
      }
    };

    const onMouseUp = (e: MouseEvent) => {
      const wasDragging = dragRef.current.dragging;
      const rect = canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;

      const didMove =
        Math.abs(x - dragRef.current.startX) > 5 || Math.abs(y - dragRef.current.startY) > 5;

      dragRef.current.dragging = false;

      if (wasDragging && !didMove) {
        const node = hitTest(x, y);
        if (node && node.node_type === "entity") {
          onEntityClick(node.id);
        }
      }
    };

    const onWheel = (e: WheelEvent) => {
      e.preventDefault();
      const rect = canvas.getBoundingClientRect();
      const mx = e.clientX - rect.left;
      const my = e.clientY - rect.top;

      const oldScale = scaleRef.current;
      const delta = e.deltaY > 0 ? 0.9 : 1.1;
      const newScale = Math.max(0.3, Math.min(3, oldScale * delta));

      panRef.current.x = mx - ((mx - panRef.current.x) / oldScale) * newScale;
      panRef.current.y = my - ((my - panRef.current.y) / oldScale) * newScale;
      scaleRef.current = newScale;
      draw();
    };

    const onDblClick = (e: MouseEvent) => {
      if (!onNodeDoubleClick) return;
      const rect = canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;
      const node = hitTest(x, y);
      if (node) {
        onNodeDoubleClick(node);
      }
    };

    canvas.addEventListener("mousedown", onMouseDown);
    canvas.addEventListener("mousemove", onMouseMove);
    canvas.addEventListener("mouseup", onMouseUp);
    canvas.addEventListener("dblclick", onDblClick);
    canvas.addEventListener("wheel", onWheel, { passive: false });

    return () => {
      canvas.removeEventListener("mousedown", onMouseDown);
      canvas.removeEventListener("mousemove", onMouseMove);
      canvas.removeEventListener("mouseup", onMouseUp);
      canvas.removeEventListener("dblclick", onDblClick);
      canvas.removeEventListener("wheel", onWheel);
    };
  }, [draw, hitTest, onEntityClick, onNodeDoubleClick]);

  return (
    <div className="w-full h-full relative">
      <canvas ref={canvasRef} className="w-full h-full" />
      {(!data || data.nodes.length === 0) && (
        <div className="absolute inset-0 flex items-center justify-center">
          <span className="text-text-muted text-sm">
            Select an entity to explore its connections
          </span>
        </div>
      )}
    </div>
  );
}

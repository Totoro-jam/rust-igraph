import { useRef, useEffect, useCallback, useState } from 'react';
import { ForceSimulation } from '../../simulation';
import { exportGml, exportDot, exportGraphml, exportEdgeList } from '../../graphExport';
import type { AlgoId, AlgoResult, Edge, AlgoResultScores, AlgoResultMembership, AlgoResultOrder } from '../../types';
import css from './index.module.css';

const PALETTE_DARK = [
  '#58a6ff', '#3fb950', '#d2a8ff', '#f0883e',
  '#f778ba', '#a5d6ff', '#ffd33d', '#ff7b72',
  '#7ee787', '#79c0ff', '#e0c060', '#f5a050',
];

const PALETTE_LIGHT = [
  '#0969da', '#1a7f37', '#8250df', '#bc4c00',
  '#bf3989', '#0550ae', '#9a6700', '#cf222e',
  '#116329', '#0969da', '#7d4e00', '#bc4c00',
];

type LayoutId = 'fr' | 'kamada_kawai' | 'circle' | 'random' | 'grid' | 'star';

const STATIC_LAYOUTS: Set<LayoutId> = new Set(['circle', 'grid', 'star', 'kamada_kawai']);

interface CanvasProps {
  coords: [number, number][] | null;
  edges: Edge[];
  vcount: number;
  result: AlgoResult | null;
  algo: AlgoId;
  directed: boolean;
  theme: 'dark' | 'light';
  layoutId: LayoutId;
  t: (key: string) => string;
}

interface ViewTransform {
  offsetX: number;
  offsetY: number;
  scale: number;
}

interface InteractionState {
  hoveredNode: number;
  selectedNode: number;
  dragNode: number;
}

function isScores(r: AlgoResult): r is AlgoResultScores {
  return 'scores' in r;
}

function isMembership(r: AlgoResult): r is AlgoResultMembership {
  return 'membership' in r;
}

function isOrder(r: AlgoResult): r is AlgoResultOrder {
  return 'order' in r;
}

function hslToRgb(h: number, s: number, l: number): string {
  const c = (1 - Math.abs(2 * l - 1)) * s;
  const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
  const m = l - c / 2;
  let r = 0, g = 0, b = 0;
  if (h < 60) { r = c; g = x; }
  else if (h < 120) { r = x; g = c; }
  else if (h < 180) { g = c; b = x; }
  else if (h < 240) { g = x; b = c; }
  else if (h < 300) { r = x; b = c; }
  else { r = c; b = x; }
  return `rgb(${Math.round((r + m) * 255)},${Math.round((g + m) * 255)},${Math.round((b + m) * 255)})`;
}

function interpolateColor(t: number, theme: 'dark' | 'light'): string {
  const hue = 240 - t * 240;
  const sat = 0.85;
  const light = theme === 'dark' ? 0.6 : 0.45;
  return hslToRgb(hue, sat, light);
}

function relativeLuminance(r: number, g: number, b: number): number {
  const rs = r / 255, gs = g / 255, bs = b / 255;
  const rl = rs <= 0.03928 ? rs / 12.92 : Math.pow((rs + 0.055) / 1.055, 2.4);
  const gl = gs <= 0.03928 ? gs / 12.92 : Math.pow((gs + 0.055) / 1.055, 2.4);
  const bl = bs <= 0.03928 ? bs / 12.92 : Math.pow((bs + 0.055) / 1.055, 2.4);
  return 0.2126 * rl + 0.7152 * gl + 0.0722 * bl;
}

function labelColorForNode(nodeColor: string): string {
  const m = nodeColor.match(/\d+/g);
  if (!m || m.length < 3) return '#ffffff';
  const lum = relativeLuminance(Number(m[0]), Number(m[1]), Number(m[2]));
  return lum > 0.3 ? '#1f2328' : '#f0f6fc';
}

function parseRgbFromHex(hex: string): [number, number, number] {
  const h = hex.replace('#', '');
  return [parseInt(h.slice(0, 2), 16), parseInt(h.slice(2, 4), 16), parseInt(h.slice(4, 6), 16)];
}

function labelColorForHex(hex: string): string {
  const [r, g, b] = parseRgbFromHex(hex);
  const lum = relativeLuminance(r, g, b);
  return lum > 0.3 ? '#1f2328' : '#f0f6fc';
}

function safeScores(arr: (number | null | undefined)[]): number[] {
  return arr.map((v) => (v != null && Number.isFinite(v) ? v : 0));
}

function getNodeColor(
  algo: AlgoId,
  result: AlgoResult | null,
  idx: number,
  theme: 'dark' | 'light',
): string {
  const palette = theme === 'dark' ? PALETTE_DARK : PALETTE_LIGHT;
  const dimmed = theme === 'dark' ? '#30363d' : '#d0d7de';
  if (!result) return palette[0]!;

  if (isMembership(result)) {
    return palette[(result.membership[idx] ?? 0) % palette.length]!;
  }
  if ('colors' in result) {
    const colors = (result as { colors: number[] }).colors;
    return palette[(colors[idx] ?? 0) % palette.length]!;
  }
  if ('hub' in result && 'authority' in result) {
    const hub = (result as { hub: number[] }).hub;
    const max = Math.max(...hub, 1e-9);
    const min = Math.min(...hub);
    const t = max > min ? ((hub[idx] ?? 0) - min) / (max - min) : 0.5;
    return interpolateColor(t, theme);
  }
  if (isScores(result)) {
    const scores = safeScores(result.scores);
    const max = Math.max(...scores, 1e-9);
    const min = Math.min(...scores);
    const t = max > min ? ((scores[idx] ?? 0) - min) / (max - min) : 0.5;
    return interpolateColor(t, theme);
  }
  if (isOrder(result)) {
    const order = result.order;
    const pos = order.indexOf(idx);
    if (pos < 0) return dimmed;
    const t = order.length > 1 ? pos / (order.length - 1) : 0;
    return interpolateColor(1 - t, theme);
  }
  if ('degrees' in result) {
    const degrees = (result as { degrees: number[] }).degrees;
    const max = Math.max(...degrees, 1);
    const min = Math.min(...degrees);
    const t = max > min ? ((degrees[idx] ?? 0) - min) / (max - min) : 0.5;
    return interpolateColor(t, theme);
  }
  if ('distances' in result) {
    const distances = (result as { distances: number[] }).distances;
    const finite = distances.filter((d) => d != null && Number.isFinite(d));
    const max = finite.length > 0 ? Math.max(...finite) : 1;
    const d = distances[idx] ?? Infinity;
    if (!Number.isFinite(d)) return dimmed;
    const t = max > 0 ? d / max : 0;
    return interpolateColor(1 - t, theme);
  }
  if ('vertices' in result) {
    const vertices = (result as { vertices: number[] }).vertices;
    if (vertices.includes(idx)) return theme === 'dark' ? '#f85149' : '#cf222e';
    return palette[0]!;
  }
  if ('edges' in result && 'count' in result) {
    if (algo === 'bridges') {
      const bridgeEdges = (result as { edges: [number, number][] }).edges;
      for (const e of bridgeEdges) {
        if (Array.isArray(e) && (e[0] === idx || e[1] === idx)) {
          return theme === 'dark' ? '#f85149' : '#cf222e';
        }
      }
    }
    return palette[0]!;
  }
  return palette[0]!;
}

function getNodeRadius(
  _algo: AlgoId,
  result: AlgoResult | null,
  idx: number,
  vcount: number,
): number {
  const base = Math.max(4, Math.min(14, 200 / Math.sqrt(Math.max(vcount, 1))));
  if (!result) return base;
  if (isScores(result)) {
    const scores = safeScores(result.scores);
    const max = Math.max(...scores, 1e-9);
    return base * (0.6 + 1.4 * (scores[idx] ?? 0) / max);
  }
  if ('hub' in result) {
    const hub = (result as { hub: number[] }).hub;
    const max = Math.max(...hub, 1e-9);
    return base * (0.6 + 1.4 * (hub[idx] ?? 0) / max);
  }
  if ('vertices' in result) {
    const vertices = (result as { vertices: number[] }).vertices;
    if (vertices.includes(idx)) return base * 1.5;
  }
  return base;
}

function formatNodeTooltip(
  idx: number,
  _algo: AlgoId,
  result: AlgoResult | null,
  edges: Edge[],
): string {
  let degree = 0;
  for (const [u, v] of edges) {
    if (u === idx || v === idx) degree++;
  }

  let info = `vertex ${idx}  (degree: ${degree})`;
  if (!result) return info;

  if ('hub' in result && 'authority' in result) {
    const r = result as { hub: number[]; authority: number[] };
    info += `\nhub: ${(r.hub[idx] ?? 0).toFixed(6)}`;
    info += `\nauthority: ${(r.authority[idx] ?? 0).toFixed(6)}`;
  } else if (isScores(result)) {
    const val = result.scores[idx];
    if (val != null && Number.isFinite(val)) info += `\nscore: ${val.toFixed(6)}`;
  } else if (isMembership(result)) {
    const c = result.membership[idx];
    if (c !== undefined) info += `\ncommunity: ${c}`;
  } else if (isOrder(result)) {
    const pos = result.order.indexOf(idx);
    if (pos >= 0) info += `\norder: ${pos}`;
  } else if ('colors' in result) {
    const colors = (result as { colors: number[] }).colors;
    info += `\ncolor: ${colors[idx]}`;
  } else if ('degrees' in result) {
    const degrees = (result as { degrees: number[] }).degrees;
    info += `\ndegree: ${degrees[idx]}`;
  } else if ('distances' in result) {
    const distances = (result as { distances: number[] }).distances;
    const d = distances[idx];
    info += `\ndistance: ${d !== undefined && Number.isFinite(d) ? d.toFixed(2) : '∞'}`;
  } else if ('vertices' in result) {
    const vertices = (result as { vertices: number[] }).vertices;
    if (vertices.includes(idx)) info += `\n★ articulation point`;
  }
  return info;
}

function isEdgeConnected(edge: Edge, nodeIdx: number): boolean {
  return edge[0] === nodeIdx || edge[1] === nodeIdx;
}

function buildNeighborSets(edges: Edge[], vcount: number): Set<number>[] {
  const sets: Set<number>[] = Array.from({ length: vcount }, () => new Set());
  for (const [u, v] of edges) {
    if (u < vcount && v < vcount) {
      sets[u]!.add(v);
      sets[v]!.add(u);
    }
  }
  return sets;
}

function worldToScreen(
  wx: number,
  wy: number,
  w: number,
  h: number,
  view: ViewTransform,
): [number, number] {
  return [
    wx * view.scale + w / 2 + view.offsetX,
    wy * view.scale + h / 2 + view.offsetY,
  ];
}

function screenToWorld(
  sx: number,
  sy: number,
  w: number,
  h: number,
  view: ViewTransform,
): [number, number] {
  return [
    (sx - w / 2 - view.offsetX) / view.scale,
    (sy - h / 2 - view.offsetY) / view.scale,
  ];
}

function drawGrid(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  view: ViewTransform,
  theme: 'dark' | 'light',
): void {
  const dotColor = theme === 'dark' ? 'rgba(88,166,255,0.06)' : 'rgba(9,105,218,0.05)';
  const spacing = 30 * view.scale;
  if (spacing < 8) return;

  const startX = (w / 2 + view.offsetX) % spacing;
  const startY = (h / 2 + view.offsetY) % spacing;
  const dotSize = Math.max(1, view.scale * 1.2);

  ctx.fillStyle = dotColor;
  for (let x = startX; x < w; x += spacing) {
    for (let y = startY; y < h; y += spacing) {
      ctx.beginPath();
      ctx.arc(x, y, dotSize, 0, Math.PI * 2);
      ctx.fill();
    }
  }
}

function drawArrow(
  ctx: CanvasRenderingContext2D,
  x1: number,
  y1: number,
  x2: number,
  y2: number,
  targetRadius: number,
  scale: number,
  color: string,
): void {
  const dx = x2 - x1, dy = y2 - y1;
  const dist = Math.sqrt(dx * dx + dy * dy) || 1;
  const nx = dx / dist, ny = dy / dist;
  const ax = x2 - nx * (targetRadius + 2);
  const ay = y2 - ny * (targetRadius + 2);
  const arrowLen = 8 * scale, arrowW = 4 * scale;
  ctx.beginPath();
  ctx.moveTo(ax, ay);
  ctx.lineTo(ax - nx * arrowLen + ny * arrowW, ay - ny * arrowLen - nx * arrowW);
  ctx.lineTo(ax - nx * arrowLen - ny * arrowW, ay - ny * arrowLen + nx * arrowW);
  ctx.closePath();
  ctx.fillStyle = color;
  ctx.fill();
}

export function Canvas({ coords, edges, vcount, result, algo, directed, theme, layoutId, t }: CanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<ViewTransform>({ offsetX: 0, offsetY: 0, scale: 1 });
  const simRef = useRef<ForceSimulation | null>(null);
  const interactionRef = useRef<InteractionState>({ hoveredNode: -1, selectedNode: -1, dragNode: -1 });
  const panDragRef = useRef<{ startX: number; startY: number; startOffsetX: number; startOffsetY: number } | null>(null);
  const neighborSetsRef = useRef<Set<number>[]>([]);

  const [tooltip, setTooltip] = useState<{ x: number; y: number; text: string } | null>(null);
  const [selectedInfo, setSelectedInfo] = useState<string | null>(null);
  const [simActive, setSimActive] = useState(false);

  const themeRef = useRef(theme);
  themeRef.current = theme;
  const algoRef = useRef(algo);
  algoRef.current = algo;
  const resultRef = useRef(result);
  resultRef.current = result;
  const edgesRef = useRef(edges);
  edgesRef.current = edges;
  const vcountRef = useRef(vcount);
  vcountRef.current = vcount;
  const directedRef = useRef(directed);
  directedRef.current = directed;
  const drawRef = useRef<() => void>(() => {});

  const draw = useCallback(() => {
    const canvas = canvasRef.current;
    const container = containerRef.current;
    const sim = simRef.current;
    if (!canvas || !container) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    const rect = container.getBoundingClientRect();
    const w = rect.width;
    const h = rect.height;

    if (canvas.width !== w * dpr || canvas.height !== h * dpr) {
      canvas.width = w * dpr;
      canvas.height = h * dpr;
      canvas.style.width = `${w}px`;
      canvas.style.height = `${h}px`;
    }
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, w, h);

    const currentTheme = themeRef.current;
    const currentAlgo = algoRef.current;
    const currentResult = resultRef.current;
    const currentEdges = edgesRef.current;
    const currentVcount = vcountRef.current;
    const currentDirected = directedRef.current;
    const view = viewRef.current;
    const interaction = interactionRef.current;

    drawGrid(ctx, w, h, view, currentTheme);

    if (!sim || currentVcount === 0) {
      ctx.fillStyle = currentTheme === 'dark' ? '#8b949e' : '#57606a';
      ctx.font = `14px ${getComputedStyle(document.documentElement).getPropertyValue('--font-sans')}`;
      ctx.textAlign = 'center';
      ctx.fillText(t('clickToRun'), w / 2, h / 2);
      return;
    }

    const simCoords = sim.getCoords();

    const screenCoords: [number, number][] = simCoords.map(
      ([wx, wy]) => worldToScreen(wx, wy, w, h, view),
    );

    const hoveredNode = interaction.hoveredNode;
    const selectedNode = interaction.selectedNode;

    // Draw edges
    for (let i = 0; i < currentEdges.length; i++) {
      const [u, v] = currentEdges[i]!;
      if (u >= currentVcount || v >= currentVcount) continue;
      const [x1, y1] = screenCoords[u]!;
      const [x2, y2] = screenCoords[v]!;

      const connected = hoveredNode >= 0 && isEdgeConnected(currentEdges[i]!, hoveredNode);
      const selectedConn = selectedNode >= 0 && isEdgeConnected(currentEdges[i]!, selectedNode);

      let edgeAlpha: number;
      let edgeWidth: number;
      if (connected || selectedConn) {
        edgeAlpha = 0.7;
        edgeWidth = 2;
      } else if (hoveredNode >= 0 || selectedNode >= 0) {
        edgeAlpha = 0.08;
        edgeWidth = 1;
      } else {
        edgeAlpha = 0.25;
        edgeWidth = 1;
      }

      const edgeColor = currentTheme === 'dark'
        ? `rgba(88,166,255,${edgeAlpha})`
        : `rgba(9,105,218,${edgeAlpha})`;

      ctx.strokeStyle = edgeColor;
      ctx.lineWidth = edgeWidth;
      ctx.beginPath();
      ctx.moveTo(x1, y1);
      ctx.lineTo(x2, y2);
      ctx.stroke();

      if (currentDirected) {
        const r = getNodeRadius(currentAlgo, currentResult, v, currentVcount) * view.scale;
        drawArrow(ctx, x1, y1, x2, y2, r, view.scale, edgeColor);
      }
    }

    // Draw nodes
    for (let i = 0; i < currentVcount; i++) {
      const [x, y] = screenCoords[i]!;
      const r = getNodeRadius(currentAlgo, currentResult, i, currentVcount) * view.scale;
      const color = getNodeColor(currentAlgo, currentResult, i, currentTheme);
      const isHovered = i === hoveredNode;
      const isSelected = i === selectedNode;
      const neighbors = neighborSetsRef.current;
      const isNeighborOfHovered = hoveredNode >= 0 && neighbors[hoveredNode]?.has(i);
      const isNeighborOfSelected = selectedNode >= 0 && neighbors[selectedNode]?.has(i);
      const isDimmed = (hoveredNode >= 0 || selectedNode >= 0) && !isHovered && !isSelected
        && !isNeighborOfHovered && !isNeighborOfSelected;

      // Shadow/glow for hovered or selected
      if (isHovered || isSelected) {
        ctx.shadowColor = color;
        ctx.shadowBlur = 12 * view.scale;
        ctx.beginPath();
        ctx.arc(x, y, r + 2 * view.scale, 0, Math.PI * 2);
        ctx.fillStyle = color;
        ctx.fill();
        ctx.shadowBlur = 0;
      }

      // Outer glow ring
      ctx.beginPath();
      ctx.arc(x, y, r + 3 * view.scale, 0, Math.PI * 2);
      const glowAlpha = isDimmed ? 0.03 : 0.12;
      ctx.fillStyle = color.replace('rgb', 'rgba').replace(')', `,${glowAlpha})`);
      ctx.fill();

      // Node body
      ctx.beginPath();
      ctx.arc(x, y, r, 0, Math.PI * 2);
      ctx.fillStyle = isDimmed ? `rgba(128,128,128,0.3)` : color;
      ctx.fill();

      // Selection ring
      if (isSelected) {
        ctx.beginPath();
        ctx.arc(x, y, r + 4 * view.scale, 0, Math.PI * 2);
        ctx.strokeStyle = currentTheme === 'dark' ? '#e6edf3' : '#24292f';
        ctx.lineWidth = 2;
        ctx.stroke();
      }

      // Label
      if (currentVcount <= 60 && view.scale >= 0.4) {
        let labelColor: string;
        if (isDimmed) {
          labelColor = currentTheme === 'dark' ? 'rgba(230,237,243,0.2)' : 'rgba(36,41,47,0.2)';
        } else if (color.startsWith('rgb')) {
          labelColor = labelColorForNode(color);
        } else {
          labelColor = labelColorForHex(color);
        }
        ctx.fillStyle = labelColor;
        ctx.font = `bold ${Math.max(9, r * 0.85)}px -apple-system, BlinkMacSystemFont, sans-serif`;
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText(String(i), x, y);
      }
    }
  }, [t]);

  drawRef.current = draw;

  const isStaticLayout = STATIC_LAYOUTS.has(layoutId);

  // Initialize/update simulation when coords change (triggered by Run)
  useEffect(() => {
    if (simRef.current) {
      simRef.current.destroy();
      simRef.current = null;
    }

    const currentEdges = edgesRef.current;
    const currentVcount = vcountRef.current;

    viewRef.current = { offsetX: 0, offsetY: 0, scale: 1 };
    interactionRef.current = { hoveredNode: -1, selectedNode: -1, dragNode: -1 };
    neighborSetsRef.current = buildNeighborSets(currentEdges, currentVcount);
    setTooltip(null);
    setSelectedInfo(null);

    if (!coords || currentVcount === 0) {
      setSimActive(false);
      drawRef.current();
      return;
    }

    if (isStaticLayout) {
      const sim = new ForceSimulation(currentVcount, currentEdges, coords, {
        linkDistance: 50,
        chargeStrength: 0,
        alphaDecay: 1,
        velocityDecay: 1,
      });
      sim.alpha = 0;
      sim.setOnTick(() => drawRef.current());
      simRef.current = sim;
      setSimActive(false);
      drawRef.current();

      return () => {
        sim.destroy();
        if (simRef.current === sim) simRef.current = null;
      };
    }

    const linkDist = Math.max(30, Math.min(80, 600 / Math.sqrt(Math.max(currentVcount, 1))));
    const charge = Math.max(-400, Math.min(-50, -4000 / Math.max(currentVcount, 1)));
    const nodeRadius = Math.max(4, Math.min(14, 200 / Math.sqrt(Math.max(currentVcount, 1))));

    const sim = new ForceSimulation(currentVcount, currentEdges, coords, {
      linkDistance: linkDist,
      chargeStrength: charge,
      alphaDecay: 0.02,
      velocityDecay: 0.35,
      collisionRadius: nodeRadius + 2,
    });

    sim.alpha = 0.15;
    sim.setOnTick(() => drawRef.current());
    simRef.current = sim;
    setSimActive(true);
    sim.start();
    drawRef.current();

    return () => {
      sim.destroy();
      if (simRef.current === sim) simRef.current = null;
    };
    // Only re-init on new coords (from Run) — edges/vcount read from refs
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [coords, isStaticLayout]);

  // Resize observer
  useEffect(() => {
    const observer = new ResizeObserver(() => draw());
    const container = containerRef.current;
    if (container) observer.observe(container);
    return () => observer.disconnect();
  }, [draw]);

  // Wheel zoom
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const handleWheel = (e: WheelEvent) => {
      e.preventDefault();
      const factor = e.deltaY > 0 ? 0.9 : 1.1;
      const view = viewRef.current;
      const newScale = Math.max(0.1, Math.min(10, view.scale * factor));

      const rect = containerRef.current?.getBoundingClientRect();
      if (rect) {
        const mx = e.clientX - rect.left;
        const my = e.clientY - rect.top;
        const ratio = newScale / view.scale;
        view.offsetX = mx - (mx - view.offsetX) * ratio;
        view.offsetY = my - (my - view.offsetY) * ratio;
      }

      view.scale = newScale;
      draw();
    };

    canvas.addEventListener('wheel', handleWheel, { passive: false });
    return () => canvas.removeEventListener('wheel', handleWheel);
  }, [draw]);

  // Find node under cursor
  const findNodeAt = useCallback((mx: number, my: number): number => {
    const sim = simRef.current;
    const container = containerRef.current;
    if (!sim || !container) return -1;

    const rect = container.getBoundingClientRect();
    const w = rect.width;
    const h = rect.height;
    const view = viewRef.current;

    const [wx, wy] = screenToWorld(mx, my, w, h, view);

    let closest = -1;
    let closestDist = Infinity;

    for (let i = 0; i < sim.nodes.length; i++) {
      const n = sim.nodes[i]!;
      const dx = wx - n.x;
      const dy = wy - n.y;
      const dist2 = dx * dx + dy * dy;
      const hitR = getNodeRadius(algoRef.current, resultRef.current, i, vcountRef.current) / view.scale + 8 / view.scale;
      if (dist2 < hitR * hitR && dist2 < closestDist) {
        closest = i;
        closestDist = dist2;
      }
    }

    return closest;
  }, []);

  // Mouse interactions
  useEffect(() => {
    const canvas = canvasRef.current;
    const container = containerRef.current;
    if (!canvas || !container) return;

    const handleMouseDown = (e: MouseEvent) => {
      if (e.button !== 0) return;
      const rect = container.getBoundingClientRect();
      const mx = e.clientX - rect.left;
      const my = e.clientY - rect.top;

      const nodeIdx = findNodeAt(mx, my);

      if (nodeIdx >= 0) {
        interactionRef.current.dragNode = nodeIdx;
        const sim = simRef.current;
        if (sim) {
          const w = rect.width;
          const h = rect.height;
          const [wx, wy] = screenToWorld(mx, my, w, h, viewRef.current);
          sim.pinNode(nodeIdx, wx, wy);
        }
        canvas.style.cursor = 'grabbing';
      } else {
        // Start panning
        const view = viewRef.current;
        panDragRef.current = {
          startX: e.clientX,
          startY: e.clientY,
          startOffsetX: view.offsetX,
          startOffsetY: view.offsetY,
        };
        canvas.style.cursor = 'grabbing';
      }
    };

    const handleMouseMove = (e: MouseEvent) => {
      const rect = container.getBoundingClientRect();
      const mx = e.clientX - rect.left;
      const my = e.clientY - rect.top;
      const interaction = interactionRef.current;

      // Node dragging
      if (interaction.dragNode >= 0) {
        const sim = simRef.current;
        if (sim) {
          const w = rect.width;
          const h = rect.height;
          const [wx, wy] = screenToWorld(mx, my, w, h, viewRef.current);
          sim.pinNode(interaction.dragNode, wx, wy);
          sim.reheat(0.1);
        }
        setTooltip(null);
        return;
      }

      // Panning
      if (panDragRef.current) {
        const pd = panDragRef.current;
        const view = viewRef.current;
        view.offsetX = pd.startOffsetX + (e.clientX - pd.startX);
        view.offsetY = pd.startOffsetY + (e.clientY - pd.startY);
        draw();
        setTooltip(null);
        return;
      }

      // Hover detection
      const nodeIdx = findNodeAt(mx, my);
      const prevHovered = interaction.hoveredNode;

      if (nodeIdx !== prevHovered) {
        interaction.hoveredNode = nodeIdx;
        draw();

        if (nodeIdx >= 0) {
          const sim = simRef.current;
          if (sim) {
            const n = sim.nodes[nodeIdx]!;
            const w = rect.width;
            const h = rect.height;
            const [sx, sy] = worldToScreen(n.x, n.y, w, h, viewRef.current);
            setTooltip({
              x: sx,
              y: sy,
              text: formatNodeTooltip(nodeIdx, algoRef.current, resultRef.current, edgesRef.current),
            });
          }
          canvas.style.cursor = 'pointer';
        } else {
          setTooltip(null);
          canvas.style.cursor = 'grab';
        }
      }
    };

    const handleMouseUp = () => {
      const interaction = interactionRef.current;

      if (interaction.dragNode >= 0) {
        const sim = simRef.current;
        if (sim) {
          sim.unpinNode(interaction.dragNode);
        }
        interaction.dragNode = -1;
      }

      panDragRef.current = null;
      canvas.style.cursor = interactionRef.current.hoveredNode >= 0 ? 'pointer' : 'grab';
    };

    const handleClick = (e: MouseEvent) => {
      const rect = container.getBoundingClientRect();
      const mx = e.clientX - rect.left;
      const my = e.clientY - rect.top;

      const nodeIdx = findNodeAt(mx, my);
      const interaction = interactionRef.current;

      if (nodeIdx >= 0) {
        if (interaction.selectedNode === nodeIdx) {
          interaction.selectedNode = -1;
          setSelectedInfo(null);
        } else {
          interaction.selectedNode = nodeIdx;
          setSelectedInfo(formatNodeTooltip(nodeIdx, algoRef.current, resultRef.current, edgesRef.current));
        }
      } else {
        interaction.selectedNode = -1;
        setSelectedInfo(null);
      }
      draw();
    };

    const handleMouseLeave = () => {
      interactionRef.current.hoveredNode = -1;
      setTooltip(null);
      draw();
    };

    // Touch events for mobile
    const handleTouchStart = (e: TouchEvent) => {
      if (e.touches.length !== 1) return;
      const touch = e.touches[0]!;
      const rect = container.getBoundingClientRect();
      const mx = touch.clientX - rect.left;
      const my = touch.clientY - rect.top;

      const nodeIdx = findNodeAt(mx, my);
      if (nodeIdx >= 0) {
        e.preventDefault();
        interactionRef.current.dragNode = nodeIdx;
        const sim = simRef.current;
        if (sim) {
          const w = rect.width;
          const h = rect.height;
          const [wx, wy] = screenToWorld(mx, my, w, h, viewRef.current);
          sim.pinNode(nodeIdx, wx, wy);
        }
      } else {
        const view = viewRef.current;
        panDragRef.current = {
          startX: touch.clientX,
          startY: touch.clientY,
          startOffsetX: view.offsetX,
          startOffsetY: view.offsetY,
        };
      }
    };

    const handleTouchMove = (e: TouchEvent) => {
      if (e.touches.length !== 1) return;
      const touch = e.touches[0]!;
      const rect = container.getBoundingClientRect();
      const mx = touch.clientX - rect.left;
      const my = touch.clientY - rect.top;
      const interaction = interactionRef.current;

      if (interaction.dragNode >= 0) {
        e.preventDefault();
        const sim = simRef.current;
        if (sim) {
          const w = rect.width;
          const h = rect.height;
          const [wx, wy] = screenToWorld(mx, my, w, h, viewRef.current);
          sim.pinNode(interaction.dragNode, wx, wy);
          sim.reheat(0.1);
        }
        return;
      }

      if (panDragRef.current) {
        e.preventDefault();
        const pd = panDragRef.current;
        const view = viewRef.current;
        view.offsetX = pd.startOffsetX + (touch.clientX - pd.startX);
        view.offsetY = pd.startOffsetY + (touch.clientY - pd.startY);
        draw();
      }
    };

    const handleTouchEnd = () => {
      const interaction = interactionRef.current;
      if (interaction.dragNode >= 0) {
        const sim = simRef.current;
        if (sim) sim.unpinNode(interaction.dragNode);
        interaction.dragNode = -1;
      }
      panDragRef.current = null;
    };

    canvas.addEventListener('mousedown', handleMouseDown);
    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
    canvas.addEventListener('click', handleClick);
    canvas.addEventListener('mouseleave', handleMouseLeave);
    canvas.addEventListener('touchstart', handleTouchStart, { passive: false });
    canvas.addEventListener('touchmove', handleTouchMove, { passive: false });
    canvas.addEventListener('touchend', handleTouchEnd);

    return () => {
      canvas.removeEventListener('mousedown', handleMouseDown);
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
      canvas.removeEventListener('click', handleClick);
      canvas.removeEventListener('mouseleave', handleMouseLeave);
      canvas.removeEventListener('touchstart', handleTouchStart);
      canvas.removeEventListener('touchmove', handleTouchMove);
      canvas.removeEventListener('touchend', handleTouchEnd);
    };
  }, [draw, findNodeAt]);

  const zoomIn = useCallback(() => {
    const view = viewRef.current;
    view.scale = Math.min(10, view.scale * 1.25);
    draw();
  }, [draw]);

  const zoomOut = useCallback(() => {
    const view = viewRef.current;
    view.scale = Math.max(0.1, view.scale * 0.8);
    draw();
  }, [draw]);

  const zoomFit = useCallback(() => {
    viewRef.current = { offsetX: 0, offsetY: 0, scale: 1 };
    draw();
  }, [draw]);

  const toggleSim = useCallback(() => {
    const sim = simRef.current;
    if (!sim) return;
    if (sim.isRunning()) {
      sim.stop();
      setSimActive(false);
    } else {
      sim.reheat(0.5);
      setSimActive(true);
    }
  }, []);

  const shakeLayout = useCallback(() => {
    const sim = simRef.current;
    if (!sim) return;
    for (const n of sim.nodes) {
      n.vx += (Math.random() - 0.5) * 20;
      n.vy += (Math.random() - 0.5) * 20;
    }
    sim.reheat(0.8);
    setSimActive(true);
  }, []);

  const [exportOpen, setExportOpen] = useState(false);

  const exportPng = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const link = document.createElement('a');
    link.download = 'graph.png';
    link.href = canvas.toDataURL('image/png');
    link.click();
  }, []);

  const handleExport = useCallback((fmt: string) => {
    setExportOpen(false);
    switch (fmt) {
      case 'png': exportPng(); break;
      case 'gml': exportGml(edges, vcount, directed); break;
      case 'dot': exportDot(edges, vcount, directed); break;
      case 'graphml': exportGraphml(edges, vcount, directed); break;
      case 'edgelist': exportEdgeList(edges, vcount, directed); break;
    }
  }, [exportPng, edges, vcount, directed]);

  useEffect(() => {
    if (!exportOpen) return;
    const close = () => setExportOpen(false);
    const timer = setTimeout(() => document.addEventListener('click', close), 0);
    return () => { clearTimeout(timer); document.removeEventListener('click', close); };
  }, [exportOpen]);

  const handleSpotlight = useCallback((e: React.MouseEvent<HTMLDivElement>) => {
    const el = e.currentTarget;
    const rect = el.getBoundingClientRect();
    el.style.setProperty('--spot-x', `${e.clientX - rect.left}px`);
    el.style.setProperty('--spot-y', `${e.clientY - rect.top}px`);
  }, []);

  return (
    <div className={css.canvasContainer} ref={containerRef} onMouseMove={handleSpotlight}>
      <canvas ref={canvasRef} style={{ cursor: 'grab' }} />

      <div className={css.toolbar}>
        <button onClick={zoomIn} title={t('zoomIn')}>+</button>
        <button onClick={zoomOut} title={t('zoomOut')}>&minus;</button>
        <div className={css.toolbarDivider} />
        <button onClick={zoomFit} title={t('fitView')}>
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
            <rect x="2" y="2" width="12" height="12" rx="1" />
            <path d="M2 6h12M2 10h12M6 2v12M10 2v12" opacity="0.3" />
          </svg>
        </button>
        {!isStaticLayout && (
          <>
            <div className={css.toolbarDivider} />
            <button
              onClick={toggleSim}
              title={simActive ? t('pauseSim') : t('resumeSim')}
              className={simActive ? css.toolbarActive : ''}
            >
              {simActive ? (
                <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
                  <rect x="3" y="2" width="4" height="12" rx="1" />
                  <rect x="9" y="2" width="4" height="12" rx="1" />
                </svg>
              ) : (
                <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
                  <path d="M4 2l10 6-10 6V2z" />
                </svg>
              )}
            </button>
            <button onClick={shakeLayout} title={t('shuffleLayout')}>
              <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
                <path d="M2 4h4l2 4-2 4H2M14 4h-4l-2 4 2 4h4" />
              </svg>
            </button>
          </>
        )}
        <div className={css.toolbarDivider} />
        <div className={css.exportWrap}>
          <button onClick={() => setExportOpen((p) => !p)} title={t('export')}>
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
              <path d="M8 2v8M5 7l3 3 3-3M3 12h10" />
            </svg>
          </button>
          {exportOpen && (
            <div className={css.exportMenu}>
              <button onClick={() => handleExport('png')}>PNG</button>
              <button onClick={() => handleExport('gml')}>GML</button>
              <button onClick={() => handleExport('dot')}>DOT</button>
              <button onClick={() => handleExport('graphml')}>GraphML</button>
              <button onClick={() => handleExport('edgelist')}>{t('exportEdgeList')}</button>
            </div>
          )}
        </div>
      </div>

      {tooltip && (
        <div
          className={css.tooltip}
          style={{ left: tooltip.x, top: tooltip.y }}
        >
          {tooltip.text}
        </div>
      )}

      {selectedInfo && (
        <div className={css.selectionInfo}>
          <div className={css.selectionHeader}>
            <span>{t('selectedNode')}</span>
            <button onClick={() => { interactionRef.current.selectedNode = -1; setSelectedInfo(null); draw(); }}>
              &times;
            </button>
          </div>
          <pre>{selectedInfo}</pre>
        </div>
      )}
    </div>
  );
}

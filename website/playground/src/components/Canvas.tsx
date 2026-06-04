import { useRef, useEffect, useCallback, useState } from 'react';
import type { AlgoId, AlgoResult, Edge, AlgoResultScores, AlgoResultMembership, AlgoResultOrder } from '../types';

const PALETTE = [
  '#58a6ff', '#3fb950', '#d2a8ff', '#f0883e',
  '#f778ba', '#a5d6ff', '#ffd33d', '#ff7b72',
];

interface CanvasProps {
  coords: [number, number][] | null;
  edges: Edge[];
  vcount: number;
  result: AlgoResult | null;
  algo: AlgoId;
  directed: boolean;
  theme: 'dark' | 'light';
  t: (key: string) => string;
}

interface ViewTransform {
  offsetX: number;
  offsetY: number;
  scale: number;
}

interface TooltipState {
  x: number;
  y: number;
  text: string;
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

function interpolateColor(t: number): string {
  const r = Math.round(t < 0.5 ? 0 : (t - 0.5) * 2 * 255);
  const g = Math.round(t < 0.5 ? t * 2 * 200 : (1 - t) * 2 * 200);
  const b = Math.round(t < 0.5 ? 255 - t * 2 * 200 : 0);
  return `rgb(${r},${g},${b})`;
}

function getNodeColor(
  algo: AlgoId,
  result: AlgoResult | null,
  idx: number,
  theme: 'dark' | 'light',
): string {
  if (!result) return PALETTE[0]!;
  if (
    (algo === 'louvain' || algo === 'components' || algo === 'infomap' || algo === 'spinglass') &&
    isMembership(result)
  ) {
    return PALETTE[(result.membership[idx] ?? 0) % PALETTE.length]!;
  }
  if (algo === 'pagerank' && isScores(result)) {
    const scores = result.scores;
    const max = Math.max(...scores, 1e-9);
    const min = Math.min(...scores);
    const t = max > min ? ((scores[idx] ?? 0) - min) / (max - min) : 0.5;
    return interpolateColor(t);
  }
  if (algo === 'betweenness' && isScores(result)) {
    const scores = result.scores;
    const max = Math.max(...scores, 1e-9);
    const t = max > 0 ? (scores[idx] ?? 0) / max : 0;
    return interpolateColor(t);
  }
  if (algo === 'bfs' && isOrder(result)) {
    const order = result.order;
    const pos = order.indexOf(idx);
    if (pos < 0) return theme === 'dark' ? '#30363d' : '#d0d7de';
    const t = order.length > 1 ? pos / (order.length - 1) : 0;
    return interpolateColor(1 - t);
  }
  return PALETTE[0]!;
}

function getNodeRadius(
  algo: AlgoId,
  result: AlgoResult | null,
  idx: number,
  vcount: number,
): number {
  const base = Math.max(4, Math.min(14, 200 / Math.sqrt(Math.max(vcount, 1))));
  if (!result) return base;
  if (algo === 'pagerank' && isScores(result)) {
    const max = Math.max(...result.scores, 1e-9);
    return base * (0.6 + 1.4 * (result.scores[idx] ?? 0) / max);
  }
  if (algo === 'betweenness' && isScores(result)) {
    const max = Math.max(...result.scores, 1e-9);
    return base * (0.6 + 1.4 * (max > 0 ? (result.scores[idx] ?? 0) / max : 0.5));
  }
  return base;
}

function formatNodeTooltip(
  idx: number,
  algo: AlgoId,
  result: AlgoResult | null,
): string {
  let info = `vertex ${idx}`;
  if (!result) return info;

  if (isScores(result)) {
    const val = result.scores[idx];
    if (val !== undefined) {
      const label = algo === 'pagerank' ? 'rank' : 'score';
      info += `\n${label}: ${val.toFixed(6)}`;
    }
  } else if (isMembership(result)) {
    const c = result.membership[idx];
    if (c !== undefined) {
      info += `\ncommunity: ${c}`;
    }
  } else if (isOrder(result)) {
    const pos = result.order.indexOf(idx);
    if (pos >= 0) {
      info += `\nbfs order: ${pos}`;
    }
  }
  return info;
}

function computeScreenCoords(
  coords: [number, number][],
  w: number,
  h: number,
  view: ViewTransform,
): [number, number][] {
  const pad = 40;
  const drawW = w - pad * 2;
  const drawH = h - pad * 2;

  let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
  for (const [x, y] of coords) {
    if (x < minX) minX = x;
    if (x > maxX) maxX = x;
    if (y < minY) minY = y;
    if (y > maxY) maxY = y;
  }
  const rangeX = maxX - minX || 1;
  const rangeY = maxY - minY || 1;

  return coords.map(([x, y]) => {
    const sx = pad + ((x - minX) / rangeX) * drawW;
    const sy = pad + ((y - minY) / rangeY) * drawH;
    return [
      (sx - w / 2) * view.scale + w / 2 + view.offsetX,
      (sy - h / 2) * view.scale + h / 2 + view.offsetY,
    ];
  });
}

export function Canvas({ coords, edges, vcount, result, algo, directed, theme, t }: CanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<ViewTransform>({ offsetX: 0, offsetY: 0, scale: 1 });
  const dragRef = useRef<{ startX: number; startY: number; startOffsetX: number; startOffsetY: number } | null>(null);
  const screenCoordsRef = useRef<[number, number][]>([]);

  const [tooltip, setTooltip] = useState<TooltipState | null>(null);

  const draw = useCallback(() => {
    const canvas = canvasRef.current;
    const container = containerRef.current;
    if (!canvas || !container) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    const rect = container.getBoundingClientRect();
    const w = rect.width;
    const h = rect.height;

    canvas.width = w * dpr;
    canvas.height = h * dpr;
    canvas.style.width = `${w}px`;
    canvas.style.height = `${h}px`;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

    ctx.clearRect(0, 0, w, h);

    if (!coords) {
      ctx.fillStyle = theme === 'dark' ? '#8b949e' : '#57606a';
      ctx.font = `14px ${getComputedStyle(document.documentElement).getPropertyValue('--font-sans')}`;
      ctx.textAlign = 'center';
      ctx.fillText(t('clickToRun'), w / 2, h / 2);
      return;
    }

    const view = viewRef.current;
    const screenCoords = computeScreenCoords(coords, w, h, view);
    screenCoordsRef.current = screenCoords;

    const edgeColor = theme === 'dark' ? 'rgba(88,166,255,0.2)' : 'rgba(9,105,218,0.2)';
    ctx.strokeStyle = edgeColor;
    ctx.lineWidth = 1;

    for (const [u, v] of edges) {
      if (u >= vcount || v >= vcount) continue;
      const [x1, y1] = screenCoords[u]!;
      const [x2, y2] = screenCoords[v]!;
      ctx.beginPath();
      ctx.moveTo(x1, y1);
      ctx.lineTo(x2, y2);
      ctx.stroke();

      if (directed) {
        const r = getNodeRadius(algo, result, v, vcount) * view.scale;
        const dx = x2 - x1, dy = y2 - y1;
        const dist = Math.sqrt(dx * dx + dy * dy) || 1;
        const nx = dx / dist, ny = dy / dist;
        const ax = x2 - nx * (r + 2), ay = y2 - ny * (r + 2);
        const arrowLen = 8 * view.scale, arrowW = 4 * view.scale;
        ctx.beginPath();
        ctx.moveTo(ax, ay);
        ctx.lineTo(ax - nx * arrowLen + ny * arrowW, ay - ny * arrowLen - nx * arrowW);
        ctx.lineTo(ax - nx * arrowLen - ny * arrowW, ay - ny * arrowLen + nx * arrowW);
        ctx.closePath();
        ctx.fillStyle = edgeColor;
        ctx.fill();
      }
    }

    for (let i = 0; i < vcount; i++) {
      const [x, y] = screenCoords[i]!;
      const r = getNodeRadius(algo, result, i, vcount) * view.scale;
      const color = getNodeColor(algo, result, i, theme);

      ctx.beginPath();
      ctx.arc(x, y, r + 3 * view.scale, 0, Math.PI * 2);
      ctx.fillStyle = color.replace('rgb', 'rgba').replace(')', ',0.15)');
      ctx.fill();

      ctx.beginPath();
      ctx.arc(x, y, r, 0, Math.PI * 2);
      ctx.fillStyle = color;
      ctx.fill();

      if (vcount <= 50 && view.scale >= 0.5) {
        ctx.fillStyle = theme === 'dark' ? '#e6edf3' : '#24292f';
        ctx.font = `bold ${Math.max(9, r * 0.9)}px -apple-system, BlinkMacSystemFont, sans-serif`;
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText(String(i), x, y);
      }
    }
  }, [coords, edges, vcount, result, algo, directed, theme, t]);

  useEffect(() => {
    viewRef.current = { offsetX: 0, offsetY: 0, scale: 1 };
    draw();
  }, [coords, draw]);

  useEffect(() => {
    const observer = new ResizeObserver(() => draw());
    const container = containerRef.current;
    if (container) observer.observe(container);
    return () => observer.disconnect();
  }, [draw]);

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

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (e.button !== 0) return;
    const view = viewRef.current;
    dragRef.current = {
      startX: e.clientX,
      startY: e.clientY,
      startOffsetX: view.offsetX,
      startOffsetY: view.offsetY,
    };
  }, []);

  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      const drag = dragRef.current;
      if (drag) {
        const view = viewRef.current;
        view.offsetX = drag.startOffsetX + (e.clientX - drag.startX);
        view.offsetY = drag.startOffsetY + (e.clientY - drag.startY);
        draw();
        setTooltip(null);
        return;
      }

      if (!coords || !containerRef.current) return;
      const rect = containerRef.current.getBoundingClientRect();
      const mx = e.clientX - rect.left;
      const my = e.clientY - rect.top;
      const view = viewRef.current;

      let closest = -1;
      let closestDist = Infinity;
      for (let i = 0; i < screenCoordsRef.current.length; i++) {
        const [sx, sy] = screenCoordsRef.current[i]!;
        const dx = mx - sx, dy = my - sy;
        const dist = dx * dx + dy * dy;
        const hitRadius = getNodeRadius(algo, result, i, vcount) * view.scale + 6;
        if (dist < hitRadius * hitRadius && dist < closestDist) {
          closest = i;
          closestDist = dist;
        }
      }

      if (closest >= 0) {
        const [sx, sy] = screenCoordsRef.current[closest]!;
        setTooltip({
          x: sx,
          y: sy,
          text: formatNodeTooltip(closest, algo, result),
        });
      } else {
        setTooltip(null);
      }
    };

    const handleMouseUp = () => {
      dragRef.current = null;
    };

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };
  }, [draw, coords, algo, result, vcount]);

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

  return (
    <div className="canvas-container" ref={containerRef}>
      <canvas
        ref={canvasRef}
        onMouseDown={handleMouseDown}
        style={{ cursor: dragRef.current ? 'grabbing' : 'grab' }}
      />
      <div className="canvas-toolbar">
        <button onClick={zoomIn} title="Zoom in">+</button>
        <button onClick={zoomOut} title="Zoom out">&minus;</button>
        <div className="toolbar-divider" />
        <button onClick={zoomFit} title="Fit to view">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
            <rect x="2" y="2" width="12" height="12" rx="1" />
            <path d="M2 6h12M2 10h12M6 2v12M10 2v12" opacity="0.3" />
          </svg>
        </button>
      </div>
      {tooltip && (
        <div
          className="canvas-tooltip"
          style={{ left: tooltip.x, top: tooltip.y }}
        >
          {tooltip.text}
        </div>
      )}
    </div>
  );
}

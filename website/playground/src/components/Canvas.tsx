import { useRef, useEffect, useCallback } from 'react';
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

export function Canvas({ coords, edges, vcount, result, algo, directed, theme, t }: CanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

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
      ctx.font = `14px -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif`;
      ctx.textAlign = 'center';
      ctx.fillText(t('clickToRun'), w / 2, h / 2);
      return;
    }

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

    const screenCoords = coords.map(([x, y]) => [
      pad + ((x - minX) / rangeX) * drawW,
      pad + ((y - minY) / rangeY) * drawH,
    ] as [number, number]);

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
        const r = getNodeRadius(algo, result, v, vcount);
        const dx = x2 - x1, dy = y2 - y1;
        const dist = Math.sqrt(dx * dx + dy * dy) || 1;
        const nx = dx / dist, ny = dy / dist;
        const ax = x2 - nx * (r + 2), ay = y2 - ny * (r + 2);
        const arrowLen = 8, arrowW = 4;
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
      const r = getNodeRadius(algo, result, i, vcount);
      const color = getNodeColor(algo, result, i, theme);

      ctx.beginPath();
      ctx.arc(x, y, r + 3, 0, Math.PI * 2);
      ctx.fillStyle = color.replace('rgb', 'rgba').replace(')', ',0.15)');
      ctx.fill();

      ctx.beginPath();
      ctx.arc(x, y, r, 0, Math.PI * 2);
      ctx.fillStyle = color;
      ctx.fill();

      if (vcount <= 50) {
        ctx.fillStyle = theme === 'dark' ? '#e6edf3' : '#24292f';
        ctx.font = `bold ${Math.max(9, r * 0.9)}px -apple-system, BlinkMacSystemFont, sans-serif`;
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText(String(i), x, y);
      }
    }
  }, [coords, edges, vcount, result, algo, directed, theme, t]);

  useEffect(() => {
    draw();
  }, [draw]);

  useEffect(() => {
    const observer = new ResizeObserver(() => draw());
    const container = containerRef.current;
    if (container) observer.observe(container);
    return () => observer.disconnect();
  }, [draw]);

  return (
    <div className="canvas-container" ref={containerRef}>
      <canvas ref={canvasRef} />
    </div>
  );
}

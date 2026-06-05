import type { Edge } from './types';

export interface SimNode {
  x: number;
  y: number;
  vx: number;
  vy: number;
  fx: number | null;
  fy: number | null;
}

export interface ForceSimulationOptions {
  chargeStrength?: number;
  linkDistance?: number;
  linkStrength?: number;
  centerStrength?: number;
  alphaDecay?: number;
  alphaMin?: number;
  velocityDecay?: number;
  collisionRadius?: number;
}

const DEFAULTS: Required<ForceSimulationOptions> = {
  chargeStrength: -120,
  linkDistance: 50,
  linkStrength: 0.3,
  centerStrength: 0.05,
  alphaDecay: 0.0228,
  alphaMin: 0.001,
  velocityDecay: 0.4,
  collisionRadius: 12,
};

export class ForceSimulation {
  nodes: SimNode[];
  edges: Edge[];
  vcount: number;
  alpha: number;
  alphaTarget: number;
  opts: Required<ForceSimulationOptions>;

  private adjList: number[][];
  private running: boolean;
  private frameId: number | null;
  private onTick: (() => void) | null;

  constructor(
    vcount: number,
    edges: Edge[],
    initialCoords: [number, number][] | null,
    options?: ForceSimulationOptions,
  ) {
    this.vcount = vcount;
    this.edges = edges;
    this.alpha = 1;
    this.alphaTarget = 0;
    this.opts = { ...DEFAULTS, ...options };
    this.running = false;
    this.frameId = null;
    this.onTick = null;

    this.adjList = Array.from({ length: vcount }, () => []);
    for (const [u, v] of edges) {
      if (u < vcount && v < vcount) {
        this.adjList[u]!.push(v);
        this.adjList[v]!.push(u);
      }
    }

    this.nodes = Array.from({ length: vcount }, (_, i) => {
      const hasCoord = initialCoords && i < initialCoords.length;
      return {
        x: hasCoord ? initialCoords[i]![0] : Math.random() * 400 - 200,
        y: hasCoord ? initialCoords[i]![1] : Math.random() * 400 - 200,
        vx: 0,
        vy: 0,
        fx: null,
        fy: null,
      };
    });

    if (initialCoords) {
      this.scaleToCenter(400, 400);
    }
  }

  private scaleToCenter(w: number, h: number): void {
    if (this.vcount === 0) return;

    let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
    for (const n of this.nodes) {
      if (n.x < minX) minX = n.x;
      if (n.x > maxX) maxX = n.x;
      if (n.y < minY) minY = n.y;
      if (n.y > maxY) maxY = n.y;
    }

    const rangeX = maxX - minX || 1;
    const rangeY = maxY - minY || 1;
    const pad = 60;
    const scaleX = (w - pad * 2) / rangeX;
    const scaleY = (h - pad * 2) / rangeY;
    const scale = Math.min(scaleX, scaleY);

    const cx = (minX + maxX) / 2;
    const cy = (minY + maxY) / 2;

    for (const n of this.nodes) {
      n.x = (n.x - cx) * scale;
      n.y = (n.y - cy) * scale;
    }
  }

  setOnTick(cb: () => void): void {
    this.onTick = cb;
  }

  tick(): void {
    this.alpha += (this.alphaTarget - this.alpha) * this.opts.alphaDecay;

    this.applyChargeForce();
    this.applyLinkForce();
    this.applyCenterForce();
    this.applyCollisionForce();

    for (const node of this.nodes) {
      if (node.fx !== null) {
        node.x = node.fx;
        node.vx = 0;
      } else {
        node.vx *= (1 - this.opts.velocityDecay);
        node.x += node.vx;
      }
      if (node.fy !== null) {
        node.y = node.fy;
        node.vy = 0;
      } else {
        node.vy *= (1 - this.opts.velocityDecay);
        node.y += node.vy;
      }
    }
  }

  private applyChargeForce(): void {
    const strength = this.opts.chargeStrength;
    for (let i = 0; i < this.vcount; i++) {
      for (let j = i + 1; j < this.vcount; j++) {
        const ni = this.nodes[i]!;
        const nj = this.nodes[j]!;
        let dx = nj.x - ni.x;
        let dy = nj.y - ni.y;
        let dist2 = dx * dx + dy * dy;
        if (dist2 < 1) dist2 = 1;
        const dist = Math.sqrt(dist2);
        const force = (strength * this.alpha) / dist2;
        const fx = dx / dist * force;
        const fy = dy / dist * force;
        ni.vx -= fx;
        ni.vy -= fy;
        nj.vx += fx;
        nj.vy += fy;
      }
    }
  }

  private applyLinkForce(): void {
    const { linkDistance, linkStrength } = this.opts;
    for (const [u, v] of this.edges) {
      if (u >= this.vcount || v >= this.vcount) continue;
      const nu = this.nodes[u]!;
      const nv = this.nodes[v]!;
      let dx = nv.x - nu.x;
      let dy = nv.y - nu.y;
      const dist = Math.sqrt(dx * dx + dy * dy) || 1;
      const force = (dist - linkDistance) * linkStrength * this.alpha;
      dx = (dx / dist) * force;
      dy = (dy / dist) * force;
      nu.vx += dx;
      nu.vy += dy;
      nv.vx -= dx;
      nv.vy -= dy;
    }
  }

  private applyCenterForce(): void {
    const strength = this.opts.centerStrength;
    let cx = 0, cy = 0;
    for (const n of this.nodes) {
      cx += n.x;
      cy += n.y;
    }
    cx /= this.vcount || 1;
    cy /= this.vcount || 1;
    for (const n of this.nodes) {
      n.vx -= cx * strength;
      n.vy -= cy * strength;
    }
  }

  private applyCollisionForce(): void {
    const r = this.opts.collisionRadius;
    if (r <= 0) return;
    const r2 = (r * 2) * (r * 2);
    for (let i = 0; i < this.vcount; i++) {
      for (let j = i + 1; j < this.vcount; j++) {
        const ni = this.nodes[i]!;
        const nj = this.nodes[j]!;
        const dx = nj.x - ni.x;
        const dy = nj.y - ni.y;
        const dist2 = dx * dx + dy * dy;
        if (dist2 < r2 && dist2 > 0) {
          const dist = Math.sqrt(dist2);
          const minDist = r * 2;
          const overlap = (minDist - dist) / dist * 0.5;
          const ox = dx * overlap;
          const oy = dy * overlap;
          ni.vx -= ox;
          ni.vy -= oy;
          nj.vx += ox;
          nj.vy += oy;
        }
      }
    }
  }

  start(): void {
    if (this.running) return;
    this.running = true;
    const loop = () => {
      if (!this.running) return;
      this.tick();
      this.onTick?.();
      if (this.alpha > this.opts.alphaMin || this.alphaTarget > 0) {
        this.frameId = requestAnimationFrame(loop);
      } else {
        this.running = false;
        this.frameId = null;
      }
    };
    this.frameId = requestAnimationFrame(loop);
  }

  stop(): void {
    this.running = false;
    if (this.frameId !== null) {
      cancelAnimationFrame(this.frameId);
      this.frameId = null;
    }
  }

  reheat(alpha = 0.3): void {
    this.alpha = Math.max(this.alpha, alpha);
    this.start();
  }

  pinNode(index: number, x: number, y: number): void {
    const node = this.nodes[index];
    if (!node) return;
    node.fx = x;
    node.fy = y;
  }

  unpinNode(index: number): void {
    const node = this.nodes[index];
    if (!node) return;
    node.fx = null;
    node.fy = null;
  }

  isRunning(): boolean {
    return this.running;
  }

  getCoords(): [number, number][] {
    return this.nodes.map((n) => [n.x, n.y]);
  }

  destroy(): void {
    this.stop();
    this.onTick = null;
  }
}

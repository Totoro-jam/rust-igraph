import type { PresetGraph } from './types';

export const PRESETS: Record<string, PresetGraph> = {
  karate: {
    id: 'karate',
    directed: false,
    edges: [
      [0,1],[0,2],[0,3],[0,4],[0,5],[0,6],[0,7],[0,8],[0,10],[0,11],[0,12],[0,13],[0,17],[0,19],[0,21],[0,31],
      [1,2],[1,3],[1,7],[1,13],[1,17],[1,19],[1,21],[1,30],
      [2,3],[2,7],[2,8],[2,9],[2,13],[2,27],[2,28],[2,32],
      [3,7],[3,12],[3,13],
      [4,6],[4,10],
      [5,6],[5,10],[5,16],
      [6,16],
      [8,30],[8,32],[8,33],
      [9,33],
      [13,33],
      [14,32],[14,33],
      [15,32],[15,33],
      [18,32],[18,33],
      [19,33],
      [20,32],[20,33],
      [22,32],[22,33],
      [23,25],[23,27],[23,29],[23,32],[23,33],
      [24,25],[24,27],[24,31],
      [25,31],
      [26,29],[26,33],
      [27,33],
      [28,31],[28,33],
      [29,32],[29,33],
      [30,32],[30,33],
      [31,32],[31,33],
      [32,33],
    ],
  },
  petersen: {
    id: 'petersen',
    directed: false,
    edges: [
      [0,1],[1,2],[2,3],[3,4],[4,0],
      [5,7],[7,9],[9,6],[6,8],[8,5],
      [0,5],[1,6],[2,7],[3,8],[4,9],
    ],
  },
  erdos_renyi: {
    id: 'erdos_renyi',
    directed: false,
    edges: (() => {
      const edges: [number, number][] = [];
      const seed = 42;
      let s = seed;
      for (let i = 0; i < 50; i++) {
        for (let j = i + 1; j < 50; j++) {
          s = (s * 1103515245 + 12345) & 0x7fffffff;
          if ((s % 1000) < 100) edges.push([i, j]);
        }
      }
      return edges;
    })(),
  },
  barabasi_albert: {
    id: 'barabasi_albert',
    directed: false,
    edges: (() => {
      const edges: [number, number][] = [[0,1],[1,2],[0,2]];
      const deg = new Array(50).fill(0);
      deg[0] = 2; deg[1] = 2; deg[2] = 2;
      let s = 42;
      for (let v = 3; v < 50; v++) {
        const targets = new Set<number>();
        const totalDeg = deg.reduce((a, b) => a + b, 0);
        while (targets.size < 2) {
          s = (s * 1103515245 + 12345) & 0x7fffffff;
          let r = s % totalDeg;
          for (let u = 0; u < v; u++) {
            r -= deg[u]!;
            if (r < 0) { targets.add(u); break; }
          }
        }
        for (const u of targets) {
          edges.push([v, u]);
          deg[v]!++;
          deg[u]!++;
        }
      }
      return edges;
    })(),
  },
  watts_strogatz: {
    id: 'watts_strogatz',
    directed: false,
    edges: (() => {
      const n = 30, k = 4;
      const edges: [number, number][] = [];
      for (let i = 0; i < n; i++) {
        for (let j = 1; j <= k / 2; j++) {
          edges.push([i, (i + j) % n]);
        }
      }
      let s = 42;
      for (let idx = 0; idx < edges.length; idx++) {
        s = (s * 1103515245 + 12345) & 0x7fffffff;
        if ((s % 100) < 10) {
          const [u] = edges[idx]!;
          s = (s * 1103515245 + 12345) & 0x7fffffff;
          const newV = s % n;
          if (newV !== u) edges[idx] = [u, newV];
        }
      }
      return edges;
    })(),
  },
  small_triangle: {
    id: 'small_triangle',
    directed: false,
    edges: [
      [0,1],[1,2],[2,3],[3,0],[2,4],[4,5],[5,3],
    ],
  },
  directed_dag: {
    id: 'directed_dag',
    directed: true,
    edges: [
      [0,1],[0,2],[1,3],[1,4],[2,4],[2,5],[3,6],[4,6],[5,6],
    ],
  },
  complete_k8: {
    id: 'complete_k8',
    directed: false,
    edges: (() => {
      const edges: [number, number][] = [];
      for (let i = 0; i < 8; i++)
        for (let j = i + 1; j < 8; j++)
          edges.push([i, j]);
      return edges;
    })(),
  },
  cycle_20: {
    id: 'cycle_20',
    directed: false,
    edges: (() => {
      const n = 20;
      const edges: [number, number][] = [];
      for (let i = 0; i < n; i++) edges.push([i, (i + 1) % n]);
      return edges;
    })(),
  },
  star_12: {
    id: 'star_12',
    directed: false,
    edges: (() => {
      const edges: [number, number][] = [];
      for (let i = 1; i < 12; i++) edges.push([0, i]);
      return edges;
    })(),
  },
  grid_5x5: {
    id: 'grid_5x5',
    directed: false,
    edges: (() => {
      const w = 5, h = 5;
      const edges: [number, number][] = [];
      for (let r = 0; r < h; r++)
        for (let c = 0; c < w; c++) {
          const id = r * w + c;
          if (c + 1 < w) edges.push([id, id + 1]);
          if (r + 1 < h) edges.push([id, id + w]);
        }
      return edges;
    })(),
  },
  binary_tree: {
    id: 'binary_tree',
    directed: false,
    edges: (() => {
      const edges: [number, number][] = [];
      for (let i = 0; i < 15; i++) {
        const left = 2 * i + 1;
        const right = 2 * i + 2;
        if (left < 15) edges.push([i, left]);
        if (right < 15) edges.push([i, right]);
      }
      return edges;
    })(),
  },
};

export const PRESET_ORDER = [
  'karate',
  'petersen',
  'erdos_renyi',
  'barabasi_albert',
  'watts_strogatz',
  'small_triangle',
  'directed_dag',
  'complete_k8',
  'cycle_20',
  'star_12',
  'grid_5x5',
  'binary_tree',
] as const;

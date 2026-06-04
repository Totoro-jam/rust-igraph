// Playground — load WASM, run algorithms, render graph on canvas

// ── Theme ──────────────────────────────────────────────────────────
function getTheme() {
  return document.documentElement.getAttribute('data-theme') || 'dark';
}

function toggleTheme() {
  const next = getTheme() === 'dark' ? 'light' : 'dark';
  document.documentElement.setAttribute('data-theme', next);
  localStorage.setItem('theme', next);
  draw();
}
window.toggleTheme = toggleTheme;

(function restoreTheme() {
  const saved = localStorage.getItem('theme');
  if (saved) document.documentElement.setAttribute('data-theme', saved);
})();

// ── DOM refs ───────────────────────────────────────────────────────
const edgeInput   = document.getElementById('edge-input');
const directedCb  = document.getElementById('directed');
const algoSelect  = document.getElementById('algo-select');
const btnRun      = document.getElementById('btn-run');
const canvas      = document.getElementById('graph-canvas');
const ctx         = canvas.getContext('2d');
const outputEl    = document.getElementById('output');
const statusEl    = document.getElementById('status');

// ── State ──────────────────────────────────────────────────────────
let wasmModule = null;
let graphState = null;  // { coords, edges, vcount, result, algo }

// Community / heat-map palette (works on both themes)
const PALETTE = [
  '#58a6ff', '#3fb950', '#d2a8ff', '#f0883e',
  '#f778ba', '#a5d6ff', '#ffd33d', '#ff7b72',
];

// ── WASM loading ───────────────────────────────────────────────────
async function loadWasm() {
  statusEl.textContent = 'Loading WASM…';
  statusEl.className = 'status loading';
  try {
    // wasm-pack --target web output lives alongside the page at /playground/wasm/
    // Use a URL string so Vite/Rollup won't try to resolve it at build time.
    const wasmPath = './wasm/igraph_wasm.js';
    const wasmJsUrl = new URL(/* @vite-ignore */ wasmPath, import.meta.url).href;
    const mod = await import(/* @vite-ignore */ wasmJsUrl);
    await mod.default();   // init wasm
    wasmModule = mod;
    statusEl.textContent = 'Ready';
    statusEl.className = 'status ready';
    btnRun.disabled = false;
  } catch (err) {
    console.error('WASM load failed:', err);
    statusEl.textContent = 'WASM unavailable — running in demo mode';
    statusEl.className = 'status error';
    wasmModule = null;
    btnRun.disabled = false;
  }
}

// ── Edge parsing ───────────────────────────────────────────────────
function parseEdges(text) {
  const edges = [];
  for (const line of text.split('\n')) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith('#') || trimmed.startsWith('//')) continue;
    const parts = trimmed.split(/[\s,;]+/).map(Number);
    if (parts.length >= 2 && Number.isFinite(parts[0]) && Number.isFinite(parts[1])) {
      edges.push([parts[0], parts[1]]);
    }
  }
  return edges;
}

function getVcount(edges) {
  let max = -1;
  for (const [u, v] of edges) {
    if (u > max) max = u;
    if (v > max) max = v;
  }
  return max + 1;
}

// ── Fallback layout (Fruchterman-Reingold in JS, for demo mode) ──
function layoutFrJS(vcount, edges, niter) {
  const W = 1, H = 1;
  const area = W * H;
  const k = Math.sqrt(area / Math.max(vcount, 1));
  const coords = Array.from({ length: vcount }, () => [Math.random(), Math.random()]);

  for (let iter = 0; iter < niter; iter++) {
    const temp = (1 - iter / niter) * 0.1;
    const disp = coords.map(() => [0, 0]);

    // Repulsion
    for (let i = 0; i < vcount; i++) {
      for (let j = i + 1; j < vcount; j++) {
        let dx = coords[i][0] - coords[j][0];
        let dy = coords[i][1] - coords[j][1];
        const dist = Math.sqrt(dx * dx + dy * dy) || 0.001;
        const force = (k * k) / dist;
        const fx = (dx / dist) * force;
        const fy = (dy / dist) * force;
        disp[i][0] += fx; disp[i][1] += fy;
        disp[j][0] -= fx; disp[j][1] -= fy;
      }
    }

    // Attraction
    for (const [u, v] of edges) {
      if (u >= vcount || v >= vcount) continue;
      let dx = coords[u][0] - coords[v][0];
      let dy = coords[u][1] - coords[v][1];
      const dist = Math.sqrt(dx * dx + dy * dy) || 0.001;
      const force = (dist * dist) / k;
      const fx = (dx / dist) * force;
      const fy = (dy / dist) * force;
      disp[u][0] -= fx; disp[u][1] -= fy;
      disp[v][0] += fx; disp[v][1] += fy;
    }

    // Apply
    for (let i = 0; i < vcount; i++) {
      const dx = disp[i][0], dy = disp[i][1];
      const dist = Math.sqrt(dx * dx + dy * dy) || 0.001;
      const cap = Math.min(dist, temp);
      coords[i][0] += (dx / dist) * cap;
      coords[i][1] += (dy / dist) * cap;
      coords[i][0] = Math.max(0.05, Math.min(0.95, coords[i][0]));
      coords[i][1] = Math.max(0.05, Math.min(0.95, coords[i][1]));
    }
  }
  return coords;
}

// ── Fallback algorithms (demo mode — no WASM) ─────────────────────
function demoBfs(vcount, edges) {
  const adj = Array.from({ length: vcount }, () => []);
  for (const [u, v] of edges) {
    if (u < vcount && v < vcount) { adj[u].push(v); adj[v].push(u); }
  }
  const visited = new Set();
  const order = [];
  const queue = [0];
  visited.add(0);
  while (queue.length > 0) {
    const v = queue.shift();
    order.push(v);
    for (const w of adj[v]) {
      if (!visited.has(w)) { visited.add(w); queue.push(w); }
    }
  }
  return { order };
}

function demoPagerank(vcount, edges, damping = 0.85, iters = 100) {
  const scores = new Float64Array(vcount).fill(1 / vcount);
  const outDeg = new Float64Array(vcount);
  const adj = Array.from({ length: vcount }, () => []);
  for (const [u, v] of edges) {
    if (u < vcount && v < vcount) { adj[v].push(u); outDeg[u]++; }
  }
  for (let t = 0; t < iters; t++) {
    const next = new Float64Array(vcount).fill((1 - damping) / vcount);
    for (let v = 0; v < vcount; v++) {
      for (const u of adj[v]) {
        if (outDeg[u] > 0) next[v] += damping * scores[u] / outDeg[u];
      }
    }
    scores.set(next);
  }
  return { scores: Array.from(scores) };
}

function demoComponents(vcount, edges) {
  const parent = Array.from({ length: vcount }, (_, i) => i);
  function find(x) { while (parent[x] !== x) { parent[x] = parent[parent[x]]; x = parent[x]; } return x; }
  function union(a, b) { parent[find(a)] = find(b); }
  for (const [u, v] of edges) {
    if (u < vcount && v < vcount) union(u, v);
  }
  const membership = parent.map(find);
  const ids = new Set(membership);
  const remap = new Map();
  let idx = 0;
  for (const id of ids) remap.set(id, idx++);
  return { membership: membership.map(m => remap.get(m)), count: ids.size };
}

function demoBetweenness(vcount, edges) {
  const adj = Array.from({ length: vcount }, () => []);
  for (const [u, v] of edges) {
    if (u < vcount && v < vcount) { adj[u].push(v); adj[v].push(u); }
  }
  const cb = new Float64Array(vcount);
  for (let s = 0; s < vcount; s++) {
    const stack = [];
    const pred = Array.from({ length: vcount }, () => []);
    const sigma = new Float64Array(vcount);
    sigma[s] = 1;
    const dist = new Int32Array(vcount).fill(-1);
    dist[s] = 0;
    const queue = [s];
    while (queue.length > 0) {
      const v = queue.shift();
      stack.push(v);
      for (const w of adj[v]) {
        if (dist[w] < 0) { queue.push(w); dist[w] = dist[v] + 1; }
        if (dist[w] === dist[v] + 1) { sigma[w] += sigma[v]; pred[w].push(v); }
      }
    }
    const delta = new Float64Array(vcount);
    while (stack.length > 0) {
      const w = stack.pop();
      for (const v of pred[w]) {
        delta[v] += (sigma[v] / sigma[w]) * (1 + delta[w]);
      }
      if (w !== s) cb[w] += delta[w];
    }
  }
  return { scores: Array.from(cb) };
}

function demoLouvain(vcount, edges) {
  // Simplified: just return connected components as communities
  const result = demoComponents(vcount, edges);
  return { membership: result.membership, modularity: 0 };
}

// ── Run algorithm ──────────────────────────────────────────────────
async function run() {
  const edges = parseEdges(edgeInput.value);
  const vcount = getVcount(edges);
  if (vcount === 0) {
    outputEl.textContent = 'No valid edges. Enter edges as "u v" per line.';
    return;
  }

  const algo = algoSelect.value;
  const directed = directedCb.checked;
  const edgesFlat = new Uint32Array(edges.length * 2);
  edges.forEach(([u, v], i) => { edgesFlat[i * 2] = u; edgesFlat[i * 2 + 1] = v; });

  let result = null;
  let coords = null;

  btnRun.disabled = true;
  statusEl.textContent = 'Running…';
  statusEl.className = 'status loading';

  try {
    if (wasmModule) {
      // WASM path
      const g = wasmModule.WasmGraph.fromEdges(edgesFlat, directed);
      const layoutJson = g.layoutFr(500);
      coords = JSON.parse(layoutJson).coords;

      let raw;
      switch (algo) {
        case 'pagerank':    raw = g.pagerank(); break;
        case 'louvain':     raw = g.louvain(); break;
        case 'betweenness': raw = g.betweenness(); break;
        case 'bfs':         raw = g.bfs(0); break;
        case 'components':  raw = g.connectedComponents(); break;
      }
      result = JSON.parse(raw);
      g.free();
    } else {
      // Demo fallback (pure JS)
      coords = layoutFrJS(vcount, edges, 300);

      switch (algo) {
        case 'pagerank':    result = demoPagerank(vcount, edges); break;
        case 'louvain':     result = demoLouvain(vcount, edges); break;
        case 'betweenness': result = demoBetweenness(vcount, edges); break;
        case 'bfs':         result = demoBfs(vcount, edges); break;
        case 'components':  result = demoComponents(vcount, edges); break;
      }
    }

    graphState = { coords, edges, vcount, result, algo };
    draw();
    formatOutput(algo, result, vcount);

    statusEl.textContent = wasmModule ? 'Ready' : 'Demo mode (no WASM)';
    statusEl.className = wasmModule ? 'status ready' : 'status error';
  } catch (err) {
    outputEl.textContent = 'Error: ' + err.message;
    statusEl.textContent = 'Error';
    statusEl.className = 'status error';
    console.error(err);
  } finally {
    btnRun.disabled = false;
  }
}

// ── Format text output ─────────────────────────────────────────────
function formatOutput(algo, result, vcount) {
  let lines = [];
  switch (algo) {
    case 'pagerank':
      lines.push('PageRank scores:');
      result.scores.forEach((s, i) => lines.push(`  vertex ${i}: ${s.toFixed(6)}`));
      break;
    case 'louvain':
      lines.push(`Louvain communities (modularity: ${result.modularity.toFixed(4)}):`);
      result.membership.forEach((c, i) => lines.push(`  vertex ${i}: community ${c}`));
      break;
    case 'betweenness':
      lines.push('Betweenness centrality:');
      result.scores.forEach((s, i) => lines.push(`  vertex ${i}: ${s.toFixed(4)}`));
      break;
    case 'bfs':
      lines.push('BFS traversal order (from vertex 0):');
      lines.push('  ' + result.order.join(' → '));
      break;
    case 'components':
      lines.push(`Connected components: ${result.count}`);
      result.membership.forEach((c, i) => lines.push(`  vertex ${i}: component ${c}`));
      break;
  }
  outputEl.textContent = lines.join('\n');
}

// ── Canvas rendering ───────────────────────────────────────────────
function resizeCanvas() {
  const rect = canvas.parentElement.getBoundingClientRect();
  const headerH = canvas.parentElement.querySelector('.panel-header')?.offsetHeight || 0;
  const outputH = outputEl?.offsetHeight || 0;
  const dpr = window.devicePixelRatio || 1;
  const w = rect.width;
  const h = rect.height - headerH - outputH;

  canvas.width = w * dpr;
  canvas.height = Math.max(h, 100) * dpr;
  canvas.style.width = w + 'px';
  canvas.style.height = Math.max(h, 100) + 'px';
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
}

function getNodeColor(algo, result, idx, vcount) {
  if (!result) return PALETTE[0];
  switch (algo) {
    case 'louvain':
    case 'components':
      return PALETTE[(result.membership?.[idx] || 0) % PALETTE.length];
    case 'pagerank': {
      const scores = result.scores || [];
      const max = Math.max(...scores, 1e-9);
      const min = Math.min(...scores);
      const t = max > min ? (scores[idx] - min) / (max - min) : 0.5;
      return interpolateColor(t);
    }
    case 'betweenness': {
      const scores = result.scores || [];
      const max = Math.max(...scores, 1e-9);
      const t = max > 0 ? scores[idx] / max : 0;
      return interpolateColor(t);
    }
    case 'bfs': {
      const order = result.order || [];
      const pos = order.indexOf(idx);
      if (pos < 0) return getTheme() === 'dark' ? '#30363d' : '#d0d7de';
      const t = order.length > 1 ? pos / (order.length - 1) : 0;
      return interpolateColor(1 - t);
    }
    default:
      return PALETTE[0];
  }
}

function interpolateColor(t) {
  // Blue (cold) → Green → Yellow → Red (hot)
  const r = Math.round(t < 0.5 ? 0 : (t - 0.5) * 2 * 255);
  const g = Math.round(t < 0.5 ? t * 2 * 200 : (1 - t) * 2 * 200);
  const b = Math.round(t < 0.5 ? 255 - t * 2 * 200 : 0);
  return `rgb(${r},${g},${b})`;
}

function getNodeRadius(algo, result, idx, vcount) {
  const base = Math.max(4, Math.min(14, 200 / Math.sqrt(Math.max(vcount, 1))));
  if (!result) return base;
  switch (algo) {
    case 'pagerank': {
      const scores = result.scores || [];
      const max = Math.max(...scores, 1e-9);
      return base * (0.6 + 1.4 * scores[idx] / max);
    }
    case 'betweenness': {
      const scores = result.scores || [];
      const max = Math.max(...scores, 1e-9);
      return base * (0.6 + 1.4 * (max > 0 ? scores[idx] / max : 0.5));
    }
    default:
      return base;
  }
}

function draw() {
  resizeCanvas();
  const w = canvas.width / (window.devicePixelRatio || 1);
  const h = canvas.height / (window.devicePixelRatio || 1);

  // Clear
  ctx.clearRect(0, 0, w, h);

  if (!graphState) {
    ctx.fillStyle = getTheme() === 'dark' ? '#8b949e' : '#57606a';
    ctx.font = '14px ' + getComputedStyle(document.body).fontFamily;
    ctx.textAlign = 'center';
    ctx.fillText('Click "Run" to visualize the graph', w / 2, h / 2);
    return;
  }

  const { coords, edges, vcount, result, algo } = graphState;
  const pad = 40;
  const drawW = w - pad * 2;
  const drawH = h - pad * 2;

  // Map coords to canvas space
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
  ]);

  // Draw edges
  const edgeColor = getTheme() === 'dark' ? 'rgba(88,166,255,0.2)' : 'rgba(9,105,218,0.2)';
  ctx.strokeStyle = edgeColor;
  ctx.lineWidth = 1;
  const directed = directedCb.checked;

  for (const [u, v] of edges) {
    if (u >= vcount || v >= vcount) continue;
    const [x1, y1] = screenCoords[u];
    const [x2, y2] = screenCoords[v];
    ctx.beginPath();
    ctx.moveTo(x1, y1);
    ctx.lineTo(x2, y2);
    ctx.stroke();

    if (directed) {
      // Arrowhead
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

  // Draw nodes
  for (let i = 0; i < vcount; i++) {
    const [x, y] = screenCoords[i];
    const r = getNodeRadius(algo, result, i, vcount);
    const color = getNodeColor(algo, result, i, vcount);

    // Glow
    ctx.beginPath();
    ctx.arc(x, y, r + 3, 0, Math.PI * 2);
    ctx.fillStyle = color.replace('rgb', 'rgba').replace(')', ',0.15)');
    ctx.fill();

    // Node
    ctx.beginPath();
    ctx.arc(x, y, r, 0, Math.PI * 2);
    ctx.fillStyle = color;
    ctx.fill();

    // Label
    if (vcount <= 50) {
      ctx.fillStyle = getTheme() === 'dark' ? '#e6edf3' : '#24292f';
      ctx.font = `bold ${Math.max(9, r * 0.9)}px ${getComputedStyle(document.body).fontFamily}`;
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      ctx.fillText(String(i), x, y);
    }
  }
}

// ── Event wiring ───────────────────────────────────────────────────
btnRun.addEventListener('click', run);

edgeInput.addEventListener('keydown', (e) => {
  if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
    e.preventDefault();
    run();
  }
});

window.addEventListener('resize', () => { if (graphState) draw(); });

new ResizeObserver(() => { if (graphState) draw(); }).observe(canvas.parentElement);

// Auto-run on load once WASM is ready
loadWasm().then(() => run());

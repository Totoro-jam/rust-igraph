// Theme toggle
function getPreferredTheme() {
  const stored = localStorage.getItem('theme');
  if (stored) return stored;
  return window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
}

function applyTheme(theme) {
  document.documentElement.setAttribute('data-theme', theme);
  localStorage.setItem('theme', theme);
}

window.toggleTheme = function () {
  const current = document.documentElement.getAttribute('data-theme');
  applyTheme(current === 'dark' ? 'light' : 'dark');
};

applyTheme(getPreferredTheme());

// Hero canvas: animated force-directed graph
(function () {
  const canvas = document.getElementById('hero-canvas');
  if (!canvas) return;
  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  let width, height;
  const nodes = [];
  const edges = [];
  const N = 40;

  function resize() {
    const rect = canvas.parentElement.getBoundingClientRect();
    width = rect.width;
    height = rect.height;
    canvas.width = width * devicePixelRatio;
    canvas.height = height * devicePixelRatio;
    canvas.style.width = width + 'px';
    canvas.style.height = height + 'px';
    ctx.setTransform(devicePixelRatio, 0, 0, devicePixelRatio, 0, 0);
  }

  function init() {
    resize();
    for (let i = 0; i < N; i++) {
      nodes.push({
        x: Math.random() * width,
        y: Math.random() * height,
        vx: (Math.random() - 0.5) * 0.3,
        vy: (Math.random() - 0.5) * 0.3,
        r: 2 + Math.random() * 2,
      });
    }
    for (let i = 0; i < N; i++) {
      const numEdges = 1 + Math.floor(Math.random() * 2);
      for (let e = 0; e < numEdges; e++) {
        const j = Math.floor(Math.random() * N);
        if (j !== i) edges.push([i, j]);
      }
    }
  }

  function draw() {
    ctx.clearRect(0, 0, width, height);
    const accent = getComputedStyle(document.documentElement)
      .getPropertyValue('--accent')
      .trim();

    // Edges
    ctx.strokeStyle = accent;
    ctx.globalAlpha = 0.15;
    ctx.lineWidth = 1;
    for (const [i, j] of edges) {
      ctx.beginPath();
      ctx.moveTo(nodes[i].x, nodes[i].y);
      ctx.lineTo(nodes[j].x, nodes[j].y);
      ctx.stroke();
    }

    // Nodes
    ctx.globalAlpha = 0.4;
    ctx.fillStyle = accent;
    for (const node of nodes) {
      ctx.beginPath();
      ctx.arc(node.x, node.y, node.r, 0, Math.PI * 2);
      ctx.fill();
    }
    ctx.globalAlpha = 1;
  }

  function step() {
    for (const node of nodes) {
      node.x += node.vx;
      node.y += node.vy;
      if (node.x < 0 || node.x > width) node.vx *= -1;
      if (node.y < 0 || node.y > height) node.vy *= -1;
      node.x = Math.max(0, Math.min(width, node.x));
      node.y = Math.max(0, Math.min(height, node.y));
    }
    draw();
    requestAnimationFrame(step);
  }

  init();
  step();
  window.addEventListener('resize', resize);
})();

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

// Mobile nav: close on link click
document.querySelectorAll('.nav-links a').forEach(function (a) {
  a.addEventListener('click', function () {
    document.querySelector('.nav').classList.remove('nav-open');
  });
});

// Hero canvas: animated force-directed graph with community coloring
(function () {
  const canvas = document.getElementById('hero-canvas');
  if (!canvas) return;
  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  let width, height;
  const nodes = [];
  const edges = [];
  const N = 60;
  const COMMUNITY_COLORS_DARK = ['#58a6ff', '#f778ba', '#7ee787', '#d2a8ff', '#ffa657'];
  const COMMUNITY_COLORS_LIGHT = ['#0969da', '#bf3989', '#1a7f37', '#8250df', '#bc4c00'];

  function getColors() {
    const theme = document.documentElement.getAttribute('data-theme');
    return theme === 'light' ? COMMUNITY_COLORS_LIGHT : COMMUNITY_COLORS_DARK;
  }

  function resize() {
    const rect = canvas.parentElement.getBoundingClientRect();
    const oldW = width;
    const oldH = height;
    width = rect.width;
    height = rect.height;
    canvas.width = width * devicePixelRatio;
    canvas.height = height * devicePixelRatio;
    canvas.style.width = width + 'px';
    canvas.style.height = height + 'px';
    ctx.setTransform(devicePixelRatio, 0, 0, devicePixelRatio, 0, 0);

    if (oldW && oldH && nodes.length > 0) {
      const sx = width / oldW;
      const sy = height / oldH;
      for (const node of nodes) {
        node.x *= sx;
        node.y *= sy;
      }
    }
  }

  function init() {
    resize();
    const communities = 5;
    const centersX = [0.2, 0.5, 0.8, 0.35, 0.65];
    const centersY = [0.3, 0.6, 0.35, 0.7, 0.55];

    for (let i = 0; i < N; i++) {
      const c = i % communities;
      const spread = 0.15;
      nodes.push({
        x: (centersX[c] + (Math.random() - 0.5) * spread) * width,
        y: (centersY[c] + (Math.random() - 0.5) * spread) * height,
        vx: (Math.random() - 0.5) * 0.25,
        vy: (Math.random() - 0.5) * 0.25,
        r: 2.5 + Math.random() * 2.5,
        community: c,
      });
    }

    for (let i = 0; i < N; i++) {
      const intra = 1 + Math.floor(Math.random() * 2);
      for (let e = 0; e < intra; e++) {
        const sameComm = nodes.map((n, idx) => idx).filter(idx => idx !== i && nodes[idx].community === nodes[i].community);
        if (sameComm.length > 0) {
          const j = sameComm[Math.floor(Math.random() * sameComm.length)];
          edges.push([i, j]);
        }
      }
      if (Math.random() < 0.15) {
        const j = Math.floor(Math.random() * N);
        if (j !== i) edges.push([i, j]);
      }
    }
  }

  function draw() {
    ctx.clearRect(0, 0, width, height);
    const colors = getColors();

    ctx.lineWidth = 1;
    for (const [i, j] of edges) {
      const sameComm = nodes[i].community === nodes[j].community;
      ctx.strokeStyle = sameComm ? colors[nodes[i].community] : colors[nodes[i].community];
      ctx.globalAlpha = sameComm ? 0.2 : 0.07;
      ctx.beginPath();
      ctx.moveTo(nodes[i].x, nodes[i].y);
      ctx.lineTo(nodes[j].x, nodes[j].y);
      ctx.stroke();
    }

    for (const node of nodes) {
      ctx.globalAlpha = 0.6;
      ctx.fillStyle = colors[node.community];
      ctx.beginPath();
      ctx.arc(node.x, node.y, node.r, 0, Math.PI * 2);
      ctx.fill();

      ctx.globalAlpha = 0.15;
      ctx.beginPath();
      ctx.arc(node.x, node.y, node.r + 3, 0, Math.PI * 2);
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

// Number counter animation
(function () {
  const observer = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
      if (!entry.isIntersecting) return;
      const el = entry.target;
      if (el.dataset.counted) return;
      el.dataset.counted = 'true';

      const text = el.textContent.trim();
      const match = text.match(/^([\d,]+)(\+?)$/);
      if (!match) return;

      const target = parseInt(match[1].replace(/,/g, ''), 10);
      const suffix = match[2];
      const duration = 1200;
      const start = performance.now();

      function tick(now) {
        const elapsed = now - start;
        const progress = Math.min(elapsed / duration, 1);
        const eased = 1 - Math.pow(1 - progress, 3);
        const current = Math.round(target * eased);
        el.textContent = current.toLocaleString() + suffix;
        if (progress < 1) requestAnimationFrame(tick);
      }

      el.textContent = '0' + suffix;
      requestAnimationFrame(tick);
    });
  }, { threshold: 0.5 });

  document.querySelectorAll('.stat-num').forEach(el => observer.observe(el));
})();

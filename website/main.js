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

// i18n
var I18N = {
  en: {
    'nav.playground': 'Playground',
    'nav.guide': 'Guide',
    'nav.api': 'API Docs',
    'hero.title': 'Graph Algorithms<br>in Pure Rust',
    'hero.sub': '1,297 APIs. Zero <code>unsafe</code>. One dependency. Runs natively and in the browser via WASM.',
    'hero.try': 'Try in Browser',
    'hero.start': 'Get Started',
    'stats.apis': 'Public APIs',
    'stats.algos': 'Algorithms',
    'stats.tests': 'Tests',
    'stats.unsafe': 'unsafe blocks',
    'stats.deps': 'Dependency',
    'code.title': 'Simple, Expressive API',
    'code.construct': 'Graph Construction & Analysis',
    'code.method': 'Method-style API',
    'features.title': 'Comprehensive Algorithm Coverage',
    'features.traversal': 'Traversal & Paths',
    'features.traversal_desc': 'BFS, DFS, Dijkstra, Bellman-Ford, A*, all-pairs shortest paths, widest paths, topological sort',
    'features.centrality': 'Centrality',
    'features.centrality_desc': 'PageRank, betweenness, closeness, eigenvector, HITS, Katz, harmonic, constraint — 15+ measures',
    'features.community': 'Community Detection',
    'features.community_desc': 'Louvain, Leiden, Infomap, Spinglass, label propagation, Walktrap, edge betweenness, fast greedy, fluid, Voronoi',
    'features.flow': 'Network Flow',
    'features.flow_desc': 'Max-flow (push-relabel), min-cut, Gomory-Hu tree, edge/vertex connectivity, disjoint paths',
    'features.iso': 'Isomorphism',
    'features.iso_desc': 'VF2 graph/subgraph matching, LAD subgraph, canonical labeling (BLISS), automorphism groups',
    'features.generators': 'Generators & Layout',
    'features.generators_desc': '30+ graph generators, 16 layout engines (FR, KK, DrL, Sugiyama, MDS, UMAP...), spatial algorithms',
    'compare.title': 'How Does It Compare?',
    'compare.scope': 'Algorithm scope',
    'compare.safety': 'Safety',
    'compare.zero_unsafe': 'Zero unsafe',
    'compare.minimal_unsafe': 'Minimal unsafe',
    'compare.c_bindings': 'C core + bindings',
    'compare.py_safe': 'Memory-safe (Python)',
    'compare.validation': 'Validation',
    'compare.cross': 'Cross-validated',
    'compare.independent': 'Independent tests',
    'compare.ref_impl': 'Reference impl',
    'compare.native': 'Native',
    'compare.na': 'Not available',
    'compare.deps': 'Dependencies',
    'compare.minimal': 'Minimal',
    'compare.c_toolchain': 'C/C++ build toolchain',
    'wasm.title': 'Runs in the Browser',
    'wasm.desc': 'Compile to WASM and run graph algorithms client-side — no server needed. Try it live in the playground.',
    'wasm.cta': 'Open Playground',
    'eco.title': 'Ecosystem',
    'eco.guide': 'Tutorial & Guide',
    'eco.api': 'API Reference',
    'footer.license': 'rust-igraph is licensed under <a href="https://github.com/Totoro-jam/rust-igraph/blob/main/LICENSE">GPL-2.0-or-later</a>.',
    'footer.ack': 'Built with Rust. Acknowledgements: <a href="https://igraph.org">igraph</a> (C core), <a href="https://github.com/igraph/python-igraph">python-igraph</a>, <a href="https://github.com/igraph/rigraph">rigraph</a>.',
  },
  zh: {
    'nav.playground': '演练场',
    'nav.guide': '教程',
    'nav.api': 'API 文档',
    'hero.title': '纯 Rust 图算法库',
    'hero.sub': '1,297 公开 API、零 <code>unsafe</code>、仅一个依赖。原生运行，也可通过 WASM 在浏览器中执行。',
    'hero.try': '在线体验',
    'hero.start': '开始使用',
    'stats.apis': '公开 API',
    'stats.algos': '算法',
    'stats.tests': '测试',
    'stats.unsafe': 'unsafe 块',
    'stats.deps': '依赖',
    'code.title': '简洁、富有表现力的 API',
    'code.construct': '图构建与分析',
    'code.method': '方法链式 API',
    'features.title': '全面的算法覆盖',
    'features.traversal': '遍历与路径',
    'features.traversal_desc': 'BFS、DFS、Dijkstra、Bellman-Ford、A*、全源最短路径、最宽路径、拓扑排序',
    'features.centrality': '中心性',
    'features.centrality_desc': 'PageRank、介数、接近、特征向量、HITS、Katz、谐波、约束—— 15+ 种指标',
    'features.community': '社区发现',
    'features.community_desc': 'Louvain、Leiden、Infomap、Spinglass、标签传播、Walktrap、边介数、快速贪婪、Fluid、Voronoi',
    'features.flow': '网络流',
    'features.flow_desc': '最大流 (push-relabel)、最小割、Gomory-Hu 树、边/点连通度、不相交路径',
    'features.iso': '图同构',
    'features.iso_desc': 'VF2 图/子图匹配、LAD 子图、规范标记 (BLISS)、自同构群',
    'features.generators': '生成器与布局',
    'features.generators_desc': '30+ 图生成器、16 种布局引擎 (FR, KK, DrL, Sugiyama, MDS, UMAP…)、空间算法',
    'compare.title': '对比其他库',
    'compare.scope': '算法规模',
    'compare.safety': '安全性',
    'compare.zero_unsafe': '零 unsafe',
    'compare.minimal_unsafe': '少量 unsafe',
    'compare.c_bindings': 'C 核心 + 绑定',
    'compare.py_safe': '内存安全 (Python)',
    'compare.validation': '验证方式',
    'compare.cross': '交叉验证',
    'compare.independent': '独立测试',
    'compare.ref_impl': '参考实现',
    'compare.native': '原生支持',
    'compare.na': '不可用',
    'compare.deps': '依赖',
    'compare.minimal': '极少',
    'compare.c_toolchain': 'C/C++ 构建工具链',
    'wasm.title': '在浏览器中运行',
    'wasm.desc': '编译为 WASM，在客户端直接运行图算法——无需服务器。立即在演练场中体验。',
    'wasm.cta': '打开演练场',
    'eco.title': '生态系统',
    'eco.guide': '教程与指南',
    'eco.api': 'API 参考',
    'footer.license': 'rust-igraph 基于 <a href="https://github.com/Totoro-jam/rust-igraph/blob/main/LICENSE">GPL-2.0-or-later</a> 许可证发布。',
    'footer.ack': '用 Rust 构建。致谢：<a href="https://igraph.org">igraph</a> (C 核心)、<a href="https://github.com/igraph/python-igraph">python-igraph</a>、<a href="https://github.com/igraph/rigraph">rigraph</a>。',
  },
};

var currentLang = localStorage.getItem('lang') || 'en';

function applyLang(lang) {
  currentLang = lang;
  localStorage.setItem('lang', lang);
  document.documentElement.setAttribute('lang', lang === 'zh' ? 'zh-CN' : 'en');
  var dict = I18N[lang] || I18N.en;
  document.querySelectorAll('[data-i18n]').forEach(function (el) {
    var key = el.getAttribute('data-i18n');
    if (dict[key] !== undefined) el.innerHTML = dict[key];
  });
  document.querySelectorAll('[data-i18n-html]').forEach(function (el) {
    var key = el.getAttribute('data-i18n-html');
    if (dict[key] !== undefined) el.innerHTML = dict[key];
  });
  var btn = document.querySelector('.lang-toggle');
  if (btn) btn.textContent = lang === 'zh' ? 'EN' : '中';
  document.querySelectorAll('a[data-i18n="nav.guide"], a[data-i18n="eco.guide"]').forEach(function (a) {
    a.href = lang === 'zh' ? 'book/zh/' : 'book/';
  });
}

window.toggleLang = function () {
  applyLang(currentLang === 'en' ? 'zh' : 'en');
};

applyLang(currentLang);

// Mobile nav: close on link click
document.querySelectorAll('.nav-links a').forEach(function (a) {
  a.addEventListener('click', function () {
    document.querySelector('.nav').classList.remove('nav-open');
  });
});

// ========== Hero canvas: Real force-directed graph simulation ==========
(function () {
  const canvas = document.getElementById('hero-canvas');
  if (!canvas) return;
  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  let width, height;
  const nodes = [];
  const edges = [];
  const N = 55;
  const COMMUNITIES = 5;

  // Physics constants
  const REPULSION = 3500;
  const SPRING_K = 0.003;
  const SPRING_L = 90;
  const GRAVITY = 0.005;
  const DAMPING = 0.88;
  const MAX_V = 2.5;
  const MARGIN = 0.05; // 5% soft margin
  const MOUSE_RADIUS = 120;
  const MOUSE_FORCE = 0.8;

  const COLORS_DARK = ['#7c8fff', '#f778ba', '#7ee787', '#a78bfa', '#ffa657'];
  const COLORS_LIGHT = ['#5563e8', '#bf3989', '#1a7f37', '#7c5ce8', '#bc4c00'];

  function getColors() {
    const theme = document.documentElement.getAttribute('data-theme');
    return theme === 'light' ? COLORS_LIGHT : COLORS_DARK;
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

  let mouseX = -1000, mouseY = -1000;

  function init() {
    resize();

    // Initialize nodes spread across the full canvas with community-based clustering
    const centersX = [0.15, 0.5, 0.85, 0.25, 0.75];
    const centersY = [0.25, 0.7, 0.25, 0.8, 0.55];

    for (let i = 0; i < N; i++) {
      const c = i % COMMUNITIES;
      const spread = 0.22;
      const x = (centersX[c] + (Math.random() - 0.5) * spread) * width;
      const y = (centersY[c] + (Math.random() - 0.5) * spread) * height;
      nodes.push({
        x: Math.max(20, Math.min(width - 20, x)),
        y: Math.max(20, Math.min(height - 20, y)),
        vx: (Math.random() - 0.5) * 0.5,
        vy: (Math.random() - 0.5) * 0.5,
        r: 2.5 + Math.random() * 2,
        community: c,
      });
    }

    // Create edges: intra-community dense, inter-community sparse
    for (let i = 0; i < N; i++) {
      const intra = 1 + Math.floor(Math.random() * 2);
      for (let e = 0; e < intra; e++) {
        const sameComm = [];
        for (let j = 0; j < N; j++) {
          if (j !== i && nodes[j].community === nodes[i].community) sameComm.push(j);
        }
        if (sameComm.length > 0) {
          const j = sameComm[Math.floor(Math.random() * sameComm.length)];
          if (!edges.some(([a, b]) => (a === i && b === j) || (a === j && b === i))) {
            edges.push([i, j]);
          }
        }
      }
      if (Math.random() < 0.12) {
        const j = Math.floor(Math.random() * N);
        if (j !== i && !edges.some(([a, b]) => (a === i && b === j) || (a === j && b === i))) {
          edges.push([i, j]);
        }
      }
    }
  }

  function applyForces() {
    const minX = width * MARGIN;
    const maxX = width * (1 - MARGIN);
    const minY = height * MARGIN;
    const maxY = height * (1 - MARGIN);

    // Repulsive forces between all pairs
    for (let i = 0; i < N; i++) {
      for (let j = i + 1; j < N; j++) {
        let dx = nodes[j].x - nodes[i].x;
        let dy = nodes[j].y - nodes[i].y;
        let dist = Math.sqrt(dx * dx + dy * dy);
        if (dist < 1) dist = 1;
        const force = REPULSION / (dist * dist);
        const fx = (dx / dist) * force;
        const fy = (dy / dist) * force;
        nodes[i].vx -= fx;
        nodes[i].vy -= fy;
        nodes[j].vx += fx;
        nodes[j].vy += fy;
      }
    }

    // Spring forces along edges
    for (const [i, j] of edges) {
      const dx = nodes[j].x - nodes[i].x;
      const dy = nodes[j].y - nodes[i].y;
      let dist = Math.sqrt(dx * dx + dy * dy);
      if (dist < 1) dist = 1;
      const force = SPRING_K * (dist - SPRING_L);
      const fx = (dx / dist) * force;
      const fy = (dy / dist) * force;
      nodes[i].vx += fx;
      nodes[i].vy += fy;
      nodes[j].vx -= fx;
      nodes[j].vy -= fy;
    }

    // Gravity toward community centers (keeps clusters apart)
    const commCentersX = [0.15, 0.5, 0.85, 0.25, 0.75];
    const commCentersY = [0.25, 0.7, 0.25, 0.8, 0.55];
    for (let i = 0; i < N; i++) {
      const c = nodes[i].community;
      const targetX = commCentersX[c] * width;
      const targetY = commCentersY[c] * height;
      const dx = targetX - nodes[i].x;
      const dy = targetY - nodes[i].y;
      nodes[i].vx += dx * GRAVITY;
      nodes[i].vy += dy * GRAVITY;
    }

    // Mouse repulsion
    if (mouseX > 0 && mouseY > 0) {
      for (let i = 0; i < N; i++) {
        const dx = nodes[i].x - mouseX;
        const dy = nodes[i].y - mouseY;
        const dist = Math.sqrt(dx * dx + dy * dy);
        if (dist < MOUSE_RADIUS && dist > 1) {
          const strength = MOUSE_FORCE * (1 - dist / MOUSE_RADIUS);
          nodes[i].vx += (dx / dist) * strength;
          nodes[i].vy += (dy / dist) * strength;
        }
      }
    }

    // Update positions with damping and velocity cap
    for (let i = 0; i < N; i++) {
      nodes[i].vx *= DAMPING;
      nodes[i].vy *= DAMPING;

      const speed = Math.sqrt(nodes[i].vx * nodes[i].vx + nodes[i].vy * nodes[i].vy);
      if (speed > MAX_V) {
        nodes[i].vx = (nodes[i].vx / speed) * MAX_V;
        nodes[i].vy = (nodes[i].vy / speed) * MAX_V;
      }

      nodes[i].x += nodes[i].vx;
      nodes[i].y += nodes[i].vy;

      // Soft boundary constraint
      if (nodes[i].x < minX) { nodes[i].x = minX; nodes[i].vx *= -0.3; }
      if (nodes[i].x > maxX) { nodes[i].x = maxX; nodes[i].vx *= -0.3; }
      if (nodes[i].y < minY) { nodes[i].y = minY; nodes[i].vy *= -0.3; }
      if (nodes[i].y > maxY) { nodes[i].y = maxY; nodes[i].vy *= -0.3; }
    }
  }

  function draw() {
    ctx.clearRect(0, 0, width, height);
    const colors = getColors();

    // Draw edges with gradient
    ctx.lineWidth = 1.2;
    for (const [i, j] of edges) {
      const sameComm = nodes[i].community === nodes[j].community;
      const grad = ctx.createLinearGradient(nodes[i].x, nodes[i].y, nodes[j].x, nodes[j].y);
      const colorI = colors[nodes[i].community];
      const colorJ = colors[nodes[j].community];
      const alpha = sameComm ? 0.25 : 0.08;
      grad.addColorStop(0, colorI);
      grad.addColorStop(1, colorJ);
      ctx.strokeStyle = grad;
      ctx.globalAlpha = alpha;
      ctx.beginPath();
      ctx.moveTo(nodes[i].x, nodes[i].y);
      ctx.lineTo(nodes[j].x, nodes[j].y);
      ctx.stroke();
    }

    // Draw nodes with glow
    for (const node of nodes) {
      const color = colors[node.community];

      // Outer glow
      ctx.globalAlpha = 0.12;
      ctx.fillStyle = color;
      ctx.beginPath();
      ctx.arc(node.x, node.y, node.r + 4, 0, Math.PI * 2);
      ctx.fill();

      // Core
      ctx.globalAlpha = 0.7;
      ctx.fillStyle = color;
      ctx.beginPath();
      ctx.arc(node.x, node.y, node.r, 0, Math.PI * 2);
      ctx.fill();

      // Inner highlight
      ctx.globalAlpha = 0.3;
      ctx.fillStyle = '#fff';
      ctx.beginPath();
      ctx.arc(node.x - node.r * 0.25, node.y - node.r * 0.25, node.r * 0.4, 0, Math.PI * 2);
      ctx.fill();
    }
    ctx.globalAlpha = 1;
  }

  let animating = true;
  let rafId = null;

  function step() {
    if (!animating) return;
    applyForces();
    draw();
    rafId = requestAnimationFrame(step);
  }

  // Toggle button
  const toggleBtn = document.getElementById('hero-toggle');
  const iconPause = toggleBtn ? toggleBtn.querySelector('.icon-pause') : null;
  const iconPlay = toggleBtn ? toggleBtn.querySelector('.icon-play') : null;

  function setAnimating(val) {
    animating = val;
    localStorage.setItem('hero-animation', val ? 'on' : 'off');
    if (iconPause && iconPlay) {
      iconPause.style.display = val ? '' : 'none';
      iconPlay.style.display = val ? 'none' : '';
    }
    if (val && !rafId) step();
  }

  if (toggleBtn) {
    toggleBtn.addEventListener('click', function () {
      setAnimating(!animating);
    });
  }

  // Check stored preference and reduced-motion
  const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  const storedPref = localStorage.getItem('hero-animation');

  init();

  if (reducedMotion || storedPref === 'off') {
    // Run physics for equilibrium (no animation loop)
    for (let i = 0; i < 150; i++) applyForces();
    draw();
    animating = false;
    if (iconPause && iconPlay) {
      iconPause.style.display = 'none';
      iconPlay.style.display = '';
    }
  } else {
    step();
  }

  canvas.addEventListener('mousemove', function (e) {
    const rect = canvas.getBoundingClientRect();
    mouseX = e.clientX - rect.left;
    mouseY = e.clientY - rect.top;
  });

  canvas.addEventListener('mouseleave', function () {
    mouseX = -1000;
    mouseY = -1000;
  });

  window.addEventListener('resize', function () {
    resize();
    if (!animating) draw();
  });

  document.addEventListener('visibilitychange', function () {
    if (document.hidden) {
      animating = false;
      if (rafId) { cancelAnimationFrame(rafId); rafId = null; }
    } else if (storedPref !== 'off' && !reducedMotion) {
      animating = true;
      step();
    }
  });
})();

// ========== Number counter animation ==========
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
      const duration = 1400;
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

// ========== Scroll reveal animations ==========
(function () {
  if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return;

  const observer = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
      if (entry.isIntersecting) {
        entry.target.classList.add('visible');
      }
    });
  }, { threshold: 0.15, rootMargin: '0px 0px -40px 0px' });

  document.querySelectorAll('.reveal, .reveal-stagger').forEach(el => {
    observer.observe(el);
  });
})();

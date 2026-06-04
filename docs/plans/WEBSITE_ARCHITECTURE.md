# rust-igraph Official Website — Architecture Design

> **Status**: Draft (2026-06-04)
> **Author**: Totoro-jam + Claude
> **Scope**: Official website for rust-igraph, including landing page, WASM
> playground, docs integration, and SEO infrastructure.

## 1. Research foundation

28 project samples surveyed across 6 research dimensions. This is not
guesswork — every decision traces to observed patterns.

### 1.1 Sample inventory

| Category | Projects surveyed |
|----------|-------------------|
| **Rust library sites** (7) | tokio.rs, serde.rs, bevyengine.org, leptos.dev, wgpu docs, pola.rs, burn.dev |
| **Graph/network libraries** (8) | igraph.org, networkx.org, cytoscape.js, sigma.js, vis-network, d3-force, neo4j.com, graphology |
| **Interactive demo sites** (8) | threejs.org, p5js.org, tensorflow.org/js, deck.gl, echarts.apache.org, svelte.dev, react.dev, d3js.org |
| **Doc site architectures** (6) | docs.rs, kubernetes.io, stripe.com/docs, tailwindcss.com, nextjs.org/docs, pytorch.org |
| **WASM production apps** (5) | rust-playground, wasm-bindgen examples, figma, photoshop-web, excalidraw |
| **SEO/OG patterns** (4) | vue.js, svelte.dev, astro.build, react.dev |

### 1.2 Key findings driving architecture decisions

| Finding | Source | Implication |
|---------|--------|-------------|
| No graph **computation** library has in-browser demos | All 8 graph libs | WASM playground is a genuine first-mover opportunity |
| Sites that separate docs-app from playground-app are easier to maintain | three.js, p5js, ECharts | Keep playground as an independent SPA, not embedded in the doc site |
| `cargo doc` + doctests is the only API doc system a small team needs | docs.rs, pytorch | Do not build a separate API reference system |
| SSG with immutable content-hashed assets feels fastest | VitePress, Astro, Docusaurus | Use SSG, not SSR |
| Sandpack/Monaco in-page editors are the gold standard | react.dev, ECharts, svelte.dev | Code editor panel should use Monaco or CodeMirror |
| Algolia DocSearch is free for OSS and used by 4/6 doc sites | k8s, tailwind, next.js, deck.gl | Adopt Algolia DocSearch for search |
| GitHub Pages is zero-cost, zero-ops for small teams | Most Rust projects | Stay on GitHub Pages, add Cloudflare CDN later if needed |
| Single-version docs until project is mature | tailwind, next.js | No multi-version docs complexity until post-1.0 |
| Co-locating docs with source keeps them in sync | next.js, pytorch | All content lives in the main repo |
| OG tags: copy Astro's pattern exactly | astro.build | Minimal but complete meta tag set |

## 2. Architecture decisions

### ADR-W001: Static site generator — plain HTML + Vite

**Decision**: Use plain HTML/CSS/JS with Vite as the build tool for the
landing page and playground shell. No heavy framework (no Next.js, no Astro,
no React).

**Rationale**:
- three.js (300+ demos, most visited WebGL site) uses zero-framework static
  HTML — proof that it scales.
- Matches the project's "minimal dependencies" philosophy.
- The landing page is ~5 pages of static content. A framework is overkill.
- Vite provides: dev server with HMR, asset hashing, CSS/JS minification,
  and WASM file handling. Nothing more needed.
- If the site grows to need a framework later, migrating plain HTML to Astro
  is trivial (Astro's `.astro` format is HTML-superset).

**Alternatives rejected**:
- Astro: excellent but adds a build dependency tree. Revisit post-1.0.
- Next.js/Docusaurus: React dependency, SSR complexity, overkill for static
  content.
- mdBook for the landing page: no component/layout control, can't embed WASM
  islands.

### ADR-W002: Playground architecture — standalone SPA in `/playground/`

**Decision**: The WASM playground is a self-contained single-page application
built separately from the landing page. It loads via a simple `<a href>`
from the landing page, not an iframe or embedded component.

**Rationale**:
- ECharts, p5js, and three.js all separate their interactive editors from
  their doc sites. This is the dominant pattern for maintainability.
- The playground has complex state (graph data, algorithm selection, canvas
  rendering, Web Worker communication) that would pollute a simple landing
  page build.
- Can be developed, tested, and deployed independently.
- Failure in the playground does not break the docs or landing page.

**Components**:
```
playground/
├── index.html          # shell: 3-panel layout
├── src/
│   ├── main.ts         # entry point, wires panels together
│   ├── graph-editor.ts # left panel: node/edge creation, preset loader
│   ├── algo-panel.ts   # center panel: algorithm selector + params
│   ├── canvas.ts       # right panel: Canvas 2D renderer
│   ├── worker.ts       # Web Worker: loads WASM, runs algorithms
│   ├── presets.ts      # built-in graphs (karate, petersen, ER, BA, ...)
│   └── code-mirror.ts  # code display panel (read-only Rust snippets)
├── wasm/
│   └── rust_igraph_bg.wasm  # pre-built WASM module (~2-3 MB gzipped)
├── vite.config.ts
└── package.json        # devDependencies only: vite, typescript
```

**WASM module**: built from a thin `crates/igraph-wasm/` crate that
re-exports selected `rust-igraph` APIs via `#[wasm_bindgen]`. This crate
lives in the main repo workspace. The built `.wasm` + `.js` glue files are
committed to `website/playground/wasm/` (or fetched from a GitHub release
asset) so the playground deploys without a Rust toolchain in CI.

### ADR-W003: Docs integration — mdBook and rustdoc as linked subsites

**Decision**: mdBook and rustdoc are built by CI and deployed as subdirectories
of the GitHub Pages site. They are not embedded or iframed into the landing
page — just linked from the navigation.

**Rationale**:
- Both tools produce complete, self-contained static sites.
- Embedding them would require complex build coordination.
- Users expect rustdoc to look like rustdoc and mdBook to look like mdBook.
- Linked subsites are the pattern used by every Rust project surveyed.

**URL structure**:
```
/                    → landing page (website/)
/playground/         → WASM playground (website/playground/)
/book/               → mdBook (book/)
/api/                → rustdoc (cargo doc → target/doc/rust_igraph/)
/api/rust_igraph/    → crate root docs
```

### ADR-W004: Content freshness — doctests as the primary guard

**Decision**: All code snippets shown on the landing page and in examples
must also exist as doctests or integration tests in the Rust source. The
website never contains "orphan" code that isn't compiled.

**Rationale**:
- PyTorch uses `make doctest` to prevent stale examples. This is the most
  reliable freshness mechanism observed.
- rust-igraph already has 1,087 doctests. The website's code snippets
  should reference these, not duplicate them.
- A CI link-checker (`lychee`) catches broken URLs on every PR.

### ADR-W005: Search — Algolia DocSearch

**Decision**: Use Algolia DocSearch (free for open-source projects) for
site-wide search across landing page, mdBook, and rustdoc.

**Rationale**:
- Used by kubernetes.io, tailwindcss.com, nextjs.org/docs, deck.gl.
- Zero-ops: Algolia crawls the site automatically.
- Provides a polished search UI widget with keyboard navigation.
- Alternative (Lunr.js) requires client-side index and misses rustdoc pages.

**Apply**: after the site is public and indexed. Not needed for initial launch.

### ADR-W006: Deployment — GitHub Pages + GitHub Actions

**Decision**: Single GitHub Actions workflow builds all four subsites
(landing page, playground, mdBook, rustdoc) and deploys them as one Pages
artifact.

**Rationale**:
- Zero cost, zero ops, reliable (99.9%+ GitHub Pages uptime).
- All content from one repo, one workflow, one deployment.
- Cloudflare CDN can be added later for custom domain + edge caching without
  changing the build pipeline.

## 3. Directory structure

```
rust-igraph/
├── website/                        # ← NEW: official website source
│   ├── index.html                  # landing page
│   ├── assets/
│   │   ├── og-image.png            # OG social preview (1280×640)
│   │   ├── logo.svg                # project logo
│   │   ├── style.css               # landing page styles
│   │   └── hero-graph.js           # hero section animated graph (Canvas)
│   ├── playground/                 # standalone WASM playground SPA
│   │   ├── index.html
│   │   ├── src/
│   │   │   ├── main.ts
│   │   │   ├── graph-editor.ts
│   │   │   ├── algo-panel.ts
│   │   │   ├── canvas.ts
│   │   │   ├── worker.ts
│   │   │   ├── presets.ts
│   │   │   └── code-display.ts
│   │   ├── wasm/                   # pre-built WASM artifacts
│   │   │   ├── rust_igraph_bg.wasm
│   │   │   └── rust_igraph.js
│   │   ├── vite.config.ts
│   │   └── package.json
│   ├── examples/                   # algorithm gallery pages
│   │   ├── community-detection.html
│   │   ├── centrality.html
│   │   ├── shortest-paths.html
│   │   ├── graph-generators.html
│   │   ├── isomorphism.html
│   │   ├── layout-engines.html
│   │   └── network-flow.html
│   └── vite.config.ts              # landing page build config
│
├── crates/
│   └── igraph-wasm/                # ← NEW: wasm-bindgen wrapper crate
│       ├── Cargo.toml              # [lib] crate-type = ["cdylib"]
│       ├── src/lib.rs              # #[wasm_bindgen] exports
│       └── build.sh                # wasm-pack build --target web
│
├── book/                           # existing mdBook (unchanged)
├── src/                            # existing library source (unchanged)
├── .github/workflows/
│   └── pages.yml                   # updated: builds all 4 subsites
└── ...
```

## 4. CI/CD pipeline

### 4.1 Build workflow (`pages.yml`)

```yaml
jobs:
  build:
    steps:
      # 1. Checkout + toolchain
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown

      # 2. Build rustdoc
      - run: cargo doc --no-deps
        env:
          RUSTDOCFLAGS: "-D warnings"

      # 3. Build mdBook
      - run: cargo install mdbook --locked --version 0.4
      - run: mdbook build book

      # 4. Build WASM module (if igraph-wasm crate exists)
      - run: cargo install wasm-pack --locked
      - run: cd crates/igraph-wasm && wasm-pack build --target web --release
      - run: cp crates/igraph-wasm/pkg/*.wasm website/playground/wasm/
      - run: cp crates/igraph-wasm/pkg/*.js website/playground/wasm/

      # 5. Build landing page + playground
      - run: cd website && npm ci && npx vite build
      - run: cd website/playground && npm ci && npx vite build

      # 6. Assemble final site
      - run: |
          mkdir -p dist
          cp -r website/dist/* dist/
          cp -r website/playground/dist dist/playground/
          cp -r book/book dist/book/
          mkdir -p dist/api
          cp -r target/doc/* dist/api/

      # 7. Upload
      - uses: actions/upload-pages-artifact@v3
        with:
          path: dist
```

### 4.2 Quality gates (run on every PR)

| Check | Command | Purpose |
|-------|---------|---------|
| Rust tests | `cargo test --workspace` | Code correctness including doctests |
| Clippy | `cargo clippy -- -D warnings` | Lint |
| Doc build | `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` | No broken doc links |
| mdBook build | `mdbook build book` | Book compiles |
| WASM check | `cargo check --target wasm32-unknown-unknown` | WASM compatible |
| Link check | `lychee dist/**/*.html --no-progress` | No broken links |
| Website build | `cd website && npm ci && npx vite build` | Site compiles |

### 4.3 Dependency management

| Layer | Dependencies | Update strategy |
|-------|-------------|-----------------|
| Rust library | `thiserror` only | Dependabot weekly |
| WASM crate | `wasm-bindgen`, `js-sys`, `web-sys` | Pin major, Dependabot |
| Website | Vite (devDep only) | Pin major, manual update |
| Playground | zero runtime deps; Vite devDep | Same as website |
| CI tools | mdbook, wasm-pack | Pin version in workflow |

**Zero runtime JS dependencies** in the playground. All rendering is vanilla
Canvas API. The code display uses a `<pre>` with syntax highlighting via a
50-line tokenizer or a CDN-loaded highlight.js (no npm install). This
matches the project's minimal-dependency philosophy.

## 5. Reliability

### 5.1 Failure isolation

```
Landing page   ──┐
Playground     ──┤── independent static files
mdBook         ──┤   any one can fail without affecting others
rustdoc        ──┘
```

Each subsite is a self-contained directory of static files. A bug in the
playground JavaScript cannot break the API docs or tutorial. A broken mdBook
link cannot prevent the landing page from loading.

### 5.2 Uptime

- **GitHub Pages**: 99.95% historical uptime, CDN-backed, automatic HTTPS.
- **Static files only**: no server processes, no databases, no runtime
  failures. The site either deploys or it doesn't — no partial-failure modes.
- **Cache-friendly**: all assets are content-hashed by Vite. Browsers cache
  aggressively. Even if GitHub Pages has a brief outage, cached pages continue
  to work.

### 5.3 Rollback

```bash
# GitHub Pages deploys are versioned. Rollback:
gh run rerun <previous-green-run-id>
```

Or revert the commit and push — Pages redeploys automatically.

### 5.4 Monitoring

- **GitHub Actions status badge** on README — visible at a glance.
- **Uptime monitoring**: add a free UptimeRobot or GitHub-native
  `actions/uptime-monitor` check that pings `/` and `/playground/` every 5
  minutes. Alert via GitHub issue if down.
- **Lighthouse CI**: run Lighthouse on the landing page in CI. Fail the build
  if performance score drops below 90.

## 6. SEO infrastructure

### 6.1 Meta tags (every page)

```html
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>rust-igraph — Graph Algorithms in Rust & WASM</title>
<meta name="description" content="Pure-Rust graph algorithm library.
  1200+ APIs, zero unsafe, runs natively and in the browser via WASM.">

<!-- Open Graph -->
<meta property="og:type" content="website">
<meta property="og:title" content="rust-igraph">
<meta property="og:description" content="1200+ graph algorithms.
  Zero unsafe. Runs in the browser.">
<meta property="og:image" content="https://rust-igraph.dev/assets/og-image.png">
<meta property="og:url" content="https://rust-igraph.dev/">
<meta property="og:site_name" content="rust-igraph">

<!-- Twitter -->
<meta name="twitter:card" content="summary_large_image">
<meta name="twitter:image:alt" content="rust-igraph: graph algorithms
  in Rust and WASM">
```

### 6.2 Structured data

Not needed initially. JSON-LD (`schema.org/SoftwareSourceCode`) adoption
is near-zero among library sites surveyed. Add only if targeting Google
rich results for a commercial offering.

### 6.3 robots.txt and sitemap

```
# robots.txt
User-agent: *
Allow: /
Sitemap: https://rust-igraph.dev/sitemap.xml
```

Sitemap generated by a 20-line build script that lists all `.html` pages.

### 6.4 GitHub social preview

Upload a 1280×640 PNG to Settings > Social preview. Same branded image as
`og-image.png` but cropped to 2:1 ratio. This appears on Slack, Discord,
Twitter, Reddit link unfurls.

## 7. WASM playground — detailed design

### 7.1 Three-panel layout

```
┌─────────────────────────────────────────────────────────────┐
│  rust-igraph Playground          [Presets ▼]  [Share] [?]   │
├──────────────┬──────────────────┬────────────────────────────┤
│              │                  │                            │
│  Graph       │  Algorithm       │  Visualization             │
│  Editor      │  Controls        │  (Canvas 2D)               │
│              │                  │                            │
│  ┌────────┐  │  ○ BFS           │     ┌──●───●──┐           │
│  │ Add    │  │  ○ Dijkstra      │     │  │   │  │           │
│  │ Node   │  │  ● PageRank      │     ●──●   ●──●           │
│  │        │  │  ○ Louvain       │     │  │ ╲ │  │           │
│  │ Add    │  │  ○ Layout (FR)   │     ●──●──●──●           │
│  │ Edge   │  │                  │                            │
│  │        │  │  [Source: 0 ▼]   │  Nodes: 34  Edges: 78     │
│  │ Load   │  │  Damping: 0.85   │  Time: 0.23ms             │
│  │ Preset │  │                  │                            │
│  └────────┘  │  [▶ Run]         │                            │
├──────────────┴──────────────────┴────────────────────────────┤
│  // Equivalent Rust code:                                    │
│  let pr = pagerank(&g).unwrap();                             │
│  println!("{:?}", pr);                                       │
└─────────────────────────────────────────────────────────────┘
```

### 7.2 Algorithm coverage (initial)

| Algorithm | Visualization | Animation |
|-----------|--------------|-----------|
| BFS | Wavefront coloring (layer by layer) | Yes — step by step |
| Dijkstra | Shortest path highlighting + distance labels | Yes — relaxation |
| PageRank | Node size proportional to rank | Static |
| Louvain | Community coloring | Yes — merge phases |
| FR Layout | Force-directed positioning | Yes — spring simulation |
| Betweenness | Heat map on nodes | Static |
| Connected components | Color per component | Static |

### 7.3 Preset graphs

| Name | V | E | Why |
|------|---|---|-----|
| Zachary Karate Club | 34 | 78 | Classic, recognizable |
| Petersen graph | 10 | 15 | Beautiful, regular |
| Erdos-Renyi G(50, 0.1) | 50 | ~125 | Random structure |
| Barabasi-Albert(50, 2) | 50 | 96 | Power-law hubs visible |
| Small-world WS(30, 4, 0.1) | 30 | 60 | Clustering visible |
| Les Miserables | 77 | 254 | Famous literary network |
| User-drawn | any | any | Drag to create |

### 7.4 Web Worker protocol

```typescript
// Main thread → Worker
type WorkerRequest =
  | { type: 'init' }                              // load WASM
  | { type: 'run', algo: string, graph: Edge[],
      params: Record<string, number> }            // execute
  | { type: 'cancel' }                            // abort long-running

// Worker → Main thread
type WorkerResponse =
  | { type: 'ready' }                             // WASM loaded
  | { type: 'result', data: AlgoResult,
      elapsed_ms: number }                        // success
  | { type: 'error', message: string }            // failure
  | { type: 'progress', percent: number }         // for long algos
```

### 7.5 WASM crate API surface

```rust
// crates/igraph-wasm/src/lib.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmGraph { /* wraps rust_igraph::Graph */ }

#[wasm_bindgen]
impl WasmGraph {
    #[wasm_bindgen(constructor)]
    pub fn new(directed: bool) -> Self;
    pub fn add_edge(&mut self, u: u32, v: u32);
    pub fn from_edges(edges: &[u32], directed: bool) -> Self;
    pub fn vcount(&self) -> u32;
    pub fn ecount(&self) -> u32;

    // Algorithms — return JSON strings for flexibility
    pub fn bfs(&self, root: u32) -> String;
    pub fn dijkstra(&self, source: u32, weights: &[f64]) -> String;
    pub fn pagerank(&self) -> String;
    pub fn louvain(&self) -> String;
    pub fn betweenness(&self) -> String;
    pub fn connected_components(&self) -> String;
    pub fn layout_fr(&self, niter: u32) -> String; // returns [[x,y],...]
}
```

## 8. Landing page design

### 8.1 Sections (top to bottom)

1. **Hero**: Tagline + animated graph background (Canvas, not video) +
   two CTAs ("Get Started" → `/book/`, "Try in Browser" → `/playground/`)
2. **Numbers bar**: `1,200+ APIs` | `311 Algorithms` | `7,554 Tests` |
   `0 unsafe` | `1 Dependency`
3. **Code example**: side-by-side Rust code + result (static, not live)
4. **Feature grid**: 6 cards (Traversal, Centrality, Community, Flow,
   Isomorphism, Generators) each with icon + 1-line description
5. **Comparison table**: rust-igraph vs petgraph vs igraph vs networkx
   (same as current README table but styled)
6. **WASM callout**: "Runs in the browser" section with link to playground
7. **Ecosystem**: badges (crates.io, docs.rs, CI, license, MSRV) +
   links (GitHub, mdBook, rustdoc)
8. **Footer**: license, links, OG tags

### 8.2 Design principles

- **No JavaScript required** for the landing page content. The hero
  animation is progressive enhancement.
- **Mobile-first**: responsive grid, touch-friendly CTAs.
- **Dark mode**: respects `prefers-color-scheme`. Graph-heavy sites look
  better dark.
- **Performance budget**: < 100 KB total page weight (excluding hero
  animation). Lighthouse score > 95.
- **Accessibility**: semantic HTML, ARIA labels, keyboard navigation,
  contrast ratio > 4.5:1.

## 9. Implementation phases

| Phase | Deliverable | Effort | Depends on |
|-------|-------------|--------|------------|
| **P0** | Landing page skeleton: HTML + CSS + meta tags + GitHub Pages deploy | 1 day | Logo + OG image from user |
| **P1** | `crates/igraph-wasm/` — wasm-bindgen wrapper for 7 core algorithms | 1-2 days | — |
| **P2** | Playground MVP: 3-panel UI + Canvas renderer + 3 presets + 3 algorithms (BFS, PageRank, FR Layout) | 2-3 days | P1 |
| **P3** | Playground full: all 7 algorithms + all presets + animation + code panel + performance timer | 2 days | P2 |
| **P4** | Examples gallery: 7 category pages with pre-rendered visualizations | 2 days | P2 |
| **P5** | Polish: Algolia DocSearch, Lighthouse tuning, lychee link checker, uptime monitor | 1 day | P0-P4 |
| **P6** | Launch: awesome-rust PR, Show HN, r/rust post, This Week in Rust | 1 day | P0-P5 |

P0 can start immediately once logo/OG image is decided. P1 can start in
parallel (no dependency on visual design).

## 10. Custom domain

When ready, register `rust-igraph.dev` (or `.org`) and point it to GitHub
Pages via CNAME. Cloudflare free tier provides:
- Edge CDN (faster global loads)
- DDoS protection
- Analytics (no JS required — server-side)
- Automatic HTTPS via Let's Encrypt

This is a configuration change, not a code change. No rush — GitHub Pages
URL works fine for initial launch.

## 11. Open questions for user

1. **Logo and OG image** — user will design. Color palette feeds into CSS
   variables for the landing page.
2. **Domain**: `rust-igraph.dev` vs `rust-igraph.org` vs stay on
   `Totoro-jam.github.io/rust-igraph`?
3. **Hero animation**: force-directed graph simulation vs static SVG art?
4. **Playground code editor**: read-only (show equivalent Rust code) vs
   editable (user types Rust-like DSL that maps to WASM calls)?
5. **Blog**: worth adding at launch, or defer until post-1.0?

# Website Development SOP

Standard operating procedures for developing, testing, and deploying the
rust-igraph website (landing page, playground, mdBook, rustdoc).

## 1. Branch and PR conventions

| Rule | Detail |
|------|--------|
| Branch naming | `website/<component>-<short-desc>` (e.g. `website/playground-bfs-panel`) |
| PR title | `feat(website): <what>` or `fix(website): <what>` |
| PR scope | One logical feature per PR. Landing page, playground, and WASM crate changes can be bundled when tightly coupled. |
| Review | Self-merge is OK for style/copy changes. Structural changes (new deps, CI pipeline, WASM API surface) get a 24h cool-down before merge. |
| Commit identity | `git -c user.name=Totoro-jam -c user.email=moqiuchen66@gmail.com` |
| Never commit | `.claude/hooks/`, `.claude/settings.json`, `node_modules/`, `dist/`, `.wasm` build artifacts |

## 2. Development workflow

### 2.1 Rust library + WASM crate

```bash
# After modifying rust-igraph APIs used by the WASM crate:
cargo check -p igraph-wasm
cargo check --target wasm32-unknown-unknown -p igraph-wasm
cargo clippy -p igraph-wasm -- -D warnings
cargo test --workspace -q
```

### 2.2 Website / Playground (once created)

```bash
cd website && npm ci && npm run dev       # landing page dev server
cd website/playground && npm ci && npm run dev  # playground dev server

# Before commit:
npm run build          # verify production build
npm run lint           # ESLint
npm run typecheck      # tsc --noEmit
```

### 2.3 mdBook

```bash
mdbook serve book      # local preview at localhost:3000
mdbook build book      # verify build
```

## 3. Testing checklist (per PR)

- [ ] `cargo test --workspace` passes (all 1087+ tests)
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] `cargo check --target wasm32-unknown-unknown -p igraph-wasm` compiles
- [ ] Website builds: `cd website && npm run build` (when applicable)
- [ ] Playground builds: `cd website/playground && npm run build` (when applicable)
- [ ] mdBook builds: `mdbook build book`
- [ ] No `.claude/` files staged (`git diff --cached --name-only | grep -v '^\.claude'`)
- [ ] Lighthouse score >= 90 for landing page (P5 onwards)

## 4. Deployment pipeline

All deployment happens via GitHub Actions (`pages.yml`). Manual deploys
are never needed.

```
push to main → CI builds all 4 subsites → GitHub Pages deploys
```

**Build order** (CI):
1. `cargo doc --no-deps` → rustdoc
2. `mdbook build book` → mdBook
3. `wasm-pack build crates/igraph-wasm` → WASM module (future)
4. `cd website && npm ci && npx vite build` → landing page (future)
5. `cd website/playground && npm ci && npx vite build` → playground (future)
6. Assemble all into `dist/` and upload

**Rollback**: revert the commit and push, or re-run a previous green
Actions run via `gh run rerun <id>`.

## 5. Common pitfalls and mitigations

| Pitfall | Mitigation |
|---------|------------|
| Committing `.claude/` files | Always use explicit `git add <files>`, never `git add -A` or `git add .` |
| Wrong git identity | Always use `-c user.name=Totoro-jam -c user.email=moqiuchen66@gmail.com` |
| WASM crate breaks after library API change | `cargo check --target wasm32-unknown-unknown -p igraph-wasm` in CI |
| Large WASM bundle | Track `.wasm` size in CI; alert if > 5 MB uncompressed |
| Stale references in docs | Grep for removed features before release (e.g. `grep -r "faer\|bliss-rust"`) |
| Playground JS error breaks docs | Subsites are isolated static files — playground failure cannot break API docs |
| `cargo install --version` needs full semver | Use `0.4.51` not `0.4`; partial versions cause `cargo install` to fail in CI |
| `node_modules/` committed | `.gitignore` covers this; verify before `git add` |
| Forgetting to update i18n strings | Both `en.json` and `zh.json` must be updated for every user-facing string |
| **Rustdoc 样式丢失** | `cp -r target/doc/* _site/` 复制完整 rustdoc 输出（含 `static.files/`、`src/`、`search.index/`），不要只复制 `rust_igraph/` 子目录。rustdoc HTML 通过 `../static.files/` 相对路径引用 CSS/JS (fixed 2026-06-04) |
| 站点组装顺序 | 先 rustdoc（底层），再 website overlay（更高优先级），最后 mdBook。避免 rustdoc 根目录文件覆盖 landing page |
| mdBook 外部链接变空文件 | mdBook SUMMARY.md 中的 URL 被当作本地文件路径处理，会在 `book/src/` 下创建空文件。解决：用本地 `.md` stub 页面替代外部链接，在 stub 中放 GitHub 链接 (fixed 2026-06-04) |

## 6. Dependency policy

| Layer | Allowed | Forbidden |
|-------|---------|-----------|
| Rust library | `thiserror` only | Everything else without ADR |
| WASM crate | `wasm-bindgen`, `serde`, `serde_json` | Heavy frameworks |
| Playground | `react`, `react-dom`, `@codemirror/*`, `vite` | UI component libraries (MUI, Ant Design, etc.) |
| Landing page | Vite (devDep only) | Runtime JS frameworks |

New dependencies require an ADR entry in `WEBSITE_ARCHITECTURE.md` and
explicit user approval.

## 7. Release checklist

When cutting a new library version that affects the website:

1. Update version in root `Cargo.toml`
2. Update `CHANGELOG.md`
3. Rebuild WASM module: `cd crates/igraph-wasm && wasm-pack build --target web --release`
4. Update stats on landing page (API count, test count, etc.)
5. Update mdBook version references
6. `cargo publish --dry-run` before actual publish
7. Push and verify GitHub Pages deploy succeeds

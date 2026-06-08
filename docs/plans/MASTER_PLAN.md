# rust-igraph 工程化主计划（合并版）

> **版本**：2026-05-15  
> **状态**：取代 `.codefuse/plans/` 下所有早期计划，作为唯一主计划。早期计划仅作历史参考。  
> **适用模式**：1 名开发者 + AI（兼职推进，强调可恢复、原子化、可复用流程）  
> **目标库**：igraph C v1.0.x（约 850 个公共 API、~135,000 SLOC）→ rust-igraph 纯 Rust 实现  

---

## 0. 这份计划解决了什么问题

### 0.1 历史计划存在的不足

`.codefuse/plans/` 下已有 6 份计划，主要差异在许可证、ARPACK 替换路径与 BLISS 策略，最终在 `AI 辅助开发版（16:06）` 中收敛。但所有计划都缺三件事，本计划集中补齐：

| 缺失项 | 表现 | 本计划的应对 |
|--------|------|-------------|
| **没有行走骨架** | Phase 1 用 2 个月只做数据结构，期间无法端到端跑任何算法。架构风险延迟暴露 | 新增 Phase 0：3 周内打通 "load → 1 算法 → output → oracle 对比 → CI 绿" 全流程 |
| **没有可复用的算法移植 SOP** | 850 个函数被列成表格但没有"批量加工"的工序 | 第四章定义 9 步 AWU（Algorithm Work Unit）流程 + 模板文件 + AI 协作分工 |
| **任务颗粒度不够原子** | 时间估算到天级、模块级，但不到"一次 AI 会话能完成"的级别，难追踪、难恢复 | 每个算法分配 ALGO-NNN 编号，状态机、DoD、PR 模板配套；ALGORITHMS.md 作为单一真相源 |
| **未考虑兼职/可恢复** | 默认全职推进，未给出长时间断档后的恢复路径 | 每个 AWU 自包含；阶段间增加 checkpoint；session 恢复指南 |

### 0.2 与已有计划的合并取舍

| 来源计划 | 继承的内容 | 废弃的内容 |
|---------|----------|-----------|
| `AI 辅助开发版（16:06）` | Phase 1-10 划分、算法清单、faer + 自研 IRLM/IRAM、BLISS C++→Rust 翻译、三层 ARPACK 后端 | 工时估算需重新分摊到 AWU 维度 |
| `ARPACK 与 BLISS 完整方案（15:23）` | EigenSolver / ArpackSolver trait、3 阶段 BLISS 路径、faer 集成方式 | (无废弃) |
| `项目实施计划（13:48）` | 分层架构图、模块目录、commit 规范 | 5 人团队估算（已切换为单人 + AI） |
| `实施计划（11:50）` | 测试三层体系、137 人月评估、模块映射表 | 5-8 人团队工期 |
| `实施计划（11:02）` | 阶段 0-12 的较细粒度划分思路 | MIT/Apache-2.0 许可（已切换至 GPL-2.0-or-later） |
| `纯 Rust 图算法库实施计划（11:40）` | 测试资产复用比例、规模基准 | clean-room 实现（已切换为参照翻译） |

> **重要**：以下章节中带 ⚠ 标记的部分明确说明哪个早期决策已被推翻。

---

## 一、核心方法论

### 1.1 行走骨架（Walking Skeleton）优先

**原则**：先用 3 周做出一个最小但**端到端**可运行的系统，比用 2 个月做完整数据结构层、再发现集成问题要好得多。

骨架包含 1 条最薄的端到端切片：

```
edge-list 文件 → Graph<u32>（最简邻接表）→ BFS → Vec<usize>
                                              ↓
                                        oracle 脚本调 python-igraph 同算法
                                              ↓
                                          浮点/整数对比 → 通过
                                              ↓
                              CI（fmt + clippy + test + wasm32-check + license）绿
```

之后所有 Phase 都遵守一条不变量（**始终可运行**）：**任何 PR 合并后，cargo test 必须仍然通过；任何新算法都接入既有的 oracle / fixture / CI 流水线。**

### 1.2 算法移植 SOP（Algorithm Work Unit, AWU）

每个算法被切成一个 AWU，编号 `ALGO-NNN`，附带：
- C 源码定位（文件:行号）
- 复杂度标签（copy / adapt / rewrite / novel）
- 前置依赖（其它 AWU、数据结构、特征值求解器）
- 9 步 pipeline（见第四章）
- DoD（验收清单）

AWU 是本计划的"工作原子"。所有计时、跟踪、AI 调度都以 AWU 为单位。

### 1.3 单人 + AI 兼职模式的约束

| 约束 | 影响 | 应对 |
|------|------|------|
| 长时间断档（数天～数周） | 上下文丢失 | 每个 AWU 自包含输入：C 源、接口、模板、邻居成品；恢复指南见第七章 |
| 单点决策瓶颈 | 接口冻结需主 reviewer | 主 agent 负责接口、subagent 负责实现/翻译 |
| 不能用大量人工对账 | numerical correctness 风险 | oracle 自动化 + nightly 全量回归 |
| 容易 burn-out | 长尾算法（Motif/HRG 等）容易被无限推迟 | 每个 Phase 设最低退出门，不追求完美 |

### 1.4 资源/时间假设修正

⚠ 早期计划的 18 个月（AI 辅助单人全职）需要修正为兼职节奏。本计划不给出"x 个月"承诺，改用：
- **AWU 平均规模**：1.5～4 小时人工时间（含 AI 协作）
- **AWU 总数**：约 850 个公共 API → 折算 ~600 个 AWU（数据结构、辅助函数被合并）
- **Phase 退出门**：以 AWU 通过率而非日期衡量
- **里程碑**：v0.1（Phase 0+1+2 完成）、v0.5（Phase 3-6 完成）、v1.0（Phase 7-10 完成）

---

## 二、架构决策（继承自最新计划）

### 2.1 关键决策固化

| 决策 | 选择 | 不可动摇的理由 |
|------|------|--------------|
| 许可证 | **GPL-2.0-or-later** | 允许直接参照翻译 igraph C 代码，工作量降至 1/3；与 igraph 同许可 |
| 实现方式 | 纯 Rust 参照翻译 | 正确性继承 20 年验证；WASM 兼容；商业层后续以独立项目隔离 |
| 线性代数 | **纯手写**（power iteration + 未来 IRLM/IRAM） | 零依赖、WASM 天然兼容、完全控制收敛行为 |
| ARPACK 替代 | 两层分级：A=手写幂迭代（PageRank 默认）/ B=手写 IRLM 大稀疏 | 与 igraph 数值精确匹配；无外部依赖 |
| 自研 IRLM / IRAM | 参照 igraph `src/linalg/arpack.c` 翻译 | 1634 行 C 翻译为 Rust ~2000 行，AI 适合此类 1:1 翻译 |
| BLISS 替代 | 直接翻译 igraph 内嵌 BLISS C++→Rust（约 9500 行） | GPL 许可证允许；100% API 兼容 |
| 同构兜底 | 阶段 1 用 VF2 + isoclass 查表覆盖主要路径 | BLISS 翻译完成前 isomorphic() 仍可用 |
| nauty C FFI | 可选 feature（`nauty-backend`），默认关闭 | 极大规模图性能优化；不影响 WASM 主路径 |
| crate 结构 | **单 crate `rust-igraph`** + 内部 `mod core` / `mod algorithms`（ADR-0009 取代 ADR-0002 的 3-crate workspace） | 对外只发一个包；模块层级仍区分 core vs algorithms |

### 2.2 Crate 结构

> ADR-0009（2026-05-15）：从 3-crate workspace 合并为单 crate `rust-igraph`，
> 保留模块层级区分。详见 `.codefuse/tracking/ARCHITECTURE.md`。

```
rust-igraph/                              # 单 crate 仓库根
├── Cargo.toml                            # name = "rust-igraph"
├── src/
│   ├── lib.rs                            # crate 根 + 顶层 re-exports
│   ├── core/                             # 数据结构 + 错误 + 迭代器 + RNG
│   │   ├── mod.rs
│   │   ├── graph.rs                      # Graph (igraph_t)
│   │   ├── vector.rs                     # Vector / VectorInt / VectorBool
│   │   ├── matrix.rs                     # Matrix / MatrixInt
│   │   ├── sparsemat.rs                  # SparseMatrix (CSR/CSC)
│   │   ├── strvector.rs                  # StringVector
│   │   ├── selectors.rs                  # VertexSelector / EdgeSelector
│   │   ├── error.rs                      # IgraphError + IgraphResult
│   │   ├── rng.rs                        # 对标 igraph_rng_t
│   │   ├── attributes.rs                 # 属性系统
│   │   ├── iterators.rs
│   │   └── internal/                     # heap, dqueue, stack, set, psumtree (pub(crate))
│   │
│   └── algorithms/                       # 算法层（按 igraph/src/ 子目录对标）
│       ├── mod.rs
│       ├── traversal/                    # bfs, dfs, random_walk
│       ├── shortest_paths/               # dijkstra, bf, johnson, fw, astar, widest
│       ├── connectivity/                 # cc, scc, biconn, articulation, bridges
│       ├── centrality/                   # bc, cc, ec, pagerank, hits, katz, harmonic
│       ├── community/                    # louvain, leiden, walktrap, fg, lpa, infomap, spinglass, eb, leading_ev, fluid, voronoi
│       ├── flow/                         # maxflow (PR/EK/Dinic), mincut, gomory_hu, dominators, all_st_cuts
│       ├── spanning/                     # mst (prim/kruskal)
│       ├── isomorphism/                  # vf2, lad, isoclasses, simplify_and_colorize
│       │   └── bliss/                    # bliss 翻译
│       ├── coloring/                     # dsatur, greedy
│       ├── matching/                     # bipartite, hungarian
│       ├── generators/                   # er, ba, ws, sbm, lattice, tree, famous, ...
│       ├── layout/                       # fr, kk, mds, sugiyama, umap, drl, gem, dh, random/circle/grid
│       ├── cliques/                      # cliquer 翻译
│       ├── motifs/                       # randesu, dyad, triad, graphlets
│       ├── cycles/                       # simple_cycles, feedback_arc, eulerian
│       ├── spectral/                     # laplacian, embedding
│       ├── operators/                    # union, intersection, complementer, linegraph, simplify
│       ├── transforms/                   # to_directed/undirected, permute, decompose
│       ├── similarity/                   # cocitation, jaccard, dice, bibcoupling
│       ├── degree_seq/                   # is_graphical, realize_degree_sequence
│       ├── epidemics/                    # sir, percolation
│       ├── hrg/                          # hrg_*
│       ├── spatial/                      # delaunay, beta_skeletons
│       ├── scan/                         # scan_*
│       ├── linalg/                       # arpack (IRLM/IRAM), eigen 高层、blas/lapack 桥接
│       └── io/                           # edgelist, ncol, lgl, pajek, gml, graphml, dot, dimacs, leda, dl
│
├── tests/
│   ├── oracle.rs                         # python-igraph live oracle
│   ├── conformance.rs                    # 三源 conformance fixture 加载
│   ├── property.rs                       # proptest 不变量
│   ├── common/mod.rs                     # 共用 helper
│   └── conformance/{c,py,r}/             # JSON fixture 数据（按算法分子目录）
│
├── benches/                              # criterion
├── examples/                             # 示例代码
├── fixtures/                             # 标准图数据（karate, dolphins, ...）
├── scripts/                              # oracle.py, test_extract/, bench_compare.py
├── templates/                            # AWU 模板（algo.rs.tpl, test.rs.tpl, ...）
├── book/                                 # mdBook 站
├── docs/plans/MASTER_PLAN.md             # 本文件
└── .codefuse/tracking/                   # ALGORITHMS.md, ARCHITECTURE.md, CONFORMANCE.md, ...
```

**对外用法**（任何用户）：
```toml
[dependencies]
rust-igraph = "..."
```
```rust
use rust_igraph::{Graph, bfs};
```

### 2.3 依赖清单

> ⚠ **2026-06-03 修订**：实际实现证明所有线性代数和图算法均可纯手写完成。
> 当前运行时唯一依赖为 `thiserror`。以下为修订后的依赖规划。

| 依赖 | 版本 | 用途 | 必须? |
|------|------|------|------|
| `thiserror` | ^2 | 错误派生 | 是（唯一运行时依赖） |
| `quick-xml` | ^0.37 | GraphML 解析 | 可选（`io-graphml`，未来 Phase B） |
| `rayon` | ^1.11 | 数据并行 | 可选（`parallel`，未来 Phase F） |
| `proptest` | ^1.6 | 属性测试 | dev |
| `criterion` | ^0.5 | 基准测试 | dev |

⚠ **明确不引入**：`faer`（依赖树过重，手写足够）、`nalgebra`（同理）、`petgraph`（API 表达力不够）、`graphalgs`（GPL-3.0 与本项目 GPL-2.0+ 不兼容）、`scirs2-sparse`（依赖链过重）。

### 2.4 Feature flags

```toml
[features]
default = []

io-graphml = ["dep:quick-xml"]
io-gml = []
io-pajek = []
io-all = ["io-graphml", "io-gml", "io-pajek"]

parallel = ["dep:rayon"]                             # 可选；数据并行加速

oracle-tests = []                                    # 启用 live python-igraph oracle 测试
proptest-harness = []                                # 启用 proptest bodies
```

WASM 验证目标：`cargo check --target wasm32-unknown-unknown`（零外部依赖，天然兼容）。

---

## 三、Phase 0：行走骨架（前 3 周，约 30-40 个 AWU）

### 3.1 Phase 0 退出准则（Definition of Done）

✅ 三句话总结：**仓库 clone 下来 5 分钟内，新人能跑出 BFS 在 Karate 图上的结果，并看到与 python-igraph 完全一致。**

具体清单（每条都需在 PR 中复核）：

- [ ] cargo workspace 编译通过（3 crate）
- [ ] `cargo test --workspace` 通过
- [ ] `cargo clippy --workspace -- -D warnings` 无告警
- [ ] `cargo fmt --check` 通过
- [ ] `cargo check --target wasm32-unknown-unknown` 通过
- [ ] `cargo deny check` 许可证扫描通过
- [ ] CI（GitHub Actions）所有 job 绿色
- [ ] `cargo run --example bfs_karate` 输出正确 BFS 顺序
- [ ] `cargo test --features oracle-tests` 至少 1 个 oracle 测试通过（BFS on Karate）
- [ ] **三源 conformance 跑通**：BFS 至少各 1 个 case 来自 igraph C tests / python-igraph tests / R-igraph testthat，全部通过
- [ ] **AI 基础设施就绪**：CLAUDE.md + 6 agent + 9 skill + 3 hook 全部入库；`/awu-start` 在 BFS 上演示一次端到端能跑通
- [ ] `cargo bench` 至少 1 个 baseline 基准跑通（BFS）
- [ ] ALGORITHMS.md 创建并填入完整 AWU 编号表（状态全部 `todo` 除 BFS）
- [ ] templates/ 下 4 个模板文件就绪（algo.rs.tpl / test.rs.tpl / oracle.py.tpl / bench.rs.tpl）
- [ ] DEVELOPMENT.md 描述 AWU 工作流（CONTRIBUTING.md 是 alpha 阶段对外声明）
- [ ] mdBook 文档站编译并部署到 GitHub Pages（或本地 build 通过）

### 3.2 Phase 0 任务清单（编号 BOOT-NN）

| ID | 任务 | 复杂度 | 估时 | 依赖 |
|----|------|-------|------|------|
| BOOT-01 | 初始化 git 仓库、`.gitignore`、LICENSE（GPL-2.0-or-later）、README 骨架 | novel | 0.5h | - |
| BOOT-02 | Cargo workspace + 3 crate 骨架（lib.rs 占位） | novel | 1h | BOOT-01 |
| BOOT-03 | `IgraphError` 枚举（先列 10-15 个核心错误码，预留扩展） | adapt | 1h | BOOT-02 |
| BOOT-04 | 极简 `Graph<u32>`（无向无权、邻接表 `Vec<Vec<u32>>`、`new` / `add_vertices` / `add_edges` / `vcount` / `ecount` / `neighbors`） | adapt | 2h | BOOT-03 |
| BOOT-05 | 极简 EdgeList 读取器（`io::read_edgelist`，纯文本 "u v" 每行） | adapt | 1h | BOOT-04 |
| BOOT-06 | 极简 BFS（`traversal::bfs(graph, root) -> Vec<u32>`，VecDeque + visited bitset） | adapt | 1h | BOOT-04 |
| BOOT-07 | example：`examples/bfs_karate.rs`（读 fixtures/karate.edges → BFS → println） | novel | 0.5h | BOOT-05, BOOT-06 |
| BOOT-08 | 标准 fixture：`fixtures/karate.edges`（来源 igraph C 测试或 networkrepository） | copy | 0.5h | - |
| BOOT-09 | `scripts/oracle.py`（接收 stdin JSON：图 + 算法名 + 参数 → stdout JSON：结果） | novel | 2h | - |
| BOOT-10 | `tests/oracle/mod.rs`：subprocess 调用 oracle.py 的辅助宏 `assert_oracle_eq!` | novel | 2h | BOOT-09 |
| BOOT-11 | 第一个 oracle 测试：BFS on karate，对比 python-igraph 的 bfs() | adapt | 0.5h | BOOT-06, BOOT-10 |
| BOOT-12 | proptest 骨架：`tests/property/mod.rs` + 1 个示例（无向图 BFS 可达集对称） | novel | 1h | BOOT-06 |
| BOOT-13 | criterion 骨架：`benches/bench_bfs.rs`（BFS on karate / ER(1000)） | novel | 1h | BOOT-06 |
| BOOT-14 | GitHub Actions：fmt + clippy + test 矩阵（stable, beta, MSRV） | novel | 1.5h | BOOT-02 |
| BOOT-15 | GitHub Actions：wasm32-unknown-unknown check job | novel | 0.5h | BOOT-14 |
| BOOT-16 | GitHub Actions：cargo-deny job（许可证 + 安全公告） | novel | 1h | BOOT-14 |
| BOOT-17 | GitHub Actions：oracle 测试 job（带 python-igraph） | novel | 1.5h | BOOT-09, BOOT-14 |
| BOOT-18 | GitHub Actions：cargo doc + GitHub Pages 部署 | novel | 1h | BOOT-14 |
| BOOT-19 | 模板：`templates/algo.rs.tpl`（含占位符注释、错误处理、文档框架） | novel | 1h | - |
| BOOT-20 | 模板：`templates/test.rs.tpl`（单元 + oracle + proptest 三块） | novel | 1h | BOOT-19 |
| BOOT-21 | 模板：`templates/oracle.py.tpl`（添加新算法到 oracle.py 的步骤） | novel | 0.5h | BOOT-09 |
| BOOT-22 | 模板：`templates/bench.rs.tpl` | novel | 0.5h | BOOT-13 |
| BOOT-23 | `.codefuse/tracking/ALGORITHMS.md`（带全部 ALGO-NNN 编号、初始状态 todo） | novel | 3h | - |
| BOOT-24 | `.codefuse/tracking/ARCHITECTURE.md`（关键决策与 ADR 索引） | novel | 1h | - |
| BOOT-25 | DEVELOPMENT.md（AWU 工作流、PR 模板、commit 规范）+ minimal CONTRIBUTING.md（alpha 阶段对外声明） | novel | 1.5h | BOOT-19, BOOT-23 |
| BOOT-26 | mdBook 骨架（src/SUMMARY.md、第一章 "Hello rust-igraph"） | novel | 1h | - |
| BOOT-27 | 性能基线脚本：`scripts/bench_compare.py`（criterion JSON + python-igraph 时间对比） | novel | 2h | BOOT-13 |
| BOOT-28 | session 恢复指南：`.codefuse/tracking/RESUME.md`（兼职断档后如何继续） | novel | 0.5h | BOOT-25 |
| BOOT-29 | `scripts/test_extract/from_c.py` 骨架（提取 BFS 的 1 个 igraph C 测试 + .out 验证流程） | novel | 2h | BOOT-09 |
| BOOT-30 | `scripts/test_extract/from_py.py` 骨架（提取 BFS 的 1 个 python-igraph 测试方法） | novel | 2h | BOOT-09 |
| BOOT-31 | `scripts/test_extract/run_r.R` + `from_r.py` 骨架（提取 BFS 的 1 个 R-igraph testthat 测试） | novel | 2h | BOOT-09 |
| BOOT-32 | `tests/conformance/mod.rs` + 第一个三源融合测试（BFS 同时跑 C/py/R conformance） | novel | 1.5h | BOOT-29..31 |
| BOOT-33 | 仓库根 `CLAUDE.md`（项目硬约束、AWU 流程、PR 模板入口、license 提醒） | novel | 1h | BOOT-25 |
| BOOT-34 | `.claude/agents/` 6 个 agent（recon/translator/tester/conformance/numerical/perf/doc，含 frontmatter + 系统提示） | novel | 2.5h | BOOT-33 |
| BOOT-35 | `.claude/skills/` 9 个 skill（awu-start/translate/test/conformance/bench/finish/oracle-add/phase-checkpoint/resume-session） | novel | 2.5h | BOOT-34 |
| BOOT-36 | `.claude/hooks/` 3 个 hook（post-edit-rust / pre-commit / post-tool-bash）+ `.claude/settings.json` 注册 | novel | 1.5h | BOOT-35 |
| BOOT-37 | `.codefuse/tracking/AI_PROMPTS.md` 骨架（含 Recon / Translate / Test 三个起步 prompt） | novel | 1h | BOOT-34 |

**Phase 0 合计**：约 46 小时纯工作（兼职 4-5 周可完成）。

### 3.3 Phase 0 关键决策点（提前 freeze）

| 决策 | 选项 | 推荐 | 备注 |
|------|------|------|------|
| Graph 顶点 ID 类型 | u32 / u64 / 泛型 N | **u32（固定）** | igraph 用 igraph_integer_t（默认 64bit），但 Rust 中 u32 足够覆盖 4B 顶点；后续可加 feature 切到 u64 |
| 顶点 ID 是否密集 | 密集 0..n / 稀疏 | **密集** | 与 igraph 一致；属性绑定到索引 |
| 邻接表存储 | `Vec<Vec<u32>>` / CSR | **Vec<Vec<u32>>**（Phase 0）→ CSR（Phase 1） | 行走骨架先简单 |
| 错误类型 | thiserror / 手写 | **thiserror** | 派生 + From 实现快 |
| oracle 通信 | subprocess + JSON / pyo3 | **subprocess + JSON**（Phase 0）→ pyo3（可选） | subprocess 更可靠、CI 简单 |

---

## 四、算法移植 SOP（核心可复用流程）

### 4.1 AWU（Algorithm Work Unit）定义

每个 AWU 是 ALGORITHMS.md 中的一行。最小元数据：

```markdown
| ID | 名称 | C 源 | 行数 | 复杂度 | 前置 | 状态 | PR | 备注 |
|----|------|------|------|-------|------|------|----|------|
| ALGO-CORE-001 | Graph 核心结构 | type_indexededgelist.c | 2013 | adapt | - | wip | #12 | CSR 重构 |
| ALGO-TR-001 | BFS（含回调） | bfs.c | 300 | adapt | ALGO-CORE-001 | done | #15 | |
| ALGO-SP-001 | Dijkstra | distances_dijkstra.c | 800 | adapt | ALGO-CORE-001, ALGO-DS-HEAP | wip | #28 | |
| ALGO-CT-005 | PageRank | pagerank.c | 721 | rewrite | ALGO-LA-IRLM | blocked | - | 等 IRLM |
```

状态机：`todo → wip → review → done → verified`（verified 表示通过 nightly 全量 oracle 回归）。

### 4.2 单 AWU 9 步 Pipeline

```
┌─────────────────────────────────────────────────────────────────┐
│ Step 1: Recon (15min, AI subagent)                              │
│   读 igraph C 源码 + 头文件 + 1-2 个 unit test → 生成摘要：    │
│   - 函数签名（C → Rust 对应）                                  │
│   - 输入/输出/副作用                                            │
│   - 边界条件清单                                                │
│   - 数值精度注意事项                                            │
│   - 测试预期形态                                                │
├─────────────────────────────────────────────────────────────────┤
│ Step 2: Interface Sketch (15min, 主 agent)                     │
│   起草 Rust pub fn 签名、文档、错误码                          │
│   → 写入 PR description（人工 review，确认接口冻结）          │
├─────────────────────────────────────────────────────────────────┤
│ Step 3: Skeleton (10min, 主 agent)                             │
│   从 templates/algo.rs.tpl 拷贝 → 改名 → 加 #[doc] →           │
│   编译通过（unimplemented!()） → 初始 commit                   │
├─────────────────────────────────────────────────────────────────┤
│ Step 4: Implementation (1-3h, AI subagent + 人工)              │
│   AI 翻译 C 实现 → 替换 unimplemented!()                       │
│   编译通过、clippy 无警告                                      │
├─────────────────────────────────────────────────────────────────┤
│ Step 5: Unit Tests (30min, AI subagent)                        │
│   templates/test.rs.tpl → 至少覆盖：                           │
│   - 空图 / 单顶点 / 完全图 / 不连通                            │
│   - 有向 vs 无向（如适用）                                     │
│   - 加权 vs 无权（如适用）                                     │
├─────────────────────────────────────────────────────────────────┤
│ Step 6: Oracle + Conformance (30min, 主 agent + subagent)      │
│   6a. 在 scripts/oracle.py 新增对应 case (live oracle)         │
│   6b. 跑三源提取脚本：                                         │
│       - scripts/test_extract/from_c.py --algo <name>          │
│       - scripts/test_extract/from_py.py --algo <name>         │
│       - scripts/test_extract/run_r.R + from_r.py --algo <name>│
│   6c. 验证 tests/conformance/{c,py,r}/<algo>/ 全部通过         │
│   6d. CONFORMANCE.md 更新覆盖矩阵                              │
├─────────────────────────────────────────────────────────────────┤
│ Step 7: Property Test (15min, AI subagent)                     │
│   找 1-2 条不变量（如 d(u,v)==d(v,u) / sum(pagerank)==1）     │
│   写 proptest                                                  │
├─────────────────────────────────────────────────────────────────┤
│ Step 8: Bench (15min, AI subagent)                             │
│   benches/ 下加 baseline + 与 python-igraph 时间对比           │
│   记录到 docs/perf/ALGO-XXX.md                                 │
├─────────────────────────────────────────────────────────────────┤
│ Step 9: Doc + Example (15min, AI subagent)                     │
│   rustdoc 完整化（参数 / 错误 / Panics / Examples）            │
│   examples/ 下加最小示例                                       │
│   ALGORITHMS.md 状态 todo → done                                │
└─────────────────────────────────────────────────────────────────┘
```

**典型 AWU 总耗时**：1.5-4 小时（取决于复杂度标签）。

### 4.3 AI 协作分工

| 角色 | 工具 | 适合的步骤 | 不适合的 |
|------|------|----------|---------|
| 主 agent（你直接对话） | 主流程、Edit/Write、决策 | Step 2, 3, 6, 接口冻结 | 大段 C 代码翻译（污染上下文） |
| `Explore` subagent | 只读搜索 | Step 1（C 源码快速摸排） | 写代码 |
| `general-purpose` subagent | 任意工具 | Step 1, 4, 5, 7, 8, 9（批量翻译/写测试） | 跨多文件设计决策 |
| `Plan` subagent | 设计方案 | 复杂算法（IRLM、BLISS 翻译）的 Step 2 | 简单 AWU |
| `code-reviewer` subagent（如可用） | code review | Step 4 完成后的独立复核 | - |

**并行策略**：同一 Phase 内**无依赖**的 AWU（如不同最短路径算法）可同时启动多个 subagent 在 Step 1 / Step 4 上并行。Step 2（接口冻结）和 Step 6（oracle 集成）必须串行经过主 agent。

### 4.4 AWU 模板文件（Phase 0 BOOT-19~22 创建）

`templates/algo.rs.tpl`（精简示意）：

```rust
//! ALGO-XXX-NNN: <算法名>
//!
//! 对标 igraph C: `<src/.../foo.c>`
//! 参考论文/文档: <可选>

use rust_igraph::{Graph, IgraphError, IgraphResult};

/// <一句话功能描述>
///
/// # Arguments
/// * `graph` - <说明>
/// * `...`
///
/// # Returns
/// <说明>
///
/// # Errors
/// - `IgraphError::InvalidArgument` 当 ...
///
/// # Examples
/// ```
/// // ...
/// ```
pub fn foo(graph: &Graph, /* ... */) -> IgraphResult</* ... */> {
    // TODO(ALGO-XXX-NNN): translate from foo.c
    unimplemented!()
}
```

`templates/test.rs.tpl`：

```rust
//! Tests for ALGO-XXX-NNN

use super::*;

// === 单元测试 ===
#[test] fn empty_graph() { /* ... */ }
#[test] fn single_vertex() { /* ... */ }
#[test] fn complete_graph_k5() { /* ... */ }
#[test] fn directed_vs_undirected() { /* ... */ }

// === Oracle 测试（与 python-igraph 对比）===
#[cfg(feature = "oracle-tests")]
mod oracle {
    use super::*;
    use crate::oracle::assert_oracle_eq;

    #[test] fn karate() { assert_oracle_eq!("foo", load_fixture("karate"), &[]); }
    #[test] fn dolphins() { assert_oracle_eq!("foo", load_fixture("dolphins"), &[]); }
    #[test] fn er_random() { /* ... */ }
}

// === 属性测试 ===
#[cfg(feature = "proptest-harness")]
mod property {
    use proptest::prelude::*;
    proptest! {
        #[test] fn invariant_xxx(g in arb_graph(0..50)) { /* ... */ }
    }
}
```

`scripts/oracle.py`（精简示意，BOOT-09 + 每次 AWU 增量更新）：

```python
import sys, json, igraph as ig

def make_graph(payload):
    g = ig.Graph(n=payload["n"], edges=payload["edges"],
                 directed=payload.get("directed", False))
    return g

def run(algo, g, params):
    if algo == "bfs":
        return list(g.bfs(params["root"])[0])
    if algo == "shortest_paths_dijkstra":
        return g.shortest_paths_dijkstra(weights=params.get("weights"))
    # ...每个新 AWU 在此添加一个 case
    raise NotImplementedError(algo)

if __name__ == "__main__":
    req = json.loads(sys.stdin.read())
    g = make_graph(req["graph"])
    print(json.dumps(run(req["algo"], g, req.get("params", {}))))
```

### 4.5 AWU 退出门（DoD）

每个 AWU PR 必须满足：

- [ ] 9 步全部完成（除非显式 TODO 标记 + Issue 链接）
- [ ] CI 全绿（含新增的 oracle 测试）
- [ ] ALGORITHMS.md 状态从 `wip` 改为 `done`
- [ ] PR 描述链接 ALGO 编号 + igraph C 源文件:行号
- [ ] 性能基准入库（即使是 baseline）
- [ ] rustdoc 含至少 1 个 doctest
- [ ] 不破坏既有 oracle 测试

### 4.6 失败回退

| 失败模式 | 回退路径 |
|---------|---------|
| AI 翻译卡壳 | 状态改 `blocked`，开 Issue 描述卡点，跳到下一个 AWU |
| oracle 数值不一致 | 优先怀疑 Rust 实现；先简化 fixture 二分定位；C 源 + igraph PR 历史查相关 bug |
| 性能 > python-igraph 10x | 标记 `done` 但加 PERF-TODO 标签；不要求 v0.x 性能达标 |
| 测试 fixture 缺失 | 直接生成（小图）或从 networkrepository / SNAP 下载（大图） |
| Phase 退出门长尾难度 | 允许 Phase "近完成"（≥85% AWU done）即解锁下一 Phase |

---

## 五、阶段化推进（Phase 1-10）

> 每个 Phase 都遵守"不变量"：CI 始终绿，oracle 测试集只增不减。

### 5.1 阶段总表

> **2026-06-04 回顾**：原计划 Phase 0-10 按顺序推进，实际执行中 Phase 1
> 吸收了大量跨阶段算法（社区检测、流/割、同构、布局等），截至 v0.6.0 已完成
> 308 个 AWU（1,091 测试），远超原始估计。下表增加了实际完成列。

| Phase | 主题 | AWU 数（估） | 实际完成 | 关键退出门 | 里程碑 |
|-------|------|-------------|---------|----------|-------|
| 0 | 行走骨架 | ~37（BOOT-NN） | **37/37** ✓ | 端到端 BFS oracle 通过 | (前置) |
| 1 | 数据结构 + 核心算法 | ~80 | **269/269** ✓ | Graph 核心 + 查询 + 迭代器 | v0.1.0-v0.4.0 |
| 2 | 遍历 + 最短路径 + 连通性 | ~45 | **全部** ✓ | 16+ 核心算法 | v0.5.0 |
| 3 | 中心性 + 特征值求解器 | ~65 | **大部分** ✓ | Lanczos/Arnoldi/PageRank/HITS/eigenvector/Katz/harmonic | v0.5.0 |
| 4 | 社区检测 | ~22 | **19/22** ✓ | Louvain/Leiden/LPA/Walktrap/EB/FG/Fluid/Infomap/Spinglass/LEV | v0.6.0 |
| 5 | 流/割 + MST + 生成器 | ~55 | **~50** ✓ | Dinic max-flow + Gomory-Hu + all-st-cuts/mincuts + 54 generators | v0.5.0 |
| 6 | 同构 + BLISS + 着色 + 匹配 | ~35 | **~25** ✓ | VF2 + LAD + BLISS I-R canonical + automorphisms + DSatur + bipartite matching | v0.6.0 |
| 7 | 布局 + 环 + 团 + Motif | ~78 | **~45** ✓ | 16 布局引擎 (FR/KK/Sugiyama/GEM/RT/DrL/DH/GraphOpt/MDS/UMAP/LGL) + cliques + motifs | v0.6.0 |
| 8 | 谱方法 + 嵌入 + HRG | ~90 | **10** (EIG-001..003 + LAP-001 + EMB-001..003 + HRG-001..003) | Lanczos + Arnoldi + adj-eigen + Laplacian + spectral embedding + dim_select + HRG (create/sample/fit/consensus/predict) | v0.6.0 |
| 9 | 文件 I/O + 属性系统 | ~75 | **15 I/O + attr** ✓ | 8 种格式 round-trip + attribute system | v0.5.0 |
| 10 | 高层 API + 文档站 + 发布 | ~80 | **大部分** ✓ | 1,297 pub fn + mdBook (中英) + landing page + Playground (React SPA) + WASM + bench CI | v0.6.0 |
| 11（可选） | nauty C FFI 后端 | ~10 | 0 | 大规模图同构性能 | v1.x |

**截至 v0.6.0 实际总计**：315 AWU done，7,718 测试，1,850 conformance fixtures，392 pub fn（~198k SLOC Rust）。

**总 AWU**：约 660 个（Phase 0 + 算法）。

### 5.2 各 Phase 详细 AWU 清单（仅示意，完整列表在 ALGORITHMS.md）

> 完整 AWU 清单（约 660 行表格）作为独立文件 `.codefuse/tracking/ALGORITHMS.md` 维护。这里给出每个 Phase 的代表 AWU 和分组。

#### Phase 1：数据结构主线（编号前缀 ALGO-DS / ALGO-CORE）

| 分组 | 代表 AWU | 数量 | 关键 C 源 |
|------|---------|------|----------|
| Graph 核心 | ALGO-CORE-001 igraph_t 等价结构 | 5 | type_indexededgelist.c (2013 行) |
| 基础查询 | ALGO-CORE-010 vcount/ecount/degree | 15 | basic_query.c (406 行) |
| Vector 系列 | ALGO-DS-V-001..030 | ~30 | vector.c (~2500 行) |
| Matrix 系列 | ALGO-DS-M-001..020 | ~20 | matrix.c (~1500 行) |
| SparseMat | ALGO-DS-S-001..010 | ~10 | sparsemat.c (3251 行) |
| Selectors | ALGO-DS-SEL-001..010 | ~10 | iterators.c (2048 行) |
| Adjlists | ALGO-DS-ADJ-001..005 | ~5 | adjlist.c (1328 行) |

#### Phase 2：遍历 + 最短路径 + 连通性（前缀 ALGO-TR / ALGO-SP / ALGO-CC）

| 算法 | AWU 编号 | C 源 |
|------|---------|------|
| BFS | ALGO-TR-001 | bfs.c |
| DFS | ALGO-TR-002 | dfs.c |
| Random Walk | ALGO-TR-003 | random_walk.c |
| Dijkstra | ALGO-SP-001 | distances_dijkstra*.c |
| Bellman-Ford | ALGO-SP-002 | distances_bellman_ford*.c |
| Johnson | ALGO-SP-003 | distances_johnson.c |
| Floyd-Warshall | ALGO-SP-004 | distances_floyd_warshall.c |
| A* | ALGO-SP-005 | astar.c |
| BFS shortest paths | ALGO-SP-006 | shortest_paths*.c |
| Widest paths | ALGO-SP-010..014 | widest_paths*.c |
| Diameter / Eccentricity / Radius | ALGO-SP-020..023 | diameter.c / eccentricity.c |
| 弱/强连通分量 | ALGO-CC-001..003 | components.c |
| Biconnected / Articulation / Bridges | ALGO-CC-010..014 | biconnected*.c |
| Reachability | ALGO-CC-020..022 | reachability.c |
| Percolation | ALGO-CC-030..032 | percolation.c |
| Eulerian | ALGO-CC-040..042 | eulerian*.c |

#### Phase 3：中心性 + 特征值求解器（前缀 ALGO-LA / ALGO-CT）

特征值求解器（线性代数基础设施，**必须先于依赖 ARPACK 的算法完成**）：

| AWU | 任务 | C 源 |
|----|------|------|
| ALGO-LA-OPT-001 | ArpackOptions / ArpackStorage | igraph_arpack.h |
| ALGO-LA-TRAIT-001 | ArpackSolver trait + EigenSolver trait | (新设计) |
| ALGO-LA-DENSE-001 | 手写 dense EVD（Jacobi/QR，小矩阵 n≤50） | (新) |
| ALGO-LA-BLAS-001..005 | BLAS 桥接（dgemv/dgemm/ddot/...） | blas.c (261 行) |
| ALGO-LA-LAPACK-001..007 | LAPACK 桥接（dgetrf/dgesv/dsyevr/dgeev/dgehrd/...） | lapack.c (1057 行) |
| **ALGO-LA-IRLM-001** | IRLM 对称迭代求解器 | arpack.c → arpack_rssolve* |
| ALGO-LA-IRAM-001 | IRAM 非对称迭代求解器 | arpack.c → arpack_rnsolve* |
| ALGO-LA-EIGEN-001..003 | eigen.c 高层分发 | eigen.c (1466 行) |

中心性算法：

| AWU | 算法 | 依赖 | C 行数 |
|----|------|------|-------|
| ALGO-CT-001 | 度中心性 / 强度 | - | ~100 |
| ALGO-CT-002 | 介数中心性（含截断） | - | 1404 |
| ALGO-CT-003 | 接近中心性（含截断 / Harmonic） | - | 805 |
| ALGO-CT-004 | 特征向量中心性 ★ | ALGO-LA-IRLM | 646 |
| ALGO-CT-005 | PageRank ★ | ALGO-LA-IRAM（仅 ARPACK 模式）+ 自研幂迭代 | 721 |
| ALGO-CT-006 | HITS ★ | ALGO-LA-IRLM | 505 |
| ALGO-CT-007..013 | Centralization 7 个辅助 | - | 723 |
| ALGO-CT-020 | Coreness（K-core） | - | 157 |
| ALGO-CT-021 | Trussness | - | 288 |

#### Phase 4：社区检测（前缀 ALGO-CM）

| AWU | 算法 | 依赖 |
|----|------|------|
| ALGO-CM-001 | Modularity + Modularity matrix | - |
| ALGO-CM-002 | Louvain (multilevel) | ALGO-CM-001 |
| ALGO-CM-003 | Leiden | ALGO-CM-001 |
| ALGO-CM-004 | Label Propagation | - |
| ALGO-CM-005 | Fast Greedy | ALGO-CM-001 |
| ALGO-CM-006 | Edge Betweenness（Girvan-Newman） | ALGO-CT-002 |
| ALGO-CM-007 | Leading Eigenvector ★ | ALGO-LA-IRLM |
| ALGO-CM-008 | Walktrap（C++ → Rust） | - |
| ALGO-CM-009 | Infomap（C++ → Rust） | - |
| ALGO-CM-010 | Spinglass（C++ → Rust） | - |
| ALGO-CM-011 | Fluid Communities | - |
| ALGO-CM-012 | Voronoi Communities | - |
| ALGO-CM-013..017 | 比较 / 成员操作 | - |
| ALGO-CM-020 | Optimal Modularity（GLPK 或 LP 替代） | feature gated |

#### Phase 5：流/割 + MST + 生成器（前缀 ALGO-FL / ALGO-MST / ALGO-GN）

| 分组 | AWU | 备注 |
|------|-----|------|
| 最大流 | ALGO-FL-001 Push-Relabel / ALGO-FL-002 Edmonds-Karp / ALGO-FL-003 Dinic | maxflow.c |
| 割 | ALGO-FL-010..015（mincut, edge/vertex connectivity, st_mincut） | st_mincut*.c |
| Gomory-Hu | ALGO-FL-020 | gomory_hu.c |
| Dominator | ALGO-FL-030 | dominator_tree.c |
| All ST cuts | ALGO-FL-040..041 | all_st_cuts*.c |
| MST | ALGO-MST-001 Prim / ALGO-MST-002 Kruskal | spanning_trees.c (620 行) |
| 生成器 | ALGO-GN-001..033（ER, BA, WS, SBM, lattice, tree, famous, ...） | 33 个 |

#### Phase 6：同构 + BLISS + 着色 + 匹配（前缀 ALGO-ISO / ALGO-BLI / ALGO-CL / ALGO-MA）

| 分组 | AWU | 备注 |
|------|-----|------|
| VF2 | ALGO-ISO-001..008（含 colored / count / get） | vf2.c (1741 行) |
| isoclass | ALGO-ISO-010..013（查表 + 高层分发） | isoclasses.c (2936 行) |
| LAD 子图同构 | ALGO-ISO-020 | lad.c (1646 行) |
| simplify_and_colorize | ALGO-ISO-030 | simplify_and_colorize.c |
| BLISS 基础设施 | ALGO-BLI-001..006（heap/kstack/orbit/uintseqhash/defs/utils） | bliss/*.cc |
| BLISS Partition | ALGO-BLI-010 | partition.cc (1127 行) |
| BLISS AbstractGraph + Graph + Digraph | ALGO-BLI-020..025 | graph.cc (5035 行)，按 6 启发式拆 |
| BLISS 桥接 API | ALGO-BLI-030 | bliss.cc (764 行) |
| 着色 | ALGO-CL-001 DSATUR / ALGO-CL-002 ColoredNeighbors / ALGO-CL-003 验证 | coloring.c (519 行) |
| 匹配 | ALGO-MA-001 二部图（Hungarian） | bipartite_matching (1013 行) |

#### Phase 7：布局 + 环 + 团 + Motif（前缀 ALGO-LO / ALGO-CY / ALGO-CQ / ALGO-MO）

| 分组 | AWU | 备注 |
|------|-----|------|
| 简单布局 | ALGO-LO-001..007（random/circle/star/sphere/grid/tree/3D） | layout_*.c |
| Reingold-Tilford | ALGO-LO-010..011 | reingold_tilford*.c |
| FR | ALGO-LO-020..021 | fruchterman_reingold*.c (676 行) |
| KK | ALGO-LO-030..031 | kamada_kawai*.c (702 行) |
| MDS ★ | ALGO-LO-040 | layout_mds.c (295 行)，依赖 IRLM |
| Sugiyama | ALGO-LO-050 | sugiyama.c (1309 行) |
| UMAP | ALGO-LO-060..061 | umap*.c |
| DrL | ALGO-LO-070..071（C++ 翻译） | drl*.c (3456 行) |
| GEM / DH / GraphOpt | ALGO-LO-080..082 | gem.c / davidson_harel.c |
| 布局合并 | ALGO-LO-090..092 | merge_dla.c |
| 团搜索（cliquer） | ALGO-CQ-001..018 | maximal_cliques*.c (4583 行) |
| Motif / Dyad / Triad | ALGO-MO-001..006 | motifs_randesu*.c (1223 行) |
| Graphlet | ALGO-MO-010..012 | graphlets*.c (881 行) |
| 环检测 | ALGO-CY-001..007 | simple_cycles*.c (2519 行) |

#### Phase 8：谱方法 + 嵌入 + 剩余（前缀 ALGO-SP2 / ALGO-EM / ALGO-EP / ALGO-HRG / ...）

#### Phase 9：文件 I/O + 属性系统（前缀 ALGO-IO / ALGO-AT）

#### Phase 10：稀疏矩阵高级 + 高层 API + 发布（前缀 ALGO-SX / ALGO-API / ALGO-RL）

> Phase 8/9/10 的完整清单见 ALGORITHMS.md。

### 5.3 跨阶段不变约束

无论在哪个 Phase，下面的事实始终为真：

1. `cargo test --workspace` 通过
2. `cargo bench` 全部 baseline 跑通（不要求性能达标）
3. CI 全绿
4. ALGORITHMS.md 状态与代码现实一致
5. 至少 1 个 example 跑通（Phase 0 的 BFS demo 不能坏）
6. WASM check 通过（默认 features）

---

## 六、测试体系

### 6.1 五层测试金字塔（整合 igraph / python-igraph / R-igraph 全部测试资产）

```
Layer 1: 单元测试  (#[test])              覆盖：API 参数、错误码、边界
Layer 2: 属性测试  (proptest)             覆盖：不变量（对称性、保持性、合法性）
Layer 3: Live Oracle  (python-igraph)     覆盖：数值正确性，运行时对比（覆盖 100% API）
Layer 4: 静态 Conformance（多源融合）      覆盖：igraph C tests/.out + python-igraph tests + R-igraph testthat
Layer 5: Fuzz / 大规模回归                覆盖：cargo-fuzz + nightly 1M+ 顶点压力
```

**关键原则**：igraph 三家官方实现的测试资产**全部纳入**，分工如下：

| 来源 | 资产规模 | 复用方式 | 价值 |
|------|---------|---------|------|
| **igraph C** `tests/unit/*.c` + `*.out` | ~425 程序 + ~302 .out | 自动提取（脚本）→ Rust conformance fixture | 权威边界条件 + 浮点精度参考；直接对应 .out 文件 |
| **python-igraph** `tests/test_*.py` | ~526 测试方法 / 30 文件 | (a) 翻译为 Rust 测试；(b) 同时作为 live oracle | 测试风格自包含，逻辑可直译；oracle 自动覆盖每个 API |
| **R-igraph** `tests/testthat/test-*.R` | 108+ 自动绑定函数 + 手写测试 | 翻译为 Rust 测试；少量作为补充 oracle | R 绑定有 igraph 自动函数测试，覆盖面最广 |

> **三家测试不重复**：C 测试侧重底层 / 边界 / .out diff；python-igraph 测试侧重高层 API + 异常场景；R-igraph 测试侧重自动绑定的全函数粗筛。三家叠加才能逼近 100% 覆盖。

### 6.2 标准 fixture 集

| Fixture | 顶点 | 边 | 用途 | 来源 |
|---------|------|----|----|------|
| empty | 0 | 0 | 边界 | 内置 |
| single | 1 | 0 | 边界 | 内置 |
| k5 | 5 | 10 | 完全图 | igraph famous() |
| star10 | 10 | 9 | 星图 | igraph famous() |
| ring10 | 10 | 10 | 环图 | igraph famous() |
| petersen | 10 | 15 | 经典 | igraph famous() |
| karate | 34 | 78 | 社区检测金标准 | igraph famous() |
| dolphins | 62 | 159 | 中型社区 | networkrepository |
| les_mis | 77 | 254 | 中心性参考 | igraph famous() |
| polbooks | 105 | 441 | 政治网络 | networkrepository |
| er_n100_p005 | 100 | ~250 | 随机基准 | seed=42 |
| ba_n1000_m3 | 1000 | ~3000 | 大图基准 | seed=42 |
| strongly_regular_27_16_10_8 | 27 | 216 | BLISS 困难图 | 内置 |

每个 fixture 同时存在三种格式：`fixtures/<name>.edges` + `<name>.gml` + `<name>.json`（含 oracle 期望输出索引）。

### 6.3 Oracle 协议

oracle.py 接口契约：

```json
// stdin
{
  "graph": {
    "n": 34,
    "edges": [[0, 1], [0, 2], ...],
    "directed": false,
    "weights": null
  },
  "algo": "betweenness",
  "params": {"vertices": null, "weights": null, "cutoff": -1}
}

// stdout
{
  "ok": true,
  "result": [0.0, 1.5, ...],
  "version": "python-igraph 0.11.x",
  "elapsed_s": 0.0023
}

// 或者
{ "ok": false, "error": "...", "version": "..." }
```

容差规则：

| 结果类型 | 容差 |
|---------|------|
| 整数 / bool / 集合 | 完全相等 |
| 浮点单值 | abs ≤ 1e-10 OR rel ≤ 1e-8 |
| 浮点向量 | 同上，逐元素 |
| 排列（如 BFS 顺序） | 在算法允许的等价类下相等 |
| 集合划分（如社区） | 用 NMI / 调整后兰德指数比较，≥ 0.99 |
| 不稳定算法（含随机性） | 验证不变量而非具体值 |

### 6.4 性能回归基线

每个 AWU 的 Step 8 把 baseline 写入 `.codefuse/tracking/perf/<ALGO-XXX>.json`。CI 在 PR 上运行差异：
- 同算法新版本回归 > 20% → CI failure（除非 PR title 含 `[perf-allowed]`）
- python-igraph 对比劣化 > 10x → 标 PERF-TODO Issue

### 6.5 测试 fixture 自动化提取（三家官方实现）

提取脚本统一存放在 `scripts/test_extract/`，输出到 `tests/conformance/<source>/<algo>/*.json`。三家官方实现各有一套提取器：

#### 6.5.1 igraph C (`scripts/test_extract/from_c.py`)

- **输入**：`igraph/tests/unit/*.c` 程序源码 + `igraph/tests/unit/*.out` 期望输出
- **解析方式**：识别 `igraph_small()` / `igraph_create()` / `IGRAPH_CHECK()` 等模式提取图构造；用 .out 文件作为期望结果
- **输出**：`tests/conformance/c/<algo_name>/<test_name>.json`，结构为 `{ graph: {n, edges}, params: {...}, expected_out: "..." }`
- **覆盖**：~425 测试 + ~302 .out 文件
- **Phase 时机**：Phase 1 末完成，伴随 ALGORITHMS.md 状态推进自动跑

#### 6.5.2 python-igraph (`scripts/test_extract/from_py.py`)

- **输入**：`python-igraph/tests/test_*.py`
- **解析方式**：用 Python AST 找到 `unittest.TestCase` 子类的所有 `test_*` 方法；每个方法分两路处理：
  1. **Live oracle 路径**：在测试中提取图构造 + 算法调用 → 加到 `scripts/oracle.py` 的 case 表；Rust 侧用 `assert_oracle_eq!` 对比
  2. **Static fixture 路径**：执行 Python 测试一次，把图 + 输入 + 输出 dump 为 JSON，存 `tests/conformance/py/<module>/<test_name>.json`
- **输出**：双格式（live + static），互相校验
- **覆盖**：~526 测试方法（30 文件）
- **Phase 时机**：Phase 0 BOOT-29（新增）做骨架；Phase 1 起每完成一个算法 AWU 时提取对应的 py 测试

#### 6.5.3 R-igraph (`scripts/test_extract/from_r.py` + `scripts/test_extract/run_r.R`)

- **输入**：`rigraph/tests/testthat/test-*.R`
- **解析方式**：
  1. R 脚本（`run_r.R`）批量执行所有 testthat 文件，**捕获每个 `expect_equal(actual, expected)` 的 actual 和 expected 值**，序列化为 JSON
  2. Python 脚本聚合 R 输出 → `tests/conformance/r/<test_file>/<expectation>.json`
- **输出**：`tests/conformance/r/*.json`，结构 `{ graph: {...}, fn: "betweenness", args: {...}, expected: ... }`
- **覆盖**：108+ 自动绑定函数 + 手写测试方法
- **Phase 时机**：Phase 1 末与 C 提取脚本同步完成；R-igraph 测试**侧重自动覆盖每个 API 一次粗筛**，对长尾 API 价值最大

#### 6.5.4 Rust 侧统一加载

```rust
// tests/conformance/mod.rs
pub fn load_conformance(source: &str, algo: &str) -> Vec<ConformanceCase> {
    // 自动扫 tests/conformance/{c,py,r}/<algo>/*.json
}

#[test]
fn conformance_c_betweenness() {
    for case in load_conformance("c", "betweenness") {
        let g = case.graph.to_graph();
        let result = rust_igraph::centrality::betweenness(&g, &case.params).unwrap();
        case.assert_matches(&result);  // 容差按 6.3 节
    }
}
```

每个 AWU 的 Step 6（Oracle Test）扩展为：

> **Step 6（更新）**：除 live oracle 之外，扫描 `tests/conformance/{c,py,r}/<algo>/` 下所有 case，全部加入测试。**新算法上线时必须把三家的 conformance fixture 一次性入库。**

### 6.6 测试资产整合总策略

**目标**：让 igraph C 20 年积累的测试 + python-igraph 526 个测试 + R-igraph 100+ 测试，全部成为 rust-igraph 的回归屏障，不重复造轮子。

| 阶段 | 动作 | 产出 |
|------|------|------|
| Phase 0 (BOOT-29~32) | 三家提取脚本骨架 + oracle.py 双格式扩展 | `scripts/test_extract/` 三脚本 + 1 个 demo（BFS）三家全跑通 |
| Phase 1 末 | 提取脚本完整化 + CI 集成 | conformance 测试 job 进 CI 矩阵；nightly 全量跑 |
| Phase 2 起 | 每个 AWU 的 Step 6 强制扫描三家 | AWU 验收门：三家 conformance 全绿才算 done |
| Phase 10（v1.0 前） | 测试覆盖率审计 | 三家 conformance 总数 ≥ 1000 case；任何官方测试遗漏需开 Issue |

**追踪指标**（写入 `.codefuse/tracking/CONFORMANCE.md`）：

```
| 算法 | C 测试数 | py 测试数 | R 测试数 | rust 通过 | 跳过原因 |
|------|---------|----------|---------|----------|---------|
| bfs  | 5       | 12       | 3       | 20/20    | -       |
| ...  |         |          |         |          |         |
```

---

## 七、工程实践

### 7.1 CI 矩阵（从 Phase 0 day 1 开始）

| Job | 触发 | 要求 |
|-----|------|------|
| fmt | push, PR | `cargo fmt --check` |
| clippy | push, PR | `cargo clippy --workspace --all-targets -- -D warnings` |
| test (stable) | push, PR | `cargo test --workspace` |
| test (beta) | push, PR | 允许失败（监控） |
| test (MSRV) | push, PR | MSRV 锁定 1.85（确保不退化） |
| oracle | push, PR | `cargo test --features oracle-tests`（需 python + python-igraph） |
| conformance | push, PR | 跑 `tests/conformance/{c,py,r}/` 全部 fixture（需 python-igraph + R + rigraph） |
| wasm32 | push, PR | `cargo check --target wasm32-unknown-unknown` |
| deny | push, PR | `cargo deny check`（许可证 + advisory） |
| doc | push | `cargo doc --no-deps`，部署 GitHub Pages |
| coverage | weekly | `cargo llvm-cov`，目标 ≥ 70% |
| bench | nightly | criterion + bench_compare.py，回归告警 |
| oracle-full | nightly | 全 fixture × 全 AWU 矩阵（断点续跑） |
| big-graph | weekly | 1M+ 顶点压力测试 |

### 7.2 版本与发布

| 版本 | 含义 | crates.io? |
|------|------|----------|
| 0.0.x | Phase 0 内部 | 不发 |
| 0.1.0 | Phase 1 完成（数据结构 + BFS oracle） | ✓ |
| 0.2-0.9 | 每 Phase 完成发一版 | ✓ |
| 1.0.0 | Phase 10 完成；850 API 覆盖；性能基线达标 | ✓（正式） |
| 1.x | nauty 后端、性能优化、bug 修 | ✓ |

semver 严格遵守：0.x.y 每个 minor 都是 breaking。

### 7.3 跟踪文档（必须维护）

| 文档 | 路径 | 维护节奏 |
|------|------|---------|
| CLAUDE.md | 仓库根 | 项目硬约束 / AWU 流程入口 / 修改 AI 行为时更 |
| ALGORITHMS.md | .codefuse/tracking/ | **每个 PR 必更**（AWU 状态） |
| ARCHITECTURE.md | .codefuse/tracking/ | 重大决策时更（ADR 形式） |
| CONFORMANCE.md | .codefuse/tracking/ | 三源测试覆盖矩阵；每 AWU Step 6 后更 |
| RESUME.md | .codefuse/tracking/ | 兼职断档恢复指南 |
| RETRO.md | .codefuse/tracking/ | 每月一次复盘 |
| CHANGELOG.md | 仓库根 | 每发版 |
| AI_PROMPTS.md | .codefuse/tracking/ | 积累有效的 AI prompt 模板 |
| perf/<ALGO>.json | .codefuse/tracking/perf/ | 每 AWU |
| .claude/agents/*.md | .claude/agents/ | 子代理定义；新增/调整能力时更 |
| .claude/skills/*/SKILL.md | .claude/skills/ | 工作流；新增 SOP 步骤时更 |
| docs/api_diff.md | docs/ | 与 igraph C API 对照表（自动生成） |

### 7.4 算法状态机

```
todo  ──→  wip  ──→  review  ──→  done  ──→  verified
                ↘             ↘
              blocked      perf-todo
```

- `blocked`: 等前置 AWU / 等外部决策
- `perf-todo`: 功能正确，但性能远落后 python-igraph，有 Issue 跟踪
- `verified`: 通过 nightly 全量 oracle 7 天无回归

### 7.5 Commit / PR 规范

Commit message：
```
<type>(<scope>): <ALGO-XXX> short description

Body 描述 why。
```

类型：`feat / fix / test / docs / refactor / perf / chore / wip`  
scope：`core / algo-xxx / oracle / ci / build / templates / docs`

PR 模板（`.github/pull_request_template.md`）：
```
## ALGO-XXX <算法名>

### C 参考
- 文件: igraph/src/.../foo.c (约 NNN 行)
- 测试: igraph/tests/unit/foo.c

### 实现要点


### Oracle / 测试覆盖
- [ ] 单元测试（empty / single / complete / random）
- [ ] Oracle 测试（karate + 至少 2 个 fixture）
- [ ] proptest 不变量
- [ ] criterion baseline

### Checklist
- [ ] cargo test
- [ ] cargo clippy / fmt
- [ ] cargo doc 含 doctest
- [ ] WASM check
- [ ] ALGORITHMS.md 状态更新
- [ ] perf/<ALGO>.json 入库
```

### 7.6 资料目录（必读 igraph 源文件）

> 路径相对于 `references/`（见第 11 章）。如未克隆，先按 README.md 指引克隆三家官方仓库。

#### ARPACK 替换必读

| # | 路径 | 行数 | 用途 |
|---|------|------|------|
| 1 | `references/igraph/src/linalg/arpack.c` | 1634 | IRLM/IRAM 翻译参照 |
| 2 | `references/igraph/src/linalg/eigen.c` | 1466 | 特征值分发逻辑 |
| 3 | `references/igraph/src/linalg/blas.c` | 261 | BLAS 桥接 |
| 4 | `references/igraph/src/linalg/lapack.c` | 1057 | LAPACK 桥接 |
| 5 | `references/igraph/include/igraph_arpack.h` | - | ARPACK API 定义 |
| 6 | `references/igraph/include/igraph_eigen.h` | - | 特征值 API 定义 |

#### BLISS 翻译必读

| # | 路径 | 行数 | 用途 |
|---|------|------|------|
| 7 | `references/igraph/src/isomorphism/bliss/graph.cc` | 5035 | BLISS 核心 |
| 8 | `references/igraph/src/isomorphism/bliss/partition.cc` | 1127 | 分割细化 |
| 9 | `references/igraph/src/isomorphism/bliss.cc` | 764 | igraph 桥接 |
| 10 | `references/igraph/src/isomorphism/vf2.c` | 1741 | VF2 翻译 |
| 11 | `references/igraph/src/isomorphism/lad.c` | 1646 | LAD 翻译 |
| 12 | `references/igraph/src/isomorphism/isoclasses.c` | 2936 | 查表数据 |

#### 关键测试文件（ARPACK 验证必过）

- `references/igraph/tests/unit/igraph_arpack_rnsolve.c`
- `references/igraph/tests/unit/igraph_eigen_matrix_symmetric_arpack.c`
- `references/igraph/tests/unit/igraph_eigenvector_centrality.c`
- `references/igraph/tests/unit/igraph_pagerank.c`
- `references/igraph/tests/unit/igraph_lapack_dsyevr.c`

#### 关键测试文件（BLISS 验证必过）

- `references/igraph/tests/unit/bliss_automorphisms.c`
- `references/igraph/tests/unit/igraph_isomorphic_bliss.c`
- `references/igraph/tests/unit/isomorphism_test.c`

#### python-igraph 测试目录（Layer 4 conformance 来源）

- `references/python-igraph/tests/test_*.py`（约 30 文件，526 测试方法）

#### R-igraph 测试目录（Layer 4 conformance 来源）

- `references/rigraph/tests/testthat/test-*.R`（108+ 自动绑定函数 + 手写测试）

---

## 八、AI 工程化实践（Agent / Skill / Hook / Memory / MCP）

> AI 时代的工程实践不是"让 Claude 干活"，而是把 AI 的能力**沉淀为仓库资产**：可复用的子代理（Agent）、可触发的技能（Skill）、自动化的钩子（Hook）、跨会话的记忆（Memory）、跨工具的上下文（MCP）。这些资产与代码一同 commit，成为团队（哪怕只有 1 人 + AI）的"操作系统"。

### 8.1 仓库内 AI 资产布局（`.claude/`）

```
.claude/
├── settings.json                  # 团队级 Claude Code 配置（提交到仓库）
├── settings.local.json            # 个人本地覆盖（gitignore）
├── agents/                        # 自定义子代理定义（per-repo）
│   ├── igraph-c-recon.md          # 读 igraph C 源 + 测试 + .out 输出摘要
│   ├── awu-translator.md          # 1:1 翻译 C → Rust，遵守 AWU 模板
│   ├── awu-tester.md              # 写单元 + oracle + proptest 三件套
│   ├── conformance-extractor.md   # 调用 from_c.py / from_py.py / from_r.R 提取 fixture
│   ├── numerical-reviewer.md      # 复核数值算法（容差、收敛、稳定性）
│   ├── perf-bencher.md            # 写 criterion + 跑 bench_compare.py
│   └── doc-writer.md              # 写 rustdoc + doctest + 示例
├── skills/                        # 可复用工作流（user 输入 /skill-name 触发）
│   ├── awu-start/                 # 启动一个 AWU（生成骨架 + 占位）
│   │   └── SKILL.md
│   ├── awu-finish/                # 收尾 AWU（更新 ALGORITHMS.md + PR 模板填充）
│   │   └── SKILL.md
│   ├── oracle-add/                # 在 oracle.py 添加新算法 case
│   │   └── SKILL.md
│   ├── conformance-sweep/         # 三家 conformance 全提取 + 入库
│   │   └── SKILL.md
│   ├── phase-checkpoint/          # Phase 退出门检查（跑全量 + 写 retro）
│   │   └── SKILL.md
│   └── resume-session/            # 断档后恢复（读 RESUME.md + ALGORITHMS.md）
│       └── SKILL.md
├── hooks/                         # 自动触发的 shell 命令
│   ├── post-edit-rust.sh          # *.rs 编辑后自动跑 cargo fmt + clippy
│   ├── pre-commit.sh              # commit 前跑 cargo test 受影响 crate
│   └── post-tool-bash.sh          # Bash 工具用后日志（追溯 AI 实际跑过什么）
├── commands/                      # （可选）斜杠命令快捷入口
└── memory/                        # 持久记忆（自动管理，由 Claude Code 维护）
    ├── MEMORY.md
    ├── user_role.md
    ├── feedback_*.md
    └── project_*.md
```

仓库根的 **CLAUDE.md** 作为项目级"系统提示"，描述项目硬约束（GPL 许可证、不引入 GPL-3 / petgraph、AWU 流程、PR 模板等）。

### 8.2 自定义子代理（Agent）目录

每个 agent 是 `.claude/agents/<name>.md`，含 frontmatter（描述、工具白名单、模型偏好）+ 系统提示。**主 agent 通过 Agent 工具调用，传入聚焦的 prompt。**

| Agent | 主要工具 | 触发场景 | 模型偏好 |
|-------|---------|---------|---------|
| `igraph-c-recon` | Read, Bash(grep/find), WebFetch | AWU Step 1（摸排 C 源 + 测试） | haiku（快、便宜） |
| `awu-translator` | Read, Write, Edit, Bash(cargo) | AWU Step 4（C → Rust 翻译） | sonnet（平衡） |
| `awu-tester` | Read, Write, Edit, Bash(cargo test) | AWU Step 5+7（单元 + proptest） | sonnet |
| `conformance-extractor` | Bash, Write | AWU Step 6b（三源 fixture 提取） | haiku |
| `numerical-reviewer` | Read, Bash(python oracle) | 数值算法（IRLM/BLISS/PageRank）专项复核 | opus（深度推理） |
| `perf-bencher` | Read, Write, Bash(cargo bench) | AWU Step 8 | haiku |
| `doc-writer` | Read, Edit | AWU Step 9（rustdoc + 示例） | haiku |

#### 示例：`.claude/agents/awu-translator.md`

```markdown
---
name: awu-translator
description: Translates a single igraph C function to Rust under the AWU pipeline. Use when an AWU's interface is frozen and Step 4 needs implementation. Operates in isolation; reads only the specified C source range and target template.
tools: Read, Write, Edit, Bash, Glob, Grep
model: sonnet
---

You are translating a single algorithm from igraph C to rust-igraph (Rust + GPL-2.0-or-later).

Hard constraints (NEVER violate):
1. No `unsafe` blocks unless explicitly approved in CLAUDE.md.
2. No `unwrap()` / `expect()` outside tests.
3. No new dependencies unless listed in ARCHITECTURE.md.
4. Match igraph C error codes to IgraphError variants.
5. Floating-point comparisons use tolerance helpers from `rust_igraph::testutil`, never `==`.
6. Public API requires rustdoc with at least one doctest.
7. Do not modify `templates/`, `scripts/`, `.github/`, or other AWUs' code.

Workflow:
1. Read the C source range provided in your task prompt.
2. Read the frozen Rust signature from the target file (already created from `templates/algo.rs.tpl`).
3. Replace the `unimplemented!()` body with a faithful Rust translation.
4. Run `cargo build` to confirm it compiles.
5. Run `cargo clippy -- -D warnings` and fix.
6. Output a 5-line summary of the translation choices (data structures, allocations, deviations from C).

Do NOT:
- Run oracle tests (that is the awu-tester agent's job).
- Add bench code (that is perf-bencher's job).
- Modify ALGORITHMS.md (the main agent does this).
- Touch other algorithms' files.
```

### 8.3 Skill 目录（可复用工作流）

Skill 比 Agent 更轻量：用户输入 `/awu-start ALGO-CT-002` 即触发，本质是带参数的 prompt 模板 + 一组建议的工具调用。

#### 示例：`.claude/skills/awu-start/SKILL.md`

```markdown
---
name: awu-start
description: Bootstrap a new AWU. Reads ALGORITHMS.md to find the AWU, reads the referenced igraph C source, drafts an interface, creates the skeleton file from templates/, and updates ALGORITHMS.md status to wip. Args: ALGO-XXX-NNN.
---

When the user invokes /awu-start ALGO-XXX-NNN:

1. Read `.codefuse/tracking/ALGORITHMS.md`, find the row for ALGO-XXX-NNN.
2. Read the referenced C source file (and headers if line ranges given).
3. Spawn `igraph-c-recon` agent to summarize.
4. Draft the Rust public signature in a comment to the user; ask confirmation BEFORE writing files.
5. After confirmation: copy `templates/algo.rs.tpl` to the target module path, fill in name/doc/signature, leave body as `unimplemented!()`.
6. Copy `templates/test.rs.tpl` to the test path with empty test bodies.
7. Append the new AWU to `scripts/oracle.py` skeleton (commented placeholder).
8. Run `cargo build` to confirm skeleton compiles.
9. Update ALGORITHMS.md row status: todo → wip, with date.
10. Print next-step suggestion: "Run /awu-translate ALGO-XXX-NNN once interface is approved."
```

完整 Skill 列表：

| Skill | 触发 | 作用 | 调用的 Agent |
|-------|------|------|------------|
| `/awu-start` | 启动 AWU | 摸排 + 接口冻结 + 骨架 | igraph-c-recon |
| `/awu-translate` | 实现 | C → Rust 翻译 | awu-translator |
| `/awu-test` | 测试 | 单元 + proptest | awu-tester |
| `/awu-conformance` | 三源 conformance | 提取 + 跑 fixture | conformance-extractor |
| `/awu-bench` | 基准 | criterion + 对比 | perf-bencher |
| `/awu-finish` | 收尾 | rustdoc + 状态机 + PR 模板 | doc-writer |
| `/oracle-add` | 加 oracle case | 维护 oracle.py | (主 agent) |
| `/phase-checkpoint` | Phase 退出门 | 跑全量 + 写 RETRO.md | numerical-reviewer + perf-bencher |
| `/resume-session` | 断档恢复 | 读 RESUME.md + 当前 wip AWU | (主 agent) |

### 8.4 Hooks（自动化触发器）

`.claude/settings.json` 注册 hooks，在事件发生时自动跑 shell：

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Edit|Write",
        "hooks": [
          { "type": "command", "command": ".claude/hooks/post-edit-rust.sh \"$TOOL_FILE_PATH\"" }
        ]
      }
    ],
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          { "type": "command", "command": ".claude/hooks/pre-bash-audit.sh" }
        ]
      }
    ]
  }
}
```

`post-edit-rust.sh`（示意）：

```bash
#!/usr/bin/env bash
# 编辑 *.rs 后自动跑 fmt + clippy on 受影响 crate
file="$1"
case "$file" in
    src/core/*.rs)       cargo fmt; cargo clippy -- -D warnings ;;
    src/algorithms/*.rs) cargo fmt; cargo clippy -- -D warnings ;;
    *.rs)                          cargo fmt --all ;;
esac
```

**Hook 用途清单**：

| Hook | 时机 | 作用 |
|------|------|------|
| post-edit-rust | 任何 .rs 编辑后 | fmt + clippy 即时反馈 |
| pre-commit | 提交前 | 跑受影响 crate 的 cargo test |
| post-tool-bash | 每次 Bash 工具用后 | 日志追溯（写 .codefuse/tracking/agent_actions.log） |
| stop-hook | 会话结束前 | 检查 ALGORITHMS.md 状态变化、未提交改动 |

### 8.5 Memory（跨会话记忆）

Claude Code 自带 file-based memory（`~/.claude/projects/.../memory/`）。**对兼职 + AI 模式至关重要**：断档数周后能 5 分钟恢复上下文。

**该写入 memory 的内容**：

| 类型 | 例子 | 何时写 |
|------|------|------|
| `user` | "用户偏好简洁回复，不要在文件里加 emoji" | 用户表达偏好时 |
| `feedback` | "Phase 0 不要追求完美数据结构，先跑通 BFS oracle" | 用户纠正方向时 |
| `project` | "已决定不依赖 petgraph；切换到 GPL-2.0+ 的原因是参照翻译" | 重大决策时 |
| `reference` | "完整算法清单见 .codefuse/tracking/ALGORITHMS.md" | 引入外部资源时 |

**不写入 memory 的内容**（避免污染）：
- 当前 wip 的具体算法细节（写在 ALGORITHMS.md / RESUME.md）
- 代码片段（仓库本身就是真相源）
- 临时调试信息

### 8.6 MCP 服务器（可选，按需扩展）

如有 MCP 服务器（GitHub / Linear / Yuque / 内部 ArkAI 等），用于：

| MCP | 用途 |
|-----|------|
| GitHub MCP / `gh` CLI | 创建 Issue（blocked AWU 自动开 Issue）、查询 PR、跑 CI |
| Yuque / Skylark MCP | 把 ARCHITECTURE.md 同步到团队知识库 |
| 内部任务 MCP | AWU 状态同步到外部任务系统 |

**注意**：尽量不依赖 MCP 跑核心流程（保证仓库自包含）；MCP 仅做边缘集成。

### 8.7 上下文最小化与 Prompt 缓存

每次主 agent 启动 Skill / 调用 subagent 时只带：

1. AWU 编号 + ALGORITHMS.md 中的 1 行（不是全表）
2. 相关 igraph C 源文件**指定行号范围**（不是整个文件）
3. 已冻结的 Rust 接口签名
4. `templates/` 下相关模板
5. 1-2 个邻居 AWU 的**最终成品文件**（作为风格参考）

**不带**：会话历史、完整 ALGORITHMS.md、其它 Phase 代码、长设计讨论。

利用 **prompt caching**（Anthropic API 自动）：把不变内容（CLAUDE.md、模板、ARCHITECTURE.md）放在 prompt 前部，缓存命中后续调用便宜 10x。

### 8.8 AI Prompt 模板沉淀（`AI_PROMPTS.md`）

每次发现一个有效 prompt（让 AI 一次答对）就写入 `.codefuse/tracking/AI_PROMPTS.md`，标注：

```markdown
## Recon prompt（适用于 AWU Step 1）

**最后更新**: 2026-05-15  
**适用**: 中等规模算法（C 源 < 1000 行）  
**命中率**: 8/10 一次过  

\`\`\`
你的任务是为 {ALGO-ID} 做 Recon。读以下文件，输出摘要（≤300 字）：
C 源: {path}:{line_start}-{line_end}
头文件: {header_path}
测试: {test_path}

输出格式：
1) 函数签名（C → 推荐 Rust 签名）
2) 输入约束
3) 输出格式
4) 边界条件
5) 数值精度提示
6) 推荐 fixture 列表
\`\`\`

**踩过的坑**：要求"不超过 300 字"很关键，否则 AI 会贴大段 C 源。
```

### 8.9 失败回退与人工介入信号

| 信号 | 回退 |
|------|------|
| AI 翻译卡住，3 次都不对 | 状态置 `blocked`，开 Issue，跳到下一 AWU |
| Oracle 数值偏差 > 容差 | 主 agent 介入，二分 fixture 定位；不让 subagent 反复试 |
| Subagent 输出包含 hallucinated API | 缩小输入上下文，重启 subagent |
| Skill 多步失败 | 退化为手动一步步走（保留 Skill log） |
| Hook 反复被打断 | 降级到 manual 触发，调试后再启用 |

### 8.10 review checklist（AI 输出后的人工复核）

每次 AI 完成 Step 4（实现）后，主 agent 必须复核：

- [ ] 没有 `unsafe` 块（除非已批准）
- [ ] 没有 `unwrap()` / `expect()`（除非测试或显式约束）
- [ ] 错误码与 igraph C 对应
- [ ] 浮点对比用容差，不用 `==`
- [ ] 整数有 `checked_*` 防溢出
- [ ] 公共 API 都有 rustdoc
- [ ] 不引入新依赖（除非 ARCHITECTURE.md 批准）
- [ ] 没有"为未来留口子"的 dead code（YAGNI）
- [ ] 没有不必要的 comments（违反 CLAUDE.md 规则）

### 8.11 AI 工程实践成熟度路标

| 阶段 | 必备 AI 资产 |
|------|------------|
| Phase 0 | CLAUDE.md + 6 个 agent + 5 个 skill + 3 个 hook + AI_PROMPTS.md 骨架 |
| Phase 1 末 | 全 9 个 skill 完整；agents 模型分配优化（haiku/sonnet/opus） |
| Phase 5 末 | conformance-extractor agent 自动化三源提取；perf-bencher 自动写报告 |
| Phase 10（v1.0 前） | 所有 SOP 步骤 ≥80% 由 skill 自动驱动；人工只做接口冻结、PR review |

---

## 九、风险登记簿

### 9.1 技术风险（继承自前期计划，更新优先级）

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| IRLM/IRAM 数值不收敛 | 中 | 高 | 逐行对照 arpack.c；oracle 逐值对比；手写 power iteration 兜底 |
| BLISS C++→Rust 翻译 bug | 中 | 高 | python-igraph canonical_permutation oracle；困难图（强正则、超立方体）专项测试 |
| C++ 算法翻译（Walktrap/Spinglass/Infomap/DrL/HRG）质量 | 高 | 中 | 论文 + 逐函数对照；oracle 兜底 |
| GLPK 依赖（optimal_modularity） | 低 | 低 | feature gated；纯 Rust LP 替代 |
| 850 API 长尾难度 | 高 | 中 | 按 Phase 设最低退出门；允许 ≥85% 解锁下 Phase |

### 9.2 进度风险（兼职模式特有）

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| 长时间断档（数周） | 高 | 中 | RESUME.md 描述如何 5 分钟恢复；ALGORITHMS.md 记录卡点 |
| 接口冻结后想改 | 中 | 中 | 0.x 版本明确允许 breaking；只在 minor bump 时改 |
| 兴趣转移导致核心算法长期 wip | 中 | 高 | "AWU 状态 wip 超 4 周自动降为 todo + 开 Issue" 规则 |
| burn-out 在长尾 Phase | 中 | 高 | Phase 退出门留余量（≥85%）；允许跳到下一 Phase |
| python-igraph oracle 失效（大版本变更） | 低 | 中 | 锁定 python-igraph 版本到 requirements.txt |

### 9.3 缓解动作清单

定期 review（每月一次，写入 .codefuse/tracking/RETRO.md）：

1. blocked 状态的 AWU 是否需要重新评估
2. perf-todo 标签的 AWU 是否累积过多
3. CI 平均运行时间是否超过 15 分钟
4. AI prompt 模板是否有可沉淀
5. 上月新增 oracle 测试覆盖率

---

## 十、退出 / 终止条件

### 10.1 各版本成熟度定义

| 版本 | API 覆盖 | Oracle 覆盖 | 性能要求 | 文档 |
|------|---------|-----------|---------|------|
| v0.1.0 | 数据结构 + BFS | ≥30% (Phase 1 项) | baseline 入库 | API rustdoc |
| v0.5.0 | 50% 公共 API | ≥70% Phase 1-5 | 同 igraph C 同数量级 | mdBook 主流程 |
| **v1.0.0** | **100% 公共 API（850 个）** | **≥95%** | **≤ python-igraph × 3** | **完整 mdBook + 迁移指南** |

### 10.2 何时该停下重构（避免无限改进陷阱）

| 信号 | 该停 / 该继续 |
|------|------|
| AWU 已 verified，但发现"更优雅"写法 | 停（创建 perf/refactor Issue） |
| oracle 数值不一致 | 继续（必修） |
| AWU 性能 > python-igraph × 3 | 停（perf-todo 标签） |
| 整个 Phase ≥ 85% 完成 | 可解锁下 Phase（剩余 ≤15% 留给后续 sweep） |
| ARCHITECTURE.md 中的决策被反复挑战 | 写一次 ADR，5 天内决策，不再挑战 |

---

## 十一、参考源码目录（`references/`，gitignored）

为了对照 igraph 三家官方实现的源码与测试，仓库根设 `references/` 目录，**整个目录加入 `.gitignore`，不参与 commit**。脚本（如 `scripts/test_extract/from_c.py`）和 AWU 实现都用相对路径 `references/<repo>/...`。

### 11.1 目录布局

```
references/                                    # gitignored
├── README.md                                  # 克隆指引（in-repo, 不忽略）
├── igraph/                                    # https://github.com/igraph/igraph (C 核心)
├── python-igraph/                             # https://github.com/igraph/python-igraph
└── rigraph/                                   # https://github.com/igraph/rigraph (R 绑定)
```

### 11.2 克隆指引（写入 `references/README.md`）

```bash
# 在仓库根执行
mkdir -p references
cd references

# === 选项 A：本地已有 igraph，使用符号链接（节省磁盘）===
# 把 <PATH-TO-LOCAL-IGRAPH> 替换为本地 igraph 仓库的绝对路径
ln -s <PATH-TO-LOCAL-IGRAPH> igraph
git -C igraph checkout v1.0.0       # 锁定到 1.0.x 稳定版
git -C igraph submodule update --init --recursive

# === 选项 B：全新克隆 ===
# git clone --depth 1 --branch v1.0.0 https://github.com/igraph/igraph.git
# git -C igraph submodule update --init --recursive

# === python-igraph ===
git clone --depth 1 https://github.com/igraph/python-igraph.git
git -C python-igraph checkout 0.11.x  # 与 oracle.py 中 pip 安装版本一致

# === R-igraph (注意 repo 名是 rigraph) ===
git clone --depth 1 https://github.com/igraph/rigraph.git

cd ..
# 验证
ls references/igraph/src/linalg/arpack.c
ls references/python-igraph/tests/
ls references/rigraph/tests/testthat/
```

### 11.3 锁定版本

为保证可复现，记录每次提交的版本到 `.codefuse/tracking/REFERENCES.md`：

| 仓库 | 锁定版本 | commit hash | 切换日期 |
|------|---------|------------|---------|
| igraph | v1.0.0 | (填) | 2026-05-15 |
| python-igraph | 0.11.x | (填) | 2026-05-15 |
| rigraph | (latest) | (填) | 2026-05-15 |

升级时同步更新 oracle.py 的 pip 依赖、conformance 提取的预期输出。

### 11.4 与 oracle.py / 提取脚本的关系

| 脚本 | 读取的 references 路径 |
|------|----------------------|
| `scripts/oracle.py` | 不直接读；通过 pip 安装的 python-igraph 跑（版本与 references 同步） |
| `scripts/test_extract/from_c.py` | `references/igraph/tests/unit/*.c` + `*.out` |
| `scripts/test_extract/from_py.py` | `references/python-igraph/tests/test_*.py` |
| `scripts/test_extract/run_r.R` | `references/rigraph/tests/testthat/test-*.R` |
| AWU Step 1 (Recon) | 按需读 `references/igraph/src/.../*.c` |

### 11.5 .gitignore 关键条目

```gitignore
# 参考源码（不入 commit）
references/igraph/
references/python-igraph/
references/rigraph/
references/.cache/

# 但保留指引
!references/README.md
```

---

## 附录 A：Phase 0 day-by-day 工时分解（兼职 4-5 周）

> 假设每周可投入 10 小时。

**Week 1（10h）**：BOOT-01～BOOT-08（仓库 + references 克隆 + 骨架 + 极简 Graph + EdgeList + BFS + Karate fixture）

**Week 2（10h）**：BOOT-33～BOOT-37（**先做 AI 基础设施**：CLAUDE.md + agents + skills + hooks + AI_PROMPTS） + BOOT-09～BOOT-11（oracle.py + 第一个 oracle 测试）

**Week 3（10h）**：BOOT-12～BOOT-18（proptest + criterion + CI 矩阵 + cargo-deny + GitHub Pages）

**Week 4（10h）**：BOOT-19～BOOT-28（templates + tracking 文档 + mdBook + RESUME 指南）

**Week 5（6h）**：BOOT-29～BOOT-32（三源 conformance 提取脚本 + BFS 三源融合测试）

**Phase 0 退出门复核**（最后 1h）：把 3.1 节清单逐项打钩。

> **顺序优化点**：先做 AI 基础设施（Week 2），让后续 BOOT 任务都能用 `/awu-start` 之类的 Skill 加速。

---

## 附录 B：ALGORITHMS.md 表头模板

```markdown
# rust-igraph 算法工作单元（AWU）跟踪表

更新规则：每个 AWU PR 合并时必须更新此表。

| ID | 名称 | C 源（文件:行） | 行数 | Cx | 前置 | 状态 | PR | Bench | Oracle |
|----|------|---------------|------|----|------|------|----|------|------|
| ALGO-CORE-001 | Graph 核心 | type_indexededgelist.c | 2013 | adapt | - | wip | #12 | - | - |
| ALGO-TR-001 | BFS | bfs.c | 300 | adapt | CORE-001 | done | #15 | 12µs/node | ✓ |
| ALGO-TR-002 | DFS | dfs.c | 200 | adapt | CORE-001 | todo | - | - | - |
| ...（约 660 行） |
```

字段含义：
- `Cx`：复杂度标签（copy / adapt / rewrite / novel）
- `状态`：todo / wip / review / done / verified / blocked / perf-todo
- `Bench`：与 python-igraph 的相对比值或绝对时间
- `Oracle`：✓（通过）/ × （失败）/ - （未集成）

---

## 附录 B2：v0.6.0 路线图（2026-06-04 制定）

> v0.5.0 完成后的下一阶段计划。聚焦三条主线并行推进。

### 主线 1：剩余核心算法（目标：~400 AWU done → ~450+）

优先级排序（按用户价值 × 实现复杂度）：

| 批次 | 算法群 | 预估 AWU | 状态 | 备注 |
|------|--------|---------|------|------|
| B1 | MST（Prim + Kruskal） | 2 | **done** (ALGO-MST-001) | 已完成 |
| B2 | Motif census（randesu） | 3-4 | **done** (MO-001..004) | Phase 7 完成 |
| B3 | Leading eigenvector community | 1 | **done** (ALGO-CO-017) | Phase 4，依赖 EIG-001 ✅ |
| B4 | Infomap community | 1 | **done** (ALGO-CO-018) | C++ → Rust 翻译 |
| B5 | Spinglass community | 1 | **done** (ALGO-CO-019) | C++ → Rust 翻译 |
| B6 | MDS layout ★ | 1 | **done** (ALGO-LY-013) | eigensolver + classical MDS |
| B7 | DrL layout | 2 | **done** (ALGO-LY-007) | C++ → Rust 翻译 |
| B8 | Davidson-Harel layout | 1 | **done** (ALGO-LY-011) | SA-based |
| B9 | GraphOpt layout | 1 | **done** (ALGO-LY-012) | 力导向 |
| B10 | UMAP layout | 2 | **done** (ALGO-LY-016) | 谱嵌入 + 近邻图 |
| B11 | isoclass 查表 | 4 | **未开始** | 2936 行 C 查表逻辑 |
| B12 | 更多图谱算法 | ~10 | **部分** | 谱嵌入/Laplacian/adjacency spectrum |
| B13 | BLISS canonical labeling | 5 | **done** (ISO-003..007) | I-R engine + 4 公开 API |

### 主线 2：文档 + 网站 + 生态

| 任务 | 优先级 | 状态 |
|------|--------|------|
| 修复 rustdoc 样式丢失 | P0 | **done** (2026-06-04) |
| Landing page 样式优化（背景/交互/暗色主题） | P1 | **done** (2026-06-04): community-colored hero, SVG icons, entrance animations, counter |
| mdBook 断链修复 + 章节重构 | P1 | **done** (2026-06-04): 外部链接改为 stub 页面 |
| README / 对比表格客观化 | P1 | **done** (2026-06-04) |
| Playground（WASM 在线交互） | P1 | **done** (2026-06-04): React SPA + WASM Worker + 20 算法可视化 + 98 测试 |
| mdBook 教程完善（更多实战章节） | P2 | 基础章节已有 |
| README 国际化（中英双语 or 中文单独） | P3 | **done** (2026-06-04): mdBook 中英双语 + 语言切换按钮 |
| crates.io 发布准备 | P2 | **done** (2026-06-04): v0.6.0 已发布 |

### 主线 3：工程质量

| 任务 | 优先级 | 状态 |
|------|--------|------|
| 全面审查网站（用户视角） | P1 | **done** (2026-06-04) |
| Conformance 覆盖率提升（当前 ~60%→80%） | P2 | **done**: 1,850 fixtures, ~98% 覆盖 |
| CI 增加 WASM 编译检查 | P2 | **done** (2026-06-04): CI + Pages 均检查 igraph-wasm |
| 性能回归监控（criterion baseline） | P3 | **done** (2026-06-04): bench.yml CI + github-action-benchmark |

### 退出门（v0.6.0 → main tag）✅ 全部达成

1. ✅ MST Prim + Kruskal 全通过
2. ✅ Landing page 样式专业化，Playground MVP 可用（React SPA + 20 算法 + 98 测试）
3. ✅ 网站/文档全面审查完成，读者体验达标
4. ✅ 308 AWU done（超过 320 目标的同等批次）
5. ✅ CI 全绿，WASM check 通过，bench regression CI 已上线
6. ✅ crates.io v0.6.0 已发布，1,297 公共 API

---

## 附录 C：废弃决策与原因

| 早期决策 | 来源计划 | 废弃原因 |
|---------|---------|---------|
| MIT/Apache-2.0 双许可 | 11:02 / 11:40 | 切换到 GPL-2.0+，允许直接参照翻译 igraph C，工作量降至 1/3 |
| nalgebra 作为主要后端 | 11:02 / 13:48 | 切换到 faer：纯 Rust、性能 2-10x、原生 EVD + sparse-linalg |
| 仅 VF2 同构（无 BLISS） | 11:02 | BLISS 必须翻译以达 100% API 兼容；VF2 仅作阶段 1 兜底 |
| 5-10 人团队 / 18 月全职 | 11:02 / 11:50 / 13:48 | 切换到 1 人 + AI 兼职，AWU 化推进，不承诺日历时间 |
| 一次性全发布 | 11:50 | 改为 v0.1 → v1.0 渐进发布 |
| ARPACK 用 nalgebra 替代（无自研） | 11:02 | 改为三层（faer 小矩阵 / 自研 IRLM / 幂迭代），与 igraph 行为精确匹配 |
| 模块直接组织在 src/ 下（单 crate） | 11:02 | 改为 workspace 3 crate，核心层与算法层解耦 |

---

## 附录 D：与早期计划的对照速查

| 主题 | 看哪份早期计划 | 本计划替换章节 |
|------|--------------|--------------|
| 完整算法清单（带 C 行数） | AI 辅助版 16:06 第二章 | 第 5.2 节 + ALGORITHMS.md |
| ARPACK / 特征值架构 | ARPACK 完整方案 15:23 | 第 2.1 节（沿用） |
| BLISS 翻译策略 | ARPACK 完整方案 15:23 第三章 | 第 2.1 节（沿用） |
| 测试三层体系 | 实施计划 11:50 第三章 | 第六章（扩为五层 + 三源融合） |
| 模块目录结构 | 项目实施计划 13:48 第五章 | 第 2.2 节 |
| 工作量估算 | AI 辅助版 16:06 / 实施计划 11:50 | **不再适用**；改用 AWU 颗粒度 |
| AI 协作 | (无) | 第八章新增（agent / skill / hook / memory / MCP） |

---

## 立即开始

1. 读完本计划。
2. 创建 `.codefuse/tracking/ALGORITHMS.md`，照附录 B 表头填入 BOOT-01 到 BOOT-37。
3. 启动 BOOT-01：`git init`、写 LICENSE、写 README 骨架。
4. **优先把 AI 基础设施 BOOT-33~37 提前到 Week 2**：CLAUDE.md + 6 agent + 9 skill 一旦就绪，从 BOOT-32 起每个任务都能用 `/awu-*` Skill 推进，速度明显加快。
5. 后续每个 AWU 严格走 9 步 SOP，状态在 ALGORITHMS.md 实时更新。

> Phase 0 完成后，再阅读 `AI 辅助开发版（16:06）` 的算法清单，按 Phase 1-10 顺序逐 AWU 推进。每个 AWU 用 `/awu-start ALGO-XXX-NNN` 触发即可，三源 conformance 自动入库。

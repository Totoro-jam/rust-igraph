# rust-igraph

**[igraph](https://igraph.org) 的纯 Rust 移植版**——网络分析库。目标是与 igraph C v1.0.x 实现完整 API 对等（约 850 个公共函数），并持续通过三个官方实现进行验证：

- **igraph C** — `tests/unit/*.c` + `*.out`
- **python-igraph** — `tests/test_*.py`
- **R-igraph** (`rigraph`) — `tests/testthat/test-*.R`

> **状态**：306 个算法工作单元已完成（Phase 1–7 & 9）。
> 687 个公共函数，7,574 个测试
>（单元 + 集成 + doctest），115 个可运行示例。支持 WASM（`wasm32-unknown-unknown`）。查看
> [总体规划](https://github.com/Totoro-jam/rust-igraph/blob/main/docs/plans/MASTER_PLAN.md)了解路线图，
> [算法追踪](https://github.com/Totoro-jam/rust-igraph/blob/main/.codefuse/tracking/ALGORITHMS.md)查看每个算法的进度。

## 已实现的功能

| 类别 | 亮点 |
|------|------|
| **遍历** | BFS、DFS、最短路径（Dijkstra、Bellman-Ford、Johnson、Floyd-Warshall、A*）、最宽路径 |
| **中心性** | PageRank、介数中心性、接近中心性、谐波中心性、特征向量中心性、HITS、约束 |
| **社区发现** | Louvain、Leiden、标签传播、流体社区、快速贪心、边介数、随机游走、主特征向量、Voronoi |
| **连通性** | 连通分量、割点、桥、双连通分量、聚合块 |
| **网络流** | 最大流 (Dinic)、最小割、Gomory-Hu 树、顶点/边连通度、所有 s-t 割 |
| **同构** | VF2（图 + 子图）、BLISS 正则标记、LAD 子图同构 |
| **着色** | 贪心顶点着色 (CN + DSatur)、弦性、二部图匹配 |
| **构造器** | 40+ 图生成器（Erdős-Rényi、Barabási-Albert、Watts-Strogatz、SBM、格子图、经典图…）|
| **I/O** | 8 种格式：GML、GraphML、DOT、Pajek、NCOL、LGL、DL、LEDA——全部支持属性 |
| **运算符** | 并、交、差、补图、简化、置换、线图 |

## 为什么还需要一个 Rust 图库？

`petgraph` 是优秀的通用图库，但它的 API 无法表达 `igraph_t` 所表达的内容（丰富的属性、顶点/边选择器、完整的 C API 对等）。从 igraph 的 C、Python 或 R 绑定迁移过来的用户需要一个 Rust 之家，在这里：

- 函数名镜像 `igraph_*`
- 数值结果与 python-igraph 在严格容差内一致
- 构建对 WASM 天然友好（无 `unsafe`，无系统依赖）

## 许可证

GPL-2.0-or-later，与上游 igraph 保持一致。[架构决策记录](https://github.com/Totoro-jam/rust-igraph/blob/main/.codefuse/tracking/ARCHITECTURE.md)解释了原因。

## 本站是如何构建的

`mdBook` 构建文档部分；`cargo doc` 构建 API rustdoc；CI 将两者发布到 GitHub Pages。本地构建：

```bash
cargo install mdbook
mdbook serve --open
```

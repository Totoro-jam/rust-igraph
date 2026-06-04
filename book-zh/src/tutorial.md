# 教程

本章通过构建一个小型网络分析流水线来逐步讲解 `rust-igraph` 的核心功能。每个代码块都是自包含的片段，可以直接粘贴到 Rust 文件中使用，配合 `use rust_igraph::prelude::*;`。

## 创建图

最简单的方式是从边列表创建图：

```rust
use rust_igraph::Graph;

// 无向三角形
let g = Graph::from_edges(&[(0, 1), (1, 2), (2, 0)], false, None).unwrap();
assert_eq!(g.vcount(), 3);
assert_eq!(g.ecount(), 3);
```

使用 `GraphBuilder` 可以获得更多控制：

```rust
use rust_igraph::GraphBuilder;

let g = GraphBuilder::undirected()
    .vertices(5)
    .edges(&[(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)])
    .build()
    .unwrap();
```

还有 40+ 个命名构造器用于常见图族：

```rust
use rust_igraph::{full_graph, ring_graph, star_graph, StarMode, erdos_renyi_gnp};

let complete = full_graph(5, false, false).unwrap();                    // K_5
let cycle = ring_graph(10, false, false, false).unwrap();              // C_10
let hub = star_graph(8, StarMode::Undirected, 0).unwrap();             // 以 0 为中心的星图
let random = erdos_renyi_gnp(100, 0.05, false, false, 42).unwrap();   // G(100, 0.05)
```

## 基本属性

```rust
use rust_igraph::{Graph, density, is_connected, ConnectednessMode, diameter};

let g = Graph::from_edges(
    &[(0,1),(1,2),(2,3),(3,0),(2,4),(4,5)], false, None
).unwrap();

println!("顶点数: {}", g.vcount());            // 6
println!("边数: {}", g.ecount());               // 6
println!("有向: {}", g.is_directed());          // false
println!("密度: {:.4}", density(&g).unwrap().unwrap_or(0.0));
println!("连通: {}", is_connected(&g, ConnectednessMode::Weak).unwrap());
println!("直径: {:?}", diameter(&g).unwrap());
```

## 中心性指标

```rust
use rust_igraph::{Graph, pagerank, betweenness, closeness};

let g = Graph::from_edges(
    &[(0,1),(0,2),(1,2),(1,3),(2,3),(3,4),(4,5),(5,6),(6,4)],
    false, None
).unwrap();

let pr = pagerank(&g).unwrap();
let bc = betweenness(&g).unwrap();
let cl = closeness(&g).unwrap();

// 找到最重要的顶点
let (top_v, top_score) = pr.iter()
    .enumerate()
    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
    .unwrap();
println!("最高 PageRank: 顶点 {} ({:.4})", top_v, top_score);
```

## 社区发现

```rust
use rust_igraph::{Graph, louvain, leiden};

let g = Graph::from_edges(
    &[(0,1),(0,2),(1,2),(3,4),(3,5),(4,5),(2,3)],
    false, None
).unwrap();

let result = louvain(&g).unwrap();
println!("社区: {:?}", result.membership);
println!("模块度: {:.4}", result.modularity);
```

可用的社区发现算法：Louvain、Leiden、标签传播、流体社区、快速贪心、边介数、随机游走和主特征向量。

## 最短路径

```rust
use rust_igraph::{Graph, distances, dijkstra_distances};

let g = Graph::from_edges(
    &[(0,1),(1,2),(2,3),(0,3),(1,3)], false, None
).unwrap();

// 从顶点 0 出发的无权距离
let dist = distances(&g, 0).unwrap();
println!("从 0 出发的距离: {:?}", dist);  // [Some(0), Some(1), Some(2), Some(1)]

// 带权最短路径
let weights = vec![1.0, 2.0, 1.0, 5.0, 1.0];
let wdist = dijkstra_distances(&g, 0, &weights).unwrap();
println!("带权距离从 0: {:?}", wdist);
```

## 图属性

顶点、边和图本身都可以携带类型化的属性：

```rust
use rust_igraph::{Graph, AttributeValue};

let mut g = Graph::from_edges(&[(0,1),(1,2)], false, None).unwrap();

// 顶点属性
g.set_vertex_attribute("name", 0, AttributeValue::String("Alice".into())).unwrap();
g.set_vertex_attribute("name", 1, AttributeValue::String("Bob".into())).unwrap();
g.set_vertex_attribute("name", 2, AttributeValue::String("Carol".into())).unwrap();

// 边属性
g.set_edge_attribute("weight", 0, AttributeValue::Numeric(1.5)).unwrap();
g.set_edge_attribute("weight", 1, AttributeValue::Numeric(2.3)).unwrap();

// 图级属性
g.set_graph_attribute("title", AttributeValue::String("My Network".into()));

// 读取属性
if let Some(name) = g.vertex_attribute("name", 0) {
    println!("顶点 0: {}", name);  // "Alice"
}
```

## 文件 I/O

最简单的方式是使用 `from_file` / `to_file`，它们会根据扩展名自动检测格式：

```rust,no_run
use rust_igraph::Graph;

// 从扩展名自动检测 GML 格式
let g = Graph::from_file("network.gml").unwrap();
println!("{g}");

// 写为 GraphML — 扩展名决定格式
g.to_file("network.graphml").unwrap();
```

支持的扩展名：`.gml`、`.graphml`/`.xml`、`.dot`/`.gv`、`.net`/`.pajek`、`.ncol`、`.lgl`、`.leda`/`.lgr`、`.dl`、`.edges`/`.edgelist`/`.txt`/`.csv`。

如需更精细的控制（例如写入缓冲区），可使用基于流的函数：

```rust
use rust_igraph::{Graph, AttributeValue, write_gml, read_gml};

let mut g = Graph::from_edges(&[(0,1),(1,2),(2,0)], false, None).unwrap();
g.set_graph_attribute("name", AttributeValue::String("triangle".into()));

// 写入内存缓冲区
let mut buf = Vec::new();
write_gml(&g, &mut buf).unwrap();

// 读回 — 属性在往返中保持完整
let g2 = read_gml(buf.as_slice()).unwrap();
assert_eq!(g2.vcount(), 3);
assert_eq!(
    g2.graph_attribute("name").and_then(AttributeValue::as_str),
    Some("triangle")
);
```

支持的格式：GML、GraphML、DOT (Graphviz)、Pajek、NCOL、LGL、DL (UCINET) 和 LEDA。

## 图同构

```rust
use rust_igraph::{full_graph, ring_graph, isomorphic};

let g1 = full_graph(4, false, false).unwrap();
let g2 = full_graph(4, false, false).unwrap();
let g3 = ring_graph(4, false, false, false).unwrap();

assert!(isomorphic(&g1, &g2).unwrap());   // K_4 ≅ K_4
assert!(!isomorphic(&g1, &g3).unwrap());  // K_4 ≇ C_4
```

### BLISS 规范标注

BLISS 引擎提供规范形式、自同构计数和顶点颜色感知的同构检测：

```rust
use rust_igraph::{Graph, canonical_permutation, count_automorphisms,
                  isomorphic_bliss, permute_vertices};

let g1 = Graph::from_edges(&[(0,1),(1,2),(2,3),(3,0)], false, None).unwrap();
let g2 = Graph::from_edges(&[(2,0),(0,3),(3,1),(1,2)], false, None).unwrap();

// 规范置换 — 同构图重标后结果相同
let p1 = canonical_permutation(&g1, None).unwrap();
let p2 = canonical_permutation(&g2, None).unwrap();
let c1 = permute_vertices(&g1, &p1).unwrap();
let c2 = permute_vertices(&g2, &p2).unwrap();
assert_eq!(c1.ecount(), c2.ecount());

// 自同构群阶: |Aut(C_4)| = 8
let aut = count_automorphisms(&g1, None).unwrap();
assert!((aut - 8.0).abs() < 1e-9);

// 基于 BLISS 的同构检测（含映射）
let result = isomorphic_bliss(&g1, &g2, None, None).unwrap();
assert!(result.iso);
```

## 图运算符

```rust
use rust_igraph::Graph;

let a = Graph::from_edges(&[(0,1),(1,2)], false, None).unwrap();
let b = Graph::from_edges(&[(1,2),(2,3)], false, None).unwrap();

let u = &a | &b;  // 并
let i = &a & &b;  // 交

println!("并: {} 个顶点, {} 条边", u.vcount(), u.ecount());
println!("交: {} 个顶点, {} 条边", i.vcount(), i.ecount());
```

## 图布局

rust-igraph 包含 16 个布局引擎用于 2D 和 3D 图可视化：

```rust
use rust_igraph::{Graph, FrParams, KkParams, layout_fruchterman_reingold, layout_circle, layout_kamada_kawai};

let g = Graph::from_edges(
    &[(0,1),(1,2),(2,3),(3,0),(0,2),(1,3)], false, None
).unwrap();

// 力导向布局 (Fruchterman-Reingold)
let coords = layout_fruchterman_reingold(&g, &FrParams::default()).unwrap();
for (i, &(x, y)) in coords.iter().enumerate() {
    println!("v{i}: ({x:.2}, {y:.2})");
}

// 圆形布局（确定性）
let circle = layout_circle(&g, None);
assert_eq!(circle.len(), g.vcount() as usize);

// Kamada-Kawai（基于能量）
let n = g.vcount() as usize;
let kk = layout_kamada_kawai(&g, None, &KkParams::default_for(n), None).unwrap();
assert_eq!(kk.len(), n);
```

可用的引擎：Fruchterman-Reingold、Kamada-Kawai、DrL、Sugiyama、GEM、Davidson-Harel、GraphOpt、MDS、LGL、UMAP、Reingold-Tilford、圆形、星形、网格、随机和球面。

## 遍历图

```rust
use rust_igraph::Graph;

let g = Graph::from_edges(&[(0,1),(1,2),(2,0)], false, None).unwrap();

// 遍历边
for (src, tgt) in &g {
    println!("{} -- {}", src, tgt);
}

// 遍历顶点 ID
for v in g.vertex_ids() {
    let deg = g.degree(v).unwrap();
    println!("顶点 {}: 度 {}", v, deg);
}
```

## 方法 API vs 自由函数

大多数算法同时提供自由函数和 `Graph` 方法两种形式：

```rust
use rust_igraph::{Graph, pagerank};

let g = Graph::from_edges(&[(0,1),(1,2),(2,0)], false, None).unwrap();

// 自由函数风格
let pr1 = pagerank(&g).unwrap();

// 方法风格
let pr2 = g.pagerank().unwrap();

// 结果相同
assert_eq!(pr1, pr2);
```

## 下一步

- 浏览 [API 文档](./api.md) 查看完整的函数列表
- 运行 `cargo run --example social_network_demo` 查看综合演示
- 查看 `examples/` 目录中的 115 个示例

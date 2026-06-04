# 实战手册

常见图分析任务的实用模式。每个示例都是自包含的——粘贴到文件中即可运行
`cargo run --example <name>`，或直接在项目中使用代码片段。

## 找出最重要的节点

组合多种中心性指标来识别关键顶点：

```rust
use rust_igraph::{Graph, pagerank, betweenness, harmonic_centrality};

let g = Graph::from_edges(
    &[(0,1),(0,2),(1,2),(1,3),(2,3),(3,4),(4,5),(5,6),(6,4),(6,7),(7,8),(8,9),(9,7)],
    false, None
).unwrap();

let pr = pagerank(&g).unwrap();
let bc = betweenness(&g).unwrap();
let hc = harmonic_centrality(&g).unwrap();

// 按 PageRank 排名
let mut ranked: Vec<(usize, f64)> = pr.iter().copied().enumerate().collect();
ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

println!("PageRank 前三顶点:");
for &(v, score) in ranked.iter().take(3) {
    println!("  v{v}: PR={score:.4}  介数={:.2}  调和={:.4}",
        bc[v], hc[v]);
}
```

## 对比社区检测算法

运行多种算法并衡量一致性：

```rust
use rust_igraph::{Graph, louvain, leiden, label_propagation,
                  compare_communities, CommunityComparison};

let g = Graph::from_edges(
    &[(0,1),(0,2),(1,2),(3,4),(3,5),(4,5),(2,3),(6,7),(6,8),(7,8),(5,6)],
    false, None
).unwrap();

let c_louvain = louvain(&g).unwrap();
let c_leiden = leiden(&g).unwrap();
let c_lpa = label_propagation(&g).unwrap();

println!("Louvain:  {} 个社区, 模块度 {:.4}",
    *c_louvain.membership.iter().max().unwrap() + 1, c_louvain.modularity);
println!("Leiden:   {} 个社区, 模块度 {:.4}",
    *c_leiden.membership.iter().max().unwrap() + 1, c_leiden.modularity);

// 使用标准化互信息比较划分
let nmi = compare_communities(
    &c_louvain.membership,
    &c_leiden.membership,
    CommunityComparison::NormalizedMutualInformation,
).unwrap();
println!("NMI(Louvain, Leiden) = {nmi:.4}");

let ari = compare_communities(
    &c_louvain.membership,
    &c_lpa.membership,
    CommunityComparison::AdjustedRand,
).unwrap();
println!("ARI(Louvain, LPA) = {ari:.4}");
```

## 构建加权图并求最短路径

```rust
use rust_igraph::{Graph, dijkstra_distances, dijkstra_paths};

let g = Graph::from_edges(
    &[(0,1),(0,2),(1,3),(2,3),(3,4),(2,4)],
    false, None
).unwrap();

let weights = vec![1.0, 4.0, 2.0, 1.0, 3.0, 2.0];

// 从源点 0 出发的距离向量
let dist = dijkstra_distances(&g, 0, &weights).unwrap();
for (v, d) in dist.iter().enumerate() {
    match d {
        Some(d) => println!("  0 -> {v}: 距离 {d:.1}"),
        None => println!("  0 -> {v}: 不可达"),
    }
}

// 完整路径重建
let paths = dijkstra_paths(&g, 0, &weights).unwrap();
if let Some(parent) = paths.parents[4] {
    println!("顶点 4 通过顶点 {parent} 到达");
}
```

## 分析图连通性

```rust
use rust_igraph::{Graph, connected_components, articulation_points, bridges};

let g = Graph::from_edges(
    &[(0,1),(1,2),(2,0),(2,3),(3,4),(4,5),(5,3)],
    false, None
).unwrap();

let cc = connected_components(&g).unwrap();
println!("连通分量: {}", cc.count);

let ap = articulation_points(&g).unwrap();
println!("割点: {:?}", ap);

let br = bridges(&g).unwrap();
println!("桥: {:?}", br);
```

## 生成和分析随机图

```rust
use rust_igraph::{Graph, density, connected_components,
                  erdos_renyi_gnp, barabasi_game_bag, watts_strogatz_game};

// Erdos-Renyi: G(n, p)
let er = erdos_renyi_gnp(100, 0.05, false, false, 42).unwrap();
let er_cc = connected_components(&er).unwrap();
println!("ER(100, 0.05): {} 条边, {} 个连通分量",
    er.ecount(), er_cc.count);

// Barabasi-Albert: 优先连接模型
let ba = barabasi_game_bag(100, 3, false, false, 42).unwrap();
println!("BA(100, m=3): {} 条边, 密度 {:.4}",
    ba.ecount(), density(&ba).unwrap().unwrap_or(0.0));

// Watts-Strogatz: 小世界网络
let ws = watts_strogatz_game(100, 4, 0.1, false, false, 42).unwrap();
println!("WS(100, k=4, p=0.1): {} 条边", ws.ecount());
```

也可以使用 `Graph` 上的便捷方法：

```rust
use rust_igraph::Graph;

let er = Graph::erdos_renyi(100, 0.05, 42).unwrap();
let ba = Graph::barabasi_albert(100, 3, 42).unwrap();
let ws = Graph::watts_strogatz(100, 4, 0.1, 42).unwrap();
```

## 通过 GML 格式保存和读取带属性的图

```rust
use rust_igraph::{Graph, AttributeValue, write_gml, read_gml};

let mut g = Graph::from_edges(&[(0,1),(1,2),(2,0)], false, None).unwrap();

// 添加顶点名称
for (i, name) in ["Alice", "Bob", "Carol"].iter().enumerate() {
    g.set_vertex_attribute("name", i as u32,
        AttributeValue::String((*name).into())).unwrap();
}

// 添加边权重
g.set_edge_attribute("weight", 0, AttributeValue::Numeric(1.5)).unwrap();
g.set_edge_attribute("weight", 1, AttributeValue::Numeric(2.0)).unwrap();
g.set_edge_attribute("weight", 2, AttributeValue::Numeric(0.8)).unwrap();

// 序列化和反序列化
let mut buf = Vec::new();
write_gml(&g, &mut buf).unwrap();

let g2 = read_gml(buf.as_slice()).unwrap();
assert_eq!(g2.vcount(), 3);
assert_eq!(
    g2.vertex_attribute("name", 0).and_then(AttributeValue::as_str),
    Some("Alice")
);
```

## 使用 BLISS 检测图同构

```rust
use rust_igraph::{Graph, isomorphic_bliss, canonical_permutation,
                  count_automorphisms, permute_vertices};

let g1 = Graph::from_edges(&[(0,1),(1,2),(2,3),(3,0)], false, None).unwrap();
// 相同的图，顶点重新标注
let g2 = Graph::from_edges(&[(2,0),(0,3),(3,1),(1,2)], false, None).unwrap();

let result = isomorphic_bliss(&g1, &g2, None, None).unwrap();
assert!(result.iso);
println!("同构: {}", result.iso);
if !result.map12.is_empty() {
    println!("映射 g1->g2: {:?}", result.map12);
}

// 规范形式：同构的图获得相同的规范标注
let perm1 = canonical_permutation(&g1, None).unwrap();
let perm2 = canonical_permutation(&g2, None).unwrap();
let c1 = permute_vertices(&g1, &perm1).unwrap();
let c2 = permute_vertices(&g2, &perm2).unwrap();
// c1 和 c2 有相同的边集

// 计算自同构数
let aut = count_automorphisms(&g1, None).unwrap();
println!("|Aut(C_4)| = {aut}");  // 8
```

## 计算最小生成树

```rust
use rust_igraph::{Graph, minimum_spanning_tree, MstAlgorithm};

let g = Graph::from_edges(
    &[(0,1),(0,2),(1,2),(1,3),(2,3),(3,4)],
    false, None
).unwrap();
let weights = vec![4.0, 2.0, 1.0, 3.0, 5.0, 1.0];

let mst_edges = minimum_spanning_tree(&g, Some(&weights), MstAlgorithm::Automatic).unwrap();
println!("MST 边: {:?}", mst_edges);

let total_weight: f64 = mst_edges.iter()
    .map(|&e| weights[e as usize])
    .sum();
println!("MST 总权重: {total_weight}");
```

## 网络流

```rust
use rust_igraph::{Graph, max_flow_value};

// 简单的流网络（有向图）
let g = Graph::from_edges(
    &[(0,1),(0,2),(1,3),(2,3),(1,2)],
    true, // 有向
    None,
).unwrap();

let capacity = vec![3.0, 2.0, 2.0, 3.0, 1.0];

let flow = max_flow_value(&g, 0, 3, Some(&capacity)).unwrap();
println!("0 到 3 的最大流: {flow}");
```

## 使用 Infomap 和 Spinglass 进行社区检测

```rust
use rust_igraph::{Graph, infomap, spinglass,
                  compare_communities, CommunityComparison};

let g = Graph::from_edges(
    &[(0,1),(0,2),(1,2),(3,4),(3,5),(4,5),(2,3),(6,7),(6,8),(7,8),(5,6)],
    false, None
).unwrap();

// Infomap — 信息论方法（地图方程）
let im = infomap(&g).unwrap();
println!("Infomap: {} 个模块, 编码长度 {:.4}",
    *im.membership.iter().max().unwrap() + 1, im.codelength);

// Spinglass — Potts 模型模拟退火
let sp = spinglass(&g, None).unwrap();
println!("Spinglass: {} 个社区, 模块度 {:.4}",
    *sp.membership.iter().max().unwrap() + 1, sp.modularity);

// 比较两种划分
let nmi = compare_communities(
    &im.membership, &sp.membership,
    CommunityComparison::NormalizedMutualInformation,
).unwrap();
println!("NMI(Infomap, Spinglass) = {nmi:.4}");
```

## 分析三元组结构（motif 普查）

```rust
use rust_igraph::{Graph, triad_census, dyad_census};

// 用于三元组分析的有向图
let g = Graph::from_edges(
    &[(0,1),(1,2),(2,0),(0,3),(3,4),(4,0),(1,4)],
    true, None
).unwrap();

let tc = triad_census(&g).unwrap();
println!("三元组普查（16 种类型）:");
for (i, &count) in tc.counts.iter().enumerate() {
    if count > 0 {
        println!("  类型 {:02}: {count}", i);
    }
}

let dc = dyad_census(&g).unwrap();
println!("二元组普查: 互惠={}, 非对称={}, 空={}",
    dc.mutual, dc.asymmetric, dc.null_count);
```

## 构建空间邻近图

```rust
use rust_igraph::{delaunay_graph, gabriel_graph};

// 二维点云
let points = vec![
    vec![0.0, 0.0],
    vec![1.0, 0.0],
    vec![0.5, 0.866],
    vec![2.0, 0.5],
    vec![1.5, 1.5],
];

// Delaunay 三角剖分——连接最近的点三元组
let dt = delaunay_graph(&points).unwrap();
println!("Delaunay: {} 个顶点, {} 条边", dt.vcount(), dt.ecount());

// Gabriel 图——Delaunay 的子集，任何边的直径圆内
// 不含第三个点
let gg = gabriel_graph(&points).unwrap();
println!("Gabriel:  {} 个顶点, {} 条边", gg.vcount(), gg.ecount());
```

## 布局导出用于可视化

```rust
use rust_igraph::{Graph, FrParams, layout_fruchterman_reingold, layout_circle};

let g = Graph::from_edges(
    &[(0,1),(1,2),(2,3),(3,4),(4,0),(0,2),(1,3)],
    false, None
).unwrap();

// 力导向布局
let coords = layout_fruchterman_reingold(&g, &FrParams::default()).unwrap();

// 输出为 CSV 供外部工具使用
println!("vertex,x,y");
for (v, &(x, y)) in coords.iter().enumerate() {
    println!("{v},{x:.6},{y:.6}");
}

// 圆形布局（确定性，适合小图）
let circle = layout_circle(&g, None);
for (v, &(x, y)) in circle.iter().enumerate() {
    println!("v{v}: ({x:.4}, {y:.4})");
}
```

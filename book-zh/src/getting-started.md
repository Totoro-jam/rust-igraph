# 快速开始

## 安装

在 `Cargo.toml` 中添加 rust-igraph：

```toml
[dependencies]
rust-igraph = "0.5"
```

## 你的第一个图

```rust
use rust_igraph::{Graph, bfs, pagerank, louvain};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 构建一个小型社交网络
    let g = Graph::from_edges(
        &[(0,1), (0,2), (1,2), (1,3), (2,3), (3,4), (4,5), (5,6), (6,4)],
        false, // 无向图
        None,  // 自动推断顶点数
    )?;

    println!("{g}");
    // => Undirected graph with 7 vertices and 9 edges

    // 从顶点 0 开始 BFS 遍历
    let order = bfs(&g, 0)?;
    println!("BFS 顺序: {order:?}");

    // PageRank 中心性
    let pr = pagerank(&g)?;
    let (top_v, top_score) = pr.iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .unwrap();
    println!("最重要的顶点: {top_v} (PageRank = {top_score:.4})");

    // 社区发现
    let communities = louvain(&g)?;
    println!("社区: {:?}", communities.membership);
    println!("模块度: {:.4}", communities.modularity);

    Ok(())
}
```

## 方法风格 API

同样的操作也可以作为 `Graph` 的方法调用：

```rust
use rust_igraph::Graph;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let g = Graph::from_edges(
        &[(0,1), (0,2), (1,2), (1,3), (2,3), (3,4), (4,5), (5,6), (6,4)],
        false, None,
    )?;

    // 这些都可以直接在图上调用
    let pr = g.pagerank()?;
    let bc = g.betweenness()?;
    let communities = g.louvain()?;
    let connected = g.is_connected()?;
    let diameter = g.diameter()?;

    println!("连通: {connected}");
    println!("直径: {diameter:?}");
    println!("模块度: {:.4}", communities.modularity);

    Ok(())
}
```

## 图构建选项

```rust
use rust_igraph::{Graph, GraphBuilder};

// 从边列表构建
let g = Graph::from_edges(&[(0,1), (1,2), (2,0)], false, None).unwrap();

// 流式构建器模式
let g = GraphBuilder::undirected()
    .vertices(5)
    .edges(&[(0,1), (1,2), (2,3), (3,4), (4,0)])
    .build()
    .unwrap();

// 经典图生成器
let er = Graph::erdos_renyi(1000, 0.01, 42).unwrap();    // 随机图
let ba = Graph::barabasi_albert(1000, 3, 42).unwrap();    // 无标度网络
let ws = Graph::watts_strogatz(1000, 6, 0.1, 42).unwrap(); // 小世界网络
```

## 运行示例

仓库包含 115 个可运行示例：

```bash
# 克隆并运行
git clone https://github.com/Totoro-jam/rust-igraph
cd rust-igraph

cargo run --example quickstart
cargo run --example social_network_demo
cargo run --example community_detection_demo
cargo run --example method_api_demo
cargo run --example layout_demo
cargo run --example file_io_demo
```

## 下一步阅读

- [教程](./tutorial.md) — 通过可运行的代码片段逐步讲解所有核心功能。
- [API 文档](https://docs.rs/rust-igraph) — 每个函数、结构体和枚举的完整 rustdoc 参考。
- [示例目录](https://github.com/Totoro-jam/rust-igraph/tree/main/examples) — 115 个涵盖每个算法类别的独立程序。

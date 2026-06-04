# Quick start

## Installation

Add rust-igraph to your `Cargo.toml`:

```toml
[dependencies]
rust-igraph = "0.5"
```

## Your first graph

```rust
use rust_igraph::{Graph, bfs, pagerank, louvain};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build a small social network
    let g = Graph::from_edges(
        &[(0,1), (0,2), (1,2), (1,3), (2,3), (3,4), (4,5), (5,6), (6,4)],
        false, // undirected
        None,  // auto-infer vertex count
    )?;

    println!("{g}");
    // => Undirected graph with 7 vertices and 9 edges

    // BFS traversal from vertex 0
    let order = bfs(&g, 0)?;
    println!("BFS order: {order:?}");

    // PageRank centrality
    let pr = pagerank(&g)?;
    let (top_v, top_score) = pr.iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .unwrap();
    println!("Most central vertex: {top_v} (PageRank = {top_score:.4})");

    // Community detection
    let communities = louvain(&g)?;
    println!("Communities: {:?}", communities.membership);
    println!("Modularity: {:.4}", communities.modularity);

    Ok(())
}
```

## Method-style API

The same operations are available as methods on `Graph`:

```rust
use rust_igraph::Graph;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let g = Graph::from_edges(
        &[(0,1), (0,2), (1,2), (1,3), (2,3), (3,4), (4,5), (5,6), (6,4)],
        false, None,
    )?;

    // All of these work directly on the graph
    let pr = g.pagerank()?;
    let bc = g.betweenness()?;
    let communities = g.louvain()?;
    let connected = g.is_connected()?;
    let diameter = g.diameter()?;

    println!("Connected: {connected}");
    println!("Diameter: {diameter:?}");
    println!("Modularity: {:.4}", communities.modularity);

    Ok(())
}
```

## Graph construction options

```rust
use rust_igraph::{Graph, GraphBuilder};

// From a flat edge list
let g = Graph::from_edges(&[(0,1), (1,2), (2,0)], false, None).unwrap();

// Fluent builder pattern
let g = GraphBuilder::undirected()
    .vertices(5)
    .edges(&[(0,1), (1,2), (2,3), (3,4), (4,0)])
    .build()
    .unwrap();

// Classic generators
let er = Graph::erdos_renyi(1000, 0.01, 42).unwrap();    // random
let ba = Graph::barabasi_albert(1000, 3, 42).unwrap();    // scale-free
let ws = Graph::watts_strogatz(1000, 6, 0.1, 42).unwrap(); // small-world
```

## Running the examples

The repository includes 115 runnable examples:

```bash
# Clone and run
git clone https://github.com/Totoro-jam/rust-igraph
cd rust-igraph

cargo run --example quickstart
cargo run --example social_network_demo
cargo run --example community_detection_demo
cargo run --example method_api_demo
cargo run --example layout_demo
cargo run --example file_io_demo
```

## Where to read next

- [Tutorial](./tutorial.md) — walks through all major features with
  runnable code snippets.
- [API documentation](https://docs.rs/rust-igraph) — full rustdoc
  reference for every function, struct, and enum.
- [Examples directory](https://github.com/Totoro-jam/rust-igraph/tree/main/examples)
  — 115 self-contained programs covering every algorithm category.

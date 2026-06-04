# Tutorial

This chapter walks through the core features of `rust-igraph` by building
a small network analysis pipeline. Every code block is a self-contained
snippet you can paste into a Rust file with `use rust_igraph::prelude::*;`.

## Creating graphs

The simplest way to create a graph is from an edge list:

```rust
use rust_igraph::Graph;

// Undirected triangle
let g = Graph::from_edges(&[(0, 1), (1, 2), (2, 0)], false, None).unwrap();
assert_eq!(g.vcount(), 3);
assert_eq!(g.ecount(), 3);
```

For more control, use the `GraphBuilder`:

```rust
use rust_igraph::GraphBuilder;

let g = GraphBuilder::undirected()
    .vertices(5)
    .edges(&[(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)])
    .build()
    .unwrap();
```

There are also 40+ named constructors for common graph families:

```rust
use rust_igraph::{full_graph, ring_graph, star_graph, StarMode, erdos_renyi_gnp};

let complete = full_graph(5, false, false).unwrap();                    // K_5
let cycle = ring_graph(10, false, false, false).unwrap();              // C_10
let hub = star_graph(8, StarMode::Undirected, 0).unwrap();             // star with center 0
let random = erdos_renyi_gnp(100, 0.05, false, false, 42).unwrap();   // G(100, 0.05)
```

## Basic properties

```rust
use rust_igraph::{Graph, density, is_connected, ConnectednessMode, diameter};

let g = Graph::from_edges(
    &[(0,1),(1,2),(2,3),(3,0),(2,4),(4,5)], false, None
).unwrap();

println!("Vertices: {}", g.vcount());            // 6
println!("Edges: {}", g.ecount());               // 6
println!("Directed: {}", g.is_directed());        // false
println!("Density: {:.4}", density(&g).unwrap().unwrap_or(0.0));
println!("Connected: {}", is_connected(&g, ConnectednessMode::Weak).unwrap());
println!("Diameter: {:?}", diameter(&g).unwrap());
```

## Centrality measures

```rust
use rust_igraph::{Graph, pagerank, betweenness, closeness};

let g = Graph::from_edges(
    &[(0,1),(0,2),(1,2),(1,3),(2,3),(3,4),(4,5),(5,6),(6,4)],
    false, None
).unwrap();

let pr = pagerank(&g).unwrap();
let bc = betweenness(&g).unwrap();
let cl = closeness(&g).unwrap();

// Find the most central vertex
let (top_v, top_score) = pr.iter()
    .enumerate()
    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
    .unwrap();
println!("Highest PageRank: vertex {} ({:.4})", top_v, top_score);
```

## Community detection

```rust
use rust_igraph::{Graph, louvain, leiden};

let g = Graph::from_edges(
    &[(0,1),(0,2),(1,2),(3,4),(3,5),(4,5),(2,3)],
    false, None
).unwrap();

let result = louvain(&g).unwrap();
println!("Communities: {:?}", result.membership);
println!("Modularity: {:.4}", result.modularity);
```

Available community detection algorithms: Louvain, Leiden, label
propagation, fluid communities, fast greedy, edge betweenness, walktrap,
and leading eigenvector.

## Shortest paths

```rust
use rust_igraph::{Graph, distances, dijkstra_distances};

let g = Graph::from_edges(
    &[(0,1),(1,2),(2,3),(0,3),(1,3)], false, None
).unwrap();

// Unweighted distances from vertex 0
let dist = distances(&g, 0).unwrap();
println!("Distances from 0: {:?}", dist);  // [Some(0), Some(1), Some(2), Some(1)]

// Weighted shortest paths
let weights = vec![1.0, 2.0, 1.0, 5.0, 1.0];
let wdist = dijkstra_distances(&g, 0, &weights).unwrap();
println!("Weighted distances from 0: {:?}", wdist);
```

## Graph attributes

Vertices, edges, and the graph itself can carry typed attributes:

```rust
use rust_igraph::{Graph, AttributeValue};

let mut g = Graph::from_edges(&[(0,1),(1,2)], false, None).unwrap();

// Vertex attributes
g.set_vertex_attribute("name", 0, AttributeValue::String("Alice".into())).unwrap();
g.set_vertex_attribute("name", 1, AttributeValue::String("Bob".into())).unwrap();
g.set_vertex_attribute("name", 2, AttributeValue::String("Carol".into())).unwrap();

// Edge attributes
g.set_edge_attribute("weight", 0, AttributeValue::Numeric(1.5)).unwrap();
g.set_edge_attribute("weight", 1, AttributeValue::Numeric(2.3)).unwrap();

// Graph-level attributes
g.set_graph_attribute("title", AttributeValue::String("My Network".into()));

// Read back
if let Some(name) = g.vertex_attribute("name", 0) {
    println!("Vertex 0: {}", name);  // "Alice"
}
```

## File I/O

The easiest way to read and write graphs is with `from_file` / `to_file`,
which auto-detect the format from the file extension:

```rust,no_run
use rust_igraph::Graph;

// Auto-detects GML from the extension
let g = Graph::from_file("network.gml").unwrap();
println!("{g}");

// Write as GraphML — extension determines format
g.to_file("network.graphml").unwrap();
```

Supported extensions: `.gml`, `.graphml`/`.xml`, `.dot`/`.gv`, `.net`/`.pajek`,
`.ncol`, `.lgl`, `.leda`/`.lgr`, `.dl`, `.edges`/`.edgelist`/`.txt`/`.csv`.

For more control (e.g. writing to a buffer), use the stream-based functions:

```rust
use rust_igraph::{Graph, AttributeValue, write_gml, read_gml};

let mut g = Graph::from_edges(&[(0,1),(1,2),(2,0)], false, None).unwrap();
g.set_graph_attribute("name", AttributeValue::String("triangle".into()));

// Write to an in-memory buffer
let mut buf = Vec::new();
write_gml(&g, &mut buf).unwrap();

// Read back — attributes survive the round-trip
let g2 = read_gml(buf.as_slice()).unwrap();
assert_eq!(g2.vcount(), 3);
assert_eq!(
    g2.graph_attribute("name").and_then(AttributeValue::as_str),
    Some("triangle")
);
```

Supported formats: GML, GraphML, DOT (Graphviz), Pajek, NCOL, LGL,
DL (UCINET), and LEDA.

## Graph isomorphism

```rust
use rust_igraph::{full_graph, ring_graph, isomorphic};

let g1 = full_graph(4, false, false).unwrap();
let g2 = full_graph(4, false, false).unwrap();
let g3 = ring_graph(4, false, false, false).unwrap();

assert!(isomorphic(&g1, &g2).unwrap());   // K_4 ≅ K_4
assert!(!isomorphic(&g1, &g3).unwrap());  // K_4 ≇ C_4
```

### BLISS canonical labeling

The BLISS engine provides canonical forms, automorphism counting, and
vertex-color-aware isomorphism:

```rust
use rust_igraph::{Graph, canonical_permutation, count_automorphisms,
                  isomorphic_bliss, permute_vertices};

let g1 = Graph::from_edges(&[(0,1),(1,2),(2,3),(3,0)], false, None).unwrap();
let g2 = Graph::from_edges(&[(2,0),(0,3),(3,1),(1,2)], false, None).unwrap();

// Canonical permutation — same for isomorphic graphs after relabeling
let p1 = canonical_permutation(&g1, None).unwrap();
let p2 = canonical_permutation(&g2, None).unwrap();
let c1 = permute_vertices(&g1, &p1).unwrap();
let c2 = permute_vertices(&g2, &p2).unwrap();
assert_eq!(c1.ecount(), c2.ecount());

// Automorphism group order: |Aut(C_4)| = 8
let aut = count_automorphisms(&g1, None).unwrap();
assert!((aut - 8.0).abs() < 1e-9);

// BLISS-backed isomorphism with mapping
let result = isomorphic_bliss(&g1, &g2, None, None).unwrap();
assert!(result.iso);
```

## Graph operators

```rust
use rust_igraph::Graph;

let a = Graph::from_edges(&[(0,1),(1,2)], false, None).unwrap();
let b = Graph::from_edges(&[(1,2),(2,3)], false, None).unwrap();

let u = &a | &b;  // union
let i = &a & &b;  // intersection

println!("Union: {} vertices, {} edges", u.vcount(), u.ecount());
println!("Intersection: {} vertices, {} edges", i.vcount(), i.ecount());
```

## Graph layouts

rust-igraph includes 16 layout engines for 2D and 3D graph visualization:

```rust
use rust_igraph::{Graph, FrParams, KkParams, layout_fruchterman_reingold, layout_circle, layout_kamada_kawai};

let g = Graph::from_edges(
    &[(0,1),(1,2),(2,3),(3,0),(0,2),(1,3)], false, None
).unwrap();

// Force-directed layout (Fruchterman-Reingold)
let coords = layout_fruchterman_reingold(&g, &FrParams::default()).unwrap();
for (i, &(x, y)) in coords.iter().enumerate() {
    println!("v{i}: ({x:.2}, {y:.2})");
}

// Circle layout (deterministic)
let circle = layout_circle(&g, None);
assert_eq!(circle.len(), g.vcount() as usize);

// Kamada-Kawai (energy-based)
let n = g.vcount() as usize;
let kk = layout_kamada_kawai(&g, None, &KkParams::default_for(n), None).unwrap();
assert_eq!(kk.len(), n);
```

Available engines: Fruchterman-Reingold, Kamada-Kawai, DrL, Sugiyama,
GEM, Davidson-Harel, GraphOpt, MDS, LGL, UMAP, Reingold-Tilford,
circle, star, grid, random, and sphere.

## Iterating over a graph

```rust
use rust_igraph::Graph;

let g = Graph::from_edges(&[(0,1),(1,2),(2,0)], false, None).unwrap();

// Iterate over edges
for (src, tgt) in &g {
    println!("{} -- {}", src, tgt);
}

// Iterate over vertex ids
for v in g.vertex_ids() {
    let deg = g.degree(v).unwrap();
    println!("Vertex {}: degree {}", v, deg);
}
```

## Method API vs free functions

Most algorithms are available both as free functions and as methods on
`Graph`:

```rust
use rust_igraph::{Graph, pagerank};

let g = Graph::from_edges(&[(0,1),(1,2),(2,0)], false, None).unwrap();

// Free function style
let pr1 = pagerank(&g).unwrap();

// Method style
let pr2 = g.pagerank().unwrap();

// Same result
assert_eq!(pr1, pr2);
```

## Next steps

- Browse the [API documentation](./api.md) for the full list of functions
- Run `cargo run --example social_network_demo` for a comprehensive demo
- Check the 115 examples in the `examples/` directory

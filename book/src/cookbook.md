# Cookbook

Practical patterns for common graph analysis tasks. Each recipe is
self-contained — paste it into a file and run with
`cargo run --example <name>`, or use the snippets directly in your project.

## Find the most important nodes

Combine multiple centrality measures to identify key vertices:

```rust
use rust_igraph::{Graph, pagerank, betweenness, harmonic_centrality};

let g = Graph::from_edges(
    &[(0,1),(0,2),(1,2),(1,3),(2,3),(3,4),(4,5),(5,6),(6,4),(6,7),(7,8),(8,9),(9,7)],
    false, None
).unwrap();

let pr = pagerank(&g).unwrap();
let bc = betweenness(&g).unwrap();
let hc = harmonic_centrality(&g).unwrap();

// Rank vertices by PageRank
let mut ranked: Vec<(usize, f64)> = pr.iter().copied().enumerate().collect();
ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

println!("Top vertices by PageRank:");
for &(v, score) in ranked.iter().take(3) {
    println!("  v{v}: PR={score:.4}  betweenness={:.2}  harmonic={:.4}",
        bc[v], hc[v]);
}
```

## Compare community detection algorithms

Run multiple algorithms and measure agreement:

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

println!("Louvain:  {} communities, modularity {:.4}",
    *c_louvain.membership.iter().max().unwrap() + 1, c_louvain.modularity);
println!("Leiden:   {} communities, modularity {:.4}",
    *c_leiden.membership.iter().max().unwrap() + 1, c_leiden.modularity);

// Compare partitions using Normalized Mutual Information
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

## Build a weighted graph and find shortest paths

```rust
use rust_igraph::{Graph, dijkstra_distances, dijkstra_paths};

let g = Graph::from_edges(
    &[(0,1),(0,2),(1,3),(2,3),(3,4),(2,4)],
    false, None
).unwrap();

let weights = vec![1.0, 4.0, 2.0, 1.0, 3.0, 2.0];

// Distance vector from source 0
let dist = dijkstra_distances(&g, 0, &weights).unwrap();
for (v, d) in dist.iter().enumerate() {
    match d {
        Some(d) => println!("  0 -> {v}: distance {d:.1}"),
        None => println!("  0 -> {v}: unreachable"),
    }
}

// Full path reconstruction
let paths = dijkstra_paths(&g, 0, &weights).unwrap();
if let Some(parent) = paths.parents[4] {
    println!("Vertex 4 reached via vertex {parent}");
}
```

## Analyze graph connectivity

```rust
use rust_igraph::{Graph, connected_components, articulation_points, bridges};

let g = Graph::from_edges(
    &[(0,1),(1,2),(2,0),(2,3),(3,4),(4,5),(5,3)],
    false, None
).unwrap();

let cc = connected_components(&g).unwrap();
println!("Components: {}", cc.count);

let ap = articulation_points(&g).unwrap();
println!("Articulation points: {:?}", ap);

let br = bridges(&g).unwrap();
println!("Bridges: {:?}", br);
```

## Generate and analyze random graphs

```rust
use rust_igraph::{Graph, density, connected_components,
                  erdos_renyi_gnp, barabasi_game_bag, watts_strogatz_game};

// Erdos-Renyi: G(n, p)
let er = erdos_renyi_gnp(100, 0.05, false, false, 42).unwrap();
let er_cc = connected_components(&er).unwrap();
println!("ER(100, 0.05): {} edges, {} components",
    er.ecount(), er_cc.count);

// Barabasi-Albert: preferential attachment
let ba = barabasi_game_bag(100, 3, false, false, 42).unwrap();
println!("BA(100, m=3): {} edges, density {:.4}",
    ba.ecount(), density(&ba).unwrap().unwrap_or(0.0));

// Watts-Strogatz: small-world
let ws = watts_strogatz_game(100, 4, 0.1, false, false, 42).unwrap();
println!("WS(100, k=4, p=0.1): {} edges", ws.ecount());
```

You can also use the convenience methods on `Graph`:

```rust
use rust_igraph::Graph;

let er = Graph::erdos_renyi(100, 0.05, 42).unwrap();
let ba = Graph::barabasi_albert(100, 3, 42).unwrap();
let ws = Graph::watts_strogatz(100, 4, 0.1, 42).unwrap();
```

## Round-trip graph with attributes through GML

```rust
use rust_igraph::{Graph, AttributeValue, write_gml, read_gml};

let mut g = Graph::from_edges(&[(0,1),(1,2),(2,0)], false, None).unwrap();

// Add vertex names
for (i, name) in ["Alice", "Bob", "Carol"].iter().enumerate() {
    g.set_vertex_attribute("name", i as u32,
        AttributeValue::String((*name).into())).unwrap();
}

// Add edge weights
g.set_edge_attribute("weight", 0, AttributeValue::Numeric(1.5)).unwrap();
g.set_edge_attribute("weight", 1, AttributeValue::Numeric(2.0)).unwrap();
g.set_edge_attribute("weight", 2, AttributeValue::Numeric(0.8)).unwrap();

// Serialize and deserialize
let mut buf = Vec::new();
write_gml(&g, &mut buf).unwrap();

let g2 = read_gml(buf.as_slice()).unwrap();
assert_eq!(g2.vcount(), 3);
assert_eq!(
    g2.vertex_attribute("name", 0).and_then(AttributeValue::as_str),
    Some("Alice")
);
```

## Check graph isomorphism with BLISS

```rust
use rust_igraph::{Graph, isomorphic_bliss, canonical_permutation,
                  count_automorphisms, permute_vertices};

let g1 = Graph::from_edges(&[(0,1),(1,2),(2,3),(3,0)], false, None).unwrap();
// Same graph with vertices relabeled
let g2 = Graph::from_edges(&[(2,0),(0,3),(3,1),(1,2)], false, None).unwrap();

let result = isomorphic_bliss(&g1, &g2, None, None).unwrap();
assert!(result.iso);
println!("Isomorphic: {}", result.iso);
if !result.map12.is_empty() {
    println!("Mapping g1->g2: {:?}", result.map12);
}

// Canonical form: two isomorphic graphs get the same canonical labeling
let perm1 = canonical_permutation(&g1, None).unwrap();
let perm2 = canonical_permutation(&g2, None).unwrap();
let c1 = permute_vertices(&g1, &perm1).unwrap();
let c2 = permute_vertices(&g2, &perm2).unwrap();
// c1 and c2 have the same edge set

// Count automorphisms
let aut = count_automorphisms(&g1, None).unwrap();
println!("|Aut(C_4)| = {aut}");  // 8
```

## Compute a minimum spanning tree

```rust
use rust_igraph::{Graph, minimum_spanning_tree, MstAlgorithm};

let g = Graph::from_edges(
    &[(0,1),(0,2),(1,2),(1,3),(2,3),(3,4)],
    false, None
).unwrap();
let weights = vec![4.0, 2.0, 1.0, 3.0, 5.0, 1.0];

let mst_edges = minimum_spanning_tree(&g, Some(&weights), MstAlgorithm::Automatic).unwrap();
println!("MST edges: {:?}", mst_edges);

let total_weight: f64 = mst_edges.iter()
    .map(|&e| weights[e as usize])
    .sum();
println!("Total MST weight: {total_weight}");
```

## Network flow

```rust
use rust_igraph::{Graph, max_flow_value};

// A simple flow network (directed)
let g = Graph::from_edges(
    &[(0,1),(0,2),(1,3),(2,3),(1,2)],
    true, // directed
    None,
).unwrap();

let capacity = vec![3.0, 2.0, 2.0, 3.0, 1.0];

let flow = max_flow_value(&g, 0, 3, Some(&capacity)).unwrap();
println!("Max flow from 0 to 3: {flow}");
```

## Layout for visualization export

```rust
use rust_igraph::{Graph, FrParams, layout_fruchterman_reingold, layout_circle};

let g = Graph::from_edges(
    &[(0,1),(1,2),(2,3),(3,4),(4,0),(0,2),(1,3)],
    false, None
).unwrap();

// Force-directed layout
let coords = layout_fruchterman_reingold(&g, &FrParams::default()).unwrap();

// Output as CSV for use in external tools
println!("vertex,x,y");
for (v, &(x, y)) in coords.iter().enumerate() {
    println!("{v},{x:.6},{y:.6}");
}

// Circle layout (deterministic, good for small graphs)
let circle = layout_circle(&g, None);
for (v, &(x, y)) in circle.iter().enumerate() {
    println!("v{v}: ({x:.4}, {y:.4})");
}
```

use rust_igraph::{Graph, trussness};

fn main() {
    println!("=== trussness demo ===\n");

    // K4 complete graph: every edge in 2 triangles → trussness 4
    let mut g = Graph::with_vertices(4);
    for u in 0..4u32 {
        for v in (u + 1)..4 {
            g.add_edge(u, v).unwrap();
        }
    }
    let t = trussness(&g).unwrap();
    println!("K4 (6 edges): trussness = {t:?}");
    println!("  (expected: all 4)\n");

    // Triangle + bridge + triangle
    let mut g2 = Graph::with_vertices(6);
    for (u, v) in [(0, 1), (0, 2), (1, 2), (2, 3), (3, 4), (3, 5), (4, 5)] {
        g2.add_edge(u, v).unwrap();
    }
    let t2 = trussness(&g2).unwrap();
    println!("Two triangles joined by a bridge:");
    println!("  edges: 0-1, 0-2, 1-2, 2-3, 3-4, 3-5, 4-5");
    println!("  trussness = {t2:?}");
    println!("  (expected: [3, 3, 3, 2, 3, 3, 3])\n");

    // 12-vertex graph from igraph C test suite
    let mut g3 = Graph::with_vertices(12);
    for (u, v) in [
        (0, 1),
        (0, 2),
        (0, 3),
        (0, 4),
        (1, 2),
        (1, 3),
        (1, 4),
        (2, 3),
        (2, 4),
        (3, 4),
        (3, 6),
        (3, 11),
        (4, 5),
        (4, 6),
        (5, 6),
        (5, 7),
        (5, 8),
        (5, 9),
        (6, 7),
        (6, 10),
        (6, 11),
        (7, 8),
        (7, 9),
        (8, 9),
        (8, 10),
    ] {
        g3.add_edge(u, v).unwrap();
    }
    let t3 = trussness(&g3).unwrap();
    println!("12-vertex igraph C test graph (25 edges):");
    println!("  trussness = {t3:?}");
    println!("  K5 core (edges 0-9): all 5");
    println!("  K4 subcore (edges 15-17, 21-23): all 4");
    println!("  Bridge-like edges (19, 24): trussness 2");
}

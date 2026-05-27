use rust_igraph::{Graph, StrengthMode, diversity, strength, strength_with_mode};

fn main() {
    println!("=== strength demo ===\n");

    // Undirected triangle
    let mut g = Graph::with_vertices(3);
    for (u, v) in [(0, 1), (0, 2), (1, 2)] {
        g.add_edge(u, v).unwrap();
    }
    let w = [1.0, 2.0, 3.0];
    let s = strength(&g, &w).unwrap();
    println!("Triangle weights={w:?}");
    println!("  strength(All): {s:?}\n");

    // Directed graph
    let mut dg = Graph::new(4, true).unwrap();
    for (u, v) in [(0, 1), (0, 2), (1, 3), (2, 3)] {
        dg.add_edge(u, v).unwrap();
    }
    let dw = [1.0, 2.0, 3.0, 4.0];
    println!("Directed 4-vertex, weights={dw:?}");
    let out = strength_with_mode(&dg, &dw, StrengthMode::Out, true).unwrap();
    let ins = strength_with_mode(&dg, &dw, StrengthMode::In, true).unwrap();
    let all = strength_with_mode(&dg, &dw, StrengthMode::All, true).unwrap();
    println!("  strength(Out): {out:?}");
    println!("  strength(In):  {ins:?}");
    println!("  strength(All): {all:?}\n");

    println!("=== diversity demo ===\n");

    // 4 vertices, 5 edges (from igraph C test)
    let mut g2 = Graph::with_vertices(4);
    for (u, v) in [(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)] {
        g2.add_edge(u, v).unwrap();
    }
    let w2 = [3.0, 2.0, 8.0, 1.0, 1.0];
    let d = diversity(&g2, &w2).unwrap();
    println!("4-vertex graph, weights={w2:?}");
    println!("  diversity: {d:?}");
    println!("  (expected: ~[0.971, 0.75, 0.691, 1.0])");
}

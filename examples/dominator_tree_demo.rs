//! ALGO-FL-030 example: Lengauer-Tarjan dominator tree of a directed
//! flowgraph.
//!
//! Run: `cargo run --example dominator_tree_demo`.
//!
//! For each case we (1) build the directed input, (2) call
//! `dominator_tree(graph, root, mode)`, (3) print the `idom` vector
//! (with `-1` at root, `-2` at unreachable vertices), the tree edges,
//! and the `leftout` list, then (4) verify the fundamental property —
//! every reachable non-root vertex's `idom` lies on every path from the
//! root to it — by enumerating simple paths via DFS.

use rust_igraph::{DominatorMode, Graph, dominator_tree};

fn print_result(label: &str, n: u32, idom: &[i32], tree: &Graph, leftout: &[u32]) {
    println!("{label}");
    print!("  idom = [");
    for (i, &d) in idom.iter().enumerate() {
        if i > 0 {
            print!(", ");
        }
        if d == -1 {
            print!("root");
        } else if d == -2 {
            print!(" --");
        } else {
            print!("{d:>3}");
        }
    }
    println!("]");
    let m = u32::try_from(tree.ecount()).expect("ecount fits u32");
    println!("  tree edges ({m} total):");
    for eid in 0..m {
        let (u, v) = tree.edge(eid).expect("tree edge");
        println!("    edge {eid:>2}: {u} -> {v}");
    }
    println!(
        "  leftout = {:?}    (reachable from root: {})",
        leftout,
        n as usize - leftout.len()
    );
}

/// Verify that `idom[w]` appears on every simple path from `root` to `w`
/// in the directed input. Brute-force; only meant for tiny demos.
fn verify_dominance(label: &str, g: &Graph, root: u32, idom: &[i32]) {
    let n = g.vcount();
    let neighbors: Vec<Vec<u32>> = (0..n)
        .map(|v| g.neighbors(v).expect("vertex in range"))
        .collect();
    let mut mismatches = 0usize;
    for w in 0..n {
        if w == root || idom[w as usize] < 0 {
            continue;
        }
        let target = u32::try_from(idom[w as usize]).expect("idom in range");
        // Enumerate simple paths root -> w; assert `target` is on each.
        let mut on_stack = vec![false; n as usize];
        let mut path = Vec::new();
        dfs_paths(
            root,
            w,
            target,
            &neighbors,
            &mut on_stack,
            &mut path,
            &mut mismatches,
        );
    }
    println!("  {label}: {mismatches} mismatch(es) over all (path, w) checks");
}

fn dfs_paths(
    cur: u32,
    target_w: u32,
    must_visit: u32,
    nbrs: &[Vec<u32>],
    on_stack: &mut [bool],
    path: &mut Vec<u32>,
    mismatches: &mut usize,
) {
    on_stack[cur as usize] = true;
    path.push(cur);
    if cur == target_w {
        // We've reached w via the current `path`. The root is the first
        // element; `target_w` is the last. `must_visit` must appear
        // somewhere strictly between (idom != root, idom != w).
        if !path[..path.len() - 1].contains(&must_visit) {
            println!(
                "    MISMATCH: path {path:?} reaches {target_w} without going through idom={must_visit}"
            );
            *mismatches += 1;
        }
    } else {
        for &next in &nbrs[cur as usize] {
            if !on_stack[next as usize] {
                dfs_paths(next, target_w, must_visit, nbrs, on_stack, path, mismatches);
            }
        }
    }
    path.pop();
    on_stack[cur as usize] = false;
}

#[allow(clippy::too_many_lines)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1) Canonical 13-vertex Lengauer-Tarjan flowgraph (igraph C unit
    //    test, mode=Out). Expected idom = [-1, 0, 1, 0, 0, 1, 5, 0, 0, 0,
    //    0, 0, 11].
    let mut g13 = Graph::new(13, true)?;
    let edges13 = [
        (0u32, 1u32),
        (0, 7),
        (0, 10),
        (1, 2),
        (1, 5),
        (2, 3),
        (3, 4),
        (4, 3),
        (4, 0),
        (5, 3),
        (5, 6),
        (6, 3),
        (7, 8),
        (7, 10),
        (7, 11),
        (8, 9),
        (9, 4),
        (9, 8),
        (10, 11),
        (11, 12),
        (12, 9),
    ];
    for (u, v) in edges13 {
        g13.add_edge(u, v)?;
    }
    let dt = dominator_tree(&g13, 0, DominatorMode::Out)?;
    print_result(
        "1) 13-vertex classical LT example (igraph C reference)",
        g13.vcount(),
        &dt.idom,
        &dt.tree,
        &dt.leftout,
    );
    verify_dominance("brute-force path check", &g13, 0, &dt.idom);
    println!();

    // 2) 6-vertex tree-like DAG from rigraph test-flow.R (root=0, mode=Out).
    //    Edges (0,1),(1,2),(2,3),(1,4),(4,5); idom = [-1,0,1,2,1,4].
    let mut tree6 = Graph::new(6, true)?;
    for (u, v) in [(0u32, 1u32), (1, 2), (2, 3), (1, 4), (4, 5)] {
        tree6.add_edge(u, v)?;
    }
    let dt = dominator_tree(&tree6, 0, DominatorMode::Out)?;
    print_result(
        "2) 6-vertex DAG (R-igraph test-flow.R)",
        tree6.vcount(),
        &dt.idom,
        &dt.tree,
        &dt.leftout,
    );
    verify_dominance("brute-force path check", &tree6, 0, &dt.idom);
    println!();

    // 3) 20-vertex graph with unreachable component {5,6,7,16..19}
    //    (igraph C / python-igraph reference, mode=Out).
    let mut g20 = Graph::new(20, true)?;
    let edges20 = [
        (0u32, 1u32),
        (0, 2),
        (0, 3),
        (1, 4),
        (2, 1),
        (2, 4),
        (2, 8),
        (3, 9),
        (3, 10),
        (4, 15),
        (8, 11),
        (9, 12),
        (10, 12),
        (10, 13),
        (11, 8),
        (11, 14),
        (12, 14),
        (13, 12),
        (14, 12),
        (14, 0),
        (15, 11),
    ];
    for (u, v) in edges20 {
        g20.add_edge(u, v)?;
    }
    let dt = dominator_tree(&g20, 0, DominatorMode::Out)?;
    print_result(
        "3) 20-vertex flowgraph with unreachable component (mode=Out)",
        g20.vcount(),
        &dt.idom,
        &dt.tree,
        &dt.leftout,
    );
    verify_dominance("brute-force path check", &g20, 0, &dt.idom);
    println!();

    // 4) Same 13-vertex graph but with all edges reversed and mode=In —
    //    the In mode is equivalent to reversing the graph and using Out
    //    (post-dominator tree).
    let mut g13_rev = Graph::new(13, true)?;
    let edges_rev = [
        (1u32, 0u32),
        (2, 0),
        (3, 0),
        (4, 1),
        (1, 2),
        (4, 2),
        (5, 2),
        (6, 3),
        (7, 3),
        (12, 4),
        (8, 5),
        (9, 6),
        (9, 7),
        (10, 7),
        (5, 8),
        (11, 8),
        (11, 9),
        (9, 10),
        (9, 11),
        (0, 11),
        (8, 12),
    ];
    for (u, v) in edges_rev {
        g13_rev.add_edge(u, v)?;
    }
    let dt = dominator_tree(&g13_rev, 0, DominatorMode::In)?;
    print_result(
        "4) 13-vertex reversed flowgraph (mode=In post-dominator)",
        g13_rev.vcount(),
        &dt.idom,
        &dt.tree,
        &dt.leftout,
    );
    println!();

    println!("All four cases compute their dominator tree in O(|V|+|E|·α) time.");

    Ok(())
}

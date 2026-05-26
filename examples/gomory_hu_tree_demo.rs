//! ALGO-FL-020 example: Gomory-Hu cut tree — compact representation of
//! the *all-pairs* minimum s-t cut.
//!
//! Run: `cargo run --example gomory_hu_tree_demo`.
//!
//! For each case below we:
//!   1. Build an undirected graph + optional capacity vector.
//!   2. Call `gomory_hu_tree(graph, capacity)`.
//!   3. Print the tree edges with their flow weights, then verify the
//!      Gomory-Hu property on every pair via `max_flow_value` — for any
//!      `(u, v)`, the minimum edge weight along the unique tree path
//!      from `u` to `v` equals the max-flow value between `u` and `v` in
//!      the original graph.

use rust_igraph::{Graph, gomory_hu_tree, max_flow_value};

fn print_tree(label: &str, gh_tree: &Graph, flows: &[f64]) {
    println!("{label}");
    let m = u32::try_from(gh_tree.ecount()).expect("ecount fits u32");
    println!("  tree vcount = {}, ecount = {}", gh_tree.vcount(), m);
    for eid in 0..m {
        let (u, v) = gh_tree.edge(eid).expect("edge");
        println!(
            "    tree edge {eid:>2}: ({u}, {v})   flow = {:>5.1}",
            flows[eid as usize]
        );
    }
}

fn verify_property(graph: &Graph, gh_tree: &Graph, flows: &[f64], cap: Option<&[f64]>) {
    let n = graph.vcount();
    if n < 2 {
        println!("  (no pairs to verify)");
        return;
    }
    let m = u32::try_from(gh_tree.ecount()).expect("ecount fits u32");
    let mut adj: Vec<Vec<(u32, u32)>> = vec![Vec::new(); n as usize];
    for eid in 0..m {
        let (u, v) = gh_tree.edge(eid).expect("tree edge");
        adj[u as usize].push((v, eid));
        adj[v as usize].push((u, eid));
    }

    let mut mismatches = 0usize;
    for u in 0..n {
        for v in (u + 1)..n {
            let mut parent: Vec<Option<(u32, u32)>> = vec![None; n as usize];
            let mut visited = vec![false; n as usize];
            visited[u as usize] = true;
            let mut q = vec![u];
            let mut hp = 0;
            while hp < q.len() && !visited[v as usize] {
                let cur = q[hp];
                hp += 1;
                for &(w, eid) in &adj[cur as usize] {
                    if !visited[w as usize] {
                        visited[w as usize] = true;
                        parent[w as usize] = Some((cur, eid));
                        q.push(w);
                    }
                }
            }
            let mut cur = v;
            let mut min_w: Option<f64> = None;
            while let Some((p, eid)) = parent[cur as usize] {
                let w = flows[eid as usize];
                min_w = Some(min_w.map_or(w, |m: f64| m.min(w)));
                cur = p;
            }
            let tree_min = min_w.expect("path exists");
            let mf = max_flow_value(graph, u, v, cap).expect("max_flow_value");
            if (tree_min - mf).abs() > 1e-9 {
                println!("    MISMATCH ({u},{v}): tree-min = {tree_min}, max-flow = {mf}");
                mismatches += 1;
            }
        }
    }
    println!(
        "  Gomory-Hu property: {} pair(s) checked, {} mismatch(es)",
        n * (n - 1) / 2,
        mismatches
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1) K_4 unit caps — every pair has min-cut 3, every tree edge weight 3.
    let mut k4 = Graph::new(4, false)?;
    for u in 0u32..4 {
        for v in (u + 1)..4 {
            k4.add_edge(u, v)?;
        }
    }
    let gh = gomory_hu_tree(&k4, None)?;
    print_tree(
        "1) K_4 unit caps (issue #1810 regression)",
        &gh.tree,
        &gh.flows,
    );
    verify_property(&k4, &gh.tree, &gh.flows, None);
    println!();

    // 2) C_5 cycle unit caps — every pair has max-flow 2.
    let mut c5 = Graph::new(5, false)?;
    for i in 0u32..5 {
        c5.add_edge(i, (i + 1) % 5)?;
    }
    let gh = gomory_hu_tree(&c5, None)?;
    print_tree("2) C_5 cycle unit caps", &gh.tree, &gh.flows);
    verify_property(&c5, &gh.tree, &gh.flows, None);
    println!();

    // 3) Canonical 6v weighted reference from
    //    references/igraph/tests/unit/igraph_gomory_hu_tree.c:178-191.
    //    Edges (0-1)(0-2)(1-2)(1-3)(1-4)(2-4)(3-4)(3-5)(4-5),
    //    caps [1,7,1,3,2,4,1,6,2].
    let mut g6 = Graph::new(6, false)?;
    let edges = [
        (0u32, 1u32),
        (0, 2),
        (1, 2),
        (1, 3),
        (1, 4),
        (2, 4),
        (3, 4),
        (3, 5),
        (4, 5),
    ];
    let caps = [1.0, 7.0, 1.0, 3.0, 2.0, 4.0, 1.0, 6.0, 2.0];
    for (u, v) in edges {
        g6.add_edge(u, v)?;
    }
    let gh = gomory_hu_tree(&g6, Some(&caps))?;
    print_tree(
        "3) Canonical igraph C unit-test 6v weighted reference",
        &gh.tree,
        &gh.flows,
    );
    verify_property(&g6, &gh.tree, &gh.flows, Some(&caps));
    println!();

    // 4) Path P_4 with non-uniform caps — the (1,2) bridge of cap 1
    //    dominates every pair crossing it.
    let mut p4 = Graph::new(4, false)?;
    p4.add_edge(0, 1)?;
    p4.add_edge(1, 2)?;
    p4.add_edge(2, 3)?;
    let pcaps = [3.0, 1.0, 5.0];
    let gh = gomory_hu_tree(&p4, Some(&pcaps))?;
    print_tree(
        "4) P_4 path with non-uniform caps [3,1,5]",
        &gh.tree,
        &gh.flows,
    );
    verify_property(&p4, &gh.tree, &gh.flows, Some(&pcaps));
    println!();

    // 5) Petersen graph (10v 3-regular) unit caps — 3-edge-connected,
    //    so every tree edge weight is 3.
    let mut pet = Graph::new(10, false)?;
    // Outer 5-cycle + inner pentagram + spokes (classic Petersen layout).
    let pet_edges = [
        (0u32, 1u32),
        (1, 2),
        (2, 3),
        (3, 4),
        (4, 0), // outer
        (5, 7),
        (7, 9),
        (9, 6),
        (6, 8),
        (8, 5), // inner pentagram
        (0, 5),
        (1, 6),
        (2, 7),
        (3, 8),
        (4, 9), // spokes
    ];
    for (u, v) in pet_edges {
        pet.add_edge(u, v)?;
    }
    let gh = gomory_hu_tree(&pet, None)?;
    print_tree(
        "5) Petersen graph (10v 3-regular) unit caps",
        &gh.tree,
        &gh.flows,
    );
    verify_property(&pet, &gh.tree, &gh.flows, None);
    println!();

    println!("All five cases satisfy the Gomory-Hu property:");
    println!("  for every (u, v), min(tree path edge weights) == max_flow_value(u, v).");

    Ok(())
}

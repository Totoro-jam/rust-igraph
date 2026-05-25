//! ALGO-CN-020 example: walk every named graph and verify the
//! published `(vcount, ecount)` invariants.
//!
//! `famous("<name>")` is the Rust analogue of `igraph_famous(name)`
//! (and `Graph.Famous(name)` in python-igraph). 31 small graphs from
//! classical graph theory ship as static edge tables in
//! `src/algorithms/constructors/famous.rs`, transliterated byte-for-byte
//! from the upstream C source so name dispatch and edge order match the
//! C, Python and R bindings.
//!
//! Run: `cargo run --example famous_demo`.

use rust_igraph::{Graph, VertexId, famous, famous_names};
use std::collections::BTreeMap;

fn degree_histogram(g: &Graph) -> BTreeMap<usize, u32> {
    let n = g.vcount();
    let mut hist: BTreeMap<usize, u32> = BTreeMap::new();
    for v in 0..n {
        let d = g.neighbors(v).expect("neighbors").len();
        *hist.entry(d).or_insert(0) += 1;
    }
    hist
}

fn is_simple_undirected(g: &Graph) -> bool {
    let n = g.vcount();
    for v in 0..n {
        let nbrs: Vec<VertexId> = g.neighbors(v).expect("neighbors");
        let mut sorted = nbrs.clone();
        sorted.sort_unstable();
        sorted.dedup();
        if sorted.len() != nbrs.len() {
            return false;
        }
        if sorted.contains(&v) {
            return false;
        }
    }
    true
}

fn print_summary(name: &str, g: &Graph) {
    let hist = degree_histogram(g);
    let min_deg = hist.keys().min().copied().unwrap_or(0);
    let max_deg = hist.keys().max().copied().unwrap_or(0);
    let regular = hist.len() == 1;
    println!(
        "  {name:<20}  v={vc:>3}  e={ec:>4}  deg=[{mn},{mx}]{reg}",
        vc = g.vcount(),
        ec = g.ecount(),
        mn = min_deg,
        mx = max_deg,
        reg = if regular { " (regular)" } else { "" },
    );
}

fn main() {
    // 1) Walk every canonical name and report (v, e, degree window).
    println!(
        "--- famous_names() catalogue ({} graphs) ---",
        famous_names().len()
    );
    for name in famous_names() {
        let g = famous(name).expect("famous(name) on a canonical name");
        print_summary(name, &g);
        assert!(
            is_simple_undirected(&g),
            "{name} should be simple undirected"
        );
    }

    // 2) Spot-check well-known counts (literature-standard).
    let petersen = famous("Petersen").expect("Petersen");
    assert_eq!((petersen.vcount(), petersen.ecount()), (10, 15));
    let zachary = famous("Zachary").expect("Zachary karate club");
    assert_eq!((zachary.vcount(), zachary.ecount()), (34, 78));
    let tutte = famous("Tutte").expect("Tutte");
    assert_eq!((tutte.vcount(), tutte.ecount()), (46, 69));
    let meredith = famous("Meredith").expect("Meredith");
    assert_eq!((meredith.vcount(), meredith.ecount()), (70, 140));

    // 3) Case-insensitive dispatch — same graph for any casing.
    let a = famous("Petersen").expect("title case");
    let b = famous("petersen").expect("lowercase");
    let c = famous("PETERSEN").expect("uppercase");
    assert_eq!(a.ecount(), b.ecount());
    assert_eq!(b.ecount(), c.ecount());

    // 4) Aliases — `dodecahedral` is the same graph as `dodecahedron`.
    let dod1 = famous("Dodecahedron").expect("dodecahedron");
    let dod2 = famous("Dodecahedral").expect("dodecahedral alias");
    assert_eq!(dod1.vcount(), dod2.vcount());
    assert_eq!(dod1.ecount(), dod2.ecount());

    // Same for German/English Grötzsch spelling.
    let g1 = famous("Grotzsch").expect("grotzsch");
    let g2 = famous("Groetzsch").expect("groetzsch alias");
    assert_eq!(g1.vcount(), g2.vcount());
    assert_eq!(g1.ecount(), g2.ecount());

    // 5) Unknown names yield a typed error rather than panicking.
    let err = famous("NotARealGraph").unwrap_err();
    println!("\nunknown-name error: {err}");

    println!("\nall famous() invariants OK ✓");
}

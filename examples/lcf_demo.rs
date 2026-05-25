//! ALGO-CN-018 example: build cubic Hamiltonian graphs from LCF
//! notation (`igraph_lcf`).
//!
//! LCF (Lederberg-Coxeter-Frucht) notation describes a 3-regular
//! Hamiltonian graph by listing the chord offsets relative to a Hamilton
//! cycle. The C call `lcf(n, &[s0, s1, ...], repeats)` builds the cycle
//! `0-1-2-…-(n-1)-0` and then, for each sptr in `0..repeats*shifts.len()`,
//! adds a chord from `sptr % n` to `(sptr + shifts[sptr % shifts.len()]) mod n`.
//! Duplicate chords and self-loops are collapsed by the inline simplify
//! pass — the result is always undirected and simple.
//!
//! This demo walks the canonical catalogue:
//!
//! * **Franklin** `[5, -5]^6` on n=12 — 18 edges, bipartite cubic.
//! * **Heawood** `[5, -5]^7` on n=14 — 21 edges, girth 6, the smallest
//!   3-regular cage of girth 6.
//! * **Frucht** `[-5,-2,-4,2,5,-2,2,5,-2,-5,4,2]^1` on n=12 — 18 edges,
//!   the only 3-regular planar graph with no non-trivial automorphism.
//! * **Truncated tetrahedron** `[2, 6, -2, -6]^3` on n=12 — 18 edges,
//!   one of the Archimedean solids.
//! * **Truncated octahedron** `[3, -7, 7, -3]^6` on n=24 — 36 edges.
//!
//! Plus three corner cases:
//!
//! * **Empty shifts** — `lcf(n, &[], k)` produces a pure Hamilton cycle.
//! * **Self-loop collapse on `n=2`** — every chord is a self-loop; the
//!   backbone (0,1) survives, ecount=1.
//! * **bug #996 regression** — `lcf(0, &[], 0)` is the empty graph.
//!
//! Run: `cargo run --example lcf_demo`.

use rust_igraph::{Graph, VertexId, lcf};
use std::collections::BTreeMap;

fn print_summary(label: &str, g: &Graph) {
    println!("--- {label} ---");
    println!("  vcount   = {}", g.vcount());
    println!("  ecount   = {}", g.ecount());
    println!("  directed = {}", g.is_directed());
}

fn assert_cubic(label: &str, g: &Graph) {
    for v in 0..g.vcount() {
        let d = g.neighbors(v).expect("neighbors").len();
        assert_eq!(d, 3, "{label}: vertex {v} expected degree 3, got {d}");
    }
}

fn assert_simple_undirected(label: &str, g: &Graph) {
    assert!(!g.is_directed(), "{label}: expected undirected");
    let m = u32::try_from(g.ecount()).expect("ecount fits u32 in example");
    let mut hist: BTreeMap<(VertexId, VertexId), u32> = BTreeMap::new();
    for e in 0..m {
        let (a, b) = g.edge(e).expect("edge in range");
        assert_ne!(a, b, "{label}: self-loop survived");
        let key = if a <= b { (a, b) } else { (b, a) };
        *hist.entry(key).or_insert(0) += 1;
    }
    for (k, count) in &hist {
        assert_eq!(*count, 1, "{label}: parallel edge {k:?} survived");
    }
}

fn main() {
    // 1) Franklin graph — [5, -5]^6, n=12.
    let franklin = lcf(12, &[5, -5], 6).expect("franklin");
    print_summary("Franklin [5,-5]^6 n=12", &franklin);
    assert_eq!(franklin.ecount(), 18);
    assert_cubic("franklin", &franklin);
    assert_simple_undirected("franklin", &franklin);

    // 2) Heawood graph — [5, -5]^7, n=14.
    let heawood = lcf(14, &[5, -5], 7).expect("heawood");
    print_summary("Heawood [5,-5]^7 n=14", &heawood);
    assert_eq!(heawood.ecount(), 21);
    assert_cubic("heawood", &heawood);
    assert_simple_undirected("heawood", &heawood);

    // 3) Frucht graph — [-5,-2,-4,2,5,-2,2,5,-2,-5,4,2]^1, n=12.
    let frucht_shifts: &[i64] = &[-5, -2, -4, 2, 5, -2, 2, 5, -2, -5, 4, 2];
    let frucht = lcf(12, frucht_shifts, 1).expect("frucht");
    print_summary("Frucht [12-shift]^1 n=12", &frucht);
    assert_eq!(frucht.ecount(), 18);
    assert_cubic("frucht", &frucht);
    assert_simple_undirected("frucht", &frucht);

    // 4) Truncated tetrahedron — [2, 6, -2, -6]^3, n=12.
    let tt = lcf(12, &[2, 6, -2, -6], 3).expect("truncated_tetrahedron");
    print_summary("Truncated tetrahedron [2,6,-2,-6]^3 n=12", &tt);
    assert_eq!(tt.ecount(), 18);
    assert_cubic("truncated_tetrahedron", &tt);
    assert_simple_undirected("truncated_tetrahedron", &tt);

    // 5) Truncated octahedron — [3, -7, 7, -3]^6, n=24.
    let to = lcf(24, &[3, -7, 7, -3], 6).expect("truncated_octahedron");
    print_summary("Truncated octahedron [3,-7,7,-3]^6 n=24", &to);
    assert_eq!(to.ecount(), 36);
    assert_cubic("truncated_octahedron", &to);
    assert_simple_undirected("truncated_octahedron", &to);

    // 6) Empty shifts → pure Hamilton cycle.
    let c6 = lcf(6, &[], 0).expect("pure cycle");
    print_summary("Pure cycle lcf(6, [], 0) → C_6", &c6);
    assert_eq!(c6.ecount(), 6);
    for v in 0..c6.vcount() {
        assert_eq!(c6.neighbors(v).expect("neighbors").len(), 2);
    }

    // 7) n=2 with chord pattern — every chord is a self-loop, only the
    //    backbone edge (0, 1) survives the simplify pass.
    let n2 = lcf(2, &[2, -2], 2).expect("n=2 collapse");
    print_summary("Collapse lcf(2, [2,-2], 2)", &n2);
    assert_eq!(n2.ecount(), 1);

    // 8) bug #996 regression — lcf(0, [], 0) is the empty graph.
    let null = lcf(0, &[], 0).expect("bug #996 null");
    print_summary("Null lcf(0, [], 0) — bug #996 regression", &null);
    assert_eq!(null.vcount(), 0);
    assert_eq!(null.ecount(), 0);

    println!("\nall lcf cases OK ✓");
}

//! Parent-vector tree-decoder benchmarks (ALGO-CN-017).
//!
//! Run: `cargo bench --bench bench_tree_from_parent_vector`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-CN-017.json`.
//!
//! The decoder is `O(n)`: each vertex is visited at most twice across
//! all rounds (once as the outer index `i`, once on the way up its
//! parent chain). The interesting axis is therefore `n` itself plus the
//! topology of the parent vector — different shapes stress different
//! branches of the per-round seen-marking machinery:
//!
//! * **`chain`** — `parents[v] = v - 1`. The very first round walks
//!   from `i = 0` to the deepest leaf in one sweep, so every subsequent
//!   `i` short-circuits on the `seen[v] != 0` skip. Worst case for the
//!   inner traversal length.
//! * **`star`** — vertex `0` is the root, every other vertex points
//!   straight at `0`. Inner loop fires once per `i` and never cascades.
//! * **`forest`** — alternating roots and parent links: every other
//!   vertex is a fresh root, the rest chain locally to the previous
//!   vertex. Many short chains, mimics decoding a wide BFS predecessor
//!   array from a disconnected graph.
//! * **`random`** — deterministic SplitMix64-seeded parent ids in
//!   `[0, v)` (still acyclic by construction since parents reference
//!   strictly smaller indices). Represents a typical mixed-depth tree
//!   with cache-unfriendly access patterns.
//!
//! Throughput is reported in `n` elements per second so cross-shape
//! numbers stay directly comparable as "trees decoded per second" in
//! vertex-count terms.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{TreeMode, tree_from_parent_vector};

fn make_chain(n: u32) -> Vec<i64> {
    let mut parents = Vec::with_capacity(n as usize);
    parents.push(-1);
    for v in 1..n {
        parents.push(i64::from(v - 1));
    }
    parents
}

fn make_star(n: u32) -> Vec<i64> {
    let mut parents = Vec::with_capacity(n as usize);
    parents.push(-1);
    parents.extend(std::iter::repeat_n(0i64, (n - 1) as usize));
    parents
}

fn make_forest(n: u32) -> Vec<i64> {
    // Even-indexed vertices are roots; odd-indexed point to the previous.
    let mut parents = Vec::with_capacity(n as usize);
    for v in 0..n {
        if v % 2 == 0 {
            parents.push(-1);
        } else {
            parents.push(i64::from(v - 1));
        }
    }
    parents
}

fn make_random(n: u32) -> Vec<i64> {
    // SplitMix64 step (same constants as bench_prufer for repo-wide
    // reproducibility). Every non-root vertex `v` picks a parent in
    // `[0, v)` so the parent vector stays acyclic by construction.
    let mut state: u64 = 0xDEAD_BEEF_CAFE_F00D;
    let mut parents = Vec::with_capacity(n as usize);
    parents.push(-1);
    for v in 1..n {
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        let r = z % u64::from(v);
        parents.push(i64::try_from(r).expect("modulo result fits i64"));
    }
    parents
}

fn bench_shape<F>(c: &mut Criterion, label: &str, build: F)
where
    F: Fn(u32) -> Vec<i64>,
{
    let mut group = c.benchmark_group(format!("tree_from_parent_vector/{label}"));
    for n in [64u32, 1_024, 16_384, 131_072] {
        let parents = build(n);
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &parents, |b, parents| {
            b.iter(|| tree_from_parent_vector(parents, TreeMode::Out).unwrap());
        });
    }
    group.finish();
}

fn bench_chain(c: &mut Criterion) {
    bench_shape(c, "chain", make_chain);
}

fn bench_star(c: &mut Criterion) {
    bench_shape(c, "star", make_star);
}

fn bench_forest(c: &mut Criterion) {
    bench_shape(c, "forest", make_forest);
}

fn bench_random(c: &mut Criterion) {
    bench_shape(c, "random", make_random);
}

criterion_group!(benches, bench_chain, bench_star, bench_forest, bench_random);
criterion_main!(benches);

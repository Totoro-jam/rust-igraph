//! Canonical labeling via a hand-rolled individualization-refinement (I-R)
//! engine (`ALGO-ISO-003`..`ALGO-ISO-006`).
//!
//! Upstream igraph backs `canonical_permutation`, `count_automorphisms`,
//! `automorphism_group` and `isomorphic_bliss` with the ~150 KB C++ `bliss`
//! library. The *canonical labeling* it produces is implementation-defined:
//! only the automorphism count, the automorphism group and the
//! isomorphism yes/no verdict are externally observable. Rather than port
//! `bliss` line-by-line we hand-roll a correct pure-Rust
//! individualization-refinement engine (the McKay/nauty/bliss family) and
//! expose the four public functions on top of it.
//!
//! The engine performs a single depth-first search over an
//! individualization-refinement tree and computes all three observables at
//! once:
//!
//! * a **canonical labeling** — the vertex ordering whose relabeled
//!   adjacency matrix is lexicographically maximal,
//! * the full **automorphism group** (and a small generating set),
//! * the **group order** `|Aut(G)|`.
//!
//! # Algorithm
//!
//! 1. **Initial partition.** Vertices are grouped into ordered cells by the
//!    pair `(supplied colour, self-loop flag)`; with no colours this is a
//!    single cell (refined immediately by degree).
//! 2. **Refinement** (1-WL / equitable partition). Each cell is split by the
//!    multiset of neighbour-counts into every other cell until the partition
//!    is stable. For directed graphs both in- and out-neighbour counts are
//!    used so orientation is respected. The cell *order* is derived purely
//!    from colour ids, hence isomorphism-invariant.
//! 3. **Leaf.** When the partition is discrete (every cell a singleton) the
//!    cell order is a candidate labeling; its relabeled adjacency matrix is
//!    the leaf's *certificate*.
//! 4. **Branch.** Otherwise the first non-singleton cell is the target cell;
//!    each of its vertices is individualized (split into its own singleton)
//!    and the search recurses.
//!
//! The canonical form is the lexicographically maximal certificate over all
//! leaves. Crucially, **every leaf whose certificate equals the canonical
//! one yields a distinct automorphism, and conversely** — so `|Aut(G)|` is
//! exactly the number of such leaves and the automorphisms are recovered by
//! composing each maximal leaf's labeling with the canonical one.
//!
//! Generators are collected on-the-fly using orbit tracking: when a
//! matching leaf produces an automorphism that merges previously-separate
//! orbits, it is added to the generator set. The generator set is capped at
//! `n` elements; after that, only orbit-merging automorphisms are added
//! (O(n) generators always suffice by Schreier's lemma). This avoids the
//! expensive full-group enumeration that a naive greedy approach would need.
//!
//! # Scope (v1)
//!
//! Simple graphs (directed and undirected) with optional vertex colours,
//! self-loops allowed. Multi-edges are rejected (bliss does not support them
//! either). The search explores the full I-R tree; orbit pruning is used
//! within the generator-extraction phase but the full tree is enumerated for
//! group-order correctness.

// Vertex indices are bounded by `vcount`, which fits `u32` by the `Graph`
// contract; the `as u32` conversions when packing results therefore cannot
// truncate or change sign.
#![allow(clippy::cast_possible_truncation)]

use std::collections::HashSet;

use crate::core::{Graph, IgraphError, IgraphResult};

pub(crate) mod automorphism_group;
pub(crate) mod canonical_permutation;
pub(crate) mod count_automorphisms;
pub(crate) mod isomorphic_bliss;

/// Full result of one canonicalization run.
pub(crate) struct Canonicalization {
    /// `labeling[v]` is the canonical position assigned to vertex `v`
    /// (vertex → position). Empty for the null graph.
    pub labeling: Vec<u32>,
    /// A small generating set of `Aut(G)`. Each entry is a permutation
    /// `g` with `g[v]` the image of vertex `v`. Empty for the trivial group.
    /// Consumed by the `automorphism_group` wrapper (ALGO-ISO-005).
    pub generators: Vec<Vec<u32>>,
    /// `|Aut(G)|` as `f64` (exact for groups up to `2^53`).
    /// Consumed by the `count_automorphisms` wrapper (ALGO-ISO-004).
    pub group_order: f64,
    /// The canonical certificate: the relabeled adjacency matrix (under the
    /// canonical labeling) packed row-major into 64-bit words. Two graphs of
    /// equal order and directedness are isomorphic iff their certificates are
    /// equal. Empty for the null graph. Consumed by the `isomorphic_bliss`
    /// wrapper (ALGO-ISO-006).
    pub certificate: Vec<u64>,
}

/// Adjacency representation used by the engine.
#[allow(clippy::struct_field_names)]
struct Adj {
    n: usize,
    directed: bool,
    /// Sorted, deduplicated out-neighbours (all neighbours when undirected).
    out_adj: Vec<Vec<usize>>,
    /// Sorted, deduplicated in-neighbours (equal to `out_adj` when undirected).
    in_adj: Vec<Vec<usize>>,
    /// Dense adjacency matrix; `mat[u][v]` is true iff edge `u -> v` exists.
    mat: Vec<Vec<bool>>,
}

impl Adj {
    fn build(graph: &Graph) -> IgraphResult<Self> {
        let n = graph.vcount() as usize;
        let directed = graph.is_directed();
        let m = graph.ecount();

        let mut out_adj = vec![Vec::new(); n];
        let mut in_adj = vec![Vec::new(); n];
        let mut mat = vec![vec![false; n]; n];

        // Reject multi-edges (bliss does not support them and would return an
        // incorrect result); self-loops are allowed.
        let mut seen: HashSet<(usize, usize)> = HashSet::with_capacity(m);
        for eid in 0..m {
            let (u, v) = graph.edge(u32::try_from(eid).map_err(|_| {
                IgraphError::InvalidArgument("edge id exceeds u32 range".to_owned())
            })?)?;
            let (ui, vi) = (u as usize, v as usize);
            let key = if directed || ui <= vi {
                (ui, vi)
            } else {
                (vi, ui)
            };
            if !seen.insert(key) {
                return Err(IgraphError::Unsupported(
                    "canonical labeling does not support multigraphs (multiple edges)",
                ));
            }
            out_adj[ui].push(vi);
            mat[ui][vi] = true;
            if directed {
                in_adj[vi].push(ui);
            } else {
                out_adj[vi].push(ui);
                mat[vi][ui] = true;
            }
        }

        for nbrs in &mut out_adj {
            nbrs.sort_unstable();
            nbrs.dedup();
        }
        if directed {
            for nbrs in &mut in_adj {
                nbrs.sort_unstable();
                nbrs.dedup();
            }
        } else {
            in_adj.clone_from(&out_adj);
        }

        Ok(Self {
            n,
            directed,
            out_adj,
            in_adj,
            mat,
        })
    }
}

/// Renumber `keys` into a contiguous, order-preserving cell id per vertex.
/// Vertices are sorted by `keys`; equal keys share a cell, and cells are
/// numbered in ascending key order. Returns `(color, num_cells)`.
fn renumber<K: Ord>(keys: &[K]) -> (Vec<usize>, usize) {
    let n = keys.len();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| keys[a].cmp(&keys[b]));
    let mut color = vec![0usize; n];
    let mut cur = 0usize;
    for idx in 0..n {
        if idx > 0 && keys[order[idx]] != keys[order[idx - 1]] {
            cur += 1;
        }
        color[order[idx]] = cur;
    }
    (color, if n == 0 { 0 } else { cur + 1 })
}

/// Refine `color` to an equitable (stable) ordered partition in place.
fn refine(color: &mut Vec<usize>, adj: &Adj) {
    let n = adj.n;
    if n == 0 {
        return;
    }
    loop {
        let k = color.iter().copied().max().map_or(0, |m| m + 1);
        let width = if adj.directed { 2 * k } else { k };
        // signature key per vertex: (current cell, neighbour-count vector)
        let mut keys: Vec<(usize, Vec<u32>)> = Vec::with_capacity(n);
        for v in 0..n {
            let mut sig = vec![0u32; width];
            for &w in &adj.out_adj[v] {
                sig[color[w]] += 1;
            }
            if adj.directed {
                for &w in &adj.in_adj[v] {
                    sig[k + color[w]] += 1;
                }
            }
            keys.push((color[v], sig));
        }
        let (new_color, new_k) = renumber(&keys);
        let stable = new_k == k;
        *color = new_color;
        if stable {
            break;
        }
    }
}

/// Split `target` cell so that `v` becomes its own singleton placed before
/// the rest of that cell; all other cells keep their order.
fn individualize(color: &[usize], target: usize, v: usize) -> Vec<usize> {
    let keys: Vec<(usize, u8)> = (0..color.len())
        .map(|u| {
            let rank = u8::from(u != v && color[u] == target);
            (color[u], rank)
        })
        .collect();
    renumber(&keys).0
}

/// Lexicographic certificate of the leaf labeling `lab` (position → vertex):
/// the relabeled adjacency matrix packed row-major into bits.
fn certificate(lab: &[usize], adj: &Adj) -> Vec<u64> {
    let n = adj.n;
    let bits = n * n;
    let words = bits.div_ceil(64);
    let mut cert = vec![0u64; words];
    let mut idx = 0usize;
    for i in 0..n {
        let row = &adj.mat[lab[i]];
        for j in 0..n {
            if row[lab[j]] {
                cert[idx / 64] |= 1u64 << (idx % 64);
            }
            idx += 1;
        }
    }
    cert
}

struct Search<'a> {
    adj: &'a Adj,
    best_cert: Option<Vec<u64>>,
    /// The first leaf labeling (position → vertex) achieving `best_cert`.
    canon_lab: Option<Vec<usize>>,
    /// Number of leaves whose certificate equals `best_cert` (= `|Aut(G)|`).
    leaf_count: usize,
    /// Automorphism generators collected on-the-fly.
    generators: Vec<Vec<u32>>,
    /// Union-find parent array for orbit tracking during generator selection.
    uf_parent: Vec<usize>,
}

impl Search<'_> {
    fn uf_find(&mut self, mut v: usize) -> usize {
        let mut root = v;
        while self.uf_parent[root] != root {
            root = self.uf_parent[root];
        }
        while v != root {
            let next = self.uf_parent[v];
            self.uf_parent[v] = root;
            v = next;
        }
        root
    }

    fn uf_union(&mut self, a: usize, b: usize) {
        let ra = self.uf_find(a);
        let rb = self.uf_find(b);
        if ra != rb {
            self.uf_parent[ra] = rb;
        }
    }

    fn recurse(&mut self, mut color: Vec<usize>) {
        refine(&mut color, self.adj);
        let n = self.adj.n;
        let k = color.iter().copied().max().map_or(0, |m| m + 1);

        if k == n {
            // Discrete: position(v) = color[v], so lab[color[v]] = v.
            let mut lab = vec![0usize; n];
            for (v, &pos) in color.iter().enumerate() {
                lab[pos] = v;
            }
            let cert = certificate(&lab, self.adj);
            match &self.best_cert {
                None => {
                    self.best_cert = Some(cert);
                    self.canon_lab = Some(lab);
                    self.leaf_count = 1;
                    self.generators.clear();
                    self.uf_parent = (0..n).collect();
                }
                Some(bc) => match cert.cmp(bc) {
                    std::cmp::Ordering::Greater => {
                        self.best_cert = Some(cert);
                        self.canon_lab = Some(lab);
                        self.leaf_count = 1;
                        self.generators.clear();
                        self.uf_parent = (0..n).collect();
                    }
                    std::cmp::Ordering::Equal => {
                        self.leaf_count += 1;
                        self.record_automorphism(&lab);
                    }
                    std::cmp::Ordering::Less => {}
                },
            }
            return;
        }

        // Target cell: the first (lowest-id) cell of size > 1.
        let mut sizes = vec![0usize; k];
        for &c in &color {
            sizes[c] += 1;
        }
        let target = (0..k).find(|&c| sizes[c] > 1).unwrap_or(0);
        let members: Vec<usize> = (0..n).filter(|&v| color[v] == target).collect();
        for &v in &members {
            let child = individualize(&color, target, v);
            self.recurse(child);
        }
    }

    fn record_automorphism(&mut self, lab: &[usize]) {
        let n = self.adj.n;
        let Some(canon) = &self.canon_lab else {
            return;
        };
        let mut linv = vec![0usize; n];
        for (pos, &v) in lab.iter().enumerate() {
            linv[v] = pos;
        }
        let phi: Vec<u32> = (0..n).map(|v| canon[linv[v]] as u32).collect();

        // Add as generator if it merges previously-separate orbits, or if we
        // have fewer than n generators (O(n) always suffice).
        let merges_orbits = (0..n).any(|v| self.uf_find(v) != self.uf_find(phi[v] as usize));

        if merges_orbits || self.generators.len() < n {
            for (v, &pv) in phi.iter().enumerate() {
                self.uf_union(v, pv as usize);
            }
            self.generators.push(phi);
        }
    }
}

/// Compose `g` after `p`: `(g ∘ p)[v] = g[p[v]]`.
#[cfg(test)]
fn compose(g: &[u32], p: &[u32]) -> Vec<u32> {
    p.iter().map(|&pv| g[pv as usize]).collect()
}

/// All permutations of the group generated by `gens` (closure from identity).
/// Used only in tests to verify generator correctness.
#[cfg(test)]
fn group_closure(gens: &[Vec<u32>], n: usize) -> HashSet<Vec<u32>> {
    let mut set: HashSet<Vec<u32>> = HashSet::new();
    let id: Vec<u32> = (0..n as u32).collect();
    set.insert(id.clone());
    let mut frontier = vec![id];
    while let Some(p) = frontier.pop() {
        for g in gens {
            let q = compose(g, &p);
            if set.insert(q.clone()) {
                frontier.push(q);
            }
        }
    }
    set
}

/// Run the full individualization-refinement search and assemble the
/// canonical labeling, an automorphism generating set, and `|Aut(G)|`.
pub(crate) fn canonicalize(
    graph: &Graph,
    colors: Option<&[u32]>,
) -> IgraphResult<Canonicalization> {
    let adj = Adj::build(graph)?;
    let n = adj.n;

    if let Some(c) = colors {
        if c.len() != n {
            return Err(IgraphError::InvalidArgument(format!(
                "colour vector length {} does not match vertex count {n}",
                c.len()
            )));
        }
    }

    // Initial partition by (colour, self-loop flag).
    let init_keys: Vec<(u32, u8)> = (0..n)
        .map(|v| {
            let col = colors.map_or(0, |c| c[v]);
            (col, u8::from(adj.mat[v][v]))
        })
        .collect();
    let (init_color, _) = renumber(&init_keys);

    let mut search = Search {
        adj: &adj,
        best_cert: None,
        canon_lab: None,
        leaf_count: 0,
        generators: Vec::new(),
        uf_parent: (0..n).collect(),
    };
    search.recurse(init_color);

    // The null graph yields a single empty leaf.
    let Some(canon) = &search.canon_lab else {
        return Ok(Canonicalization {
            labeling: Vec::new(),
            generators: Vec::new(),
            group_order: 1.0,
            certificate: Vec::new(),
        });
    };

    let canon_certificate = search.best_cert.clone().unwrap_or_default();
    // labeling[vertex] = canonical position.
    let mut labeling = vec![0u32; n];
    for (pos, &v) in canon.iter().enumerate() {
        labeling[v] = pos as u32;
    }

    // f64 group order: exact up to 2^53, intentionally lossy beyond (documented).
    #[allow(clippy::cast_precision_loss)]
    let group_order = search.leaf_count as f64;

    Ok(Canonicalization {
        labeling,
        generators: search.generators,
        group_order,
        certificate: canon_certificate,
    })
}

#[cfg(test)]
#[allow(clippy::cast_precision_loss)] // small factorial/orbit counts, exact below 2^52
mod tests {
    use super::*;

    /// Brute-force `|Aut(G)|` by enumerating all `n!` permutations and
    /// counting those that preserve the (coloured) adjacency matrix.
    fn brute_force_aut_count(graph: &Graph, colors: Option<&[u32]>) -> u64 {
        let adj = Adj::build(graph).expect("adj");
        let n = adj.n;
        let mut perm: Vec<usize> = (0..n).collect();
        let mut count = 0u64;
        // Heap's algorithm over all permutations.
        let mut c = vec![0usize; n];
        let is_aut = |p: &[usize]| -> bool {
            for u in 0..n {
                if let Some(cl) = colors {
                    if cl[u] != cl[p[u]] {
                        return false;
                    }
                }
                for v in 0..n {
                    if adj.mat[u][v] != adj.mat[p[u]][p[v]] {
                        return false;
                    }
                }
            }
            true
        };
        if n == 0 {
            return 1;
        }
        if is_aut(&perm) {
            count += 1;
        }
        let mut i = 0;
        while i < n {
            if c[i] < i {
                if i % 2 == 0 {
                    perm.swap(0, i);
                } else {
                    perm.swap(c[i], i);
                }
                if is_aut(&perm) {
                    count += 1;
                }
                c[i] += 1;
                i = 0;
            } else {
                c[i] = 0;
                i += 1;
            }
        }
        count
    }

    fn cycle(n: u32, directed: bool) -> Graph {
        let mut g = Graph::new(n, directed).expect("graph");
        for i in 0..n {
            g.add_edge(i, (i + 1) % n).expect("edge");
        }
        g
    }

    fn complete(n: u32) -> Graph {
        let mut g = Graph::new(n, false).expect("graph");
        for a in 0..n {
            for b in (a + 1)..n {
                g.add_edge(a, b).expect("edge");
            }
        }
        g
    }

    fn path(n: u32) -> Graph {
        let mut g = Graph::new(n, false).expect("graph");
        for i in 0..n.saturating_sub(1) {
            g.add_edge(i, i + 1).expect("edge");
        }
        g
    }

    /// The 10-vertex Petersen graph (outer 5-cycle, inner pentagram, spokes).
    fn petersen() -> Graph {
        let mut g = Graph::new(10, false).expect("graph");
        for i in 0..5 {
            g.add_edge(i, (i + 1) % 5).expect("edge"); // outer cycle
            g.add_edge(i + 5, (i + 2) % 5 + 5).expect("edge"); // inner pentagram
            g.add_edge(i, i + 5).expect("edge"); // spoke
        }
        g
    }

    /// Generators generate a group of exactly `|Aut(G)|` elements.
    fn assert_generators_complete(graph: &Graph, colors: Option<&[u32]>, order: u64) {
        let c = canonicalize(graph, colors).expect("canonicalize");
        let n = graph.vcount() as usize;
        let closure = group_closure(&c.generators, n);
        assert_eq!(
            closure.len() as u64,
            order,
            "generators do not generate the full automorphism group"
        );
        // Every generator is genuinely an automorphism.
        let adj = Adj::build(graph).expect("adj");
        for g in &c.generators {
            for u in 0..n {
                for v in 0..n {
                    assert_eq!(
                        adj.mat[u][v], adj.mat[g[u] as usize][g[v] as usize],
                        "generator is not an automorphism"
                    );
                }
            }
        }
    }

    fn check(graph: &Graph, colors: Option<&[u32]>) {
        let expected = brute_force_aut_count(graph, colors);
        let c = canonicalize(graph, colors).expect("canonicalize");
        assert!(
            (c.group_order - expected as f64).abs() < 0.5,
            "engine |Aut| {} != brute force {}",
            c.group_order,
            expected
        );
        assert_generators_complete(graph, colors, expected);
    }

    #[test]
    fn complete_graphs_have_factorial_automorphisms() {
        // |Aut(K_n)| = n!
        let fact = [1u64, 1, 2, 6, 24, 120, 720];
        for n in 1u32..=6 {
            let c = canonicalize(&complete(n), None).expect("canon");
            assert!((c.group_order - fact[n as usize] as f64).abs() < 0.5);
        }
    }

    #[test]
    fn cycle_graphs_have_dihedral_automorphisms() {
        // |Aut(C_n)| = 2n for n >= 3.
        for n in 3u32..=7 {
            let c = canonicalize(&cycle(n, false), None).expect("canon");
            assert!((c.group_order - f64::from(2 * n)).abs() < 0.5, "C_{n}");
        }
    }

    #[test]
    fn path_graphs_have_two_automorphisms() {
        for n in 2u32..=7 {
            let c = canonicalize(&path(n), None).expect("canon");
            assert!((c.group_order - 2.0).abs() < 0.5, "path_{n}");
        }
        // Single vertex: trivial group.
        let c = canonicalize(&path(1), None).expect("canon");
        assert!((c.group_order - 1.0).abs() < 0.5);
    }

    #[test]
    fn petersen_has_120_automorphisms() {
        let c = canonicalize(&petersen(), None).expect("canon");
        assert!((c.group_order - 120.0).abs() < 0.5);
        assert_generators_complete(&petersen(), None, 120);
    }

    #[test]
    fn directed_cycle_has_n_rotations() {
        // A directed n-cycle has exactly n rotational automorphisms (no reflection).
        for n in 3u32..=7 {
            let c = canonicalize(&cycle(n, true), None).expect("canon");
            assert!((c.group_order - f64::from(n)).abs() < 0.5, "directed C_{n}");
        }
    }

    #[test]
    fn matches_brute_force_battery() {
        // Undirected structured graphs.
        check(&complete(5), None);
        check(&cycle(6, false), None);
        check(&path(6), None);
        check(&petersen(), None);

        // Directed.
        check(&cycle(5, true), None);
        let mut d = Graph::new(4, true).expect("graph");
        for (u, v) in [(0, 1), (1, 2), (2, 3), (3, 0), (0, 2)] {
            d.add_edge(u, v).expect("edge");
        }
        check(&d, None);

        // Coloured: colouring K_4 with two pairs breaks symmetry to 2*2 = 4.
        let k4 = complete(4);
        let colors = [0u32, 0, 1, 1];
        check(&k4, Some(&colors));

        // A graph with a self-loop on one vertex of a triangle: the loop
        // pins that vertex, leaving a single swap of the other two.
        let mut tri_loop = Graph::new(3, false).expect("graph");
        for (u, v) in [(0, 1), (1, 2), (0, 2), (0, 0)] {
            tri_loop.add_edge(u, v).expect("edge");
        }
        check(&tri_loop, None);

        // Disconnected: two disjoint edges -> swap-within plus swap-between = 8.
        let mut two_edges = Graph::new(4, false).expect("graph");
        two_edges.add_edge(0, 1).expect("edge");
        two_edges.add_edge(2, 3).expect("edge");
        check(&two_edges, None);

        // Empty graph on 4 vertices -> S_4 = 24.
        check(&Graph::new(4, false).expect("graph"), None);
    }
}

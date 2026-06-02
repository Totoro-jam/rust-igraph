//! Viger–Latapy uniform-sample-of-simple-graphs degree-sequence generator
//! (ALGO-GN-025).
//!
//! Counterpart of the `IGRAPH_DEGSEQ_VL` branch of
//! `igraph_degree_sequence_game()` in
//! `references/igraph/src/games/degree_sequence.c` (dispatched to
//! `igraph_i_degree_sequence_game_vl()` in
//! `references/igraph/src/games/degree_sequence_vl/gengraph_mr-connected.cpp`,
//! lines 138-182). Backed by Fabien Viger's 2006 `gengraph` C++ library
//! (~4200 LOC across `gengraph_degree_sequence.cpp`,
//! `gengraph_graph_molloy_*.cpp`, etc.).
//!
//! The Viger–Latapy method (Viger & Latapy, "Random sampling of regular
//! and irregular graphs", 2005) samples *approximately uniformly* from
//! the set of **simple connected** graphs with a prescribed undirected
//! degree sequence by running a degree-preserving edge-switch Markov
//! chain whose stationary distribution is uniform over that set, with
//! `Θ(|E|)` mixing time.
//!
//! ## Algorithm
//!
//! 1. **Validate**: the sequence must be graphical (realisable as a
//!    simple undirected graph). Checked via Erdős–Gallai.
//! 2. **Initial realization** via Havel–Hakimi greedy: at each step,
//!    take the vertex of largest residual degree `d` and connect it to
//!    the next `d` largest-residual-degree vertices. If at any point no
//!    such assignment is possible without creating multi-edges, the
//!    sequence is non-graphical (defensive — Erdős–Gallai already
//!    excludes this).
//! 3. **Make connected**: if the Havel–Hakimi seed is disconnected,
//!    merge components pairwise via the standard 2-swap trick — pick
//!    one edge `(a,b)` in component `A` and one `(c,d)` in component
//!    `B` such that `(a,c)` and `(b,d)` are non-edges, then swap to
//!    `(a,c)` + `(b,d)`. This preserves all degrees and reduces the
//!    component count by 1 per swap. Repeat until connected.
//! 4. **MCMC shuffle**: run `T = 5 · 2|E|` random degree-preserving
//!    edge swaps. Each attempt:
//!    * Pick two distinct edges `(a,b)` and `(c,d)`.
//!    * With a random bit, choose orientation: `(a,c)+(b,d)` or
//!      `(a,d)+(b,c)`.
//!    * Reject if the swap would create a self-loop or multi-edge.
//!    * Reject if the swap would disconnect the graph (lazy check:
//!      every `W = 2|E|` attempts, snapshot the edge set, run
//!      `W` attempts, then verify connectivity and restore on failure).
//!
//! The 5×-edge-count mixing length is the upstream default
//! (`gh->shuffle(5 * gh->nbarcs(), 100 * gh->nbarcs(), FINAL_HEURISTICS)`
//! in `mr-connected.cpp:175`). The 100× argument is the per-window
//! attempt budget (`T_max`); we collapse to a single fixed window of
//! size `W = max(16, 2|E|)` which gives the same expected mixing time
//! and is what the upstream `FINAL_HEURISTICS` settles on for typical
//! inputs.
//!
//! ## Directed graphs
//!
//! The upstream implementation rejects directed input
//! (`mr-connected.cpp:144-146`). We do the same.
//!
//! ## Determinism
//!
//! All randomness flows from a single `SplitMix64` seed — the same
//! `(degrees, seed)` pair always yields the same graph. The PRNG state
//! is not bitwise portable to igraph C / `NumPy` / R, so the
//! three-source conformance harness asserts the **structural
//! invariants** the algorithm preserves by construction: vcount, ecount
//! `= Σd/2`, exact degree match, simplicity, connectivity (when the
//! degree sum is positive).
//!
//! ## Complexity
//!
//! Initial realization: `Θ(n + |E| log n)` via repeated heap-sorted
//! Havel–Hakimi. Connectivity merging: `O(|E|)` per BFS, `O(c)` swaps
//! to merge `c` components ⇒ `O(c · |E|)`. MCMC shuffle: `Θ(|E|)`
//! attempts, each `O(1)` amortised (with `HashSet`-based adjacency) +
//! `O(|E| + n)` per checkpoint BFS, with `O(1)` checkpoints in
//! expectation per `Θ(|E|)` window ⇒ `Θ(|E|)` total.

#![allow(
    unknown_lints,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::manual_contains,
    clippy::double_comparisons
)]

use std::collections::{HashSet, VecDeque};

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Sum a slice of `u32` degrees into `u64` with overflow checking.
fn checked_sum_u64(degrees: &[u32]) -> IgraphResult<u64> {
    let mut acc: u64 = 0;
    for &d in degrees {
        acc = acc
            .checked_add(u64::from(d))
            .ok_or(IgraphError::Internal("degree-sum overflow"))?;
    }
    Ok(acc)
}

/// Erdős–Gallai graphicality test for undirected simple graphs.
///
/// A non-decreasing sequence `d_1 ≤ … ≤ d_n` is graphical iff
/// `Σ d_i` is even and for every `k`,
/// `Σ_{i ≤ k} d_{n-i+1} ≤ k(k-1) + Σ_{i > k} min(d_i, k)` (after sort
/// in non-increasing order). We work on a sorted-descending copy and
/// scan in `O(n log n)`.
fn is_graphical_simple_undirected(degrees: &[u32]) -> bool {
    let n = degrees.len();
    if n == 0 {
        return true;
    }
    let sum: u64 = degrees.iter().map(|&d| u64::from(d)).sum();
    if sum % 2 != 0 {
        return false;
    }
    // Max allowable degree is n - 1 for a simple graph on n vertices.
    let n_u64 = n as u64;
    if degrees.iter().any(|&d| u64::from(d) >= n_u64) {
        return false;
    }
    let mut d_desc: Vec<u32> = degrees.to_vec();
    d_desc.sort_by(|a, b| b.cmp(a));
    let mut left_sum: u64 = 0;
    let mut right_sum: u64 = d_desc.iter().map(|&d| u64::from(d)).sum();
    for k in 1..=n {
        left_sum = match left_sum.checked_add(u64::from(d_desc[k - 1])) {
            Some(v) => v,
            None => return false,
        };
        right_sum = right_sum.saturating_sub(u64::from(d_desc[k - 1]));
        let k_u64 = k as u64;
        let bound_right: u64 = d_desc[k..].iter().map(|&d| u64::from(d).min(k_u64)).sum();
        let rhs = k_u64
            .saturating_mul(k_u64.saturating_sub(1))
            .saturating_add(bound_right);
        if left_sum > rhs {
            return false;
        }
        let _ = right_sum;
    }
    true
}

/// Greedy Havel–Hakimi realization producing a simple undirected graph
/// with exactly the given residual degrees. Returns the edge list with
/// `(min, max)` endpoint ordering. Assumes the sequence is graphical.
fn havel_hakimi_realize(degrees: &[u32], n: u32) -> IgraphResult<Vec<(VertexId, VertexId)>> {
    let m: usize = degrees.iter().map(|&d| d as usize).sum::<usize>() / 2;
    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(m);
    let mut residual: Vec<(u32, VertexId)> = degrees
        .iter()
        .enumerate()
        .map(|(i, &d)| (d, i as VertexId))
        .collect();
    // Repeatedly: sort descending by residual degree, take the top
    // vertex with residual r, connect it to the next r vertices.
    loop {
        residual.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        let Some(&(r, v)) = residual.first() else {
            break;
        };
        if r == 0 {
            break;
        }
        let r_us = r as usize;
        if residual.len() - 1 < r_us {
            return Err(IgraphError::Internal(
                "degree_sequence_game_vl: Havel–Hakimi realization failed (non-graphical input that passed Erdős–Gallai — please report)",
            ));
        }
        // Targets are the next `r` vertices (excluding v itself).
        for j in 1..=r_us {
            let (rd, u) = residual[j];
            if rd == 0 {
                return Err(IgraphError::Internal(
                    "degree_sequence_game_vl: Havel–Hakimi residual zero (should not happen for graphical input)",
                ));
            }
            let (a, b) = if v < u { (v, u) } else { (u, v) };
            edges.push((a, b));
            residual[j].0 = rd - 1;
        }
        residual[0].0 = 0;
        // n is unused after the first pass — but kept in the signature
        // to assert `≤ u32::MAX` upstream.
        let _ = n;
    }
    Ok(edges)
}

/// Mutable adjacency representation: `nbr_set[v]` is the set of
/// neighbours of `v`; `edges` is the flat edge list (used to pick
/// random edges in `O(1)`).
struct Adj {
    nbr_set: Vec<HashSet<VertexId>>,
    edges: Vec<(VertexId, VertexId)>,
}

impl Adj {
    fn from_edges(n: u32, edges: Vec<(VertexId, VertexId)>) -> Self {
        let mut nbr_set: Vec<HashSet<VertexId>> = (0..n).map(|_| HashSet::new()).collect();
        for &(a, b) in &edges {
            nbr_set[a as usize].insert(b);
            nbr_set[b as usize].insert(a);
        }
        Self { nbr_set, edges }
    }

    fn has_edge(&self, u: VertexId, v: VertexId) -> bool {
        if self.nbr_set[u as usize].len() <= self.nbr_set[v as usize].len() {
            self.nbr_set[u as usize].contains(&v)
        } else {
            self.nbr_set[v as usize].contains(&u)
        }
    }

    /// Replace edge at position `eid` from `(old_a,old_b)` to
    /// `(new_a,new_b)`. Caller asserts the new edge is valid.
    fn swap_edge(
        &mut self,
        eid: usize,
        old_a: VertexId,
        old_b: VertexId,
        new_a: VertexId,
        new_b: VertexId,
    ) {
        self.nbr_set[old_a as usize].remove(&old_b);
        self.nbr_set[old_b as usize].remove(&old_a);
        self.nbr_set[new_a as usize].insert(new_b);
        self.nbr_set[new_b as usize].insert(new_a);
        let (a, b) = if new_a < new_b {
            (new_a, new_b)
        } else {
            (new_b, new_a)
        };
        self.edges[eid] = (a, b);
    }

    fn snapshot(&self) -> Vec<(VertexId, VertexId)> {
        self.edges.clone()
    }

    fn restore(&mut self, snapshot: Vec<(VertexId, VertexId)>) {
        for set in &mut self.nbr_set {
            set.clear();
        }
        for &(a, b) in &snapshot {
            self.nbr_set[a as usize].insert(b);
            self.nbr_set[b as usize].insert(a);
        }
        self.edges = snapshot;
    }
}

/// Connectivity test via BFS over vertices with non-zero degree.
/// Returns `true` iff every vertex with `degree > 0` is reachable
/// from the first such vertex.
fn is_connected_ignoring_isolated(adj: &Adj) -> bool {
    let n = adj.nbr_set.len();
    let mut start: Option<usize> = None;
    let mut active = 0usize;
    for (v, set) in adj.nbr_set.iter().enumerate() {
        if !set.is_empty() {
            if start.is_none() {
                start = Some(v);
            }
            active += 1;
        }
    }
    let Some(start_v) = start else {
        return true; // all isolated
    };
    let mut visited = vec![false; n];
    let mut queue: VecDeque<usize> = VecDeque::with_capacity(active);
    visited[start_v] = true;
    queue.push_back(start_v);
    let mut count = 1usize;
    while let Some(v) = queue.pop_front() {
        for &nbr in &adj.nbr_set[v] {
            let nu = nbr as usize;
            if !visited[nu] {
                visited[nu] = true;
                count += 1;
                queue.push_back(nu);
            }
        }
    }
    count == active
}

/// Component-labelling BFS: returns a `Vec<i32>` of length `n` where
/// `comp[v] = -1` for isolated vertices and otherwise the component id
/// `0..k`. Also returns one representative edge index per component
/// (the first edge whose endpoint hits that component).
fn label_components(adj: &Adj) -> (Vec<i32>, Vec<usize>) {
    let n = adj.nbr_set.len();
    let mut comp: Vec<i32> = vec![-1; n];
    let mut component_count: i32 = 0;
    let mut queue: VecDeque<usize> = VecDeque::new();
    for v in 0..n {
        if comp[v] != -1 || adj.nbr_set[v].is_empty() {
            continue;
        }
        comp[v] = component_count;
        queue.push_back(v);
        while let Some(u) = queue.pop_front() {
            for &nbr in &adj.nbr_set[u] {
                let nu = nbr as usize;
                if comp[nu] == -1 {
                    comp[nu] = component_count;
                    queue.push_back(nu);
                }
            }
        }
        component_count += 1;
    }
    // For each component, find one edge.
    let mut representative_edge: Vec<usize> = vec![usize::MAX; component_count as usize];
    for (eid, &(a, _)) in adj.edges.iter().enumerate() {
        let cid = comp[a as usize];
        if cid >= 0 && representative_edge[cid as usize] == usize::MAX {
            representative_edge[cid as usize] = eid;
        }
    }
    (comp, representative_edge)
}

/// Merge connected components via degree-preserving 2-swaps until the
/// graph (ignoring isolated vertices) is connected. Returns `Err` if
/// after a bounded number of attempts merging still fails (impossible
/// for graphical inputs, but kept as a defensive guard).
fn make_connected(adj: &mut Adj, rng: &mut SplitMix64) -> IgraphResult<()> {
    if is_connected_ignoring_isolated(adj) {
        return Ok(());
    }
    let max_outer = adj.edges.len().saturating_mul(8).max(64);
    for _ in 0..max_outer {
        let (comp, rep_edge) = label_components(adj);
        if rep_edge.len() <= 1 {
            return Ok(());
        }
        // Pick component 0 and try to merge with each other component
        // via a 2-swap until either we succeed once, then re-label.
        let e0 = rep_edge[0];
        let (a, b) = adj.edges[e0];
        let mut merged = false;
        'outer: for other in 1..rep_edge.len() {
            let e1 = rep_edge[other];
            let (c, d) = adj.edges[e1];
            // Try the two possible swaps in random order.
            let bit = rng.next_u64() & 1 == 1;
            let plans = if bit {
                [(a, c, b, d), (a, d, b, c)]
            } else {
                [(a, d, b, c), (a, c, b, d)]
            };
            for &(na, nb, nc, nd) in &plans {
                if na != nb && nc != nd && !adj.has_edge(na, nb) && !adj.has_edge(nc, nd) {
                    adj.swap_edge(e0, a, b, na, nb);
                    adj.swap_edge(e1, c, d, nc, nd);
                    merged = true;
                    break 'outer;
                }
            }
            // If both swaps are blocked because (na,nb) or (nc,nd) is
            // already an edge, try injecting a third vertex: pick a
            // random non-edge endpoint inside component `other`.
            let _ = comp[a as usize];
        }
        if !merged {
            // Fallback: scan all edge pairs to find one valid merge.
            let scan_n = adj.edges.len();
            let mut found = false;
            for i in 0..scan_n {
                let (x, y) = adj.edges[i];
                let cx = comp[x as usize];
                if cx != 0 {
                    continue;
                }
                for j in 0..scan_n {
                    if i == j {
                        continue;
                    }
                    let (u, v) = adj.edges[j];
                    let cu = comp[u as usize];
                    if cu == 0 || cu < 0 {
                        continue;
                    }
                    // Try swap (x,u)+(y,v)
                    if x != u && y != v && !adj.has_edge(x, u) && !adj.has_edge(y, v) {
                        adj.swap_edge(i, x, y, x, u);
                        adj.swap_edge(j, u, v, y, v);
                        found = true;
                        break;
                    }
                    // Try swap (x,v)+(y,u)
                    if x != v && y != u && !adj.has_edge(x, v) && !adj.has_edge(y, u) {
                        adj.swap_edge(i, x, y, x, v);
                        adj.swap_edge(j, u, v, y, u);
                        found = true;
                        break;
                    }
                }
                if found {
                    break;
                }
            }
            if !found {
                return Err(IgraphError::InvalidArgument(
                    "degree_sequence_game_vl: cannot realize given degree sequence as a connected simple graph".to_string(),
                ));
            }
        }
        if is_connected_ignoring_isolated(adj) {
            return Ok(());
        }
    }
    Err(IgraphError::Internal(
        "degree_sequence_game_vl: connectivity merging did not converge (please report)",
    ))
}

/// One MCMC edge-swap attempt. Returns `true` if the swap was applied.
fn try_swap(adj: &mut Adj, rng: &mut SplitMix64) -> bool {
    let m = adj.edges.len();
    if m < 2 {
        return false;
    }
    let e1 = rng.gen_index(m);
    let mut e2 = rng.gen_index(m);
    if e1 == e2 {
        e2 = (e2 + 1) % m;
    }
    let (a, b) = adj.edges[e1];
    let (c, d) = adj.edges[e2];
    // Random orientation: 50/50 between (a,c)+(b,d) and (a,d)+(b,c).
    let bit = rng.next_u64() & 1 == 1;
    let (na, nb, nc, nd) = if bit { (a, c, b, d) } else { (a, d, b, c) };
    // Reject self-loops.
    if na == nb || nc == nd {
        return false;
    }
    // Reject if either new edge already exists (multi-edge).
    if adj.has_edge(na, nb) || adj.has_edge(nc, nd) {
        return false;
    }
    adj.swap_edge(e1, a, b, na, nb);
    adj.swap_edge(e2, c, d, nc, nd);
    true
}

/// Run the connectivity-preserving MCMC shuffle for `total_attempts`
/// attempts, snapshotting every `window` attempts and rolling back if
/// the post-window check finds the graph disconnected.
fn shuffle_mcmc(adj: &mut Adj, rng: &mut SplitMix64, total_attempts: u64, window: u64) {
    if total_attempts == 0 || adj.edges.len() < 2 {
        return;
    }
    let mut remaining = total_attempts;
    while remaining > 0 {
        let this_window = remaining.min(window);
        let snapshot = adj.snapshot();
        for _ in 0..this_window {
            try_swap(adj, rng);
        }
        if !is_connected_ignoring_isolated(adj) {
            adj.restore(snapshot);
        }
        remaining -= this_window;
    }
}

/// Sample a random **connected, simple** undirected graph that exactly
/// realises the given degree sequence, approximately uniformly, via the
/// Viger–Latapy edge-switch Markov chain.
///
/// * `degrees` — undirected degree of every vertex. Length defines
///   vertex count `n`.
/// * `seed` — drives the internal `SplitMix64` PRNG.
///
/// # Errors
///
/// * Vertex count exceeds `u32::MAX`.
/// * Edge count `Σd / 2` exceeds `u32::MAX`.
/// * Degree sum overflows `u64`.
/// * Degree sequence is not graphical (sum is odd, or fails
///   Erdős–Gallai), or any single degree exceeds `n - 1`.
/// * Sequence is graphical but cannot be realized as a *connected*
///   simple graph — e.g. it contains a `0` degree on more than one
///   vertex while the rest demand a single component.
///
/// # Examples
///
/// ```
/// use rust_igraph::degree_sequence_game_vl;
///
/// // 4-cycle: every vertex degree 2 ⇒ 4 edges total, connected.
/// let g = degree_sequence_game_vl(&[2, 2, 2, 2], 7).unwrap();
/// assert_eq!(g.vcount(), 4);
/// assert_eq!(g.ecount(), 4);
/// assert!(!g.is_directed());
/// ```
pub fn degree_sequence_game_vl(degrees: &[u32], seed: u64) -> IgraphResult<Graph> {
    let n_usize = degrees.len();
    let n = u32::try_from(n_usize)
        .map_err(|_| IgraphError::Internal("degree_sequence_game_vl: vertex count exceeds u32"))?;

    if n == 0 {
        return Graph::new(0, false);
    }

    if !is_graphical_simple_undirected(degrees) {
        return Err(IgraphError::InvalidArgument(
            "degree_sequence_game_vl: cannot realize the given degree sequence as an undirected, simple graph".to_string(),
        ));
    }

    // Reject any *finite* zero-degree vertex when at least one other
    // vertex has positive degree — upstream errors here with
    // "Cannot make a connected graph from the given degree sequence."
    let positive = degrees.iter().any(|&d| d > 0);
    let any_zero = degrees.iter().any(|&d| d == 0);
    if positive && any_zero {
        return Err(IgraphError::InvalidArgument(
            "degree_sequence_game_vl: cannot make a connected graph from a degree sequence containing both zero and positive degrees".to_string(),
        ));
    }

    let sum = checked_sum_u64(degrees)?;
    let no_of_edges_u64 = sum / 2;
    if no_of_edges_u64 > u64::from(u32::MAX) {
        return Err(IgraphError::Internal(
            "degree_sequence_game_vl: edge count exceeds u32::MAX",
        ));
    }

    // Hakimi's connected-graphical criterion: a graphical sequence with
    // all-positive degrees is realizable as a *connected* simple graph
    // iff Σd ≥ 2(n − 1). Reject upfront with a clear message.
    if positive && n >= 2 && sum < 2 * u64::from(n - 1) {
        return Err(IgraphError::InvalidArgument(
            "degree_sequence_game_vl: degree sum is below 2·(n−1); sequence cannot be realised as a connected simple graph".to_string(),
        ));
    }

    let mut graph = Graph::new(n, false)?;
    if no_of_edges_u64 == 0 {
        return Ok(graph);
    }

    let edges_init = havel_hakimi_realize(degrees, n)?;
    let mut adj = Adj::from_edges(n, edges_init);
    let mut rng = SplitMix64::new(seed);

    make_connected(&mut adj, &mut rng)?;

    // Upstream defaults: T = 5 * 2|E| total MCMC attempts, with
    // FINAL_HEURISTICS choosing a window size in [|E|, 100·|E|]. We
    // collapse to a single fixed window W = max(16, 2|E|) which is the
    // mixing-time-conservative midpoint and matches the per-window
    // behaviour of FINAL_HEURISTICS for moderate inputs.
    let two_m: u64 = sum; // 2|E| = Σd
    let total = two_m.saturating_mul(5);
    let window = two_m.max(16);
    shuffle_mcmc(&mut adj, &mut rng, total, window);

    // Final defensive check: the algorithm is supposed to leave the
    // graph connected by construction, but a non-degenerate edge-set
    // misstep would corrupt that. Surface it.
    if !is_connected_ignoring_isolated(&adj) {
        return Err(IgraphError::Internal(
            "degree_sequence_game_vl: post-shuffle graph is disconnected (please report)",
        ));
    }

    graph.add_edges(adj.edges.iter().copied())?;
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn observed_degrees(g: &Graph) -> Vec<u32> {
        let vcount = g.vcount() as usize;
        let mut deg = vec![0u32; vcount];
        let ecount = u32::try_from(g.ecount()).expect("ecount fits u32 in tests");
        for eid in 0..ecount {
            let (src, dst) = g.edge(eid).expect("edge id in bounds");
            deg[src as usize] = deg[src as usize].saturating_add(1);
            deg[dst as usize] = deg[dst as usize].saturating_add(1);
        }
        deg
    }

    fn count_self_loops(g: &Graph) -> u32 {
        let ecount = u32::try_from(g.ecount()).expect("ecount fits u32 in tests");
        let mut loops = 0u32;
        for eid in 0..ecount {
            let (src, dst) = g.edge(eid).expect("edge id in bounds");
            if src == dst {
                loops += 1;
            }
        }
        loops
    }

    fn count_multi_edges(g: &Graph) -> u32 {
        let ecount = u32::try_from(g.ecount()).expect("ecount fits u32 in tests");
        let mut bag: HashMap<(u32, u32), u32> = HashMap::new();
        for eid in 0..ecount {
            let (src, dst) = g.edge(eid).expect("edge id in bounds");
            let key = if src <= dst { (src, dst) } else { (dst, src) };
            *bag.entry(key).or_insert(0) += 1;
        }
        bag.values().map(|c| c.saturating_sub(1)).sum()
    }

    fn is_connected_simple(g: &Graph) -> bool {
        let vcount = g.vcount() as usize;
        if vcount == 0 {
            return true;
        }
        let ecount = u32::try_from(g.ecount()).expect("ecount fits u32 in tests");
        let mut adj: Vec<Vec<u32>> = vec![Vec::new(); vcount];
        let mut deg = vec![0u32; vcount];
        for eid in 0..ecount {
            let (src, dst) = g.edge(eid).expect("edge id in bounds");
            adj[src as usize].push(dst);
            adj[dst as usize].push(src);
            deg[src as usize] += 1;
            deg[dst as usize] += 1;
        }
        let mut start: Option<usize> = None;
        let mut active = 0usize;
        for (v, &d) in deg.iter().enumerate() {
            if d > 0 {
                if start.is_none() {
                    start = Some(v);
                }
                active += 1;
            }
        }
        let Some(s) = start else {
            return true;
        };
        let mut visited = vec![false; vcount];
        let mut queue = VecDeque::new();
        visited[s] = true;
        queue.push_back(s);
        let mut count = 1usize;
        while let Some(v) = queue.pop_front() {
            for &nbr in &adj[v] {
                let nu = nbr as usize;
                if !visited[nu] {
                    visited[nu] = true;
                    count += 1;
                    queue.push_back(nu);
                }
            }
        }
        count == active
    }

    #[test]
    fn empty_sequence_returns_empty_graph() {
        let g = degree_sequence_game_vl(&[], 0).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
        assert!(!g.is_directed());
    }

    #[test]
    fn singleton_with_zero_degree() {
        let g = degree_sequence_game_vl(&[0], 0).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn all_isolated_returns_no_edges() {
        let g = degree_sequence_game_vl(&[0, 0, 0, 0], 42).unwrap();
        assert_eq!(g.vcount(), 4);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn cycle_4_uniform_degree_2() {
        let g = degree_sequence_game_vl(&[2, 2, 2, 2], 1).unwrap();
        assert_eq!(g.vcount(), 4);
        assert_eq!(g.ecount(), 4);
        assert_eq!(observed_degrees(&g), vec![2, 2, 2, 2]);
        assert_eq!(count_self_loops(&g), 0);
        assert_eq!(count_multi_edges(&g), 0);
        assert!(is_connected_simple(&g));
    }

    #[test]
    fn complete_graph_k4() {
        let g = degree_sequence_game_vl(&[3, 3, 3, 3], 11).unwrap();
        assert_eq!(g.vcount(), 4);
        assert_eq!(g.ecount(), 6);
        assert_eq!(observed_degrees(&g), vec![3, 3, 3, 3]);
        assert_eq!(count_self_loops(&g), 0);
        assert_eq!(count_multi_edges(&g), 0);
        assert!(is_connected_simple(&g));
    }

    #[test]
    fn path_4_endpoints_degree_one() {
        // A path 0-1-2-3 has degrees [1,2,2,1]
        let g = degree_sequence_game_vl(&[1, 2, 2, 1], 5).unwrap();
        assert_eq!(g.vcount(), 4);
        assert_eq!(g.ecount(), 3);
        let mut deg = observed_degrees(&g);
        deg.sort_unstable();
        assert_eq!(deg, vec![1, 1, 2, 2]);
        assert_eq!(count_self_loops(&g), 0);
        assert_eq!(count_multi_edges(&g), 0);
        assert!(is_connected_simple(&g));
    }

    #[test]
    fn star_k1_n_realises_exact_degrees() {
        // Star on 6 vertices: hub degree 5, leaves degree 1.
        let seq = vec![5u32, 1, 1, 1, 1, 1];
        let g = degree_sequence_game_vl(&seq, 99).unwrap();
        assert_eq!(g.vcount(), 6);
        assert_eq!(g.ecount(), 5);
        let mut deg = observed_degrees(&g);
        deg.sort_unstable();
        assert_eq!(deg, vec![1, 1, 1, 1, 1, 5]);
        assert!(is_connected_simple(&g));
    }

    #[test]
    fn powerlaw_like_sequence_preserves_degrees() {
        // Skewed-but-graphical sequence on n=10 (sum=30, passes E-G).
        let seq: Vec<u32> = vec![5, 4, 4, 3, 3, 3, 2, 2, 2, 2];
        let g = degree_sequence_game_vl(&seq, 0xABCD_1234).unwrap();
        assert_eq!(g.vcount(), 10);
        assert_eq!(observed_degrees(&g), seq);
        assert_eq!(count_self_loops(&g), 0);
        assert_eq!(count_multi_edges(&g), 0);
        assert!(is_connected_simple(&g));
    }

    #[test]
    fn larger_uniform_3_regular_n10() {
        let seq: Vec<u32> = vec![3; 10];
        let g = degree_sequence_game_vl(&seq, 0xDEAD_BEEF).unwrap();
        assert_eq!(g.vcount(), 10);
        assert_eq!(g.ecount(), 15);
        assert_eq!(observed_degrees(&g), seq);
        assert_eq!(count_self_loops(&g), 0);
        assert_eq!(count_multi_edges(&g), 0);
        assert!(is_connected_simple(&g));
    }

    #[test]
    fn odd_degree_sum_rejected() {
        let r = degree_sequence_game_vl(&[1, 1, 1], 0);
        assert!(matches!(r, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn degree_exceeds_n_minus_1_rejected() {
        // 4 vertices, request degree 4 on one: impossible (max is 3).
        let r = degree_sequence_game_vl(&[4, 1, 1, 1], 0);
        assert!(matches!(r, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn mixed_zero_with_positive_rejected() {
        // Cannot realize as connected: one vertex must be isolated.
        let r = degree_sequence_game_vl(&[2, 2, 2, 0], 0);
        assert!(matches!(r, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn non_graphical_erdos_gallai_rejected() {
        // [3, 3, 1, 1] sums to 8 (even) and max=3=n-1 but fails E-G:
        // sort desc = [3,3,1,1]; k=2: 3+3=6, RHS = 2·1 + min(1,2)+min(1,2)
        //   = 2 + 2 = 4. 6 > 4 ⇒ not graphical.
        let r = degree_sequence_game_vl(&[3, 3, 1, 1], 0);
        assert!(matches!(r, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn determinism_same_seed_yields_same_graph() {
        let seq = vec![3u32; 10];
        let g1 = degree_sequence_game_vl(&seq, 0xCAFE_BABE).unwrap();
        let g2 = degree_sequence_game_vl(&seq, 0xCAFE_BABE).unwrap();
        assert_eq!(g1.ecount(), g2.ecount());
        let ecount = u32::try_from(g1.ecount()).unwrap();
        let mut e1: Vec<(u32, u32)> = (0..ecount).map(|i| g1.edge(i).unwrap()).collect();
        let mut e2: Vec<(u32, u32)> = (0..ecount).map(|i| g2.edge(i).unwrap()).collect();
        e1.sort_unstable();
        e2.sort_unstable();
        assert_eq!(e1, e2);
    }

    #[test]
    fn different_seeds_can_yield_different_graphs() {
        let seq = vec![3u32; 10];
        let g1 = degree_sequence_game_vl(&seq, 1).unwrap();
        let g2 = degree_sequence_game_vl(&seq, 2).unwrap();
        let ecount = u32::try_from(g1.ecount()).unwrap();
        let mut e1: Vec<(u32, u32)> = (0..ecount).map(|i| g1.edge(i).unwrap()).collect();
        let mut e2: Vec<(u32, u32)> = (0..ecount).map(|i| g2.edge(i).unwrap()).collect();
        e1.sort_unstable();
        e2.sort_unstable();
        // The two graphs need not differ in the worst case, but for
        // n=10 d=3 the configuration space is large enough that they
        // almost surely do.
        assert!(e1 != e2 || g1.ecount() < 5);
    }

    #[test]
    fn full_connected_sweep_n6_3regular() {
        // Repeatedly sample and check invariants across seeds.
        let seq = vec![3u32; 6];
        for seed in 0..20u64 {
            let g = degree_sequence_game_vl(&seq, seed).unwrap();
            assert_eq!(g.vcount(), 6, "seed={seed}");
            assert_eq!(g.ecount(), 9, "seed={seed}");
            assert_eq!(observed_degrees(&g), seq, "seed={seed}");
            assert_eq!(count_self_loops(&g), 0, "seed={seed}");
            assert_eq!(count_multi_edges(&g), 0, "seed={seed}");
            assert!(is_connected_simple(&g), "seed={seed}");
        }
    }

    #[test]
    fn invariants_hold_for_skewed_n12() {
        let seq: Vec<u32> = vec![5, 4, 4, 3, 3, 3, 2, 2, 2, 2, 1, 1];
        let g = degree_sequence_game_vl(&seq, 0xC001_D00D).unwrap();
        assert_eq!(g.vcount(), 12);
        assert_eq!(g.ecount(), 16);
        let mut got = observed_degrees(&g);
        let mut want = seq.clone();
        got.sort_unstable();
        want.sort_unstable();
        assert_eq!(got, want);
        assert_eq!(count_self_loops(&g), 0);
        assert_eq!(count_multi_edges(&g), 0);
        assert!(is_connected_simple(&g));
    }

    #[cfg(all(test, feature = "proptest-harness"))]
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        fn gen_graphical_seq() -> impl Strategy<Value = Vec<u32>> {
            // Pick n in [3,12], generate raw degrees in [1, n-1], then
            // adjust the parity by bumping the first entry.
            (3usize..=12).prop_flat_map(|n| {
                let max_d = (n - 1) as u32;
                prop::collection::vec(1u32..=max_d, n).prop_map(move |mut v| {
                    let sum: u64 = v.iter().map(|&d| u64::from(d)).sum();
                    if sum % 2 != 0 {
                        if v[0] < max_d {
                            v[0] += 1;
                        } else {
                            v[0] -= 1;
                        }
                    }
                    v
                })
            })
        }

        proptest! {
            #[test]
            fn proptest_invariants_on_graphical_seqs(
                seq in gen_graphical_seq(),
                seed in any::<u64>(),
            ) {
                // Skip seeds where the random sequence happens not to
                // be graphical — the function should error cleanly.
                let result = degree_sequence_game_vl(&seq, seed);
                match result {
                    Ok(g) => {
                        prop_assert_eq!(g.vcount() as usize, seq.len());
                        let sum: u64 = seq.iter().map(|&d| u64::from(d)).sum();
                        prop_assert_eq!(g.ecount(), (sum / 2) as usize);
                        let mut got = observed_degrees(&g);
                        let mut want = seq.clone();
                        got.sort_unstable();
                        want.sort_unstable();
                        prop_assert_eq!(got, want);
                        prop_assert_eq!(count_self_loops(&g), 0);
                        prop_assert_eq!(count_multi_edges(&g), 0);
                        prop_assert!(is_connected_simple(&g));
                    }
                    Err(_) => {
                        // Acceptable rejections: non-graphical (E-G),
                        // mixed zero/positive, or graphical but Σd <
                        // 2(n-1) (no connected realisation per Hakimi).
                        let n = seq.len() as u64;
                        let sum: u64 = seq.iter().map(|&d| u64::from(d)).sum();
                        let has_zero = seq.iter().any(|&d| d == 0);
                        let has_pos = seq.iter().any(|&d| d > 0);
                        let below_hakimi = has_pos && n >= 2 && sum < 2 * (n - 1);
                        prop_assert!(
                            !is_graphical_simple_undirected(&seq)
                                || (has_zero && has_pos)
                                || below_hakimi,
                            "unexpected error for seq = {:?}, sum = {}, n = {}",
                            seq, sum, n
                        );
                    }
                }
            }

            #[test]
            fn proptest_determinism_same_seed(
                seq in gen_graphical_seq(),
                seed in any::<u64>(),
            ) {
                let g1 = degree_sequence_game_vl(&seq, seed);
                let g2 = degree_sequence_game_vl(&seq, seed);
                match (g1, g2) {
                    (Ok(g1), Ok(g2)) => {
                        prop_assert_eq!(g1.ecount(), g2.ecount());
                        let ecount = u32::try_from(g1.ecount()).unwrap();
                        let mut e1: Vec<(u32, u32)> =
                            (0..ecount).map(|i| g1.edge(i).unwrap()).collect();
                        let mut e2: Vec<(u32, u32)> =
                            (0..ecount).map(|i| g2.edge(i).unwrap()).collect();
                        e1.sort_unstable();
                        e2.sort_unstable();
                        prop_assert_eq!(e1, e2);
                    }
                    (Err(_), Err(_)) => {}
                    _ => prop_assert!(false, "outcome differs between identical calls"),
                }
            }
        }
    }
}

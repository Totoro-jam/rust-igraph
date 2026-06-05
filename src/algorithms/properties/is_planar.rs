//! Planarity testing via the Left-Right Planarity Test.
//!
//! Implements Ulrik Brandes' simplified version (2009) of the
//! de Fraysseix–Rosenstiehl LR planarity algorithm. Runs in O(V + E).
//!
//! A graph is planar iff it can be embedded in the plane without edge
//! crossings. Equivalently (Kuratowski), a graph is planar iff it has
//! no `K_5` or `K_{3,3}` subdivision.

use crate::core::{Graph, IgraphResult};

/// Test whether a graph is planar.
///
/// Uses the Left-Right Planarity Test (Brandes 2009) which runs in
/// O(V + E) time. Directed graphs are tested as if undirected.
/// Self-loops and multi-edges are handled (self-loops are ignored;
/// multi-edges contribute to the edge count for Euler's criterion).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_planar};
///
/// // K_4 is planar
/// let mut g = Graph::with_vertices(4);
/// for i in 0..4u32 {
///     for j in (i+1)..4 {
///         g.add_edge(i, j).unwrap();
///     }
/// }
/// assert!(is_planar(&g).unwrap());
///
/// // K_5 is NOT planar
/// let mut g = Graph::with_vertices(5);
/// for i in 0..5u32 {
///     for j in (i+1)..5 {
///         g.add_edge(i, j).unwrap();
///     }
/// }
/// assert!(!is_planar(&g).unwrap());
/// ```
pub fn is_planar(graph: &Graph) -> IgraphResult<bool> {
    let n = graph.vcount() as usize;

    if n <= 4 {
        return Ok(true);
    }

    // Build undirected adjacency (deduplicated, no self-loops)
    let (adj, simple_edge_count) = build_simple_undirected_adj(graph);

    // Euler's formula: planar simple graphs have m <= 3n - 6
    if simple_edge_count > 3 * n - 6 {
        return Ok(false);
    }

    // Test each connected component
    let mut visited = vec![false; n];
    for start in 0..n {
        if visited[start] {
            continue;
        }
        if !test_component_planar(&adj, start, &mut visited) {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Build undirected adjacency list (simple: no self-loops, no duplicates).
/// Returns `(adj_list, simple_edge_count)`.
fn build_simple_undirected_adj(graph: &Graph) -> (Vec<Vec<usize>>, usize) {
    let n = graph.vcount() as usize;
    let ecount = graph.ecount();
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];

    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        if let Ok((u, v)) = graph.edge(eid as u32) {
            let u = u as usize;
            let v = v as usize;
            if u != v {
                adj[u].push(v);
                adj[v].push(u);
            }
        }
    }

    // Deduplicate each adjacency list
    for list in &mut adj {
        list.sort_unstable();
        list.dedup();
    }

    let edge_count = adj.iter().map(Vec::len).sum::<usize>() / 2;
    (adj, edge_count)
}

/// Sentinel for "no edge".
const NONE: u32 = u32::MAX;

/// An interval [low, high] of back edges on one side.
/// `low` and `high` are edge indices into `oriented_edges`.
/// `NONE` means the interval is empty.
#[derive(Clone, Copy)]
struct Interval {
    low: u32,
    high: u32,
}

impl Interval {
    fn empty_interval() -> Self {
        Self {
            low: NONE,
            high: NONE,
        }
    }

    fn new(low: u32, high: u32) -> Self {
        Self { low, high }
    }

    fn is_empty(self) -> bool {
        self.low == NONE && self.high == NONE
    }

    fn conflicting(self, b: u32, lowpt: &[u32]) -> bool {
        !self.is_empty() && lowpt[self.high as usize] > lowpt[b as usize]
    }
}

/// A conflict pair: two intervals forced to opposite sides.
#[derive(Clone, Copy)]
struct ConflictPair {
    left: Interval,
    right: Interval,
}

impl ConflictPair {
    fn new(left: Interval, right: Interval) -> Self {
        Self { left, right }
    }

    fn swap(&mut self) {
        std::mem::swap(&mut self.left, &mut self.right);
    }

    fn lowest(&self, lowpt: &[u32]) -> u32 {
        if self.left.is_empty() {
            return lowpt[self.right.low as usize];
        }
        if self.right.is_empty() {
            return lowpt[self.left.low as usize];
        }
        std::cmp::min(
            lowpt[self.left.low as usize],
            lowpt[self.right.low as usize],
        )
    }
}

/// State for the LR planarity test on a single component.
struct LrState {
    oriented_edges: Vec<(usize, usize, bool)>,
    out_edges: Vec<Vec<u32>>,
    height: Vec<i32>,
    parent_edge: Vec<u32>,
    lowpt: Vec<u32>,
    lowpt2: Vec<u32>,
}

/// Test planarity of one connected component using the full Brandes LR algorithm.
#[allow(clippy::too_many_lines)]
fn test_component_planar(adj: &[Vec<usize>], start: usize, visited: &mut [bool]) -> bool {
    let n = adj.len();
    let mut state = orient_and_compute_lowpoints(adj, start, visited, n);

    let num_edges = state.oriented_edges.len();
    let comp_size = state.oriented_edges.iter().filter(|e| e.2).count() + 1;
    if comp_size <= 4 {
        return true;
    }
    if num_edges > 3 * comp_size - 6 {
        return false;
    }

    // Compute nesting depth and sort adjacency lists
    let ordered_adjs = compute_nesting_and_sort(&state, n);

    // Phase 2: LR testing
    lr_testing(&mut state, &ordered_adjs, n)
}

/// Phase 1: DFS orientation + lowpoint computation.
#[allow(clippy::too_many_lines)]
fn orient_and_compute_lowpoints(
    adj: &[Vec<usize>],
    start: usize,
    visited: &mut [bool],
    n: usize,
) -> LrState {
    let mut oriented_edges: Vec<(usize, usize, bool)> = Vec::new();
    let mut out_edges: Vec<Vec<u32>> = vec![Vec::new(); n];
    let mut height: Vec<i32> = vec![-1; n];
    let mut parent_edge: Vec<u32> = vec![NONE; n];

    // Phase 1a: DFS orientation
    {
        let mut oriented_set: std::collections::HashSet<(usize, usize)> =
            std::collections::HashSet::new();
        let mut stack: Vec<(usize, usize)> = Vec::new();
        height[start] = 0;
        visited[start] = true;
        stack.push((start, 0));

        while let Some(&mut (v, ref mut idx)) = stack.last_mut() {
            if *idx < adj[v].len() {
                let w = adj[v][*idx];
                *idx += 1;
                let key = if v < w { (v, w) } else { (w, v) };
                if oriented_set.contains(&key) {
                    continue;
                }
                oriented_set.insert(key);
                #[allow(clippy::cast_possible_truncation)]
                let eidx = oriented_edges.len() as u32;
                out_edges[v].push(eidx);
                if height[w] == -1 {
                    oriented_edges.push((v, w, true));
                    parent_edge[w] = eidx;
                    height[w] = height[v] + 1;
                    visited[w] = true;
                    stack.push((w, 0));
                } else {
                    oriented_edges.push((v, w, false));
                }
            } else {
                stack.pop();
            }
        }
    }

    let num_edges = oriented_edges.len();

    // Phase 1b: Compute lowpoints bottom-up
    #[allow(clippy::cast_sign_loss)]
    let mut lowpt: Vec<u32> = vec![0; num_edges];
    let mut lowpt2: Vec<u32> = vec![0; num_edges];

    for eidx in 0..num_edges {
        let (source, target, is_tree) = oriented_edges[eidx];
        #[allow(clippy::cast_sign_loss)]
        if is_tree {
            lowpt[eidx] = height[source] as u32;
            lowpt2[eidx] = height[source] as u32;
        } else {
            lowpt[eidx] = height[target] as u32;
            lowpt2[eidx] = height[source] as u32;
        }
    }

    // Post-order traversal
    let comp_size = oriented_edges.iter().filter(|e| e.2).count() + 1;
    let mut post_order: Vec<usize> = Vec::with_capacity(comp_size);
    {
        let mut stack: Vec<(usize, usize)> = vec![(start, 0)];
        while let Some(&mut (v, ref mut idx)) = stack.last_mut() {
            if *idx < out_edges[v].len() {
                let eidx = out_edges[v][*idx] as usize;
                *idx += 1;
                let (_, w, is_tree) = oriented_edges[eidx];
                if is_tree {
                    stack.push((w, 0));
                }
            } else {
                post_order.push(v);
                stack.pop();
            }
        }
    }

    // Propagate lowpoints bottom-up
    for &v in &post_order {
        let e = parent_edge[v];
        if e == NONE {
            continue;
        }
        let e_idx = e as usize;
        for &child_eidx in &out_edges[v] {
            let cidx = child_eidx as usize;
            let clp = lowpt[cidx];
            let clp2 = lowpt2[cidx];
            match clp.cmp(&lowpt[e_idx]) {
                std::cmp::Ordering::Less => {
                    lowpt2[e_idx] = std::cmp::min(lowpt[e_idx], clp2);
                    lowpt[e_idx] = clp;
                }
                std::cmp::Ordering::Greater => {
                    lowpt2[e_idx] = std::cmp::min(lowpt2[e_idx], clp);
                }
                std::cmp::Ordering::Equal => {
                    lowpt2[e_idx] = std::cmp::min(lowpt2[e_idx], clp2);
                }
            }
        }
    }

    LrState {
        oriented_edges,
        out_edges,
        height,
        parent_edge,
        lowpt,
        lowpt2,
    }
}

/// Compute nesting depth and sort adjacency lists by nesting depth.
fn compute_nesting_and_sort(state: &LrState, n: usize) -> Vec<Vec<u32>> {
    let num_edges = state.oriented_edges.len();
    let mut nesting_depth: Vec<i32> = vec![0; num_edges];

    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    for (eidx, nd) in nesting_depth.iter_mut().enumerate().take(num_edges) {
        let (source, _, _) = state.oriented_edges[eidx];
        *nd = 2 * (state.lowpt[eidx] as i32);
        if state.lowpt2[eidx] < state.height[source] as u32 {
            *nd += 1;
        }
    }

    let mut ordered_adjs: Vec<Vec<u32>> = vec![Vec::new(); n];
    for (v, slot) in ordered_adjs.iter_mut().enumerate().take(n) {
        if state.height[v] == -1 {
            continue;
        }
        let mut edges: Vec<u32> = state.out_edges[v].clone();
        edges.sort_by_key(|&e| nesting_depth[e as usize]);
        *slot = edges;
    }
    ordered_adjs
}

/// Phase 2: LR constraint testing.
fn lr_testing(state: &mut LrState, ordered_adjs: &[Vec<u32>], n: usize) -> bool {
    let num_edges = state.oriented_edges.len();
    let mut ref_edge: Vec<u32> = vec![NONE; num_edges];
    let mut stack: Vec<ConflictPair> = Vec::new();
    let mut stack_bottom: Vec<usize> = vec![0; num_edges];
    let mut lowpt_edge: Vec<u32> = vec![NONE; num_edges];

    let mut dfs_stack: Vec<usize> = Vec::new();
    // Find start vertex (first with height == 0)
    for v in 0..n {
        if state.height[v] == 0 {
            dfs_stack.push(v);
            break;
        }
    }

    let mut ind: Vec<usize> = vec![0; n];
    let mut skip_init_edge: Vec<bool> = vec![false; num_edges];

    while let Some(&v) = dfs_stack.last() {
        let e = state.parent_edge[v];
        let mut skip_final = false;

        while ind[v] < ordered_adjs[v].len() {
            let ei = ordered_adjs[v][ind[v]];
            let ei_usize = ei as usize;

            if !skip_init_edge[ei_usize] {
                stack_bottom[ei_usize] = stack.len();
                let (_, w, is_tree) = state.oriented_edges[ei_usize];

                if is_tree {
                    skip_init_edge[ei_usize] = true;
                    dfs_stack.push(w);
                    skip_final = true;
                    break;
                }
                lowpt_edge[ei_usize] = ei;
                stack.push(ConflictPair::new(
                    Interval::empty_interval(),
                    Interval::new(ei, ei),
                ));
            }

            // Integrate new return edges
            #[allow(clippy::cast_sign_loss)]
            if state.lowpt[ei_usize] < state.height[v] as u32 {
                if ind[v] == 0 {
                    if e != NONE {
                        lowpt_edge[e as usize] = lowpt_edge[ei_usize];
                    }
                } else if e != NONE
                    && !add_constraints(
                        ei,
                        e,
                        &state.lowpt,
                        &mut ref_edge,
                        &mut stack,
                        &stack_bottom,
                        &lowpt_edge,
                    )
                {
                    return false;
                }
            }

            ind[v] += 1;
        }

        if !skip_final {
            dfs_stack.pop();
            if e != NONE {
                remove_back_edges(
                    e,
                    &state.oriented_edges,
                    &state.height,
                    &state.lowpt,
                    &mut ref_edge,
                    &mut stack,
                );
            }
        }
    }

    true
}

/// Add constraints for edge `ei` against parent tree edge `e`.
/// Returns false if the graph is non-planar.
fn add_constraints(
    ei: u32,
    e: u32,
    lowpt: &[u32],
    ref_edge: &mut [u32],
    stack: &mut Vec<ConflictPair>,
    stack_bottom: &[usize],
    lowpt_edge: &[u32],
) -> bool {
    let mut p = ConflictPair::new(Interval::empty_interval(), Interval::empty_interval());

    // Merge return edges of e_i into P.right
    loop {
        let Some(mut q) = stack.pop() else {
            return false;
        };
        if !q.left.is_empty() {
            q.swap();
        }
        if !q.left.is_empty() {
            return false;
        }
        if lowpt[q.right.low as usize] > lowpt[e as usize] {
            if p.right.is_empty() {
                p.right = q.right;
            } else {
                ref_edge[p.right.low as usize] = q.right.high;
                p.right.low = q.right.low;
            }
        } else {
            ref_edge[q.right.low as usize] = lowpt_edge[e as usize];
        }
        if stack.len() == stack_bottom[ei as usize] {
            break;
        }
    }

    // Merge conflicting return edges of e_1,...,e_{i-1} into P.left
    while let Some(top) = stack.last() {
        if !top.left.conflicting(ei, lowpt) && !top.right.conflicting(ei, lowpt) {
            break;
        }
        let mut q = stack.pop().unwrap();
        if q.right.conflicting(ei, lowpt) {
            q.swap();
        }
        if q.right.conflicting(ei, lowpt) {
            return false;
        }
        // Merge Q.right interval into P.right
        if p.right.low != NONE {
            ref_edge[p.right.low as usize] = q.right.high;
        }
        if q.right.low != NONE {
            p.right.low = q.right.low;
        }
        if p.right.is_empty() && q.right.low != NONE {
            p.right = Interval::new(q.right.low, q.right.high);
        }

        // Merge Q.left into P.left
        if p.left.is_empty() {
            p.left = q.left;
        } else {
            ref_edge[p.left.low as usize] = q.left.high;
            p.left.low = q.left.low;
        }
    }

    if !p.left.is_empty() || !p.right.is_empty() {
        stack.push(p);
    }
    true
}

/// Remove back edges returning to the parent of tree edge `e`.
fn remove_back_edges(
    e: u32,
    oriented_edges: &[(usize, usize, bool)],
    height: &[i32],
    lowpt: &[u32],
    ref_edge: &mut [u32],
    stack: &mut Vec<ConflictPair>,
) {
    let u = oriented_edges[e as usize].0;
    #[allow(clippy::cast_sign_loss)]
    let h_u = height[u] as u32;

    // Trim: drop entire conflict pairs whose lowest return == height[u]
    while let Some(top) = stack.last() {
        if top.lowest(lowpt) != h_u {
            break;
        }
        stack.pop();
    }

    if let Some(p) = stack.last_mut() {
        // Trim left interval: advance high through ref chain
        while p.left.high != NONE {
            let h_edge = p.left.high as usize;
            let (_, target, _) = oriented_edges[h_edge];
            if target != u {
                break;
            }
            p.left.high = ref_edge[h_edge];
        }
        if p.left.high == NONE && p.left.low != NONE {
            ref_edge[p.left.low as usize] = p.right.low;
            p.left.low = NONE;
        }

        // Trim right interval
        while p.right.high != NONE {
            let h_edge = p.right.high as usize;
            let (_, target, _) = oriented_edges[h_edge];
            if target != u {
                break;
            }
            p.right.high = ref_edge[h_edge];
        }
        if p.right.high == NONE && p.right.low != NONE {
            ref_edge[p.right.low as usize] = p.left.low;
            p.right.low = NONE;
        }

        // If both intervals are now empty, remove the pair
        if p.left.is_empty() && p.right.is_empty() {
            stack.pop();
        }
    }

    // Set ref for tree edge e
    if lowpt[e as usize] < h_u {
        if let Some(top) = stack.last() {
            let hl = top.left.high;
            let hr = top.right.high;
            if hl != NONE && (hr == NONE || lowpt[hl as usize] > lowpt[hr as usize]) {
                ref_edge[e as usize] = hl;
            } else if hr != NONE {
                ref_edge[e as usize] = hr;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_planar(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_planar(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_planar(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_planar(&g).unwrap());
    }

    #[test]
    fn k4_planar() {
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_planar(&g).unwrap());
    }

    #[test]
    fn k5_not_planar() {
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(!is_planar(&g).unwrap());
    }

    #[test]
    fn k33_not_planar() {
        let mut g = Graph::with_vertices(6);
        for i in 0..3u32 {
            for j in 3..6u32 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(!is_planar(&g).unwrap());
    }

    #[test]
    fn petersen_not_planar() {
        let mut g = Graph::with_vertices(10);
        // Outer cycle
        for i in 0..5u32 {
            g.add_edge(i, (i + 1) % 5).unwrap();
        }
        // Inner pentagram
        for i in 0..5u32 {
            g.add_edge(5 + i, 5 + (i + 2) % 5).unwrap();
        }
        // Spokes
        for i in 0..5u32 {
            g.add_edge(i, 5 + i).unwrap();
        }
        assert!(!is_planar(&g).unwrap());
    }

    #[test]
    fn tree_planar() {
        let mut g = Graph::with_vertices(10);
        for i in 1..10u32 {
            g.add_edge(i, i / 2).unwrap();
        }
        assert!(is_planar(&g).unwrap());
    }

    #[test]
    fn cycle_planar() {
        let mut g = Graph::with_vertices(20);
        for i in 0..20u32 {
            g.add_edge(i, (i + 1) % 20).unwrap();
        }
        assert!(is_planar(&g).unwrap());
    }

    #[test]
    fn grid_4x4_planar() {
        let mut g = Graph::with_vertices(16);
        for r in 0..4u32 {
            for c in 0..4u32 {
                let v = r * 4 + c;
                if c + 1 < 4 {
                    g.add_edge(v, v + 1).unwrap();
                }
                if r + 1 < 4 {
                    g.add_edge(v, v + 4).unwrap();
                }
            }
        }
        assert!(is_planar(&g).unwrap());
    }

    #[test]
    fn self_loops_ignored() {
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        g.add_edge(0, 0).unwrap();
        assert!(!is_planar(&g).unwrap());
    }

    #[test]
    fn disconnected_planar() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 3).unwrap();
        assert!(is_planar(&g).unwrap());
    }

    #[test]
    fn disconnected_non_planar() {
        let mut g = Graph::with_vertices(6);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(!is_planar(&g).unwrap());
    }

    #[test]
    fn wheel_5_planar() {
        let mut g = Graph::with_vertices(6);
        for i in 1..6u32 {
            g.add_edge(0, i).unwrap();
        }
        for i in 1..5u32 {
            g.add_edge(i, i + 1).unwrap();
        }
        g.add_edge(5, 1).unwrap();
        assert!(is_planar(&g).unwrap());
    }

    #[test]
    fn k5_minus_edge_planar() {
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                if !(i == 0 && j == 4) {
                    g.add_edge(i, j).unwrap();
                }
            }
        }
        assert!(is_planar(&g).unwrap());
    }

    #[test]
    fn k33_minus_edge_planar() {
        let mut g = Graph::with_vertices(6);
        for i in 0..3u32 {
            for j in 3..6u32 {
                if !(i == 0 && j == 3) {
                    g.add_edge(i, j).unwrap();
                }
            }
        }
        assert!(is_planar(&g).unwrap());
    }

    #[test]
    fn icosahedron_planar() {
        let mut g = Graph::with_vertices(12);
        let edges: &[(u32, u32)] = &[
            (0, 1),
            (0, 2),
            (0, 3),
            (0, 4),
            (0, 5),
            (1, 2),
            (2, 3),
            (3, 4),
            (4, 5),
            (5, 1),
            (1, 6),
            (2, 6),
            (2, 7),
            (3, 7),
            (3, 8),
            (4, 8),
            (4, 9),
            (5, 9),
            (5, 10),
            (1, 10),
            (6, 7),
            (7, 8),
            (8, 9),
            (9, 10),
            (10, 6),
            (6, 11),
            (7, 11),
            (8, 11),
            (9, 11),
            (10, 11),
        ];
        for &(u, v) in edges {
            g.add_edge(u, v).unwrap();
        }
        assert!(is_planar(&g).unwrap());
    }
}

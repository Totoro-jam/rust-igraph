//! Prüfer-sequence tree decoder (ALGO-CN-016).
//!
//! Counterpart of `igraph_from_prufer()` in
//! `references/igraph/src/constructors/prufer.c:55-125`.
//!
//! A *Prüfer sequence* is a length-`(n-2)` string over the alphabet
//! `{0, 1, …, n-1}` that uniquely encodes a labelled tree on `n`
//! vertices. The bijection works in both directions:
//!
//! * **Encoding** (not provided here — see [`igraph_to_prufer`] upstream):
//!   repeatedly remove the smallest-labelled leaf and record its sole
//!   neighbour's label; stop with two vertices left.
//! * **Decoding** (this module): walk the sequence left-to-right, peel
//!   off the smallest leaf at each step, and emit the implied edges.
//!
//! Implementation follows Micikevičius, Caminiti & Deo, *"Linear-time
//! Algorithms for Encoding Trees as Sequences of Node Labels"*, which
//! achieves `O(n)` instead of the naïve `O(n²)` priority-queue version.
//! The trick is to track each vertex's residual degree (how many times
//! it still appears in the unread tail of the Prüfer sequence) and
//! cascade through leaves discovered as a side-effect of removing one.
//!
//! The decoder always produces an **undirected** tree on `n` vertices
//! with exactly `n - 1` edges. The empty Prüfer sequence yields the
//! 2-vertex path graph `P_2` (single edge `0—1`).
//!
//! Time complexity: `O(|V|)` where `|V| = prufer.len() + 2`.
//!
//! [`igraph_to_prufer`]: https://igraph.org/c/html/latest/igraph-Generators.html#igraph_to_prufer

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Decode a Prüfer sequence into the unique labelled tree it represents.
///
/// `prufer` must contain values in `[0, n)` where `n = prufer.len() + 2`.
/// The resulting graph has exactly `n` vertices and `n - 1` undirected
/// edges, forming a tree. The empty input slice produces the 2-vertex
/// path graph `P_2`.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — some `prufer[i]` is `>= n`,
///   or `prufer.len() + 2` overflows [`u32`].
///
/// # Examples
///
/// ```
/// use rust_igraph::from_prufer;
///
/// // Prüfer sequence [2, 3, 2, 3] encodes a 6-vertex tree.
/// let tree = from_prufer(&[2, 3, 2, 3]).unwrap();
/// assert_eq!(tree.vcount(), 6);
/// assert_eq!(tree.ecount(), 5);
/// assert!(!tree.is_directed());
/// ```
pub fn from_prufer(prufer: &[u32]) -> IgraphResult<Graph> {
    let len = prufer.len();
    let n = u32::try_from(len)
        .ok()
        .and_then(|l| l.checked_add(2))
        .ok_or_else(|| {
            IgraphError::InvalidArgument("from_prufer: prufer.len() + 2 overflows u32".to_string())
        })?;

    // n is at least 2 here (len >= 0 → n >= 2). Always emit one edge.
    let mut degree: Vec<u32> = vec![0; n as usize];
    for &w in prufer {
        if w >= n {
            return Err(IgraphError::InvalidArgument(format!(
                "from_prufer: invalid Prufer entry {w} (must be < {n})",
            )));
        }
        degree[w as usize] += 1;
    }

    let edge_count = (n - 1) as usize;
    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(edge_count);

    // Linear-time decode (Micikevičius–Caminiti–Deo). The loop walks
    // each vertex i once; whenever the current candidate u is already a
    // leaf (degree 0 in the unread tail), we emit its edge to the parent
    // v read from `prufer[k]`, decrement v's degree, and try again at v.
    let mut v: u32 = 0;
    let mut k: usize = 0;
    let mut last_i: u32 = 0;
    let limit = len; // == n as usize - 2
    for i in 0..n {
        last_i = i;
        let mut u = i;
        while k < limit && u <= i && degree[u as usize] == 0 {
            v = prufer[k];
            edges.push((v, u));
            k += 1;
            degree[v as usize] -= 1;
            u = v;
        }
        if k == limit {
            break;
        }
    }

    // Find the lone remaining leaf u > last_i, u != v.
    let mut last_u: u32 = 0;
    for cand in (last_i + 1)..n {
        if degree[cand as usize] == 0 && cand != v {
            last_u = cand;
            break;
        }
    }
    edges.push((v, last_u));

    let mut g = Graph::new(n, false)?;
    g.add_edges(edges)?;
    Ok(g)
}

/// Encode a labelled tree into its unique Prüfer sequence.
///
/// The graph must be an undirected tree with at least 2 vertices.
/// Returns a `Vec<u32>` of length `n - 2` where `n = graph.vcount()`.
///
/// The algorithm iteratively removes the smallest-labelled leaf and
/// records its sole neighbour's label. Runs in `O(n)` time using the
/// linear-time variant that avoids repeated linear scans.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — graph is not a tree, or has
///   fewer than 2 vertices.
///
/// # Examples
///
/// ```
/// use rust_igraph::{from_prufer, to_prufer};
///
/// // Round-trip: encode a known tree, decode it, verify same edges.
/// let tree = from_prufer(&[2, 3, 2, 3]).unwrap();
/// let seq = to_prufer(&tree).unwrap();
/// assert_eq!(seq, vec![2, 3, 2, 3]);
/// ```
pub fn to_prufer(graph: &Graph) -> IgraphResult<Vec<u32>> {
    let n = graph.vcount();

    if n < 2 {
        return Err(IgraphError::InvalidArgument(
            "to_prufer: tree must have at least 2 vertices".into(),
        ));
    }

    // Verify it's a tree: connected + ecount == n - 1
    let ecount = graph.ecount();
    if ecount != (n as usize) - 1 {
        return Err(IgraphError::InvalidArgument(
            "to_prufer: graph is not a tree (wrong edge count)".into(),
        ));
    }

    // Build adjacency lists
    let mut adj: Vec<Vec<VertexId>> = vec![Vec::new(); n as usize];
    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (from, to) = graph.edge(eid as u32)?;
        if from == to {
            return Err(IgraphError::InvalidArgument(
                "to_prufer: graph contains a self-loop, not a tree".into(),
            ));
        }
        adj[from as usize].push(to);
        adj[to as usize].push(from);
    }

    // Compute degrees (safe: degree bounded by n which is u32)
    #[allow(clippy::cast_possible_truncation)]
    let mut degrees: Vec<u32> = adj.iter().map(|v| v.len() as u32).collect();

    // Check connectivity via BFS
    let mut bfs_visited = vec![false; n as usize];
    let mut bfs_queue = std::collections::VecDeque::new();
    bfs_queue.push_back(0u32);
    bfs_visited[0] = true;
    let mut visit_count: u32 = 1;
    while let Some(v) = bfs_queue.pop_front() {
        for &nbr in &adj[v as usize] {
            if !bfs_visited[nbr as usize] {
                bfs_visited[nbr as usize] = true;
                visit_count += 1;
                bfs_queue.push_back(nbr);
            }
        }
    }
    if visit_count != n {
        return Err(IgraphError::InvalidArgument(
            "to_prufer: graph is not connected, not a tree".into(),
        ));
    }

    // Linear-time Prüfer encoding
    let mut prufer: Vec<u32> = Vec::with_capacity((n - 2) as usize);
    let mut prufer_idx: usize = 0;
    let target_len = (n - 2) as usize;

    for u in 0..n {
        let mut leaf = u;
        let mut deg = degrees[leaf as usize];

        while deg == 1 && leaf <= u && prufer_idx < target_len {
            degrees[leaf as usize] = 0;

            // Find the unique remaining neighbor
            let mut neighbor = 0u32;
            for &nbr in &adj[leaf as usize] {
                if degrees[nbr as usize] > 0 {
                    neighbor = nbr;
                    break;
                }
            }

            degrees[neighbor as usize] -= 1;
            deg = degrees[neighbor as usize];

            if deg > 0 {
                prufer.push(neighbor);
                prufer_idx += 1;
            }
            leaf = neighbor;
        }
    }

    Ok(prufer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn collect_edges_canonical(g: &Graph) -> BTreeSet<(VertexId, VertexId)> {
        let m = u32::try_from(g.ecount()).expect("ecount fits u32 in tests");
        (0..m)
            .map(|eid| g.edge(eid).expect("edge in range"))
            .map(|(a, b)| if a <= b { (a, b) } else { (b, a) })
            .collect()
    }

    fn uf_find(parent: &mut [u32], mut node: u32) -> u32 {
        while parent[node as usize] != node {
            let grand = parent[parent[node as usize] as usize];
            parent[node as usize] = grand;
            node = grand;
        }
        node
    }

    fn is_tree(graph: &Graph) -> bool {
        // Connected via union-find + edge count == n - 1.
        let vcount = graph.vcount();
        let ecount = graph.ecount();
        if vcount == 0 {
            return ecount == 0;
        }
        if ecount != (vcount as usize) - 1 {
            return false;
        }
        let mut parent: Vec<u32> = (0..vcount).collect();
        let em = u32::try_from(ecount).expect("ecount fits u32 in tests");
        for eid in 0..em {
            let (src, dst) = graph.edge(eid).expect("edge in range");
            let rs = uf_find(&mut parent, src);
            let rd = uf_find(&mut parent, dst);
            if rs == rd {
                return false; // cycle
            }
            parent[rs as usize] = rd;
        }
        let root = uf_find(&mut parent, 0);
        (1..vcount).all(|v| uf_find(&mut parent, v) == root)
    }

    #[test]
    fn empty_prufer_yields_p2() {
        let g = from_prufer(&[]).expect("empty");
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 1);
        assert!(!g.is_directed());
        let edges = collect_edges_canonical(&g);
        let expected: BTreeSet<(u32, u32)> = [(0, 1)].into_iter().collect();
        assert_eq!(edges, expected);
    }

    #[test]
    fn upstream_fixture_2_3_2_3() {
        // From references/igraph/tests/unit/igraph_from_prufer.out:
        // edges: (2,0) (3,1) (4,2) (3,2) (5,3)
        let g = from_prufer(&[2, 3, 2, 3]).expect("prufer1");
        assert_eq!(g.vcount(), 6);
        assert_eq!(g.ecount(), 5);
        let edges = collect_edges_canonical(&g);
        let expected: BTreeSet<(u32, u32)> = [(0, 2), (1, 3), (2, 4), (2, 3), (3, 5)]
            .into_iter()
            .collect();
        assert_eq!(edges, expected);
        assert!(is_tree(&g));
    }

    #[test]
    fn upstream_fixture_0_2_4_1_1_0() {
        // From references/igraph/tests/unit/igraph_from_prufer.out:
        // edges: (3,0) (5,2) (4,2) (4,1) (6,1) (1,0) (7,0)
        let g = from_prufer(&[0, 2, 4, 1, 1, 0]).expect("prufer2");
        assert_eq!(g.vcount(), 8);
        assert_eq!(g.ecount(), 7);
        let edges = collect_edges_canonical(&g);
        let expected: BTreeSet<(u32, u32)> =
            [(0, 3), (2, 5), (2, 4), (1, 4), (1, 6), (0, 1), (0, 7)]
                .into_iter()
                .collect();
        assert_eq!(edges, expected);
        assert!(is_tree(&g));
    }

    #[test]
    fn invalid_entry_out_of_range_errors() {
        // n = 4, entry 4 is out of [0, 4).
        let err = from_prufer(&[0, 4]).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn constant_sequence_yields_star() {
        // Repeating a single vertex u in every Prufer slot makes u the
        // centre of a star (it is the parent of every other vertex).
        // prufer = [0, 0, 0, 0] → n = 6, star centred on 0.
        let g = from_prufer(&[0, 0, 0, 0]).expect("star");
        assert_eq!(g.vcount(), 6);
        assert_eq!(g.ecount(), 5);
        assert!(is_tree(&g));
        // Vertex 0 should have degree 5.
        let deg0 = g.neighbors(0).expect("neighbors of 0").len();
        assert_eq!(deg0, 5);
    }

    #[test]
    fn ascending_sequence_yields_path() {
        // prufer = [1, 2, 3, 4] → tree is the path 0-1-2-3-4-5.
        let g = from_prufer(&[1, 2, 3, 4]).expect("path");
        assert_eq!(g.vcount(), 6);
        assert_eq!(g.ecount(), 5);
        assert!(is_tree(&g));
        // Every interior vertex has degree 2, endpoints have degree 1.
        for v in 0..6u32 {
            let d = g.neighbors(v).expect("neighbors").len();
            let expected = if v == 0 || v == 5 { 1 } else { 2 };
            assert_eq!(d, expected, "vertex {v} degree");
        }
    }

    #[test]
    fn single_entry_prufer() {
        // prufer = [0] → n = 3, decode yields edges (0,1) and (0,2).
        let g = from_prufer(&[0]).expect("single");
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 2);
        assert!(is_tree(&g));
        let deg0 = g.neighbors(0).expect("neighbors").len();
        assert_eq!(deg0, 2); // 0 is the centre.
    }

    #[test]
    fn always_undirected() {
        let g = from_prufer(&[2, 3, 2, 3]).expect("ok");
        assert!(!g.is_directed());
    }

    #[test]
    fn no_self_loops_or_multi_edges() {
        let g = from_prufer(&[0, 2, 4, 1, 1, 0]).expect("ok");
        let m = u32::try_from(g.ecount()).expect("m fits u32 in tests");
        let mut seen: BTreeSet<(u32, u32)> = BTreeSet::new();
        for e in 0..m {
            let (a, b) = g.edge(e).expect("edge");
            assert_ne!(a, b, "tree must not have self-loops");
            let canon = if a <= b { (a, b) } else { (b, a) };
            assert!(seen.insert(canon), "duplicate edge {canon:?}");
        }
    }

    // --- to_prufer tests ---

    #[test]
    fn to_prufer_roundtrip_2_3_2_3() {
        let tree = from_prufer(&[2, 3, 2, 3]).expect("decode");
        let seq = to_prufer(&tree).expect("encode");
        assert_eq!(seq, vec![2, 3, 2, 3]);
    }

    #[test]
    fn to_prufer_roundtrip_constant() {
        // Star centered at 3: Prüfer = [3, 3, 3]
        let tree = from_prufer(&[3, 3, 3]).expect("decode");
        let seq = to_prufer(&tree).expect("encode");
        assert_eq!(seq, vec![3, 3, 3]);
    }

    #[test]
    fn to_prufer_roundtrip_path() {
        // Path 0-1-2-3: Prüfer = [1, 2]
        let tree = from_prufer(&[1, 2]).expect("decode");
        let seq = to_prufer(&tree).expect("encode");
        assert_eq!(seq, vec![1, 2]);
    }

    #[test]
    fn to_prufer_p2() {
        // P_2 has empty Prüfer sequence
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).expect("add edge");
        let seq = to_prufer(&g).expect("encode");
        assert!(seq.is_empty());
    }

    #[test]
    fn to_prufer_not_a_tree_cycle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).expect("ok");
        g.add_edge(1, 2).expect("ok");
        g.add_edge(2, 0).expect("ok");
        assert!(to_prufer(&g).is_err());
    }

    #[test]
    fn to_prufer_not_a_tree_disconnected() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).expect("ok");
        g.add_edge(2, 3).expect("ok");
        // 2 edges but needs 3 for a tree on 4 vertices
        assert!(to_prufer(&g).is_err());
    }

    #[test]
    fn to_prufer_single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(to_prufer(&g).is_err());
    }

    #[test]
    fn to_prufer_roundtrip_large() {
        let seq = vec![0, 2, 4, 1, 1, 0];
        let tree = from_prufer(&seq).expect("decode");
        let result = to_prufer(&tree).expect("encode");
        assert_eq!(result, seq);
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::BTreeSet;

    fn arb_prufer() -> impl Strategy<Value = Vec<u32>> {
        // Strategy: pick n in [2, 30], then build a length-(n-2) vector
        // whose entries are in [0, n).
        (2u32..30).prop_flat_map(|n| prop::collection::vec(0u32..n, (n - 2) as usize))
    }

    fn uf_find(parent: &mut [u32], mut node: u32) -> u32 {
        while parent[node as usize] != node {
            let grand = parent[parent[node as usize] as usize];
            parent[node as usize] = grand;
            node = grand;
        }
        node
    }

    proptest! {
        #[test]
        fn always_a_tree(prufer in arb_prufer()) {
            let g = from_prufer(&prufer).unwrap();
            let n = u32::try_from(prufer.len()).unwrap() + 2;
            prop_assert_eq!(g.vcount(), n);
            prop_assert_eq!(g.ecount(), (n - 1) as usize);
            prop_assert!(!g.is_directed());
        }

        #[test]
        fn no_self_loops(prufer in arb_prufer()) {
            let g = from_prufer(&prufer).unwrap();
            let m = u32::try_from(g.ecount()).unwrap();
            for e in 0..m {
                let (a, b) = g.edge(e).unwrap();
                prop_assert_ne!(a, b);
            }
        }

        #[test]
        fn no_duplicate_edges(prufer in arb_prufer()) {
            let g = from_prufer(&prufer).unwrap();
            let m = u32::try_from(g.ecount()).unwrap();
            let mut seen: BTreeSet<(u32, u32)> = BTreeSet::new();
            for e in 0..m {
                let (a, b) = g.edge(e).unwrap();
                let canon = if a <= b { (a, b) } else { (b, a) };
                prop_assert!(seen.insert(canon));
            }
        }

        #[test]
        fn connected_undirected_tree(prufer in arb_prufer()) {
            // Tree iff connected and exactly n-1 edges (already checked).
            let graph = from_prufer(&prufer).unwrap();
            let vcount = graph.vcount();
            let mut parent: Vec<u32> = (0..vcount).collect();
            let ecount = u32::try_from(graph.ecount()).unwrap();
            for eid in 0..ecount {
                let (src, dst) = graph.edge(eid).unwrap();
                let rs = uf_find(&mut parent, src);
                let rd = uf_find(&mut parent, dst);
                prop_assert_ne!(rs, rd, "cycle detected — not a tree");
                parent[rs as usize] = rd;
            }
            let root = uf_find(&mut parent, 0);
            for v in 1..vcount {
                prop_assert_eq!(uf_find(&mut parent, v), root);
            }
        }

        #[test]
        fn invalid_entry_errors(
            n in 3u32..20,
            bad in 20u32..30,
        ) {
            // Construct a prufer of length n-2 where the first slot is
            // out of range (n is at least 3, so bad in [20, 30) >> n).
            let len = (n - 2) as usize;
            let mut p = vec![0u32; len];
            p[0] = bad; // bad >= 20 > n
            let err = from_prufer(&p).unwrap_err();
            prop_assert!(matches!(err, IgraphError::InvalidArgument(_)));
        }
    }
}

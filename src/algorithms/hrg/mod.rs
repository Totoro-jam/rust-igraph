//! Hierarchical Random Graph (HRG) models.
//!
//! A hierarchical random graph is an ensemble of undirected graphs
//! defined via a binary tree with `n` leaf vertices and `n-1` internal
//! vertices labelled with connection probabilities. The probability
//! that two leaf vertices are connected equals the probability at
//! their lowest common ancestor.
//!
//! Reference: A. Clauset, C. Moore, and M.E.J. Newman. "Hierarchical
//! structure and the prediction of missing links in networks." Nature
//! 453, 98–101 (2008).

mod mcmc;

pub use mcmc::{hrg_consensus, hrg_fit, hrg_predict};

use crate::core::error::{IgraphError, IgraphResult};
use crate::core::graph::{Graph, VertexId};
use crate::core::rng::SplitMix64;

/// A hierarchical random graph represented as a binary dendrogram.
///
/// Internal vertices are identified by negative indices starting at `-1`
/// (the root). Leaf vertices use non-negative indices `0..n`. Each
/// internal vertex `-(i+1)` has its data at index `i` in the vectors.
///
/// # Example
///
/// ```
/// use rust_igraph::HrgTree;
///
/// let hrg = HrgTree::new(3);
/// assert_eq!(hrg.size(), 3);
/// assert_eq!(hrg.num_internal(), 2);
/// ```
#[derive(Debug, Clone)]
pub struct HrgTree {
    /// Number of leaf vertices.
    n: u32,
    /// Left child of each internal vertex. Negative = internal, non-negative = leaf.
    pub left: Vec<i32>,
    /// Right child of each internal vertex.
    pub right: Vec<i32>,
    /// Connection probability at each internal vertex.
    pub prob: Vec<f64>,
    /// Number of leaf vertices in the subtree of each internal vertex.
    pub vertices: Vec<i32>,
    /// Number of tree edges in the subtree below each internal vertex.
    pub edges: Vec<i32>,
}

impl HrgTree {
    /// Create a new HRG with `n` leaf vertices, all arrays zero-initialized.
    pub fn new(n: u32) -> Self {
        let internal = if n <= 1 { 0 } else { (n - 1) as usize };
        HrgTree {
            n,
            left: vec![0; internal],
            right: vec![0; internal],
            prob: vec![0.0; internal],
            vertices: vec![0; internal],
            edges: vec![0; internal],
        }
    }

    /// The number of leaf vertices.
    pub fn size(&self) -> u32 {
        self.n
    }

    /// The number of internal vertices (`size - 1`, or 0 for trivial trees).
    pub fn num_internal(&self) -> usize {
        self.left.len()
    }

    /// Resize the HRG to hold `new_size` leaf vertices.
    pub fn resize(&mut self, new_size: u32) {
        self.n = new_size;
        let internal = if new_size <= 1 {
            0
        } else {
            (new_size - 1) as usize
        };
        self.left.resize(internal, 0);
        self.right.resize(internal, 0);
        self.prob.resize(internal, 0.0);
        self.vertices.resize(internal, 0);
        self.edges.resize(internal, 0);
    }
}

#[allow(clippy::cast_sign_loss)]
fn internal_idx(neg: i32) -> usize {
    debug_assert!(neg < 0);
    (-neg - 1) as usize
}

fn to_u32(v: usize) -> IgraphResult<u32> {
    u32::try_from(v).map_err(|_| IgraphError::Internal("value exceeds u32::MAX"))
}

/// Validate preconditions and compute in/out degrees for `hrg_create`.
fn hrg_validate_and_degrees(graph: &Graph, prob: &[f64]) -> IgraphResult<(Vec<u32>, Vec<u32>)> {
    let n = graph.vcount();

    if n < 3 {
        return Err(IgraphError::InvalidArgument(
            "HRG tree must have at least three vertices".into(),
        ));
    }
    if !graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "HRG graph must be directed".into(),
        ));
    }
    if n % 2 == 0 {
        return Err(IgraphError::InvalidArgument(
            "Complete HRG graph must have an odd number of vertices".into(),
        ));
    }
    if !graph.is_simple()? {
        return Err(IgraphError::InvalidArgument(
            "HRG graph must be a simple graph".into(),
        ));
    }

    let no_of_internal = (n as usize) / 2;
    if prob.len() != no_of_internal {
        return Err(IgraphError::InvalidArgument(format!(
            "HRG probability vector size ({}) should equal the number of internal nodes ({})",
            prob.len(),
            no_of_internal,
        )));
    }

    let mut in_deg = vec![0u32; n as usize];
    let mut out_deg = vec![0u32; n as usize];
    for eid in 0..graph.ecount() {
        let src = graph.edge_source(to_u32(eid)?)?;
        let tgt = graph.edge_target(to_u32(eid)?)?;
        out_deg[src as usize] = out_deg[src as usize]
            .checked_add(1)
            .ok_or(IgraphError::Internal("degree overflow"))?;
        in_deg[tgt as usize] = in_deg[tgt as usize]
            .checked_add(1)
            .ok_or(IgraphError::Internal("degree overflow"))?;
    }

    Ok((in_deg, out_deg))
}

/// Find the root (in-degree 0) and validate degree structure.
fn hrg_find_root_and_validate(
    n: u32,
    in_deg: &[u32],
    out_deg: &[u32],
) -> IgraphResult<(VertexId, u32)> {
    let mut root: Option<VertexId> = None;
    let mut root_count = 0u32;
    for v in 0..n {
        match in_deg[v as usize] {
            0 => {
                root_count = root_count
                    .checked_add(1)
                    .ok_or(IgraphError::Internal("count overflow"))?;
                root = Some(v);
            }
            1 => {}
            _ => {
                return Err(IgraphError::InvalidArgument(
                    "HRG nodes must have in-degree 0 or 1".into(),
                ));
            }
        }
    }
    if root_count != 1 {
        return Err(IgraphError::InvalidArgument(
            "HRG must have exactly one root vertex (in-degree 0)".into(),
        ));
    }
    let root = root.ok_or_else(|| IgraphError::InvalidArgument("HRG has no root vertex".into()))?;

    let mut leaf_count = 0u32;
    let mut internal_count = 0u32;
    for v in 0..n {
        match out_deg[v as usize] {
            0 => {
                leaf_count = leaf_count
                    .checked_add(1)
                    .ok_or(IgraphError::Internal("count overflow"))?;
            }
            2 => {
                internal_count = internal_count
                    .checked_add(1)
                    .ok_or(IgraphError::Internal("count overflow"))?;
            }
            _ => {
                return Err(IgraphError::InvalidArgument(
                    "HRG nodes must have out-degree 2 (internal) or 0 (leaf)".into(),
                ));
            }
        }
    }
    let expected_leaves = internal_count
        .checked_add(1)
        .ok_or(IgraphError::Internal("count overflow"))?;
    if leaf_count != expected_leaves {
        return Err(IgraphError::InvalidArgument(
            "HRG degrees are incorrect, maybe multiple components?".into(),
        ));
    }

    Ok((root, internal_count))
}

/// Build the HRG tree structure (children, probs, subtree counts).
fn hrg_build_tree(
    graph: &Graph,
    prob: &[f64],
    root: VertexId,
    out_deg: &[u32],
    internal_count: u32,
) -> IgraphResult<HrgTree> {
    let n = graph.vcount();

    // Build index: root first, then remaining internals in ascending order, then leaves
    let mut idx = vec![0i32; n as usize];
    let mut ii: i32 = 0;
    let mut il: i32 = 0;

    idx[root as usize] = -(ii + 1);
    ii += 1;

    for v in 0..n {
        if v == root {
            continue;
        }
        if out_deg[v as usize] == 2 {
            idx[v as usize] = -(ii + 1);
            ii += 1;
        } else {
            idx[v as usize] = il;
            il += 1;
        }
    }

    let leaf_total = internal_count
        .checked_add(1)
        .ok_or(IgraphError::Internal("size overflow"))?;
    let mut hrg = HrgTree::new(leaf_total);

    // Build out-edge adjacency
    let mut out_edges: Vec<Vec<VertexId>> = vec![Vec::new(); n as usize];
    for eid in 0..graph.ecount() {
        let src = graph.edge_source(to_u32(eid)?)?;
        let tgt = graph.edge_target(to_u32(eid)?)?;
        out_edges[src as usize].push(tgt);
    }

    // Fill left/right children and assign probabilities
    let mut prob_idx = 0usize;
    hrg.prob[0] = prob[prob_idx];
    prob_idx += 1;

    for v in 0..n {
        let ri = idx[v as usize];
        if ri >= 0 {
            continue;
        }
        let children = &out_edges[v as usize];
        if children.len() != 2 {
            return Err(IgraphError::InvalidArgument(format!(
                "Internal vertex {} has {} out-edges, expected 2",
                v,
                children.len()
            )));
        }
        let hi = internal_idx(ri);
        hrg.left[hi] = idx[children[0] as usize];
        hrg.right[hi] = idx[children[1] as usize];

        if v != root {
            hrg.prob[hi] = prob[prob_idx];
            prob_idx += 1;
        }
    }

    // Compute subtree counts via iterative post-order traversal
    let mut stack: Vec<i32> = vec![-1];
    while let Some(&current) = stack.last() {
        let ci = internal_idx(current);
        let lc = hrg.left[ci];
        let rc = hrg.right[ci];

        if lc < 0 && hrg.vertices[internal_idx(lc)] == 0 {
            stack.push(lc);
            continue;
        }
        if rc < 0 && hrg.vertices[internal_idx(rc)] == 0 {
            stack.push(rc);
            continue;
        }

        let lv = if lc < 0 {
            hrg.vertices[internal_idx(lc)]
        } else {
            1
        };
        let rv = if rc < 0 {
            hrg.vertices[internal_idx(rc)]
        } else {
            1
        };
        hrg.vertices[ci] = lv
            .checked_add(rv)
            .ok_or(IgraphError::Internal("vertex count overflow"))?;

        let le = if lc < 0 {
            hrg.edges[internal_idx(lc)]
                .checked_add(1)
                .ok_or(IgraphError::Internal("edge count overflow"))?
        } else {
            1
        };
        let re = if rc < 0 {
            hrg.edges[internal_idx(rc)]
                .checked_add(1)
                .ok_or(IgraphError::Internal("edge count overflow"))?
        } else {
            1
        };
        hrg.edges[ci] = le
            .checked_add(re)
            .ok_or(IgraphError::Internal("edge count overflow"))?;

        stack.pop();
    }

    Ok(hrg)
}

/// Create an [`HrgTree`] from a directed binary tree graph.
///
/// The input `graph` must be a directed, simple binary tree with an odd
/// number of vertices `2n-1`: `n-1` internal vertices (out-degree 2) and
/// `n` leaves (out-degree 0). Exactly one vertex (the root) must have
/// in-degree 0; all others must have in-degree 1.
///
/// `prob` has one entry per internal vertex. Internal vertices are
/// enumerated as: root first, then remaining internals in ascending
/// vertex-id order.
///
/// # Example
///
/// ```
/// use rust_igraph::{Graph, hrg_create};
///
/// // 5-vertex binary tree: root(0) -> 1,2; vertex 1 -> 3,4
/// let g = Graph::from_edges(&[(0,1),(0,2),(1,3),(1,4)], true, Some(5)).unwrap();
/// let prob = vec![0.3, 0.7]; // root=0.3, vertex1=0.7
/// let hrg = hrg_create(&g, &prob).unwrap();
/// assert_eq!(hrg.size(), 3);
/// ```
pub fn hrg_create(graph: &Graph, prob: &[f64]) -> IgraphResult<HrgTree> {
    let (in_deg, out_deg) = hrg_validate_and_degrees(graph, prob)?;
    let n = graph.vcount();
    let (root, internal_count) = hrg_find_root_and_validate(n, &in_deg, &out_deg)?;
    hrg_build_tree(graph, prob, root, &out_deg, internal_count)
}

/// Result of converting an [`HrgTree`] to a dendrogram graph.
#[derive(Debug, Clone)]
pub struct HrgDendrogram {
    /// The dendrogram as a directed graph. Vertices `0..n` are leaves,
    /// vertices `n..2n-1` are internal nodes.
    pub graph: Graph,
    /// Connection probability per vertex. Leaves have `f64::NAN`.
    pub prob: Vec<f64>,
}

fn hrg_child_vertex(orig_nodes: u32, child: i32) -> IgraphResult<u32> {
    if child < 0 {
        let ci =
            u32::try_from(-child - 1).map_err(|_| IgraphError::Internal("vertex id overflow"))?;
        orig_nodes
            .checked_add(ci)
            .ok_or(IgraphError::Internal("vertex id overflow"))
    } else {
        u32::try_from(child).map_err(|_| IgraphError::Internal("vertex id overflow"))
    }
}

/// Convert an [`HrgTree`] into a directed graph dendrogram.
///
/// The graph has `2n - 1` vertices (`n = hrg.size()`): vertices `0..n`
/// are leaves, `n..2n-1` are internal. Each internal vertex has two
/// directed edges to its children.
///
/// # Example
///
/// ```
/// use rust_igraph::{HrgTree, from_hrg_dendrogram};
///
/// let mut hrg = HrgTree::new(3);
/// hrg.left[0] = 0;   hrg.right[0] = -2;  hrg.prob[0] = 0.5;
/// hrg.left[1] = 1;   hrg.right[1] = 2;   hrg.prob[1] = 0.8;
/// hrg.vertices = vec![3, 2];
/// hrg.edges = vec![4, 2];
///
/// let d = from_hrg_dendrogram(&hrg).unwrap();
/// assert_eq!(d.graph.vcount(), 5);
/// assert_eq!(d.graph.ecount(), 4);
/// assert!(d.prob[0].is_nan()); // leaf
/// assert!((d.prob[3] - 0.5).abs() < 1e-10);
/// ```
pub fn from_hrg_dendrogram(hrg: &HrgTree) -> IgraphResult<HrgDendrogram> {
    let orig_nodes = hrg.size();
    if orig_nodes <= 1 && hrg.num_internal() == 0 {
        let g = Graph::new(orig_nodes, true)?;
        let prob_vec = if orig_nodes == 1 {
            vec![f64::NAN]
        } else {
            vec![]
        };
        return Ok(HrgDendrogram {
            graph: g,
            prob: prob_vec,
        });
    }

    let no_of_nodes = orig_nodes
        .checked_mul(2)
        .and_then(|x| x.checked_sub(1))
        .ok_or(IgraphError::Internal("node count overflow"))?;

    let mut prob_vec = Vec::with_capacity(no_of_nodes as usize);
    for _ in 0..orig_nodes {
        prob_vec.push(f64::NAN);
    }
    for i in 0..hrg.num_internal() {
        prob_vec.push(hrg.prob[i]);
    }

    let mut edges: Vec<(u32, u32)> = Vec::with_capacity(2 * hrg.num_internal());
    for i in 0..hrg.num_internal() {
        let parent = orig_nodes
            .checked_add(to_u32(i)?)
            .ok_or(IgraphError::Internal("vertex id overflow"))?;

        let left_v = hrg_child_vertex(orig_nodes, hrg.left[i])?;
        let right_v = hrg_child_vertex(orig_nodes, hrg.right[i])?;

        edges.push((parent, left_v));
        edges.push((parent, right_v));
    }

    let graph = Graph::from_edges(&edges, true, Some(no_of_nodes))?;

    Ok(HrgDendrogram {
        graph,
        prob: prob_vec,
    })
}

/// Find the lowest common ancestor of two leaves in an HRG tree.
///
/// Returns the internal index of the LCA. Works by precomputing parent
/// pointers and walking up from both leaves until they meet.
#[allow(clippy::cast_sign_loss)]
fn find_lca(hrg: &HrgTree, parent: &[i32], leaf_a: i32, leaf_b: i32) -> usize {
    let num_internal = hrg.num_internal();

    // Walk both up via parent pointers, mark visited from one path
    let mut visited = vec![false; num_internal];

    let mut cur = leaf_a;
    loop {
        if cur < 0 {
            let idx = internal_idx(cur);
            visited[idx] = true;
            if idx == 0 {
                break; // reached root
            }
            cur = parent[idx];
        } else {
            // leaf → go to parent (cur is non-negative here)
            cur = parent[num_internal + cur as usize];
        }
    }

    let mut cur = leaf_b;
    loop {
        if cur < 0 {
            let idx = internal_idx(cur);
            if visited[idx] {
                return idx;
            }
            if idx == 0 {
                return 0; // root is always LCA
            }
            cur = parent[idx];
        } else {
            cur = parent[num_internal + cur as usize];
        }
    }
}

/// Build a parent-pointer array for all nodes in the HRG.
///
/// Layout: `parent[0..num_internal]` = parent of internal node i,
/// `parent[num_internal..num_internal+n]` = parent of leaf i.
/// Root's parent is stored as 0 (unused sentinel).
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
fn build_parent_map(hrg: &HrgTree) -> Vec<i32> {
    let num_internal = hrg.num_internal();
    let n = hrg.size() as usize;
    // parent[i] for i in 0..num_internal -> parent internal id of internal node i
    // parent[num_internal + leaf_id] -> parent internal id of leaf
    let mut parent = vec![0i32; num_internal + n];

    for i in 0..num_internal {
        let self_id = -(i as i32 + 1);
        let lc = hrg.left[i];
        let rc = hrg.right[i];

        if lc < 0 {
            parent[internal_idx(lc)] = self_id;
        } else {
            parent[num_internal + lc as usize] = self_id;
        }
        if rc < 0 {
            parent[internal_idx(rc)] = self_id;
        } else {
            parent[num_internal + rc as usize] = self_id;
        }
    }

    parent
}

#[allow(clippy::cast_possible_wrap)]
fn sample_one(hrg: &HrgTree, parent: &[i32], rng: &mut SplitMix64) -> IgraphResult<Graph> {
    let n = hrg.size();
    let mut edges: Vec<(u32, u32)> = Vec::new();

    for i in 0..n {
        for j in (i + 1)..n {
            let lca = find_lca(hrg, parent, i as i32, j as i32);
            if rng.gen_unit() < hrg.prob[lca] {
                edges.push((i, j));
            }
        }
    }

    Graph::from_edges(&edges, false, Some(n))
}

/// Sample a random graph from a hierarchical random graph model.
///
/// For each pair of leaf vertices `(i, j)`, an undirected edge is added
/// with probability equal to the connection probability at their lowest
/// common ancestor in the dendrogram.
///
/// `seed` initialises the internal PRNG. Same `(hrg, seed)` always
/// produces the same graph.
///
/// # Example
///
/// ```
/// use rust_igraph::{HrgTree, hrg_sample};
///
/// let mut hrg = HrgTree::new(4);
/// hrg.left[0] = -2;  hrg.right[0] = -3;  hrg.prob[0] = 0.5;
/// hrg.left[1] = 0;   hrg.right[1] = 1;   hrg.prob[1] = 1.0;
/// hrg.left[2] = 2;   hrg.right[2] = 3;   hrg.prob[2] = 1.0;
/// hrg.vertices = vec![4, 2, 2];
/// hrg.edges = vec![6, 2, 2];
///
/// let g = hrg_sample(&hrg, 42).unwrap();
/// assert_eq!(g.vcount(), 4);
/// // With prob=1.0 within subtrees, leaves 0-1 and 2-3 are always connected
/// ```
pub fn hrg_sample(hrg: &HrgTree, seed: u64) -> IgraphResult<Graph> {
    let n = hrg.size();
    if n <= 1 {
        return Graph::new(n, false);
    }
    let parent = build_parent_map(hrg);
    let mut rng = SplitMix64::new(seed);
    sample_one(hrg, &parent, &mut rng)
}

/// Sample multiple random graphs from a hierarchical random graph model.
///
/// Generates `num_samples` independent draws from the HRG ensemble.
///
/// # Example
///
/// ```
/// use rust_igraph::{HrgTree, hrg_sample_many};
///
/// let mut hrg = HrgTree::new(3);
/// hrg.left[0] = 0;   hrg.right[0] = -2;  hrg.prob[0] = 0.5;
/// hrg.left[1] = 1;   hrg.right[1] = 2;   hrg.prob[1] = 1.0;
/// hrg.vertices = vec![3, 2];
/// hrg.edges = vec![4, 2];
///
/// let graphs = hrg_sample_many(&hrg, 5, 123).unwrap();
/// assert_eq!(graphs.len(), 5);
/// for g in &graphs {
///     assert_eq!(g.vcount(), 3);
/// }
/// ```
pub fn hrg_sample_many(hrg: &HrgTree, num_samples: usize, seed: u64) -> IgraphResult<Vec<Graph>> {
    let n = hrg.size();
    if n <= 1 {
        let g = Graph::new(n, false)?;
        return Ok(vec![g; num_samples]);
    }
    let parent = build_parent_map(hrg);
    let mut rng = SplitMix64::new(seed);
    let mut results = Vec::with_capacity(num_samples);
    for _ in 0..num_samples {
        results.push(sample_one(hrg, &parent, &mut rng)?);
    }
    Ok(results)
}

/// Generate a hierarchical random graph (alias for [`hrg_sample`]).
pub fn hrg_game(hrg: &HrgTree, seed: u64) -> IgraphResult<Graph> {
    hrg_sample(hrg, seed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hrg_tree_new_and_size() {
        let hrg = HrgTree::new(5);
        assert_eq!(hrg.size(), 5);
        assert_eq!(hrg.num_internal(), 4);
    }

    #[test]
    fn hrg_tree_single_vertex() {
        let hrg = HrgTree::new(1);
        assert_eq!(hrg.size(), 1);
        assert_eq!(hrg.num_internal(), 0);
    }

    #[test]
    fn hrg_tree_resize() {
        let mut hrg = HrgTree::new(3);
        assert_eq!(hrg.num_internal(), 2);
        hrg.resize(5);
        assert_eq!(hrg.num_internal(), 4);
        assert_eq!(hrg.size(), 5);
    }

    #[test]
    fn from_hrg_dendrogram_three_leaves() {
        let mut hrg = HrgTree::new(3);
        hrg.left[0] = 0;
        hrg.right[0] = -2;
        hrg.prob[0] = 0.5;
        hrg.left[1] = 1;
        hrg.right[1] = 2;
        hrg.prob[1] = 0.8;
        hrg.vertices = vec![3, 2];
        hrg.edges = vec![4, 2];

        let d = from_hrg_dendrogram(&hrg).expect("should succeed");
        assert_eq!(d.graph.vcount(), 5);
        assert_eq!(d.graph.ecount(), 4);
        assert!(d.graph.is_directed());

        assert!(d.prob[0].is_nan());
        assert!(d.prob[1].is_nan());
        assert!(d.prob[2].is_nan());
        assert!((d.prob[3] - 0.5).abs() < 1e-10);
        assert!((d.prob[4] - 0.8).abs() < 1e-10);
    }

    #[test]
    fn hrg_create_five_vertex_tree() {
        let g = Graph::from_edges(&[(0, 1), (0, 2), (1, 3), (1, 4)], true, Some(5))
            .expect("graph creation");
        let prob = vec![0.3, 0.7];
        let hrg = hrg_create(&g, &prob).expect("hrg_create");

        assert_eq!(hrg.size(), 3);
        assert_eq!(hrg.num_internal(), 2);
        assert!((hrg.prob[0] - 0.3).abs() < 1e-10);
        assert!((hrg.prob[1] - 0.7).abs() < 1e-10);
        assert_eq!(hrg.vertices[0], 3);
    }

    #[test]
    fn hrg_create_rejects_undirected() {
        let g = Graph::from_edges(&[(0, 1), (0, 2)], false, Some(3)).expect("graph creation");
        assert!(hrg_create(&g, &[0.5]).is_err());
    }

    #[test]
    fn hrg_create_rejects_even_vertices() {
        let g =
            Graph::from_edges(&[(0, 1), (0, 2), (2, 3)], true, Some(4)).expect("graph creation");
        assert!(hrg_create(&g, &[0.5, 0.6]).is_err());
    }

    #[test]
    fn hrg_create_rejects_too_small() {
        let g = Graph::new(1, true).expect("graph creation");
        assert!(hrg_create(&g, &[]).is_err());
    }

    #[test]
    fn hrg_create_rejects_wrong_prob_len() {
        let g = Graph::from_edges(&[(0, 1), (0, 2), (1, 3), (1, 4)], true, Some(5))
            .expect("graph creation");
        assert!(hrg_create(&g, &[0.3, 0.7, 0.9]).is_err());
    }

    #[test]
    fn roundtrip_create_and_dendrogram() {
        let g = Graph::from_edges(&[(0, 1), (0, 2), (1, 3), (1, 4)], true, Some(5))
            .expect("graph creation");
        let prob = vec![0.3, 0.7];
        let hrg = hrg_create(&g, &prob).expect("hrg_create");
        let d = from_hrg_dendrogram(&hrg).expect("from_hrg_dendrogram");

        assert_eq!(d.graph.vcount(), 5);
        assert_eq!(d.graph.ecount(), 4);
        for i in 0..3u32 {
            assert!(d.prob[i as usize].is_nan());
        }
        assert!((d.prob[3] - 0.3).abs() < 1e-10);
        assert!((d.prob[4] - 0.7).abs() < 1e-10);
    }

    #[test]
    fn from_hrg_dendrogram_empty() {
        let hrg = HrgTree::new(0);
        let d = from_hrg_dendrogram(&hrg).expect("should succeed");
        assert_eq!(d.graph.vcount(), 0);
    }

    #[test]
    fn hrg_create_seven_vertex_tree() {
        let g = Graph::from_edges(
            &[(0, 1), (0, 2), (1, 3), (1, 4), (2, 5), (2, 6)],
            true,
            Some(7),
        )
        .expect("graph creation");
        let prob = vec![0.1, 0.2, 0.3];
        let hrg = hrg_create(&g, &prob).expect("hrg_create");

        assert_eq!(hrg.size(), 4);
        assert_eq!(hrg.num_internal(), 3);
        assert_eq!(hrg.vertices[0], 4);
        assert!((hrg.prob[0] - 0.1).abs() < 1e-10);
        assert!((hrg.prob[1] - 0.2).abs() < 1e-10);
        assert!((hrg.prob[2] - 0.3).abs() < 1e-10);
    }

    // ── HRG-002: hrg_sample / hrg_sample_many / hrg_game ──

    fn make_sample_hrg() -> HrgTree {
        // 3-leaf HRG: root splits into leaf 0 and internal node 1;
        // internal node 1 splits into leaves 1 and 2.
        let mut hrg = HrgTree::new(3);
        hrg.left[0] = 0;
        hrg.right[0] = -2; // internal node 1
        hrg.prob[0] = 0.5;
        hrg.left[1] = 1;
        hrg.right[1] = 2;
        hrg.prob[1] = 1.0;
        hrg.vertices = vec![3, 2];
        hrg.edges = vec![4, 2];
        hrg
    }

    #[test]
    fn hrg_sample_correct_vcount() {
        let hrg = make_sample_hrg();
        let g = hrg_sample(&hrg, 42).expect("hrg_sample");
        assert_eq!(g.vcount(), 3);
        assert!(!g.is_directed());
    }

    #[test]
    fn hrg_sample_deterministic() {
        let hrg = make_sample_hrg();
        let g1 = hrg_sample(&hrg, 99).expect("hrg_sample");
        let g2 = hrg_sample(&hrg, 99).expect("hrg_sample");
        assert_eq!(g1.ecount(), g2.ecount());
        for eid in 0..g1.ecount() {
            #[allow(clippy::cast_possible_truncation)]
            let eid32 = eid as u32;
            let (s1, t1) = g1.edge(eid32).expect("edge");
            let (s2, t2) = g2.edge(eid32).expect("edge");
            assert_eq!(s1, s2);
            assert_eq!(t1, t2);
        }
    }

    #[test]
    fn hrg_sample_prob_one_always_connects() {
        // All probs = 1.0 → complete graph on 3 vertices (3 edges)
        let mut hrg = HrgTree::new(3);
        hrg.left[0] = 0;
        hrg.right[0] = -2;
        hrg.prob[0] = 1.0;
        hrg.left[1] = 1;
        hrg.right[1] = 2;
        hrg.prob[1] = 1.0;
        hrg.vertices = vec![3, 2];
        hrg.edges = vec![4, 2];

        for seed in 0..20u64 {
            let g = hrg_sample(&hrg, seed).expect("hrg_sample");
            assert_eq!(g.ecount(), 3, "prob=1.0 should yield K3");
        }
    }

    #[test]
    fn hrg_sample_prob_zero_never_connects() {
        let mut hrg = HrgTree::new(3);
        hrg.left[0] = 0;
        hrg.right[0] = -2;
        hrg.prob[0] = 0.0;
        hrg.left[1] = 1;
        hrg.right[1] = 2;
        hrg.prob[1] = 0.0;
        hrg.vertices = vec![3, 2];
        hrg.edges = vec![4, 2];

        for seed in 0..20u64 {
            let g = hrg_sample(&hrg, seed).expect("hrg_sample");
            assert_eq!(g.ecount(), 0, "prob=0.0 should yield empty graph");
        }
    }

    #[test]
    fn hrg_sample_single_vertex() {
        let hrg = HrgTree::new(1);
        let g = hrg_sample(&hrg, 0).expect("hrg_sample");
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn hrg_sample_empty() {
        let hrg = HrgTree::new(0);
        let g = hrg_sample(&hrg, 0).expect("hrg_sample");
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn hrg_sample_many_correct_count() {
        let hrg = make_sample_hrg();
        let graphs = hrg_sample_many(&hrg, 10, 42).expect("hrg_sample_many");
        assert_eq!(graphs.len(), 10);
        for g in &graphs {
            assert_eq!(g.vcount(), 3);
        }
    }

    #[test]
    fn hrg_sample_many_zero_samples() {
        let hrg = make_sample_hrg();
        let graphs = hrg_sample_many(&hrg, 0, 42).expect("hrg_sample_many");
        assert!(graphs.is_empty());
    }

    #[test]
    fn hrg_sample_many_deterministic() {
        let hrg = make_sample_hrg();
        let g1 = hrg_sample_many(&hrg, 5, 77).expect("hrg_sample_many");
        let g2 = hrg_sample_many(&hrg, 5, 77).expect("hrg_sample_many");
        for (a, b) in g1.iter().zip(g2.iter()) {
            assert_eq!(a.ecount(), b.ecount());
        }
    }

    #[test]
    fn hrg_game_is_alias() {
        let hrg = make_sample_hrg();
        let g1 = hrg_sample(&hrg, 123).expect("hrg_sample");
        let g2 = hrg_game(&hrg, 123).expect("hrg_game");
        assert_eq!(g1.ecount(), g2.ecount());
    }

    #[test]
    fn hrg_sample_statistical_edge_count() {
        // With prob[0]=0.5, prob[1]=1.0, leaf pair (1,2) always
        // connects (via internal 1, prob=1.0), while pairs (0,1) and
        // (0,2) each connect with prob 0.5 (via root, prob=0.5).
        // Expected edges = 1 + 0.5 + 0.5 = 2.0
        let hrg = make_sample_hrg();
        let n = 1000;
        let graphs = hrg_sample_many(&hrg, n, 42).expect("hrg_sample_many");
        let total_edges: usize = graphs.iter().map(Graph::ecount).sum();
        #[allow(clippy::cast_precision_loss)]
        let mean = total_edges as f64 / n as f64;
        assert!((mean - 2.0).abs() < 0.2, "expected mean ~2.0, got {mean}");
    }
}

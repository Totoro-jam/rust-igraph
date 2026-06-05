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

use crate::core::error::{IgraphError, IgraphResult};
use crate::core::graph::{Graph, VertexId};

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
}

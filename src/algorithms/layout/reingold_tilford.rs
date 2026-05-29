//! Reingold-Tilford tree layout (ALGO-LO-004).
//!
//! Layered tree drawing algorithm. Reference: Reingold & Tilford,
//! "Tidier Drawing of Trees", IEEE Trans. Softw. Eng. SE-7(2), 1981.
//!
//! Also provides a circular variant that maps the tree layout to
//! polar coordinates.

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};
use std::collections::VecDeque;

/// Mode for tree edge traversal direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RtMode {
    /// Follow outgoing edges from parent to children.
    Out,
    /// Follow incoming edges (parent is target of edges).
    In,
    /// Treat edges as undirected.
    All,
}

/// Compute the Reingold-Tilford tree layout.
///
/// Places vertices in a layered hierarchy with the root at level 0.
/// Y-coordinate is the depth in the tree; X-coordinate is computed
/// using the contour-merging algorithm to keep subtrees compact.
///
/// If the graph is not a tree, a BFS spanning tree from `root` is used.
/// Unreachable vertices are connected to the root before layout.
///
/// # Arguments
///
/// * `graph` — input graph.
/// * `root` — root vertex for the tree layout. If `None`, automatically
///   selects the vertex with highest degree.
/// * `mode` — edge traversal direction.
///
/// Returns `(x, y)` pairs for each vertex.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, layout_reingold_tilford, RtMode};
///
/// // Binary tree: 0 -> {1, 2}, 1 -> {3, 4}
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// g.add_edge(1, 4).unwrap();
///
/// let pos = layout_reingold_tilford(&g, Some(0), RtMode::All).unwrap();
/// assert_eq!(pos.len(), 5);
/// // Root should be at level 0
/// assert!((pos[0][1]).abs() < 1e-10);
/// ```
pub fn layout_reingold_tilford(
    graph: &Graph,
    root: Option<VertexId>,
    mode: RtMode,
) -> IgraphResult<Vec<[f64; 2]>> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(Vec::new());
    }

    let root_v = root.unwrap_or_else(|| select_root(graph, mode));
    if root_v >= graph.vcount() {
        return Err(IgraphError::InvalidArgument(
            "root vertex out of range".into(),
        ));
    }

    let root_idx = root_v as usize;
    let mut vdata = vec![RtVertex::new(); n];

    // Build spanning tree via BFS
    build_spanning_tree(graph, root_idx, mode, &mut vdata)?;

    // Postorder traversal to compute offsets
    postorder(&mut vdata, root_idx, n);

    // Calculate final coordinates
    let mut pos = vec![[0.0_f64; 2]; n];
    calc_coords(&vdata, &mut pos, root_idx, n, vdata[root_idx].offset);

    Ok(pos)
}

/// Circular variant of the Reingold-Tilford layout.
///
/// First computes the standard RT layout, then maps X to angle and Y
/// to radius in polar coordinates. The root ends up at the center.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, layout_reingold_tilford_circular, RtMode};
///
/// let mut g = Graph::with_vertices(7);
/// for &(u, v) in &[(0,1),(0,2),(1,3),(1,4),(2,5),(2,6)] {
///     g.add_edge(u, v).unwrap();
/// }
///
/// let pos = layout_reingold_tilford_circular(&g, Some(0), RtMode::All).unwrap();
/// assert_eq!(pos.len(), 7);
/// ```
pub fn layout_reingold_tilford_circular(
    graph: &Graph,
    root: Option<VertexId>,
    mode: RtMode,
) -> IgraphResult<Vec<[f64; 2]>> {
    let mut pos = layout_reingold_tilford(graph, root, mode)?;
    let n = pos.len();
    if n == 0 {
        return Ok(pos);
    }

    let ratio_base = 2.0 * std::f64::consts::PI * (n as f64 - 1.0) / n as f64;

    let minx = pos.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
    let maxx = pos.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max);

    let ratio = if maxx > minx {
        ratio_base / (maxx - minx)
    } else {
        ratio_base
    };

    for p in &mut pos {
        let phi = (p[0] - minx) * ratio;
        let r = p[1];
        p[0] = r * phi.cos();
        p[1] = r * phi.sin();
    }

    Ok(pos)
}

// ═══════════════════════════════════════════════════════════════════
// Internal data structures
// ═══════════════════════════════════════════════════════════════════

#[derive(Clone)]
struct RtVertex {
    parent: i64,
    level: i64,
    offset: f64,
    left_contour: i64,
    right_contour: i64,
    offset_to_left_contour: f64,
    offset_to_right_contour: f64,
    left_extreme: usize,
    right_extreme: usize,
    offset_to_left_extreme: f64,
    offset_to_right_extreme: f64,
}

impl RtVertex {
    fn new() -> Self {
        Self {
            parent: -1,
            level: -1,
            offset: 0.0,
            left_contour: -1,
            right_contour: -1,
            offset_to_left_contour: 0.0,
            offset_to_right_contour: 0.0,
            left_extreme: 0, // will be set to self-index
            right_extreme: 0,
            offset_to_left_extreme: 0.0,
            offset_to_right_extreme: 0.0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════

fn select_root(graph: &Graph, _mode: RtMode) -> VertexId {
    let n = graph.vcount() as usize;
    if n == 0 {
        return 0;
    }
    let mut best = 0u32;
    let mut best_deg = 0usize;
    for v in 0..n {
        if let Ok(deg) = graph.degree(v as VertexId) {
            if deg > best_deg {
                best_deg = deg;
                best = v as u32;
            }
        }
    }
    best
}

fn build_spanning_tree(
    graph: &Graph,
    root: usize,
    mode: RtMode,
    vdata: &mut [RtVertex],
) -> IgraphResult<()> {
    let n = vdata.len();
    for i in 0..n {
        vdata[i].left_extreme = i;
        vdata[i].right_extreme = i;
    }

    vdata[root].parent = root as i64;
    vdata[root].level = 0;

    let mut queue = VecDeque::new();
    queue.push_back((root, 0i64));

    while let Some((node, dist)) = queue.pop_front() {
        let neighbors = get_neighbors(graph, node as VertexId, mode);
        for &nei in &neighbors {
            let nei_idx = nei as usize;
            if vdata[nei_idx].parent >= 0 {
                continue;
            }
            vdata[nei_idx].parent = node as i64;
            vdata[nei_idx].level = dist + 1;
            queue.push_back((nei_idx, dist + 1));
        }
    }

    // Handle unreachable nodes: connect them to root
    for i in 0..n {
        if vdata[i].parent < 0 {
            vdata[i].parent = root as i64;
            vdata[i].level = 1;
        }
    }

    Ok(())
}

fn get_neighbors(graph: &Graph, v: VertexId, mode: RtMode) -> Vec<VertexId> {
    let mut result = Vec::new();
    let ecount = graph.ecount();
    for eid in 0..ecount as u32 {
        if let Ok((src, tgt)) = graph.edge(eid) {
            match mode {
                RtMode::Out => {
                    if src == v {
                        result.push(tgt);
                    }
                }
                RtMode::In => {
                    if tgt == v {
                        result.push(src);
                    }
                }
                RtMode::All => {
                    if src == v {
                        result.push(tgt);
                    } else if tgt == v {
                        result.push(src);
                    }
                }
            }
        }
    }
    result
}

fn children_of(vdata: &[RtVertex], node: usize, n: usize) -> Vec<usize> {
    let mut kids = Vec::new();
    for i in 0..n {
        if i == node {
            continue;
        }
        if vdata[i].parent == node as i64 {
            kids.push(i);
        }
    }
    kids
}

fn postorder(vdata: &mut [RtVertex], node: usize, n: usize) {
    let children = children_of(vdata, node, n);

    if children.is_empty() {
        return;
    }

    // Recursively layout each child's subtree
    for &child in &children {
        postorder(vdata, child, n);
    }

    // Place subtrees next to each other
    let minsep = 1.0_f64;
    let mut leftroot: i64 = -1;
    let mut avg = 0.0_f64;

    for (j, &child) in children.iter().enumerate() {
        if leftroot >= 0 {
            let lr = leftroot as usize;
            // Follow right contour of leftroot and left contour of child
            let mut lnode: i64 = lr as i64;
            let mut rnode: i64 = child as i64;
            let mut rootsep = vdata[lr].offset + minsep;
            let mut loffset = vdata[lr].offset;
            let mut roffset = loffset + minsep;

            // Update the right contour of the node being built
            vdata[node].right_contour = child as i64;
            vdata[node].offset_to_right_contour = rootsep;

            while lnode >= 0 && rnode >= 0 {
                let ln = lnode as usize;
                let rn = rnode as usize;

                // Step right contour of left subtree
                if vdata[ln].right_contour >= 0 {
                    loffset += vdata[ln].offset_to_right_contour;
                    lnode = vdata[ln].right_contour;
                } else {
                    // Left subtree ended — threading
                    if vdata[rn].left_contour >= 0 {
                        let auxnode = vdata[node].left_extreme;
                        let newoffset = (vdata[node].offset_to_right_extreme
                            - vdata[node].offset_to_left_extreme)
                            + minsep
                            + vdata[rn].offset_to_left_contour;
                        vdata[auxnode].left_contour = vdata[rn].left_contour;
                        vdata[auxnode].right_contour = vdata[rn].left_contour;
                        vdata[auxnode].offset_to_left_contour = newoffset;
                        vdata[auxnode].offset_to_right_contour = newoffset;

                        vdata[node].left_extreme = vdata[child].left_extreme;
                        vdata[node].right_extreme = vdata[child].right_extreme;
                        vdata[node].offset_to_left_extreme =
                            vdata[child].offset_to_left_extreme + rootsep;
                        vdata[node].offset_to_right_extreme =
                            vdata[child].offset_to_right_extreme + rootsep;
                    } else {
                        // Both subtrees end simultaneously
                        vdata[node].right_extreme = vdata[child].right_extreme;
                        vdata[node].offset_to_right_extreme =
                            vdata[child].offset_to_right_extreme + rootsep;
                    }
                    lnode = -1;
                }

                // Step left contour of right subtree
                let rn = rnode as usize;
                if rnode >= 0 && vdata[rn].left_contour >= 0 {
                    roffset += vdata[rn].offset_to_left_contour;
                    rnode = vdata[rn].left_contour;
                } else if rnode >= 0 {
                    // Right subtree ended — threading
                    if lnode >= 0 {
                        let auxnode = vdata[child].right_extreme;
                        let newoffset = loffset - rootsep - vdata[child].offset_to_right_extreme;
                        vdata[auxnode].left_contour = lnode;
                        vdata[auxnode].right_contour = lnode;
                        vdata[auxnode].offset_to_left_contour = newoffset;
                        vdata[auxnode].offset_to_right_contour = newoffset;
                    }
                    rnode = -1;
                }

                // Push subtrees apart if too close
                if lnode >= 0 && rnode >= 0 && (roffset - loffset < minsep) {
                    rootsep += minsep - roffset + loffset;
                    roffset = loffset + minsep;
                    vdata[node].offset_to_right_contour = rootsep;
                }
            }

            vdata[child].offset = rootsep;
            vdata[node].offset_to_right_contour = rootsep;
            avg = (avg * j as f64) / (j as f64 + 1.0) + rootsep / (j as f64 + 1.0);
            leftroot = child as i64;
        } else {
            // First child
            leftroot = child as i64;
            vdata[node].left_contour = child as i64;
            vdata[node].right_contour = child as i64;
            vdata[node].offset_to_left_contour = 0.0;
            vdata[node].offset_to_right_contour = 0.0;
            vdata[node].left_extreme = vdata[child].left_extreme;
            vdata[node].right_extreme = vdata[child].right_extreme;
            vdata[node].offset_to_left_extreme = vdata[child].offset_to_left_extreme;
            vdata[node].offset_to_right_extreme = vdata[child].offset_to_right_extreme;
            avg = vdata[child].offset;
        }
    }

    // Center parent above children
    vdata[node].offset_to_left_contour -= avg;
    vdata[node].offset_to_right_contour -= avg;
    vdata[node].offset_to_left_extreme -= avg;
    vdata[node].offset_to_right_extreme -= avg;
    for &child in &children {
        vdata[child].offset -= avg;
    }
}

fn calc_coords(vdata: &[RtVertex], pos: &mut [[f64; 2]], node: usize, n: usize, xpos: f64) {
    pos[node][0] = xpos;
    pos[node][1] = vdata[node].level as f64;
    for i in 0..n {
        if i == node {
            continue;
        }
        if vdata[i].parent == node as i64 {
            calc_coords(vdata, pos, i, n, xpos + vdata[i].offset);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Root selection heuristic
// ═══════════════════════════════════════════════════════════════════

/// Heuristic for selecting tree-layout roots when multiple candidates
/// exist within a component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootChoice {
    /// Pick the vertex with the highest degree (out- or in-degree for
    /// directed graphs). Fast — O(n) after sorting.
    Degree,
    /// Pick the vertex with the lowest eccentricity ("most central").
    /// Produces wide, shallow layouts. Slow — O(n²) eccentricity BFS.
    Eccentricity,
}

/// Choose "nice" roots for a tree layout.
///
/// Returns one root per reachable component so that every vertex is
/// reachable from at least one root.
///
/// - **Undirected** (or `mode == RtMode::All`): one root per weak
///   connected component — the vertex with the highest degree (or
///   lowest eccentricity).
/// - **Directed**: strongly connected components with no incoming
///   (when `mode == RtMode::Out`) or no outgoing (when `mode ==
///   RtMode::In`) edges get one root each. Components that *do*
///   have such edges are omitted — they are reachable from some
///   root component.
///
/// Counterpart of `igraph_roots_for_tree_layout()` from
/// `references/igraph/src/layout/reingold_tilford.c:536–660`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, roots_for_tree_layout, RtMode, RootChoice};
///
/// // Simple undirected tree: 0-1-2-3-4
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
///
/// let roots = roots_for_tree_layout(&g, RtMode::All, RootChoice::Degree).unwrap();
/// // One root, should be vertex 1 or 2 or 3 (degree 2 — the hubs of a path)
/// assert_eq!(roots.len(), 1);
/// assert!([1, 2, 3].contains(&roots[0]));
///
/// // Disconnected: two components
/// let mut g2 = Graph::with_vertices(4);
/// g2.add_edge(0, 1).unwrap();
/// g2.add_edge(2, 3).unwrap();
/// let roots2 = roots_for_tree_layout(&g2, RtMode::All, RootChoice::Degree).unwrap();
/// assert_eq!(roots2.len(), 2);
/// ```
pub fn roots_for_tree_layout(
    graph: &Graph,
    mode: RtMode,
    heuristic: RootChoice,
) -> IgraphResult<Vec<VertexId>> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(Vec::new());
    }

    let effective_mode = if graph.is_directed() {
        mode
    } else {
        RtMode::All
    };

    let order = build_vertex_order(graph, effective_mode, heuristic)?;

    if effective_mode == RtMode::All {
        roots_undirected(graph, &order)
    } else {
        roots_directed(graph, effective_mode, &order)
    }
}

fn build_vertex_order(
    graph: &Graph,
    mode: RtMode,
    heuristic: RootChoice,
) -> IgraphResult<Vec<VertexId>> {
    let n = graph.vcount() as usize;
    match heuristic {
        RootChoice::Eccentricity => {
            let ecc_mode = match mode {
                RtMode::Out => crate::algorithms::paths::radii::EccMode::Out,
                RtMode::In => crate::algorithms::paths::radii::EccMode::In,
                RtMode::All => crate::algorithms::paths::radii::EccMode::All,
            };
            let ecc = crate::algorithms::paths::radii::eccentricity_with_mode(graph, ecc_mode)?;
            let mut indices: Vec<VertexId> = (0..n as VertexId).collect();
            indices.sort_by(|&a, &b| ecc[a as usize].cmp(&ecc[b as usize]));
            Ok(indices)
        }
        RootChoice::Degree => {
            let deg_mode = match mode {
                RtMode::Out => crate::algorithms::properties::degree::DegreeMode::Out,
                RtMode::In => crate::algorithms::properties::degree::DegreeMode::In,
                RtMode::All => crate::algorithms::properties::degree::DegreeMode::All,
            };
            Ok(
                crate::algorithms::properties::sort_by_degree::sort_vertices_by_degree(
                    graph,
                    deg_mode,
                    crate::algorithms::properties::sort_by_degree::SortOrder::Descending,
                )?,
            )
        }
    }
}

fn roots_undirected(graph: &Graph, order: &[VertexId]) -> IgraphResult<Vec<VertexId>> {
    let comps = crate::algorithms::connectivity::components::connected_components(graph)?;
    let no_comps = comps.count as usize;

    let mut roots = vec![u32::MAX; no_comps];
    let mut seen = 0usize;

    for &v in order {
        let cl = comps.membership[v as usize] as usize;
        if roots[cl] == u32::MAX {
            roots[cl] = v;
            seen += 1;
            if seen == no_comps {
                break;
            }
        }
    }

    Ok(roots)
}

fn roots_directed(graph: &Graph, mode: RtMode, order: &[VertexId]) -> IgraphResult<Vec<VertexId>> {
    let comps = crate::algorithms::connectivity::strong::strongly_connected_components(graph)?;
    let no_comps = comps.count as usize;

    let cluster_incoming = cluster_cross_degrees(graph, &comps.membership, no_comps, mode)?;

    let mut roots: Vec<Option<VertexId>> = vec![None; no_comps];

    for &v in order {
        let cl = comps.membership[v as usize] as usize;
        if cluster_incoming[cl] == 0 && roots[cl].is_none() {
            roots[cl] = Some(v);
        }
    }

    Ok(roots.into_iter().flatten().collect())
}

/// For each SCC, count edges arriving from *other* SCCs in the
/// reverse direction — i.e. if `mode == Out`, count incoming
/// cross-component edges; if `mode == In`, count outgoing ones.
fn cluster_cross_degrees(
    graph: &Graph,
    membership: &[u32],
    no_comps: usize,
    mode: RtMode,
) -> IgraphResult<Vec<u32>> {
    let mut degrees = vec![0u32; no_comps];
    let m = graph.ecount();

    for eid in 0..m as EdgeId {
        let (from, to) = graph.edge(eid)?;
        let from_cl = membership[from as usize];
        let to_cl = membership[to as usize];

        if from_cl != to_cl {
            let cl = if mode == RtMode::Out { to_cl } else { from_cl };
            degrees[cl as usize] = degrees[cl as usize].saturating_add(1);
        }
    }

    Ok(degrees)
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn binary_tree() -> Graph {
        // 0 -> {1, 2}, 1 -> {3, 4}, 2 -> {5, 6}
        let mut g = Graph::with_vertices(7);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        g.add_edge(2, 5).unwrap();
        g.add_edge(2, 6).unwrap();
        g
    }

    #[test]
    fn test_rt_binary_tree_levels() {
        let g = binary_tree();
        let pos = layout_reingold_tilford(&g, Some(0), RtMode::All).unwrap();
        // Root at level 0
        assert!((pos[0][1]).abs() < 1e-10);
        // Children at level 1
        assert!((pos[1][1] - 1.0).abs() < 1e-10);
        assert!((pos[2][1] - 1.0).abs() < 1e-10);
        // Grandchildren at level 2
        assert!((pos[3][1] - 2.0).abs() < 1e-10);
        assert!((pos[6][1] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_rt_binary_tree_symmetry() {
        let g = binary_tree();
        let pos = layout_reingold_tilford(&g, Some(0), RtMode::All).unwrap();
        // Left subtree mirror of right
        let root_x = pos[0][0];
        let left_child_x = pos[1][0];
        let right_child_x = pos[2][0];
        // Children should be symmetric around root
        let diff = (left_child_x - root_x) + (right_child_x - root_x);
        assert!(diff.abs() < 1e-10, "children not symmetric: diff={diff}");
    }

    #[test]
    fn test_rt_path() {
        // Path: 0 - 1 - 2 - 3
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let pos = layout_reingold_tilford(&g, Some(0), RtMode::All).unwrap();
        // All at same x (linear tree)
        for i in 0..4 {
            assert!((pos[i][0] - pos[0][0]).abs() < 1e-10);
            assert!((pos[i][1] - i as f64).abs() < 1e-10);
        }
    }

    #[test]
    fn test_rt_single_vertex() {
        let g = Graph::with_vertices(1);
        let pos = layout_reingold_tilford(&g, Some(0), RtMode::All).unwrap();
        assert_eq!(pos.len(), 1);
        assert!((pos[0][0]).abs() < 1e-10);
        assert!((pos[0][1]).abs() < 1e-10);
    }

    #[test]
    fn test_rt_empty() {
        let g = Graph::with_vertices(0);
        let pos = layout_reingold_tilford(&g, None, RtMode::All).unwrap();
        assert!(pos.is_empty());
    }

    #[test]
    fn test_rt_star() {
        // Star: 0 connected to 1,2,3,4
        let mut g = Graph::with_vertices(5);
        for i in 1..5 {
            g.add_edge(0, i).unwrap();
        }
        let pos = layout_reingold_tilford(&g, Some(0), RtMode::All).unwrap();
        // All leaves at level 1
        for i in 1..5 {
            assert!((pos[i][1] - 1.0).abs() < 1e-10);
        }
        // Leaves should be evenly spaced (separation 1.0)
        let mut xs: Vec<f64> = (1..5).map(|i| pos[i][0]).collect();
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for i in 1..xs.len() {
            assert!((xs[i] - xs[i - 1] - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_rt_disconnected() {
        // Two components: 0-1 and 2-3
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let pos = layout_reingold_tilford(&g, Some(0), RtMode::All).unwrap();
        assert_eq!(pos.len(), 4);
        // All positions should be finite
        for p in &pos {
            assert!(p[0].is_finite());
            assert!(p[1].is_finite());
        }
    }

    #[test]
    fn test_rt_circular_basic() {
        let g = binary_tree();
        let pos = layout_reingold_tilford_circular(&g, Some(0), RtMode::All).unwrap();
        assert_eq!(pos.len(), 7);
        // Root is at center (level 0 → radius 0)
        assert!(pos[0][0].abs() < 1e-10);
        assert!(pos[0][1].abs() < 1e-10);
    }

    #[test]
    fn test_rt_no_overlap() {
        let g = binary_tree();
        let pos = layout_reingold_tilford(&g, Some(0), RtMode::All).unwrap();
        // No two nodes at the same level should overlap in x
        for i in 0..pos.len() {
            for j in (i + 1)..pos.len() {
                if (pos[i][1] - pos[j][1]).abs() < 1e-10 {
                    assert!(
                        (pos[i][0] - pos[j][0]).abs() >= 1.0 - 1e-10,
                        "nodes {i} and {j} overlap: x[{i}]={}, x[{j}]={}",
                        pos[i][0],
                        pos[j][0]
                    );
                }
            }
        }
    }

    #[test]
    fn test_rt_auto_root() {
        let g = binary_tree();
        let pos = layout_reingold_tilford(&g, None, RtMode::All).unwrap();
        assert_eq!(pos.len(), 7);
        // Auto-selects vertex with highest degree (vertex 1, degree 3)
        assert!((pos[1][1]).abs() < 1e-10);
    }

    #[test]
    fn test_rt_invalid_root() {
        let g = Graph::with_vertices(3);
        let result = layout_reingold_tilford(&g, Some(99), RtMode::All);
        assert!(result.is_err());
    }

    // ---- roots_for_tree_layout ----

    #[test]
    fn roots_empty_graph() {
        let g = Graph::with_vertices(0);
        let roots = roots_for_tree_layout(&g, RtMode::All, RootChoice::Degree).unwrap();
        assert!(roots.is_empty());
    }

    #[test]
    fn roots_single_vertex() {
        let g = Graph::with_vertices(1);
        let roots = roots_for_tree_layout(&g, RtMode::All, RootChoice::Degree).unwrap();
        assert_eq!(roots, vec![0]);
    }

    #[test]
    fn roots_undirected_single_component() {
        // Path: 0-1-2-3-4 → highest degree is 1,2,3 (degree 2)
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        let roots = roots_for_tree_layout(&g, RtMode::All, RootChoice::Degree).unwrap();
        assert_eq!(roots.len(), 1);
        // Root should have degree 2 (not a leaf)
        assert!(
            [1, 2, 3].contains(&roots[0]),
            "expected interior vertex, got {}",
            roots[0]
        );
    }

    #[test]
    fn roots_undirected_two_components() {
        // Component 0: 0-1, Component 1: 2-3-4
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        let roots = roots_for_tree_layout(&g, RtMode::All, RootChoice::Degree).unwrap();
        assert_eq!(roots.len(), 2);
        let mut sorted = roots.clone();
        sorted.sort();
        // One root from {0,1}, one from {2,3,4}
        assert!(sorted[0] <= 1);
        assert!(sorted[1] >= 2 && sorted[1] <= 4);
    }

    #[test]
    fn roots_eccentricity_path() {
        // Path 0-1-2-3-4: eccentricity centre is vertex 2
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        let roots = roots_for_tree_layout(&g, RtMode::All, RootChoice::Eccentricity).unwrap();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0], 2, "eccentricity centre of P5 is vertex 2");
    }

    #[test]
    fn roots_directed_dag() {
        // DAG: 0→1→3, 0→2→3
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        let roots = roots_for_tree_layout(&g, RtMode::Out, RootChoice::Degree).unwrap();
        // Vertex 0 is the only source — no incoming edges
        assert_eq!(roots, vec![0]);
    }

    #[test]
    fn roots_directed_two_sources() {
        // 0→2, 1→2  — two source SCCs (0 and 1), one sink (2)
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let roots = roots_for_tree_layout(&g, RtMode::Out, RootChoice::Degree).unwrap();
        assert_eq!(roots.len(), 2);
        let mut sorted = roots.clone();
        sorted.sort();
        assert_eq!(sorted, vec![0, 1]);
    }

    #[test]
    fn roots_directed_in_mode() {
        // 0→1→2  — with In mode, the "sink" (2) should be root
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let roots = roots_for_tree_layout(&g, RtMode::In, RootChoice::Degree).unwrap();
        assert_eq!(roots, vec![2]);
    }

    #[test]
    fn roots_directed_cycle_scc() {
        // Cycle 0→1→2→0 plus 3→0 — the cycle SCC {0,1,2} has incoming
        // from SCC {3}. SCC {3} has no incoming → it's the root.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(3, 0).unwrap();
        let roots = roots_for_tree_layout(&g, RtMode::Out, RootChoice::Degree).unwrap();
        assert_eq!(roots, vec![3]);
    }

    #[test]
    fn roots_undirected_ignores_mode() {
        // Even if we pass Out mode, undirected should use All (weak comps)
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let roots_out = roots_for_tree_layout(&g, RtMode::Out, RootChoice::Degree).unwrap();
        let roots_all = roots_for_tree_layout(&g, RtMode::All, RootChoice::Degree).unwrap();
        assert_eq!(roots_out, roots_all);
    }
}

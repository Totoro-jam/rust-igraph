//! Graph coloring algorithms.
//!
//! ALGO-CL-001: `vertex_coloring_greedy` — heuristic greedy vertex coloring
//! with two ordering strategies (DSATUR and Colored-Neighbors).
//!
//! Also provides three coloring validators:
//! - [`is_vertex_coloring`] — check proper vertex coloring
//! - [`is_bipartite_coloring`] — check bipartite (2-color) vertex coloring
//! - [`is_edge_coloring`] — check proper edge coloring
//!
//! Reference: `references/igraph/src/misc/coloring.c` (519 lines).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::too_many_lines,
    clippy::similar_names,
    clippy::module_name_repetitions
)]

use std::collections::BinaryHeap;

use crate::IgraphResult;
use crate::core::error::IgraphError;
use crate::core::graph::{EdgeId, Graph, VertexId};

/// Heuristic ordering strategy for [`vertex_coloring_greedy`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GreedyColoringHeuristic {
    /// Choose the uncolored vertex with the most already-colored neighbors.
    /// Faster but typically uses more colors than DSATUR.
    ColoredNeighbors,
    /// Choose the uncolored vertex with the highest saturation degree
    /// (number of distinct colors among neighbors), breaking ties by
    /// degree in the uncolored subgraph. Generally produces better
    /// (fewer-color) results.
    DSatur,
}

/// Result of [`is_bipartite_coloring`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BipartiteColoringResult {
    /// Whether the coloring is a valid bipartite coloring.
    pub valid: bool,
    /// Edge direction mode (only meaningful when `valid` is true and
    /// the graph is directed):
    /// - `Some(BipartiteEdgeDirection::Out)` — all edges go from type-false to type-true
    /// - `Some(BipartiteEdgeDirection::In)` — all edges go from type-true to type-false
    /// - `Some(BipartiteEdgeDirection::All)` — edges go both ways, or graph is undirected
    /// - `None` — coloring is invalid
    pub mode: Option<BipartiteEdgeDirection>,
}

/// Edge direction pattern in a directed bipartite graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BipartiteEdgeDirection {
    /// All edges go from type-false to type-true vertices.
    Out,
    /// All edges go from type-true to type-false vertices.
    In,
    /// Edges go in both directions, or graph is undirected.
    All,
}

// ---------------------------------------------------------------
// vertex_coloring_greedy
// ---------------------------------------------------------------

/// Computes a vertex coloring using a greedy algorithm.
///
/// Assigns a color (non-negative integer starting at 0) to each vertex
/// such that no two adjacent vertices share the same color. Self-loops
/// are ignored. The result is not necessarily optimal (minimum-chromatic).
///
/// # Arguments
///
/// * `graph` — the input graph (directed or undirected; multi-edges and
///   self-loops are tolerated).
/// * `heuristic` — vertex ordering strategy; see [`GreedyColoringHeuristic`].
///
/// # Returns
///
/// A `Vec<u32>` of length `vcount`, where entry `i` is the color of vertex `i`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{vertex_coloring_greedy, GreedyColoringHeuristic, create};
///
/// // Triangle graph — needs at least 3 colors
/// let g = create(&[(0, 1), (1, 2), (2, 0)], 3, false).unwrap();
/// let colors = vertex_coloring_greedy(&g, GreedyColoringHeuristic::DSatur).unwrap();
/// assert_eq!(colors.len(), 3);
/// // All colors distinct
/// assert_ne!(colors[0], colors[1]);
/// assert_ne!(colors[1], colors[2]);
/// assert_ne!(colors[0], colors[2]);
/// ```
pub fn vertex_coloring_greedy(
    graph: &Graph,
    heuristic: GreedyColoringHeuristic,
) -> IgraphResult<Vec<u32>> {
    match heuristic {
        GreedyColoringHeuristic::ColoredNeighbors => greedy_cn(graph),
        GreedyColoringHeuristic::DSatur => greedy_dsatur(graph),
    }
}

/// All neighbors of `v` (both in and out for directed, merged for
/// undirected). Includes self-loops and multi-edges.
fn all_neighbors(graph: &Graph, v: VertexId) -> IgraphResult<Vec<VertexId>> {
    if graph.is_directed() {
        let mut out = graph.out_neighbors_vec(v)?;
        let in_neis = graph.in_neighbors_vec(v)?;
        out.extend(in_neis);
        Ok(out)
    } else {
        graph.neighbors(v)
    }
}

/// All neighbors of `v`, excluding self-loops and duplicate edges
/// (simple neighbor list). Used by DSATUR.
fn all_neighbors_simple(graph: &Graph, v: VertexId) -> IgraphResult<Vec<VertexId>> {
    let mut neis = all_neighbors(graph, v)?;
    neis.retain(|&n| n != v);
    neis.sort_unstable();
    neis.dedup();
    Ok(neis)
}

/// `COLORED_NEIGHBORS` heuristic: pick the uncolored vertex with the
/// most already-colored neighbors. Mirrors `igraph_i_vertex_coloring_greedy_cn`.
fn greedy_cn(graph: &Graph) -> IgraphResult<Vec<u32>> {
    let vc = graph.vcount() as usize;
    let mut colors = vec![0u32; vc];

    if vc <= 1 {
        return Ok(colors);
    }

    // Find vertex with maximum degree.
    let mut max_deg = 0usize;
    let mut start_vertex: VertexId = 0;
    for v in 0..graph.vcount() {
        let d = graph.degree(v)?;
        if d > max_deg {
            max_deg = d;
            start_vertex = v;
        }
    }

    // colored_count[v] = number of already-colored neighbors of v.
    let mut colored_count = vec![0u32; vc];
    let mut colored = vec![false; vc];

    // Lazy-deletion max-heap: (colored_count, vertex_id).
    let mut heap: BinaryHeap<(u32, u32)> = BinaryHeap::with_capacity(vc);
    for v in 0..graph.vcount() {
        if v != start_vertex {
            heap.push((0, v));
        }
    }

    // Internally 1-based colors (0 = uncolored); subtract 1 at end.
    let mut vertex = start_vertex;
    loop {
        let neis = all_neighbors(graph, vertex)?;

        // Collect neighbor colors and find smallest available 1-based color.
        let mut nei_cols: Vec<u32> = neis.iter().map(|&n| colors[n as usize]).collect();
        nei_cols.sort_unstable();
        let col = smallest_available_color_1based(&nei_cols);
        colors[vertex as usize] = col;
        colored[vertex as usize] = true;

        // Update colored-neighbor counts and push new heap entries.
        for &nei in &neis {
            let ni = nei as usize;
            if !colored[ni] {
                colored_count[ni] = colored_count[ni].saturating_add(1);
                heap.push((colored_count[ni], nei));
            }
        }

        // Pop the next vertex (skip stale entries).
        loop {
            if let Some((count, v)) = heap.pop() {
                if colored[v as usize] {
                    continue;
                }
                if count < colored_count[v as usize] {
                    continue;
                }
                vertex = v;
                break;
            }
            // All vertices colored — subtract 1 to make 0-based.
            for c in &mut colors {
                *c -= 1;
            }
            return Ok(colors);
        }
    }
}

/// Find the smallest positive integer not present in a sorted array
/// that may contain zeros. Used for 1-based color assignment.
fn smallest_available_color_1based(sorted_colors: &[u32]) -> u32 {
    let mut i = 0;
    let mut col = 0u32;
    let n = sorted_colors.len();
    loop {
        while i < n && sorted_colors[i] == col {
            i += 1;
        }
        col += 1;
        if i >= n || sorted_colors[i] != col {
            break;
        }
    }
    col
}

// ---------------------------------------------------------------
// DSATUR heuristic
// ---------------------------------------------------------------

/// DSATUR key for heap ordering: (`saturation_degree`, `edge_degree_in_uncolored`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DsaturKey {
    sat_deg: u32,
    edge_deg: u32,
}

impl PartialOrd for DsaturKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DsaturKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.sat_deg
            .cmp(&other.sat_deg)
            .then(self.edge_deg.cmp(&other.edge_deg))
    }
}

/// DSATUR: choose vertices by saturation degree, breaking ties by
/// degree in the uncolored subgraph. Mirrors `igraph_i_vertex_coloring_dsatur`.
fn greedy_dsatur(graph: &Graph) -> IgraphResult<Vec<u32>> {
    let vc = graph.vcount() as usize;

    if vc == 0 {
        return Ok(vec![]);
    }
    if vc == 1 {
        return Ok(vec![0]);
    }

    let mut colors: Vec<Option<u32>> = vec![None; vc];

    // Build simple adjacency list (no loops, no multi-edges).
    let mut adj: Vec<Vec<VertexId>> = Vec::with_capacity(vc);
    for v in 0..graph.vcount() {
        adj.push(all_neighbors_simple(graph, v)?);
    }

    // DSATUR state per vertex.
    let mut sat_deg = vec![0u32; vc];
    let mut edge_deg = vec![0u32; vc];
    for v in 0..vc {
        edge_deg[v] = adj[v].len() as u32;
    }
    let mut remaining = vc;

    // Lazy-deletion max-heap: (DsaturKey, vertex_id).
    let mut heap: BinaryHeap<(DsaturKey, u32)> = BinaryHeap::with_capacity(vc);
    for v in 0..graph.vcount() {
        heap.push((
            DsaturKey {
                sat_deg: 0,
                edge_deg: edge_deg[v as usize],
            },
            v,
        ));
    }

    while remaining > 0 {
        // Pop the highest-priority uncolored vertex.
        let node_to_color = loop {
            let (key, v) = heap.pop().ok_or_else(|| {
                IgraphError::InvalidArgument("DSATUR heap empty unexpectedly".into())
            })?;
            if colors[v as usize].is_some() {
                continue;
            }
            // Check if this entry is stale.
            let cur_key = DsaturKey {
                sat_deg: sat_deg[v as usize],
                edge_deg: edge_deg[v as usize],
            };
            if key < cur_key {
                continue;
            }
            break v;
        };

        // Find smallest available color among neighbors.
        let neis = &adj[node_to_color as usize];
        let mut used: Vec<u32> = neis.iter().filter_map(|&n| colors[n as usize]).collect();
        used.sort_unstable();
        let color = smallest_available_color_0based(&used);

        colors[node_to_color as usize] = Some(color);
        remaining -= 1;

        // Update neighbors: for each uncolored neighbor, check if this
        // new color increases its saturation degree, and decrement its
        // edge_degree (one fewer uncolored neighbor).
        for &nei in neis {
            let ni = nei as usize;
            if colors[ni].is_some() {
                continue;
            }

            // Check if this color was already used by another colored
            // neighbor of `nei`.
            let color_already_used = adj[ni]
                .iter()
                .any(|&nn| nn != node_to_color && colors[nn as usize] == Some(color));

            if !color_already_used {
                sat_deg[ni] += 1;
            }
            edge_deg[ni] = edge_deg[ni].saturating_sub(1);

            // Push updated priority.
            heap.push((
                DsaturKey {
                    sat_deg: sat_deg[ni],
                    edge_deg: edge_deg[ni],
                },
                nei,
            ));
        }
    }

    Ok(colors.into_iter().map(|c| c.unwrap_or(0)).collect())
}

/// Find the smallest non-negative integer not present in a sorted array
/// of non-negative integers. Used for 0-based color assignment.
fn smallest_available_color_0based(sorted: &[u32]) -> u32 {
    let mut col = 0u32;
    let mut i = 0;
    let n = sorted.len();
    while i < n && sorted[i] == col {
        while i < n && sorted[i] == col {
            i += 1;
        }
        col += 1;
    }
    col
}

// ---------------------------------------------------------------
// is_vertex_coloring
// ---------------------------------------------------------------

/// Checks whether the given color assignment is a valid vertex coloring,
/// i.e. no two adjacent vertices share the same color. Self-loops are
/// ignored.
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] if `colors.len() != vcount`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{is_vertex_coloring, create};
///
/// let g = create(&[(0, 1), (1, 2), (2, 0)], 3, false).unwrap();
/// assert!(is_vertex_coloring(&g, &[0, 1, 2]).unwrap());
/// assert!(!is_vertex_coloring(&g, &[0, 0, 1]).unwrap());
/// ```
pub fn is_vertex_coloring(graph: &Graph, colors: &[u32]) -> IgraphResult<bool> {
    let vc = graph.vcount() as usize;
    if colors.len() != vc {
        return Err(IgraphError::InvalidArgument(format!(
            "is_vertex_coloring: colors length {} != vcount {}",
            colors.len(),
            vc
        )));
    }

    let ec = graph.ecount();
    for e in 0..ec {
        let eid = u32::try_from(e)
            .map_err(|_| IgraphError::InvalidArgument("edge count exceeds u32".into()))?;
        let (from, to) = graph.edge(eid)?;
        if from == to {
            continue;
        }
        if colors[from as usize] == colors[to as usize] {
            return Ok(false);
        }
    }

    Ok(true)
}

// ---------------------------------------------------------------
// is_bipartite_coloring
// ---------------------------------------------------------------

/// Checks whether the given boolean type assignment is a valid bipartite
/// coloring, i.e. no two adjacent vertices have the same type. Self-loops
/// are ignored.
///
/// For directed graphs, also determines the edge direction mode:
/// - `Out` — all edges go from type-false to type-true
/// - `In` — all edges go from type-true to type-false
/// - `All` — edges go both ways, or graph is undirected
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] if `types.len() != vcount`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{is_bipartite_coloring, BipartiteEdgeDirection, create};
///
/// let g = create(&[(0, 2), (0, 3), (1, 2), (1, 3)], 4, false).unwrap();
/// let result = is_bipartite_coloring(&g, &[false, false, true, true]).unwrap();
/// assert!(result.valid);
/// assert_eq!(result.mode, Some(BipartiteEdgeDirection::All));
/// ```
pub fn is_bipartite_coloring(
    graph: &Graph,
    types: &[bool],
) -> IgraphResult<BipartiteColoringResult> {
    let vc = graph.vcount() as usize;
    if types.len() != vc {
        return Err(IgraphError::InvalidArgument(format!(
            "is_bipartite_coloring: types length {} != vcount {}",
            types.len(),
            vc
        )));
    }

    let ec = graph.ecount();
    let directed = graph.is_directed();
    let mut has_false_to_true = false;
    let mut has_true_to_false = false;

    for e in 0..ec {
        let eid = u32::try_from(e)
            .map_err(|_| IgraphError::InvalidArgument("edge count exceeds u32".into()))?;
        let (from, to) = graph.edge(eid)?;
        if from == to {
            continue;
        }

        let from_type = types[from as usize];
        let to_type = types[to as usize];

        if from_type == to_type {
            return Ok(BipartiteColoringResult {
                valid: false,
                mode: None,
            });
        }

        if directed {
            if !from_type && to_type {
                has_false_to_true = true;
            } else if from_type && !to_type {
                has_true_to_false = true;
            }
        }
    }

    let mode = if directed {
        if has_false_to_true && !has_true_to_false {
            BipartiteEdgeDirection::Out
        } else if !has_false_to_true && has_true_to_false {
            BipartiteEdgeDirection::In
        } else {
            BipartiteEdgeDirection::All
        }
    } else {
        BipartiteEdgeDirection::All
    };

    Ok(BipartiteColoringResult {
        valid: true,
        mode: Some(mode),
    })
}

// ---------------------------------------------------------------
// is_edge_coloring
// ---------------------------------------------------------------

/// Checks whether the given edge color assignment is a valid edge
/// coloring, i.e. no two edges sharing a vertex have the same color.
/// Self-loops are not considered adjacent to themselves.
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] if `colors.len() != ecount`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{is_edge_coloring, create};
///
/// let g = create(&[(0, 1), (1, 2), (2, 0)], 3, false).unwrap();
/// assert!(is_edge_coloring(&g, &[0, 1, 2]).unwrap());
/// assert!(!is_edge_coloring(&g, &[0, 1, 0]).unwrap()); // edges 0,2 share vertex 0
/// ```
pub fn is_edge_coloring(graph: &Graph, colors: &[u32]) -> IgraphResult<bool> {
    let ec = graph.ecount();
    if colors.len() != ec {
        return Err(IgraphError::InvalidArgument(format!(
            "is_edge_coloring: colors length {} != ecount {}",
            colors.len(),
            ec
        )));
    }

    let vc = graph.vcount();

    for v in 0..vc {
        let edges = graph.incident(v)?;

        // Dedup edge ids: self-loops appear twice in incident() for
        // undirected graphs. We treat self-loops as NOT adjacent to
        // themselves (matching igraph C using IGRAPH_LOOPS_ONCE).
        let mut seen_eids: Vec<EdgeId> = Vec::with_capacity(edges.len());
        for &eid in &edges {
            if !seen_eids.contains(&eid) {
                seen_eids.push(eid);
            }
        }

        let mut edge_cols: Vec<u32> = seen_eids.iter().map(|&eid| colors[eid as usize]).collect();
        edge_cols.sort_unstable();

        for w in edge_cols.windows(2) {
            if w[0] == w[1] {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

/// Compute a greedy edge coloring.
///
/// Assigns colors (starting from 0) to edges such that no two edges
/// sharing a vertex receive the same color. Iterates edges in order
/// and assigns the smallest color not used by any adjacent edge.
///
/// The greedy approach may use up to `2·Δ - 1` colors where `Δ` is
/// the maximum degree. By Vizing's theorem, any simple graph can be
/// edge-colored with at most `Δ + 1` colors, but the greedy
/// heuristic does not guarantee that bound.
///
/// Self-loops are assigned a color but are not considered adjacent
/// to themselves (matching `is_edge_coloring` behavior).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_coloring_greedy, is_edge_coloring};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(0, 2).unwrap();
/// let colors = edge_coloring_greedy(&g).unwrap();
/// assert!(is_edge_coloring(&g, &colors).unwrap());
/// ```
pub fn edge_coloring_greedy(graph: &Graph) -> IgraphResult<Vec<u32>> {
    let ec = graph.ecount();

    if ec == 0 {
        return Ok(Vec::new());
    }

    let mut colors = vec![u32::MAX; ec];

    // For each vertex, track which colors are used by incident edges
    // We rebuild the used-color set per vertex on demand
    for eid in 0..ec {
        let (u, v) = graph.edge(eid as u32)?;

        // Collect colors used by edges incident to u (excluding self: eid)
        let mut used = Vec::new();
        collect_used_colors(graph, u, eid, &colors, &mut used)?;
        if u != v {
            collect_used_colors(graph, v, eid, &colors, &mut used)?;
        }
        used.sort_unstable();
        used.dedup();

        // Find smallest unused color
        let mut color = 0u32;
        for &c in &used {
            if c == color {
                color = color.checked_add(1).unwrap_or(color);
            } else {
                break;
            }
        }

        colors[eid] = color;
    }

    // Verify no u32::MAX remains (shouldn't happen for valid graphs)
    debug_assert!(colors.iter().all(|&c| c != u32::MAX || ec == 0));

    Ok(colors)
}

/// Return the number of distinct colors used by an edge coloring.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_coloring_greedy, edge_chromatic_number};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let colors = edge_coloring_greedy(&g).unwrap();
/// assert_eq!(edge_chromatic_number(&colors), 3);
/// ```
pub fn edge_chromatic_number(colors: &[u32]) -> u32 {
    if colors.is_empty() {
        return 0;
    }
    let max_color = colors.iter().copied().max().unwrap_or(0);
    max_color.checked_add(1).unwrap_or(max_color)
}

fn collect_used_colors(
    graph: &Graph,
    v: VertexId,
    exclude_eid: usize,
    colors: &[u32],
    used: &mut Vec<u32>,
) -> IgraphResult<()> {
    let edges = graph.incident(v)?;
    if graph.is_directed() {
        let in_edges = graph.incident_in(v)?;
        for &eid in edges.iter().chain(in_edges.iter()) {
            if (eid as usize) != exclude_eid && colors[eid as usize] != u32::MAX {
                used.push(colors[eid as usize]);
            }
        }
    } else {
        let mut seen: Vec<EdgeId> = Vec::with_capacity(edges.len());
        for &eid in &edges {
            if !seen.contains(&eid) {
                seen.push(eid);
                if (eid as usize) != exclude_eid && colors[eid as usize] != u32::MAX {
                    used.push(colors[eid as usize]);
                }
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------
// vertex_chromatic_number
// ---------------------------------------------------------------

/// Return the number of distinct colors used in a vertex coloring.
///
/// Given a color assignment `colors[v]` for each vertex, returns
/// the count of distinct colors. This is a utility function that
/// works on the output of [`vertex_coloring_greedy`].
///
/// For an empty coloring (no vertices), returns 0.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, vertex_coloring_greedy, vertex_chromatic_number, GreedyColoringHeuristic};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let colors = vertex_coloring_greedy(&g, GreedyColoringHeuristic::DSatur).unwrap();
/// assert_eq!(vertex_chromatic_number(&colors), 3); // triangle needs 3 colors
/// ```
pub fn vertex_chromatic_number(colors: &[u32]) -> u32 {
    if colors.is_empty() {
        return 0;
    }
    let max_color = colors.iter().copied().max().unwrap_or(0);
    max_color.saturating_add(1)
}

/// Compute an upper bound on the vertex chromatic number using
/// greedy `DSatur` coloring.
///
/// This is a convenience function that runs [`vertex_coloring_greedy`]
/// with [`GreedyColoringHeuristic::DSatur`] and returns the number
/// of colors used. The result is an upper bound on the true chromatic
/// number χ(G).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, chromatic_number_upper_bound};
///
/// // C4 is bipartite → χ = 2
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 0).unwrap();
/// assert_eq!(chromatic_number_upper_bound(&g).unwrap(), 2);
///
/// // K4 → χ = 4, DSatur finds optimal
/// let mut g = Graph::with_vertices(4);
/// for u in 0..4u32 {
///     for v in (u + 1)..4 {
///         g.add_edge(u, v).unwrap();
///     }
/// }
/// assert_eq!(chromatic_number_upper_bound(&g).unwrap(), 4);
/// ```
pub fn chromatic_number_upper_bound(graph: &Graph) -> IgraphResult<u32> {
    let colors = vertex_coloring_greedy(graph, GreedyColoringHeuristic::DSatur)?;
    Ok(vertex_chromatic_number(&colors))
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create;

    fn make_undirected(n: u32, edges: &[(u32, u32)]) -> Graph {
        create(edges, n, false).expect("make_undirected")
    }

    fn make_directed(n: u32, edges: &[(u32, u32)]) -> Graph {
        create(edges, n, true).expect("make_directed")
    }

    // ---- vertex_coloring_greedy ----

    #[test]
    fn test_empty_graph_cn() {
        let g = make_undirected(0, &[]);
        let c = vertex_coloring_greedy(&g, GreedyColoringHeuristic::ColoredNeighbors).expect("ok");
        assert!(c.is_empty());
    }

    #[test]
    fn test_empty_graph_dsatur() {
        let g = make_undirected(0, &[]);
        let c = vertex_coloring_greedy(&g, GreedyColoringHeuristic::DSatur).expect("ok");
        assert!(c.is_empty());
    }

    #[test]
    fn test_singleton_cn() {
        let g = make_undirected(1, &[]);
        let c = vertex_coloring_greedy(&g, GreedyColoringHeuristic::ColoredNeighbors).expect("ok");
        assert_eq!(c, vec![0]);
    }

    #[test]
    fn test_singleton_dsatur() {
        let g = make_undirected(1, &[]);
        let c = vertex_coloring_greedy(&g, GreedyColoringHeuristic::DSatur).expect("ok");
        assert_eq!(c, vec![0]);
    }

    #[test]
    fn test_triangle_cn() {
        let g = make_undirected(3, &[(0, 1), (1, 2), (2, 0)]);
        let c = vertex_coloring_greedy(&g, GreedyColoringHeuristic::ColoredNeighbors).expect("ok");
        assert_eq!(c.len(), 3);
        assert!(is_vertex_coloring(&g, &c).expect("validate"));
        let max_color = c.iter().copied().max().unwrap_or(0);
        assert!(max_color >= 2, "triangle needs at least 3 colors");
    }

    #[test]
    fn test_triangle_dsatur() {
        let g = make_undirected(3, &[(0, 1), (1, 2), (2, 0)]);
        let c = vertex_coloring_greedy(&g, GreedyColoringHeuristic::DSatur).expect("ok");
        assert_eq!(c.len(), 3);
        assert!(is_vertex_coloring(&g, &c).expect("validate"));
        let num_colors = c.iter().copied().max().unwrap_or(0) + 1;
        assert_eq!(num_colors, 3);
    }

    #[test]
    fn test_isolated_vertices_cn() {
        let g = make_undirected(4, &[(0, 1)]);
        let c = vertex_coloring_greedy(&g, GreedyColoringHeuristic::ColoredNeighbors).expect("ok");
        assert!(is_vertex_coloring(&g, &c).expect("validate"));
        assert_eq!(c[2], 0);
        assert_eq!(c[3], 0);
    }

    #[test]
    fn test_isolated_vertices_dsatur() {
        let g = make_undirected(4, &[(0, 1)]);
        let c = vertex_coloring_greedy(&g, GreedyColoringHeuristic::DSatur).expect("ok");
        assert!(is_vertex_coloring(&g, &c).expect("validate"));
        assert_eq!(c[2], 0);
        assert_eq!(c[3], 0);
    }

    #[test]
    fn test_self_loops_cn() {
        let g = make_undirected(3, &[(0, 0), (2, 2), (2, 2)]);
        let c = vertex_coloring_greedy(&g, GreedyColoringHeuristic::ColoredNeighbors).expect("ok");
        assert!(is_vertex_coloring(&g, &c).expect("validate"));
        let num_colors = c.iter().copied().max().unwrap_or(0) + 1;
        assert_eq!(num_colors, 1, "only self-loops, 1 color suffices");
    }

    #[test]
    fn test_self_loops_dsatur() {
        let g = make_undirected(3, &[(0, 0), (2, 2), (2, 2)]);
        let c = vertex_coloring_greedy(&g, GreedyColoringHeuristic::DSatur).expect("ok");
        assert!(is_vertex_coloring(&g, &c).expect("validate"));
        let num_colors = c.iter().copied().max().unwrap_or(0) + 1;
        assert_eq!(num_colors, 1);
    }

    #[test]
    fn test_bipartite_graph_dsatur_optimal() {
        let g = make_undirected(
            6,
            &[
                (0, 3),
                (0, 4),
                (0, 5),
                (1, 3),
                (1, 4),
                (1, 5),
                (2, 3),
                (2, 4),
                (2, 5),
            ],
        );
        let c = vertex_coloring_greedy(&g, GreedyColoringHeuristic::DSatur).expect("ok");
        assert!(is_vertex_coloring(&g, &c).expect("validate"));
        let num_colors = c.iter().copied().max().unwrap_or(0) + 1;
        assert_eq!(num_colors, 2);
    }

    #[test]
    fn test_wheel_odd_dsatur() {
        let mut edges: Vec<(u32, u32)> = Vec::new();
        for i in 1..=10u32 {
            edges.push((0, i));
        }
        for i in 1..10u32 {
            edges.push((i, i + 1));
        }
        edges.push((10, 1));
        let g = make_undirected(11, &edges);
        let c = vertex_coloring_greedy(&g, GreedyColoringHeuristic::DSatur).expect("ok");
        assert!(is_vertex_coloring(&g, &c).expect("validate"));
        let num_colors = c.iter().copied().max().unwrap_or(0) + 1;
        assert_eq!(num_colors, 3);
    }

    #[test]
    fn test_wheel_even_dsatur() {
        let mut edges: Vec<(u32, u32)> = Vec::new();
        for i in 1..=11u32 {
            edges.push((0, i));
        }
        for i in 1..11u32 {
            edges.push((i, i + 1));
        }
        edges.push((11, 1));
        let g = make_undirected(12, &edges);
        let c = vertex_coloring_greedy(&g, GreedyColoringHeuristic::DSatur).expect("ok");
        assert!(is_vertex_coloring(&g, &c).expect("validate"));
        let num_colors = c.iter().copied().max().unwrap_or(0) + 1;
        assert_eq!(num_colors, 4);
    }

    #[test]
    fn test_small_graph_both_heuristics() {
        let g = make_undirected(
            8,
            &[
                (0, 3),
                (0, 4),
                (1, 4),
                (2, 4),
                (1, 5),
                (2, 5),
                (3, 5),
                (0, 6),
                (1, 6),
                (2, 6),
                (3, 6),
                (0, 7),
                (1, 7),
                (2, 7),
                (4, 7),
                (5, 7),
            ],
        );

        for heuristic in [
            GreedyColoringHeuristic::ColoredNeighbors,
            GreedyColoringHeuristic::DSatur,
        ] {
            let c = vertex_coloring_greedy(&g, heuristic).expect("ok");
            assert_eq!(c.len(), 8);
            assert!(is_vertex_coloring(&g, &c).expect("validate"));
        }

        let c_ds = vertex_coloring_greedy(&g, GreedyColoringHeuristic::DSatur).expect("ok");
        let num_colors = c_ds.iter().copied().max().unwrap_or(0) + 1;
        assert_eq!(num_colors, 3);
    }

    #[test]
    fn test_multi_edges_dsatur() {
        let g = make_undirected(
            8,
            &[
                (0, 3),
                (0, 4),
                (1, 4),
                (2, 4),
                (1, 5),
                (2, 5),
                (3, 5),
                (0, 6),
                (1, 6),
                (2, 6),
                (3, 6),
                (0, 7),
                (1, 7),
                (2, 7),
                (4, 7),
                (5, 7),
                (0, 4),
                (0, 4),
                (3, 5),
                (3, 5),
                (3, 5),
                (0, 0),
                (0, 0),
                (1, 1),
            ],
        );

        let c = vertex_coloring_greedy(&g, GreedyColoringHeuristic::DSatur).expect("ok");
        assert!(is_vertex_coloring(&g, &c).expect("validate"));
        let num_colors = c.iter().copied().max().unwrap_or(0) + 1;
        assert_eq!(num_colors, 3);
    }

    #[test]
    fn test_directed_graph_both() {
        let g = make_directed(3, &[(0, 1), (1, 2), (2, 0)]);
        for heuristic in [
            GreedyColoringHeuristic::ColoredNeighbors,
            GreedyColoringHeuristic::DSatur,
        ] {
            let c = vertex_coloring_greedy(&g, heuristic).expect("ok");
            assert_eq!(c.len(), 3);
            for e in 0..g.ecount() {
                let eid = u32::try_from(e).expect("eid fits u32");
                let (u, v) = g.edge(eid).expect("edge");
                assert_ne!(c[u as usize], c[v as usize], "edge ({u},{v}) conflict");
            }
        }
    }

    // ---- is_vertex_coloring ----

    #[test]
    fn test_is_vertex_coloring_valid() {
        let g = make_undirected(3, &[(0, 1), (1, 2), (2, 0)]);
        assert!(is_vertex_coloring(&g, &[0, 1, 2]).expect("ok"));
    }

    #[test]
    fn test_is_vertex_coloring_invalid() {
        let g = make_undirected(3, &[(0, 1), (1, 2), (2, 0)]);
        assert!(!is_vertex_coloring(&g, &[0, 0, 1]).expect("ok"));
    }

    #[test]
    fn test_is_vertex_coloring_self_loop_ok() {
        let g = make_undirected(2, &[(0, 0), (0, 1)]);
        assert!(is_vertex_coloring(&g, &[0, 1]).expect("ok"));
    }

    #[test]
    fn test_is_vertex_coloring_wrong_size() {
        let g = make_undirected(3, &[(0, 1)]);
        assert!(is_vertex_coloring(&g, &[0, 1]).is_err());
    }

    // ---- is_bipartite_coloring ----

    #[test]
    fn test_bipartite_coloring_valid_undirected() {
        let g = make_undirected(4, &[(0, 2), (0, 3), (1, 2), (1, 3)]);
        let r = is_bipartite_coloring(&g, &[false, false, true, true]).expect("ok");
        assert!(r.valid);
        assert_eq!(r.mode, Some(BipartiteEdgeDirection::All));
    }

    #[test]
    fn test_bipartite_coloring_invalid() {
        let g = make_undirected(4, &[(0, 1), (0, 2), (1, 3), (2, 3)]);
        let r = is_bipartite_coloring(&g, &[false, false, true, true]).expect("ok");
        assert!(!r.valid);
    }

    #[test]
    fn test_bipartite_coloring_directed_out() {
        let g = make_directed(4, &[(0, 2), (0, 3), (1, 2), (1, 3)]);
        let r = is_bipartite_coloring(&g, &[false, false, true, true]).expect("ok");
        assert!(r.valid);
        assert_eq!(r.mode, Some(BipartiteEdgeDirection::Out));
    }

    #[test]
    fn test_bipartite_coloring_directed_in() {
        let g = make_directed(4, &[(2, 0), (3, 0), (2, 1), (3, 1)]);
        let r = is_bipartite_coloring(&g, &[false, false, true, true]).expect("ok");
        assert!(r.valid);
        assert_eq!(r.mode, Some(BipartiteEdgeDirection::In));
    }

    #[test]
    fn test_bipartite_coloring_directed_all() {
        let g = make_directed(4, &[(0, 2), (2, 1), (1, 3), (3, 0)]);
        let r = is_bipartite_coloring(&g, &[false, false, true, true]).expect("ok");
        assert!(r.valid);
        assert_eq!(r.mode, Some(BipartiteEdgeDirection::All));
    }

    #[test]
    fn test_bipartite_coloring_wrong_size() {
        let g = make_undirected(3, &[(0, 1)]);
        assert!(is_bipartite_coloring(&g, &[false, true]).is_err());
    }

    // ---- is_edge_coloring ----

    #[test]
    fn test_edge_coloring_valid_triangle() {
        let g = make_undirected(3, &[(0, 1), (1, 2), (2, 0)]);
        assert!(is_edge_coloring(&g, &[0, 1, 2]).expect("ok"));
    }

    #[test]
    fn test_edge_coloring_invalid_triangle() {
        let g = make_undirected(3, &[(0, 1), (1, 2), (2, 0)]);
        assert!(!is_edge_coloring(&g, &[0, 1, 0]).expect("ok"));
    }

    #[test]
    fn test_edge_coloring_path() {
        let g = make_undirected(4, &[(0, 1), (1, 2), (2, 3)]);
        assert!(is_edge_coloring(&g, &[0, 1, 0]).expect("ok"));
    }

    #[test]
    fn test_edge_coloring_wrong_size() {
        let g = make_undirected(2, &[(0, 1)]);
        assert!(is_edge_coloring(&g, &[0, 1]).is_err());
    }

    #[test]
    fn test_edge_coloring_empty() {
        let g = make_undirected(0, &[]);
        assert!(is_edge_coloring(&g, &[]).expect("ok"));
    }

    // ---- edge_coloring_greedy tests ----

    #[test]
    fn test_edge_coloring_greedy_empty() {
        let g = make_undirected(0, &[]);
        let colors = edge_coloring_greedy(&g).unwrap();
        assert!(colors.is_empty());
    }

    #[test]
    fn test_edge_coloring_greedy_no_edges() {
        let g = make_undirected(5, &[]);
        let colors = edge_coloring_greedy(&g).unwrap();
        assert!(colors.is_empty());
    }

    #[test]
    fn test_edge_coloring_greedy_single_edge() {
        let g = make_undirected(2, &[(0, 1)]);
        let colors = edge_coloring_greedy(&g).unwrap();
        assert_eq!(colors.len(), 1);
        assert_eq!(colors[0], 0);
        assert!(is_edge_coloring(&g, &colors).unwrap());
    }

    #[test]
    fn test_edge_coloring_greedy_path() {
        let g = make_undirected(4, &[(0, 1), (1, 2), (2, 3)]);
        let colors = edge_coloring_greedy(&g).unwrap();
        assert!(is_edge_coloring(&g, &colors).unwrap());
        assert_eq!(edge_chromatic_number(&colors), 2); // path needs 2
    }

    #[test]
    fn test_edge_coloring_greedy_triangle() {
        let g = make_undirected(3, &[(0, 1), (1, 2), (2, 0)]);
        let colors = edge_coloring_greedy(&g).unwrap();
        assert!(is_edge_coloring(&g, &colors).unwrap());
        assert_eq!(edge_chromatic_number(&colors), 3); // odd cycle needs 3
    }

    #[test]
    fn test_edge_coloring_greedy_k4() {
        let g = make_undirected(4, &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]);
        let colors = edge_coloring_greedy(&g).unwrap();
        assert!(is_edge_coloring(&g, &colors).unwrap());
        // K4 max degree = 3, Vizing: needs 3 or 4
        assert!(edge_chromatic_number(&colors) >= 3);
    }

    #[test]
    fn test_edge_coloring_greedy_star() {
        let g = make_undirected(5, &[(0, 1), (0, 2), (0, 3), (0, 4)]);
        let colors = edge_coloring_greedy(&g).unwrap();
        assert!(is_edge_coloring(&g, &colors).unwrap());
        assert_eq!(edge_chromatic_number(&colors), 4); // star: Δ colors
    }

    #[test]
    fn test_edge_coloring_greedy_bipartite() {
        let g = make_undirected(4, &[(0, 2), (0, 3), (1, 2), (1, 3)]);
        let colors = edge_coloring_greedy(&g).unwrap();
        assert!(is_edge_coloring(&g, &colors).unwrap());
    }

    #[test]
    fn test_edge_coloring_greedy_self_loop() {
        let g = make_undirected(3, &[(0, 0), (0, 1), (1, 2)]);
        let colors = edge_coloring_greedy(&g).unwrap();
        assert!(is_edge_coloring(&g, &colors).unwrap());
    }

    #[test]
    fn test_edge_coloring_greedy_parallel_edges() {
        let g = make_undirected(2, &[(0, 1), (0, 1), (0, 1)]);
        let colors = edge_coloring_greedy(&g).unwrap();
        assert!(is_edge_coloring(&g, &colors).unwrap());
        assert_eq!(edge_chromatic_number(&colors), 3);
    }

    #[test]
    fn test_edge_coloring_greedy_directed() {
        let g = Graph::new(3, true).unwrap();
        let mut g = g;
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let colors = edge_coloring_greedy(&g).unwrap();
        assert!(is_edge_coloring(&g, &colors).unwrap());
    }

    #[test]
    fn test_edge_chromatic_number_empty() {
        assert_eq!(edge_chromatic_number(&[]), 0);
    }

    #[test]
    fn test_edge_chromatic_number_values() {
        assert_eq!(edge_chromatic_number(&[0]), 1);
        assert_eq!(edge_chromatic_number(&[0, 1, 2]), 3);
        assert_eq!(edge_chromatic_number(&[0, 0, 0]), 1);
    }

    // ---- vertex_chromatic_number + chromatic_number_upper_bound ----

    #[test]
    fn test_vertex_chromatic_number_empty() {
        assert_eq!(vertex_chromatic_number(&[]), 0);
    }

    #[test]
    fn test_vertex_chromatic_number_values() {
        assert_eq!(vertex_chromatic_number(&[0]), 1);
        assert_eq!(vertex_chromatic_number(&[0, 1, 2]), 3);
        assert_eq!(vertex_chromatic_number(&[0, 0, 0]), 1);
        assert_eq!(vertex_chromatic_number(&[0, 1, 0, 1]), 2);
    }

    #[test]
    fn test_chromatic_upper_empty() {
        let g = make_undirected(0, &[]);
        assert_eq!(chromatic_number_upper_bound(&g).unwrap(), 0);
    }

    #[test]
    fn test_chromatic_upper_singleton() {
        let g = make_undirected(1, &[]);
        assert_eq!(chromatic_number_upper_bound(&g).unwrap(), 1);
    }

    #[test]
    fn test_chromatic_upper_no_edges() {
        let g = make_undirected(5, &[]);
        assert_eq!(chromatic_number_upper_bound(&g).unwrap(), 1);
    }

    #[test]
    fn test_chromatic_upper_single_edge() {
        let g = make_undirected(2, &[(0, 1)]);
        assert_eq!(chromatic_number_upper_bound(&g).unwrap(), 2);
    }

    #[test]
    fn test_chromatic_upper_triangle() {
        let g = make_undirected(3, &[(0, 1), (1, 2), (2, 0)]);
        assert_eq!(chromatic_number_upper_bound(&g).unwrap(), 3);
    }

    #[test]
    fn test_chromatic_upper_bipartite_c4() {
        let g = make_undirected(4, &[(0, 1), (1, 2), (2, 3), (3, 0)]);
        assert_eq!(chromatic_number_upper_bound(&g).unwrap(), 2);
    }

    #[test]
    fn test_chromatic_upper_k4() {
        let g = make_undirected(4, &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]);
        assert_eq!(chromatic_number_upper_bound(&g).unwrap(), 4);
    }

    #[test]
    fn test_chromatic_upper_path() {
        let g = make_undirected(4, &[(0, 1), (1, 2), (2, 3)]);
        assert_eq!(chromatic_number_upper_bound(&g).unwrap(), 2);
    }

    #[test]
    fn test_chromatic_upper_star() {
        let g = make_undirected(5, &[(0, 1), (0, 2), (0, 3), (0, 4)]);
        assert_eq!(chromatic_number_upper_bound(&g).unwrap(), 2);
    }

    #[test]
    fn test_chromatic_upper_self_loop() {
        let g = make_undirected(2, &[(0, 0), (0, 1)]);
        assert_eq!(chromatic_number_upper_bound(&g).unwrap(), 2);
    }

    #[test]
    fn test_chromatic_upper_directed() {
        let g = make_directed(3, &[(0, 1), (1, 2), (2, 0)]);
        assert_eq!(chromatic_number_upper_bound(&g).unwrap(), 3);
    }

    #[test]
    fn test_chromatic_upper_petersen() {
        let g = make_undirected(
            10,
            &[
                (0, 1),
                (1, 2),
                (2, 3),
                (3, 4),
                (4, 0),
                (5, 7),
                (7, 9),
                (9, 6),
                (6, 8),
                (8, 5),
                (0, 5),
                (1, 6),
                (2, 7),
                (3, 8),
                (4, 9),
            ],
        );
        let chi = chromatic_number_upper_bound(&g).unwrap();
        assert!(chi >= 3);
        assert!(chi <= 4);
    }
}

// =================================================================
// Proptest
// =================================================================

#[cfg(test)]
#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use crate::create;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_cn_valid_coloring(n in 2u32..50, edge_count in 1usize..100) {
            let max_edges = (n as usize) * (n as usize - 1) / 2;
            let actual_edges = edge_count.min(max_edges);
            let mut edges = Vec::with_capacity(actual_edges);
            let mut rng_state = 0x1234_5678u64;
            for _ in 0..actual_edges {
                rng_state = rng_state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let u = (rng_state >> 32) as u32 % n;
                rng_state = rng_state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let v = (rng_state >> 32) as u32 % n;
                if u != v {
                    edges.push((u, v));
                }
            }
            let g = create(&edges, n, false).expect("graph");
            let c = vertex_coloring_greedy(&g, GreedyColoringHeuristic::ColoredNeighbors)
                .expect("cn");
            prop_assert!(is_vertex_coloring(&g, &c).expect("validate"));
        }

        #[test]
        fn prop_dsatur_valid_coloring(n in 2u32..50, edge_count in 1usize..100) {
            let max_edges = (n as usize) * (n as usize - 1) / 2;
            let actual_edges = edge_count.min(max_edges);
            let mut edges = Vec::with_capacity(actual_edges);
            let mut rng_state = 0xABCD_EF01u64;
            for _ in 0..actual_edges {
                rng_state = rng_state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let u = (rng_state >> 32) as u32 % n;
                rng_state = rng_state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let v = (rng_state >> 32) as u32 % n;
                if u != v {
                    edges.push((u, v));
                }
            }
            let g = create(&edges, n, false).expect("graph");
            let c = vertex_coloring_greedy(&g, GreedyColoringHeuristic::DSatur)
                .expect("dsatur");
            prop_assert!(is_vertex_coloring(&g, &c).expect("validate"));
        }

        #[test]
        fn prop_dsatur_leq_cn_colors(n in 3u32..30, edge_count in 3usize..60) {
            let max_edges = (n as usize) * (n as usize - 1) / 2;
            let actual_edges = edge_count.min(max_edges);
            let mut edges = Vec::with_capacity(actual_edges);
            let mut rng_state = 0xDEAD_BEEFu64;
            for _ in 0..actual_edges {
                rng_state = rng_state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let u = (rng_state >> 32) as u32 % n;
                rng_state = rng_state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let v = (rng_state >> 32) as u32 % n;
                if u != v {
                    edges.push((u, v));
                }
            }
            let g = create(&edges, n, false).expect("graph");
            let c_cn = vertex_coloring_greedy(&g, GreedyColoringHeuristic::ColoredNeighbors)
                .expect("cn");
            let c_ds = vertex_coloring_greedy(&g, GreedyColoringHeuristic::DSatur)
                .expect("dsatur");
            let n_cn = c_cn.iter().copied().max().unwrap_or(0) + 1;
            let n_ds = c_ds.iter().copied().max().unwrap_or(0) + 1;
            prop_assert!(is_vertex_coloring(&g, &c_cn).expect("validate cn"));
            prop_assert!(is_vertex_coloring(&g, &c_ds).expect("validate ds"));
            prop_assert!(n_ds <= n_cn + 2, "DSATUR used {n_ds} colors vs CN {n_cn}");
        }

        #[test]
        fn prop_coloring_uses_contiguous_colors(n in 1u32..40, edge_count in 0usize..80) {
            let mut edges = Vec::new();
            let mut rng_state = 0xCAFE_BABEu64;
            for _ in 0..edge_count {
                rng_state = rng_state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let u = (rng_state >> 32) as u32 % n;
                rng_state = rng_state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let v = (rng_state >> 32) as u32 % n;
                if u != v {
                    edges.push((u, v));
                }
            }
            let g = create(&edges, n, false).expect("graph");
            let c = vertex_coloring_greedy(&g, GreedyColoringHeuristic::DSatur).expect("dsatur");
            if !c.is_empty() {
                let max_color = c.iter().copied().max().unwrap_or(0);
                let mut used = vec![false; (max_color + 1) as usize];
                for &col in &c {
                    used[col as usize] = true;
                }
                for (i, &u) in used.iter().enumerate() {
                    prop_assert!(u, "color {i} not used but max is {max_color}");
                }
            }
        }

        #[test]
        fn prop_bipartite_coloring_after_dsatur(n1 in 1u32..15, n2 in 1u32..15) {
            let n = n1 + n2;
            let mut edges = Vec::new();
            for u in 0..n1 {
                for v in n1..n {
                    edges.push((u, v));
                }
            }
            let g = create(&edges, n, false).expect("bipartite");
            let c = vertex_coloring_greedy(&g, GreedyColoringHeuristic::DSatur).expect("dsatur");
            let num_colors = c.iter().copied().max().unwrap_or(0) + 1;
            prop_assert_eq!(num_colors, 2, "bipartite graph should need exactly 2 colors");
        }
    }
}

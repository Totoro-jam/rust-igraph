//! VF2 graph isomorphism (`ALGO-ISO-001`).
//!
//! Port of the VF2 *graph-isomorphism* engine and its three public
//! wrappers from `references/igraph/src/isomorphism/vf2.c`:
//!
//! - the generic backtracking engine `igraph_get_isomorphisms_vf2_callback`
//!   (lines 137-697),
//! - [`isomorphic_vf2`] — stop at the first mapping (`igraph_isomorphic_vf2`,
//!   line 790),
//! - [`count_isomorphisms_vf2`] — count all mappings
//!   (`igraph_count_isomorphisms_vf2`, line 872),
//! - [`get_isomorphisms_vf2`] — collect every mapping
//!   (`igraph_get_isomorphisms_vf2`, line 947).
//!
//! VF2 (Cordella, Foggia, Sansone, Vento) explores a search tree of
//! candidate vertex pairs `(cand1, cand2)`, maintaining a partial mapping
//! plus the "terminal sets" `in_*`/`out_*` of not-yet-mapped vertices that
//! are adjacent to the current partial mapping. At each step it picks the
//! next candidate pair, runs the feasibility/pruning rules (degree match,
//! optional vertex/edge colours, edge-consistency in both directions for
//! directed graphs, and the look-ahead counts `xin`/`xout`), then either
//! extends the mapping or backtracks. When the mapping is complete it
//! reports the isomorphism through a callback.
//!
//! Like upstream, this engine **does not support self-loops** (rejected up
//! front) and treats colours via optional per-vertex / per-edge label
//! slices: two vertices (edges) can match only if their labels are equal.
//! The optional `node_compat_fn` / `edge_compat_fn` callbacks of the C API
//! are intentionally not exposed here — colours cover every documented
//! conformance case and the callback hooks can be added by a later AWU if a
//! real use case appears.

// The VF2 engine is a faithful port of a pointer/index-heavy C algorithm.
// It juggles three integer views of the same vertex ids: `usize` for slice
// indexing, `u32` for the [`Graph`] API, and `i64` for the `-1`-sentinel
// `core`/`last` slots. Every value is bounded by the vertex/edge count,
// which fits `u32` by the [`Graph`] contract, so the conversions cannot
// truncate, wrap, or lose sign in practice.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult};

/// Result of an exact VF2 graph-isomorphism test ([`isomorphic_vf2`]).
#[derive(Debug, Clone)]
pub struct Vf2Isomorphism {
    /// Whether the two graphs are isomorphic.
    pub iso: bool,
    /// The permutation taking `graph1` to `graph2`: `map12[i]` is the image
    /// of vertex `i` of `graph1`. Empty when the graphs are not isomorphic.
    pub map12: Vec<u32>,
    /// The inverse permutation taking `graph2` to `graph1`. Empty when the
    /// graphs are not isomorphic.
    pub map21: Vec<u32>,
}

/// Build deduplicated, ascending neighbour lists with self-loops removed,
/// reproducing igraph's `lazy_adjlist(mode, NO_LOOPS, NO_MULTIPLE)`.
///
/// `incoming = false` yields out-neighbours, `incoming = true` yields
/// in-neighbours. On undirected graphs both directions return the full
/// neighbour set (every edge is bidirectional), matching upstream.
pub(crate) fn adjacency(graph: &Graph, incoming: bool) -> IgraphResult<Vec<Vec<u32>>> {
    let n = graph.vcount();
    let mut adj: Vec<Vec<u32>> = Vec::with_capacity(n as usize);
    for v in 0..n {
        let mut neis: Vec<u32> = if !graph.is_directed() {
            // Undirected: all neighbours regardless of requested direction.
            graph.neighbors(v)?
        } else if incoming {
            // Directed in-neighbours: sources of incoming edges.
            let mut s = Vec::new();
            for eid in graph.incident_in(v)? {
                s.push(graph.edge_source(eid)?);
            }
            s
        } else {
            // Directed out-neighbours (already ascending from `oi`).
            graph.neighbors(v)?
        };
        neis.retain(|&u| u != v); // NO_LOOPS
        neis.sort_unstable();
        neis.dedup(); // NO_MULTIPLE
        adj.push(neis);
    }
    Ok(adj)
}

/// Out-degree (counting edge multiplicity, loops would count but are
/// rejected up front), matching `igraph_degree(_, OUT, LOOPS)`.
pub(crate) fn out_degree(graph: &Graph, v: u32) -> IgraphResult<i64> {
    if graph.is_directed() {
        Ok(graph.incident(v)?.len() as i64)
    } else {
        Ok(graph.degree(v)? as i64)
    }
}

/// In-degree counterpart of [`out_degree`].
pub(crate) fn in_degree(graph: &Graph, v: u32) -> IgraphResult<i64> {
    if graph.is_directed() {
        Ok(graph.incident_in(v)?.len() as i64)
    } else {
        Ok(graph.degree(v)? as i64)
    }
}

/// Whether a sorted slice contains `x` (binary search).
pub(crate) fn contains_sorted(slice: &[u32], x: u32) -> bool {
    slice.binary_search(&x).is_ok()
}

/// Multiset equality of two colour vectors, used for the cheap pre-filter
/// "the colour distributions must match or no isomorphism can exist".
fn same_color_distribution(a: &[u32], b: &[u32]) -> bool {
    let mut a = a.to_vec();
    let mut b = b.to_vec();
    a.sort_unstable();
    b.sort_unstable();
    a == b
}

/// Action returned by the per-isomorphism callback of [`vf2_engine`].
pub(crate) enum Flow {
    /// Keep searching for further isomorphisms.
    Continue,
    /// Stop the search immediately (a result has been captured).
    Stop,
}

pub(crate) fn perform_pre_checks(graph1: &Graph, graph2: &Graph) -> IgraphResult<()> {
    if graph1.is_directed() != graph2.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "cannot compare directed and undirected graphs".into(),
        ));
    }
    let has_loops = graph_has_loop(graph1)? || graph_has_loop(graph2)?;
    if has_loops {
        return Err(IgraphError::InvalidArgument(
            "the VF2 algorithm does not support graphs with loop edges".into(),
        ));
    }
    Ok(())
}

fn graph_has_loop(graph: &Graph) -> IgraphResult<bool> {
    for e in 0..graph.ecount() {
        let eid =
            EdgeId::try_from(e).map_err(|_| IgraphError::Internal("vf2: edge id overflow"))?;
        let (from, to) = graph.edge(eid)?;
        if from == to {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The generic VF2 backtracking engine for graph isomorphism.
///
/// Faithful translation of `igraph_get_isomorphisms_vf2_callback`. For every
/// complete mapping found, `isohandler` is invoked with `(core_1, core_2)`
/// where `core_1[i]` is the image in `graph2` of vertex `i` of `graph1` and
/// `core_2` is its inverse; returning [`Flow::Stop`] ends the search.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
fn vf2_engine(
    graph1: &Graph,
    graph2: &Graph,
    mut vertex_color1: Option<&[u32]>,
    mut vertex_color2: Option<&[u32]>,
    mut edge_color1: Option<&[u32]>,
    mut edge_color2: Option<&[u32]>,
    mut isohandler: impl FnMut(&[i64], &[i64]) -> Flow,
) -> IgraphResult<()> {
    perform_pre_checks(graph1, graph2)?;

    let no_of_nodes = i64::from(graph1.vcount());
    let no_of_edges = graph1.ecount() as i64;

    // Only one graph coloured -> ignore colours, matching upstream.
    if vertex_color1.is_some() != vertex_color2.is_some() {
        vertex_color1 = None;
        vertex_color2 = None;
    }
    if edge_color1.is_some() != edge_color2.is_some() {
        edge_color1 = None;
        edge_color2 = None;
    }

    // Different size -> cannot be isomorphic.
    if no_of_nodes != i64::from(graph2.vcount()) || no_of_edges != graph2.ecount() as i64 {
        return Ok(());
    }

    if let (Some(c1), Some(c2)) = (vertex_color1, vertex_color2) {
        if c1.len() as i64 != no_of_nodes || c2.len() as i64 != no_of_nodes {
            return Err(IgraphError::InvalidArgument(
                "invalid vertex color vector length".into(),
            ));
        }
        if !same_color_distribution(c1, c2) {
            return Ok(());
        }
    }
    if let (Some(c1), Some(c2)) = (edge_color1, edge_color2) {
        if c1.len() as i64 != no_of_edges || c2.len() as i64 != no_of_edges {
            return Err(IgraphError::InvalidArgument(
                "invalid edge color vector length".into(),
            ));
        }
        if !same_color_distribution(c1, c2) {
            return Ok(());
        }
    }

    let n = no_of_nodes as usize;

    let inneis_1 = adjacency(graph1, true)?;
    let outneis_1 = adjacency(graph1, false)?;
    let inneis_2 = adjacency(graph2, true)?;
    let outneis_2 = adjacency(graph2, false)?;

    let mut indeg1 = vec![0i64; n];
    let mut indeg2 = vec![0i64; n];
    let mut outdeg1 = vec![0i64; n];
    let mut outdeg2 = vec![0i64; n];
    for v in 0..no_of_nodes {
        let vu = v as usize;
        indeg1[vu] = in_degree(graph1, v as u32)?;
        indeg2[vu] = in_degree(graph2, v as u32)?;
        outdeg1[vu] = out_degree(graph1, v as u32)?;
        outdeg2[vu] = out_degree(graph2, v as u32)?;
    }

    // core_1[i] = image of vertex i of graph1 in graph2, or -1.
    let mut core_1 = vec![-1i64; n];
    let mut core_2 = vec![-1i64; n];
    // in_*/out_*[v] = depth at which v entered the terminal set, or 0.
    let mut in_1 = vec![0i64; n];
    let mut in_2 = vec![0i64; n];
    let mut out_1 = vec![0i64; n];
    let mut out_2 = vec![0i64; n];
    let mut in_1_size = 0i64;
    let mut in_2_size = 0i64;
    let mut out_1_size = 0i64;
    let mut out_2_size = 0i64;

    // path holds (cand1, cand2) pairs pushed on each successful step.
    let mut path: Vec<i64> = Vec::with_capacity(2 * n);
    let mut matched_nodes = 0i64;
    let mut depth = 0i64;
    let mut last1 = -1i64;
    let mut last2 = -1i64;

    while depth >= 0 {
        let mut cand1 = -1i64;
        let mut cand2 = -1i64;

        // Search for the next pair to try.
        if in_1_size != in_2_size || out_1_size != out_2_size {
            // step back, nothing to do
        } else if out_1_size > 0 && out_2_size > 0 {
            if last2 >= 0 {
                cand2 = last2;
            } else {
                let mut i = 0i64;
                while cand2 < 0 && i < no_of_nodes {
                    if out_2[i as usize] > 0 && core_2[i as usize] < 0 {
                        cand2 = i;
                    }
                    i += 1;
                }
            }
            let mut i = last1 + 1;
            while cand1 < 0 && i < no_of_nodes {
                if out_1[i as usize] > 0 && core_1[i as usize] < 0 {
                    cand1 = i;
                }
                i += 1;
            }
        } else if in_1_size > 0 && in_2_size > 0 {
            if last2 >= 0 {
                cand2 = last2;
            } else {
                let mut i = 0i64;
                while cand2 < 0 && i < no_of_nodes {
                    if in_2[i as usize] > 0 && core_2[i as usize] < 0 {
                        cand2 = i;
                    }
                    i += 1;
                }
            }
            let mut i = last1 + 1;
            while cand1 < 0 && i < no_of_nodes {
                if in_1[i as usize] > 0 && core_1[i as usize] < 0 {
                    cand1 = i;
                }
                i += 1;
            }
        } else {
            if last2 >= 0 {
                cand2 = last2;
            } else {
                let mut i = 0i64;
                while cand2 < 0 && i < no_of_nodes {
                    if core_2[i as usize] < 0 {
                        cand2 = i;
                    }
                    i += 1;
                }
            }
            let mut i = last1 + 1;
            while cand1 < 0 && i < no_of_nodes {
                if core_1[i as usize] < 0 {
                    cand1 = i;
                }
                i += 1;
            }
        }

        if cand1 < 0 || cand2 < 0 {
            // Dead end: step back if possible, otherwise terminate.
            if depth >= 1 {
                last2 = path.pop().ok_or(IgraphError::Internal("vf2: empty path"))?;
                last1 = path.pop().ok_or(IgraphError::Internal("vf2: empty path"))?;
                let l1 = last1 as usize;
                let l2 = last2 as usize;
                matched_nodes -= 1;
                core_1[l1] = -1;
                core_2[l2] = -1;

                if in_1[l1] != 0 {
                    in_1_size += 1;
                }
                if out_1[l1] != 0 {
                    out_1_size += 1;
                }
                if in_2[l2] != 0 {
                    in_2_size += 1;
                }
                if out_2[l2] != 0 {
                    out_2_size += 1;
                }

                for &node in &inneis_1[l1] {
                    if in_1[node as usize] == depth {
                        in_1[node as usize] = 0;
                        in_1_size -= 1;
                    }
                }
                for &node in &outneis_1[l1] {
                    if out_1[node as usize] == depth {
                        out_1[node as usize] = 0;
                        out_1_size -= 1;
                    }
                }
                for &node in &inneis_2[l2] {
                    if in_2[node as usize] == depth {
                        in_2[node as usize] = 0;
                        in_2_size -= 1;
                    }
                }
                for &node in &outneis_2[l2] {
                    if out_2[node as usize] == depth {
                        out_2[node as usize] = 0;
                        out_2_size -= 1;
                    }
                }
            }
            depth -= 1;
        } else {
            // Step forward if the (cand1, cand2) pair is feasible.
            let c1 = cand1 as usize;
            let c2 = cand2 as usize;
            let mut xin1 = 0i64;
            let mut xin2 = 0i64;
            let mut xout1 = 0i64;
            let mut xout2 = 0i64;
            let mut end = false;

            if indeg1[c1] != indeg2[c2] || outdeg1[c1] != outdeg2[c2] {
                end = true;
            }
            if let (Some(vc1), Some(vc2)) = (vertex_color1, vertex_color2) {
                if vc1[c1] != vc2[c2] {
                    end = true;
                }
            }

            // cand1's in-neighbours.
            for &node in &inneis_1[c1] {
                if end {
                    break;
                }
                let nu = node as usize;
                if core_1[nu] >= 0 {
                    let node2 = core_1[nu];
                    if !contains_sorted(&inneis_2[c2], node2 as u32) {
                        end = true;
                    } else if edge_color1.is_some() {
                        let eid1 = graph1.get_eid(node, cand1 as u32)? as usize;
                        let eid2 = graph2.get_eid(node2 as u32, cand2 as u32)? as usize;
                        if let (Some(ec1), Some(ec2)) = (edge_color1, edge_color2) {
                            if ec1[eid1] != ec2[eid2] {
                                end = true;
                            }
                        }
                    }
                } else {
                    if in_1[nu] != 0 {
                        xin1 += 1;
                    }
                    if out_1[nu] != 0 {
                        xout1 += 1;
                    }
                }
            }
            // cand1's out-neighbours.
            for &node in &outneis_1[c1] {
                if end {
                    break;
                }
                let nu = node as usize;
                if core_1[nu] >= 0 {
                    let node2 = core_1[nu];
                    if !contains_sorted(&outneis_2[c2], node2 as u32) {
                        end = true;
                    } else if edge_color1.is_some() {
                        let eid1 = graph1.get_eid(cand1 as u32, node)? as usize;
                        let eid2 = graph2.get_eid(cand2 as u32, node2 as u32)? as usize;
                        if let (Some(ec1), Some(ec2)) = (edge_color1, edge_color2) {
                            if ec1[eid1] != ec2[eid2] {
                                end = true;
                            }
                        }
                    }
                } else {
                    if in_1[nu] != 0 {
                        xin1 += 1;
                    }
                    if out_1[nu] != 0 {
                        xout1 += 1;
                    }
                }
            }
            // cand2's in-neighbours.
            for &node in &inneis_2[c2] {
                if end {
                    break;
                }
                let nu = node as usize;
                if core_2[nu] >= 0 {
                    let node2 = core_2[nu];
                    if !contains_sorted(&inneis_1[c1], node2 as u32) {
                        end = true;
                    } else if edge_color1.is_some() {
                        let eid1 = graph1.get_eid(node2 as u32, cand1 as u32)? as usize;
                        let eid2 = graph2.get_eid(node, cand2 as u32)? as usize;
                        if let (Some(ec1), Some(ec2)) = (edge_color1, edge_color2) {
                            if ec1[eid1] != ec2[eid2] {
                                end = true;
                            }
                        }
                    }
                } else {
                    if in_2[nu] != 0 {
                        xin2 += 1;
                    }
                    if out_2[nu] != 0 {
                        xout2 += 1;
                    }
                }
            }
            // cand2's out-neighbours.
            for &node in &outneis_2[c2] {
                if end {
                    break;
                }
                let nu = node as usize;
                if core_2[nu] >= 0 {
                    let node2 = core_2[nu];
                    if !contains_sorted(&outneis_1[c1], node2 as u32) {
                        end = true;
                    } else if edge_color1.is_some() {
                        let eid1 = graph1.get_eid(cand1 as u32, node2 as u32)? as usize;
                        let eid2 = graph2.get_eid(cand2 as u32, node)? as usize;
                        if let (Some(ec1), Some(ec2)) = (edge_color1, edge_color2) {
                            if ec1[eid1] != ec2[eid2] {
                                end = true;
                            }
                        }
                    }
                } else {
                    if in_2[nu] != 0 {
                        xin2 += 1;
                    }
                    if out_2[nu] != 0 {
                        xout2 += 1;
                    }
                }
            }

            if !end && xin1 == xin2 && xout1 == xout2 {
                // Add (cand1, cand2) to the mapping.
                depth += 1;
                path.push(cand1);
                path.push(cand2);
                matched_nodes += 1;
                core_1[c1] = cand2;
                core_2[c2] = cand1;

                if in_1[c1] != 0 {
                    in_1_size -= 1;
                }
                if out_1[c1] != 0 {
                    out_1_size -= 1;
                }
                if in_2[c2] != 0 {
                    in_2_size -= 1;
                }
                if out_2[c2] != 0 {
                    out_2_size -= 1;
                }

                for &node in &inneis_1[c1] {
                    let nu = node as usize;
                    if in_1[nu] == 0 && core_1[nu] < 0 {
                        in_1[nu] = depth;
                        in_1_size += 1;
                    }
                }
                for &node in &outneis_1[c1] {
                    let nu = node as usize;
                    if out_1[nu] == 0 && core_1[nu] < 0 {
                        out_1[nu] = depth;
                        out_1_size += 1;
                    }
                }
                for &node in &inneis_2[c2] {
                    let nu = node as usize;
                    if in_2[nu] == 0 && core_2[nu] < 0 {
                        in_2[nu] = depth;
                        in_2_size += 1;
                    }
                }
                for &node in &outneis_2[c2] {
                    let nu = node as usize;
                    if out_2[nu] == 0 && core_2[nu] < 0 {
                        out_2[nu] = depth;
                        out_2_size += 1;
                    }
                }

                last1 = -1;
                last2 = -1;
            } else {
                last1 = cand1;
                last2 = cand2;
            }
        }

        if matched_nodes == no_of_nodes {
            if let Flow::Stop = isohandler(&core_1, &core_2) {
                break;
            }
        }
    }

    Ok(())
}

/// Test whether two graphs are isomorphic using the VF2 algorithm.
///
/// Optional `vertex_color*` / `edge_color*` slices restrict the matching:
/// two vertices (edges) may correspond only if their colours are equal. Pass
/// `None` for uncoloured graphs; supplying a colour for only one side makes
/// that colour be ignored (matching upstream).
///
/// On success [`Vf2Isomorphism::iso`] tells whether a mapping exists; when it
/// does, `map12` / `map21` hold the permutation and its inverse, otherwise
/// they are empty.
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] if the two graphs differ in
/// directedness, if either contains a self-loop (VF2 does not support loops),
/// or if a supplied colour vector has the wrong length.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, isomorphic_vf2};
///
/// // Two triangles with relabelled vertices are isomorphic.
/// let mut a = Graph::new(3, false).unwrap();
/// a.add_edge(0, 1).unwrap();
/// a.add_edge(1, 2).unwrap();
/// a.add_edge(2, 0).unwrap();
/// let mut b = Graph::new(3, false).unwrap();
/// b.add_edge(0, 2).unwrap();
/// b.add_edge(2, 1).unwrap();
/// b.add_edge(1, 0).unwrap();
/// let r = isomorphic_vf2(&a, &b, None, None, None, None).unwrap();
/// assert!(r.iso);
/// assert_eq!(r.map12.len(), 3);
/// ```
#[allow(clippy::too_many_arguments)]
pub fn isomorphic_vf2(
    graph1: &Graph,
    graph2: &Graph,
    vertex_color1: Option<&[u32]>,
    vertex_color2: Option<&[u32]>,
    edge_color1: Option<&[u32]>,
    edge_color2: Option<&[u32]>,
) -> IgraphResult<Vf2Isomorphism> {
    let mut map12: Vec<u32> = Vec::new();
    let mut map21: Vec<u32> = Vec::new();
    let mut iso = false;

    vf2_engine(
        graph1,
        graph2,
        vertex_color1,
        vertex_color2,
        edge_color1,
        edge_color2,
        |core_1, core_2| {
            iso = true;
            map12 = core_1.iter().map(|&x| x as u32).collect();
            map21 = core_2.iter().map(|&x| x as u32).collect();
            Flow::Stop
        },
    )?;

    if !iso {
        map12.clear();
        map21.clear();
    }
    Ok(Vf2Isomorphism { iso, map12, map21 })
}

/// Count the number of VF2 isomorphic mappings between two graphs.
///
/// Calling this with the same graph as both arguments counts the graph's
/// automorphisms. Colour arguments behave as in [`isomorphic_vf2`].
///
/// # Errors
///
/// Same conditions as [`isomorphic_vf2`].
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, count_isomorphisms_vf2};
///
/// // A 4-cycle has 8 automorphisms (4 rotations x 2 reflections).
/// let mut g = Graph::new(4, false).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 0).unwrap();
/// let count = count_isomorphisms_vf2(&g, &g, None, None, None, None).unwrap();
/// assert_eq!(count, 8);
/// ```
#[allow(clippy::too_many_arguments)]
pub fn count_isomorphisms_vf2(
    graph1: &Graph,
    graph2: &Graph,
    vertex_color1: Option<&[u32]>,
    vertex_color2: Option<&[u32]>,
    edge_color1: Option<&[u32]>,
    edge_color2: Option<&[u32]>,
) -> IgraphResult<u64> {
    let mut count = 0u64;
    vf2_engine(
        graph1,
        graph2,
        vertex_color1,
        vertex_color2,
        edge_color1,
        edge_color2,
        |_core_1, _core_2| {
            count += 1;
            Flow::Continue
        },
    )?;
    Ok(count)
}

/// Collect every VF2 isomorphic mapping between two graphs.
///
/// Each returned vector is a `map21` mapping: position `j` holds the vertex
/// of `graph1` that vertex `j` of `graph2` maps to. The list is empty when
/// the graphs are not isomorphic. Colour arguments behave as in
/// [`isomorphic_vf2`].
///
/// # Errors
///
/// Same conditions as [`isomorphic_vf2`].
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, get_isomorphisms_vf2};
///
/// // A single edge has two automorphisms (identity and the swap).
/// let mut g = Graph::new(2, false).unwrap();
/// g.add_edge(0, 1).unwrap();
/// let maps = get_isomorphisms_vf2(&g, &g, None, None, None, None).unwrap();
/// assert_eq!(maps.len(), 2);
/// ```
#[allow(clippy::too_many_arguments)]
pub fn get_isomorphisms_vf2(
    graph1: &Graph,
    graph2: &Graph,
    vertex_color1: Option<&[u32]>,
    vertex_color2: Option<&[u32]>,
    edge_color1: Option<&[u32]>,
    edge_color2: Option<&[u32]>,
) -> IgraphResult<Vec<Vec<u32>>> {
    let mut maps: Vec<Vec<u32>> = Vec::new();
    vf2_engine(
        graph1,
        graph2,
        vertex_color1,
        vertex_color2,
        edge_color1,
        edge_color2,
        |_core_1, core_2| {
            maps.push(core_2.iter().map(|&x| x as u32).collect());
            Flow::Continue
        },
    )?;
    Ok(maps)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Undirected ring on `n` vertices: 0-1-...-(n-1)-0.
    fn ring(n: u32, directed: bool) -> Graph {
        let mut g = Graph::new(n, directed).expect("graph");
        for i in 0..n {
            g.add_edge(i, (i + 1) % n).expect("edge");
        }
        g
    }

    /// Build a graph from an explicit edge list.
    fn graph_from(n: u32, directed: bool, edges: &[(u32, u32)]) -> Graph {
        let mut g = Graph::new(n, directed).expect("graph");
        for &(u, v) in edges {
            g.add_edge(u, v).expect("edge");
        }
        g
    }

    #[test]
    fn empty_graphs_are_isomorphic() {
        let a = Graph::new(0, false).expect("graph");
        let b = Graph::new(0, false).expect("graph");
        let r = isomorphic_vf2(&a, &b, None, None, None, None).expect("ok");
        assert!(r.iso);
        assert!(r.map12.is_empty());
        // Exactly one (empty) mapping.
        let c = count_isomorphisms_vf2(&a, &b, None, None, None, None).expect("ok");
        assert_eq!(c, 1);
    }

    #[test]
    fn single_vertex_one_automorphism() {
        let g = Graph::new(1, false).expect("graph");
        let c = count_isomorphisms_vf2(&g, &g, None, None, None, None).expect("ok");
        assert_eq!(c, 1);
        let r = isomorphic_vf2(&g, &g, None, None, None, None).expect("ok");
        assert!(r.iso);
        assert_eq!(r.map12, vec![0]);
        assert_eq!(r.map21, vec![0]);
    }

    #[test]
    fn different_vertex_counts_not_isomorphic() {
        let a = ring(4, false);
        let b = ring(5, false);
        let r = isomorphic_vf2(&a, &b, None, None, None, None).expect("ok");
        assert!(!r.iso);
        assert!(r.map12.is_empty());
        let c = count_isomorphisms_vf2(&a, &b, None, None, None, None).expect("ok");
        assert_eq!(c, 0);
    }

    #[test]
    fn same_vertices_different_edge_counts_not_isomorphic() {
        let path = graph_from(4, false, &[(0, 1), (1, 2), (2, 3)]);
        let star = graph_from(4, false, &[(0, 1), (0, 2), (0, 3)]);
        // Same vcount (4) and ecount (3), but degree sequences differ.
        let r = isomorphic_vf2(&path, &star, None, None, None, None).expect("ok");
        assert!(!r.iso);
    }

    #[test]
    fn triangle_relabelled_is_isomorphic_with_consistent_maps() {
        let a = graph_from(3, false, &[(0, 1), (1, 2), (2, 0)]);
        let b = graph_from(3, false, &[(0, 2), (2, 1), (1, 0)]);
        let r = isomorphic_vf2(&a, &b, None, None, None, None).expect("ok");
        assert!(r.iso);
        // map12 and map21 are mutual inverses.
        for i in 0..3usize {
            assert_eq!(r.map21[r.map12[i] as usize] as usize, i);
            assert_eq!(r.map12[r.map21[i] as usize] as usize, i);
        }
    }

    #[test]
    fn undirected_ring_automorphisms() {
        // ring(n) undirected has 2n automorphisms (n rotations x 2 reflections).
        assert_eq!(
            count_isomorphisms_vf2(&ring(4, false), &ring(4, false), None, None, None, None)
                .expect("ok"),
            8
        );
        assert_eq!(
            count_isomorphisms_vf2(&ring(6, false), &ring(6, false), None, None, None, None)
                .expect("ok"),
            12
        );
        // The headline oracle value from the upstream unit test.
        assert_eq!(
            count_isomorphisms_vf2(&ring(100, false), &ring(100, false), None, None, None, None)
                .expect("ok"),
            200
        );
    }

    #[test]
    fn directed_ring_has_only_rotations() {
        // A directed cycle has exactly n automorphisms (rotations; no reflection).
        assert_eq!(
            count_isomorphisms_vf2(&ring(4, true), &ring(4, true), None, None, None, None)
                .expect("ok"),
            4
        );
        assert_eq!(
            count_isomorphisms_vf2(&ring(100, true), &ring(100, true), None, None, None, None)
                .expect("ok"),
            100
        );
    }

    #[test]
    fn single_edge_has_two_automorphisms() {
        let g = graph_from(2, false, &[(0, 1)]);
        let maps = get_isomorphisms_vf2(&g, &g, None, None, None, None).expect("ok");
        assert_eq!(maps.len(), 2);
        // Identity and the swap.
        assert!(maps.contains(&vec![0, 1]));
        assert!(maps.contains(&vec![1, 0]));
    }

    #[test]
    fn distinct_vertex_colors_pin_the_mapping() {
        let g = ring(6, false);
        let colors: Vec<u32> = (0..6).collect();
        // Every vertex has a unique colour -> only the identity survives.
        let c =
            count_isomorphisms_vf2(&g, &g, Some(&colors), Some(&colors), None, None).expect("ok");
        assert_eq!(c, 1);
    }

    #[test]
    fn mismatched_color_distribution_blocks_isomorphism() {
        let g = ring(4, false);
        let c1 = [0u32, 0, 0, 0];
        let c2 = [0u32, 0, 0, 1];
        let r = isomorphic_vf2(&g, &g, Some(&c1), Some(&c2), None, None).expect("ok");
        assert!(!r.iso);
    }

    #[test]
    fn vertex_colors_can_reduce_symmetry() {
        // Colour two opposite vertices of a 4-cycle one colour, the other
        // two another colour. Automorphisms preserving the colouring: the
        // identity, the half-turn, and the two reflections through the
        // coloured pairs -> 4 of the 8 uncoloured automorphisms.
        let g = ring(4, false);
        let colors = [0u32, 1, 0, 1];
        let c =
            count_isomorphisms_vf2(&g, &g, Some(&colors), Some(&colors), None, None).expect("ok");
        assert_eq!(c, 4);
    }

    #[test]
    fn only_one_side_colored_ignores_colors() {
        let g = ring(4, false);
        let colors = [0u32, 1, 2, 3];
        // graph2 has no colours -> colours ignored, full symmetry restored.
        let c = count_isomorphisms_vf2(&g, &g, Some(&colors), None, None, None).expect("ok");
        assert_eq!(c, 8);
    }

    #[test]
    fn edge_colors_constrain_matching() {
        // Triangle with one distinctly coloured edge: automorphisms must fix
        // that edge, leaving identity + the reflection swapping its endpoints.
        let g = graph_from(3, false, &[(0, 1), (1, 2), (2, 0)]);
        // Edge ids follow insertion/canonical order; colour edge (1,2)
        // differently from the other two.
        let mut ec = vec![0u32; g.ecount()];
        // Find the edge between 1 and 2 and colour it 1.
        for (e, slot) in ec.iter_mut().enumerate() {
            let eid = u32::try_from(e).expect("fits");
            let (a, b) = g.edge(eid).expect("edge");
            if (a, b) == (1, 2) || (a, b) == (2, 1) {
                *slot = 1;
            }
        }
        let c = count_isomorphisms_vf2(&g, &g, None, None, Some(&ec), Some(&ec)).expect("ok");
        assert_eq!(c, 2);
    }

    #[test]
    fn self_loops_are_rejected() {
        let g = graph_from(2, false, &[(0, 0), (0, 1)]);
        let h = graph_from(2, false, &[(0, 1)]);
        assert!(isomorphic_vf2(&g, &h, None, None, None, None).is_err());
        assert!(isomorphic_vf2(&h, &g, None, None, None, None).is_err());
    }

    #[test]
    fn directedness_mismatch_is_rejected() {
        let u = ring(3, false);
        let d = ring(3, true);
        assert!(isomorphic_vf2(&u, &d, None, None, None, None).is_err());
    }

    #[test]
    fn wrong_color_vector_length_errors() {
        let g = ring(3, false);
        let short = [0u32, 0];
        assert!(isomorphic_vf2(&g, &g, Some(&short), Some(&short), None, None).is_err());
    }

    #[test]
    fn get_and_count_agree() {
        let g = ring(6, false);
        let maps = get_isomorphisms_vf2(&g, &g, None, None, None, None).expect("ok");
        let c = count_isomorphisms_vf2(&g, &g, None, None, None, None).expect("ok");
        assert_eq!(maps.len() as u64, c);
    }

    #[test]
    fn returned_mapping_preserves_adjacency() {
        // Two different drawings of the same 5-cycle.
        let src = graph_from(5, false, &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)]);
        let dst = graph_from(5, false, &[(0, 2), (2, 4), (4, 1), (1, 3), (3, 0)]);
        let r = isomorphic_vf2(&src, &dst, None, None, None, None).expect("ok");
        assert!(r.iso);
        // Every edge (u,v) in src must map to an edge in dst.
        for e in 0..src.ecount() {
            let eid = u32::try_from(e).expect("fits");
            let (u, v) = src.edge(eid).expect("edge");
            let mu = r.map12[u as usize];
            let mv = r.map12[v as usize];
            assert!(
                dst.find_eid(mu, mv).expect("lookup").is_some(),
                "edge {u}-{v} not preserved"
            );
        }
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::HashSet;

    /// Generate a simple, loopless graph (VF2 precondition): `n` vertices and
    /// a deduplicated, self-loop-free edge set, with a directedness flag.
    fn arb_simple_graph(max_v: u32) -> impl Strategy<Value = (u32, Vec<(u32, u32)>, bool)> {
        (1..=max_v, any::<bool>()).prop_flat_map(|(n, directed)| {
            proptest::collection::vec((0..n, 0..n), 0..=12).prop_map(move |raw| {
                let mut seen: HashSet<(u32, u32)> = HashSet::new();
                let mut edges = Vec::new();
                for (u, v) in raw {
                    if u == v {
                        continue; // no self-loops
                    }
                    let key = if directed {
                        (u, v)
                    } else {
                        (u.min(v), u.max(v))
                    };
                    if seen.insert(key) {
                        edges.push((u, v));
                    }
                }
                (n, edges, directed)
            })
        })
    }

    /// Apply a vertex permutation `perm` (perm[old] = new) to an edge list.
    fn relabel(edges: &[(u32, u32)], perm: &[u32]) -> Vec<(u32, u32)> {
        edges
            .iter()
            .map(|&(u, v)| (perm[u as usize], perm[v as usize]))
            .collect()
    }

    fn build(n: u32, directed: bool, edges: &[(u32, u32)]) -> Graph {
        let mut g = Graph::new(n, directed).expect("graph");
        for &(u, v) in edges {
            g.add_edge(u, v).expect("edge");
        }
        g
    }

    proptest! {
        /// A graph is always isomorphic to itself, with a self-consistent
        /// (mutually inverse) pair of mapping permutations.
        #[test]
        fn self_isomorphic((n, edges, directed) in arb_simple_graph(7)) {
            let g = build(n, directed, &edges);
            let r = isomorphic_vf2(&g, &g, None, None, None, None).expect("ok");
            prop_assert!(r.iso);
            prop_assert_eq!(r.map12.len(), n as usize);
            for i in 0..n as usize {
                prop_assert_eq!(r.map21[r.map12[i] as usize] as usize, i);
            }
        }

        /// Relabelling a graph by a random permutation yields an isomorphic
        /// graph, and the automorphism count is preserved under relabelling.
        #[test]
        fn isomorphic_under_relabelling(
            (n, edges, directed) in arb_simple_graph(6),
            seed in any::<u64>(),
        ) {
            // Build a permutation from the seed via a Fisher-Yates shuffle.
            let mut perm: Vec<u32> = (0..n).collect();
            let mut state = seed;
            for i in (1..n as usize).rev() {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                let j = (state >> 33) as usize % (i + 1);
                perm.swap(i, j);
            }
            let g = build(n, directed, &edges);
            let h = build(n, directed, &relabel(&edges, &perm));

            let r = isomorphic_vf2(&g, &h, None, None, None, None).expect("ok");
            prop_assert!(r.iso, "relabelled graph must be isomorphic");

            // Every edge of g maps to an edge of h.
            for e in 0..g.ecount() {
                let eid = u32::try_from(e).expect("fits");
                let (u, v) = g.edge(eid).expect("edge");
                let mu = r.map12[u as usize];
                let mv = r.map12[v as usize];
                prop_assert!(h.find_eid(mu, mv).expect("lookup").is_some());
            }

            // Automorphism count is a graph invariant.
            let cg = count_isomorphisms_vf2(&g, &g, None, None, None, None).expect("ok");
            let ch = count_isomorphisms_vf2(&h, &h, None, None, None, None).expect("ok");
            prop_assert_eq!(cg, ch);
        }

        /// `get_isomorphisms_vf2` returns exactly as many maps as
        /// `count_isomorphisms_vf2` reports, and the self-count is >= 1.
        #[test]
        fn get_count_consistency((n, edges, directed) in arb_simple_graph(6)) {
            let g = build(n, directed, &edges);
            let maps = get_isomorphisms_vf2(&g, &g, None, None, None, None).expect("ok");
            let c = count_isomorphisms_vf2(&g, &g, None, None, None, None).expect("ok");
            prop_assert_eq!(maps.len() as u64, c);
            prop_assert!(c >= 1, "identity automorphism always exists");
            // Each returned map is a valid permutation.
            for m in &maps {
                prop_assert_eq!(m.len(), n as usize);
                let mut sorted = m.clone();
                sorted.sort_unstable();
                let expected: Vec<u32> = (0..n).collect();
                prop_assert_eq!(sorted, expected);
            }
        }

        /// Giving every vertex a distinct colour forces a unique (identity)
        /// mapping for self-comparison.
        #[test]
        fn distinct_colors_force_unique_self_map(
            (n, edges, directed) in arb_simple_graph(6),
        ) {
            let g = build(n, directed, &edges);
            let colors: Vec<u32> = (0..n).collect();
            let c = count_isomorphisms_vf2(&g, &g, Some(&colors), Some(&colors), None, None)
                .expect("ok");
            prop_assert_eq!(c, 1);
        }
    }
}

//! VF2 subgraph isomorphism (`ALGO-ISO-002`).
//!
//! Port of the VF2 *subgraph-isomorphism* engine and its three public
//! wrappers from `references/igraph/src/isomorphism/vf2.c`:
//!
//! - the backtracking engine `igraph_get_subisomorphisms_vf2_callback`
//!   (lines 1023-1514),
//! - [`subisomorphic_vf2`] — stop at the first embedding
//!   (`igraph_subisomorphic_vf2`, line 1570),
//! - [`count_subisomorphisms_vf2`] — count all embeddings
//!   (`igraph_count_subisomorphisms_vf2`, line 1652),
//! - [`get_subisomorphisms_vf2`] — collect every embedding
//!   (`igraph_get_subisomorphisms_vf2`, line 1717).
//!
//! Unlike [`isomorphic_vf2`](super::vf2::isomorphic_vf2), which decides whether
//! two whole graphs are isomorphic, the subgraph engine looks for copies of a
//! *pattern* `graph2` inside a (usually larger) *target* `graph1`. The match is
//! the **non-induced** subgraph isomorphism igraph implements: every edge of
//! the pattern must map to an edge of the target, but the target may carry
//! extra edges among the mapped vertices. The search shares VF2's structure —
//! candidate pairs, the `in_*`/`out_*` terminal sets, depth-first backtracking
//! — but relaxes every exact test to an inequality: a target vertex may host a
//! pattern vertex only if its degree is *at least* as large, and the look-ahead
//! counts must dominate rather than match.
//!
//! As in upstream, self-loops are rejected up front and colours are optional
//! per-vertex / per-edge label slices: a pattern element matches a target
//! element only if their labels are equal. Supplying a colour for only one
//! side makes that colour be ignored.

// See the sibling `vf2` module for the rationale: this engine juggles `usize`,
// `u32`, and `-1`-sentinel `i64` views of the same bounded vertex ids, so the
// integer casts cannot truncate, wrap, or lose sign in practice.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

use super::vf2::{Flow, adjacency, contains_sorted, in_degree, out_degree, perform_pre_checks};
use crate::core::{Graph, IgraphError, IgraphResult};

/// Result of a VF2 subgraph-isomorphism test ([`subisomorphic_vf2`]).
#[derive(Debug, Clone)]
pub struct Vf2Subisomorphism {
    /// Whether the pattern (`graph2`) is a subgraph of the target (`graph1`).
    pub iso: bool,
    /// The embedding of the pattern into the target: `map21[j]` is the vertex
    /// of `graph1` that pattern vertex `j` maps to. Empty when no embedding
    /// exists.
    pub map21: Vec<u32>,
    /// The reverse view: `map12[i]` is `Some(p)` when target vertex `i` hosts
    /// pattern vertex `p`, or `None` when it is unused by the embedding. Empty
    /// when no embedding exists.
    pub map12: Vec<Option<u32>>,
}

/// The VF2 backtracking engine for (non-induced) subgraph isomorphism.
///
/// Faithful translation of `igraph_get_subisomorphisms_vf2_callback`. `graph1`
/// is the target and `graph2` the pattern. For every complete embedding found,
/// `isohandler` is invoked with `(core_1, core_2)` where `core_1[i]` is the
/// pattern vertex hosted by target vertex `i` (or `-1`) and `core_2[j]` is the
/// target vertex hosting pattern vertex `j`; returning [`Flow::Stop`] ends the
/// search.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
fn subiso_engine(
    graph1: &Graph,
    graph2: &Graph,
    mut vertex_color1: Option<&[u32]>,
    mut vertex_color2: Option<&[u32]>,
    mut edge_color1: Option<&[u32]>,
    mut edge_color2: Option<&[u32]>,
    mut isohandler: impl FnMut(&[i64], &[i64]) -> Flow,
) -> IgraphResult<()> {
    perform_pre_checks(graph1, graph2)?;

    let no_of_nodes1 = i64::from(graph1.vcount());
    let no_of_nodes2 = i64::from(graph2.vcount());
    let no_of_edges1 = graph1.ecount() as i64;
    let no_of_edges2 = graph2.ecount() as i64;

    // The pattern cannot fit into a smaller target.
    if no_of_nodes1 < no_of_nodes2 || no_of_edges1 < no_of_edges2 {
        return Ok(());
    }

    // Only one graph coloured -> ignore colours, matching upstream.
    if vertex_color1.is_some() != vertex_color2.is_some() {
        vertex_color1 = None;
        vertex_color2 = None;
    }
    if edge_color1.is_some() != edge_color2.is_some() {
        edge_color1 = None;
        edge_color2 = None;
    }

    if let (Some(c1), Some(c2)) = (vertex_color1, vertex_color2) {
        if c1.len() as i64 != no_of_nodes1 || c2.len() as i64 != no_of_nodes2 {
            return Err(IgraphError::InvalidArgument(
                "invalid vertex color vector length".into(),
            ));
        }
    }
    if let (Some(c1), Some(c2)) = (edge_color1, edge_color2) {
        if c1.len() as i64 != no_of_edges1 || c2.len() as i64 != no_of_edges2 {
            return Err(IgraphError::InvalidArgument(
                "invalid edge color vector length".into(),
            ));
        }
    }

    let n1 = no_of_nodes1 as usize;
    let n2 = no_of_nodes2 as usize;

    let inneis_1 = adjacency(graph1, true)?;
    let outneis_1 = adjacency(graph1, false)?;
    let inneis_2 = adjacency(graph2, true)?;
    let outneis_2 = adjacency(graph2, false)?;

    let mut indeg1 = vec![0i64; n1];
    let mut outdeg1 = vec![0i64; n1];
    for v in 0..no_of_nodes1 {
        let vu = v as usize;
        indeg1[vu] = in_degree(graph1, v as u32)?;
        outdeg1[vu] = out_degree(graph1, v as u32)?;
    }
    let mut indeg2 = vec![0i64; n2];
    let mut outdeg2 = vec![0i64; n2];
    for v in 0..no_of_nodes2 {
        let vu = v as usize;
        indeg2[vu] = in_degree(graph2, v as u32)?;
        outdeg2[vu] = out_degree(graph2, v as u32)?;
    }

    // core_1[i] = pattern vertex hosted by target vertex i, or -1.
    let mut core_1 = vec![-1i64; n1];
    // core_2[j] = target vertex hosting pattern vertex j, or -1.
    let mut core_2 = vec![-1i64; n2];
    // in_*/out_*[v] = depth at which v entered the terminal set, or 0.
    let mut in_1 = vec![0i64; n1];
    let mut in_2 = vec![0i64; n2];
    let mut out_1 = vec![0i64; n1];
    let mut out_2 = vec![0i64; n2];
    let mut in_1_size = 0i64;
    let mut in_2_size = 0i64;
    let mut out_1_size = 0i64;
    let mut out_2_size = 0i64;

    // path holds (cand1, cand2) pairs pushed on each successful step.
    let mut path: Vec<i64> = Vec::with_capacity(2 * n2);
    let mut matched_nodes = 0i64;
    let mut depth = 0i64;
    let mut last1 = -1i64;
    let mut last2 = -1i64;

    while depth >= 0 {
        let mut cand1 = -1i64;
        let mut cand2 = -1i64;

        // Search for the next pair to try. The pattern's terminal sets must
        // not outgrow the target's, otherwise step back.
        if in_1_size < in_2_size || out_1_size < out_2_size {
            // step back, nothing to do
        } else if out_1_size > 0 && out_2_size > 0 {
            if last2 >= 0 {
                cand2 = last2;
            } else {
                let mut i = 0i64;
                while cand2 < 0 && i < no_of_nodes2 {
                    if out_2[i as usize] > 0 && core_2[i as usize] < 0 {
                        cand2 = i;
                    }
                    i += 1;
                }
            }
            let mut i = last1 + 1;
            while cand1 < 0 && i < no_of_nodes1 {
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
                while cand2 < 0 && i < no_of_nodes2 {
                    if in_2[i as usize] > 0 && core_2[i as usize] < 0 {
                        cand2 = i;
                    }
                    i += 1;
                }
            }
            let mut i = last1 + 1;
            while cand1 < 0 && i < no_of_nodes1 {
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
                while cand2 < 0 && i < no_of_nodes2 {
                    if core_2[i as usize] < 0 {
                        cand2 = i;
                    }
                    i += 1;
                }
            }
            let mut i = last1 + 1;
            while cand1 < 0 && i < no_of_nodes1 {
                if core_1[i as usize] < 0 {
                    cand1 = i;
                }
                i += 1;
            }
        }

        if cand1 < 0 || cand2 < 0 {
            // Dead end: step back if possible, otherwise terminate.
            if depth >= 1 {
                last2 = path
                    .pop()
                    .ok_or(IgraphError::Internal("subiso: empty path"))?;
                last1 = path
                    .pop()
                    .ok_or(IgraphError::Internal("subiso: empty path"))?;
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

            // The target host must have at least the pattern vertex's degree.
            if indeg1[c1] < indeg2[c2] || outdeg1[c1] < outdeg2[c2] {
                end = true;
            }
            if let (Some(vc1), Some(vc2)) = (vertex_color1, vertex_color2) {
                if vc1[c1] != vc2[c2] {
                    end = true;
                }
            }

            // cand1's neighbours only contribute look-ahead counts; the
            // target may have extra edges among mapped vertices (non-induced).
            for &node in &inneis_1[c1] {
                if end {
                    break;
                }
                let nu = node as usize;
                if core_1[nu] < 0 {
                    if in_1[nu] != 0 {
                        xin1 += 1;
                    }
                    if out_1[nu] != 0 {
                        xout1 += 1;
                    }
                }
            }
            for &node in &outneis_1[c1] {
                if end {
                    break;
                }
                let nu = node as usize;
                if core_1[nu] < 0 {
                    if in_1[nu] != 0 {
                        xin1 += 1;
                    }
                    if out_1[nu] != 0 {
                        xout1 += 1;
                    }
                }
            }
            // cand2's neighbours: every mapped pattern edge must exist in the
            // target, otherwise reject; unmapped ones feed the look-ahead.
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

            if !end && xin1 >= xin2 && xout1 >= xout2 {
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

        if matched_nodes == no_of_nodes2 {
            if let Flow::Stop = isohandler(&core_1, &core_2) {
                break;
            }
        }
    }

    Ok(())
}

/// Test whether the pattern `graph2` is a (non-induced) subgraph of the target
/// `graph1`, using the VF2 algorithm.
///
/// Optional `vertex_color*` / `edge_color*` slices restrict the matching: a
/// pattern vertex (edge) may map onto a target vertex (edge) only if their
/// colours are equal. Pass `None` for uncoloured graphs; supplying a colour for
/// only one side makes that colour be ignored (matching upstream).
///
/// On success [`Vf2Subisomorphism::iso`] tells whether an embedding exists; when
/// it does, `map21` holds the embedding (pattern → target) and `map12` its
/// reverse view, otherwise both are empty.
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
/// use rust_igraph::{Graph, subisomorphic_vf2};
///
/// // A triangle (pattern) sits inside K4 (target).
/// let mut k4 = Graph::new(4, false).unwrap();
/// for i in 0..4u32 {
///     for j in (i + 1)..4 {
///         k4.add_edge(i, j).unwrap();
///     }
/// }
/// let mut tri = Graph::new(3, false).unwrap();
/// tri.add_edge(0, 1).unwrap();
/// tri.add_edge(1, 2).unwrap();
/// tri.add_edge(2, 0).unwrap();
/// let r = subisomorphic_vf2(&k4, &tri, None, None, None, None).unwrap();
/// assert!(r.iso);
/// assert_eq!(r.map21.len(), 3);
/// ```
#[allow(clippy::too_many_arguments)]
pub fn subisomorphic_vf2(
    graph1: &Graph,
    graph2: &Graph,
    vertex_color1: Option<&[u32]>,
    vertex_color2: Option<&[u32]>,
    edge_color1: Option<&[u32]>,
    edge_color2: Option<&[u32]>,
) -> IgraphResult<Vf2Subisomorphism> {
    let mut map21: Vec<u32> = Vec::new();
    let mut map12: Vec<Option<u32>> = Vec::new();
    let mut iso = false;

    subiso_engine(
        graph1,
        graph2,
        vertex_color1,
        vertex_color2,
        edge_color1,
        edge_color2,
        |core_1, core_2| {
            iso = true;
            map21 = core_2.iter().map(|&x| x as u32).collect();
            map12 = core_1
                .iter()
                .map(|&x| if x < 0 { None } else { Some(x as u32) })
                .collect();
            Flow::Stop
        },
    )?;

    if !iso {
        map21.clear();
        map12.clear();
    }
    Ok(Vf2Subisomorphism { iso, map21, map12 })
}

/// Count the number of VF2 subgraph-isomorphic embeddings of the pattern
/// `graph2` into the target `graph1`.
///
/// Colour arguments behave as in [`subisomorphic_vf2`].
///
/// # Errors
///
/// Same conditions as [`subisomorphic_vf2`].
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, count_subisomorphisms_vf2};
///
/// // A single edge embeds into a triangle in 6 ways (3 edges x 2 directions).
/// let mut tri = Graph::new(3, false).unwrap();
/// tri.add_edge(0, 1).unwrap();
/// tri.add_edge(1, 2).unwrap();
/// tri.add_edge(2, 0).unwrap();
/// let mut edge = Graph::new(2, false).unwrap();
/// edge.add_edge(0, 1).unwrap();
/// let count = count_subisomorphisms_vf2(&tri, &edge, None, None, None, None).unwrap();
/// assert_eq!(count, 6);
/// ```
#[allow(clippy::too_many_arguments)]
pub fn count_subisomorphisms_vf2(
    graph1: &Graph,
    graph2: &Graph,
    vertex_color1: Option<&[u32]>,
    vertex_color2: Option<&[u32]>,
    edge_color1: Option<&[u32]>,
    edge_color2: Option<&[u32]>,
) -> IgraphResult<u64> {
    let mut count = 0u64;
    subiso_engine(
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

/// Collect every VF2 subgraph-isomorphic embedding of the pattern `graph2` into
/// the target `graph1`.
///
/// Each returned vector is a `map21` embedding: position `j` holds the target
/// vertex that pattern vertex `j` maps to. The list is empty when the pattern
/// does not embed. Colour arguments behave as in [`subisomorphic_vf2`].
///
/// # Errors
///
/// Same conditions as [`subisomorphic_vf2`].
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, get_subisomorphisms_vf2};
///
/// // A path P3 embeds into a 4-cycle in 8 ways.
/// let mut c4 = Graph::new(4, false).unwrap();
/// c4.add_edge(0, 1).unwrap();
/// c4.add_edge(1, 2).unwrap();
/// c4.add_edge(2, 3).unwrap();
/// c4.add_edge(3, 0).unwrap();
/// let mut p3 = Graph::new(3, false).unwrap();
/// p3.add_edge(0, 1).unwrap();
/// p3.add_edge(1, 2).unwrap();
/// let maps = get_subisomorphisms_vf2(&c4, &p3, None, None, None, None).unwrap();
/// assert_eq!(maps.len(), 8);
/// ```
#[allow(clippy::too_many_arguments)]
pub fn get_subisomorphisms_vf2(
    graph1: &Graph,
    graph2: &Graph,
    vertex_color1: Option<&[u32]>,
    vertex_color2: Option<&[u32]>,
    edge_color1: Option<&[u32]>,
    edge_color2: Option<&[u32]>,
) -> IgraphResult<Vec<Vec<u32>>> {
    let mut maps: Vec<Vec<u32>> = Vec::new();
    subiso_engine(
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

    /// Undirected (or directed) ring on `n` vertices: 0-1-...-(n-1)-0.
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

    /// Complete graph `K_n` (undirected).
    fn complete(n: u32) -> Graph {
        let mut g = Graph::new(n, false).expect("graph");
        for i in 0..n {
            for j in (i + 1)..n {
                g.add_edge(i, j).expect("edge");
            }
        }
        g
    }

    fn triangle() -> Graph {
        graph_from(3, false, &[(0, 1), (1, 2), (2, 0)])
    }

    #[test]
    fn triangle_embeds_into_k4() {
        let k4 = complete(4);
        let tri = triangle();
        let r = subisomorphic_vf2(&k4, &tri, None, None, None, None).expect("ok");
        assert!(r.iso);
        assert_eq!(r.map21.len(), 3);
        // The embedding must preserve adjacency: every pattern edge maps to a
        // target edge.
        for e in 0..tri.ecount() {
            let eid = u32::try_from(e).expect("fits");
            let (u, v) = tri.edge(eid).expect("edge");
            let mu = r.map21[u as usize];
            let mv = r.map21[v as usize];
            assert!(k4.find_eid(mu, mv).expect("lookup").is_some());
        }
        // map12 is the reverse view of map21.
        for (j, &t) in r.map21.iter().enumerate() {
            assert_eq!(r.map12[t as usize], Some(j as u32));
        }
    }

    #[test]
    fn triangle_count_into_k4_and_k5() {
        // K4 has 4 triangles, each with 3! = 6 orderings -> 24.
        assert_eq!(
            count_subisomorphisms_vf2(&complete(4), &triangle(), None, None, None, None)
                .expect("ok"),
            24
        );
        // K5 has 10 triangles x 6 = 60.
        assert_eq!(
            count_subisomorphisms_vf2(&complete(5), &triangle(), None, None, None, None)
                .expect("ok"),
            60
        );
    }

    #[test]
    fn edge_count_into_triangle_and_path() {
        let edge = graph_from(2, false, &[(0, 1)]);
        // A triangle has 3 edges, each embeddable in 2 orientations -> 6.
        assert_eq!(
            count_subisomorphisms_vf2(&triangle(), &edge, None, None, None, None).expect("ok"),
            6
        );
        // P4 (path on 4 vertices) has 3 edges -> 6 embeddings of a single edge.
        let p4 = graph_from(4, false, &[(0, 1), (1, 2), (2, 3)]);
        assert_eq!(
            count_subisomorphisms_vf2(&p4, &edge, None, None, None, None).expect("ok"),
            6
        );
    }

    #[test]
    fn path_embeds_into_cycle() {
        // P3 (path on 3 vertices, 2 edges) embeds into C4 in 8 ways.
        let c4 = ring(4, false);
        let p3 = graph_from(3, false, &[(0, 1), (1, 2)]);
        let maps = get_subisomorphisms_vf2(&c4, &p3, None, None, None, None).expect("ok");
        assert_eq!(maps.len(), 8);
        let c = count_subisomorphisms_vf2(&c4, &p3, None, None, None, None).expect("ok");
        assert_eq!(c, 8);
    }

    #[test]
    fn triangle_not_in_cycle() {
        // A 4-cycle is triangle-free.
        let c4 = ring(4, false);
        let r = subisomorphic_vf2(&c4, &triangle(), None, None, None, None).expect("ok");
        assert!(!r.iso);
        assert!(r.map21.is_empty());
        assert!(r.map12.is_empty());
        assert_eq!(
            count_subisomorphisms_vf2(&c4, &triangle(), None, None, None, None).expect("ok"),
            0
        );
    }

    #[test]
    fn pattern_larger_than_target_has_no_embedding() {
        let tri = triangle();
        let edge = graph_from(2, false, &[(0, 1)]);
        // More pattern vertices than target vertices.
        let r = subisomorphic_vf2(&edge, &tri, None, None, None, None).expect("ok");
        assert!(!r.iso);
        assert_eq!(
            count_subisomorphisms_vf2(&edge, &tri, None, None, None, None).expect("ok"),
            0
        );
        // Equal vertices, more pattern edges than target edges.
        let path = graph_from(3, false, &[(0, 1), (1, 2)]);
        assert_eq!(
            count_subisomorphisms_vf2(&path, &triangle(), None, None, None, None).expect("ok"),
            0
        );
    }

    #[test]
    fn self_subiso_equals_automorphism_count() {
        // When pattern == target, every subgraph isomorphism is an
        // automorphism, so the count matches ISO-001's verified values.
        assert_eq!(
            count_subisomorphisms_vf2(&ring(6, false), &ring(6, false), None, None, None, None)
                .expect("ok"),
            12
        );
        assert_eq!(
            count_subisomorphisms_vf2(&ring(4, false), &ring(4, false), None, None, None, None)
                .expect("ok"),
            8
        );
        assert_eq!(
            count_subisomorphisms_vf2(&ring(4, true), &ring(4, true), None, None, None, None)
                .expect("ok"),
            4
        );
    }

    #[test]
    fn empty_pattern_embeds_once() {
        // The empty pattern embeds into any target in exactly one (empty) way.
        let empty = Graph::new(0, false).expect("graph");
        let tri = triangle();
        let r = subisomorphic_vf2(&tri, &empty, None, None, None, None).expect("ok");
        assert!(r.iso);
        assert!(r.map21.is_empty());
        assert_eq!(
            count_subisomorphisms_vf2(&tri, &empty, None, None, None, None).expect("ok"),
            1
        );
    }

    #[test]
    fn directed_pattern_respects_orientation() {
        // Directed path 0->1->2 embeds into a directed triangle 0->1->2->0
        // following the orientation; there are 3 such directed paths.
        let dtri = ring(3, true);
        let dpath = graph_from(3, true, &[(0, 1), (1, 2)]);
        assert_eq!(
            count_subisomorphisms_vf2(&dtri, &dpath, None, None, None, None).expect("ok"),
            3
        );
    }

    #[test]
    fn vertex_colors_constrain_embedding() {
        // Colour the target triangle's vertices distinctly and require the
        // single pattern edge's endpoints to carry colours 0 and 1.
        let tri = triangle();
        let tcolors = [0u32, 1, 2];
        let edge = graph_from(2, false, &[(0, 1)]);
        let pcolors = [0u32, 1];
        // Only the (0,1) target edge matches, in one orientation -> 1.
        let c = count_subisomorphisms_vf2(&tri, &edge, Some(&tcolors), Some(&pcolors), None, None)
            .expect("ok");
        assert_eq!(c, 1);
    }

    #[test]
    fn only_one_side_colored_ignores_colors() {
        let tri = triangle();
        let tcolors = [0u32, 1, 2];
        let edge = graph_from(2, false, &[(0, 1)]);
        // Pattern uncoloured -> colours ignored, full 6 embeddings restored.
        let c =
            count_subisomorphisms_vf2(&tri, &edge, Some(&tcolors), None, None, None).expect("ok");
        assert_eq!(c, 6);
    }

    #[test]
    fn edge_colors_constrain_embedding() {
        // Triangle target with one distinctly coloured edge; a single
        // coloured pattern edge can only land on the matching target edge.
        let tri = triangle();
        let mut tec = vec![0u32; tri.ecount()];
        for (e, slot) in tec.iter_mut().enumerate() {
            let eid = u32::try_from(e).expect("fits");
            let (a, b) = tri.edge(eid).expect("edge");
            if (a, b) == (1, 2) || (a, b) == (2, 1) {
                *slot = 1;
            }
        }
        let edge = graph_from(2, false, &[(0, 1)]);
        let pec = [1u32];
        // Only the colour-1 target edge matches, in 2 orientations -> 2.
        let c =
            count_subisomorphisms_vf2(&tri, &edge, None, None, Some(&tec), Some(&pec)).expect("ok");
        assert_eq!(c, 2);
    }

    #[test]
    fn get_and_count_agree() {
        let k4 = complete(4);
        let tri = triangle();
        let maps = get_subisomorphisms_vf2(&k4, &tri, None, None, None, None).expect("ok");
        let c = count_subisomorphisms_vf2(&k4, &tri, None, None, None, None).expect("ok");
        assert_eq!(maps.len() as u64, c);
        // Every returned embedding preserves pattern adjacency.
        for m in &maps {
            for e in 0..tri.ecount() {
                let eid = u32::try_from(e).expect("fits");
                let (u, v) = tri.edge(eid).expect("edge");
                assert!(
                    k4.find_eid(m[u as usize], m[v as usize])
                        .expect("lookup")
                        .is_some()
                );
            }
        }
    }

    #[test]
    fn self_loops_are_rejected() {
        let g = graph_from(2, false, &[(0, 0), (0, 1)]);
        let h = graph_from(2, false, &[(0, 1)]);
        assert!(subisomorphic_vf2(&g, &h, None, None, None, None).is_err());
        assert!(subisomorphic_vf2(&h, &g, None, None, None, None).is_err());
    }

    #[test]
    fn directedness_mismatch_is_rejected() {
        let u = ring(3, false);
        let d = ring(3, true);
        assert!(subisomorphic_vf2(&u, &d, None, None, None, None).is_err());
    }

    #[test]
    fn wrong_color_vector_length_errors() {
        let tri = triangle();
        let edge = graph_from(2, false, &[(0, 1)]);
        let short = [0u32];
        // Target vertex colour vector too short.
        assert!(
            subisomorphic_vf2(&tri, &edge, Some(&short), Some(&[0u32, 0]), None, None).is_err()
        );
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::HashSet;

    /// Generate a simple, loopless graph: `n` vertices and a deduplicated,
    /// self-loop-free edge set, with a directedness flag.
    fn arb_simple_graph(max_v: u32) -> impl Strategy<Value = (u32, Vec<(u32, u32)>, bool)> {
        (1..=max_v, any::<bool>()).prop_flat_map(|(n, directed)| {
            proptest::collection::vec((0..n, 0..n), 0..=12).prop_map(move |raw| {
                let mut seen: HashSet<(u32, u32)> = HashSet::new();
                let mut edges = Vec::new();
                for (u, v) in raw {
                    if u == v {
                        continue;
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
        /// A graph always subgraph-embeds into itself (the identity is one such
        /// embedding), so the self-count equals the automorphism count and is
        /// at least 1.
        #[test]
        fn self_embeds((n, edges, directed) in arb_simple_graph(6)) {
            let g = build(n, directed, &edges);
            let r = subisomorphic_vf2(&g, &g, None, None, None, None).expect("ok");
            prop_assert!(r.iso);
            prop_assert_eq!(r.map21.len(), n as usize);
            let c = count_subisomorphisms_vf2(&g, &g, None, None, None, None).expect("ok");
            prop_assert!(c >= 1, "identity embedding always exists");
        }

        /// `get_subisomorphisms_vf2` returns exactly as many embeddings as
        /// `count_subisomorphisms_vf2`, and every embedding preserves the
        /// pattern's adjacency in the target.
        #[test]
        fn get_count_consistency_and_adjacency(
            (n, edges, directed) in arb_simple_graph(6),
        ) {
            let g = build(n, directed, &edges);
            let maps = get_subisomorphisms_vf2(&g, &g, None, None, None, None).expect("ok");
            let c = count_subisomorphisms_vf2(&g, &g, None, None, None, None).expect("ok");
            prop_assert_eq!(maps.len() as u64, c);
            for m in &maps {
                prop_assert_eq!(m.len(), n as usize);
                for e in 0..g.ecount() {
                    let eid = u32::try_from(e).expect("fits");
                    let (u, v) = g.edge(eid).expect("edge");
                    prop_assert!(
                        g.find_eid(m[u as usize], m[v as usize]).expect("lookup").is_some()
                    );
                }
            }
        }

        /// Relabelling the target by a random permutation preserves the number
        /// of embeddings of a fixed pattern.
        #[test]
        fn count_invariant_under_target_relabelling(
            (n, edges, directed) in arb_simple_graph(6),
            seed in any::<u64>(),
        ) {
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
            // Use a single edge as the fixed pattern (well-defined whenever the
            // graph has at least one edge); else the empty-vertex pattern.
            let pattern = build(2, directed, &[(0, 1)]);
            let cg = count_subisomorphisms_vf2(&g, &pattern, None, None, None, None).expect("ok");
            let ch = count_subisomorphisms_vf2(&h, &pattern, None, None, None, None).expect("ok");
            prop_assert_eq!(cg, ch);
        }

        /// A pattern with more vertices or edges than the target never embeds.
        #[test]
        fn oversized_pattern_never_embeds(
            (n, edges, directed) in arb_simple_graph(5),
        ) {
            let target = build(n, directed, &edges);
            // Pattern = target plus one extra isolated vertex: strictly more
            // vertices, so no embedding can exist.
            let bigger = build(n + 1, directed, &edges);
            let c = count_subisomorphisms_vf2(&target, &bigger, None, None, None, None)
                .expect("ok");
            prop_assert_eq!(c, 0);
        }
    }
}

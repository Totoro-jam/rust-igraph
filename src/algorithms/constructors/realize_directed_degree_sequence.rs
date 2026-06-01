//! Directed degree sequence realization (ALGO-CO-034).
//!
//! Counterpart of the directed branch of `igraph_realize_degree_sequence()`
//! from `references/igraph/src/misc/degree_sequence.cpp`.
//!
//! Constructs a directed simple graph from given out-degree and in-degree
//! sequences using the Kleitman-Wang algorithm.

use crate::algorithms::constructors::realize_degree_sequence::RealizeDegseqMethod;
use crate::core::error::IgraphError;
use crate::core::{Graph, IgraphResult, VertexId};

/// Realize a directed simple graph from out-degree and in-degree sequences.
///
/// Uses the Kleitman-Wang algorithm: repeatedly select a vertex, then
/// connect its out-stubs to the vertices with the highest remaining
/// in-degrees (lexicographically by (in, out) degree pairs).
///
/// # Arguments
///
/// * `outdeg` — the target out-degree for each vertex.
/// * `indeg` — the target in-degree for each vertex. Must have the same
///   length as `outdeg`.
/// * `method` — vertex selection order (see [`RealizeDegseqMethod`]).
///
/// # Errors
///
/// Returns `InvalidArgument` if:
/// - `outdeg` and `indeg` have different lengths.
/// - The sums of out-degrees and in-degrees differ.
/// - Any out-degree exceeds `n - 1` (no simple realization exists).
/// - The sequence is not digraphical (Kleitman-Wang fails).
///
/// # Examples
///
/// ```
/// use rust_igraph::{realize_directed_degree_sequence, RealizeDegseqMethod};
///
/// // Realize a directed cycle: each vertex has in=1, out=1
/// let g = realize_directed_degree_sequence(
///     &[1, 1, 1], &[1, 1, 1], RealizeDegseqMethod::Smallest,
/// ).unwrap();
/// assert_eq!(g.vcount(), 3);
/// assert_eq!(g.ecount(), 3);
/// assert!(g.is_directed());
/// ```
pub fn realize_directed_degree_sequence(
    outdeg: &[u32],
    indeg: &[u32],
    method: RealizeDegseqMethod,
) -> IgraphResult<Graph> {
    let n = outdeg.len();

    if indeg.len() != n {
        return Err(IgraphError::InvalidArgument(
            "in- and out-degree sequences must have the same length".to_string(),
        ));
    }

    if n == 0 {
        return Graph::new(0, true);
    }

    let n_u32 = u32::try_from(n)
        .map_err(|_| IgraphError::InvalidArgument("vertex count exceeds u32::MAX".to_string()))?;

    let out_sum: u64 = outdeg.iter().map(|&d| u64::from(d)).sum();
    let in_sum: u64 = indeg.iter().map(|&d| u64::from(d)).sum();

    if out_sum != in_sum {
        return Err(IgraphError::InvalidArgument(format!(
            "out-degree sum ({out_sum}) differs from in-degree sum ({in_sum})"
        )));
    }

    for (i, &d) in outdeg.iter().enumerate() {
        if d >= n_u32 {
            return Err(IgraphError::InvalidArgument(format!(
                "outdeg[{i}] = {d} >= n = {n_u32}, cannot realize as simple directed graph"
            )));
        }
    }

    let num_edges = usize::try_from(out_sum)
        .map_err(|_| IgraphError::InvalidArgument("edge count overflows usize".to_string()))?;

    if num_edges == 0 {
        return Graph::new(n_u32, true);
    }

    let edges = match method {
        RealizeDegseqMethod::Smallest => kleitman_wang(outdeg, indeg, n, true)?,
        RealizeDegseqMethod::Largest => kleitman_wang(outdeg, indeg, n, false)?,
        RealizeDegseqMethod::Index => kleitman_wang_index(outdeg, indeg, n)?,
    };

    let mut graph = Graph::new(n_u32, true)?;
    graph.add_edges(edges)?;
    Ok(graph)
}

/// (vertex, `in_degree`, `out_degree`) triple.
#[derive(Clone, Copy)]
struct Vbd {
    vertex: u32,
    ind: u32,
    outd: u32,
}

impl Vbd {
    fn bideg_key(&self) -> (u32, u32) {
        (self.ind, self.outd)
    }
}

fn kleitman_wang(
    outdeg: &[u32],
    indeg: &[u32],
    n: usize,
    smallest: bool,
) -> IgraphResult<Vec<(VertexId, VertexId)>> {
    let mut vertices: Vec<Vbd> = (0..n)
        .map(|i| {
            #[allow(clippy::cast_possible_truncation)]
            let idx = i as u32;
            Vbd {
                vertex: idx,
                ind: indeg[i],
                outd: outdeg[i],
            }
        })
        .collect();

    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();

    loop {
        // Sort by (in, out) descending
        vertices.sort_unstable_by_key(|v| std::cmp::Reverse(v.bideg_key()));

        // Remove (0,0)-degree vertices from the tail
        while let Some(last) = vertices.last() {
            if last.ind == 0 && last.outd == 0 {
                vertices.pop();
            } else {
                break;
            }
        }

        if vertices.is_empty() {
            break;
        }

        // Find a vertex with non-zero out-degree
        let vdp_idx = if smallest {
            vertices.iter().rposition(|v| v.outd > 0).ok_or_else(|| {
                IgraphError::InvalidArgument("directed degree sequence not digraphical".to_string())
            })?
        } else {
            vertices.iter().position(|v| v.outd > 0).ok_or_else(|| {
                IgraphError::InvalidArgument("directed degree sequence not digraphical".to_string())
            })?
        };

        let hub_vertex = vertices[vdp_idx].vertex;
        let hub_outd = vertices[vdp_idx].outd;

        // Check sufficient vertices to connect to (excluding self)
        let available = vertices.iter().filter(|v| v.vertex != hub_vertex).count();
        if (hub_outd as usize) > available {
            return Err(IgraphError::InvalidArgument(
                "directed degree sequence not digraphical (insufficient vertices)".to_string(),
            ));
        }

        // Connect hub's out-stubs to vertices with highest in-degree
        let mut k: u32 = 0;
        for vbd in &mut vertices {
            if k >= hub_outd {
                break;
            }
            if vbd.vertex == hub_vertex {
                continue;
            }
            if vbd.ind == 0 {
                return Err(IgraphError::InvalidArgument(
                    "directed degree sequence not digraphical".to_string(),
                ));
            }
            vbd.ind -= 1;
            edges.push((hub_vertex, vbd.vertex));
            k += 1;
        }

        vertices[vdp_idx].outd = 0;
    }

    Ok(edges)
}

fn kleitman_wang_index(
    outdeg: &[u32],
    indeg: &[u32],
    n: usize,
) -> IgraphResult<Vec<(VertexId, VertexId)>> {
    // Build vertices with their original indices for index-order traversal
    let mut vertices: Vec<Vbd> = (0..n)
        .map(|i| {
            #[allow(clippy::cast_possible_truncation)]
            let idx = i as u32;
            Vbd {
                vertex: idx,
                ind: indeg[i],
                outd: outdeg[i],
            }
        })
        .collect();

    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();

    #[allow(clippy::cast_possible_truncation)]
    let n_u32 = n as u32;
    for vi in 0..n_u32 {
        // Sort by (in, out) descending
        vertices.sort_unstable_by_key(|v| std::cmp::Reverse(v.bideg_key()));

        // Find this vertex in the sorted list
        let Some(vdp_idx) = vertices.iter().position(|v| v.vertex == vi) else {
            continue;
        };

        let hub_outd = vertices[vdp_idx].outd;
        if hub_outd == 0 {
            continue;
        }

        let hub_vertex = vertices[vdp_idx].vertex;

        // Connect hub's out-stubs to vertices with highest in-degree
        let mut k: u32 = 0;
        for vbd in &mut vertices {
            if k >= hub_outd {
                break;
            }
            if vbd.vertex == hub_vertex {
                continue;
            }
            if vbd.ind == 0 {
                return Err(IgraphError::InvalidArgument(
                    "directed degree sequence not digraphical".to_string(),
                ));
            }
            vbd.ind -= 1;
            edges.push((hub_vertex, vbd.vertex));
            k += 1;
        }

        if k < hub_outd {
            return Err(IgraphError::InvalidArgument(
                "directed degree sequence not digraphical".to_string(),
            ));
        }

        vertices[vdp_idx].outd = 0;
    }

    Ok(edges)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verify_directed_degrees(graph: &Graph, outdeg: &[u32], indeg: &[u32]) {
        let n = graph.vcount();
        assert_eq!(n as usize, outdeg.len());
        assert_eq!(n as usize, indeg.len());
        for v in 0..n {
            let out_d = graph.incident(v).unwrap().len();
            let in_d = graph.incident_in(v).unwrap().len();
            assert_eq!(
                out_d, outdeg[v as usize] as usize,
                "vertex {v}: out-degree {out_d}, expected {}",
                outdeg[v as usize]
            );
            assert_eq!(
                in_d, indeg[v as usize] as usize,
                "vertex {v}: in-degree {in_d}, expected {}",
                indeg[v as usize]
            );
        }
    }

    #[test]
    fn empty_sequence() {
        let g = realize_directed_degree_sequence(&[], &[], RealizeDegseqMethod::Largest).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
        assert!(g.is_directed());
    }

    #[test]
    fn all_zeros() {
        let g =
            realize_directed_degree_sequence(&[0, 0, 0], &[0, 0, 0], RealizeDegseqMethod::Largest)
                .unwrap();
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn single_edge() {
        let g = realize_directed_degree_sequence(&[1, 0], &[0, 1], RealizeDegseqMethod::Largest)
            .unwrap();
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 1);
        verify_directed_degrees(&g, &[1, 0], &[0, 1]);
    }

    #[test]
    fn directed_cycle() {
        let g =
            realize_directed_degree_sequence(&[1, 1, 1], &[1, 1, 1], RealizeDegseqMethod::Smallest)
                .unwrap();
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 3);
        verify_directed_degrees(&g, &[1, 1, 1], &[1, 1, 1]);
    }

    #[test]
    fn complete_directed_k3() {
        // K3 directed: each vertex has out=2, in=2
        let g =
            realize_directed_degree_sequence(&[2, 2, 2], &[2, 2, 2], RealizeDegseqMethod::Largest)
                .unwrap();
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 6);
        verify_directed_degrees(&g, &[2, 2, 2], &[2, 2, 2]);
    }

    #[test]
    fn star_out() {
        // Star: vertex 0 sends to all others
        let g = realize_directed_degree_sequence(
            &[3, 0, 0, 0],
            &[0, 1, 1, 1],
            RealizeDegseqMethod::Largest,
        )
        .unwrap();
        assert_eq!(g.vcount(), 4);
        assert_eq!(g.ecount(), 3);
        verify_directed_degrees(&g, &[3, 0, 0, 0], &[0, 1, 1, 1]);
    }

    #[test]
    fn mismatched_sums() {
        let result =
            realize_directed_degree_sequence(&[1, 1], &[1, 2], RealizeDegseqMethod::Largest);
        assert!(result.is_err());
    }

    #[test]
    fn different_lengths() {
        let result = realize_directed_degree_sequence(&[1, 1], &[1], RealizeDegseqMethod::Largest);
        assert!(result.is_err());
    }

    #[test]
    fn non_digraphical() {
        // out=[2,0,0], in=[0,0,2]: vertex 0 needs 2 targets but vertex 2
        // can only receive 2, and vertex 0 can only send to 2 others.
        // Actually this IS graphical: 0->2, 0->2 would be multi-edge.
        // For simple: 0 can send to vertex 1 and 2, but vertex 1 has in=0.
        let result =
            realize_directed_degree_sequence(&[2, 0, 0], &[0, 0, 2], RealizeDegseqMethod::Largest);
        assert!(result.is_err());
    }

    #[test]
    fn method_index() {
        let g =
            realize_directed_degree_sequence(&[1, 1, 1], &[1, 1, 1], RealizeDegseqMethod::Index)
                .unwrap();
        assert_eq!(g.ecount(), 3);
        verify_directed_degrees(&g, &[1, 1, 1], &[1, 1, 1]);
    }

    #[test]
    fn method_smallest() {
        let g =
            realize_directed_degree_sequence(&[2, 1, 1], &[1, 1, 2], RealizeDegseqMethod::Smallest)
                .unwrap();
        assert_eq!(g.ecount(), 4);
        verify_directed_degrees(&g, &[2, 1, 1], &[1, 1, 2]);
    }

    #[test]
    fn larger_directed() {
        let outdeg = [2, 2, 1, 1, 0];
        let indeg = [1, 1, 1, 1, 2];
        let g = realize_directed_degree_sequence(&outdeg, &indeg, RealizeDegseqMethod::Largest)
            .unwrap();
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 6);
        verify_directed_degrees(&g, &outdeg, &indeg);
    }

    #[test]
    fn degree_too_large() {
        let result =
            realize_directed_degree_sequence(&[3, 0, 0], &[0, 1, 2], RealizeDegseqMethod::Largest);
        assert!(result.is_err());
    }

    #[test]
    fn all_methods_produce_valid_graph() {
        let outdeg = [2, 1, 1, 2];
        let indeg = [1, 2, 2, 1];
        for method in [
            RealizeDegseqMethod::Largest,
            RealizeDegseqMethod::Smallest,
            RealizeDegseqMethod::Index,
        ] {
            let g = realize_directed_degree_sequence(&outdeg, &indeg, method).unwrap();
            verify_directed_degrees(&g, &outdeg, &indeg);
        }
    }
}

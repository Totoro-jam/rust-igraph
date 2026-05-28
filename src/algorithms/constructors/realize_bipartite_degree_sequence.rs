//! Bipartite degree sequence realization (ALGO-CO-034).
//!
//! Counterpart of `igraph_realize_bipartite_degree_sequence()` from
//! `references/igraph/src/misc/degree_sequence.cpp`.
//!
//! Constructs a bipartite simple graph with a given bidegree sequence
//! using a Havel-Hakimi–like algorithm.

use crate::core::error::IgraphError;
use crate::core::{Graph, IgraphResult, VertexId};

use super::realize_degree_sequence::RealizeDegseqMethod;

/// Realize a bipartite simple graph from two degree sequences.
///
/// Uses a Havel-Hakimi–like algorithm: repeatedly pick a vertex from
/// one partition and connect it to the vertices with the highest
/// remaining degrees in the opposite partition.
///
/// The resulting graph has `degrees1.len() + degrees2.len()` vertices.
/// Vertices `0..n1` belong to partition 1 and vertices `n1..n1+n2` to
/// partition 2.
///
/// # Arguments
///
/// * `degrees1` — target degrees for partition-1 vertices.
/// * `degrees2` — target degrees for partition-2 vertices.
/// * `method` — vertex selection order (see [`RealizeDegseqMethod`]).
///
/// # Errors
///
/// Returns `InvalidArgument` if:
/// - The sums of `degrees1` and `degrees2` differ.
/// - The bidegree sequence is not bigraphical.
///
/// # Examples
///
/// ```
/// use rust_igraph::{realize_bipartite_degree_sequence, RealizeDegseqMethod};
///
/// // Complete bipartite K_{2,3}: partition 1 has degrees [3,3], partition 2 has [2,2,2]
/// let g = realize_bipartite_degree_sequence(&[3, 3], &[2, 2, 2], RealizeDegseqMethod::Largest).unwrap();
/// assert_eq!(g.vcount(), 5);
/// assert_eq!(g.ecount(), 6);
/// ```
pub fn realize_bipartite_degree_sequence(
    degrees1: &[u32],
    degrees2: &[u32],
    method: RealizeDegseqMethod,
) -> IgraphResult<Graph> {
    let n1 = degrees1.len();
    let n2 = degrees2.len();
    let n = n1.checked_add(n2).ok_or_else(|| {
        IgraphError::InvalidArgument("total vertex count overflows usize".to_string())
    })?;

    if n == 0 {
        return Graph::new(0, false);
    }

    let n_u32 = u32::try_from(n)
        .map_err(|_| IgraphError::InvalidArgument("vertex count exceeds u32::MAX".to_string()))?;

    let sum1: u64 = degrees1.iter().map(|&d| u64::from(d)).sum();
    let sum2: u64 = degrees2.iter().map(|&d| u64::from(d)).sum();

    if sum1 != sum2 {
        return Err(IgraphError::InvalidArgument(
            "degree sums of the two partitions must be equal".to_string(),
        ));
    }

    if sum1 == 0 {
        return Graph::new(n_u32, false);
    }

    #[allow(clippy::cast_possible_truncation)]
    let num_edges = sum1 as usize;

    let offset = u32::try_from(n1)
        .map_err(|_| IgraphError::InvalidArgument("partition 1 too large".to_string()))?;

    // Build (vertex_id, remaining_degree) pairs for each partition.
    // Partition 2 vertices are offset by n1.
    let mut vd1: Vec<(u32, u32)> = degrees1
        .iter()
        .enumerate()
        .map(|(i, &d)| {
            #[allow(clippy::cast_possible_truncation)]
            let idx = i as u32;
            (idx, d)
        })
        .collect();

    let mut vd2: Vec<(u32, u32)> = degrees2
        .iter()
        .enumerate()
        .map(|(i, &d)| {
            #[allow(clippy::cast_possible_truncation)]
            let idx = (i as u32)
                .checked_add(offset)
                .ok_or_else(|| IgraphError::InvalidArgument("vertex index overflow".to_string()));
            (idx.unwrap_or(0), d)
        })
        .collect();

    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(num_edges);

    match method {
        RealizeDegseqMethod::Largest => {
            realize_largest_or_smallest(&mut vd1, &mut vd2, &mut edges, true)?;
        }
        RealizeDegseqMethod::Smallest => {
            realize_largest_or_smallest(&mut vd1, &mut vd2, &mut edges, false)?;
        }
        RealizeDegseqMethod::Index => realize_index(&mut vd1, &mut vd2, &mut edges)?,
    }

    let mut graph = Graph::new(n_u32, false)?;
    graph.add_edges(edges)?;
    Ok(graph)
}

fn realize_largest_or_smallest(
    vd1: &mut Vec<(u32, u32)>,
    vd2: &mut Vec<(u32, u32)>,
    edges: &mut Vec<(VertexId, VertexId)>,
    largest: bool,
) -> IgraphResult<()> {
    while !vd1.is_empty() && !vd2.is_empty() {
        // Sort both partitions by degree descending
        vd1.sort_unstable_by(|a, b| b.1.cmp(&a.1));
        vd2.sort_unstable_by(|a, b| b.1.cmp(&a.1));

        // Select the hub: from the partition that has the largest (or smallest) degree vertex
        let (src_vs, dest_vs) = if largest {
            let max1 = vd1[0].1;
            let max2 = vd2[0].1;
            if max1 >= max2 {
                (vd1 as &mut Vec<(u32, u32)>, vd2 as &mut Vec<(u32, u32)>)
            } else {
                (vd2 as &mut Vec<(u32, u32)>, vd1 as &mut Vec<(u32, u32)>)
            }
        } else {
            let min1 = vd1.last().map_or(0, |v| v.1);
            let min2 = vd2.last().map_or(0, |v| v.1);
            if min1 <= min2 {
                (vd1 as &mut Vec<(u32, u32)>, vd2 as &mut Vec<(u32, u32)>)
            } else {
                (vd2 as &mut Vec<(u32, u32)>, vd1 as &mut Vec<(u32, u32)>)
            }
        };

        let hub = if largest {
            let h = src_vs[0];
            src_vs.remove(0);
            h
        } else {
            src_vs.pop().unwrap_or((0, 0))
        };

        if hub.1 == 0 {
            continue;
        }

        let d = hub.1 as usize;
        if d > dest_vs.len() {
            return Err(IgraphError::InvalidArgument(
                "bidegree sequence is not bigraphical (not realizable as simple bipartite graph)"
                    .to_string(),
            ));
        }

        // Connect hub to the d vertices with highest degree in the opposite partition
        // dest_vs is already sorted descending
        for item in dest_vs.iter_mut().take(d) {
            if item.1 == 0 {
                return Err(IgraphError::InvalidArgument(
                    "bidegree sequence is not bigraphical (not realizable as simple bipartite graph)"
                        .to_string(),
                ));
            }
            edges.push((hub.0, item.0));
            item.1 -= 1;
        }
    }

    // Verify all remaining degrees are zero
    for vs in [&*vd1, &*vd2] {
        for &(v, d) in vs {
            if d != 0 {
                return Err(IgraphError::InvalidArgument(format!(
                    "bidegree sequence not bigraphical: vertex {v} has {d} remaining degree"
                )));
            }
        }
    }

    Ok(())
}

fn realize_index(
    vd1: &mut Vec<(u32, u32)>,
    vd2: &mut Vec<(u32, u32)>,
    edges: &mut Vec<(VertexId, VertexId)>,
) -> IgraphResult<()> {
    // In index mode, always pick from partition 1 in order, connect to
    // highest-degree vertices in partition 2.
    while !vd1.is_empty() && !vd2.is_empty() {
        // Sort partition 2 by degree descending
        vd2.sort_unstable_by(|a, b| b.1.cmp(&a.1));

        let hub = vd1.remove(0);

        if hub.1 == 0 {
            continue;
        }

        let d = hub.1 as usize;
        if d > vd2.len() {
            return Err(IgraphError::InvalidArgument(
                "bidegree sequence is not bigraphical (not realizable as simple bipartite graph)"
                    .to_string(),
            ));
        }

        for item in vd2.iter_mut().take(d) {
            if item.1 == 0 {
                return Err(IgraphError::InvalidArgument(
                    "bidegree sequence is not bigraphical (not realizable as simple bipartite graph)"
                        .to_string(),
                ));
            }
            edges.push((hub.0, item.0));
            item.1 -= 1;
        }
    }

    // Verify remaining degrees
    for vs in [&*vd1, &*vd2] {
        for &(v, d) in vs {
            if d != 0 {
                return Err(IgraphError::InvalidArgument(format!(
                    "bidegree sequence not bigraphical: vertex {v} has {d} remaining degree"
                )));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verify_bipartite_degrees(graph: &Graph, expected1: &[u32], expected2: &[u32]) {
        let n1 = expected1.len();
        let n2 = expected2.len();
        assert_eq!(graph.vcount() as usize, n1 + n2);

        for (i, &exp) in expected1.iter().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            let v = i as u32;
            let deg = graph.degree(v).unwrap();
            assert_eq!(
                deg, exp as usize,
                "partition 1 vertex {v}: got degree {deg}, expected {exp}"
            );
        }
        for (i, &exp) in expected2.iter().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            let v = (i + n1) as u32;
            let deg = graph.degree(v).unwrap();
            assert_eq!(
                deg, exp as usize,
                "partition 2 vertex {v}: got degree {deg}, expected {exp}"
            );
        }
    }

    fn verify_bipartite_structure(graph: &Graph, n1: usize) {
        // Every edge must cross partitions
        for eid in 0..graph.ecount() {
            #[allow(clippy::cast_possible_truncation)]
            let eid_u32 = eid as u32;
            let (s, t) = graph.edge(eid_u32).unwrap();
            let s_in_p1 = (s as usize) < n1;
            let t_in_p1 = (t as usize) < n1;
            assert_ne!(
                s_in_p1, t_in_p1,
                "edge ({s}, {t}) does not cross partitions (n1={n1})"
            );
        }
    }

    #[test]
    fn empty_sequences() {
        let g = realize_bipartite_degree_sequence(&[], &[], RealizeDegseqMethod::Largest).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn one_partition_empty() {
        let g =
            realize_bipartite_degree_sequence(&[0, 0], &[], RealizeDegseqMethod::Largest).unwrap();
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn all_zeros() {
        let g =
            realize_bipartite_degree_sequence(&[0, 0], &[0, 0, 0], RealizeDegseqMethod::Largest)
                .unwrap();
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn single_edge() {
        let g =
            realize_bipartite_degree_sequence(&[1], &[1], RealizeDegseqMethod::Largest).unwrap();
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 1);
        verify_bipartite_degrees(&g, &[1], &[1]);
        verify_bipartite_structure(&g, 1);
    }

    #[test]
    fn complete_bipartite_k23() {
        let g =
            realize_bipartite_degree_sequence(&[3, 3], &[2, 2, 2], RealizeDegseqMethod::Largest)
                .unwrap();
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 6);
        verify_bipartite_degrees(&g, &[3, 3], &[2, 2, 2]);
        verify_bipartite_structure(&g, 2);
    }

    #[test]
    fn complete_bipartite_k33() {
        let g =
            realize_bipartite_degree_sequence(&[3, 3, 3], &[3, 3, 3], RealizeDegseqMethod::Largest)
                .unwrap();
        assert_eq!(g.vcount(), 6);
        assert_eq!(g.ecount(), 9);
        verify_bipartite_degrees(&g, &[3, 3, 3], &[3, 3, 3]);
        verify_bipartite_structure(&g, 3);
    }

    #[test]
    fn star_bipartite() {
        // One vertex in p1 connected to all in p2
        let g =
            realize_bipartite_degree_sequence(&[4], &[1, 1, 1, 1], RealizeDegseqMethod::Largest)
                .unwrap();
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 4);
        verify_bipartite_degrees(&g, &[4], &[1, 1, 1, 1]);
        verify_bipartite_structure(&g, 1);
    }

    #[test]
    fn unequal_sums_fails() {
        let result =
            realize_bipartite_degree_sequence(&[2, 2], &[1, 1], RealizeDegseqMethod::Largest);
        assert!(result.is_err());
    }

    #[test]
    fn not_bigraphical_fails() {
        // Sum matches (4 = 4) but can't realize: one vertex needs degree 4
        // but partition 2 has only 2 vertices
        let result = realize_bipartite_degree_sequence(&[4], &[2, 2], RealizeDegseqMethod::Largest);
        assert!(result.is_err());
    }

    #[test]
    fn method_smallest() {
        let g = realize_bipartite_degree_sequence(
            &[2, 2, 1],
            &[1, 2, 2],
            RealizeDegseqMethod::Smallest,
        )
        .unwrap();
        assert_eq!(g.ecount(), 5);
        verify_bipartite_degrees(&g, &[2, 2, 1], &[1, 2, 2]);
        verify_bipartite_structure(&g, 3);
    }

    #[test]
    fn method_index() {
        let g =
            realize_bipartite_degree_sequence(&[2, 1, 1], &[2, 1, 1], RealizeDegseqMethod::Index)
                .unwrap();
        assert_eq!(g.ecount(), 4);
        verify_bipartite_degrees(&g, &[2, 1, 1], &[2, 1, 1]);
        verify_bipartite_structure(&g, 3);
    }

    #[test]
    fn larger_bipartite() {
        let g = realize_bipartite_degree_sequence(
            &[3, 3, 2, 2],
            &[2, 2, 2, 2, 2],
            RealizeDegseqMethod::Largest,
        )
        .unwrap();
        assert_eq!(g.vcount(), 9);
        assert_eq!(g.ecount(), 10);
        verify_bipartite_degrees(&g, &[3, 3, 2, 2], &[2, 2, 2, 2, 2]);
        verify_bipartite_structure(&g, 4);
    }

    #[test]
    fn asymmetric_partitions() {
        let g =
            realize_bipartite_degree_sequence(&[5], &[1, 1, 1, 1, 1], RealizeDegseqMethod::Largest)
                .unwrap();
        assert_eq!(g.vcount(), 6);
        assert_eq!(g.ecount(), 5);
        verify_bipartite_degrees(&g, &[5], &[1, 1, 1, 1, 1]);
        verify_bipartite_structure(&g, 1);
    }
}

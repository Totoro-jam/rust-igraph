//! Minimum weight cycle basis (ALGO-CY-004).
//!
//! Computes a minimum weight cycle basis using a modified Horton's
//! algorithm. Candidate cycles are generated via BFS from each vertex
//! of degree ≥ 3, sorted by length, deduplicated, then filtered via
//! Gaussian elimination over GF(2) to select linearly independent
//! cycles of minimum total weight.
//!
//! Edge directions are ignored. Multi-edges and self-loops are
//! supported.
//!
//! Counterpart of `igraph_minimum_cycle_basis`.

use std::collections::VecDeque;

use crate::algorithms::connectivity::components::connected_components;
use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Compute the minimum weight cycle basis of a graph.
///
/// Returns a list of cycles, where each cycle is a `Vec<EdgeId>`
/// listing the edge IDs forming that cycle. The cycle basis has the
/// smallest possible total weight (sum of cycle lengths for unweighted
/// graphs).
///
/// If `bfs_cutoff` is `Some(k)`, only cycles of length at most
/// `2*k + 1` are guaranteed to be minimum-weight. Longer cycles may
/// still be included to complete the basis if `complete` is true.
///
/// If `complete` is true, a complete cycle basis spanning the entire
/// cycle space is returned. If false and `bfs_cutoff` is set, only
/// short cycles are returned (the result may not span the full cycle
/// space).
///
/// Edge directions are ignored (the graph is treated as undirected).
///
/// # Errors
///
/// Returns an error if internal computation fails.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, minimum_cycle_basis};
///
/// // Triangle: one cycle of length 3.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let cycles = minimum_cycle_basis(&g, None, true).unwrap();
/// assert_eq!(cycles.len(), 1);
/// assert_eq!(cycles[0].len(), 3);
///
/// // K4: 3 independent cycles, each of length 3.
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// g.add_edge(2, 3).unwrap();
/// let cycles = minimum_cycle_basis(&g, None, true).unwrap();
/// assert_eq!(cycles.len(), 3);
/// // All minimum cycles in K4 are triangles (length 3)
/// for c in &cycles {
///     assert_eq!(c.len(), 3);
/// }
/// ```
pub fn minimum_cycle_basis(
    graph: &Graph,
    bfs_cutoff: Option<u32>,
    complete: bool,
) -> IgraphResult<Vec<Vec<EdgeId>>> {
    let n = graph.vcount();
    let ecount = graph.ecount();

    if n == 0 || ecount == 0 {
        return Ok(Vec::new());
    }

    let adj = build_incident_all(graph)?;

    let cc = connected_components(graph)?;
    let num_components = cc.count;

    #[allow(clippy::cast_possible_truncation)]
    let rank = (ecount as u32)
        .saturating_sub(n)
        .saturating_add(num_components);

    if rank == 0 {
        return Ok(Vec::new());
    }

    let degrees = compute_degrees(&adj, n);

    let mut visited: Vec<u32> = vec![0; n as usize];
    let mut candidates: Vec<Vec<EdgeId>> = Vec::new();
    let mut mark: u32 = 0;

    for i in 0..n {
        let deg = degrees[i as usize];
        let vis = visited[i as usize] % 3 != 0;

        if deg <= 1 || (vis && deg < 3) {
            continue;
        }

        let cutoff = if vis || !complete { bfs_cutoff } else { None };

        fundamental_cycles_bfs_horton(graph, &adj, &mut candidates, i, cutoff, &mut visited, mark)?;
        mark = mark.checked_add(3).ok_or_else(|| {
            IgraphError::InvalidArgument("mark overflow in minimum_cycle_basis".into())
        })?;
    }

    // Sort candidates by size, then lexicographically
    for c in &mut candidates {
        c.sort_unstable();
    }
    candidates.sort_unstable_by(cycle_cmp);
    candidates.dedup();

    // Gaussian elimination over GF(2) to find independent cycles
    let mut result: Vec<Vec<EdgeId>> = Vec::with_capacity(rank as usize);
    let mut reduced_matrix: Vec<Vec<EdgeId>> = Vec::new();

    for cycle in &candidates {
        let independent = gaussian_elimination_check(&mut reduced_matrix, cycle);
        if independent {
            result.push(cycle.clone());
        }
        if reduced_matrix.len() == rank as usize {
            break;
        }
    }

    Ok(result)
}

fn build_incident_all(graph: &Graph) -> IgraphResult<Vec<Vec<(EdgeId, VertexId)>>> {
    let n = graph.vcount() as usize;
    let mut adj: Vec<Vec<(EdgeId, VertexId)>> = vec![Vec::new(); n];

    for eid in 0..graph.ecount() {
        #[allow(clippy::cast_possible_truncation)]
        let eid32 = eid as EdgeId;
        let (from, to) = graph.edge(eid32)?;
        if from == to {
            adj[from as usize].push((eid32, to));
        } else {
            adj[from as usize].push((eid32, to));
            adj[to as usize].push((eid32, from));
        }
    }

    Ok(adj)
}

fn compute_degrees(adj: &[Vec<(EdgeId, VertexId)>], n: u32) -> Vec<u32> {
    let mut degrees = vec![0u32; n as usize];
    for (i, neighbors) in adj.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        {
            degrees[i] = neighbors.len() as u32;
        }
    }
    degrees
}

fn fundamental_cycles_bfs_horton(
    graph: &Graph,
    adj: &[Vec<(EdgeId, VertexId)>],
    result: &mut Vec<Vec<EdgeId>>,
    start: VertexId,
    bfs_cutoff: Option<u32>,
    visited: &mut [u32],
    mark: u32,
) -> IgraphResult<()> {
    let n = graph.vcount() as usize;
    let mut pred_edge: Vec<i64> = vec![-1; n];
    let mut queue: VecDeque<(VertexId, u32)> = VecDeque::new();

    visited[start as usize] = mark + 1;
    queue.push_back((start, 0));

    while let Some((v, vdist)) = queue.pop_front() {
        for &(eid, u) in &adj[v as usize] {
            if i64::from(eid) == pred_edge[v as usize] {
                continue;
            }

            let u_state = visited[u as usize];

            if u_state == mark + 2 {
                continue;
            }
            if u_state == mark + 1 {
                let cycle = trace_cycle(graph, pred_edge.as_slice(), eid, u, v)?;
                result.push(cycle);
            } else if bfs_cutoff.is_none() || vdist < bfs_cutoff.unwrap_or(0) {
                visited[u as usize] = mark + 1;
                pred_edge[u as usize] = i64::from(eid);
                queue.push_back((u, vdist.saturating_add(1)));
            }
        }

        visited[v as usize] = mark + 2;
    }

    Ok(())
}

fn trace_cycle(
    graph: &Graph,
    pred_edge: &[i64],
    closing_edge: EdgeId,
    u: VertexId,
    v: VertexId,
) -> IgraphResult<Vec<EdgeId>> {
    let mut u_back: Vec<EdgeId> = Vec::new();
    let mut v_back: Vec<EdgeId> = vec![closing_edge];

    let mut up = u;
    let mut vp = v;

    loop {
        if up == vp {
            break;
        }

        let u_pred = pred_edge[up as usize];
        if u_pred < 0 {
            break;
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let u_eid = u_pred as EdgeId;
        u_back.push(u_eid);
        up = graph.edge_other(u_eid, up)?;

        if up == vp {
            break;
        }

        let v_pred = pred_edge[vp as usize];
        if v_pred < 0 {
            break;
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let v_eid = v_pred as EdgeId;
        v_back.push(v_eid);
        vp = graph.edge_other(v_eid, vp)?;
    }

    let mut cycle = v_back;
    cycle.extend(u_back.into_iter().rev());
    Ok(cycle)
}

fn cycle_cmp(a: &Vec<EdgeId>, b: &Vec<EdgeId>) -> std::cmp::Ordering {
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}

/// Symmetric difference of two sorted edge-id vectors (GF(2) addition).
fn cycle_add(a: &[EdgeId], b: &[EdgeId]) -> Vec<EdgeId> {
    let mut result = Vec::with_capacity(a.len() + b.len());
    let mut ia = 0;
    let mut ib = 0;

    while ia < a.len() && ib < b.len() {
        match a[ia].cmp(&b[ib]) {
            std::cmp::Ordering::Less => {
                result.push(a[ia]);
                ia += 1;
            }
            std::cmp::Ordering::Equal => {
                ia += 1;
                ib += 1;
            }
            std::cmp::Ordering::Greater => {
                result.push(b[ib]);
                ib += 1;
            }
        }
    }

    result.extend_from_slice(&a[ia..]);
    result.extend_from_slice(&b[ib..]);
    result
}

/// Check if `cycle` is linearly independent of the reduced matrix.
/// If independent, inserts it into the matrix and returns true.
fn gaussian_elimination_check(reduced_matrix: &mut Vec<Vec<EdgeId>>, cycle: &[EdgeId]) -> bool {
    let mut work: Vec<EdgeId> = cycle.to_vec();

    let mut insert_pos = 0;
    for (i, row) in reduced_matrix.iter().enumerate() {
        if work.is_empty() {
            return false;
        }
        match row[0].cmp(&work[0]) {
            std::cmp::Ordering::Less => {
                insert_pos = i + 1;
            }
            std::cmp::Ordering::Equal => {
                work = cycle_add(row, &work);
                if work.is_empty() {
                    return false;
                }
                insert_pos = i + 1;
            }
            std::cmp::Ordering::Greater => {
                insert_pos = i;
                break;
            }
        }
    }

    if work.is_empty() {
        return false;
    }

    if insert_pos >= reduced_matrix.len() {
        reduced_matrix.push(work);
    } else {
        reduced_matrix.insert(insert_pos, work);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let cycles = minimum_cycle_basis(&g, None, true).unwrap();
        assert!(cycles.is_empty());
    }

    #[test]
    fn tree_no_cycles() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        let cycles = minimum_cycle_basis(&g, None, true).unwrap();
        assert!(cycles.is_empty());
    }

    #[test]
    fn single_triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let cycles = minimum_cycle_basis(&g, None, true).unwrap();
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].len(), 3);
    }

    #[test]
    fn k4_all_triangles() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(0, 2).unwrap(); // 1
        g.add_edge(0, 3).unwrap(); // 2
        g.add_edge(1, 2).unwrap(); // 3
        g.add_edge(1, 3).unwrap(); // 4
        g.add_edge(2, 3).unwrap(); // 5
        let cycles = minimum_cycle_basis(&g, None, true).unwrap();
        // K4: rank = 6 - 4 + 1 = 3, all minimum cycles are triangles
        assert_eq!(cycles.len(), 3);
        for c in &cycles {
            assert_eq!(c.len(), 3);
        }
    }

    #[test]
    fn cycle_5_single_cycle() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        let cycles = minimum_cycle_basis(&g, None, true).unwrap();
        // rank = 5 - 5 + 1 = 1
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].len(), 5);
    }

    #[test]
    fn two_triangles_sharing_edge() {
        // 0-1-2-0 and 1-2-3-1
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 2).unwrap(); // 1
        g.add_edge(2, 0).unwrap(); // 2
        g.add_edge(2, 3).unwrap(); // 3
        g.add_edge(3, 1).unwrap(); // 4
        let cycles = minimum_cycle_basis(&g, None, true).unwrap();
        // rank = 5 - 4 + 1 = 2, both minimum cycles are triangles
        assert_eq!(cycles.len(), 2);
        for c in &cycles {
            assert_eq!(c.len(), 3);
        }
    }

    #[test]
    fn two_components() {
        let mut g = Graph::with_vertices(6);
        // Triangle 0-1-2
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        // Triangle 3-4-5
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 3).unwrap();
        let cycles = minimum_cycle_basis(&g, None, true).unwrap();
        // rank = 6 - 6 + 2 = 2
        assert_eq!(cycles.len(), 2);
    }

    #[test]
    fn cycle_rank_formula() {
        // Connected graph: rank = |E| - |V| + 1
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        g.add_edge(0, 2).unwrap();
        let cycles = minimum_cycle_basis(&g, None, true).unwrap();
        // rank = 6 - 5 + 1 = 2
        assert_eq!(cycles.len(), 2);
    }

    #[test]
    fn minimum_weight_property() {
        // Graph with triangles and a longer cycle
        // 0-1-2-0 (triangle, weight 3)
        // 0-1-3-4-0 (4-cycle, weight 4)
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 2).unwrap(); // 1
        g.add_edge(2, 0).unwrap(); // 2
        g.add_edge(1, 3).unwrap(); // 3
        g.add_edge(3, 4).unwrap(); // 4
        g.add_edge(4, 0).unwrap(); // 5
        let cycles = minimum_cycle_basis(&g, None, true).unwrap();
        // rank = 6 - 5 + 1 = 2
        assert_eq!(cycles.len(), 2);
        // The minimum basis should prefer shorter cycles
        // Both cycles should be length 3 or 4, with total weight minimized
        let total_weight: usize = cycles.iter().map(Vec::len).sum();
        assert!(total_weight <= 8); // triangle(3) + 4-cycle(4) or two 4-cycles
    }

    #[test]
    fn self_loop() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap(); // self-loop
        g.add_edge(0, 1).unwrap();
        let cycles = minimum_cycle_basis(&g, None, true).unwrap();
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].len(), 1);
    }

    #[test]
    fn multi_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        let cycles = minimum_cycle_basis(&g, None, true).unwrap();
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].len(), 2);
    }

    #[test]
    fn directed_treated_as_undirected() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let cycles = minimum_cycle_basis(&g, None, true).unwrap();
        assert_eq!(cycles.len(), 1);
    }

    #[test]
    fn bfs_cutoff_complete() {
        // With cutoff and complete=true, should still get a full basis
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        let cycles = minimum_cycle_basis(&g, Some(1), true).unwrap();
        assert_eq!(cycles.len(), 3);
    }

    #[test]
    fn isolated_vertices() {
        let g = Graph::with_vertices(10);
        let cycles = minimum_cycle_basis(&g, None, true).unwrap();
        assert!(cycles.is_empty());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        let cycles = minimum_cycle_basis(&g, None, true).unwrap();
        assert!(cycles.is_empty());
    }

    #[test]
    fn petersen_graph_rank() {
        // Petersen graph: 10 vertices, 15 edges, rank = 15 - 10 + 1 = 6
        // All minimum cycles are 5-cycles (girth = 5)
        let mut g = Graph::with_vertices(10);
        // Outer cycle
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        // Inner pentagram
        g.add_edge(5, 7).unwrap();
        g.add_edge(7, 9).unwrap();
        g.add_edge(9, 6).unwrap();
        g.add_edge(6, 8).unwrap();
        g.add_edge(8, 5).unwrap();
        // Rungs
        g.add_edge(0, 5).unwrap();
        g.add_edge(1, 6).unwrap();
        g.add_edge(2, 7).unwrap();
        g.add_edge(3, 8).unwrap();
        g.add_edge(4, 9).unwrap();
        let cycles = minimum_cycle_basis(&g, None, true).unwrap();
        assert_eq!(cycles.len(), 6);
        // All minimum cycles in Petersen graph have length 5
        for c in &cycles {
            assert_eq!(c.len(), 5);
        }
    }
}

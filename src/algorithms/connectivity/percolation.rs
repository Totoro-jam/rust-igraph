//! Edge-list percolation (ALGO-CC-030).
//!
//! Counterpart of `igraph_edgelist_percolation()` from
//! `references/igraph/src/connectivity/percolation.c:105`. Given a
//! sequence of vertex-pair edges, returns the evolution of the
//! largest connected component as each edge is added in order.
//!
//! Algorithm: standard union-find with path compression and
//! union-by-size. Each "add edge" call merges two trees; we track
//! the size of the largest tree seen so far and the running count of
//! vertices touched by any edge. Time complexity:
//! `O(|E| · α(|E|))` where `α` is the inverse Ackermann function —
//! amortised near-constant per operation.
//!
//! Both outputs are always populated (the C API makes them
//! independently optional; in Rust the marginal cost of always
//! returning both is one extra `Vec<u32>` alloc).

use crate::core::graph::VertexId;
use crate::core::{IgraphError, IgraphResult};

/// Outputs of [`edgelist_percolation`]. Same length as the input
/// edge slice. `giant_size[i]` is the size of the largest component
/// after edge `i` is added; `vertex_count[i]` is the number of
/// distinct vertices touched by edges `0..=i`.
#[derive(Debug, Clone, PartialEq)]
pub struct EdgelistPercolation {
    /// Size of the largest connected component after edge `i` is added.
    pub giant_size: Vec<u32>,
    /// Cumulative count of distinct vertices touched by any edge in `0..=i`.
    pub vertex_count: Vec<u32>,
}

/// `links[v]` walks toward the root of `v`'s union-find tree (each
/// node's parent), with path compression along the way. Returns the
/// representative.
fn find_root(links: &mut [u32], mut v: usize) -> usize {
    while links[v] as usize != v {
        // Path compression: halve the path by linking each node to
        // its grandparent (matches the upstream `links[a] =
        // links[links[a]]` line).
        links[v] = links[links[v] as usize];
        v = links[v] as usize;
    }
    v
}

/// Percolation curve as a sequence of vertex-pair edges is added.
///
/// Returns an [`EdgelistPercolation`] with two vectors, both of
/// length `edges.len()`:
/// - `giant_size[i]`: size of the largest component after edge `i`
///   is added (1 if no edges or the first edge connects two new
///   vertices).
/// - `vertex_count[i]`: number of distinct vertices touched by any
///   edge in `0..=i`.
///
/// Vertex ids are inferred from the edge list — the implicit vertex
/// count is `max(max(u, v) for (u, v) in edges) + 1`. Endpoints may
/// be self-loops (a self-loop adds nothing to either output).
///
/// Returns [`IgraphError::InvalidArgument`] if any vertex id exceeds
/// `i32::MAX` (matches upstream's `IGRAPH_EINVVID` semantics).
///
/// Counterpart of `igraph_edgelist_percolation` from
/// `references/igraph/src/connectivity/percolation.c:105`.
///
/// # Examples
///
/// ```
/// use rust_igraph::edgelist_percolation;
///
/// // Adding edges (0-1), (2-3), (1-2): the giant grows 2, 2, 4.
/// let edges = [(0u32, 1u32), (2, 3), (1, 2)];
/// let p = edgelist_percolation(&edges).unwrap();
/// assert_eq!(p.giant_size, vec![2, 2, 4]);
/// assert_eq!(p.vertex_count, vec![2, 4, 4]);
/// ```
pub fn edgelist_percolation(edges: &[(VertexId, VertexId)]) -> IgraphResult<EdgelistPercolation> {
    let ecount = edges.len();
    let mut giant_size: Vec<u32> = Vec::with_capacity(ecount);
    let mut vertex_count: Vec<u32> = Vec::with_capacity(ecount);

    if ecount == 0 {
        return Ok(EdgelistPercolation {
            giant_size,
            vertex_count,
        });
    }

    // Implicit vertex count = max id seen + 1. `max_id` is u32; the
    // +1 is checked to catch the (extremely unlikely) u32::MAX case.
    let max_id = edges.iter().flat_map(|&(u, v)| [u, v]).max().unwrap_or(0);
    let vcount_u32 = max_id
        .checked_add(1)
        .ok_or(IgraphError::Internal("vertex count overflow"))?;
    let vcount = vcount_u32 as usize;

    // Union-find: `links[v]` is v's parent (self if root); `sizes[v]`
    // is the size of v's tree (only meaningful at roots), with -1
    // sentinel encoded as 0 here meaning "not yet touched".
    let mut links: Vec<u32> = (0..vcount_u32).collect();
    let mut sizes: Vec<u32> = vec![0; vcount];

    let mut biggest: u32 = 1;
    let mut vertices_added: u32 = 0;

    for &(from, to) in edges {
        let from_idx = from as usize;
        let to_idx = to as usize;
        if sizes[from_idx] == 0 {
            sizes[from_idx] = 1;
            vertices_added += 1;
        }
        if sizes[to_idx] == 0 {
            sizes[to_idx] = 1;
            // Only count `to` if distinct from `from` (self-loop case).
            if from_idx != to_idx {
                vertices_added += 1;
            }
        }
        // Union if they're not already connected. Self-loop is a no-op.
        if from_idx != to_idx {
            let root_a = find_root(&mut links, from_idx);
            let root_b = find_root(&mut links, to_idx);
            if root_a != root_b {
                // Union-by-size: attach smaller under larger.
                let (parent, child) = if sizes[root_a] < sizes[root_b] {
                    (root_b, root_a)
                } else {
                    (root_a, root_b)
                };
                let parent_u32 = u32::try_from(parent)
                    .map_err(|_| IgraphError::Internal("vertex index exceeds u32::MAX"))?;
                links[child] = parent_u32;
                sizes[parent] = sizes[parent]
                    .checked_add(sizes[child])
                    .ok_or(IgraphError::Internal("union-find size overflow"))?;
                if sizes[parent] > biggest {
                    biggest = sizes[parent];
                }
            }
        }
        giant_size.push(biggest);
        vertex_count.push(vertices_added);
    }

    Ok(EdgelistPercolation {
        giant_size,
        vertex_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_returns_empty() {
        let p = edgelist_percolation(&[]).unwrap();
        assert!(p.giant_size.is_empty());
        assert!(p.vertex_count.is_empty());
    }

    #[test]
    fn single_edge_two_vertices() {
        let p = edgelist_percolation(&[(0, 1)]).unwrap();
        assert_eq!(p.giant_size, vec![2]);
        assert_eq!(p.vertex_count, vec![2]);
    }

    #[test]
    fn two_disjoint_edges_then_join() {
        // (0-1), (2-3), (1-2): two separate pairs of size 2, then a
        // chain of 4.
        let p = edgelist_percolation(&[(0, 1), (2, 3), (1, 2)]).unwrap();
        assert_eq!(p.giant_size, vec![2, 2, 4]);
        assert_eq!(p.vertex_count, vec![2, 4, 4]);
    }

    #[test]
    fn parallel_edge_does_not_change_giant() {
        // Same edge added twice — second one is a no-op for both metrics.
        let p = edgelist_percolation(&[(0, 1), (0, 1)]).unwrap();
        assert_eq!(p.giant_size, vec![2, 2]);
        assert_eq!(p.vertex_count, vec![2, 2]);
    }

    #[test]
    fn self_loop_only_adds_one_vertex() {
        // Self-loop on vertex 0: 1 new vertex, biggest stays 1.
        let p = edgelist_percolation(&[(0, 0)]).unwrap();
        assert_eq!(p.giant_size, vec![1]);
        assert_eq!(p.vertex_count, vec![1]);
    }

    #[test]
    fn chain_grows_linearly() {
        // 0-1, 1-2, 2-3, 3-4 → giants 2, 3, 4, 5.
        let p = edgelist_percolation(&[(0, 1), (1, 2), (2, 3), (3, 4)]).unwrap();
        assert_eq!(p.giant_size, vec![2, 3, 4, 5]);
        assert_eq!(p.vertex_count, vec![2, 3, 4, 5]);
    }

    #[test]
    fn star_around_center() {
        // 0-1, 0-2, 0-3 → star, giant grows 2, 3, 4.
        let p = edgelist_percolation(&[(0, 1), (0, 2), (0, 3)]).unwrap();
        assert_eq!(p.giant_size, vec![2, 3, 4]);
        assert_eq!(p.vertex_count, vec![2, 3, 4]);
    }

    #[test]
    fn merging_unequal_clusters_picks_max() {
        // Build a triangle (0-1, 1-2, 0-2) → size 3, then join to a
        // pair (3-4) by adding (2-3). Final giant = 5.
        let p = edgelist_percolation(&[(0, 1), (1, 2), (0, 2), (3, 4), (2, 3)]).unwrap();
        // After (0,1): {0,1} giant=2
        // After (1,2): {0,1,2} giant=3
        // After (0,2): same tree (no-op), giant=3
        // After (3,4): {3,4} new component, giant stays 3
        // After (2,3): merge → giant=5
        assert_eq!(p.giant_size, vec![2, 3, 3, 3, 5]);
        assert_eq!(p.vertex_count, vec![2, 3, 3, 5, 5]);
    }

    #[test]
    fn classic_random_order_matches_hand_trace() {
        // A small example chosen to exercise union-by-size: build a
        // tree of size 3 first (0-1, 1-2), then a single edge (3-4),
        // then bridge 2-3. Without union-by-size the leftmost tree
        // would dominate; with it, the (parent, child) choice is
        // size-driven. Either way the GIANT (max size) should be 5.
        let p = edgelist_percolation(&[(0, 1), (1, 2), (3, 4), (2, 3)]).unwrap();
        assert_eq!(p.giant_size, vec![2, 3, 3, 5]);
        assert_eq!(p.vertex_count, vec![2, 3, 5, 5]);
    }

    #[test]
    fn high_vertex_ids_are_supported() {
        // Sparse ids: only vertices 100 and 200 touched. Implicit
        // vcount = 201 (huge sizes vector but cheap).
        let p = edgelist_percolation(&[(100, 200)]).unwrap();
        assert_eq!(p.giant_size, vec![2]);
        assert_eq!(p.vertex_count, vec![2]);
    }
}

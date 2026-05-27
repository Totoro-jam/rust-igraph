//! Sort vertex IDs by degree (ALGO-PR-054).
//!
//! Counterpart of `igraph_sort_vertex_ids_by_degree()` from
//! `references/igraph/src/properties/degrees.c:712`.
//!
//! Returns vertex IDs sorted by their degree in ascending or
//! descending order.

use crate::algorithms::properties::degree::DegreeMode;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Sort order for vertex degree sorting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    /// Smallest degree first.
    Ascending,
    /// Largest degree first.
    Descending,
}

/// Return vertex IDs sorted by their degree.
///
/// - `mode`: which degree to use (`Out`, `In`, or `All`). For
///   undirected graphs all modes are equivalent.
/// - `order`: ascending (smallest degree first) or descending.
///
/// Returns a `Vec<VertexId>` containing all vertex IDs in the
/// requested order. Ties are broken by vertex ID (stable sort).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, sort_vertices_by_degree, DegreeMode, SortOrder};
///
/// // Star: center vertex has degree 3, leaves have degree 1.
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// let sorted = sort_vertices_by_degree(&g, DegreeMode::All, SortOrder::Descending).unwrap();
/// assert_eq!(sorted[0], 0); // highest degree vertex first
/// ```
pub fn sort_vertices_by_degree(
    graph: &Graph,
    mode: DegreeMode,
    order: SortOrder,
) -> IgraphResult<Vec<VertexId>> {
    let n = graph.vcount();

    if n == 0 {
        return Ok(Vec::new());
    }

    let n_usize = n as usize;
    let ecount = graph.ecount();
    let directed = graph.is_directed();

    let mut deg = vec![0u32; n_usize];

    if ecount > 0 {
        let m_u32 =
            u32::try_from(ecount).map_err(|_| IgraphError::Internal("ecount exceeds u32::MAX"))?;

        for eid in 0..m_u32 {
            let (from, to) = graph.edge(eid)?;
            let f = from as usize;
            let t = to as usize;

            if directed {
                match mode {
                    DegreeMode::Out => {
                        deg[f] += 1;
                    }
                    DegreeMode::In => {
                        deg[t] += 1;
                    }
                    DegreeMode::All => {
                        deg[f] += 1;
                        deg[t] += 1;
                    }
                }
            } else if from == to {
                deg[f] += 2;
            } else {
                deg[f] += 1;
                deg[t] += 1;
            }
        }
    }

    let mut ids: Vec<VertexId> = (0..n).collect();

    match order {
        SortOrder::Ascending => {
            ids.sort_by(|&a, &b| deg[a as usize].cmp(&deg[b as usize]));
        }
        SortOrder::Descending => {
            ids.sort_by(|&a, &b| deg[b as usize].cmp(&deg[a as usize]));
        }
    }

    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let s = sort_vertices_by_degree(&g, DegreeMode::All, SortOrder::Ascending).unwrap();
        assert!(s.is_empty());
    }

    #[test]
    fn no_edges_ascending() {
        let g = Graph::with_vertices(5);
        let s = sort_vertices_by_degree(&g, DegreeMode::All, SortOrder::Ascending).unwrap();
        // All degree 0, stable sort → original order.
        assert_eq!(s, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn star_descending() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        let s = sort_vertices_by_degree(&g, DegreeMode::All, SortOrder::Descending).unwrap();
        // deg = [3, 1, 1, 1]. Descending: 0 first, then 1,2,3.
        assert_eq!(s[0], 0);
        assert_eq!(s.len(), 4);
    }

    #[test]
    fn star_ascending() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        let s = sort_vertices_by_degree(&g, DegreeMode::All, SortOrder::Ascending).unwrap();
        // deg = [3, 1, 1, 1]. Ascending: 1,2,3 first (degree 1), then 0 (degree 3).
        assert_eq!(s[3], 0);
    }

    #[test]
    fn path_5_ascending() {
        let mut g = Graph::with_vertices(5);
        for i in 0..4u32 {
            g.add_edge(i, i + 1).unwrap();
        }
        let s = sort_vertices_by_degree(&g, DegreeMode::All, SortOrder::Ascending).unwrap();
        // deg = [1, 2, 2, 2, 1]. Sorted ascending: 0,4 (deg 1), then 1,2,3 (deg 2).
        assert!(s[0] == 0 || s[0] == 4);
        assert!(s[1] == 0 || s[1] == 4);
        assert!(s[0] != s[1]);
    }

    #[test]
    fn directed_out_degree() {
        // 0→1, 0→2, 1→2
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let s = sort_vertices_by_degree(&g, DegreeMode::Out, SortOrder::Descending).unwrap();
        // out-deg = [2, 1, 0]. Desc: 0, 1, 2.
        assert_eq!(s, vec![0, 1, 2]);
    }

    #[test]
    fn directed_in_degree() {
        // 0→1, 0→2, 1→2
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let s = sort_vertices_by_degree(&g, DegreeMode::In, SortOrder::Descending).unwrap();
        // in-deg = [0, 1, 2]. Desc: 2, 1, 0.
        assert_eq!(s, vec![2, 1, 0]);
    }

    #[test]
    fn k4_stable_order() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let s = sort_vertices_by_degree(&g, DegreeMode::All, SortOrder::Ascending).unwrap();
        // All same degree → stable sort preserves original order.
        assert_eq!(s, vec![0, 1, 2, 3]);
    }

    #[test]
    fn self_loop_counted() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap(); // self-loop: deg(0) += 2
        g.add_edge(0, 1).unwrap(); // deg(0) += 1, deg(1) += 1
        g.add_edge(1, 2).unwrap(); // deg(1) += 1, deg(2) += 1
        let s = sort_vertices_by_degree(&g, DegreeMode::All, SortOrder::Descending).unwrap();
        // deg = [3, 2, 1]. Desc: 0, 1, 2.
        assert_eq!(s, vec![0, 1, 2]);
    }
}

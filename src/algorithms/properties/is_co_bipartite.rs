//! Co-bipartite graph predicate (ALGO-PR-110).
//!
//! A graph is co-bipartite if its complement is bipartite. Equivalently,
//! the vertex set can be partitioned into two cliques (each part is a
//! complete subgraph).
//!
//! For directed graphs, the function returns `false`.

use crate::core::{Graph, IgraphResult};

/// Check whether a graph is co-bipartite.
///
/// A co-bipartite graph's vertex set can be partitioned into two
/// cliques. This is equivalent to the complement being bipartite.
///
/// Returns `false` for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_co_bipartite};
///
/// // `K_3`: single clique → co-bipartite (other part empty)
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert!(is_co_bipartite(&g).unwrap());
///
/// // `C_5` is NOT co-bipartite (complement `C_5` is not bipartite)
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(4, 0).unwrap();
/// assert!(!is_co_bipartite(&g).unwrap());
/// ```
pub fn is_co_bipartite(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n <= 2 {
        return Ok(true);
    }

    let n_usize = n as usize;
    let mut adj = vec![vec![false; n_usize]; n_usize];
    for v in 0..n {
        let nbrs = graph.neighbors(v)?;
        for &w in &nbrs {
            adj[v as usize][w as usize] = true;
        }
    }

    // 2-color the complement graph. If the complement is bipartite,
    // the original is co-bipartite. Use BFS on complement edges.
    let mut color: Vec<i8> = vec![-1; n_usize];

    for start in 0..n_usize {
        if color[start] != -1 {
            continue;
        }
        color[start] = 0;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(start);

        while let Some(u) = queue.pop_front() {
            let u_color = color[u];
            let next_color = 1 - u_color;
            for v in 0..n_usize {
                if v == u || adj[u][v] {
                    continue;
                }
                // u-v is a complement edge
                if color[v] == -1 {
                    color[v] = next_color;
                    queue.push_back(v);
                } else if color[v] == u_color {
                    return Ok(false);
                }
            }
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_co_bipartite(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_co_bipartite(&g).unwrap());
    }

    #[test]
    fn two_vertices_no_edge() {
        // Complement is `K_2` which is bipartite
        let g = Graph::with_vertices(2);
        assert!(is_co_bipartite(&g).unwrap());
    }

    #[test]
    fn two_vertices_with_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_co_bipartite(&g).unwrap());
    }

    #[test]
    fn triangle() {
        // `K_3`: single clique, other part empty
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_co_bipartite(&g).unwrap());
    }

    #[test]
    fn two_cliques() {
        // `K_2` union `K_3`: partition {0,1} and {2,3,4}
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(2, 4).unwrap();
        assert!(is_co_bipartite(&g).unwrap());
    }

    #[test]
    fn complete_graph() {
        // `K_5`: single clique → co-bipartite
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_co_bipartite(&g).unwrap());
    }

    #[test]
    fn c4_co_bipartite() {
        // `C_4` complement: {0,1,2,3} with edges 0-2, 1-3 → two disjoint
        // edges → bipartite. So `C_4` IS co-bipartite.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_co_bipartite(&g).unwrap());
    }

    #[test]
    fn c5_not_co_bipartite() {
        // `C_5` complement is also `C_5` (self-complementary), which is
        // odd cycle → not bipartite → not co-bipartite
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(!is_co_bipartite(&g).unwrap());
    }

    #[test]
    fn path_p3_co_bipartite() {
        // `P_3`: 0-1-2. Complement has edge 0-2 only → bipartite.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(is_co_bipartite(&g).unwrap());
    }

    #[test]
    fn independent_set_3() {
        // 3 isolated vertices. Complement is `K_3` → bipartite? No,
        // `K_3` is a triangle → not bipartite (odd cycle).
        // So 3 isolated vertices is NOT co-bipartite.
        let g = Graph::with_vertices(3);
        assert!(!is_co_bipartite(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_co_bipartite(&g).unwrap());
    }

    #[test]
    fn star_not_co_bipartite() {
        // `S_4`: center 0, leaves 1,2,3,4. Complement: 0 isolated, 1-2-3-4
        // complete (`K_4`). Is `K_4` bipartite? No (triangle) → not co-bipartite.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        assert!(!is_co_bipartite(&g).unwrap());
    }

    #[test]
    fn complete_bipartite_k22() {
        // `K_{2,2}` = `C_4` → co-bipartite (see c4 test)
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(is_co_bipartite(&g).unwrap());
    }
}

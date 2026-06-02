//! Dart-free graph predicate (ALGO-PR-107).
//!
//! A graph is dart-free if it contains no induced dart. The dart is a
//! diamond (`K_4` minus one edge) plus one pendant edge from a
//! degree-2 vertex of the diamond: 5 vertices, 6 edges.
//!
//! Concretely: vertices {a, b, c, d, e} where {a, b, c, d} form a
//! diamond with edges a-b, a-c, a-d, b-c, b-d (missing c-d), and
//! pendant edge c-e (or d-e). Vertex e is adjacent only to c.
//!
//! For directed graphs, the function returns `false`.

use crate::core::{Graph, IgraphResult};

/// Check whether a graph is dart-free (no induced dart).
///
/// The dart is a diamond plus a pendant from a degree-2 vertex of the
/// diamond. A dart-free graph has no induced dart.
///
/// Returns `false` for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_dart_free};
///
/// // Diamond is dart-free (only 4 vertices)
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// assert!(is_dart_free(&g).unwrap());
///
/// // Dart: diamond {0,1,2,3} (missing 2-3) + pendant 2-4
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// g.add_edge(2, 4).unwrap();
/// assert!(!is_dart_free(&g).unwrap());
/// ```
pub fn is_dart_free(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n < 5 {
        return Ok(true);
    }

    let n_usize = n as usize;
    let mut adj = vec![vec![false; n_usize]; n_usize];
    let mut nbrs_list: Vec<Vec<u32>> = Vec::with_capacity(n_usize);

    for v in 0..n {
        let nbrs = graph.neighbors(v)?;
        for &w in &nbrs {
            adj[v as usize][w as usize] = true;
        }
        nbrs_list.push(nbrs);
    }

    // Diamond {a, b, c, d}: a-b, a-c, a-d, b-c, b-d edges, c-d missing.
    // a and b are the degree-3 vertices (the "spine"), c and d are
    // degree-2 (the "wings").
    //
    // Dart: pendant e adjacent to c (or d) but not to a, b, or d (or c).
    //
    // Strategy: for each edge (a,b) that share ≥ 2 common neighbors,
    // find pairs (c,d) of common neighbors where c-d is NOT an edge
    // (forming a diamond). Then check if c or d has a pendant neighbor
    // outside {a,b,c,d} not adjacent to the other 3.
    for a in 0..n {
        let ai = a as usize;
        for &b in &nbrs_list[ai] {
            if b <= a {
                continue;
            }
            let bi = b as usize;
            // Common neighbors of a and b
            let common: Vec<usize> = nbrs_list[ai]
                .iter()
                .filter(|&&w| w != b && adj[bi][w as usize])
                .map(|&w| w as usize)
                .collect();

            if common.len() < 2 {
                continue;
            }

            for (i, &ci) in common.iter().enumerate() {
                for &di in &common[(i + 1)..] {
                    if adj[ci][di] {
                        continue;
                    }
                    // Diamond: {a, b} spine, {c, d} wings (c-d missing)
                    // Check c for pendant
                    if has_dart_pendant(&adj, &nbrs_list, ci, ai, bi, di) {
                        return Ok(false);
                    }
                    // Check d for pendant
                    if has_dart_pendant(&adj, &nbrs_list, di, ai, bi, ci) {
                        return Ok(false);
                    }
                }
            }
        }
    }

    Ok(true)
}

/// Check if wing vertex has a neighbor outside the diamond
/// that is not adjacent to the other three diamond vertices.
fn has_dart_pendant(
    adj: &[Vec<bool>],
    nbrs_list: &[Vec<u32>],
    wing: usize,
    s1: usize,
    s2: usize,
    other_wing: usize,
) -> bool {
    for &e_u32 in &nbrs_list[wing] {
        let e = e_u32 as usize;
        if e == s1 || e == s2 || e == other_wing {
            continue;
        }
        if !adj[s1][e] && !adj[s2][e] && !adj[other_wing][e] {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_dart_free(&g).unwrap());
    }

    #[test]
    fn small_graphs() {
        let g = Graph::with_vertices(4);
        assert!(is_dart_free(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_dart_free(&g).unwrap());
    }

    #[test]
    fn diamond_dart_free() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(is_dart_free(&g).unwrap());
    }

    #[test]
    fn dart() {
        // Diamond {0,1,2,3}: edges 0-1, 0-2, 0-3, 1-2, 1-3 (missing 2-3)
        // Pendant: 2-4
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 4).unwrap();
        assert!(!is_dart_free(&g).unwrap());
    }

    #[test]
    fn dart_pendant_from_other_wing() {
        // Diamond {0,1,2,3}, pendant from d=3 instead of c=2
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(!is_dart_free(&g).unwrap());
    }

    #[test]
    fn k5_dart_free() {
        // `K_5`: no missing edges in any 4-vertex subgraph → no diamond → no dart
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_dart_free(&g).unwrap());
    }

    #[test]
    fn pendant_from_spine_not_dart() {
        // Diamond {0,1,2,3} with pendant from spine vertex 0 (degree-3 vertex)
        // Induced subgraph: {0,1,2,3,4} has edges 0-1,0-2,0-3,1-2,1-3,0-4
        // Vertex 4 connects to 0 (spine) → this is a diamond + pendant from
        // spine, which is NOT a dart (dart needs pendant from wing).
        // Actually, the induced subgraph on {1,0,2,3,4}: 1-0, 0-2, 0-3, 1-2,
        // 1-3, 0-4 → diamond {1,0,2,3} missing 2-3, pendant from 0 to 4.
        // 0 is a spine vertex (deg 3 in diamond). Not a wing vertex.
        // But wait: {2,3} are common neighbors of {0,1}. Missing edge 2-3.
        // Wings are 2 and 3. Pendant 4 is from vertex 0 (spine), not wing.
        // So it IS NOT a dart... but we need to check: does 2 or 3 have a
        // pendant? Only if 2 or 3 has a neighbor outside {0,1,2,3} that
        // is not adjacent to 0, 1, or the other wing. Vertex 4 is not a
        // neighbor of 2 or 3, so no dart.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        assert!(is_dart_free(&g).unwrap());
    }

    #[test]
    fn pendant_connected_to_other_wing_not_dart() {
        // Diamond {0,1,2,3} + vertex 4 adjacent to both wings 2 and 3
        // → not a pendant (adjacent to two diamond vertices)
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 4).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_dart_free(&g).unwrap());
    }

    #[test]
    fn path_dart_free() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_dart_free(&g).unwrap());
    }

    #[test]
    fn star_dart_free() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(0, 5).unwrap();
        assert!(is_dart_free(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_dart_free(&g).unwrap());
    }

    #[test]
    fn c4_dart_free() {
        // No diamond in `C_4` → dart-free
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_dart_free(&g).unwrap());
    }

    #[test]
    fn c5_dart_free() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(is_dart_free(&g).unwrap());
    }
}

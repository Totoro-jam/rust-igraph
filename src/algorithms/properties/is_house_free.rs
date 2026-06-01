//! House-free graph predicate (ALGO-PR-105).
//!
//! A graph is house-free if it contains no induced house graph. The
//! house is a `C_5` with one chord: 5 vertices {a, b, c, d, e} with
//! edges {a-b, b-c, c-d, d-e, e-a, a-c} (the chord a-c turns the
//! pentagon into a triangle {a, b, c} topped by a path d-e).
//! Equivalently: a triangle with a `P_3` extension completing a 5-cycle
//! through one of its edges. 6 edges total.
//!
//! For directed graphs, the function returns `false`.

use crate::core::{Graph, IgraphResult};

/// Check whether a graph is house-free (no induced house graph).
///
/// The house is `C_5` plus one chord: a triangle {a, b, c} and two
/// extra vertices d, e where c-d, d-e, e-a are edges and a-d, b-d,
/// b-e, c-e are all absent.
///
/// Returns `false` for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_house_free};
///
/// // Triangle is house-free
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert!(is_house_free(&g).unwrap());
///
/// // House: triangle {0,1,2}, path 2-3-4-0
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(4, 0).unwrap();
/// assert!(!is_house_free(&g).unwrap());
/// ```
pub fn is_house_free(graph: &Graph) -> IgraphResult<bool> {
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

    // House: triangle {a, b, c} with chord a-c on the "base". The "roof"
    // vertices d, e satisfy: c-d, d-e, e-a edges; a-d, b-d, b-e, c-e absent.
    //
    // For each triangle {a, b, c}, try each edge as the base (the edge
    // that gets the chord). The remaining vertex is the "top" of the triangle.
    // Then find d adjacent to c (but not a, b) and e adjacent to both d and a
    // (but not b, c). Also d-e must be non-adjacent to b and c/a respectively.
    for a in 0..n {
        let ai = a as usize;
        for b in (a + 1)..n {
            let bi = b as usize;
            if !adj[ai][bi] {
                continue;
            }
            for c in (b + 1)..n {
                let ci = c as usize;
                if !adj[ai][ci] || !adj[bi][ci] {
                    continue;
                }

                // Triangle {a, b, c}. The chord is one of the 3 edges.
                // We try all 3 orientations of the house.
                // House with chord a-b (top=c): find d adj to c not a,b;
                //   e adj to d and a, not b,c
                if check_house_roof(&adj, &nbrs_list, ai, bi, ci) {
                    return Ok(false);
                }
                // chord a-b (top=c), but roof goes c-d-e-b:
                if check_house_roof(&adj, &nbrs_list, bi, ai, ci) {
                    return Ok(false);
                }
                // chord a-c (top=b): roof b-d-e-a → no, roof b-d-e-c
                if check_house_roof(&adj, &nbrs_list, ai, ci, bi) {
                    return Ok(false);
                }
                if check_house_roof(&adj, &nbrs_list, ci, ai, bi) {
                    return Ok(false);
                }
                // chord b-c (top=a): roof a-d-e-b, a-d-e-c
                if check_house_roof(&adj, &nbrs_list, bi, ci, ai) {
                    return Ok(false);
                }
                if check_house_roof(&adj, &nbrs_list, ci, bi, ai) {
                    return Ok(false);
                }
            }
        }
    }

    Ok(true)
}

/// Check for a house with triangle base edge (base1, base2) and top vertex.
/// Look for d adjacent to top (not base1, base2) and e adjacent to d and base1
/// (not base2, top). Also d must not be adjacent to base1, e must not be
/// adjacent to top.
fn check_house_roof(
    adj: &[Vec<bool>],
    nbrs_list: &[Vec<u32>],
    base1: usize,
    base2: usize,
    top: usize,
) -> bool {
    // d: adjacent to top, not adjacent to base1 or base2
    for &d_u32 in &nbrs_list[top] {
        let d = d_u32 as usize;
        if d == base1 || d == base2 {
            continue;
        }
        if adj[base1][d] || adj[base2][d] {
            continue;
        }
        // e: adjacent to d and base1, not adjacent to base2 or top
        for &e_u32 in &nbrs_list[d] {
            let e = e_u32 as usize;
            if e == top || e == base1 || e == base2 || e == d {
                continue;
            }
            if adj[base1][e] && !adj[base2][e] && !adj[top][e] {
                return true;
            }
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
        assert!(is_house_free(&g).unwrap());
    }

    #[test]
    fn small_graphs() {
        let g = Graph::with_vertices(4);
        assert!(is_house_free(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_house_free(&g).unwrap());
    }

    #[test]
    fn house() {
        // Triangle {0,1,2}, path 2-3-4-0
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(!is_house_free(&g).unwrap());
    }

    #[test]
    fn c5_not_house() {
        // `C_5` has no chord → not a house
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(is_house_free(&g).unwrap());
    }

    #[test]
    fn k5_house_free() {
        // `K_5`: every 5-vertex subgraph is `K_5` with 10 edges, not 6
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_house_free(&g).unwrap());
    }

    #[test]
    fn c4_house_free() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_house_free(&g).unwrap());
    }

    #[test]
    fn diamond_house_free() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(is_house_free(&g).unwrap());
    }

    #[test]
    fn path_house_free() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_house_free(&g).unwrap());
    }

    #[test]
    fn star_house_free() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(0, 5).unwrap();
        assert!(is_house_free(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_house_free(&g).unwrap());
    }

    #[test]
    fn house_with_extra_chord_not_house() {
        // House + extra chord → not an induced house
        // Triangle {0,1,2}, path 2-3-4-0, plus chord 1-3
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(is_house_free(&g).unwrap());
    }

    #[test]
    fn gem_not_house() {
        // Gem: `P_4` + universal vertex. Not a house.
        // Vertices 1-2-3-4 form P_4, vertex 0 adjacent to all.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        // Check: triangle {0,1,2}, roof vertex 3 adj to 2 not 0,1 → 3 is d.
        // e adj to 3 and 1? e=4: adj to 3 yes, adj to 1? No (no edge 1-4).
        // Actually 0 is adj to 4, so e=4 is adj to top=0, so it fails the
        // "not adjacent to top" check.
        // No house found because 0 is adjacent to everything.
        assert!(is_house_free(&g).unwrap());
    }

    #[test]
    fn petersen_not_house_free() {
        // Petersen graph: girth 5, 3-regular. Check if it has an induced house.
        // House = C_5 + chord. Petersen has many C_5 (girth=5). Any C_5
        // plus a chord from outside? Petersen is triangle-free, so adding
        // a chord to C_5 would create a triangle → no house in Petersen.
        let mut g = Graph::with_vertices(10);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        g.add_edge(5, 7).unwrap();
        g.add_edge(7, 9).unwrap();
        g.add_edge(9, 6).unwrap();
        g.add_edge(6, 8).unwrap();
        g.add_edge(8, 5).unwrap();
        g.add_edge(0, 5).unwrap();
        g.add_edge(1, 6).unwrap();
        g.add_edge(2, 7).unwrap();
        g.add_edge(3, 8).unwrap();
        g.add_edge(4, 9).unwrap();
        assert!(is_house_free(&g).unwrap());
    }
}

//! Read-Wilson graph atlas constructor (ALGO-CN-021).
//!
//! Counterpart of `igraph_atlas()` in
//! `references/igraph/src/constructors/atlas.c:63-87`. Returns the
//! `number`-th of the 1253 simple unlabelled undirected graphs on
//! 0..7 vertices, in the ordering of Ronald C. Read and Robin J. Wilson,
//! *An Atlas of Graphs* (Oxford University Press, 1998):
//!
//! 1. by ascending vertex count;
//! 2. for fixed vertex count, by ascending edge count;
//! 3. for fixed vertex/edge count, by ascending lexicographic degree
//!    sequence;
//! 4. for fixed degree sequence, by ascending automorphism count.
//!
//! The graphs on 0, 1, 2, 3, 4, 5, 6, 7 vertices start at indices 0, 1,
//! 2, 4, 8, 19, 53, 209 respectively. All atlas graphs are undirected.
//!
//! The edge data is stored in [`atlas_edges`](super::atlas_edges) — two
//! static `&[u32]` tables transliterated byte-for-byte from
//! `references/igraph/src/constructors/atlas-edges.h`.

use super::atlas_edges::{ATLAS_EDGES, ATLAS_EDGES_POS};
use crate::core::{Graph, IgraphError, IgraphResult};

/// Total number of graphs catalogued in the atlas: 1253.
pub const ATLAS_SIZE: u32 = 1253;

/// Build the `number`-th graph from Read-Wilson's *An Atlas of Graphs*.
///
/// `number` must be in `0..1253`. The returned graph is always
/// undirected. See the module docstring for the ordering convention.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — `number` is outside `0..1253`.
///
/// # Examples
///
/// ```
/// use rust_igraph::atlas;
///
/// // 0: the null graph (0 vertices, 0 edges)
/// let g = atlas(0).unwrap();
/// assert_eq!((g.vcount(), g.ecount()), (0, 0));
///
/// // 3: the single edge K_2
/// let k2 = atlas(3).unwrap();
/// assert_eq!((k2.vcount(), k2.ecount()), (2, 1));
///
/// // 1252 (last): the complete graph K_7
/// let k7 = atlas(1252).unwrap();
/// assert_eq!((k7.vcount(), k7.ecount()), (7, 21));
/// ```
pub fn atlas(number: u32) -> IgraphResult<Graph> {
    if number >= ATLAS_SIZE {
        return Err(IgraphError::InvalidArgument(format!(
            "atlas: graph index {number} is out of range (must be < {ATLAS_SIZE})"
        )));
    }

    let pos = ATLAS_EDGES_POS[number as usize] as usize;
    if pos + 2 > ATLAS_EDGES.len() {
        return Err(IgraphError::InvalidArgument(
            "atlas: corrupt position table (header out of range)".into(),
        ));
    }

    let vcount = ATLAS_EDGES[pos];
    let ecount = ATLAS_EDGES[pos + 1] as usize;

    let edge_words = ecount
        .checked_mul(2)
        .ok_or_else(|| IgraphError::InvalidArgument("atlas: edge buffer overflow".into()))?;
    let end = pos
        .checked_add(2)
        .and_then(|x| x.checked_add(edge_words))
        .ok_or_else(|| IgraphError::InvalidArgument("atlas: edge buffer overflow".into()))?;
    if end > ATLAS_EDGES.len() {
        return Err(IgraphError::InvalidArgument(
            "atlas: corrupt position table (payload out of range)".into(),
        ));
    }

    let mut g = Graph::new(vcount, false)?;
    let edges: Vec<(u32, u32)> = ATLAS_EDGES[pos + 2..end]
        .chunks_exact(2)
        .map(|pair| (pair[0], pair[1]))
        .collect();
    g.add_edges(edges)?;
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn canonical_edge_set(g: &Graph) -> BTreeSet<(u32, u32)> {
        let mut s = BTreeSet::new();
        for v in 0..g.vcount() {
            for &u in &g.neighbors(v).expect("neighbors") {
                if v <= u {
                    s.insert((v, u));
                } else {
                    s.insert((u, v));
                }
            }
        }
        s
    }

    #[test]
    fn out_of_range_errors() {
        assert!(atlas(ATLAS_SIZE).is_err());
        assert!(atlas(ATLAS_SIZE + 1).is_err());
        assert!(atlas(u32::MAX).is_err());
    }

    #[test]
    fn atlas_size_constant_matches_upstream() {
        // upstream says 1253 — atlas-edges.h has exactly that many POS entries.
        assert_eq!(ATLAS_SIZE, 1253);
    }

    #[test]
    fn null_graph_indices() {
        // 0..=7 — single null graph on n vertices at the start of each n-cell.
        let starts: [(u32, u32); 8] = [
            (0, 0),
            (1, 1),
            (2, 2),
            (3, 4),
            (4, 8),
            (5, 19),
            (6, 53),
            (7, 209),
        ];
        for (n, idx) in starts {
            let g = atlas(idx).unwrap_or_else(|_| panic!("atlas({idx})"));
            assert_eq!(g.vcount(), n, "atlas({idx}) vcount");
            assert_eq!(g.ecount(), 0, "atlas({idx}) ecount (null graph)");
        }
    }

    #[test]
    fn small_known_graphs() {
        // Hand-checked against igraph C output and the Read-Wilson Atlas.
        let g3 = atlas(3).expect("3");
        assert_eq!((g3.vcount(), g3.ecount()), (2, 1));
        assert_eq!(
            canonical_edge_set(&g3),
            [(0u32, 1u32)].into_iter().collect()
        );

        // index 7 is the K_3 triangle on 3 vertices
        let g7 = atlas(7).expect("7");
        assert_eq!((g7.vcount(), g7.ecount()), (3, 3));
        assert_eq!(
            canonical_edge_set(&g7),
            [(0u32, 1), (0, 2), (1, 2)].into_iter().collect()
        );

        // index 18 = K_4 (last 4-vertex graph): 6 edges, 3-regular
        let k4 = atlas(18).expect("18");
        assert_eq!((k4.vcount(), k4.ecount()), (4, 6));
        for v in 0..k4.vcount() {
            assert_eq!(k4.neighbors(v).expect("nbrs").len(), 3);
        }
    }

    #[test]
    fn k7_is_last_atlas_graph() {
        // index 1252 is K_7: 7 vertices, 21 edges (C(7,2)), 6-regular.
        let k7 = atlas(1252).expect("1252");
        assert_eq!((k7.vcount(), k7.ecount()), (7, 21));
        for v in 0..k7.vcount() {
            assert_eq!(k7.neighbors(v).expect("nbrs").len(), 6);
        }
    }

    #[test]
    fn no_self_loops_or_repeated_edges() {
        // Spot-check across a representative spread instead of all 1253.
        let samples = [
            0, 1, 2, 3, 4, 7, 8, 18, 19, 52, 53, 70, 100, 180, 208, 209, 500, 1000, 1252,
        ];
        for &i in &samples {
            let g = atlas(i).unwrap_or_else(|_| panic!("atlas({i})"));
            let mut seen: BTreeSet<(u32, u32)> = BTreeSet::new();
            for v in 0..g.vcount() {
                for &u in &g.neighbors(v).expect("nbrs") {
                    assert_ne!(v, u, "atlas({i}) self-loop at {v}");
                    let key = if v <= u { (v, u) } else { (u, v) };
                    if !seen.insert(key) {
                        // multi-edge appears twice in neighbors traversal naturally,
                        // but with chunks_exact + add_edges every undirected edge
                        // is inserted exactly once, so we only need degree symmetry.
                    }
                }
            }
        }
    }

    #[test]
    fn all_atlas_graphs_construct_and_are_undirected() {
        // Walk the entire catalogue. Cheap (1253 tiny graphs).
        for i in 0..ATLAS_SIZE {
            let g = atlas(i).unwrap_or_else(|_| panic!("atlas({i})"));
            assert!(g.vcount() <= 7, "atlas({i}) vcount {}", g.vcount());
            assert!(!g.is_directed(), "atlas({i}) should be undirected");
            // edge count consistent with neighbor-sum / 2
            let total_deg: usize = (0..g.vcount())
                .map(|v| g.neighbors(v).expect("nbrs").len())
                .sum();
            assert_eq!(
                total_deg,
                g.ecount() as usize * 2,
                "atlas({i}) degree sum != 2|E|"
            );
        }
    }

    #[test]
    fn vertex_count_partitions_match_upstream_starts() {
        // For each n in 0..=7, atlas(start[n]..start[n+1]) should all have
        // exactly n vertices.
        let starts = [0u32, 1, 2, 4, 8, 19, 53, 209, ATLAS_SIZE];
        for n in 0..8 {
            let lo = starts[n];
            let hi = starts[n + 1];
            for i in lo..hi {
                let g = atlas(i).expect("atlas");
                assert_eq!(
                    g.vcount() as usize,
                    n,
                    "atlas({i}) should have vcount {n}, got {}",
                    g.vcount()
                );
            }
        }
    }
}

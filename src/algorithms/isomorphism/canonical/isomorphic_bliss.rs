//! `isomorphic_bliss` (`ALGO-ISO-006`).

use super::canonicalize;
use crate::algorithms::isomorphism::vf2::Vf2Isomorphism;
use crate::core::{Graph, IgraphError, IgraphResult};

/// Test whether two graphs are isomorphic via canonical labeling (BLISS).
///
/// Both graphs are reduced to their canonical form by the shared
/// individualization-refinement engine (ALGO-ISO-003); they are isomorphic
/// iff the canonical certificates (and, when colours are supplied, the colour
/// sequence in canonical order) coincide. When they match, a concrete vertex
/// mapping is recovered by composing the two canonical labelings.
///
/// Optional `vertex_color1` / `vertex_color2` slices restrict the matching:
/// two vertices may correspond only if their colours are equal. Pass `None`
/// for uncoloured graphs; supplying a colour for only one side makes that
/// colour be ignored (matching upstream). Unlike VF2, self-loops are
/// supported (BLISS folds a loop into the vertex invariant); multi-edges are
/// rejected.
///
/// On success [`Vf2Isomorphism::iso`] tells whether a mapping exists; when it
/// does, `map12` / `map21` hold the permutation taking `graph1` to `graph2`
/// and its inverse, otherwise they are empty.
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] if the two graphs differ in
/// directedness or if a supplied colour vector has the wrong length, and
/// [`IgraphError::Unsupported`] if either graph has multiple edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, isomorphic_bliss};
///
/// // Two triangles with relabelled vertices are isomorphic.
/// let mut a = Graph::new(3, false).unwrap();
/// a.add_edge(0, 1).unwrap();
/// a.add_edge(1, 2).unwrap();
/// a.add_edge(2, 0).unwrap();
/// let mut b = Graph::new(3, false).unwrap();
/// b.add_edge(0, 2).unwrap();
/// b.add_edge(2, 1).unwrap();
/// b.add_edge(1, 0).unwrap();
/// let r = isomorphic_bliss(&a, &b, None, None).unwrap();
/// assert!(r.iso);
/// assert_eq!(r.map12.len(), 3);
/// ```
pub fn isomorphic_bliss(
    graph1: &Graph,
    graph2: &Graph,
    vertex_color1: Option<&[u32]>,
    vertex_color2: Option<&[u32]>,
) -> IgraphResult<Vf2Isomorphism> {
    if graph1.is_directed() != graph2.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "cannot compare directed and undirected graphs".to_owned(),
        ));
    }

    // Supplying a colour for only one side ignores both (matches upstream).
    let (color1, color2) = if vertex_color1.is_some() == vertex_color2.is_some() {
        (vertex_color1, vertex_color2)
    } else {
        (None, None)
    };

    let n1 = graph1.vcount() as usize;
    let n2 = graph2.vcount() as usize;

    // Different vertex counts: trivially non-isomorphic. (canonicalize still
    // validates colour-vector lengths so callers get a consistent error.)
    let canon1 = canonicalize(graph1, color1)?;
    let canon2 = canonicalize(graph2, color2)?;

    let not_iso = || Vf2Isomorphism {
        iso: false,
        map12: Vec::new(),
        map21: Vec::new(),
    };

    if n1 != n2 || canon1.certificate != canon2.certificate {
        return Ok(not_iso());
    }

    // Canonical orders (position -> vertex), inverting `labeling` (vertex ->
    // position).
    let mut order1 = vec![0u32; n1];
    for (v, &pos) in canon1.labeling.iter().enumerate() {
        order1[pos as usize] = v as u32;
    }
    let mut order2 = vec![0u32; n2];
    for (v, &pos) in canon2.labeling.iter().enumerate() {
        order2[pos as usize] = v as u32;
    }

    // Colour-aware check: vertices sharing a canonical position must share a
    // raw colour value, otherwise the map is not colour-preserving.
    if let (Some(c1), Some(c2)) = (color1, color2) {
        for pos in 0..n1 {
            if c1[order1[pos] as usize] != c2[order2[pos] as usize] {
                return Ok(not_iso());
            }
        }
    }

    // map12[v1] = the graph2 vertex sharing v1's canonical position.
    let mut map12 = vec![0u32; n1];
    let mut map21 = vec![0u32; n2];
    for (v1, &pos) in canon1.labeling.iter().enumerate() {
        let v2 = order2[pos as usize];
        map12[v1] = v2;
        map21[v2 as usize] = v1 as u32;
    }

    Ok(Vf2Isomorphism {
        iso: true,
        map12,
        map21,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle(perm: [u32; 3]) -> Graph {
        let mut g = Graph::new(3, false).expect("graph");
        g.add_edge(perm[0], perm[1]).expect("edge");
        g.add_edge(perm[1], perm[2]).expect("edge");
        g.add_edge(perm[2], perm[0]).expect("edge");
        g
    }

    fn path(n: u32) -> Graph {
        let mut g = Graph::new(n, false).expect("graph");
        for i in 0..n.saturating_sub(1) {
            g.add_edge(i, i + 1).expect("edge");
        }
        g
    }

    fn complete(n: u32) -> Graph {
        let mut g = Graph::new(n, false).expect("graph");
        for a in 0..n {
            for b in (a + 1)..n {
                g.add_edge(a, b).expect("edge");
            }
        }
        g
    }

    fn directed_cycle(n: u32) -> Graph {
        let mut g = Graph::new(n, true).expect("graph");
        for i in 0..n {
            g.add_edge(i, (i + 1) % n).expect("edge");
        }
        g
    }

    /// Assert `map12` is a genuine edge-preserving bijection from `a` to `b`.
    fn assert_valid_map(a: &Graph, b: &Graph, r: &Vf2Isomorphism) {
        let n = a.vcount() as usize;
        assert_eq!(r.map12.len(), n);
        assert_eq!(r.map21.len(), b.vcount() as usize);
        // Bijection: map12 is a permutation and map21 is its inverse.
        let mut seen = vec![false; n];
        for (v, &iv) in r.map12.iter().enumerate() {
            assert!(!seen[iv as usize], "map12 not injective");
            seen[iv as usize] = true;
            assert_eq!(r.map21[iv as usize], v as u32, "map21 not inverse of map12");
        }
        // Edge preservation.
        let b_edges: std::collections::HashSet<(u32, u32)> = (0..b.ecount())
            .map(|e| b.edge(e as u32).expect("edge"))
            .map(|(u, v)| {
                if b.is_directed() || u <= v {
                    (u, v)
                } else {
                    (v, u)
                }
            })
            .collect();
        for e in 0..a.ecount() {
            let (u, v) = a.edge(e as u32).expect("edge");
            let (iu, iv) = (r.map12[u as usize], r.map12[v as usize]);
            let key = if a.is_directed() || iu <= iv {
                (iu, iv)
            } else {
                (iv, iu)
            };
            assert!(b_edges.contains(&key), "edge not preserved by map");
        }
    }

    #[test]
    fn relabelled_triangles_are_isomorphic() {
        let a = triangle([0, 1, 2]);
        let b = triangle([2, 0, 1]);
        let r = isomorphic_bliss(&a, &b, None, None).expect("bliss");
        assert!(r.iso);
        assert_valid_map(&a, &b, &r);
    }

    #[test]
    fn triangle_is_not_a_path() {
        let r = isomorphic_bliss(&triangle([0, 1, 2]), &path(3), None, None).expect("bliss");
        assert!(!r.iso);
        assert!(r.map12.is_empty());
        assert!(r.map21.is_empty());
    }

    #[test]
    fn different_vertex_counts_are_not_isomorphic() {
        let r = isomorphic_bliss(&complete(4), &complete(5), None, None).expect("bliss");
        assert!(!r.iso);
    }

    #[test]
    fn complete_graphs_match_under_any_relabelling() {
        let a = complete(5);
        let b = complete(5);
        let r = isomorphic_bliss(&a, &b, None, None).expect("bliss");
        assert!(r.iso);
        assert_valid_map(&a, &b, &r);
    }

    #[test]
    fn directed_cycles_respect_orientation() {
        let a = directed_cycle(4);
        let b = directed_cycle(4);
        let r = isomorphic_bliss(&a, &b, None, None).expect("bliss");
        assert!(r.iso);
        assert_valid_map(&a, &b, &r);
    }

    #[test]
    fn directedness_mismatch_is_an_error() {
        let und = path(3);
        let dir = directed_cycle(3);
        assert!(isomorphic_bliss(&und, &dir, None, None).is_err());
    }

    #[test]
    fn colours_must_match_for_isomorphism() {
        // Same structure (P_3), but the centre vertex carries a different
        // colour, so no colour-preserving map exists.
        let a = path(3);
        let b = path(3);
        // a: ends colour 0, centre colour 1.
        let ca = [0u32, 1, 0];
        // b: ends colour 1, centre colour 0 -> colour classes differ.
        let cb = [1u32, 0, 1];
        let r = isomorphic_bliss(&a, &b, Some(&ca), Some(&cb)).expect("bliss");
        assert!(!r.iso);
    }

    #[test]
    fn colour_matching_uses_raw_values_not_class_structure() {
        // Same structure (P_3) and identical colour-class *sizes* {2, 1}, but
        // disjoint raw colour values: no colour-preserving map exists because
        // BLISS matches colours by value, not by class shape (verified against
        // python-igraph 0.11.9).
        let a = path(3);
        let b = path(3);
        let ca = [5u32, 6, 5];
        let cb = [7u32, 8, 7];
        let r = isomorphic_bliss(&a, &b, Some(&ca), Some(&cb)).expect("bliss");
        assert!(!r.iso);
    }

    #[test]
    fn matching_colours_yield_isomorphism() {
        let a = path(3);
        let b = path(3);
        let ca = [0u32, 1, 0];
        let cb = [0u32, 1, 0];
        let r = isomorphic_bliss(&a, &b, Some(&ca), Some(&cb)).expect("bliss");
        assert!(r.iso);
        assert_valid_map(&a, &b, &r);
    }

    #[test]
    fn one_sided_colour_is_ignored() {
        // Supplying a colour for only one side ignores both (upstream parity).
        let a = triangle([0, 1, 2]);
        let b = triangle([1, 2, 0]);
        let ca = [0u32, 1, 2];
        let r = isomorphic_bliss(&a, &b, Some(&ca), None).expect("bliss");
        assert!(r.iso);
    }

    #[test]
    fn wrong_colour_length_is_an_error() {
        let a = triangle([0, 1, 2]);
        let b = triangle([0, 1, 2]);
        let bad = [0u32, 0];
        assert!(isomorphic_bliss(&a, &b, Some(&bad), Some(&[0, 0, 0])).is_err());
    }

    #[test]
    fn rejects_multigraph() {
        let mut a = Graph::new(2, false).expect("graph");
        a.add_edge(0, 1).expect("edge");
        a.add_edge(0, 1).expect("edge");
        let b = path(2);
        assert!(isomorphic_bliss(&a, &b, None, None).is_err());
    }
}

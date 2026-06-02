//! Triad census (ALGO-MO-002).
//!
//! Classifies all vertex triples in a directed graph into the 16 isomorphism
//! classes defined by Davis and Leinhardt (1972).
//! Counterpart of `igraph_triad_census`.

use crate::core::{Graph, IgraphError, IgraphResult};

/// The 16 triad types in Davis-Leinhardt MAN notation.
///
/// Each variant represents one of the 16 isomorphism classes of directed triads.
/// The name encodes: number of Mutual, Asymmetric, and Null dyads, plus a
/// letter suffix for orientation (D=down, U=up, C=chain, T=transitive).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TriadType {
    /// 003: A, B, C (empty graph on 3 vertices)
    T003 = 0,
    /// 012: A->B, C
    T012 = 1,
    /// 102: A<->B, C
    T102 = 2,
    /// 021D: A<-B->C
    T021D = 3,
    /// 021U: A->B<-C
    T021U = 4,
    /// 021C: A->B->C
    T021C = 5,
    /// 111D: A<->B<-C
    T111D = 6,
    /// 111U: A<->B->C
    T111U = 7,
    /// 030T: A->B<-C, A->C
    T030T = 8,
    /// 030C: A<-B<-C, A->C
    T030C = 9,
    /// 201: A<->B<->C
    T201 = 10,
    /// 120D: A<-B->C, A<->C
    T120D = 11,
    /// 120U: A->B<-C, A<->C
    T120U = 12,
    /// 120C: A->B->C, A<->C
    T120C = 13,
    /// 210: A->B<->C, A<->C
    T210 = 14,
    /// 300: A<->B<->C, A<->C (complete graph)
    T300 = 15,
}

/// Result of a triad census.
#[derive(Debug, Clone, PartialEq)]
pub struct TriadCensus {
    /// Counts for each of the 16 triad types, indexed by `TriadType` ordinal.
    pub counts: [f64; 16],
}

impl TriadCensus {
    /// Get the count for a specific triad type.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::{Graph, triad_census, TriadType};
    ///
    /// let mut g = Graph::new(3, true).unwrap();
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(1, 2).unwrap();
    /// g.add_edge(2, 0).unwrap();
    /// let tc = triad_census(&g).unwrap();
    /// assert_eq!(tc.get(TriadType::T030C), 1.0);
    /// ```
    pub fn get(&self, triad_type: TriadType) -> f64 {
        self.counts[triad_type as usize]
    }
}

/// Performs a triad census on a directed graph.
///
/// Classifies all `n*(n-1)*(n-2)/6` vertex triples into the 16
/// Davis-Leinhardt triad types. Returns counts as `f64` to avoid overflow
/// for large graphs.
///
/// For undirected graphs, edges are treated as mutual connections.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, triad_census, TriadType};
///
/// // Complete directed graph on 3 vertices: all triads are type 300
/// let mut g = Graph::new(3, true).unwrap();
/// for i in 0..3u32 {
///     for j in 0..3u32 {
///         if i != j { g.add_edge(i, j).unwrap(); }
///     }
/// }
/// let tc = triad_census(&g).unwrap();
/// assert!((tc.get(TriadType::T300) - 1.0).abs() < 1e-10);
///
/// // Directed 3-cycle: 0->1->2->0 — all asymmetric, one 030C triad
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let tc = triad_census(&g).unwrap();
/// assert!((tc.get(TriadType::T030C) - 1.0).abs() < 1e-10);
/// ```
pub fn triad_census(graph: &Graph) -> IgraphResult<TriadCensus> {
    let n = graph.vcount();

    if n < 3 {
        return Ok(TriadCensus { counts: [0.0; 16] });
    }

    let adj = build_dyad_matrix(graph)?;
    let mut counts = [0.0_f64; 16];

    for i in 0..n {
        for j in (i + 1)..n {
            for k in (j + 1)..n {
                let ab = adj[(i as usize) * (n as usize) + (j as usize)];
                let ac = adj[(i as usize) * (n as usize) + (k as usize)];
                let bc = adj[(j as usize) * (n as usize) + (k as usize)];
                let idx = lookup_triad_type(ab, ac, bc);
                counts[idx] += 1.0;
            }
        }
    }

    Ok(TriadCensus { counts })
}

/// Dyad code encoding: for each ordered pair (u, v):
/// - 0 = no edge
/// - 1 = u->v only
/// - 2 = v->u only
/// - 3 = mutual (both directions)
fn build_dyad_matrix(graph: &Graph) -> IgraphResult<Vec<u8>> {
    let n = graph.vcount();
    let size = (n as usize)
        .checked_mul(n as usize)
        .ok_or_else(|| IgraphError::InvalidArgument("graph too large for triad census".into()))?;
    let mut matrix = vec![0u8; size];
    let nn = n as usize;
    let ecount = graph.ecount();

    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (src, tgt) = graph.edge(eid as u32)?;
        if src == tgt {
            continue;
        }
        let idx_st = (src as usize) * nn + (tgt as usize);
        let idx_ts = (tgt as usize) * nn + (src as usize);
        matrix[idx_st] |= 1;
        matrix[idx_ts] |= 2;
    }

    if !graph.is_directed() {
        // Undirected: every edge is mutual
        for cell in &mut matrix {
            if *cell != 0 {
                *cell = 3;
            }
        }
    }

    Ok(matrix)
}

/// Lookup table mapping three dyad codes to the triad type index (0..15).
///
/// Each dyad code is 0-3. We encode the triple as a single index
/// into a precomputed table. The table is symmetric under vertex permutation:
/// we canonicalize the triple by sorting the MAN counts.
fn lookup_triad_type(ab: u8, ac: u8, bc: u8) -> usize {
    // Count mutual (M), asymmetric (A), null (N) dyads
    let mut m = 0u8;
    let mut a = 0u8;
    let mut n_count = 0u8;

    for &d in &[ab, ac, bc] {
        match d {
            0 => n_count += 1,
            3 => m += 1,
            _ => a += 1,
        }
    }

    match (m, a, n_count) {
        (0, 1, 2) => 1, // 012
        (1, 0, 2) => 2, // 102
        (0, 2, 1) => classify_021(ab, ac, bc),
        (1, 1, 1) => classify_111(ab, ac, bc),
        (0, 3, 0) => classify_030(ab, ac, bc),
        (2, 0, 1) => 10, // 201
        (1, 2, 0) => classify_120(ab, ac, bc),
        (2, 1, 0) => 14, // 210
        (3, 0, 0) => 15, // 300
        _ => 0,          // 003 and any impossible combos
    }
}

/// 021 subtypes: two asymmetric dyads, one null.
/// 021D (3): out-star from center
/// 021U (4): in-star to center
/// 021C (5): directed chain
fn classify_021(ab: u8, ac: u8, bc: u8) -> usize {
    // Find the null dyad to identify the two non-adjacent vertices.
    // The shared vertex (center) connects to both others asymmetrically.
    let (from_center_1, from_center_2) = if bc == 0 {
        // Center = A (vertex i), connects to B and C
        (ab, ac)
    } else if ac == 0 {
        // Center = B (vertex j), connects to A and C
        (flip_dyad(ab), bc)
    } else {
        // ab == 0: Center = C (vertex k), connects to A and B
        (flip_dyad(ac), flip_dyad(bc))
    };

    // from_center: 1 = center->other, 2 = other->center
    match (from_center_1, from_center_2) {
        (1, 1) => 3, // 021D: center->both (out-star)
        (2, 2) => 4, // 021U: both->center (in-star)
        _ => 5,      // 021C: chain
    }
}

/// 111 subtypes: one mutual, one asymmetric, one null.
/// 111D (6): third->mutual_vertex
/// 111U (7): mutual_vertex->third
fn classify_111(ab: u8, ac: u8, bc: u8) -> usize {
    // The asymmetric dyad connects the mutual pair to the third vertex.
    // Direction of the asymmetric edge from the mutual-pair-vertex determines type.
    let asym_from_mutual_vertex = if ab == 3 {
        // Mutual: A-B. Third = C. Asymmetric connects mutual pair to C.
        // ac: from A to C (1=A->C, 2=C->A). bc: from B to C (1=B->C, 2=C->B).
        if ac != 0 { ac } else { bc }
    } else if ac == 3 {
        // Mutual: A-C. Third = B. Asymmetric connects mutual pair to B.
        // ab: from A to B (1=A->B, 2=B->A). bc: from B to C (need C's view to B = flip).
        if ab != 0 { ab } else { flip_dyad(bc) }
    } else {
        // Mutual: B-C. Third = A. Asymmetric connects mutual pair to A.
        // ab: from A to B (need B's view to A = flip). ac: from A to C (need C's view to A = flip).
        if ab != 0 {
            flip_dyad(ab)
        } else {
            flip_dyad(ac)
        }
    };

    // 1 = mutual_vertex->third (111U), 2 = third->mutual_vertex (111D)
    if asym_from_mutual_vertex == 1 {
        7 // 111U
    } else {
        6 // 111D
    }
}

/// 030 subtypes: three asymmetric dyads.
/// 030T (8): transitive (one vertex has out-degree 2)
/// 030C (9): cyclic (each vertex has out-degree 1)
fn classify_030(ab: u8, ac: u8, bc: u8) -> usize {
    // Count out-degree for each vertex
    let mut out_a = 0u8;
    let mut out_b = 0u8;
    let mut out_c = 0u8;

    if ab == 1 {
        out_a += 1;
    } else {
        out_b += 1;
    }
    if ac == 1 {
        out_a += 1;
    } else {
        out_c += 1;
    }
    if bc == 1 {
        out_b += 1;
    } else {
        out_c += 1;
    }

    if out_a == 2 || out_b == 2 || out_c == 2 {
        8 // 030T: transitive
    } else {
        9 // 030C: cyclic
    }
}

/// 120 subtypes: one mutual, two asymmetric, zero null.
/// 120D (11): both mutual-pair vertices point OUT to third
/// 120U (12): third points IN to both mutual-pair vertices
/// 120C (13): chain through third
fn classify_120(ab: u8, ac: u8, bc: u8) -> usize {
    // Find the mutual dyad. The other two dyads are asymmetric, connecting
    // the mutual-pair vertices to the third vertex.
    let (to_third_1, to_third_2) = if ab == 3 {
        // Mutual: A-B. Third = C. Asymmetric: AC and BC.
        (ac, bc)
    } else if ac == 3 {
        // Mutual: A-C. Third = B. Asymmetric: AB and CB=flip(BC).
        (ab, flip_dyad(bc))
    } else {
        // Mutual: B-C. Third = A. Asymmetric: BA=flip(AB) and CA=flip(AC).
        (flip_dyad(ab), flip_dyad(ac))
    };

    // to_third: 1 = mutual_vertex->third, 2 = third->mutual_vertex
    match (to_third_1, to_third_2) {
        (2, 2) => 11, // 120D: third sends to both mutual vertices
        (1, 1) => 12, // 120U: both mutual vertices send to third
        _ => 13,      // 120C: chain
    }
}

/// Flip a dyad code (swap perspective between the two vertices).
fn flip_dyad(d: u8) -> u8 {
    match d {
        1 => 2,
        2 => 1,
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let g = Graph::new(0, true).unwrap();
        let tc = triad_census(&g).unwrap();
        assert!(tc.counts.iter().all(|&c| c.abs() < 1e-10));
    }

    #[test]
    fn test_two_vertices() {
        let g = Graph::new(2, true).unwrap();
        let tc = triad_census(&g).unwrap();
        assert!(tc.counts.iter().all(|&c| c.abs() < 1e-10));
    }

    #[test]
    fn test_three_vertices_no_edges() {
        let g = Graph::new(3, true).unwrap();
        let tc = triad_census(&g).unwrap();
        assert!((tc.get(TriadType::T003) - 1.0).abs() < 1e-10);
        assert!(tc.counts[1..].iter().all(|&c| c.abs() < 1e-10));
    }

    #[test]
    fn test_single_edge() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        let tc = triad_census(&g).unwrap();
        assert!((tc.get(TriadType::T012) - 1.0).abs() < 1e-10);
        assert!((tc.get(TriadType::T003)).abs() < 1e-10);
    }

    #[test]
    fn test_mutual_edge() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        let tc = triad_census(&g).unwrap();
        assert!((tc.get(TriadType::T102) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_directed_3_cycle() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let tc = triad_census(&g).unwrap();
        assert!((tc.get(TriadType::T030C) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_transitive_triple() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let tc = triad_census(&g).unwrap();
        assert!((tc.get(TriadType::T030T) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_complete_directed() {
        let mut g = Graph::new(3, true).unwrap();
        for i in 0..3u32 {
            for j in 0..3u32 {
                if i != j {
                    g.add_edge(i, j).unwrap();
                }
            }
        }
        let tc = triad_census(&g).unwrap();
        assert!((tc.get(TriadType::T300) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_021d_out_star() {
        // B->A, B->C (out-star from B=1): 021D
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(1, 2).unwrap();
        let tc = triad_census(&g).unwrap();
        assert!((tc.get(TriadType::T021D) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_021u_in_star() {
        // A->B, C->B (in-star to B=1): 021U
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 1).unwrap();
        let tc = triad_census(&g).unwrap();
        assert!((tc.get(TriadType::T021U) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_021c_chain() {
        // A->B->C (chain): 021C
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let tc = triad_census(&g).unwrap();
        assert!((tc.get(TriadType::T021C) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_four_vertices_sum() {
        // 4 vertices: C(4,3) = 4 total triples
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let tc = triad_census(&g).unwrap();
        let total: f64 = tc.counts.iter().sum();
        assert!((total - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_201_two_mutual() {
        // 0<->1, 0<->2, no edge between 1 and 2: 201
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let tc = triad_census(&g).unwrap();
        assert!((tc.get(TriadType::T201) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_undirected_triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        let tc = triad_census(&g).unwrap();
        assert!((tc.get(TriadType::T300) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_undirected_path() {
        // Undirected path 0-1-2: mutual(0,1), mutual(1,2), null(0,2) -> 201
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let tc = triad_census(&g).unwrap();
        assert!((tc.get(TriadType::T201) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_counts_sum_to_total() {
        // n=5, total triples = C(5,3) = 10
        let mut g = Graph::new(5, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        let tc = triad_census(&g).unwrap();
        let total: f64 = tc.counts.iter().sum();
        assert!((total - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_111d() {
        // 111D: A<->B<-C. Mutual (A,B), asymmetric C->B, null (A,C)
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(2, 1).unwrap();
        let tc = triad_census(&g).unwrap();
        assert!((tc.get(TriadType::T111D) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_111u() {
        // 111U: A<->B->C. Mutual (A,B), asymmetric B->C, null (A,C)
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(1, 2).unwrap();
        let tc = triad_census(&g).unwrap();
        assert!((tc.get(TriadType::T111U) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_210() {
        // 210: 2 mutual + 1 asymmetric + 0 null
        // A<->B, A<->C, A->... wait: mutual(B,C), mutual(A,C), asymmetric(A,B) = A->B
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let tc = triad_census(&g).unwrap();
        assert!((tc.get(TriadType::T210) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_self_loops_ignored() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let tc = triad_census(&g).unwrap();
        assert!((tc.get(TriadType::T012) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_120d() {
        // 120D: A<-B->C, A<->C
        // Mutual: A-C. Asymmetric: B->A, B->C. So B points out to both.
        // Vertices: 0=A, 1=B, 2=C
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(1, 2).unwrap();
        let tc = triad_census(&g).unwrap();
        assert!((tc.get(TriadType::T120D) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_120u() {
        // 120U: A->B<-C, A<->C
        // Mutual: A-C. Asymmetric: A->B, C->B. So both point to B.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 1).unwrap();
        let tc = triad_census(&g).unwrap();
        assert!((tc.get(TriadType::T120U) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_120c() {
        // 120C: A->B->C, A<->C
        // Mutual: A-C. Asymmetric: A->B, B->C. Chain through B.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let tc = triad_census(&g).unwrap();
        assert!((tc.get(TriadType::T120C) - 1.0).abs() < 1e-10);
    }
}

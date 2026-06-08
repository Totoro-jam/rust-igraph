//! Weisfeiler-Lehman graph hashing (ALGO-ISO-007).
//!
//! Computes a structural fingerprint for vertices and for the entire graph
//! using the 1-WL (color refinement) algorithm. Two graphs that are NOT
//! isomorphic are guaranteed to produce different hashes (with high
//! probability). Isomorphic graphs MAY hash to the same value, but
//! non-isomorphic graphs that 1-WL cannot distinguish will also collide.
//!
//! This is the standard technique for graph-level classification kernels
//! and GNN expressiveness bounds.

use crate::core::{Graph, IgraphResult};

/// Result of Weisfeiler-Lehman hashing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WlHashResult {
    /// Per-vertex hash after the final iteration.
    pub vertex_hashes: Vec<u64>,
    /// Graph-level hash (aggregation of sorted vertex hashes).
    pub graph_hash: u64,
    /// Number of iterations actually performed (may be less than
    /// `max_iterations` if colors stabilized early).
    pub iterations: u32,
}

/// Compute the Weisfeiler-Lehman hash of a graph.
///
/// Uses 1-WL (color refinement): each vertex starts with an initial color
/// derived from its label (or degree if no labels provided), then
/// iteratively updates its color by hashing the sorted multiset of
/// neighbor colors together with its own color.
///
/// # Parameters
///
/// - `graph` — The input graph.
/// - `labels` — Optional initial vertex labels. If `None`, vertex degrees
///   are used as initial colors.
/// - `max_iterations` — Maximum number of WL iterations. The algorithm
///   stops early if the coloring stabilizes (no new colors created).
///
/// # Returns
///
/// A [`WlHashResult`] containing per-vertex hashes, the graph-level hash,
/// and the number of iterations performed.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, wl_hash};
///
/// // Two triangles — same WL hash
/// let g1 = Graph::from_edges(&[(0,1),(1,2),(2,0)], false, Some(3)).unwrap();
/// let g2 = Graph::from_edges(&[(0,1),(1,2),(2,0)], false, Some(3)).unwrap();
/// let h1 = wl_hash(&g1, None, 5).unwrap();
/// let h2 = wl_hash(&g2, None, 5).unwrap();
/// assert_eq!(h1.graph_hash, h2.graph_hash);
///
/// // Triangle vs path — different WL hash
/// let g3 = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// let h3 = wl_hash(&g3, None, 5).unwrap();
/// assert_ne!(h1.graph_hash, h3.graph_hash);
/// ```
pub fn wl_hash(
    graph: &Graph,
    labels: Option<&[u32]>,
    max_iterations: u32,
) -> IgraphResult<WlHashResult> {
    let n = graph.vcount() as usize;

    if n == 0 {
        return Ok(WlHashResult {
            vertex_hashes: Vec::new(),
            graph_hash: 0,
            iterations: 0,
        });
    }

    if let Some(l) = labels {
        if l.len() != n {
            return Err(crate::core::IgraphError::InvalidArgument(format!(
                "labels length {} != vcount {}",
                l.len(),
                n
            )));
        }
    }

    let mut colors: Vec<u64> = if let Some(l) = labels {
        l.iter().map(|&x| u64::from(x)).collect()
    } else {
        let mut c = Vec::with_capacity(n);
        for v in 0..graph.vcount() {
            c.push(graph.degree(v)? as u64);
        }
        c
    };

    let mut iterations = 0u32;
    let mut prev_num_colors = count_distinct(&colors);

    for _ in 0..max_iterations {
        let mut new_colors: Vec<u64> = Vec::with_capacity(n);

        for v in 0..graph.vcount() {
            let neighbors = graph.neighbors(v)?;
            let mut neighbor_hashes: Vec<u64> =
                neighbors.iter().map(|&u| colors[u as usize]).collect();
            neighbor_hashes.sort_unstable();

            let new_hash = hash_multiset(colors[v as usize], &neighbor_hashes);
            new_colors.push(new_hash);
        }

        colors = new_colors;
        iterations += 1;

        let num_colors = count_distinct(&colors);
        if num_colors == prev_num_colors {
            break;
        }
        prev_num_colors = num_colors;
    }

    let graph_hash = compute_graph_hash(&colors);

    Ok(WlHashResult {
        vertex_hashes: colors,
        graph_hash,
        iterations,
    })
}

/// Compute WL hashes at each iteration level (subtree patterns of depth 0..k).
///
/// Returns a vector of [`WlHashResult`] for iterations 0 through
/// `max_iterations` (or until stabilization). This is useful for
/// WL subtree kernel computation where you need hashes at every depth.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, wl_hash_iterations};
///
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3),(3,0)], false, Some(4)).unwrap();
/// let iters = wl_hash_iterations(&g, None, 3).unwrap();
/// assert!(!iters.is_empty());
/// assert!(iters.len() <= 4); // initial + up to 3 iterations
/// ```
pub fn wl_hash_iterations(
    graph: &Graph,
    labels: Option<&[u32]>,
    max_iterations: u32,
) -> IgraphResult<Vec<WlHashResult>> {
    let n = graph.vcount() as usize;

    if n == 0 {
        return Ok(vec![WlHashResult {
            vertex_hashes: Vec::new(),
            graph_hash: 0,
            iterations: 0,
        }]);
    }

    if let Some(l) = labels {
        if l.len() != n {
            return Err(crate::core::IgraphError::InvalidArgument(format!(
                "labels length {} != vcount {}",
                l.len(),
                n
            )));
        }
    }

    let mut colors: Vec<u64> = if let Some(l) = labels {
        l.iter().map(|&x| u64::from(x)).collect()
    } else {
        let mut c = Vec::with_capacity(n);
        for v in 0..graph.vcount() {
            c.push(graph.degree(v)? as u64);
        }
        c
    };

    let mut results: Vec<WlHashResult> = Vec::with_capacity((max_iterations + 1) as usize);

    results.push(WlHashResult {
        vertex_hashes: colors.clone(),
        graph_hash: compute_graph_hash(&colors),
        iterations: 0,
    });

    let mut prev_num_colors = count_distinct(&colors);

    for iter in 0..max_iterations {
        let mut new_colors: Vec<u64> = Vec::with_capacity(n);

        for v in 0..graph.vcount() {
            let neighbors = graph.neighbors(v)?;
            let mut neighbor_hashes: Vec<u64> =
                neighbors.iter().map(|&u| colors[u as usize]).collect();
            neighbor_hashes.sort_unstable();

            let new_hash = hash_multiset(colors[v as usize], &neighbor_hashes);
            new_colors.push(new_hash);
        }

        colors = new_colors;

        results.push(WlHashResult {
            vertex_hashes: colors.clone(),
            graph_hash: compute_graph_hash(&colors),
            iterations: iter + 1,
        });

        let num_colors = count_distinct(&colors);
        if num_colors == prev_num_colors {
            break;
        }
        prev_num_colors = num_colors;
    }

    Ok(results)
}

/// Check if two graphs have equal WL hash (necessary but not sufficient
/// for isomorphism).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, wl_isomorphic};
///
/// let g1 = Graph::from_edges(&[(0,1),(1,2),(2,0)], false, Some(3)).unwrap();
/// let g2 = Graph::from_edges(&[(0,1),(1,2),(2,0)], false, Some(3)).unwrap();
/// assert!(wl_isomorphic(&g1, &g2, None, None, 5).unwrap());
///
/// let g3 = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert!(!wl_isomorphic(&g1, &g3, None, None, 5).unwrap());
/// ```
pub fn wl_isomorphic(
    g1: &Graph,
    g2: &Graph,
    labels1: Option<&[u32]>,
    labels2: Option<&[u32]>,
    max_iterations: u32,
) -> IgraphResult<bool> {
    if g1.vcount() != g2.vcount() || g1.ecount() != g2.ecount() {
        return Ok(false);
    }

    let h1 = wl_hash(g1, labels1, max_iterations)?;
    let h2 = wl_hash(g2, labels2, max_iterations)?;

    if h1.graph_hash != h2.graph_hash {
        return Ok(false);
    }

    let mut s1 = h1.vertex_hashes;
    let mut s2 = h2.vertex_hashes;
    s1.sort_unstable();
    s2.sort_unstable();
    Ok(s1 == s2)
}

// --- Internal helpers ---

fn count_distinct(colors: &[u64]) -> usize {
    let mut sorted = colors.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    sorted.len()
}

fn hash_multiset(self_color: u64, neighbor_colors: &[u64]) -> u64 {
    let mut h = self_color;
    h = mix(h, 0x9e37_79b9_7f4a_7c15);
    for &c in neighbor_colors {
        h = mix(h, c);
    }
    finalize(h)
}

fn compute_graph_hash(colors: &[u64]) -> u64 {
    let mut sorted: Vec<u64> = colors.to_vec();
    sorted.sort_unstable();

    let mut h: u64 = sorted.len() as u64;
    for &c in &sorted {
        h = mix(h, c);
    }
    finalize(h)
}

#[inline]
fn mix(mut h: u64, k: u64) -> u64 {
    h ^= k;
    h = h.wrapping_mul(0x517c_c1b7_2722_0a95);
    h ^= h >> 33;
    h
}

#[inline]
fn finalize(mut h: u64) -> u64 {
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    h ^= h >> 33;
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 0)], false, Some(3)).unwrap()
    }

    fn path3() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2)], false, Some(3)).unwrap()
    }

    fn cycle4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 0)], false, Some(4)).unwrap()
    }

    fn complete4() -> Graph {
        Graph::from_edges(
            &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            false,
            Some(4),
        )
        .unwrap()
    }

    #[test]
    fn same_graph_same_hash() {
        let g1 = triangle();
        let g2 = triangle();
        let h1 = wl_hash(&g1, None, 5).unwrap();
        let h2 = wl_hash(&g2, None, 5).unwrap();
        assert_eq!(h1.graph_hash, h2.graph_hash);
        assert_eq!(h1.vertex_hashes, h2.vertex_hashes);
    }

    #[test]
    fn different_structure_different_hash() {
        let g1 = triangle();
        let g2 = path3();
        let h1 = wl_hash(&g1, None, 5).unwrap();
        let h2 = wl_hash(&g2, None, 5).unwrap();
        assert_ne!(h1.graph_hash, h2.graph_hash);
    }

    #[test]
    fn cycle_vs_complete() {
        let g1 = cycle4();
        let g2 = complete4();
        let h1 = wl_hash(&g1, None, 5).unwrap();
        let h2 = wl_hash(&g2, None, 5).unwrap();
        assert_ne!(h1.graph_hash, h2.graph_hash);
    }

    #[test]
    fn triangle_vertices_all_same() {
        let g = triangle();
        let h = wl_hash(&g, None, 5).unwrap();
        assert_eq!(h.vertex_hashes[0], h.vertex_hashes[1]);
        assert_eq!(h.vertex_hashes[1], h.vertex_hashes[2]);
    }

    #[test]
    fn path_endpoints_same_center_different() {
        let g = path3();
        let h = wl_hash(&g, None, 5).unwrap();
        assert_eq!(h.vertex_hashes[0], h.vertex_hashes[2]);
        assert_ne!(h.vertex_hashes[0], h.vertex_hashes[1]);
    }

    #[test]
    fn with_labels() {
        let g = triangle();
        let labels1 = vec![0, 0, 0];
        let labels2 = vec![0, 1, 0];
        let h1 = wl_hash(&g, Some(&labels1), 5).unwrap();
        let h2 = wl_hash(&g, Some(&labels2), 5).unwrap();
        assert_ne!(h1.graph_hash, h2.graph_hash);
    }

    #[test]
    fn deterministic() {
        let g = cycle4();
        let h1 = wl_hash(&g, None, 10).unwrap();
        let h2 = wl_hash(&g, None, 10).unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn stabilization() {
        let g = triangle();
        let h = wl_hash(&g, None, 100).unwrap();
        assert!(h.iterations <= 2);
    }

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let h = wl_hash(&g, None, 5).unwrap();
        assert!(h.vertex_hashes.is_empty());
        assert_eq!(h.graph_hash, 0);
        assert_eq!(h.iterations, 0);
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        let h = wl_hash(&g, None, 5).unwrap();
        assert_eq!(h.vertex_hashes.len(), 1);
        assert!(h.iterations >= 1);
    }

    #[test]
    fn disconnected_components() {
        // Two isolated edges: 0-1 and 2-3
        let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        let h = wl_hash(&g, None, 5).unwrap();
        assert_eq!(h.vertex_hashes[0], h.vertex_hashes[2]);
        assert_eq!(h.vertex_hashes[1], h.vertex_hashes[3]);
    }

    #[test]
    fn wl_isomorphic_same() {
        let g1 = cycle4();
        let g2 = cycle4();
        assert!(wl_isomorphic(&g1, &g2, None, None, 5).unwrap());
    }

    #[test]
    fn wl_isomorphic_different() {
        let g1 = triangle();
        let g2 = path3();
        assert!(!wl_isomorphic(&g1, &g2, None, None, 5).unwrap());
    }

    #[test]
    fn wl_isomorphic_different_size() {
        let g1 = triangle();
        let g2 = cycle4();
        assert!(!wl_isomorphic(&g1, &g2, None, None, 5).unwrap());
    }

    #[test]
    fn wl_hash_iterations_count() {
        let g = cycle4();
        let iters = wl_hash_iterations(&g, None, 5).unwrap();
        assert!(iters.len() >= 2);
        assert_eq!(iters[0].iterations, 0);
        assert_eq!(
            iters.last().unwrap().iterations,
            u32::try_from(iters.len()).unwrap() - 1
        );
    }

    #[test]
    fn isomorphic_relabeled() {
        // C4: 0-1-2-3-0 vs relabeled: 0-2-1-3-0
        let g1 = Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 0)], false, Some(4)).unwrap();
        let g2 = Graph::from_edges(&[(0, 2), (2, 1), (1, 3), (3, 0)], false, Some(4)).unwrap();
        assert!(wl_isomorphic(&g1, &g2, None, None, 5).unwrap());
    }

    #[test]
    fn directed_graph() {
        let g1 = Graph::from_edges(&[(0, 1), (1, 2), (2, 0)], true, Some(3)).unwrap();
        let g2 = Graph::from_edges(&[(0, 1), (1, 2), (2, 0)], true, Some(3)).unwrap();
        let h1 = wl_hash(&g1, None, 5).unwrap();
        let h2 = wl_hash(&g2, None, 5).unwrap();
        assert_eq!(h1.graph_hash, h2.graph_hash);
    }

    #[test]
    fn labels_length_mismatch() {
        let g = triangle();
        let result = wl_hash(&g, Some(&[0, 1]), 5);
        assert!(result.is_err());
    }
}

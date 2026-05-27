//! Degree correlation function `k_nn(k)` (ALGO-PR-039).
//!
//! Computes the mean degree of neighbors at distance 1, grouped by
//! the source vertex degree. Supports directed mode selection for
//! both source and target degrees, optional weights, and the
//! `directed_neighbors` flag.
//!
//! Counterpart of `igraph_degree_correlation_vector` from
//! `references/igraph/src/properties/degrees.c`.

use crate::algorithms::properties::degree::DegreeMode;
use crate::core::{Graph, IgraphResult};

/// Compute the degree correlation function `k_nn(k)`.
///
/// For each degree value `d`, computes the weighted average degree of
/// vertices connected to by vertices of degree `d`. The result vector
/// is indexed by degree: `result[d]` = mean neighbor degree for source
/// degree `d`.
///
/// - `from_mode`: how to measure source vertex degree (Out/In/All).
/// - `to_mode`: how to measure target vertex degree (Out/In/All).
/// - `directed_neighbors`: if true, only follow edge direction (u→v).
///   If false, treat edges as reciprocal (both u→v and v→u counted).
///   Ignored for undirected graphs.
/// - `weights`: optional edge weights. If `None`, all weights are 1.
///
/// Returns a vector of length `max_from_degree + 1`. Entries for
/// degrees with no edges will be `f64::NAN`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, DegreeMode, degree_correlation_vector};
///
/// // Star graph: center has degree 4, leaves have degree 1
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// g.add_edge(0, 4).unwrap();
/// let knnk = degree_correlation_vector(
///     &g, DegreeMode::All, DegreeMode::All, false, None,
/// ).unwrap();
/// // k_nn(1) = 4.0 (leaves connect to center with deg 4)
/// assert!((knnk[1] - 4.0).abs() < 1e-10);
/// // k_nn(4) = 1.0 (center connects to leaves with deg 1)
/// assert!((knnk[4] - 1.0).abs() < 1e-10);
/// ```
pub fn degree_correlation_vector(
    graph: &Graph,
    from_mode: DegreeMode,
    to_mode: DegreeMode,
    directed_neighbors: bool,
    weights: Option<&[f64]>,
) -> IgraphResult<Vec<f64>> {
    let ecount = graph.ecount();

    if graph.vcount() == 0 {
        return Ok(Vec::new());
    }

    if let Some(w) = weights {
        if w.len() != ecount {
            return Err(crate::core::IgraphError::InvalidArgument(format!(
                "degree_correlation_vector: weight vector length ({}) does not match edge count ({ecount})",
                w.len()
            )));
        }
    }

    let (from_mode_eff, to_mode_eff, directed_eff) = if graph.is_directed() {
        (from_mode, to_mode, directed_neighbors)
    } else {
        (DegreeMode::All, DegreeMode::All, false)
    };

    let deg_from = compute_degrees(graph, from_mode_eff)?;
    let deg_to = compute_degrees(graph, to_mode_eff)?;

    let max_from_deg = if ecount > 0 {
        deg_from.iter().copied().max().unwrap_or(0) as usize
    } else {
        0
    };

    let mut knnk = vec![0.0_f64; max_from_deg + 1];
    let mut weight_sums = vec![0.0_f64; max_from_deg + 1];

    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (from, to) = graph.edge(eid as u32)?;
        let from_deg = deg_from[from as usize] as usize;
        let to_deg = f64::from(deg_to[to as usize]);
        let w = weights.map_or(1.0, |ws| ws[eid]);

        weight_sums[from_deg] += w;
        knnk[from_deg] += w * to_deg;

        if !directed_eff {
            let to_from_deg = deg_from[to as usize] as usize;
            let from_to_deg = f64::from(deg_to[from as usize]);
            weight_sums[to_from_deg] += w;
            knnk[to_from_deg] += w * from_to_deg;
        }
    }

    for i in 0..knnk.len() {
        if weight_sums[i] > 0.0 {
            knnk[i] /= weight_sums[i];
        } else {
            knnk[i] = f64::NAN;
        }
    }

    Ok(knnk)
}

fn compute_degrees(graph: &Graph, mode: DegreeMode) -> IgraphResult<Vec<u32>> {
    let n = graph.vcount() as usize;
    let ecount = graph.ecount();
    let mut deg = vec![0u32; n];

    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (from, to) = graph.edge(eid as u32)?;

        match mode {
            DegreeMode::Out => {
                deg[from as usize] += 1;
            }
            DegreeMode::In => {
                deg[to as usize] += 1;
            }
            DegreeMode::All => {
                deg[from as usize] += 1;
                if from == to {
                    deg[from as usize] += 1;
                } else {
                    deg[to as usize] += 1;
                }
            }
        }
    }

    Ok(deg)
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-10
    }

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let knnk =
            degree_correlation_vector(&g, DegreeMode::All, DegreeMode::All, false, None).unwrap();
        assert!(knnk.is_empty());
    }

    #[test]
    fn isolated_vertices() {
        let g = Graph::with_vertices(5);
        let knnk =
            degree_correlation_vector(&g, DegreeMode::All, DegreeMode::All, false, None).unwrap();
        assert_eq!(knnk.len(), 1); // max_deg = 0
        assert!(knnk[0].is_nan());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let knnk =
            degree_correlation_vector(&g, DegreeMode::All, DegreeMode::All, false, None).unwrap();
        // Both vertices have degree 1, connected to degree 1
        assert_eq!(knnk.len(), 2);
        assert!(knnk[0].is_nan());
        assert!(approx_eq(knnk[1], 1.0));
    }

    #[test]
    fn star_graph() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        let knnk =
            degree_correlation_vector(&g, DegreeMode::All, DegreeMode::All, false, None).unwrap();
        assert_eq!(knnk.len(), 5); // max_deg = 4
        assert!(knnk[0].is_nan());
        assert!(approx_eq(knnk[1], 4.0)); // leaves → center (deg 4)
        assert!(knnk[2].is_nan());
        assert!(knnk[3].is_nan());
        assert!(approx_eq(knnk[4], 1.0)); // center → leaves (deg 1)
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let knnk =
            degree_correlation_vector(&g, DegreeMode::All, DegreeMode::All, false, None).unwrap();
        // All vertices deg 2, all neighbors deg 2
        assert!(approx_eq(knnk[2], 2.0));
    }

    #[test]
    fn path_3() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let knnk =
            degree_correlation_vector(&g, DegreeMode::All, DegreeMode::All, false, None).unwrap();
        // deg 1 vertices (0, 2) connect to deg 2 vertex (1): knnk[1] = 2.0
        // deg 2 vertex (1) connects to deg 1 vertices: knnk[2] = 1.0
        assert!(approx_eq(knnk[1], 2.0));
        assert!(approx_eq(knnk[2], 1.0));
    }

    #[test]
    fn directed_out_in() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap(); // 0 out=1, 1 in=1
        g.add_edge(0, 2).unwrap(); // 0 out=2, 2 in=1
        g.add_edge(1, 2).unwrap(); // 1 out=1, 2 in=2
        // from_mode=Out, to_mode=In, directed=true
        let knnk =
            degree_correlation_vector(&g, DegreeMode::Out, DegreeMode::In, true, None).unwrap();
        // from out-degrees: 0→2, 1→1
        // Edge 0→1: from_deg[0]=2 (out), to_deg[1]=1 (in)
        // Edge 0→2: from_deg[0]=2 (out), to_deg[2]=2 (in)
        // Edge 1→2: from_deg[1]=1 (out), to_deg[2]=2 (in)
        // knnk[2] = (1*1 + 1*2) / (1+1) = 3/2 = 1.5
        // knnk[1] = (1*2) / 1 = 2.0
        assert_eq!(knnk.len(), 3);
        assert!(knnk[0].is_nan());
        assert!(approx_eq(knnk[1], 2.0));
        assert!(approx_eq(knnk[2], 1.5));
    }

    #[test]
    fn directed_undirected_neighbors() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        // directed_neighbors=false: treat each edge as reciprocal
        let knnk =
            degree_correlation_vector(&g, DegreeMode::Out, DegreeMode::Out, false, None).unwrap();
        // out-degrees: 0→1, 1→1, 2→0
        // Edge 0→1: forward: from_deg[0]=1, to_deg[1]=1; reverse: from_deg[1]=1, to_deg[0]=1
        // Edge 1→2: forward: from_deg[1]=1, to_deg[2]=0; reverse: from_deg[2]=0, to_deg[1]=1
        // For deg 1: contributions = (1*1) + (1*1) + (1*0) = 2; weight_sum = 3
        // knnk[1] = 2/3
        // For deg 0: contributions = (1*1); weight_sum = 1; knnk[0] = 1.0
        assert!(approx_eq(knnk[0], 1.0));
        assert!(approx_eq(knnk[1], 2.0 / 3.0));
    }

    #[test]
    fn weighted() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let weights = vec![2.0, 1.0];
        let knnk =
            degree_correlation_vector(&g, DegreeMode::All, DegreeMode::All, false, Some(&weights))
                .unwrap();
        // deg 1: vertices 0 and 2
        //   Edge 0-1 (w=2): 0(deg1)→1(deg2) contributes 2*2=4 / ws+=2
        //                  + 1(deg2)→0(deg1) contributes 2*1=2 / ws+=2
        //   Edge 1-2 (w=1): 2(deg1)→1(deg2) contributes 1*2=2 / ws+=1
        //                  + 1(deg2)→2(deg1) contributes 1*1=1 / ws+=1
        // knnk[1] = (4 + 2) / (2 + 1) = 6/3 = 2.0
        // knnk[2] = (2 + 1) / (2 + 1) = 3/3 = 1.0
        assert!(approx_eq(knnk[1], 2.0));
        assert!(approx_eq(knnk[2], 1.0));
    }

    #[test]
    fn weight_length_mismatch() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let bad_weights = vec![1.0, 2.0, 3.0];
        let result = degree_correlation_vector(
            &g,
            DegreeMode::All,
            DegreeMode::All,
            false,
            Some(&bad_weights),
        );
        assert!(result.is_err());
    }

    #[test]
    fn self_loop_counted() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap(); // self-loop: adds 2 to deg(0)
        g.add_edge(0, 1).unwrap();
        let knnk =
            degree_correlation_vector(&g, DegreeMode::All, DegreeMode::All, false, None).unwrap();
        // deg(0) = 3 (self-loop counts 2 + edge to 1), deg(1) = 1
        // Edge 0-0: from_deg[0]=3, to_deg[0]=3
        //   forward: ws[3] += 1, knnk[3] += 3
        //   reverse: ws[3] += 1, knnk[3] += 3
        // Edge 0-1: from_deg[0]=3, to_deg[1]=1
        //   forward: ws[3] += 1, knnk[3] += 1
        //   reverse: ws[1] += 1, knnk[1] += 3
        // knnk[3] = (3+3+1) / (1+1+1) = 7/3
        // knnk[1] = 3 / 1 = 3.0
        assert!(approx_eq(knnk[1], 3.0));
        assert!(approx_eq(knnk[3], 7.0 / 3.0));
    }

    #[test]
    fn complete_k4() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        let knnk =
            degree_correlation_vector(&g, DegreeMode::All, DegreeMode::All, false, None).unwrap();
        // All deg 3, all neighbors deg 3
        assert!(approx_eq(knnk[3], 3.0));
    }
}

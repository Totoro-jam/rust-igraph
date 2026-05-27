//! Stochastic adjacency matrix (ALGO-PR-043).
//!
//! Returns the row-wise or column-wise normalized adjacency matrix,
//! representing transition probabilities of a random walk.
//!
//! Counterpart of `igraph_get_stochastic` from
//! `references/igraph/src/misc/conversion.c`.

use crate::core::{Graph, IgraphError, IgraphResult};

/// Compute the stochastic adjacency matrix of a graph.
///
/// The stochastic matrix is the adjacency matrix normalized so that each
/// row (or column) sums to 1. Row-normalized = right-stochastic =
/// random walk following edge directions. Column-normalized =
/// left-stochastic = random walk against edge directions.
///
/// When a vertex has zero out-degree (or in-degree for column-wise),
/// the corresponding row (or column) will be all zeros.
///
/// - `column_wise`: if false, row-normalize (right-stochastic);
///   if true, column-normalize (left-stochastic).
/// - `weights`: optional edge weights. If `None`, each edge counts as 1.
///
/// Returns an n×n matrix as `Vec<Vec<f64>>`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, get_stochastic};
///
/// // Path 0-1-2: row-stochastic
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let s = get_stochastic(&g, false, None).unwrap();
/// // Row 0 sums to 1: only edge to vertex 1
/// assert!((s[0][1] - 1.0).abs() < 1e-10);
/// // Row 1: edges to 0 and 2, each 0.5
/// assert!((s[1][0] - 0.5).abs() < 1e-10);
/// assert!((s[1][2] - 0.5).abs() < 1e-10);
/// ```
pub fn get_stochastic(
    graph: &Graph,
    column_wise: bool,
    weights: Option<&[f64]>,
) -> IgraphResult<Vec<Vec<f64>>> {
    let n = graph.vcount() as usize;
    let ecount = graph.ecount();

    if let Some(w) = weights {
        if w.len() != ecount {
            return Err(IgraphError::InvalidArgument(format!(
                "get_stochastic: weight vector length ({}) does not match edge count ({ecount})",
                w.len()
            )));
        }
    }

    let directed = graph.is_directed();

    // Compute strength (weighted degree) for normalization
    let mut sums = vec![0.0_f64; n];
    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (from, to) = graph.edge(eid as u32)?;
        let w = weights.map_or(1.0, |ws| ws[eid]);

        if directed {
            if column_wise {
                sums[to as usize] += w;
            } else {
                sums[from as usize] += w;
            }
        } else {
            // Undirected: both endpoints get the weight
            sums[from as usize] += w;
            if from == to {
                sums[from as usize] += w;
            } else {
                sums[to as usize] += w;
            }
        }
    }

    let mut res = vec![vec![0.0_f64; n]; n];

    if directed {
        for eid in 0..ecount {
            #[allow(clippy::cast_possible_truncation)]
            let (from, to) = graph.edge(eid as u32)?;
            let f = from as usize;
            let t = to as usize;
            let w = weights.map_or(1.0, |ws| ws[eid]);

            let divisor = if column_wise { sums[t] } else { sums[f] };
            if divisor > 0.0 {
                res[f][t] += w / divisor;
            }
        }
    } else {
        for eid in 0..ecount {
            #[allow(clippy::cast_possible_truncation)]
            let (from, to) = graph.edge(eid as u32)?;
            let f = from as usize;
            let t = to as usize;
            let w = weights.map_or(1.0, |ws| ws[eid]);

            let div_from = if column_wise { sums[t] } else { sums[f] };
            if div_from > 0.0 {
                res[f][t] += w / div_from;
            }

            let div_to = if column_wise { sums[f] } else { sums[t] };
            if div_to > 0.0 {
                res[t][f] += w / div_to;
            }
        }
    }

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-10
    }

    fn row_sum(matrix: &[Vec<f64>], row: usize) -> f64 {
        matrix[row].iter().sum()
    }

    fn col_sum(matrix: &[Vec<f64>], col: usize) -> f64 {
        matrix.iter().map(|row| row[col]).sum()
    }

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let s = get_stochastic(&g, false, None).unwrap();
        assert!(s.is_empty());
    }

    #[test]
    fn isolated_vertex() {
        let g = Graph::with_vertices(2);
        let s = get_stochastic(&g, false, None).unwrap();
        // All zeros
        for row in &s {
            for &val in row {
                assert!(approx_eq(val, 0.0));
            }
        }
    }

    #[test]
    fn single_edge_row() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let s = get_stochastic(&g, false, None).unwrap();
        // Row 0: only connects to 1, sum = 1
        assert!(approx_eq(s[0][1], 1.0));
        assert!(approx_eq(s[1][0], 1.0));
        assert!(approx_eq(row_sum(&s, 0), 1.0));
        assert!(approx_eq(row_sum(&s, 1), 1.0));
    }

    #[test]
    fn path_3_row() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let s = get_stochastic(&g, false, None).unwrap();
        // vertex 0: deg=1, only to 1
        assert!(approx_eq(s[0][1], 1.0));
        // vertex 1: deg=2, to 0 and 2
        assert!(approx_eq(s[1][0], 0.5));
        assert!(approx_eq(s[1][2], 0.5));
        // vertex 2: deg=1, only to 1
        assert!(approx_eq(s[2][1], 1.0));
        // All rows sum to 1
        for i in 0..3 {
            assert!(approx_eq(row_sum(&s, i), 1.0));
        }
    }

    #[test]
    fn path_3_column() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let s = get_stochastic(&g, true, None).unwrap();
        // Columns should sum to 1 for non-isolated vertices
        assert!(approx_eq(col_sum(&s, 0), 1.0));
        assert!(approx_eq(col_sum(&s, 1), 1.0));
        assert!(approx_eq(col_sum(&s, 2), 1.0));
    }

    #[test]
    fn directed_row() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let s = get_stochastic(&g, false, None).unwrap();
        // out-deg(0)=2: s[0][1]=0.5, s[0][2]=0.5
        assert!(approx_eq(s[0][1], 0.5));
        assert!(approx_eq(s[0][2], 0.5));
        // out-deg(1)=1: s[1][2]=1.0
        assert!(approx_eq(s[1][2], 1.0));
        // out-deg(2)=0: row all zeros
        assert!(approx_eq(row_sum(&s, 2), 0.0));
    }

    #[test]
    fn directed_column() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let s = get_stochastic(&g, true, None).unwrap();
        // in-deg(0)=0: column all zeros
        assert!(approx_eq(col_sum(&s, 0), 0.0));
        // in-deg(1)=1: s[0][1]=1.0
        assert!(approx_eq(s[0][1], 1.0));
        // in-deg(2)=2: s[0][2]+s[1][2]=1.0
        assert!(approx_eq(s[0][2], 0.5));
        assert!(approx_eq(s[1][2], 0.5));
    }

    #[test]
    fn weighted_row() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        let weights = vec![2.0, 3.0];
        let s = get_stochastic(&g, false, Some(&weights)).unwrap();
        // strength(0)=5: s[0][1]=2/5, s[0][2]=3/5
        assert!(approx_eq(s[0][1], 2.0 / 5.0));
        assert!(approx_eq(s[0][2], 3.0 / 5.0));
        assert!(approx_eq(row_sum(&s, 0), 1.0));
    }

    #[test]
    fn self_loop() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let s = get_stochastic(&g, false, None).unwrap();
        // deg(0)=3 (self-loop counts 2 + edge to 1)
        // s[0][0] = 1/3 (self-loop once in matrix) — wait, the C code
        // adds to res[from][to], so self-loop adds w/sum once.
        // Actually for undirected, both from→to and to→from are added.
        // Self-loop: from=to=0, so res[0][0] gets w/sums[0] twice
        // sums[0] = 2 + 1 = 3 (self-loop=2, edge=1)
        // res[0][0] += 1/3 (first add) + 1/3 (second add) = 2/3
        // res[0][1] += 1/3
        // res[1][0] += 1/1 = 1.0
        assert!(approx_eq(s[0][0], 2.0 / 3.0));
        assert!(approx_eq(s[0][1], 1.0 / 3.0));
        assert!(approx_eq(row_sum(&s, 0), 1.0));
    }

    #[test]
    fn complete_k3_row() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let s = get_stochastic(&g, false, None).unwrap();
        // All degrees = 2, uniform distribution to neighbors
        for i in 0..3 {
            assert!(approx_eq(row_sum(&s, i), 1.0));
        }
        assert!(approx_eq(s[0][1], 0.5));
        assert!(approx_eq(s[0][2], 0.5));
    }

    #[test]
    fn weight_mismatch() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let result = get_stochastic(&g, false, Some(&[1.0, 2.0]));
        assert!(result.is_err());
    }
}

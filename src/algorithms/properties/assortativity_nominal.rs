//! Categorical (nominal) assortativity coefficient (ALGO-PR-046).
//!
//! Computes the assortativity coefficient of a graph based on discrete
//! vertex categories. Measures the tendency of vertices to connect to
//! others in the same category.
//!
//! Counterpart of `igraph_assortativity_nominal` from
//! `references/igraph/src/misc/mixing.c`.

use crate::core::{Graph, IgraphError, IgraphResult};

/// Compute categorical (nominal) assortativity coefficient.
///
/// Given integer category labels for each vertex, measures whether edges
/// tend to connect vertices of the same type (positive) or different
/// types (negative). The coefficient ranges from -1 to 1: +1 means all
/// edges stay within categories, 0 means random mixing, -1 means
/// perfectly disassortative.
///
/// The unnormalized version equals the modularity.
///
/// - `types`: integer category label for each vertex (0-based, length must
///   equal vertex count).
/// - `directed`: if true and the graph is directed, respect edge direction;
///   otherwise treat as undirected (each edge counted in both directions).
/// - `normalized`: if true, return the standard normalized assortativity;
///   if false, return the unnormalized version (= modularity).
///
/// Returns `f64::NAN` for graphs with no vertices or no edges.
///
/// # References
///
/// M. E. J. Newman: Mixing patterns in networks,
/// Phys. Rev. E 67, 026126 (2003).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, assortativity_nominal};
///
/// // Perfect assortative: all edges within same type
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap(); // type 0 - type 0
/// g.add_edge(2, 3).unwrap(); // type 1 - type 1
/// let types = vec![0u32, 0, 1, 1];
/// let r = assortativity_nominal(&g, &types, false, true).unwrap();
/// assert!(r > 0.99); // close to 1.0
/// ```
pub fn assortativity_nominal(
    graph: &Graph,
    types: &[u32],
    directed: bool,
    normalized: bool,
) -> IgraphResult<f64> {
    let n = graph.vcount();
    let ecount = graph.ecount();

    #[allow(clippy::cast_possible_truncation)]
    if types.len() != n as usize {
        return Err(IgraphError::InvalidArgument(format!(
            "assortativity_nominal: types length ({}) does not match vertex count ({n})",
            types.len()
        )));
    }

    if n == 0 || ecount == 0 {
        return Ok(f64::NAN);
    }

    let directed = directed && graph.is_directed();

    let no_of_types = types
        .iter()
        .copied()
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| {
            IgraphError::InvalidArgument("assortativity_nominal: type value overflow".to_string())
        })? as usize;

    let mut ai: Vec<u64> = vec![0; no_of_types];
    let mut bi: Vec<u64> = vec![0; no_of_types];
    let mut eii: Vec<u64> = vec![0; no_of_types];

    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (from, to) = graph.edge(eid as u32)?;
        let from_type = types[from as usize] as usize;
        let to_type = types[to as usize] as usize;

        ai[from_type] += 1;
        bi[to_type] += 1;
        if from_type == to_type {
            eii[from_type] += 1;
        }
        if !directed {
            if from_type == to_type {
                eii[from_type] += 1;
            }
            ai[to_type] += 1;
            bi[from_type] += 1;
        }
    }

    #[allow(clippy::cast_precision_loss)]
    let m = ecount as f64;

    let mut sumaibi = 0.0_f64;
    let mut sumeii = 0.0_f64;

    for i in 0..no_of_types {
        #[allow(clippy::cast_precision_loss)]
        let ai_f = ai[i] as f64;
        #[allow(clippy::cast_precision_loss)]
        let bi_f = bi[i] as f64;
        #[allow(clippy::cast_precision_loss)]
        let eii_f = eii[i] as f64;

        sumaibi += (ai_f / m) * (bi_f / m);
        sumeii += eii_f / m;
    }

    if !directed {
        sumaibi /= 4.0;
        sumeii /= 2.0;
    }

    if normalized {
        let denom = 1.0 - sumaibi;
        if denom.abs() < f64::EPSILON {
            return Ok(f64::NAN);
        }
        Ok((sumeii - sumaibi) / denom)
    } else {
        Ok(sumeii - sumaibi)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-10
    }

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let r = assortativity_nominal(&g, &[], false, true).unwrap();
        assert!(r.is_nan());
    }

    #[test]
    fn no_edges() {
        let g = Graph::with_vertices(3);
        let types = vec![0, 1, 2];
        let r = assortativity_nominal(&g, &types, false, true).unwrap();
        assert!(r.is_nan());
    }

    #[test]
    fn perfect_assortative_undirected() {
        // Two clusters, all internal edges
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let types = vec![0, 0, 1, 1];
        let r = assortativity_nominal(&g, &types, false, true).unwrap();
        assert!(approx_eq(r, 1.0));
    }

    #[test]
    fn perfect_disassortative_undirected() {
        // Complete bipartite K2,2: all edges cross types
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        let types = vec![0, 0, 1, 1];
        let r = assortativity_nominal(&g, &types, false, true).unwrap();
        assert!(approx_eq(r, -1.0));
    }

    #[test]
    fn mixed_undirected() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap(); // type 0 - type 0
        g.add_edge(1, 2).unwrap(); // type 0 - type 1
        g.add_edge(0, 2).unwrap(); // type 0 - type 1
        let types = vec![0, 0, 1];
        let r = assortativity_nominal(&g, &types, false, true).unwrap();
        assert!(approx_eq(r, -0.5));
    }

    #[test]
    fn directed_graph() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap(); // type 0 → type 0
        g.add_edge(1, 2).unwrap(); // type 0 → type 1
        let types = vec![0, 0, 1];
        let r = assortativity_nominal(&g, &types, true, true).unwrap();
        // Verified against python-igraph: 0.0
        assert!(approx_eq(r, 0.0));
    }

    #[test]
    fn directed_as_undirected() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let types = vec![0, 0, 1, 1];
        // directed=false → treat as undirected
        let r = assortativity_nominal(&g, &types, false, true).unwrap();
        assert!(approx_eq(r, 1.0));
    }

    #[test]
    fn unnormalized_equals_modularity() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        let types = vec![0, 0, 1, 1];
        let r = assortativity_nominal(&g, &types, false, false).unwrap();
        // Verified against python-igraph: 1/6
        assert!(approx_eq(r, 1.0 / 6.0));
    }

    #[test]
    fn types_length_mismatch() {
        let g = Graph::with_vertices(3);
        let types = vec![0, 1]; // wrong length
        assert!(assortativity_nominal(&g, &types, false, true).is_err());
    }

    #[test]
    fn single_type_all_same() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let types = vec![0, 0, 0];
        let r = assortativity_nominal(&g, &types, false, true).unwrap();
        // All same type → sumaibi=1.0, sumeii=1.0
        // (1-1)/(1-1) → NaN (degenerate)
        assert!(r.is_nan());
    }

    #[test]
    fn self_loop_undirected() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap(); // self-loop
        g.add_edge(0, 1).unwrap();
        let types = vec![0, 1];
        let r = assortativity_nominal(&g, &types, false, true).unwrap();
        // Verified against python-igraph: -1/3
        assert!(approx_eq(r, -1.0 / 3.0));
    }

    #[test]
    fn many_types() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap(); // type 0 - type 1
        g.add_edge(2, 3).unwrap(); // type 2 - type 0
        g.add_edge(4, 5).unwrap(); // type 1 - type 2
        let types = vec![0, 1, 2, 0, 1, 2];
        let r = assortativity_nominal(&g, &types, false, true).unwrap();
        assert!(r < 0.0);
    }
}

//! Graph product operators (ALGO-OP-015, ALGO-OP-027, ALGO-OP-029).
//!
//! Implements six graph products: Cartesian, tensor (categorical),
//! strong, lexicographic, rooted, and modular. Also provides a unified
//! `graph_product` dispatcher (`igraph_product`).

use crate::core::error::IgraphError;
use crate::core::{Graph, IgraphResult, VertexId};

/// Selects which graph product type to compute.
///
/// Passed to [`graph_product`] to select the product type.
/// See each variant's documentation for the adjacency condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphProductType {
    /// Cartesian product: `(u1,v1) ~ (u2,v2)` iff
    /// `u1=u2, v1~v2` or `u1~u2, v1=v2`.
    Cartesian,
    /// Lexicographic product: `(u1,v1) ~ (u2,v2)` iff
    /// `u1~u2` or (`u1=u2` and `v1~v2`). Not commutative.
    Lexicographic,
    /// Strong product (normal product): union of Cartesian and tensor.
    Strong,
    /// Tensor (categorical/direct) product: `(u1,v1) ~ (u2,v2)` iff
    /// `u1~u2` and `v1~v2`.
    Tensor,
    /// Modular product: `(u1,v1) ~ (u2,v2)` iff
    /// (`u1~u2` and `v1~v2`) or (`u1≁u2` and `v1≁v2`),
    /// where `u1≠u2` and `v1≠v2`. Requires simple inputs.
    Modular,
}

/// Computes a graph product selected by `product_type`.
///
/// Unified dispatcher for all five non-rooted graph product types,
/// matching the C `igraph_product()` function. The rooted product
/// requires an extra `root` parameter and is available separately
/// via [`rooted_product`].
///
/// Both graphs must have the same directedness. The result has
/// `|V1| * |V2|` vertices where vertex `(i, j)` is identified by
/// `i * |V2| + j`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, graph_product, GraphProductType};
///
/// let mut g1 = Graph::with_vertices(2);
/// g1.add_edge(0, 1).unwrap();
/// let mut g2 = Graph::with_vertices(2);
/// g2.add_edge(0, 1).unwrap();
///
/// let p = graph_product(&g1, &g2, GraphProductType::Cartesian).unwrap();
/// assert_eq!(p.vcount(), 4);
/// assert_eq!(p.ecount(), 4);
/// ```
pub fn graph_product(
    g1: &Graph,
    g2: &Graph,
    product_type: GraphProductType,
) -> IgraphResult<Graph> {
    match product_type {
        GraphProductType::Cartesian => cartesian_product(g1, g2),
        GraphProductType::Lexicographic => lexicographic_product(g1, g2),
        GraphProductType::Strong => strong_product(g1, g2),
        GraphProductType::Tensor => tensor_product(g1, g2),
        GraphProductType::Modular => modular_product(g1, g2),
    }
}

/// Computes the Cartesian product of two graphs.
///
/// The result has `|V1| * |V2|` vertices. Vertex `(i, j)` in the product
/// is identified by index `i * |V2| + j`. An edge exists between `(i, j)`
/// and `(k, l)` iff:
/// - `i == k` and `(j, l)` is an edge in `g2`, OR
/// - `j == l` and `(i, k)` is an edge in `g1`.
///
/// Both graphs must have the same directedness.
///
/// # Arguments
///
/// * `g1` — the first factor graph.
/// * `g2` — the second factor graph.
///
/// # Errors
///
/// Returns `InvalidArgument` if the graphs differ in directedness, or if
/// the product vertex count overflows `u32`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, cartesian_product};
///
/// // P2 □ P2 = C4 (path of 2 vertices □ path of 2 vertices = 4-cycle)
/// let mut g1 = Graph::with_vertices(2);
/// g1.add_edge(0, 1).unwrap();
/// let mut g2 = Graph::with_vertices(2);
/// g2.add_edge(0, 1).unwrap();
///
/// let p = cartesian_product(&g1, &g2).unwrap();
/// assert_eq!(p.vcount(), 4);
/// assert_eq!(p.ecount(), 4);
/// ```
pub fn cartesian_product(g1: &Graph, g2: &Graph) -> IgraphResult<Graph> {
    check_same_directedness(g1, g2, "cartesian_product")?;

    let n1 = g1.vcount();
    let n2 = g2.vcount();
    let directed = g1.is_directed();

    let n = product_vertex_count(n1, n2)?;

    if n == 0 {
        return Graph::new(0, directed);
    }

    let e1 = g1.ecount();
    let e2 = g2.ecount();

    let total_edges = (n1 as usize)
        .checked_mul(e2)
        .and_then(|a| a.checked_add((n2 as usize).checked_mul(e1)?))
        .ok_or_else(|| {
            IgraphError::InvalidArgument("edge count overflow in cartesian_product".to_string())
        })?;

    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(total_edges);

    // For each g1 edge (u, v), add edges (u,j)→(v,j) for all j in V2
    for eid in 0..e1 {
        #[allow(clippy::cast_possible_truncation)]
        let (u, v) = g1.edge(eid as u32)?;
        for j in 0..n2 {
            let src = u * n2 + j;
            let tgt = v * n2 + j;
            edges.push((src, tgt));
        }
    }

    // For each g2 edge (u, v), add edges (i,u)→(i,v) for all i in V1
    for eid in 0..e2 {
        #[allow(clippy::cast_possible_truncation)]
        let (u, v) = g2.edge(eid as u32)?;
        for i in 0..n1 {
            let src = i * n2 + u;
            let tgt = i * n2 + v;
            edges.push((src, tgt));
        }
    }

    let mut result = Graph::new(n, directed)?;
    result.add_edges(edges)?;
    Ok(result)
}

/// Computes the tensor (categorical/direct) product of two graphs.
///
/// The result has `|V1| * |V2|` vertices. Vertex `(i, j)` is identified by
/// `i * |V2| + j`. An edge exists between `(i, j)` and `(k, l)` iff
/// `(i, k)` is an edge in `g1` AND `(j, l)` is an edge in `g2`.
///
/// For undirected graphs, each pair of edges generates two product edges
/// (one for each orientation).
///
/// Both graphs must have the same directedness.
///
/// # Arguments
///
/// * `g1` — the first factor graph.
/// * `g2` — the second factor graph.
///
/// # Errors
///
/// Returns `InvalidArgument` if the graphs differ in directedness, or if
/// the product vertex count overflows `u32`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, tensor_product};
///
/// let mut g1 = Graph::with_vertices(3);
/// g1.add_edge(0, 1).unwrap();
/// g1.add_edge(1, 2).unwrap();
///
/// let mut g2 = Graph::with_vertices(2);
/// g2.add_edge(0, 1).unwrap();
///
/// let p = tensor_product(&g1, &g2).unwrap();
/// assert_eq!(p.vcount(), 6);
/// // 2 edges × 1 edge × 2 (undirected) = 4 edges
/// assert_eq!(p.ecount(), 4);
/// ```
pub fn tensor_product(g1: &Graph, g2: &Graph) -> IgraphResult<Graph> {
    check_same_directedness(g1, g2, "tensor_product")?;

    let n1 = g1.vcount();
    let n2 = g2.vcount();
    let directed = g1.is_directed();

    let n = product_vertex_count(n1, n2)?;

    if n == 0 {
        return Graph::new(0, directed);
    }

    let e1 = g1.ecount();
    let e2 = g2.ecount();

    let multiplier: usize = if directed { 1 } else { 2 };
    let total_edges = e1
        .checked_mul(e2)
        .and_then(|a| a.checked_mul(multiplier))
        .ok_or_else(|| {
            IgraphError::InvalidArgument("edge count overflow in tensor_product".to_string())
        })?;

    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(total_edges);

    for eid1 in 0..e1 {
        #[allow(clippy::cast_possible_truncation)]
        let (u1, v1) = g1.edge(eid1 as u32)?;
        for eid2 in 0..e2 {
            #[allow(clippy::cast_possible_truncation)]
            let (u2, v2) = g2.edge(eid2 as u32)?;

            // (u1, u2) → (v1, v2)
            let src = u1 * n2 + u2;
            let tgt = v1 * n2 + v2;
            edges.push((src, tgt));

            if !directed {
                // (u1, v2) → (v1, u2)
                let src2 = u1 * n2 + v2;
                let tgt2 = v1 * n2 + u2;
                edges.push((src2, tgt2));
            }
        }
    }

    let mut result = Graph::new(n, directed)?;
    result.add_edges(edges)?;
    Ok(result)
}

/// Computes the strong product of two graphs.
///
/// The strong product is the union of the Cartesian product and the tensor
/// product. An edge exists between `(i, j)` and `(k, l)` iff:
/// - `i == k` and `(j, l)` is an edge in `g2`, OR
/// - `j == l` and `(i, k)` is an edge in `g1`, OR
/// - `(i, k)` is an edge in `g1` AND `(j, l)` is an edge in `g2`.
///
/// Both graphs must have the same directedness.
///
/// # Arguments
///
/// * `g1` — the first factor graph.
/// * `g2` — the second factor graph.
///
/// # Errors
///
/// Returns `InvalidArgument` if the graphs differ in directedness, or if
/// the product vertex count overflows `u32`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, strong_product};
///
/// let mut g1 = Graph::with_vertices(2);
/// g1.add_edge(0, 1).unwrap();
/// let mut g2 = Graph::with_vertices(2);
/// g2.add_edge(0, 1).unwrap();
///
/// let p = strong_product(&g1, &g2).unwrap();
/// assert_eq!(p.vcount(), 4);
/// // Cartesian: 4 edges + Tensor: 2 edges = 6 edges (K4 minus one edge... actually it's 5 for the strong product of K2 x K2... let me check)
/// // Actually: K2 ⊠ K2 gives 5 edges (it's a complete graph minus one edge? No.)
/// // Cartesian of K2×K2 = C4 (4 edges), tensor of K2×K2 = 2 edges. Combined = 6? But some may overlap... no, they don't overlap for this case.
/// // Wait - let me recompute. C4 has edges: (0,0)-(0,1), (1,0)-(1,1), (0,0)-(1,0), (0,1)-(1,1) = 4 edges
/// // Tensor: (0,0)-(1,1), (0,1)-(1,0) = 2 edges. Total = 6.
/// // But K2 ⊠ K2 = K4 has 6 edges. Yes!
/// assert_eq!(p.ecount(), 6);
/// ```
pub fn strong_product(g1: &Graph, g2: &Graph) -> IgraphResult<Graph> {
    check_same_directedness(g1, g2, "strong_product")?;

    let n1 = g1.vcount();
    let n2 = g2.vcount();
    let directed = g1.is_directed();

    let n = product_vertex_count(n1, n2)?;

    if n == 0 {
        return Graph::new(0, directed);
    }

    let e1 = g1.ecount();
    let e2 = g2.ecount();

    let multiplier: usize = if directed { 1 } else { 2 };
    let cartesian_count = (n1 as usize) * e2 + (n2 as usize) * e1;
    let tensor_count = e1 * e2 * multiplier;
    let total_edges = cartesian_count.checked_add(tensor_count).ok_or_else(|| {
        IgraphError::InvalidArgument("edge count overflow in strong_product".to_string())
    })?;

    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(total_edges);

    // Cartesian part: g1 edges × V2
    for eid in 0..e1 {
        #[allow(clippy::cast_possible_truncation)]
        let (u, v) = g1.edge(eid as u32)?;
        for j in 0..n2 {
            edges.push((u * n2 + j, v * n2 + j));
        }
    }

    // Cartesian part: V1 × g2 edges
    for eid in 0..e2 {
        #[allow(clippy::cast_possible_truncation)]
        let (u, v) = g2.edge(eid as u32)?;
        for i in 0..n1 {
            edges.push((i * n2 + u, i * n2 + v));
        }
    }

    // Tensor part
    for eid1 in 0..e1 {
        #[allow(clippy::cast_possible_truncation)]
        let (u1, v1) = g1.edge(eid1 as u32)?;
        for eid2 in 0..e2 {
            #[allow(clippy::cast_possible_truncation)]
            let (u2, v2) = g2.edge(eid2 as u32)?;
            edges.push((u1 * n2 + u2, v1 * n2 + v2));
            if !directed {
                edges.push((u1 * n2 + v2, v1 * n2 + u2));
            }
        }
    }

    let mut result = Graph::new(n, directed)?;
    result.add_edges(edges)?;
    Ok(result)
}

/// Computes the lexicographic product of two graphs.
///
/// The result has `|V1| * |V2|` vertices. Vertex `(i, j)` is identified by
/// `i * |V2| + j`. An edge exists between `(i, j)` and `(k, l)` iff:
/// - `(i, k)` is an edge in `g1` (regardless of `j` and `l`), OR
/// - `i == k` and `(j, l)` is an edge in `g2`.
///
/// Note: unlike the other products, the lexicographic product is NOT
/// commutative.
///
/// Both graphs must have the same directedness.
///
/// # Arguments
///
/// * `g1` — the first factor graph (outer).
/// * `g2` — the second factor graph (inner).
///
/// # Errors
///
/// Returns `InvalidArgument` if the graphs differ in directedness, or if
/// the product vertex count overflows `u32`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, lexicographic_product};
///
/// let mut g1 = Graph::with_vertices(2);
/// g1.add_edge(0, 1).unwrap();
/// let g2 = Graph::with_vertices(3); // 3 isolated vertices
///
/// let p = lexicographic_product(&g1, &g2).unwrap();
/// assert_eq!(p.vcount(), 6);
/// // 1 edge in g1 × 3² = 9 cross-edges (undirected, so 9 unique pairs)
/// assert_eq!(p.ecount(), 9);
/// ```
pub fn lexicographic_product(g1: &Graph, g2: &Graph) -> IgraphResult<Graph> {
    check_same_directedness(g1, g2, "lexicographic_product")?;

    let n1 = g1.vcount();
    let n2 = g2.vcount();
    let directed = g1.is_directed();

    let n = product_vertex_count(n1, n2)?;

    if n == 0 {
        return Graph::new(0, directed);
    }

    let e1 = g1.ecount();
    let e2 = g2.ecount();

    // Edges from g2 part: V1 copies of g2's edges
    let g2_part = (n1 as usize) * e2;

    // Edges from g1 part: for each g1 edge, all pairs (j, l) in V2×V2
    let pairs_per_edge: usize = if directed {
        (n2 as usize) * (n2 as usize)
    } else {
        // For undirected, we only add (j,l) with j <= l for the unique pairs
        // Actually igraph C adds n2*n2 pairs and lets the graph store handle it
        // But since undirected edges are canonicalized (smaller first), we need
        // all n2*n2 pairs to get the full set of edges
        (n2 as usize) * (n2 as usize)
    };
    let g1_part = e1.checked_mul(pairs_per_edge).ok_or_else(|| {
        IgraphError::InvalidArgument("edge count overflow in lexicographic_product".to_string())
    })?;

    let total_edges = g2_part.checked_add(g1_part).ok_or_else(|| {
        IgraphError::InvalidArgument("edge count overflow in lexicographic_product".to_string())
    })?;

    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(total_edges);

    // Part 1: g2 edges replicated for each vertex in V1 (same as Cartesian g2 part)
    for eid in 0..e2 {
        #[allow(clippy::cast_possible_truncation)]
        let (u, v) = g2.edge(eid as u32)?;
        for i in 0..n1 {
            edges.push((i * n2 + u, i * n2 + v));
        }
    }

    // Part 2: for each g1 edge (u, v), connect all (u, j) to all (v, l)
    for eid in 0..e1 {
        #[allow(clippy::cast_possible_truncation)]
        let (u, v) = g1.edge(eid as u32)?;
        for j in 0..n2 {
            for l in 0..n2 {
                edges.push((u * n2 + j, v * n2 + l));
            }
        }
    }

    let mut result = Graph::new(n, directed)?;
    result.add_edges(edges)?;
    Ok(result)
}

/// Computes the rooted product of two graphs.
///
/// The result has `|V1| * |V2|` vertices. Vertex `(i, j)` is identified by
/// `i * |V2| + j`. An edge exists between `(i1, j1)` and `(i2, j2)` iff:
/// - `i1 == i2` and `(j1, j2)` is an edge in `g2`, OR
/// - `j1 == j2 == root` and `(i1, i2)` is an edge in `g1`.
///
/// Intuitively, this replaces each vertex of `g1` with a copy of `g2`,
/// connecting the copies through their `root` vertices according to the
/// edges of `g1`.
///
/// The number of edges in the product is `|V1| * |E2| + |E1|`.
///
/// Both graphs must have the same directedness.
///
/// # Arguments
///
/// * `g1` — the first factor graph (whose vertices are "replaced").
/// * `g2` — the second factor graph (the "replacement" graph).
/// * `root` — a vertex in `g2` used as the connection point.
///
/// # Errors
///
/// Returns `InvalidArgument` if the graphs differ in directedness, if the
/// product vertex count overflows `u32`, or if `root >= g2.vcount()`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, rooted_product};
///
/// // P3 with K2 rooted at vertex 0
/// let mut g1 = Graph::with_vertices(3);
/// g1.add_edge(0, 1).unwrap();
/// g1.add_edge(1, 2).unwrap();
///
/// let mut g2 = Graph::with_vertices(2);
/// g2.add_edge(0, 1).unwrap();
///
/// let p = rooted_product(&g1, &g2, 0).unwrap();
/// assert_eq!(p.vcount(), 6); // 3 * 2
/// assert_eq!(p.ecount(), 5); // 3 * 1 + 2
/// ```
pub fn rooted_product(g1: &Graph, g2: &Graph, root: u32) -> IgraphResult<Graph> {
    check_same_directedness(g1, g2, "rooted_product")?;

    let n1 = g1.vcount();
    let n2 = g2.vcount();

    if n2 == 0 || root >= n2 {
        return Err(IgraphError::InvalidArgument(
            "root vertex is not present in the second graph".to_string(),
        ));
    }

    let directed = g1.is_directed();
    let n = product_vertex_count(n1, n2)?;

    if n == 0 {
        return Graph::new(0, directed);
    }

    let e1 = g1.ecount();
    let e2 = g2.ecount();

    // Total edges: |V1| * |E2| + |E1|
    let total_edges = (n1 as usize)
        .checked_mul(e2)
        .and_then(|v| v.checked_add(e1))
        .ok_or_else(|| {
            IgraphError::InvalidArgument("edge count overflow in rooted_product".to_string())
        })?;

    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(total_edges);

    // Edges from g1: connect root copies.
    // Edge (u, v) in g1 becomes ((u, root), (v, root)) in the product.
    for eid_usize in 0..e1 {
        #[allow(clippy::cast_possible_truncation)]
        let eid = eid_usize as u32;
        let (from, to) = g1.edge(eid)?;
        let new_from = from * n2 + root;
        let new_to = to * n2 + root;
        edges.push((new_from, new_to));
    }

    // Edges from g2: for each vertex j in g1, copy all g2 edges.
    // Edge (a, b) in g2 becomes ((j, a), (j, b)) for each j.
    for eid_usize in 0..e2 {
        #[allow(clippy::cast_possible_truncation)]
        let eid = eid_usize as u32;
        let (from, to) = g2.edge(eid)?;
        for j in 0..n1 {
            let new_from = j * n2 + from;
            let new_to = j * n2 + to;
            edges.push((new_from, new_to));
        }
    }

    let mut result = Graph::new(n, directed)?;
    result.add_edges(edges)?;
    Ok(result)
}

/// Computes the modular product of two graphs.
///
/// The result has `|V1| * |V2|` vertices. Vertex `(i, j)` is identified by
/// `i * |V2| + j`. An edge exists between `(i1, j1)` and `(i2, j2)` iff:
/// - `(i1, i2)` is an edge in `g1` AND `(j1, j2)` is an edge in `g2`, OR
/// - `(i1, i2)` is NOT an edge in `g1` AND `(j1, j2)` is NOT an edge in `g2`
///   (with `i1 ≠ i2` and `j1 ≠ j2`).
///
/// Both graphs must be simple (no self-loops, no multi-edges) and have the
/// same directedness.
///
/// Computed as `tensor(g1, g2) ∪ tensor(complement(g1), complement(g2))`.
///
/// # Arguments
///
/// * `g1` — the first factor graph.
/// * `g2` — the second factor graph.
///
/// # Errors
///
/// Returns `InvalidArgument` if:
/// - the graphs differ in directedness,
/// - either graph is not simple,
/// - the product vertex count overflows `u32`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, modular_product};
///
/// // P3 modular P3: modular product of two paths on 3 vertices
/// let mut g1 = Graph::with_vertices(3);
/// g1.add_edge(0, 1).unwrap();
/// g1.add_edge(1, 2).unwrap();
/// let mut g2 = Graph::with_vertices(3);
/// g2.add_edge(0, 1).unwrap();
/// g2.add_edge(1, 2).unwrap();
///
/// let p = modular_product(&g1, &g2).unwrap();
/// assert_eq!(p.vcount(), 9);
/// ```
pub fn modular_product(g1: &Graph, g2: &Graph) -> IgraphResult<Graph> {
    use crate::algorithms::operators::complementer::complementer;
    use crate::algorithms::operators::union::union;
    use crate::algorithms::properties::is_simple::is_simple;

    check_same_directedness(g1, g2, "modular_product")?;

    let simple1 = is_simple(g1)?;
    let simple2 = is_simple(g2)?;
    if !simple1 || !simple2 {
        return Err(IgraphError::InvalidArgument(
            "modular product requires simple graphs as input".to_string(),
        ));
    }

    let n1 = g1.vcount();
    let n2 = g2.vcount();

    if n1 == 0 || n2 == 0 {
        let directed = g1.is_directed();
        return Graph::new(0, directed);
    }

    let g1_compl = complementer(g1, false)?;
    let g2_compl = complementer(g2, false)?;

    let tp_orig = tensor_product(g1, g2)?;
    let tp_compl = tensor_product(&g1_compl, &g2_compl)?;

    union(&tp_orig, &tp_compl)
}

fn check_same_directedness(g1: &Graph, g2: &Graph, op: &str) -> IgraphResult<()> {
    if g1.is_directed() != g2.is_directed() {
        return Err(IgraphError::InvalidArgument(format!(
            "cannot compute {op} of directed and undirected graphs"
        )));
    }
    Ok(())
}

fn product_vertex_count(n1: u32, n2: u32) -> IgraphResult<u32> {
    let count = u64::from(n1) * u64::from(n2);
    u32::try_from(count).map_err(|_| {
        IgraphError::InvalidArgument("product vertex count exceeds u32::MAX".to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Cartesian product tests ---

    #[test]
    fn test_cartesian_k2_k2() {
        let mut g1 = Graph::with_vertices(2);
        g1.add_edge(0, 1).unwrap();
        let mut g2 = Graph::with_vertices(2);
        g2.add_edge(0, 1).unwrap();

        let p = cartesian_product(&g1, &g2).unwrap();
        assert_eq!(p.vcount(), 4);
        // C4: 4 edges
        assert_eq!(p.ecount(), 4);
    }

    #[test]
    fn test_cartesian_k2_k3() {
        let mut g1 = Graph::with_vertices(2);
        g1.add_edge(0, 1).unwrap();

        let mut g2 = Graph::with_vertices(3);
        g2.add_edge(0, 1).unwrap();
        g2.add_edge(1, 2).unwrap();
        g2.add_edge(0, 2).unwrap();

        let p = cartesian_product(&g1, &g2).unwrap();
        assert_eq!(p.vcount(), 6);
        // 2*3 + 3*1 = 9 edges
        assert_eq!(p.ecount(), 9);
    }

    #[test]
    fn test_cartesian_empty_graph() {
        let g1 = Graph::with_vertices(0);
        let g2 = Graph::with_vertices(3);
        let p = cartesian_product(&g1, &g2).unwrap();
        assert_eq!(p.vcount(), 0);
        assert_eq!(p.ecount(), 0);
    }

    #[test]
    fn test_cartesian_isolated_vertices() {
        let g1 = Graph::with_vertices(3);
        let g2 = Graph::with_vertices(4);
        let p = cartesian_product(&g1, &g2).unwrap();
        assert_eq!(p.vcount(), 12);
        assert_eq!(p.ecount(), 0);
    }

    #[test]
    fn test_cartesian_directed() {
        let mut g1 = Graph::new(2, true).unwrap();
        g1.add_edge(0, 1).unwrap();
        let mut g2 = Graph::new(2, true).unwrap();
        g2.add_edge(0, 1).unwrap();

        let p = cartesian_product(&g1, &g2).unwrap();
        assert!(p.is_directed());
        assert_eq!(p.vcount(), 4);
        // 2*1 + 2*1 = 4 directed edges
        assert_eq!(p.ecount(), 4);
    }

    #[test]
    fn test_cartesian_mixed_error() {
        let g1 = Graph::new(2, true).unwrap();
        let g2 = Graph::with_vertices(2);
        assert!(cartesian_product(&g1, &g2).is_err());
    }

    // --- Tensor product tests ---

    #[test]
    fn test_tensor_k2_k2() {
        let mut g1 = Graph::with_vertices(2);
        g1.add_edge(0, 1).unwrap();
        let mut g2 = Graph::with_vertices(2);
        g2.add_edge(0, 1).unwrap();

        let p = tensor_product(&g1, &g2).unwrap();
        assert_eq!(p.vcount(), 4);
        // 1 edge × 1 edge × 2 (undirected) = 2 edges
        assert_eq!(p.ecount(), 2);
    }

    #[test]
    fn test_tensor_path_k2() {
        let mut g1 = Graph::with_vertices(3);
        g1.add_edge(0, 1).unwrap();
        g1.add_edge(1, 2).unwrap();

        let mut g2 = Graph::with_vertices(2);
        g2.add_edge(0, 1).unwrap();

        let p = tensor_product(&g1, &g2).unwrap();
        assert_eq!(p.vcount(), 6);
        // 2 edges × 1 edge × 2 = 4 edges
        assert_eq!(p.ecount(), 4);
    }

    #[test]
    fn test_tensor_directed() {
        let mut g1 = Graph::new(2, true).unwrap();
        g1.add_edge(0, 1).unwrap();
        let mut g2 = Graph::new(2, true).unwrap();
        g2.add_edge(0, 1).unwrap();

        let p = tensor_product(&g1, &g2).unwrap();
        assert!(p.is_directed());
        assert_eq!(p.vcount(), 4);
        // 1 × 1 × 1 (directed) = 1 edge: (0,0)→(1,1)
        assert_eq!(p.ecount(), 1);
        assert_eq!(p.edge(0).unwrap(), (0, 3)); // vertex (0,0)=0, (1,1)=1*2+1=3
    }

    #[test]
    fn test_tensor_no_edges() {
        let g1 = Graph::with_vertices(3);
        let mut g2 = Graph::with_vertices(2);
        g2.add_edge(0, 1).unwrap();

        let p = tensor_product(&g1, &g2).unwrap();
        assert_eq!(p.vcount(), 6);
        assert_eq!(p.ecount(), 0);
    }

    #[test]
    fn test_tensor_empty() {
        let g1 = Graph::with_vertices(0);
        let g2 = Graph::with_vertices(5);
        let p = tensor_product(&g1, &g2).unwrap();
        assert_eq!(p.vcount(), 0);
    }

    // --- Strong product tests ---

    #[test]
    fn test_strong_k2_k2() {
        let mut g1 = Graph::with_vertices(2);
        g1.add_edge(0, 1).unwrap();
        let mut g2 = Graph::with_vertices(2);
        g2.add_edge(0, 1).unwrap();

        let p = strong_product(&g1, &g2).unwrap();
        assert_eq!(p.vcount(), 4);
        // Cartesian: 4 + Tensor: 2 = 6 (= K4)
        assert_eq!(p.ecount(), 6);
    }

    #[test]
    fn test_strong_directed() {
        let mut g1 = Graph::new(2, true).unwrap();
        g1.add_edge(0, 1).unwrap();
        let mut g2 = Graph::new(2, true).unwrap();
        g2.add_edge(0, 1).unwrap();

        let p = strong_product(&g1, &g2).unwrap();
        assert!(p.is_directed());
        assert_eq!(p.vcount(), 4);
        // Cartesian: 4 + Tensor: 1 = 5
        assert_eq!(p.ecount(), 5);
    }

    #[test]
    fn test_strong_one_edgeless() {
        let mut g1 = Graph::with_vertices(2);
        g1.add_edge(0, 1).unwrap();
        let g2 = Graph::with_vertices(3);

        let p = strong_product(&g1, &g2).unwrap();
        assert_eq!(p.vcount(), 6);
        // Cartesian: 0 + 3*1 = 3, Tensor: 0. Total = 3
        assert_eq!(p.ecount(), 3);
    }

    // --- Lexicographic product tests ---

    #[test]
    fn test_lexicographic_k2_isolated() {
        let mut g1 = Graph::with_vertices(2);
        g1.add_edge(0, 1).unwrap();
        let g2 = Graph::with_vertices(3);

        let p = lexicographic_product(&g1, &g2).unwrap();
        assert_eq!(p.vcount(), 6);
        // g2 part: 0 edges. g1 part: 1 edge × 3² = 9.
        assert_eq!(p.ecount(), 9);
    }

    #[test]
    fn test_lexicographic_k2_k2() {
        let mut g1 = Graph::with_vertices(2);
        g1.add_edge(0, 1).unwrap();
        let mut g2 = Graph::with_vertices(2);
        g2.add_edge(0, 1).unwrap();

        let p = lexicographic_product(&g1, &g2).unwrap();
        assert_eq!(p.vcount(), 4);
        // g2 part: 2 * 1 = 2. g1 part: 1 × 2² = 4. Total = 6.
        assert_eq!(p.ecount(), 6);
    }

    #[test]
    fn test_lexicographic_directed() {
        let mut g1 = Graph::new(2, true).unwrap();
        g1.add_edge(0, 1).unwrap();
        let mut g2 = Graph::new(3, true).unwrap();
        g2.add_edge(0, 1).unwrap();

        let p = lexicographic_product(&g1, &g2).unwrap();
        assert!(p.is_directed());
        assert_eq!(p.vcount(), 6);
        // g2 part: 2 * 1 = 2. g1 part: 1 × 3² = 9. Total = 11.
        assert_eq!(p.ecount(), 11);
    }

    #[test]
    fn test_lexicographic_both_edgeless() {
        let g1 = Graph::with_vertices(3);
        let g2 = Graph::with_vertices(4);

        let p = lexicographic_product(&g1, &g2).unwrap();
        assert_eq!(p.vcount(), 12);
        assert_eq!(p.ecount(), 0);
    }

    #[test]
    fn test_lexicographic_not_commutative() {
        let mut g1 = Graph::with_vertices(2);
        g1.add_edge(0, 1).unwrap();
        let g2 = Graph::with_vertices(3);

        let p1 = lexicographic_product(&g1, &g2).unwrap();
        let p2 = lexicographic_product(&g2, &g1).unwrap();
        // g1[g2]: 6 vertices, 0 + 1*3² = 9 edges
        // g2[g1]: 6 vertices, 3*1 + 0 = 3 edges (inner g1 edges replicated)
        assert_eq!(p1.ecount(), 9);
        assert_eq!(p2.ecount(), 3);
        assert_ne!(p1.ecount(), p2.ecount());
    }

    // --- Rooted product tests ---

    #[test]
    fn test_rooted_p3_k2() {
        // P3 (0-1-2) with K2 (0-1) rooted at 0
        let mut g1 = Graph::with_vertices(3);
        g1.add_edge(0, 1).unwrap();
        g1.add_edge(1, 2).unwrap();

        let mut g2 = Graph::with_vertices(2);
        g2.add_edge(0, 1).unwrap();

        let p = rooted_product(&g1, &g2, 0).unwrap();
        assert_eq!(p.vcount(), 6); // 3 * 2
        // |V1|*|E2| + |E1| = 3*1 + 2 = 5
        assert_eq!(p.ecount(), 5);
    }

    #[test]
    fn test_rooted_k3_p3() {
        // K3 (triangle) with P3 (0-1-2) rooted at vertex 1
        let mut g1 = Graph::with_vertices(3);
        g1.add_edge(0, 1).unwrap();
        g1.add_edge(1, 2).unwrap();
        g1.add_edge(0, 2).unwrap();

        let mut g2 = Graph::with_vertices(3);
        g2.add_edge(0, 1).unwrap();
        g2.add_edge(1, 2).unwrap();

        let p = rooted_product(&g1, &g2, 1).unwrap();
        assert_eq!(p.vcount(), 9); // 3 * 3
        // |V1|*|E2| + |E1| = 3*2 + 3 = 9
        assert_eq!(p.ecount(), 9);
    }

    #[test]
    fn test_rooted_single_vertex_g1() {
        // Single vertex * K2 rooted at 0
        let g1 = Graph::with_vertices(1);
        let mut g2 = Graph::with_vertices(2);
        g2.add_edge(0, 1).unwrap();

        let p = rooted_product(&g1, &g2, 0).unwrap();
        assert_eq!(p.vcount(), 2); // 1 * 2
        assert_eq!(p.ecount(), 1); // 1*1 + 0
    }

    #[test]
    fn test_rooted_no_edges_g2() {
        // P3 with 2 isolated vertices rooted at 0
        let mut g1 = Graph::with_vertices(3);
        g1.add_edge(0, 1).unwrap();
        g1.add_edge(1, 2).unwrap();

        let g2 = Graph::with_vertices(2);

        let p = rooted_product(&g1, &g2, 0).unwrap();
        assert_eq!(p.vcount(), 6);
        // 3*0 + 2 = 2
        assert_eq!(p.ecount(), 2);
    }

    #[test]
    fn test_rooted_no_edges_g1() {
        // 3 isolated vertices with K2 rooted at 0
        let g1 = Graph::with_vertices(3);
        let mut g2 = Graph::with_vertices(2);
        g2.add_edge(0, 1).unwrap();

        let p = rooted_product(&g1, &g2, 0).unwrap();
        assert_eq!(p.vcount(), 6);
        // 3*1 + 0 = 3
        assert_eq!(p.ecount(), 3);
    }

    #[test]
    fn test_rooted_directed() {
        let mut g1 = Graph::new(2, true).unwrap();
        g1.add_edge(0, 1).unwrap();

        let mut g2 = Graph::new(2, true).unwrap();
        g2.add_edge(0, 1).unwrap();

        let p = rooted_product(&g1, &g2, 0).unwrap();
        assert!(p.is_directed());
        assert_eq!(p.vcount(), 4);
        // 2*1 + 1 = 3
        assert_eq!(p.ecount(), 3);
    }

    #[test]
    fn test_rooted_invalid_root() {
        let g1 = Graph::with_vertices(2);
        let g2 = Graph::with_vertices(3);

        assert!(rooted_product(&g1, &g2, 3).is_err());
        assert!(rooted_product(&g1, &g2, 5).is_err());
    }

    #[test]
    fn test_rooted_directedness_mismatch() {
        let g1 = Graph::with_vertices(2);
        let g2 = Graph::new(2, true).unwrap();

        assert!(rooted_product(&g1, &g2, 0).is_err());
    }

    #[test]
    fn test_rooted_empty_g2() {
        let g1 = Graph::with_vertices(2);
        let g2 = Graph::with_vertices(0);

        assert!(rooted_product(&g1, &g2, 0).is_err());
    }

    // --- Modular product tests ---

    #[test]
    fn test_modular_k2_k2() {
        let mut g1 = Graph::with_vertices(2);
        g1.add_edge(0, 1).unwrap();
        let mut g2 = Graph::with_vertices(2);
        g2.add_edge(0, 1).unwrap();

        let p = modular_product(&g1, &g2).unwrap();
        assert_eq!(p.vcount(), 4);
        // tensor(K2, K2) has 2 edges: (0,0)-(1,1), (0,1)-(1,0)
        // complement of K2 is edgeless → tensor of complements = 0 edges
        // union = 2 edges
        assert_eq!(p.ecount(), 2);
    }

    #[test]
    fn test_modular_p3_p3() {
        // P3 = 0-1-2
        let mut g1 = Graph::with_vertices(3);
        g1.add_edge(0, 1).unwrap();
        g1.add_edge(1, 2).unwrap();
        let mut g2 = Graph::with_vertices(3);
        g2.add_edge(0, 1).unwrap();
        g2.add_edge(1, 2).unwrap();

        let p = modular_product(&g1, &g2).unwrap();
        assert_eq!(p.vcount(), 9);
        // P3 tensor P3 has 2*2*2 = 8 edges (undirected, each pair generates 2)
        // complement(P3) = single edge (0,2), so tensor of complements
        // has 1*1*2 = 2 edges
        // Union of 8 + 2 = 10 edges
        assert_eq!(p.ecount(), 10);
    }

    #[test]
    fn test_modular_empty_graphs() {
        let g1 = Graph::with_vertices(0);
        let g2 = Graph::with_vertices(3);
        let p = modular_product(&g1, &g2).unwrap();
        assert_eq!(p.vcount(), 0);
        assert_eq!(p.ecount(), 0);
    }

    #[test]
    fn test_modular_edgeless() {
        // Two edgeless graphs: complement is complete, so
        // tensor of complements generates all cross-edges
        let g1 = Graph::with_vertices(3);
        let g2 = Graph::with_vertices(3);

        let p = modular_product(&g1, &g2).unwrap();
        assert_eq!(p.vcount(), 9);
        // tensor(edgeless, edgeless) = 0 edges
        // tensor(K3, K3) = 3*3*2 = 18 edges
        // union = 18 edges
        assert_eq!(p.ecount(), 18);
    }

    #[test]
    fn test_modular_complete_graphs() {
        // K3 modular K3: tensor of originals + tensor of edgeless complements
        let mut g1 = Graph::with_vertices(3);
        for i in 0..3u32 {
            for j in (i + 1)..3 {
                g1.add_edge(i, j).unwrap();
            }
        }
        let mut g2 = Graph::with_vertices(3);
        for i in 0..3u32 {
            for j in (i + 1)..3 {
                g2.add_edge(i, j).unwrap();
            }
        }

        let p = modular_product(&g1, &g2).unwrap();
        assert_eq!(p.vcount(), 9);
        // tensor(K3, K3) = 3*3*2 = 18 edges
        // complement(K3) = edgeless → tensor = 0
        // union = 18 edges
        assert_eq!(p.ecount(), 18);
    }

    #[test]
    fn test_modular_directed() {
        let mut g1 = Graph::new(2, true).unwrap();
        g1.add_edge(0, 1).unwrap();
        let mut g2 = Graph::new(2, true).unwrap();
        g2.add_edge(0, 1).unwrap();

        let p = modular_product(&g1, &g2).unwrap();
        assert!(p.is_directed());
        assert_eq!(p.vcount(), 4);
        // tensor(g1, g2) directed: 1 edge (0,0)→(1,1)
        // complement(g1) = 1→0, complement(g2) = 1→0
        // tensor of complements: 1 edge (1,1)→(0,0)
        // union: 2 edges
        assert_eq!(p.ecount(), 2);
    }

    #[test]
    fn test_modular_not_simple_error() {
        // Multi-edge graph
        let mut g1 = Graph::with_vertices(2);
        g1.add_edge(0, 1).unwrap();
        g1.add_edge(0, 1).unwrap();
        let g2 = Graph::with_vertices(2);

        assert!(modular_product(&g1, &g2).is_err());
    }

    #[test]
    fn test_modular_self_loop_error() {
        let mut g1 = Graph::with_vertices(2);
        g1.add_edge(0, 0).unwrap();
        let g2 = Graph::with_vertices(2);

        assert!(modular_product(&g1, &g2).is_err());
    }

    #[test]
    fn test_modular_mixed_error() {
        let g1 = Graph::new(2, true).unwrap();
        let g2 = Graph::with_vertices(2);
        assert!(modular_product(&g1, &g2).is_err());
    }

    // --- graph_product dispatcher tests ---

    #[test]
    fn test_graph_product_dispatcher() {
        let mut g1 = Graph::with_vertices(2);
        g1.add_edge(0, 1).unwrap();
        let mut g2 = Graph::with_vertices(2);
        g2.add_edge(0, 1).unwrap();

        let p_c = graph_product(&g1, &g2, GraphProductType::Cartesian).unwrap();
        assert_eq!(p_c.ecount(), cartesian_product(&g1, &g2).unwrap().ecount());

        let p_t = graph_product(&g1, &g2, GraphProductType::Tensor).unwrap();
        assert_eq!(p_t.ecount(), tensor_product(&g1, &g2).unwrap().ecount());

        let p_s = graph_product(&g1, &g2, GraphProductType::Strong).unwrap();
        assert_eq!(p_s.ecount(), strong_product(&g1, &g2).unwrap().ecount());

        let p_l = graph_product(&g1, &g2, GraphProductType::Lexicographic).unwrap();
        assert_eq!(
            p_l.ecount(),
            lexicographic_product(&g1, &g2).unwrap().ecount()
        );

        let p_m = graph_product(&g1, &g2, GraphProductType::Modular).unwrap();
        assert_eq!(p_m.ecount(), modular_product(&g1, &g2).unwrap().ecount());
    }

    // --- Overflow tests ---

    #[test]
    fn test_product_overflow() {
        // u32::MAX ≈ 4.3 billion; 70000² > u32::MAX
        let g1 = Graph::with_vertices(70000);
        let g2 = Graph::with_vertices(70000);
        assert!(cartesian_product(&g1, &g2).is_err());
        assert!(tensor_product(&g1, &g2).is_err());
        assert!(strong_product(&g1, &g2).is_err());
        assert!(lexicographic_product(&g1, &g2).is_err());
        assert!(rooted_product(&g1, &g2, 0).is_err());
    }
}

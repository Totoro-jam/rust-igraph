//! `Graph` — pure-Rust port of `igraph_t`.
//!
//! Storage is the **indexed edge list** that upstream igraph uses (see
//! `references/igraph/include/igraph_datatype.h:105-116`):
//!
//! - `from[e]`, `to[e]` — canonical edge list. Edge `e` runs from
//!   `from[e]` to `to[e]`; `|from| == |to| == ecount`.
//! - `oi[i]` — edge ids ordered by `from` (and then `to`).
//! - `ii[i]` — edge ids ordered by `to` (and then `from`).
//! - `os[v]..os[v+1]` — slice of `oi` covering vertex `v`'s out-edges.
//! - `is[v]..is[v+1]` — slice of `ii` covering vertex `v`'s in-edges.
//!
//! For undirected graphs the edge list is canonicalised so `from[e] <= to[e]`
//! (matching upstream igraph's invariant in `type_indexededgelist.c:282-288`).
//! The doubled in/out indexing makes `neighbors()` symmetric for undirected
//! graphs without storing each edge twice.
//!
//! ALGO-CORE-001a (Phase 1, this file): struct + `new`/`with_vertices` +
//! `add_vertices`/`add_edge`/`add_edges` + `vcount`/`ecount`/`is_directed` +
//! `neighbors`/`degree` + `Clone`.
//!
//! Follow-up AWUs:
//! - 001b: `incident`, edge-id helpers.
//! - 001c: `delete_vertices`/`delete_edges`.
//! - 001d: `edge`/`edges`/`get_eid`/`get_eids`/`get_all_eids_between`.
//! - 001e: property cache, `is_same_graph`.
//!
//! Attribute system → ALGO-AT-* (out of scope here).

use super::cache::{
    CachedProperty, PropertyCache, invalidate_after_add_edges, invalidate_after_add_vertices,
};
use super::error::{IgraphError, IgraphResult};

/// Vertex id. The Phase-0 ADR-0007 fixes this to `u32`; `Option<VertexId>`
/// is the idiomatic "no vertex" sentinel (igraph C uses `-1`).
pub type VertexId = u32;

/// Edge id. Same width as [`VertexId`]; an edge id is its position in
/// `from`/`to`.
pub type EdgeId = u32;

/// Iterator over graph edges as `(from, to)` pairs.
pub type EdgeIter<'a> = std::iter::Map<
    std::iter::Zip<std::slice::Iter<'a, VertexId>, std::slice::Iter<'a, VertexId>>,
    fn((&'a VertexId, &'a VertexId)) -> (VertexId, VertexId),
>;

/// Zero-allocation iterator over the neighbors of a vertex.
///
/// For directed graphs, yields out-neighbors in ascending order.
/// For undirected graphs, yields all neighbors in ascending order by
/// merging the out-edge and in-edge sublists on the fly.
///
/// Created by [`Graph::neighbors_iter`].
pub struct NeighborsIter<'a> {
    graph: &'a Graph,
    out_pos: usize,
    out_end: usize,
    in_pos: usize,
    in_end: usize,
    directed: bool,
}

impl Iterator for NeighborsIter<'_> {
    type Item = VertexId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.directed {
            if self.out_pos < self.out_end {
                let eid = self.graph.oi[self.out_pos] as usize;
                self.out_pos += 1;
                Some(self.graph.to[eid])
            } else {
                None
            }
        } else {
            let have_out = self.out_pos < self.out_end;
            let have_in = self.in_pos < self.in_end;
            match (have_out, have_in) {
                (false, false) => None,
                (true, false) => {
                    let eid = self.graph.oi[self.out_pos] as usize;
                    self.out_pos += 1;
                    Some(self.graph.to[eid])
                }
                (false, true) => {
                    let eid = self.graph.ii[self.in_pos] as usize;
                    self.in_pos += 1;
                    Some(self.graph.from[eid])
                }
                (true, true) => {
                    let a = self.graph.to[self.graph.oi[self.out_pos] as usize];
                    let b = self.graph.from[self.graph.ii[self.in_pos] as usize];
                    if a <= b {
                        self.out_pos += 1;
                        Some(a)
                    } else {
                        self.in_pos += 1;
                        Some(b)
                    }
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.out_end - self.out_pos) + (self.in_end - self.in_pos);
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for NeighborsIter<'_> {}

/// Counterpart of `igraph_t` (see `references/igraph/include/igraph_datatype.h`).
///
/// Phase-0 callers (`bfs`, `read_edgelist`, oracle tests) only depended on
/// `with_vertices`, `add_edge`, `add_edges`, `vcount`, `ecount`, `neighbors`,
/// `degree` — those signatures are preserved here, so existing call sites
/// compile unchanged. New for Phase 1: `new` (with `directed` flag),
/// `is_directed`.
#[derive(Debug, Clone, Default)]
pub struct Graph {
    /// Vertex count. Redundant with the highest used id; mirrors `igraph_t::n`.
    n: u32,
    /// Whether the graph is directed.
    directed: bool,
    /// Source endpoints, one per edge.
    from: Vec<VertexId>,
    /// Target endpoints, one per edge.
    to: Vec<VertexId>,
    /// Edge ids in `from`-major order.
    oi: Vec<EdgeId>,
    /// Edge ids in `to`-major order.
    ii: Vec<EdgeId>,
    /// `os[v]..os[v+1]` is the slice of `oi` for vertex `v`'s out-edges.
    /// Length is `n + 1`; `os[0] == 0`, `os[n] == ecount`.
    os: Vec<u32>,
    /// `is[v]..is[v+1]` for incoming. Same shape as `os`.
    is: Vec<u32>,
    /// Boolean property cache. Mirrors `igraph_t::cache`.
    cache: PropertyCache,
}

impl Graph {
    /// Construct an empty graph on `n` vertices.
    ///
    /// Counterpart of `igraph_empty()`; `directed` defaults to `false` if
    /// you use [`Graph::with_vertices`] instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::new(5, true).unwrap();
    /// assert_eq!(g.vcount(), 5);
    /// assert_eq!(g.ecount(), 0);
    /// assert!(g.is_directed());
    /// ```
    pub fn new(n: u32, directed: bool) -> IgraphResult<Self> {
        let mut g = Self {
            n: 0,
            directed,
            from: Vec::new(),
            to: Vec::new(),
            oi: Vec::new(),
            ii: Vec::new(),
            os: vec![0],
            is: vec![0],
            cache: PropertyCache::new(),
        };
        g.add_vertices(n)?;
        Ok(g)
    }

    /// Build a graph from an edge list, inferring the vertex count from
    /// the highest endpoint.
    ///
    /// This is the most ergonomic way to create a small graph. The vertex
    /// count is `max(u, v) + 1` over all `(u, v)` pairs (or 0 if `edges`
    /// is empty and `n_override` is `None`).
    ///
    /// `n_override` can force a minimum vertex count (useful when you want
    /// isolated vertices beyond the edges). Pass `None` to auto-derive.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0, 1), (1, 2), (2, 0)], false, None).unwrap();
    /// assert_eq!(g.vcount(), 3);
    /// assert_eq!(g.ecount(), 3);
    /// assert!(!g.is_directed());
    /// ```
    pub fn from_edges(
        edges: &[(u32, u32)],
        directed: bool,
        n_override: Option<u32>,
    ) -> IgraphResult<Self> {
        let max_id = edges
            .iter()
            .flat_map(|&(u, v)| [u, v])
            .max()
            .map_or(Some(0), |m| m.checked_add(1));
        let auto_n = max_id.ok_or(IgraphError::InvalidArgument(
            "vertex id overflow in from_edges".to_owned(),
        ))?;
        let n = n_override.map_or(auto_n, |ov| ov.max(auto_n));
        let mut g = Self::new(n, directed)?;
        g.add_edges(edges.to_vec())?;
        Ok(g)
    }

    /// Build a graph from weighted edges, returning both the graph and the
    /// weight vector (indexed by edge id).
    ///
    /// Each element of `edges` is `(from, to, weight)`. The resulting weight
    /// vector has length equal to the edge count, with `weights[eid]`
    /// corresponding to the edge added from the `eid`-th tuple.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let (g, weights) = Graph::from_weighted_edges(
    ///     &[(0, 1, 1.5), (1, 2, 2.0), (2, 0, 0.5)],
    ///     false,
    ///     None,
    /// ).unwrap();
    /// assert_eq!(g.vcount(), 3);
    /// assert_eq!(g.ecount(), 3);
    /// assert_eq!(weights, vec![1.5, 2.0, 0.5]);
    /// ```
    pub fn from_weighted_edges(
        edges: &[(u32, u32, f64)],
        directed: bool,
        n_override: Option<u32>,
    ) -> IgraphResult<(Self, Vec<f64>)> {
        let plain: Vec<(u32, u32)> = edges.iter().map(|&(u, v, _)| (u, v)).collect();
        let weights: Vec<f64> = edges.iter().map(|&(_, _, w)| w).collect();
        let g = Self::from_edges(&plain, directed, n_override)?;
        Ok((g, weights))
    }

    /// Parse an undirected graph from an edge-list string.
    ///
    /// Each non-empty, non-comment line should contain two whitespace-separated
    /// vertex ids. Lines starting with `#` are ignored. This is the most
    /// convenient way to construct a graph inline (e.g. in tests or examples).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edge_list_str("0 1\n1 2\n2 0").unwrap();
    /// assert_eq!(g.vcount(), 3);
    /// assert_eq!(g.ecount(), 3);
    /// ```
    pub fn from_edge_list_str(s: &str) -> IgraphResult<Self> {
        use std::io::Cursor;
        crate::algorithms::io::edgelist::read_edgelist(Cursor::new(s))
    }

    /// Construct a graph from an adjacency matrix.
    ///
    /// Counterpart of `igraph_adjacency()`. The matrix should be a
    /// square `n×n` slice-of-slices where `matrix[i][j]` gives the
    /// number of edges from vertex `i` to vertex `j` (or the edge
    /// weight; see below).
    ///
    /// For undirected graphs (`directed = false`), only the upper
    /// triangle is used (including diagonal for self-loops); the lower
    /// triangle is ignored. Each non-zero entry `matrix[i][j]` (with
    /// `i <= j`) creates one edge.
    ///
    /// For directed graphs, every non-zero entry creates one edge.
    ///
    /// Entries are rounded to the nearest integer to determine edge
    /// count. If you need fractional weights, use
    /// [`from_adjacency_matrix_weighted`](Graph::from_adjacency_matrix_weighted).
    ///
    /// # Errors
    ///
    /// Returns an error if the matrix is not square.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let adj = vec![
    ///     vec![0.0, 1.0, 1.0],
    ///     vec![1.0, 0.0, 1.0],
    ///     vec![1.0, 1.0, 0.0],
    /// ];
    /// let g = Graph::from_adjacency_matrix(&adj, false).unwrap();
    /// assert_eq!(g.vcount(), 3);
    /// assert_eq!(g.ecount(), 3); // triangle
    /// ```
    pub fn from_adjacency_matrix(matrix: &[Vec<f64>], directed: bool) -> IgraphResult<Self> {
        let n = matrix.len();
        for row in matrix {
            if row.len() != n {
                return Err(IgraphError::InvalidArgument(format!(
                    "adjacency matrix is not square: got row of length {} for {}×{} matrix",
                    row.len(),
                    n,
                    n
                )));
            }
        }

        let n_u32 = u32::try_from(n)
            .map_err(|_| IgraphError::InvalidArgument("matrix too large for u32".to_owned()))?;
        let mut graph = Self::new(n_u32, directed)?;

        #[allow(clippy::cast_possible_truncation)]
        if directed {
            for (i, row) in matrix.iter().enumerate() {
                for (j, &val) in row.iter().enumerate() {
                    let count = val.round() as i64;
                    for _ in 0..count.max(0) {
                        graph.add_edge(i as u32, j as u32)?;
                    }
                }
            }
        } else {
            for (i, row) in matrix.iter().enumerate() {
                for (j, &val) in row.iter().enumerate().skip(i) {
                    let count = val.round() as i64;
                    for _ in 0..count.max(0) {
                        graph.add_edge(i as u32, j as u32)?;
                    }
                }
            }
        }

        Ok(graph)
    }

    /// Construct a graph from an adjacency matrix, also returning edge weights.
    ///
    /// Like [`from_adjacency_matrix`](Graph::from_adjacency_matrix), but
    /// instead of rounding entries to edge counts, each non-zero entry
    /// creates exactly one edge with the matrix value as its weight.
    ///
    /// Returns the graph and a weight vector aligned with edge indices.
    ///
    /// # Errors
    ///
    /// Returns an error if the matrix is not square.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let adj = vec![
    ///     vec![0.0, 2.5, 0.0],
    ///     vec![2.5, 0.0, 1.0],
    ///     vec![0.0, 1.0, 0.0],
    /// ];
    /// let (g, weights) = Graph::from_adjacency_matrix_weighted(&adj, false).unwrap();
    /// assert_eq!(g.vcount(), 3);
    /// assert_eq!(g.ecount(), 2);
    /// assert!((weights[0] - 2.5).abs() < 1e-10);
    /// assert!((weights[1] - 1.0).abs() < 1e-10);
    /// ```
    pub fn from_adjacency_matrix_weighted(
        matrix: &[Vec<f64>],
        directed: bool,
    ) -> IgraphResult<(Self, Vec<f64>)> {
        let n = matrix.len();
        for row in matrix {
            if row.len() != n {
                return Err(IgraphError::InvalidArgument(format!(
                    "adjacency matrix is not square: got row of length {} for {}×{} matrix",
                    row.len(),
                    n,
                    n
                )));
            }
        }

        let n_u32 = u32::try_from(n)
            .map_err(|_| IgraphError::InvalidArgument("matrix too large for u32".to_owned()))?;
        let mut graph = Self::new(n_u32, directed)?;
        let mut weights = Vec::new();

        #[allow(clippy::cast_possible_truncation)]
        if directed {
            for (i, row) in matrix.iter().enumerate() {
                for (j, &w) in row.iter().enumerate() {
                    if w != 0.0 {
                        graph.add_edge(i as u32, j as u32)?;
                        weights.push(w);
                    }
                }
            }
        } else {
            for (i, row) in matrix.iter().enumerate() {
                for (j, &w) in row.iter().enumerate().skip(i) {
                    if w != 0.0 {
                        graph.add_edge(i as u32, j as u32)?;
                        weights.push(w);
                    }
                }
            }
        }

        Ok((graph, weights))
    }

    /// Construct a graph from an adjacency list.
    ///
    /// `adj_list[v]` contains the neighbors of vertex `v`. The number of
    /// vertices is `adj_list.len()`.
    ///
    /// For undirected graphs (`directed = false`), an edge `(u, v)` should
    /// appear in both `adj_list[u]` and `adj_list[v]`; each pair is
    /// counted once (duplicates are deduplicated by only adding edge `(u, v)`
    /// when `u <= v` or when it appears only in `adj_list[u]`).
    ///
    /// For directed graphs, `adj_list[v]` lists the **out-neighbors** of `v`.
    ///
    /// # Errors
    ///
    /// Returns an error if any neighbor index is out of range.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// // Triangle: 0-1, 1-2, 0-2
    /// let adj = vec![vec![1, 2], vec![0, 2], vec![0, 1]];
    /// let g = Graph::from_adjacency_list(&adj, false).unwrap();
    /// assert_eq!(g.vcount(), 3);
    /// assert_eq!(g.ecount(), 3);
    /// ```
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// // Directed: 0->1, 0->2, 1->2
    /// let adj = vec![vec![1, 2], vec![2], vec![]];
    /// let g = Graph::from_adjacency_list(&adj, true).unwrap();
    /// assert_eq!(g.vcount(), 3);
    /// assert_eq!(g.ecount(), 3);
    /// assert!(g.is_directed());
    /// ```
    pub fn from_adjacency_list(adj_list: &[Vec<u32>], directed: bool) -> IgraphResult<Self> {
        let n = u32::try_from(adj_list.len()).map_err(|_| {
            IgraphError::InvalidArgument("adjacency list too large for u32".to_owned())
        })?;

        let mut graph = Self::new(n, directed)?;

        if directed {
            for (src, neighbors) in adj_list.iter().enumerate() {
                #[allow(clippy::cast_possible_truncation)]
                let src_u32 = src as u32;
                for &tgt in neighbors {
                    if tgt >= n {
                        return Err(IgraphError::VertexOutOfRange { id: tgt, n });
                    }
                    graph.add_edge(src_u32, tgt)?;
                }
            }
        } else {
            for (src, neighbors) in adj_list.iter().enumerate() {
                #[allow(clippy::cast_possible_truncation)]
                let src_u32 = src as u32;
                for &tgt in neighbors {
                    if tgt >= n {
                        return Err(IgraphError::VertexOutOfRange { id: tgt, n });
                    }
                    if src_u32 <= tgt {
                        graph.add_edge(src_u32, tgt)?;
                    }
                }
            }
        }

        Ok(graph)
    }

    /// Construct an empty *undirected* graph on `n` vertices.
    ///
    /// Builds the graph directly (no intermediate `Result`) since an
    /// empty undirected graph with `n` vertices cannot fail to construct.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::with_vertices(4);
    /// assert_eq!(g.vcount(), 4);
    /// assert!(!g.is_directed());
    /// ```
    pub fn with_vertices(n: u32) -> Self {
        let len = n as usize + 1;
        Self {
            n,
            directed: false,
            from: Vec::new(),
            to: Vec::new(),
            oi: Vec::new(),
            ii: Vec::new(),
            os: vec![0; len],
            is: vec![0; len],
            cache: PropertyCache::new(),
        }
    }

    /// Number of vertices. Counterpart of `igraph_vcount()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::with_vertices(10);
    /// assert_eq!(g.vcount(), 10);
    /// ```
    #[must_use]
    pub fn vcount(&self) -> u32 {
        self.n
    }

    /// Number of edges. Counterpart of `igraph_ecount()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let mut g = Graph::with_vertices(3);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(1, 2).unwrap();
    /// assert_eq!(g.ecount(), 2);
    /// ```
    #[must_use]
    pub fn ecount(&self) -> usize {
        self.from.len()
    }

    /// `true` if the graph is directed. Counterpart of `igraph_is_directed()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::new(3, true).unwrap();
    /// assert!(g.is_directed());
    ///
    /// let g2 = Graph::with_vertices(3);
    /// assert!(!g2.is_directed());
    /// ```
    #[must_use]
    pub fn is_directed(&self) -> bool {
        self.directed
    }

    /// Iterator over vertex ids `0..vcount()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::with_vertices(4);
    /// let ids: Vec<u32> = g.vertex_ids().collect();
    /// assert_eq!(ids, vec![0, 1, 2, 3]);
    /// ```
    pub fn vertex_ids(&self) -> impl Iterator<Item = VertexId> {
        0..self.n
    }

    /// Iterator over edge ids `0..ecount()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let mut g = Graph::with_vertices(3);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(1, 2).unwrap();
    /// let ids: Vec<u32> = g.edge_ids().collect();
    /// assert_eq!(ids, vec![0, 1]);
    /// ```
    pub fn edge_ids(&self) -> impl Iterator<Item = u32> {
        let m = u32::try_from(self.from.len()).unwrap_or(u32::MAX);
        0..m
    }

    /// Iterator over all edges as `(from, to)` pairs.
    ///
    /// Yields edges in edge-id order. For undirected graphs, `from <= to`
    /// (canonicalised storage order).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let mut g = Graph::with_vertices(3);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(1, 2).unwrap();
    /// let edges: Vec<(u32, u32)> = g.edges().collect();
    /// assert_eq!(edges, vec![(0, 1), (1, 2)]);
    /// ```
    pub fn edges(&self) -> impl Iterator<Item = (VertexId, VertexId)> + '_ {
        self.from.iter().zip(self.to.iter()).map(|(&u, &v)| (u, v))
    }

    /// Returns an iterator over edges as `(from, to)` pairs in edge-id order.
    ///
    /// This is the named-return counterpart to the `IntoIterator` impl
    /// for `&Graph`, enabling `graph.iter().filter(...)` usage.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let mut g = Graph::with_vertices(3);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(1, 2).unwrap();
    ///
    /// let edges: Vec<_> = g.iter().collect();
    /// assert_eq!(edges, vec![(0, 1), (1, 2)]);
    /// ```
    pub fn iter(&self) -> EdgeIter<'_> {
        self.from.iter().zip(self.to.iter()).map(|(&a, &b)| (a, b))
    }

    /// Check whether an edge exists between `from` and `to`.
    ///
    /// On undirected graphs `(u, v)` and `(v, u)` are equivalent.
    /// Returns `false` for out-of-range vertex ids rather than erroring.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let mut g = Graph::with_vertices(3);
    /// g.add_edge(0, 1).unwrap();
    /// assert!(g.has_edge(0, 1));
    /// assert!(g.has_edge(1, 0)); // undirected
    /// assert!(!g.has_edge(0, 2));
    /// ```
    pub fn has_edge(&self, from: VertexId, to: VertexId) -> bool {
        self.find_eid(from, to).ok().flatten().is_some()
    }

    /// Append `nv` isolated vertices, returning the inclusive id range
    /// `(first, last)` of the new vertices. If `nv == 0` returns
    /// `(self.n, self.n)` and does nothing.
    ///
    /// Counterpart of `igraph_add_vertices()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let mut g = Graph::with_vertices(3);
    /// let (first, last) = g.add_vertices(2).unwrap();
    /// assert_eq!(first, 3);
    /// assert_eq!(last, 4);
    /// assert_eq!(g.vcount(), 5);
    /// ```
    pub fn add_vertices(&mut self, nv: u32) -> IgraphResult<(VertexId, VertexId)> {
        let new_n = self
            .n
            .checked_add(nv)
            .ok_or(IgraphError::Internal("vertex count overflow"))?;
        let first = self.n;
        // os/is grow by `nv` entries, all initialised to ecount.
        let ec = u32::try_from(self.ecount())
            .map_err(|_| IgraphError::Internal("edge count exceeds u32::MAX"))?;
        for _ in 0..nv {
            self.os.push(ec);
            self.is.push(ec);
        }
        self.n = new_n;
        if nv > 0 {
            invalidate_after_add_vertices(&self.cache);
        }
        Ok((first, new_n.saturating_sub(1)))
    }

    /// Add a single edge from `u` to `v`.
    ///
    /// Self-loops and parallel edges are allowed. For undirected graphs the
    /// edge is canonicalised so the stored `from <= to`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let mut g = Graph::with_vertices(3);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(1, 2).unwrap();
    /// assert_eq!(g.ecount(), 2);
    /// ```
    pub fn add_edge(&mut self, u: VertexId, v: VertexId) -> IgraphResult<()> {
        self.add_edges(std::iter::once((u, v)))
    }

    /// Add a sequence of edges. After all edges are appended, the indexes
    /// (`oi` / `ii` / `os` / `is`) are rebuilt in one pass — counterpart of
    /// `igraph_add_edges` (`type_indexededgelist.c:254-367`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let mut g = Graph::with_vertices(4);
    /// g.add_edges(vec![(0, 1), (1, 2), (2, 3)]).unwrap();
    /// assert_eq!(g.ecount(), 3);
    /// ```
    pub fn add_edges<I>(&mut self, edges: I) -> IgraphResult<()>
    where
        I: IntoIterator<Item = (VertexId, VertexId)>,
    {
        let m_before = self.ecount();
        for (u, v) in edges {
            self.check_vertex(u)?;
            self.check_vertex(v)?;
            // Undirected canonicalisation: store the smaller endpoint as `from`.
            if !self.directed && u > v {
                self.from.push(v);
                self.to.push(u);
            } else {
                self.from.push(u);
                self.to.push(v);
            }
        }
        self.rebuild_indexes()?;
        if self.ecount() > m_before {
            invalidate_after_add_edges(&self.cache);
        }
        Ok(())
    }

    /// Out-edge neighbour iterator for vertex `v`.
    ///
    /// For undirected graphs this returns *all* neighbours (since the
    /// indexing tracks both endpoints symmetrically). Order is the upstream
    /// igraph order — edges are visited in `oi` order, then `ii` order, with
    /// duplicates suppressed when the same edge is incident on both.
    ///
    /// Counterpart of `igraph_neighbors(graph, _, vid, IGRAPH_ALL, ...)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let mut g = Graph::with_vertices(4);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(0, 2).unwrap();
    /// g.add_edge(0, 3).unwrap();
    /// let neis = g.neighbors(0).unwrap();
    /// assert_eq!(neis, vec![1, 2, 3]);
    /// ```
    pub fn neighbors(&self, v: VertexId) -> IgraphResult<Vec<VertexId>> {
        self.check_vertex(v)?;
        let v_idx = v as usize;
        if self.directed {
            // Directed: only outgoing neighbours; oi sorted by (from, to)
            // so the out-neighbour list is already sorted ascending.
            let out_range = self.os[v_idx] as usize..self.os[v_idx + 1] as usize;
            let out: Vec<VertexId> = self.oi[out_range]
                .iter()
                .map(|&e| self.to[e as usize])
                .collect();
            Ok(out)
        } else {
            // Undirected: merge the two already-sorted sublists from oi
            // (out-side, ascending in `to`) and ii (in-side, ascending
            // in `from`) into one ascending neighbour list. Matches
            // upstream `igraph_neighbors(_, _, _, IGRAPH_ALL)` and
            // python-igraph's `Graph.neighbors(v)` exactly.
            let out_start = self.os[v_idx] as usize;
            let out_end = self.os[v_idx + 1] as usize;
            let in_start = self.is[v_idx] as usize;
            let in_end = self.is[v_idx + 1] as usize;
            let mut out = Vec::with_capacity((out_end - out_start) + (in_end - in_start));
            let mut out_idx = out_start;
            let mut in_idx = in_start;
            while out_idx < out_end && in_idx < in_end {
                let a = self.to[self.oi[out_idx] as usize];
                let b = self.from[self.ii[in_idx] as usize];
                if a <= b {
                    out.push(a);
                    out_idx += 1;
                } else {
                    out.push(b);
                    in_idx += 1;
                }
            }
            while out_idx < out_end {
                out.push(self.to[self.oi[out_idx] as usize]);
                out_idx += 1;
            }
            while in_idx < in_end {
                out.push(self.from[self.ii[in_idx] as usize]);
                in_idx += 1;
            }
            Ok(out)
        }
    }

    /// Zero-allocation iterator over the neighbors of vertex `v`.
    ///
    /// For directed graphs, yields out-neighbors in ascending order.
    /// For undirected graphs, yields all neighbors in ascending order
    /// (merged from out-edge and in-edge sublists without allocation).
    ///
    /// Prefer this over [`Graph::neighbors`] in hot loops where avoiding
    /// a `Vec` allocation matters.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let mut g = Graph::with_vertices(4);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(0, 2).unwrap();
    /// g.add_edge(0, 3).unwrap();
    /// let neis: Vec<u32> = g.neighbors_iter(0).unwrap().collect();
    /// assert_eq!(neis, vec![1, 2, 3]);
    /// ```
    pub fn neighbors_iter(&self, v: VertexId) -> IgraphResult<NeighborsIter<'_>> {
        self.check_vertex(v)?;
        let v_idx = v as usize;
        let out_pos = self.os[v_idx] as usize;
        let out_end = self.os[v_idx + 1] as usize;
        let (in_pos, in_end) = if self.directed {
            (0, 0)
        } else {
            (self.is[v_idx] as usize, self.is[v_idx + 1] as usize)
        };
        Ok(NeighborsIter {
            graph: self,
            out_pos,
            out_end,
            in_pos,
            in_end,
            directed: self.directed,
        })
    }

    /// Convert the graph to an adjacency list representation.
    ///
    /// Returns a `Vec<Vec<u32>>` where `result[v]` contains the neighbors
    /// of vertex `v`. For directed graphs, returns out-neighbors.
    ///
    /// For undirected graphs, each edge `(u, v)` causes `v` to appear in
    /// `result[u]` and `u` to appear in `result[v]`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let mut g = Graph::with_vertices(3);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(1, 2).unwrap();
    /// let adj = g.to_adjacency_list().unwrap();
    /// assert_eq!(adj[0], vec![1]);
    /// assert_eq!(adj[1], vec![0, 2]);
    /// assert_eq!(adj[2], vec![1]);
    /// ```
    pub fn to_adjacency_list(&self) -> IgraphResult<Vec<Vec<VertexId>>> {
        let n = self.vcount();
        let mut adj = vec![Vec::new(); n as usize];
        for v in 0..n {
            adj[v as usize] = self.neighbors(v)?;
        }
        Ok(adj)
    }

    /// Return the adjacency matrix as a dense `n × n` matrix of `f64`.
    ///
    /// Entry `[i][j]` is the number of edges from vertex `i` to vertex `j`.
    /// For undirected graphs the matrix is symmetric. Self-loops contribute
    /// 1 to `[i][i]` (not 2).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2)], false, None).unwrap();
    /// let m = g.to_adjacency_matrix();
    /// assert_eq!(m[0][1], 1.0);
    /// assert_eq!(m[1][0], 1.0);
    /// assert_eq!(m[0][2], 0.0);
    /// ```
    pub fn to_adjacency_matrix(&self) -> Vec<Vec<f64>> {
        let n = self.n as usize;
        let mut mat = vec![vec![0.0f64; n]; n];
        for eid in 0..self.ecount() {
            let u = self.from[eid] as usize;
            let v = self.to[eid] as usize;
            mat[u][v] += 1.0;
            if !self.directed && u != v {
                mat[v][u] += 1.0;
            }
        }
        mat
    }

    /// Degree of vertex `v` — number of edges incident to it.
    ///
    /// On undirected graphs every edge counts once except a self-loop which
    /// counts twice (matches upstream igraph's `IGRAPH_LOOPS = TWICE` default
    /// at `type_indexededgelist.c:1162`).
    ///
    /// Counterpart of `igraph_degree_1(_, _, _, IGRAPH_ALL, IGRAPH_LOOPS_TWICE)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let mut g = Graph::with_vertices(3);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(0, 2).unwrap();
    /// assert_eq!(g.degree(0).unwrap(), 2);
    /// assert_eq!(g.degree(1).unwrap(), 1);
    /// ```
    pub fn degree(&self, v: VertexId) -> IgraphResult<usize> {
        self.check_vertex(v)?;
        let v_idx = v as usize;
        let out = (self.os[v_idx + 1] - self.os[v_idx]) as usize;
        let in_count = (self.is[v_idx + 1] - self.is[v_idx]) as usize;
        Ok(out + in_count)
    }

    /// Out-degree of vertex `v` (number of outgoing edges).
    ///
    /// For undirected graphs, this equals the total degree.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (0,2), (1,0)], true, None).unwrap();
    /// assert_eq!(g.out_degree(0).unwrap(), 2);
    /// assert_eq!(g.out_degree(1).unwrap(), 1);
    /// ```
    pub fn out_degree(&self, v: VertexId) -> IgraphResult<usize> {
        self.check_vertex(v)?;
        let v_idx = v as usize;
        if self.directed {
            Ok((self.os[v_idx + 1] - self.os[v_idx]) as usize)
        } else {
            let out = (self.os[v_idx + 1] - self.os[v_idx]) as usize;
            let in_count = (self.is[v_idx + 1] - self.is[v_idx]) as usize;
            Ok(out + in_count)
        }
    }

    /// In-degree of vertex `v` (number of incoming edges).
    ///
    /// For undirected graphs, this equals the total degree.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (0,2), (1,0)], true, None).unwrap();
    /// assert_eq!(g.in_degree(0).unwrap(), 1);
    /// assert_eq!(g.in_degree(1).unwrap(), 1);
    /// assert_eq!(g.in_degree(2).unwrap(), 1);
    /// ```
    pub fn in_degree(&self, v: VertexId) -> IgraphResult<usize> {
        self.check_vertex(v)?;
        let v_idx = v as usize;
        if self.directed {
            Ok((self.is[v_idx + 1] - self.is[v_idx]) as usize)
        } else {
            let out = (self.os[v_idx + 1] - self.os[v_idx]) as usize;
            let in_count = (self.is[v_idx + 1] - self.is[v_idx]) as usize;
            Ok(out + in_count)
        }
    }

    /// Maximum degree across all vertices (total degree for undirected,
    /// out-degree for directed). Returns 0 for empty graphs.
    ///
    /// Counterpart of `igraph_maxdegree()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let mut g = Graph::with_vertices(4);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(0, 2).unwrap();
    /// g.add_edge(0, 3).unwrap();
    /// assert_eq!(g.max_degree(), 3);
    /// ```
    pub fn max_degree(&self) -> usize {
        let n = self.vcount();
        if n == 0 {
            return 0;
        }
        (0..n)
            .map(|v| {
                let v_idx = v as usize;
                let out = (self.os[v_idx + 1] - self.os[v_idx]) as usize;
                let inc = (self.is[v_idx + 1] - self.is[v_idx]) as usize;
                if self.directed { out } else { out + inc }
            })
            .max()
            .unwrap_or(0)
    }

    /// Minimum degree across all vertices (total degree for undirected,
    /// out-degree for directed). Returns 0 for empty graphs.
    ///
    /// Counterpart of `igraph_mindegree()` (custom extension).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let mut g = Graph::with_vertices(4);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(0, 2).unwrap();
    /// // vertex 3 has degree 0
    /// assert_eq!(g.min_degree(), 0);
    /// ```
    pub fn min_degree(&self) -> usize {
        let n = self.vcount();
        if n == 0 {
            return 0;
        }
        (0..n)
            .map(|v| {
                let v_idx = v as usize;
                let out = (self.os[v_idx + 1] - self.os[v_idx]) as usize;
                let inc = (self.is[v_idx + 1] - self.is[v_idx]) as usize;
                if self.directed { out } else { out + inc }
            })
            .min()
            .unwrap_or(0)
    }

    // ---------------------------------------------------------------
    // ALGO-CORE-001b: edge-id helpers + incident edges.
    // ---------------------------------------------------------------

    /// Source endpoint of edge `eid`. Counterpart of `IGRAPH_FROM`
    /// (`igraph_interface.h:115`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let mut g = Graph::with_vertices(3);
    /// g.add_edge(0, 2).unwrap();
    /// assert_eq!(g.edge_source(0).unwrap(), 0);
    /// ```
    pub fn edge_source(&self, eid: EdgeId) -> IgraphResult<VertexId> {
        self.check_edge(eid)?;
        Ok(self.from[eid as usize])
    }

    /// Target endpoint of edge `eid`. Counterpart of `IGRAPH_TO`
    /// (`igraph_interface.h:128`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let mut g = Graph::with_vertices(3);
    /// g.add_edge(0, 2).unwrap();
    /// assert_eq!(g.edge_target(0).unwrap(), 2);
    /// ```
    pub fn edge_target(&self, eid: EdgeId) -> IgraphResult<VertexId> {
        self.check_edge(eid)?;
        Ok(self.to[eid as usize])
    }

    /// Both endpoints of edge `eid`, ordered as `(from, to)`. Counterpart
    /// of `igraph_edge` (`igraph_interface.h:71`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let mut g = Graph::with_vertices(3);
    /// g.add_edge(0, 1).unwrap();
    /// let (from, to) = g.edge(0).unwrap();
    /// assert_eq!(from, 0);
    /// assert_eq!(to, 1);
    /// ```
    pub fn edge(&self, eid: EdgeId) -> IgraphResult<(VertexId, VertexId)> {
        self.check_edge(eid)?;
        let i = eid as usize;
        Ok((self.from[i], self.to[i]))
    }

    /// The other endpoint of `eid` given one endpoint `vid`. Counterpart
    /// of `IGRAPH_OTHER` (`igraph_interface.h:145`). Errors if `vid` is
    /// not actually an endpoint of `eid`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let mut g = Graph::with_vertices(3);
    /// g.add_edge(0, 2).unwrap();
    /// assert_eq!(g.edge_other(0, 0).unwrap(), 2);
    /// assert_eq!(g.edge_other(0, 2).unwrap(), 0);
    /// ```
    pub fn edge_other(&self, eid: EdgeId, vid: VertexId) -> IgraphResult<VertexId> {
        let (u, v) = self.edge(eid)?;
        if vid == u {
            Ok(v)
        } else if vid == v {
            Ok(u)
        } else {
            Err(IgraphError::InvalidArgument(format!(
                "vertex {vid} is not an endpoint of edge {eid} ({u}, {v})"
            )))
        }
    }

    /// Edge ids incident to vertex `v`, in the same iteration order as
    /// [`Graph::neighbors`].
    ///
    /// For undirected graphs returns the union of out-side (`oi`) and
    /// in-side (`ii`) edges — every edge incident to `v` once, except
    /// self-loops which appear twice (matching `igraph_neighbors` /
    /// `igraph_degree`'s `IGRAPH_LOOPS_TWICE` default at
    /// `type_indexededgelist.c:1162`).
    ///
    /// For directed graphs returns out-edges only, mirroring this AWU's
    /// `neighbors()` choice. (The full mode-aware variant lands later
    /// alongside `igraph_neighbors(mode = IN/OUT/ALL)`.)
    ///
    /// Counterpart of `igraph_incident(_, _, v, IGRAPH_ALL, IGRAPH_LOOPS_TWICE)`
    /// for undirected; `IGRAPH_OUT` mode for directed.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let mut g = Graph::with_vertices(3);
    /// g.add_edge(0, 1).unwrap(); // edge 0
    /// g.add_edge(0, 2).unwrap(); // edge 1
    /// let inc = g.incident(0).unwrap();
    /// assert_eq!(inc.len(), 2);
    /// ```
    pub fn incident(&self, v: VertexId) -> IgraphResult<Vec<EdgeId>> {
        self.check_vertex(v)?;
        let v_idx = v as usize;
        let out_range = self.os[v_idx] as usize..self.os[v_idx + 1] as usize;
        if self.directed {
            Ok(self.oi[out_range].to_vec())
        } else {
            let in_range = self.is[v_idx] as usize..self.is[v_idx + 1] as usize;
            let mut out = Vec::with_capacity(out_range.len() + in_range.len());
            out.extend_from_slice(&self.oi[out_range]);
            out.extend_from_slice(&self.ii[in_range]);
            Ok(out)
        }
    }

    /// Companion to [`incident`](Self::incident): returns *only* the
    /// edges incoming to `v` for directed graphs. For undirected
    /// graphs the result is identical to `incident` (every edge is
    /// bidirectional).
    ///
    /// Counterpart of `igraph_incident(_, _, v, IGRAPH_IN, IGRAPH_LOOPS_TWICE)`.
    pub(crate) fn incident_in(&self, v: VertexId) -> IgraphResult<Vec<EdgeId>> {
        self.check_vertex(v)?;
        let v_idx = v as usize;
        if self.directed {
            let in_range = self.is[v_idx] as usize..self.is[v_idx + 1] as usize;
            Ok(self.ii[in_range].to_vec())
        } else {
            self.incident(v)
        }
    }

    /// Edge id between `from` and `to`, if any.
    ///
    /// On undirected graphs `(u, v)` and `(v, u)` are equivalent.
    /// On directed graphs the search follows the edge direction
    /// `from -> to`. Returns [`crate::IgraphError::InvalidArgument`]
    /// when no such edge exists; for the "no error, return None" variant
    /// use [`Self::find_eid`].
    ///
    /// Counterpart of
    /// `igraph_get_eid(_, _, from, to, /*directed=*/true, /*error=*/true)`
    /// from `references/igraph/src/graph/type_indexededgelist.c:1522-1555`.
    /// Phase-1 minimal slice: linear scan across the from-bucket; the
    /// upstream binary-search optimisation lands in a perf pass.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let mut g = Graph::with_vertices(3);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(1, 2).unwrap();
    /// assert_eq!(g.get_eid(0, 1).unwrap(), 0);
    /// assert_eq!(g.get_eid(1, 2).unwrap(), 1);
    /// assert!(g.get_eid(0, 2).is_err());
    /// ```
    pub fn get_eid(&self, from: VertexId, to: VertexId) -> IgraphResult<EdgeId> {
        self.find_eid(from, to)?
            .ok_or_else(|| IgraphError::InvalidArgument(format!("no edge between {from} and {to}")))
    }

    /// Edge id between `from` and `to`, or `None` if not connected.
    ///
    /// Same semantics as [`Self::get_eid`] but no-error variant
    /// matching upstream's `error=false` mode. When parallel edges
    /// exist, returns the lowest edge id (matching upstream's
    /// "always returns the same edge ID" guarantee).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let mut g = Graph::with_vertices(3);
    /// g.add_edge(0, 1).unwrap();
    /// assert_eq!(g.find_eid(0, 1).unwrap(), Some(0));
    /// assert_eq!(g.find_eid(0, 2).unwrap(), None);
    /// ```
    pub fn find_eid(&self, from: VertexId, to: VertexId) -> IgraphResult<Option<EdgeId>> {
        self.check_vertex(from)?;
        self.check_vertex(to)?;
        if self.directed {
            // Search out-bucket of `from` for `to[e] == to`.
            let range = self.os[from as usize] as usize..self.os[from as usize + 1] as usize;
            for &e in &self.oi[range] {
                if self.to[e as usize] == to {
                    return Ok(Some(e));
                }
            }
            Ok(None)
        } else {
            // Undirected: edges canonicalised so `from[e] <= to[e]`.
            // Search the bucket of the smaller endpoint for the larger.
            let (lo, hi) = if from <= to { (from, to) } else { (to, from) };
            let range = self.os[lo as usize] as usize..self.os[lo as usize + 1] as usize;
            for &e in &self.oi[range] {
                if self.to[e as usize] == hi {
                    return Ok(Some(e));
                }
            }
            Ok(None)
        }
    }

    /// All edge ids between `from` and `to`, including parallel edges
    /// and (for self-loops) the loop edge once.
    ///
    /// Counterpart of
    /// `igraph_get_all_eids_between()` from
    /// `references/igraph/src/graph/type_indexededgelist.c:~1700`.
    /// On undirected graphs `(u, v)` and `(v, u)` are equivalent. The
    /// returned vector is sorted ascending by edge id.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let mut g = Graph::with_vertices(2);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(0, 1).unwrap(); // parallel edge
    /// let eids = g.get_all_eids_between(0, 1).unwrap();
    /// assert_eq!(eids, vec![0, 1]);
    /// ```
    pub fn get_all_eids_between(&self, from: VertexId, to: VertexId) -> IgraphResult<Vec<EdgeId>> {
        self.check_vertex(from)?;
        self.check_vertex(to)?;
        let mut out = Vec::new();
        if self.directed {
            let range = self.os[from as usize] as usize..self.os[from as usize + 1] as usize;
            for &e in &self.oi[range] {
                if self.to[e as usize] == to {
                    out.push(e);
                }
            }
        } else {
            let (lo, hi) = if from <= to { (from, to) } else { (to, from) };
            let range = self.os[lo as usize] as usize..self.os[lo as usize + 1] as usize;
            for &e in &self.oi[range] {
                if self.to[e as usize] == hi {
                    out.push(e);
                }
            }
        }
        out.sort_unstable();
        Ok(out)
    }

    /// Out-neighbours of `v` (always — directed or undirected). Each
    /// edge contributes one entry, in `oi[os[v]..os[v+1]]` order
    /// (lex by `(from, to)`). Self-loops appear once.
    ///
    /// Internal helper used by direction-aware algorithms (e.g.
    /// strongly connected components). The full mode-aware public
    /// surface ships with the next `igraph_neighbors` AWU.
    pub(crate) fn out_neighbors_vec(&self, v: VertexId) -> IgraphResult<Vec<VertexId>> {
        self.check_vertex(v)?;
        let v_idx = v as usize;
        let range = self.os[v_idx] as usize..self.os[v_idx + 1] as usize;
        Ok(self.oi[range]
            .iter()
            .map(|&e| self.to[e as usize])
            .collect())
    }

    /// In-neighbours of `v` (always — directed or undirected). Each
    /// edge contributes one entry, in `ii[is[v]..is[v+1]]` order
    /// (lex by `(to, from)`). Self-loops appear once.
    ///
    /// Companion to [`out_neighbors_vec`](Self::out_neighbors_vec); see
    /// its doc for context on visibility.
    pub(crate) fn in_neighbors_vec(&self, v: VertexId) -> IgraphResult<Vec<VertexId>> {
        self.check_vertex(v)?;
        let v_idx = v as usize;
        let range = self.is[v_idx] as usize..self.is[v_idx + 1] as usize;
        Ok(self.ii[range]
            .iter()
            .map(|&e| self.from[e as usize])
            .collect())
    }

    // ---------------------------------------------------------------
    // ALGO-CORE-001c: delete_edges + delete_vertices + delete_vertices_map.
    // ---------------------------------------------------------------

    /// Remove the given edges from the graph.
    ///
    /// `edges` may contain the same id more than once — the second and
    /// later occurrences are no-ops. Remaining edges keep their
    /// pairwise relative order but are renumbered so edge ids stay
    /// contiguous starting at 0. Returns
    /// [`IgraphError::EdgeOutOfRange`] if any id is `>= ecount()`; on
    /// error the graph is left unchanged.
    ///
    /// Counterpart of `igraph_delete_edges`
    /// (`references/igraph/src/graph/type_indexededgelist.c:500`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let mut g = Graph::with_vertices(3);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(1, 2).unwrap();
    /// g.add_edge(0, 2).unwrap();
    /// g.delete_edges(&[1]).unwrap(); // remove edge 1-2
    /// assert_eq!(g.ecount(), 2);
    /// ```
    pub fn delete_edges(&mut self, edges: &[EdgeId]) -> IgraphResult<()> {
        let m = self.ecount();
        let m_u32 = u32::try_from(m).unwrap_or(u32::MAX);

        // Validate up front so a bad id leaves graph state untouched.
        for &eid in edges {
            if (eid as usize) >= m {
                return Err(IgraphError::EdgeOutOfRange { id: eid, m: m_u32 });
            }
        }
        if edges.is_empty() {
            return Ok(());
        }

        let mut remove = vec![false; m];
        for &eid in edges {
            remove[eid as usize] = true;
        }

        let mut new_from: Vec<VertexId> = Vec::with_capacity(m);
        let mut new_to: Vec<VertexId> = Vec::with_capacity(m);
        for (e, &is_removed) in remove.iter().enumerate() {
            if !is_removed {
                new_from.push(self.from[e]);
                new_to.push(self.to[e]);
            }
        }
        self.from = new_from;
        self.to = new_to;
        self.rebuild_indexes()?;
        self.cache.invalidate_all();
        Ok(())
    }

    /// Remove the given vertices and all their incident edges.
    ///
    /// `vertices` may repeat ids freely. Surviving vertices get
    /// renumbered so the new id space is `0..new_vcount` in their
    /// previous relative order. Returns
    /// [`IgraphError::VertexOutOfRange`] if any id is `>= vcount()`;
    /// on error the graph is left unchanged.
    ///
    /// Counterpart of `igraph_delete_vertices`
    /// (`references/igraph/src/graph/type_indexededgelist.c:540`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let mut g = Graph::with_vertices(4);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(1, 2).unwrap();
    /// g.add_edge(2, 3).unwrap();
    /// g.delete_vertices(&[1]).unwrap();
    /// assert_eq!(g.vcount(), 3);
    /// assert_eq!(g.ecount(), 1); // only edge 2-3 survives (renumbered)
    /// ```
    pub fn delete_vertices(&mut self, vertices: &[VertexId]) -> IgraphResult<()> {
        self.delete_vertices_map(vertices).map(|_| ())
    }

    /// Like [`delete_vertices`](Self::delete_vertices), but also returns
    /// the old↔new vertex id mappings.
    ///
    /// Returns `(map, invmap)` where:
    /// - `map[old_id] == Some(new_id)` if the vertex was retained, else
    ///   `None`. Length is the *original* vertex count.
    /// - `invmap[new_id] == old_id`. Length is the *new* vertex count.
    ///
    /// Counterpart of `igraph_delete_vertices_map`
    /// (`references/igraph/src/graph/type_indexededgelist.c:645`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let mut g = Graph::with_vertices(4);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(2, 3).unwrap();
    /// let (map, invmap) = g.delete_vertices_map(&[1, 2]).unwrap();
    /// assert_eq!(g.vcount(), 2);
    /// assert_eq!(map, vec![Some(0), None, None, Some(1)]);
    /// assert_eq!(invmap, vec![0, 3]);
    /// ```
    pub fn delete_vertices_map(
        &mut self,
        vertices: &[VertexId],
    ) -> IgraphResult<(Vec<Option<VertexId>>, Vec<VertexId>)> {
        let n_u32 = self.n;
        let n = n_u32 as usize;

        // Validate first.
        for &vid in vertices {
            if vid >= n_u32 {
                return Err(IgraphError::VertexOutOfRange { id: vid, n: n_u32 });
            }
        }

        let mut remove = vec![false; n];
        for &vid in vertices {
            remove[vid as usize] = true;
        }

        // Build map (old → new) and invmap (new → old).
        let mut map: Vec<Option<VertexId>> = vec![None; n];
        let mut invmap: Vec<VertexId> = Vec::new();
        let mut next_new: u32 = 0;
        for (i, &is_removed) in remove.iter().enumerate() {
            if !is_removed {
                let i_u32 = u32::try_from(i)
                    .map_err(|_| IgraphError::Internal("vertex index exceeds u32::MAX"))?;
                map[i] = Some(next_new);
                invmap.push(i_u32);
                next_new = next_new
                    .checked_add(1)
                    .ok_or(IgraphError::Internal("new vertex count overflow"))?;
            }
        }

        // Filter edges: keep only those with both endpoints retained,
        // renumber endpoints via `map`.
        let m = self.ecount();
        let mut new_from: Vec<VertexId> = Vec::with_capacity(m);
        let mut new_to: Vec<VertexId> = Vec::with_capacity(m);
        for (u, v) in self.from.iter().zip(self.to.iter()) {
            if let (Some(nu), Some(nv)) = (map[*u as usize], map[*v as usize]) {
                new_from.push(nu);
                new_to.push(nv);
            }
        }

        self.n = next_new;
        self.from = new_from;
        self.to = new_to;
        self.rebuild_indexes()?;
        self.cache.invalidate_all();

        Ok((map, invmap))
    }

    /// Look up a cached boolean property without computing it.
    ///
    /// Returns `None` if the property has not been cached yet. Pair with
    /// [`Self::cache_set`] in compute functions:
    ///
    /// ```ignore
    /// if let Some(v) = g.cache_get(CachedProperty::IsDag) { return v; }
    /// let v = compute_is_dag(g);
    /// g.cache_set(CachedProperty::IsDag, v);
    /// v
    /// ```
    ///
    /// Counterpart of `igraph_i_property_cache_has` + `_get_bool` from
    /// `references/igraph/src/graph/caching.c`.
    #[must_use]
    pub fn cache_get(&self, prop: CachedProperty) -> Option<bool> {
        self.cache.get(prop)
    }

    /// Store the value of a cached boolean property.
    ///
    /// Takes `&self` (interior mutability via `Cell`) — populating the
    /// cache from a compute function is **not** considered a mutation of
    /// the graph, matching igraph C semantics where compute helpers take
    /// `const igraph_t *` and still write to the cache.
    ///
    /// Counterpart of `igraph_i_property_cache_set_bool`.
    pub fn cache_set(&self, prop: CachedProperty, value: bool) {
        self.cache.set(prop, value);
    }

    /// Drop the cached value of a single property (no-op if not cached).
    ///
    /// Use this if you change the graph via a private path that doesn't
    /// go through `add_edges` / `delete_*`.
    ///
    /// Counterpart of `igraph_i_property_cache_invalidate`.
    pub fn cache_invalidate(&self, prop: CachedProperty) {
        self.cache.invalidate(prop);
    }

    /// Drop every cached boolean property.
    ///
    /// Counterpart of `igraph_i_property_cache_invalidate_all`.
    pub fn cache_invalidate_all(&self) {
        self.cache.invalidate_all();
    }

    fn check_vertex(&self, v: VertexId) -> IgraphResult<()> {
        if v >= self.n {
            return Err(IgraphError::VertexOutOfRange { id: v, n: self.n });
        }
        Ok(())
    }

    fn check_edge(&self, eid: EdgeId) -> IgraphResult<()> {
        let m = self.ecount();
        let m_u32 = u32::try_from(m).unwrap_or(u32::MAX);
        if (eid as usize) >= m {
            return Err(IgraphError::EdgeOutOfRange { id: eid, m: m_u32 });
        }
        Ok(())
    }

    /// Recompute `oi`, `ii`, `os`, `is` from `from`/`to`. Called after
    /// any structural change.
    ///
    /// Each side does a stable lexicographic sort: `oi` orders edges by
    /// `(from[e], to[e])`, `ii` by `(to[e], from[e])`. Time complexity
    /// is `O(|V| + |E| log |E|)` (Rust stable sort) — same asymptotic
    /// as upstream's `igraph_vector_int_pair_order`.
    ///
    /// The within-bucket secondary sort matches upstream igraph; without
    /// it, `neighbors(v)` for an unsorted-edge-input graph diverges from
    /// `python-igraph`'s output and breaks DFS order parity. (Counted
    /// for an oracle-test failure during ALGO-TR-002 — see
    /// `tests/oracle.rs::dfs_small_synthetic_matches_python_igraph`.)
    ///
    /// Counterpart of `igraph_i_create_start_vectors` + the
    /// `igraph_vector_int_pair_order` calls in
    /// `type_indexededgelist.c:309-336`.
    fn rebuild_indexes(&mut self) -> IgraphResult<()> {
        let m = self.ecount();
        let n = self.n as usize;

        // Build (primary_key, secondary_key, edge_id) tuples for each
        // side, sort them lexicographically, then extract edge ids and
        // the offset array.

        // ---- Out-side: sort by (from, to). ----
        let mut tuples: Vec<(VertexId, VertexId, u32)> = (0..m)
            .map(|e| {
                Ok::<_, IgraphError>((
                    self.from[e],
                    self.to[e],
                    u32::try_from(e)
                        .map_err(|_| IgraphError::Internal("edge id exceeds u32::MAX"))?,
                ))
            })
            .collect::<Result<_, _>>()?;
        tuples.sort_unstable_by_key(|a| (a.0, a.1));
        self.oi = tuples.iter().map(|t| t.2).collect();
        // os[v] = number of entries with primary_key < v.
        self.os = vec![0u32; n + 1];
        for &(u, _, _) in &tuples {
            self.os[u as usize + 1] = self.os[u as usize + 1]
                .checked_add(1)
                .ok_or(IgraphError::Internal("degree overflow in rebuild_indexes"))?;
        }
        for i in 1..=n {
            self.os[i] = self.os[i]
                .checked_add(self.os[i - 1])
                .ok_or(IgraphError::Internal("offset overflow in rebuild_indexes"))?;
        }

        // ---- In-side: sort by (to, from). ----
        let mut tuples: Vec<(VertexId, VertexId, u32)> = (0..m)
            .map(|e| {
                Ok::<_, IgraphError>((
                    self.to[e],
                    self.from[e],
                    u32::try_from(e)
                        .map_err(|_| IgraphError::Internal("edge id exceeds u32::MAX"))?,
                ))
            })
            .collect::<Result<_, _>>()?;
        tuples.sort_unstable_by_key(|a| (a.0, a.1));
        self.ii = tuples.iter().map(|t| t.2).collect();
        self.is = vec![0u32; n + 1];
        for &(v, _, _) in &tuples {
            self.is[v as usize + 1] = self.is[v as usize + 1]
                .checked_add(1)
                .ok_or(IgraphError::Internal("degree overflow in rebuild_indexes"))?;
        }
        for i in 1..=n {
            self.is[i] = self.is[i]
                .checked_add(self.is[i - 1])
                .ok_or(IgraphError::Internal("offset overflow in rebuild_indexes"))?;
        }

        Ok(())
    }
}

// — Convenience methods delegating to free-function algorithms —

impl Graph {
    /// Compute the density of this graph.
    ///
    /// Density is the ratio of actual edges to possible edges.
    /// Returns `None` for graphs with fewer than 2 vertices.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,0)], false, None).unwrap();
    /// let d = g.density().unwrap().unwrap();
    /// assert!((d - 1.0).abs() < 1e-10); // K_3 is fully connected
    /// ```
    pub fn density(&self) -> IgraphResult<Option<f64>> {
        crate::algorithms::properties::basic::density(self)
    }

    /// Check whether the graph is connected.
    ///
    /// For directed graphs this checks weak connectivity by default.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2)], false, None).unwrap();
    /// assert!(g.is_connected().unwrap());
    /// ```
    pub fn is_connected(&self) -> IgraphResult<bool> {
        crate::algorithms::connectivity::is_connected::is_connected(
            self,
            crate::algorithms::connectivity::is_connected::ConnectednessMode::Weak,
        )
    }

    /// Check whether the graph is simple (no self-loops, no multi-edges).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2)], false, None).unwrap();
    /// assert!(g.is_simple().unwrap());
    /// ```
    pub fn is_simple(&self) -> IgraphResult<bool> {
        crate::algorithms::properties::is_simple::is_simple(self)
    }

    /// Compute connected components.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let mut g = Graph::new(4, false).unwrap();
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(2, 3).unwrap();
    /// let cc = g.connected_components().unwrap();
    /// assert_eq!(cc.count, 2);
    /// ```
    pub fn connected_components(
        &self,
    ) -> IgraphResult<crate::algorithms::connectivity::components::ConnectedComponents> {
        crate::algorithms::connectivity::components::connected_components(self)
    }

    /// Compute `PageRank` centrality for all vertices.
    ///
    /// Uses the default damping factor (0.85).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,0)], false, None).unwrap();
    /// let pr = g.pagerank().unwrap();
    /// assert_eq!(pr.len(), 3);
    /// ```
    pub fn pagerank(&self) -> IgraphResult<Vec<f64>> {
        crate::algorithms::properties::pagerank::pagerank(self)
    }

    /// Compute betweenness centrality for all vertices.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,3)], false, None).unwrap();
    /// let bc = g.betweenness().unwrap();
    /// // Middle vertices have higher betweenness
    /// assert!(bc[1] > bc[0]);
    /// ```
    pub fn betweenness(&self) -> IgraphResult<Vec<f64>> {
        crate::algorithms::properties::betweenness::betweenness(self)
    }

    /// Compute closeness centrality for all vertices.
    ///
    /// For each vertex, closeness is the reciprocal of the average shortest
    /// path distance to all reachable vertices. Returns `None` for isolated
    /// vertices.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,3)], false, None).unwrap();
    /// let cl = g.closeness().unwrap();
    /// assert_eq!(cl.len(), 4);
    /// // Middle vertices have higher closeness
    /// assert!(cl[1].unwrap() > cl[0].unwrap());
    /// ```
    pub fn closeness(&self) -> IgraphResult<Vec<Option<f64>>> {
        crate::algorithms::properties::closeness::closeness(self)
    }

    /// Compute eigenvector centrality for all vertices.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,0)], false, None).unwrap();
    /// let ec = g.eigenvector_centrality().unwrap();
    /// assert_eq!(ec.len(), 3);
    /// ```
    pub fn eigenvector_centrality(&self) -> IgraphResult<Vec<f64>> {
        crate::algorithms::properties::eigenvector::eigenvector_centrality(self)
    }

    /// Compute per-vertex local clustering coefficients.
    ///
    /// Returns the fraction of actual edges between each vertex's neighbours
    /// out of all possible edges. Vertices with fewer than 2 neighbours
    /// return `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// // A triangle: all vertices have clustering coefficient 1.0
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,0)], false, None).unwrap();
    /// let cc = g.clustering_coefficients().unwrap();
    /// assert!((cc[0].unwrap() - 1.0).abs() < 1e-10);
    /// ```
    pub fn clustering_coefficients(&self) -> IgraphResult<Vec<Option<f64>>> {
        crate::algorithms::properties::triangles::transitivity_local_undirected(self)
    }

    /// Compute the complement graph.
    ///
    /// The complement has the same vertices but edges wherever the original
    /// does not (excluding self-loops by default).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1)], false, Some(3)).unwrap();
    /// let c = g.complement().unwrap();
    /// // K_3 has 3 edges; original has 1; complement has 2
    /// assert_eq!(c.ecount(), 2);
    /// ```
    pub fn complement(&self) -> IgraphResult<Graph> {
        crate::algorithms::operators::complementer::complementer(self, false)
    }

    /// Construct the line graph L(G).
    ///
    /// The line graph has one vertex per edge of this graph. Two vertices
    /// in L(G) are adjacent iff the corresponding edges share an endpoint.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let mut g = Graph::with_vertices(3);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(1, 2).unwrap();
    /// g.add_edge(2, 0).unwrap();
    /// let lg = g.line_graph().unwrap();
    /// assert_eq!(lg.vcount(), 3);
    /// assert_eq!(lg.ecount(), 3);
    /// ```
    pub fn line_graph(&self) -> IgraphResult<Graph> {
        crate::algorithms::operators::line_graph::line_graph(self)
    }

    /// Detect communities using the Louvain algorithm.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(
    ///     &[(0,1), (0,2), (1,2), (3,4), (3,5), (4,5), (2,3)],
    ///     false, None,
    /// ).unwrap();
    /// let result = g.louvain().unwrap();
    /// assert!(result.modularity > 0.0);
    /// ```
    pub fn louvain(&self) -> IgraphResult<crate::algorithms::community::louvain::LouvainResult> {
        crate::algorithms::community::louvain::louvain(self)
    }

    /// Detect communities using the Leiden algorithm.
    ///
    /// Leiden improves upon Louvain by guaranteeing well-connected communities
    /// and avoiding the "poorly connected community" pathology.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(
    ///     &[(0,1), (0,2), (1,2), (3,4), (3,5), (4,5), (2,3)],
    ///     false, None,
    /// ).unwrap();
    /// let result = g.leiden().unwrap();
    /// assert!(result.quality > 0.0);
    /// ```
    pub fn leiden(&self) -> IgraphResult<crate::algorithms::community::leiden::LeidenResult> {
        crate::algorithms::community::leiden::leiden(self)
    }

    /// Find all bridge edges (edges whose removal disconnects the graph).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// // 0-1-2 with 1-2 as bridge vs. 0-1, 0-2, 1-2 (triangle, no bridges)
    /// let g = Graph::from_edges(&[(0,1), (1,2)], false, None).unwrap();
    /// let br = g.bridges().unwrap();
    /// assert_eq!(br.len(), 2); // both edges are bridges in a path
    /// ```
    pub fn bridges(&self) -> IgraphResult<Vec<EdgeId>> {
        crate::algorithms::connectivity::bridges::bridges(self)
    }

    /// Compute the k-core decomposition (coreness of each vertex).
    ///
    /// The coreness of a vertex is the largest `k` such that the vertex
    /// belongs to a k-core — a maximal subgraph where every vertex has
    /// degree at least `k`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// // Triangle (0,1,2) plus pendant vertex 3 attached to 0
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,0), (0,3)], false, None).unwrap();
    /// let cores = g.coreness().unwrap();
    /// assert_eq!(cores[0], 2); // part of the triangle
    /// assert_eq!(cores[3], 1); // pendant
    /// ```
    pub fn coreness(&self) -> IgraphResult<Vec<u32>> {
        crate::algorithms::properties::coreness::coreness(self)
    }

    /// Create the induced subgraph on the given vertex set.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,3), (3,0)], false, None).unwrap();
    /// let sub = g.induced_subgraph(&[0, 1, 2]).unwrap();
    /// assert_eq!(sub.graph.vcount(), 3);
    /// assert_eq!(sub.graph.ecount(), 2); // edges 0-1 and 1-2
    /// ```
    pub fn induced_subgraph(
        &self,
        vertices: &[VertexId],
    ) -> IgraphResult<crate::algorithms::operators::induced_subgraph::InducedSubgraphResult> {
        crate::algorithms::operators::induced_subgraph::induced_subgraph(self, vertices)
    }

    /// Export the graph in DOT (Graphviz) format as a string.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2)], false, None).unwrap();
    /// let dot = g.to_dot(None).unwrap();
    /// assert!(dot.contains("--"));
    /// ```
    pub fn to_dot(&self, labels: Option<&[String]>) -> IgraphResult<String> {
        let mut buf = Vec::new();
        crate::algorithms::io::dot::write_dot(self, labels, &mut buf)?;
        String::from_utf8(buf).map_err(|e| {
            IgraphError::InvalidArgument(format!("DOT output is not valid UTF-8: {e}"))
        })
    }

    /// BFS traversal from a root vertex, returning visit order.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (0,2), (1,3)], false, None).unwrap();
    /// let order = g.bfs(0).unwrap();
    /// assert_eq!(order[0], 0);
    /// assert_eq!(order.len(), 4);
    /// ```
    pub fn bfs(&self, root: VertexId) -> IgraphResult<Vec<VertexId>> {
        crate::algorithms::traversal::bfs::bfs(self, root)
    }

    /// DFS traversal from a root vertex, returning visit order.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (0,2), (1,3)], false, None).unwrap();
    /// let order = g.dfs(0).unwrap();
    /// assert_eq!(order[0], 0);
    /// assert_eq!(order.len(), 4);
    /// ```
    pub fn dfs(&self, root: VertexId) -> IgraphResult<Vec<VertexId>> {
        crate::algorithms::traversal::dfs::dfs(self, root)
    }

    /// Unweighted shortest paths from a source to all reachable vertices.
    ///
    /// Returns a vector of paths (each path is a `Vec<VertexId>`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,3)], false, None).unwrap();
    /// let paths = g.shortest_paths(0).unwrap();
    /// assert_eq!(paths[3], vec![0, 1, 2, 3]);
    /// ```
    pub fn shortest_paths(&self, source: VertexId) -> IgraphResult<Vec<Vec<VertexId>>> {
        crate::algorithms::paths::shortest_paths::get_shortest_paths(self, source)
    }

    /// Weighted shortest-path distances from a source (Dijkstra).
    ///
    /// Returns distances to all vertices; `None` for unreachable vertices.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,3)], false, None).unwrap();
    /// let weights = vec![1.0, 2.0, 3.0];
    /// let dist = g.dijkstra(0, &weights).unwrap();
    /// assert!((dist[3].unwrap() - 6.0).abs() < 1e-10);
    /// ```
    pub fn dijkstra(&self, source: VertexId, weights: &[f64]) -> IgraphResult<Vec<Option<f64>>> {
        crate::algorithms::paths::dijkstra::dijkstra_distances(self, source, weights)
    }

    /// Compute the degree sequence (degree of each vertex).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (0,2), (1,2)], false, None).unwrap();
    /// let seq = g.degree_sequence().unwrap();
    /// assert_eq!(seq, vec![2, 2, 2]);
    /// ```
    pub fn degree_sequence(&self) -> IgraphResult<Vec<u32>> {
        crate::algorithms::properties::degree::degree_sequence(
            self,
            crate::algorithms::properties::degree::DegreeMode::All,
        )
    }

    /// Compute the graph diameter (longest shortest path).
    ///
    /// Returns `None` for graphs with zero vertices.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,3)], false, None).unwrap();
    /// assert_eq!(g.diameter().unwrap(), Some(3));
    /// ```
    pub fn diameter(&self) -> IgraphResult<Option<u32>> {
        crate::algorithms::paths::radii::diameter(self)
    }

    /// Compute the global transitivity (clustering coefficient).
    ///
    /// Returns `None` if there are no connected triples.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,0)], false, None).unwrap();
    /// let t = g.transitivity().unwrap().unwrap();
    /// assert!((t - 1.0).abs() < 1e-10); // triangle is fully transitive
    /// ```
    pub fn transitivity(&self) -> IgraphResult<Option<f64>> {
        crate::algorithms::properties::triangles::transitivity_undirected(self)
    }

    /// Compute the clique number (size of the largest clique).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,0), (2,3)], false, None).unwrap();
    /// assert_eq!(g.clique_number().unwrap(), 3);
    /// ```
    pub fn clique_number(&self) -> IgraphResult<u32> {
        crate::algorithms::cliques::clique_number(self)
    }

    /// Find all articulation points (cut vertices).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,3), (1,3)], false, None).unwrap();
    /// let ap = g.articulation_points().unwrap();
    /// assert_eq!(ap, vec![1]); // vertex 1 is the only cut vertex
    /// ```
    pub fn articulation_points(&self) -> IgraphResult<Vec<VertexId>> {
        crate::algorithms::connectivity::articulation::articulation_points(self)
    }

    /// Topological sort (DAG only, returns error for cyclic graphs).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (0,2)], true, None).unwrap();
    /// let order = g.topological_sort().unwrap();
    /// assert_eq!(order[0], 0); // source comes first
    /// ```
    pub fn topological_sort(&self) -> IgraphResult<Vec<VertexId>> {
        crate::algorithms::properties::topological_sorting::topological_sorting(
            self,
            crate::algorithms::paths::dijkstra::DijkstraMode::Out,
        )
    }

    /// Compute a minimum spanning tree (unweighted).
    ///
    /// Returns the edge ids forming the MST.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,0), (2,3)], false, None).unwrap();
    /// let mst_edges = g.minimum_spanning_tree().unwrap();
    /// assert_eq!(mst_edges.len(), 3); // n-1 edges for connected graph
    /// ```
    pub fn minimum_spanning_tree(&self) -> IgraphResult<Vec<EdgeId>> {
        crate::algorithms::spanning::mst::minimum_spanning_tree(
            self,
            None,
            crate::algorithms::spanning::mst::MstAlgorithm::Automatic,
        )
    }

    /// Compute a quick structural summary of the graph.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,0)], false, None).unwrap();
    /// let s = g.summary().unwrap();
    /// assert_eq!(s.vcount, 3);
    /// assert!(s.connected);
    /// ```
    pub fn summary(&self) -> IgraphResult<crate::algorithms::properties::summary::GraphSummary> {
        crate::algorithms::properties::summary::graph_summary(self)
    }

    /// Compute the maximum flow value between two vertices.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(
    ///     &[(0,1), (0,2), (1,3), (2,3)], true, None
    /// ).unwrap();
    /// let flow = g.max_flow(0, 3).unwrap();
    /// assert!((flow - 2.0).abs() < 1e-10);
    /// ```
    pub fn max_flow(&self, source: VertexId, target: VertexId) -> IgraphResult<f64> {
        crate::algorithms::flow::max_flow::max_flow_value(self, source, target, None)
    }

    /// Decompose the graph into its connected components as separate graphs.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (2,3)], false, None).unwrap();
    /// let components = g.decompose().unwrap();
    /// assert_eq!(components.len(), 2);
    /// ```
    pub fn decompose(&self) -> IgraphResult<Vec<Graph>> {
        crate::algorithms::connectivity::decompose::decompose(self)
    }

    /// Check whether the graph is biconnected.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,0)], false, None).unwrap();
    /// assert!(g.is_biconnected().unwrap());
    /// ```
    pub fn is_biconnected(&self) -> IgraphResult<bool> {
        crate::algorithms::connectivity::is_biconnected::is_biconnected(self)
    }

    /// Run label propagation community detection.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(
    ///     &[(0,1), (1,2), (2,0), (3,4), (4,5), (5,3)], false, None
    /// ).unwrap();
    /// let result = g.label_propagation().unwrap();
    /// assert!(result.membership.len() == 6);
    /// ```
    pub fn label_propagation(
        &self,
    ) -> IgraphResult<crate::algorithms::community::label_propagation::LpaResult> {
        crate::algorithms::community::label_propagation::label_propagation(self)
    }

    /// Run Walktrap community detection.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(
    ///     &[(0,1), (1,2), (2,0), (3,4), (4,5), (5,3), (2,3)], false, None
    /// ).unwrap();
    /// let result = g.walktrap().unwrap();
    /// assert!(result.membership.len() == 6);
    /// ```
    pub fn walktrap(&self) -> IgraphResult<crate::algorithms::community::walktrap::WalktrapResult> {
        crate::algorithms::community::walktrap::walktrap(self)
    }

    /// Run fast greedy modularity community detection.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(
    ///     &[(0,1), (1,2), (2,0), (3,4), (4,5), (5,3), (2,3)], false, None
    /// ).unwrap();
    /// let result = g.fast_greedy().unwrap();
    /// assert!(result.membership.len() == 6);
    /// ```
    pub fn fast_greedy(
        &self,
    ) -> IgraphResult<crate::algorithms::community::fast_greedy_modularity::FastGreedyResult> {
        crate::algorithms::community::fast_greedy_modularity::fast_greedy_modularity(self)
    }

    /// Compute hub and authority scores (HITS algorithm).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,0)], true, None).unwrap();
    /// let hits = g.hits().unwrap();
    /// assert_eq!(hits.hub.len(), 3);
    /// assert_eq!(hits.authority.len(), 3);
    /// ```
    pub fn hits(&self) -> IgraphResult<crate::algorithms::properties::hits::HitsScores> {
        crate::algorithms::properties::hits::hub_and_authority_scores(self)
    }

    /// Compute Katz centrality for all vertices.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,0)], false, None).unwrap();
    /// let katz = g.katz_centrality(0.1, 1.0).unwrap();
    /// assert_eq!(katz.len(), 3);
    /// ```
    pub fn katz_centrality(&self, alpha: f64, beta: f64) -> IgraphResult<Vec<f64>> {
        crate::algorithms::properties::katz_centrality::katz_centrality(
            self, alpha, beta, None, None,
        )
    }

    /// Compute degree assortativity of the graph.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,3)], false, None).unwrap();
    /// let a = g.assortativity().unwrap();
    /// assert!(a.is_some());
    /// ```
    pub fn assortativity(&self) -> IgraphResult<Option<f64>> {
        crate::algorithms::properties::assortativity::assortativity_degree(self)
    }

    /// Read a graph from an edge list file.
    ///
    /// Each line should contain two space-separated vertex ids.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edgelist_file("my_graph.edges").unwrap();
    /// println!("{}", g.vcount());
    /// ```
    pub fn from_edgelist_file<P: AsRef<std::path::Path>>(path: P) -> IgraphResult<Self> {
        let file = std::fs::File::open(path.as_ref())
            .map_err(|e| IgraphError::InvalidArgument(format!("cannot open file: {e}")))?;
        crate::algorithms::io::edgelist::read_edgelist(std::io::BufReader::new(file))
    }

    /// Write the graph to a file in edge list format.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2)], false, None).unwrap();
    /// g.to_edgelist_file("output.edges").unwrap();
    /// ```
    pub fn to_edgelist_file<P: AsRef<std::path::Path>>(&self, path: P) -> IgraphResult<()> {
        let mut file = std::fs::File::create(path.as_ref())
            .map_err(|e| IgraphError::InvalidArgument(format!("cannot create file: {e}")))?;
        crate::algorithms::io::edgelist::write_edgelist(self, &mut file)
    }

    /// Read a graph from a GML file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_gml_file("network.gml").unwrap();
    /// ```
    pub fn from_gml_file<P: AsRef<std::path::Path>>(path: P) -> IgraphResult<Self> {
        let file = std::fs::File::open(path.as_ref())
            .map_err(|e| IgraphError::InvalidArgument(format!("cannot open file: {e}")))?;
        crate::algorithms::io::gml::read_gml(std::io::BufReader::new(file))
    }

    /// Write the graph to a file in GML format.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2)], false, None).unwrap();
    /// g.to_gml_file("output.gml").unwrap();
    /// ```
    pub fn to_gml_file<P: AsRef<std::path::Path>>(&self, path: P) -> IgraphResult<()> {
        let mut file = std::fs::File::create(path.as_ref())
            .map_err(|e| IgraphError::InvalidArgument(format!("cannot create file: {e}")))?;
        crate::algorithms::io::gml::write_gml(self, &mut file)
    }

    /// Read a graph from a `GraphML` file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_graphml_file("network.graphml").unwrap();
    /// ```
    pub fn from_graphml_file<P: AsRef<std::path::Path>>(path: P) -> IgraphResult<Self> {
        let file = std::fs::File::open(path.as_ref())
            .map_err(|e| IgraphError::InvalidArgument(format!("cannot open file: {e}")))?;
        let result = crate::algorithms::io::graphml::read_graphml(std::io::BufReader::new(file))?;
        Ok(result.graph)
    }

    /// Write the graph to a file in `GraphML` format.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2)], false, None).unwrap();
    /// g.to_graphml_file("output.graphml").unwrap();
    /// ```
    pub fn to_graphml_file<P: AsRef<std::path::Path>>(&self, path: P) -> IgraphResult<()> {
        let mut file = std::fs::File::create(path.as_ref())
            .map_err(|e| IgraphError::InvalidArgument(format!("cannot create file: {e}")))?;
        crate::algorithms::io::graphml::write_graphml(self, None, &mut file)
    }

    /// Read a graph from a DOT (Graphviz) file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_dot_file("network.dot").unwrap();
    /// ```
    pub fn from_dot_file<P: AsRef<std::path::Path>>(path: P) -> IgraphResult<Self> {
        let file = std::fs::File::open(path.as_ref())
            .map_err(|e| IgraphError::InvalidArgument(format!("cannot open file: {e}")))?;
        let result = crate::algorithms::io::dot::read_dot(std::io::BufReader::new(file))?;
        Ok(result.graph)
    }

    /// Write the graph to a file in DOT (Graphviz) format.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2)], false, None).unwrap();
    /// g.to_dot_file("output.dot").unwrap();
    /// ```
    pub fn to_dot_file<P: AsRef<std::path::Path>>(&self, path: P) -> IgraphResult<()> {
        let mut file = std::fs::File::create(path.as_ref())
            .map_err(|e| IgraphError::InvalidArgument(format!("cannot create file: {e}")))?;
        crate::algorithms::io::dot::write_dot(self, None, &mut file)
    }

    /// Read a graph from a Pajek (.net) file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_pajek_file("network.net").unwrap();
    /// ```
    pub fn from_pajek_file<P: AsRef<std::path::Path>>(path: P) -> IgraphResult<Self> {
        let file = std::fs::File::open(path.as_ref())
            .map_err(|e| IgraphError::InvalidArgument(format!("cannot open file: {e}")))?;
        let result = crate::algorithms::io::pajek::read_pajek(std::io::BufReader::new(file))?;
        Ok(result.graph)
    }

    /// Write the graph to a file in Pajek (.net) format.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2)], false, None).unwrap();
    /// g.to_pajek_file("output.net").unwrap();
    /// ```
    pub fn to_pajek_file<P: AsRef<std::path::Path>>(&self, path: P) -> IgraphResult<()> {
        let mut file = std::fs::File::create(path.as_ref())
            .map_err(|e| IgraphError::InvalidArgument(format!("cannot create file: {e}")))?;
        crate::algorithms::io::pajek::write_pajek(self, None, None, &mut file)
    }

    /// Generate an Erdos-Renyi G(n, p) random graph.
    ///
    /// Each possible edge exists independently with probability `p`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::erdos_renyi(100, 0.05, 42).unwrap();
    /// assert_eq!(g.vcount(), 100);
    /// assert!(!g.is_directed());
    /// ```
    pub fn erdos_renyi(n: u32, p: f64, seed: u64) -> IgraphResult<Self> {
        crate::algorithms::games::erdos_renyi::erdos_renyi_gnp(n, p, false, false, seed)
    }

    /// Generate a Barabasi-Albert preferential attachment graph.
    ///
    /// Starts with one vertex and adds `n - 1` vertices, each connecting
    /// to `m` existing vertices chosen with probability proportional to degree.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::barabasi_albert(100, 2, 42).unwrap();
    /// assert_eq!(g.vcount(), 100);
    /// assert!(!g.is_directed());
    /// ```
    pub fn barabasi_albert(n: u32, m: u32, seed: u64) -> IgraphResult<Self> {
        crate::algorithms::games::barabasi::barabasi_game_bag(n, m, true, false, seed)
    }

    /// Generate a Watts-Strogatz small-world graph.
    ///
    /// Creates a ring lattice with `n` vertices where each vertex is connected
    /// to its `k` nearest neighbours (must be even), then rewires each edge
    /// with probability `p`. This produces graphs with both high clustering
    /// and short path lengths — the "small-world" property.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::watts_strogatz(20, 4, 0.3, 42).unwrap();
    /// assert_eq!(g.vcount(), 20);
    /// assert_eq!(g.ecount(), 40); // n * k / 2 = 20 * 4 / 2
    /// ```
    pub fn watts_strogatz(n: u32, k: u32, p: f64, seed: u64) -> IgraphResult<Self> {
        crate::algorithms::games::watts::watts_strogatz_game(n, k / 2, p, false, false, seed)
    }

    /// Compute strongly connected components (directed graphs).
    ///
    /// For undirected graphs, this is equivalent to connected components.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,0), (2,3)], true, None).unwrap();
    /// let scc = g.strongly_connected_components().unwrap();
    /// assert_eq!(scc.count, 2);
    /// ```
    pub fn strongly_connected_components(
        &self,
    ) -> IgraphResult<crate::algorithms::connectivity::components::ConnectedComponents> {
        crate::algorithms::connectivity::strong::strongly_connected_components(self)
    }

    /// Find the shortest path between two vertices.
    ///
    /// Uses BFS for unweighted graphs, Dijkstra/Bellman-Ford for weighted.
    /// Returns the vertex and edge sequences along the path.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(
    ///     &[(0,1), (1,2), (2,3), (0,3)], false, None
    /// ).unwrap();
    /// let path = g.shortest_path_to(0, 3, None).unwrap();
    /// assert_eq!(path.vertices, vec![0, 3]);
    /// ```
    pub fn shortest_path_to(
        &self,
        source: VertexId,
        target: VertexId,
        weights: Option<&[f64]>,
    ) -> IgraphResult<crate::algorithms::paths::get_shortest_path::ShortestPath> {
        use crate::algorithms::paths::dijkstra::DijkstraMode;
        let mode = if self.directed {
            DijkstraMode::Out
        } else {
            DijkstraMode::All
        };
        crate::algorithms::paths::get_shortest_path::get_shortest_path(
            self, source, target, weights, mode,
        )
    }

    /// Compute the average path length of the graph.
    ///
    /// Returns the mean shortest-path distance over all reachable vertex pairs.
    /// Unreachable pairs are excluded. Returns `None` if the graph has fewer
    /// than 2 vertices or no reachable pairs exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,3)], false, None).unwrap();
    /// let apl = g.average_path_length().unwrap().unwrap();
    /// assert!((apl - 5.0 / 3.0).abs() < 1e-10); // (1+2+3+1+2+1)/6
    /// ```
    pub fn average_path_length(&self) -> IgraphResult<Option<f64>> {
        crate::algorithms::properties::basic::mean_distance(self)
    }

    /// Check if the graph is bipartite and return the partition if so.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,3)], false, None).unwrap();
    /// let result = g.is_bipartite().unwrap();
    /// assert!(result.is_bipartite);
    /// ```
    pub fn is_bipartite(
        &self,
    ) -> IgraphResult<crate::algorithms::properties::is_bipartite::BipartiteResult> {
        crate::algorithms::properties::is_bipartite::is_bipartite(self)
    }

    /// Remove self-loops and/or multi-edges from the graph.
    ///
    /// Returns a new simplified graph.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let mut g = Graph::with_vertices(3);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(0, 1).unwrap(); // multi-edge
    /// g.add_edge(1, 1).unwrap(); // self-loop
    /// let simple = g.simplify(true, true).unwrap();
    /// assert_eq!(simple.ecount(), 1);
    /// ```
    pub fn simplify(&self, remove_multiple: bool, remove_loops: bool) -> IgraphResult<Graph> {
        crate::algorithms::operators::simplify::simplify(self, remove_multiple, remove_loops)
    }

    /// Reverse all edge directions (directed graphs only).
    ///
    /// For undirected graphs, returns a copy of the graph unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2)], true, None).unwrap();
    /// let r = g.reverse().unwrap();
    /// assert_eq!(r.neighbors(2).unwrap(), vec![1]);
    /// ```
    pub fn reverse(&self) -> IgraphResult<Graph> {
        crate::algorithms::operators::reverse::reverse(self)
    }

    /// Convert an undirected graph to directed.
    ///
    /// In `Mutual` mode each undirected edge becomes two directed edges
    /// (u→v and v→u). In `Arbitrary` mode each edge gets one direction
    /// (smaller → larger vertex id). Already-directed graphs are copied
    /// unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::{Graph, ToDirectedMode};
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2)], false, None).unwrap();
    /// let d = g.to_directed(ToDirectedMode::Mutual).unwrap();
    /// assert!(d.is_directed());
    /// assert_eq!(d.ecount(), 4);
    /// ```
    pub fn to_directed(
        &self,
        mode: crate::algorithms::operators::to_directed::ToDirectedMode,
    ) -> IgraphResult<Graph> {
        crate::algorithms::operators::to_directed::to_directed(self, mode)
    }

    /// Convert a directed graph to undirected.
    ///
    /// `Each` keeps every directed edge as undirected. `Collapse` merges
    /// mutual pairs into one edge. `Mutual` keeps only edges that exist
    /// in both directions. Already-undirected graphs are copied unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::{Graph, ToUndirectedMode};
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,0), (1,2)], true, None).unwrap();
    /// let u = g.to_undirected(ToUndirectedMode::Collapse).unwrap();
    /// assert!(!u.is_directed());
    /// assert_eq!(u.ecount(), 2);
    /// ```
    pub fn to_undirected(
        &self,
        mode: crate::algorithms::operators::to_undirected::ToUndirectedMode,
    ) -> IgraphResult<Graph> {
        crate::algorithms::operators::to_undirected::to_undirected(self, mode)
    }

    /// Contract vertices according to a mapping.
    ///
    /// `mapping[v]` specifies the new vertex id for vertex `v`. Vertices
    /// with the same mapping value are merged into one vertex.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,3)], false, None).unwrap();
    /// // Merge vertices 0,1 → 0 and 2,3 → 1
    /// let contracted = g.contract_vertices(&[0, 0, 1, 1]).unwrap();
    /// assert_eq!(contracted.vcount(), 2);
    /// ```
    pub fn contract_vertices(&self, mapping: &[VertexId]) -> IgraphResult<Graph> {
        crate::algorithms::operators::contract_vertices::contract_vertices(self, mapping)
    }

    /// Perform a random walk starting from a given vertex.
    ///
    /// Returns the sequence of visited vertex ids (length = `steps + 1`
    /// including the starting vertex, or shorter if the walk gets stuck).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(
    ///     &[(0,1), (1,2), (2,3), (3,0)], false, None
    /// ).unwrap();
    /// let (vertices, edges) = g.random_walk(0, 10, 42).unwrap();
    /// assert_eq!(vertices[0], 0);
    /// assert!(vertices.len() <= 11);
    /// assert_eq!(edges.len(), vertices.len() - 1);
    /// ```
    pub fn random_walk(
        &self,
        start: VertexId,
        steps: u32,
        seed: u64,
    ) -> IgraphResult<(Vec<VertexId>, Vec<EdgeId>)> {
        use crate::algorithms::paths::dijkstra::DijkstraMode;
        let mode = if self.directed {
            DijkstraMode::Out
        } else {
            DijkstraMode::All
        };
        crate::algorithms::paths::random_walk::random_walk(self, None, start, mode, steps, seed)
    }

    // ── Graph properties ─────────────────────────────────────────────

    /// Compute the radius (minimum eccentricity) of the graph.
    ///
    /// Returns `None` for the empty graph.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,3)], false, None).unwrap();
    /// assert_eq!(g.radius().unwrap(), Some(2));
    /// ```
    pub fn radius(&self) -> IgraphResult<Option<u32>> {
        crate::algorithms::paths::radii::radius(self)
    }

    /// Compute the eccentricity of every vertex.
    ///
    /// `result[v]` is the maximum shortest-path distance from vertex `v`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2)], false, None).unwrap();
    /// assert_eq!(g.eccentricity().unwrap(), vec![2, 1, 2]);
    /// ```
    pub fn eccentricity(&self) -> IgraphResult<Vec<u32>> {
        crate::algorithms::paths::radii::eccentricity(self)
    }

    /// Compute the girth (length of the shortest cycle) of the graph.
    ///
    /// Returns `None` if the graph is acyclic.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,0)], false, None).unwrap();
    /// assert_eq!(g.girth().unwrap(), Some(3));
    /// ```
    pub fn girth(&self) -> IgraphResult<Option<u32>> {
        crate::algorithms::properties::girth::girth(self)
    }

    /// Check whether the graph is a tree.
    ///
    /// Returns `Some(root)` where `root` is the first root vertex found,
    /// or `None` if the graph is not a tree. The `mode` parameter controls
    /// how edges are followed for directed graphs.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::{Graph, DijkstraMode};
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (1,3)], false, None).unwrap();
    /// assert!(g.is_tree(DijkstraMode::All).unwrap().is_some());
    /// ```
    pub fn is_tree(
        &self,
        mode: crate::algorithms::paths::dijkstra::DijkstraMode,
    ) -> IgraphResult<Option<VertexId>> {
        crate::algorithms::properties::is_tree::is_tree(self, mode)
    }

    /// Check whether the directed graph is a DAG.
    ///
    /// Returns `false` for undirected graphs.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2)], true, None).unwrap();
    /// assert!(g.is_dag());
    /// ```
    pub fn is_dag(&self) -> bool {
        crate::algorithms::properties::is_dag::is_dag(self)
    }

    /// Count the total number of triangles in the graph.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,0)], false, None).unwrap();
    /// assert_eq!(g.count_triangles().unwrap(), 1);
    /// ```
    pub fn count_triangles(&self) -> IgraphResult<u64> {
        crate::algorithms::properties::triangles::count_triangles(self)
    }

    /// Compute the harmonic centrality of all vertices.
    ///
    /// Harmonic centrality of `v` is the sum of inverse distances
    /// from `v` to all other reachable vertices.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2)], false, None).unwrap();
    /// let h = g.harmonic_centrality().unwrap();
    /// assert_eq!(h.len(), 3);
    /// ```
    pub fn harmonic_centrality(&self) -> IgraphResult<Vec<f64>> {
        crate::algorithms::properties::harmonic::harmonic_centrality(self)
    }

    /// Compute the k-hop neighborhood size for every vertex.
    ///
    /// `result[v]` is the number of vertices within distance `order` from
    /// `v` (including `v` itself).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,3)], false, None).unwrap();
    /// let sizes = g.neighborhood_size(1).unwrap();
    /// assert_eq!(sizes, vec![2, 3, 3, 2]);
    /// ```
    pub fn neighborhood_size(&self, order: i32) -> IgraphResult<Vec<u32>> {
        crate::algorithms::properties::neighborhood::neighborhood_size(self, order)
    }

    // ── Connectivity ─────────────────────────────────────────────────

    /// Compute the vertex connectivity (minimum vertex cut) of the graph.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(
    ///     &[(0,1), (0,2), (1,3), (2,3)], false, None,
    /// ).unwrap();
    /// assert_eq!(g.vertex_connectivity().unwrap(), 2);
    /// ```
    pub fn vertex_connectivity(&self) -> IgraphResult<i64> {
        crate::algorithms::flow::vertex_connectivity::vertex_connectivity(self, true)
    }

    /// Compute the edge connectivity (minimum edge cut) of the graph.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(
    ///     &[(0,1), (0,2), (1,3), (2,3)], false, None,
    /// ).unwrap();
    /// assert_eq!(g.edge_connectivity().unwrap(), 2);
    /// ```
    pub fn edge_connectivity(&self) -> IgraphResult<i64> {
        crate::algorithms::flow::edge_connectivity::edge_connectivity(self, true)
    }

    /// Find all vertices reachable from `source`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::{Graph, SubcomponentMode};
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (3,4)], false, None).unwrap();
    /// let comp = g.subcomponent(0, SubcomponentMode::All).unwrap();
    /// assert_eq!(comp.len(), 3);
    /// ```
    pub fn subcomponent(
        &self,
        source: VertexId,
        mode: crate::algorithms::connectivity::subcomponent::SubcomponentMode,
    ) -> IgraphResult<Vec<VertexId>> {
        crate::algorithms::connectivity::subcomponent::subcomponent(self, source, mode)
    }

    // ── Cliques ──────────────────────────────────────────────────────

    /// Find all cliques in the graph within a size range.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,0)], false, None).unwrap();
    /// let c = g.cliques(3, 3, None).unwrap();
    /// assert_eq!(c.len(), 1);
    /// ```
    pub fn cliques(
        &self,
        min_size: u32,
        max_size: u32,
        max_results: Option<usize>,
    ) -> IgraphResult<Vec<Vec<VertexId>>> {
        crate::algorithms::cliques::cliques(self, min_size, max_size, max_results)
    }

    /// Find all maximal cliques in the graph.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,0)], false, None).unwrap();
    /// let mc = g.maximal_cliques().unwrap();
    /// assert_eq!(mc.len(), 1);
    /// assert_eq!(mc[0].len(), 3);
    /// ```
    pub fn maximal_cliques(&self) -> IgraphResult<Vec<Vec<VertexId>>> {
        crate::algorithms::cliques::maximal_cliques(self)
    }

    /// Compute the independence number (max independent set size).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// // Triangle: independence number is 1 (can pick at most 1 non-adjacent vertex pair... no, 1)
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,0)], false, None).unwrap();
    /// assert_eq!(g.independence_number().unwrap(), 1);
    /// ```
    pub fn independence_number(&self) -> IgraphResult<u32> {
        crate::algorithms::cliques::independence_number(self)
    }

    // ── Operators ────────────────────────────────────────────────────

    /// Permute the vertices of the graph.
    ///
    /// `permutation[v]` gives the new id for vertex `v`. Returns a new
    /// graph with edges reconnected accordingly.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// // permutation[new] = old: new 0 ← old 2, new 1 ← old 0, new 2 ← old 1
    /// let g = Graph::from_edges(&[(0,1), (1,2)], false, None).unwrap();
    /// let p = g.permute_vertices(&[2, 0, 1]).unwrap();
    /// assert!(p.has_edge(1, 2));
    /// assert!(p.has_edge(2, 0));
    /// ```
    pub fn permute_vertices(&self, permutation: &[VertexId]) -> IgraphResult<Graph> {
        crate::algorithms::operators::permute_vertices::permute_vertices(self, permutation)
    }

    // ── Layout ───────────────────────────────────────────────────────

    /// Fruchterman-Reingold force-directed layout with default parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,0)], false, None).unwrap();
    /// let coords = g.layout_fruchterman_reingold().unwrap();
    /// assert_eq!(coords.len(), 3);
    /// ```
    pub fn layout_fruchterman_reingold(&self) -> IgraphResult<Vec<(f64, f64)>> {
        use crate::algorithms::layout::fruchterman_reingold::FrParams;
        crate::algorithms::layout::fruchterman_reingold::layout_fruchterman_reingold(
            self,
            &FrParams::default(),
        )
    }

    /// Kamada-Kawai spring-embedder layout with default parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,3)], false, None).unwrap();
    /// let coords = g.layout_kamada_kawai().unwrap();
    /// assert_eq!(coords.len(), 4);
    /// ```
    pub fn layout_kamada_kawai(&self) -> IgraphResult<Vec<[f64; 2]>> {
        use crate::algorithms::layout::kamada_kawai::KkParams;
        let params = KkParams::default_for(self.vcount() as usize);
        crate::algorithms::layout::kamada_kawai::layout_kamada_kawai(self, None, &params, None)
    }

    /// `DrL` (Distributed Recursive Layout) with default options.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,0)], false, None).unwrap();
    /// let coords = g.layout_drl().unwrap();
    /// assert_eq!(coords.len(), 3);
    /// ```
    pub fn layout_drl(&self) -> IgraphResult<Vec<[f64; 2]>> {
        use crate::algorithms::layout::drl::DrlOptions;
        crate::algorithms::layout::drl::layout_drl(self, None, &DrlOptions::default(), None)
    }

    /// Circular layout.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,0)], false, None).unwrap();
    /// let coords = g.layout_circle();
    /// assert_eq!(coords.len(), 3);
    /// ```
    pub fn layout_circle(&self) -> Vec<(f64, f64)> {
        crate::algorithms::layout::simple::layout_circle(self, None)
    }

    /// Random layout with the given RNG seed.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::with_vertices(5);
    /// let coords = g.layout_random(42);
    /// assert_eq!(coords.len(), 5);
    /// ```
    pub fn layout_random(&self, seed: u64) -> Vec<(f64, f64)> {
        crate::algorithms::layout::simple::layout_random(self, seed)
    }

    /// Grid layout.
    ///
    /// `width` specifies the number of columns. Pass 0 to auto-compute
    /// (ceil of square root of vertex count).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::with_vertices(9);
    /// let coords = g.layout_grid(3);
    /// assert_eq!(coords.len(), 9);
    /// ```
    pub fn layout_grid(&self, width: i32) -> Vec<(f64, f64)> {
        crate::algorithms::layout::simple::layout_grid(self, width)
    }

    // ── Triangle / local clustering ──────────────────────────────────

    /// Per-vertex triangle count.
    ///
    /// `result[v]` is the number of triangles incident to vertex `v`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1),(1,2),(2,0),(2,3)], false, None).unwrap();
    /// let t = g.count_adjacent_triangles().unwrap();
    /// assert_eq!(t[0], 1);
    /// assert_eq!(t[3], 0);
    /// ```
    pub fn count_adjacent_triangles(&self) -> IgraphResult<Vec<u64>> {
        crate::algorithms::properties::triangles::count_adjacent_triangles(self)
    }

    /// Per-vertex local clustering coefficient (local transitivity).
    ///
    /// `result[v]` is `None` for vertices with degree < 2.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1),(1,2),(2,0),(2,3)], false, None).unwrap();
    /// let lcc = g.transitivity_local_undirected().unwrap();
    /// assert!(lcc[0].unwrap() > 0.9); // vertex 0 in triangle
    /// assert!(lcc[3].is_none());       // degree 1
    /// ```
    pub fn transitivity_local_undirected(&self) -> IgraphResult<Vec<Option<f64>>> {
        crate::algorithms::properties::triangles::transitivity_local_undirected(self)
    }

    // ── Network metrics ──────────────────────────────────────────────

    /// Reciprocity of a directed graph.
    ///
    /// Returns the fraction of edges that are reciprocated, or `None` for
    /// empty graphs.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1),(1,0),(1,2)], true, None).unwrap();
    /// let r = g.reciprocity().unwrap().unwrap();
    /// assert!((r - 2.0/3.0).abs() < 1e-12);
    /// ```
    pub fn reciprocity(&self) -> IgraphResult<Option<f64>> {
        crate::algorithms::properties::reciprocity::reciprocity(self)
    }

    /// Burt's constraint for each vertex.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1),(0,2),(1,2)], false, None).unwrap();
    /// let c = g.constraint(None).unwrap();
    /// assert_eq!(c.len(), 3);
    /// ```
    pub fn constraint(&self, weights: Option<&[f64]>) -> IgraphResult<Vec<f64>> {
        crate::algorithms::properties::constraint::constraint(self, weights)
    }

    /// Whether the graph has multi-edges.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1),(0,1)], false, None).unwrap();
    /// assert!(g.has_multiple().unwrap());
    /// ```
    pub fn has_multiple(&self) -> IgraphResult<bool> {
        crate::algorithms::properties::multiplicity::has_multiple(self)
    }

    /// Per-edge multiplicity count.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1),(0,1),(1,2)], false, None).unwrap();
    /// let mc = g.count_multiple().unwrap();
    /// assert_eq!(mc[0], 2);
    /// assert_eq!(mc[2], 1);
    /// ```
    pub fn count_multiple(&self) -> IgraphResult<Vec<usize>> {
        crate::algorithms::properties::multiplicity::count_multiple(self)
    }

    /// Test whether two vertices are adjacent.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1),(1,2)], false, None).unwrap();
    /// assert!(g.are_adjacent(0, 1).unwrap());
    /// assert!(!g.are_adjacent(0, 2).unwrap());
    /// ```
    pub fn are_adjacent(&self, v1: VertexId, v2: VertexId) -> IgraphResult<bool> {
        crate::algorithms::properties::are_adjacent::are_adjacent(self, v1, v2)
    }

    // ── Motifs ───────────────────────────────────────────────────────

    /// Triad census of a directed graph.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::{Graph, TriadType};
    ///
    /// let g = Graph::from_edges(&[(0,1),(1,2),(2,0)], true, None).unwrap();
    /// let tc = g.triad_census().unwrap();
    /// assert!(tc.get(TriadType::T030C) > 0.0);
    /// ```
    pub fn triad_census(
        &self,
    ) -> IgraphResult<crate::algorithms::motifs::triad_census::TriadCensus> {
        crate::algorithms::motifs::triad_census::triad_census(self)
    }

    /// Dyad census of a directed graph.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1),(1,0),(1,2)], true, None).unwrap();
    /// let dc = g.dyad_census().unwrap();
    /// assert!((dc.mutual - 1.0).abs() < 1e-12);
    /// ```
    pub fn dyad_census(&self) -> IgraphResult<crate::algorithms::motifs::DyadCensus> {
        crate::algorithms::motifs::dyad_census(self)
    }

    // ── Similarity ───────────────────────────────────────────────────

    /// Jaccard similarity between all pairs of vertices.
    ///
    /// Returns a flattened `n × n` matrix (row-major).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1),(0,2),(1,2)], false, None).unwrap();
    /// let sim = g.similarity_jaccard().unwrap();
    /// assert_eq!(sim.len(), 9); // 3×3 matrix
    /// ```
    pub fn similarity_jaccard(&self) -> IgraphResult<Vec<f64>> {
        crate::algorithms::properties::similarity::similarity_jaccard(self)
    }

    /// Co-citation scores for all vertex pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,2),(1,2)], true, None).unwrap();
    /// let cc = g.cocitation().unwrap();
    /// assert!(!cc.is_empty());
    /// ```
    pub fn cocitation(&self) -> IgraphResult<Vec<u32>> {
        crate::algorithms::properties::similarity::cocitation(self)
    }

    // ── Graph structure recognizers ─────────────────────────────────

    /// Check whether the graph is a cograph.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (0,2), (1,2)], false, None).unwrap();
    /// assert!(g.is_cograph().unwrap());
    /// ```
    pub fn is_cograph(&self) -> IgraphResult<bool> {
        crate::algorithms::properties::is_cograph::is_cograph(self)
    }

    /// Check whether the graph is series-parallel.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,3)], false, None).unwrap();
    /// assert!(g.is_series_parallel().unwrap());
    /// ```
    pub fn is_series_parallel(&self) -> IgraphResult<bool> {
        crate::algorithms::properties::is_series_parallel::is_series_parallel(self)
    }

    /// Check whether the graph is outerplanar.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,0)], false, None).unwrap();
    /// assert!(g.is_outerplanar().unwrap());
    /// ```
    pub fn is_outerplanar(&self) -> IgraphResult<bool> {
        crate::algorithms::properties::is_outerplanar::is_outerplanar(self)
    }

    /// Check whether the graph is chordal.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(
    ///     &[(0,1), (1,2), (2,0), (0,3), (1,3), (2,3)], false, None
    /// ).unwrap();
    /// assert!(g.is_chordal().unwrap());
    /// ```
    pub fn is_chordal(&self) -> IgraphResult<bool> {
        let result = crate::algorithms::chordality::is_chordal(self, None)?;
        Ok(result.chordal)
    }

    /// Check whether the graph is a forest (acyclic).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,3)], false, None).unwrap();
    /// assert!(g.is_forest().unwrap());
    /// ```
    pub fn is_forest(&self) -> IgraphResult<bool> {
        Ok(crate::algorithms::properties::is_forest::is_forest(
            self,
            crate::algorithms::paths::dijkstra::DijkstraMode::Out,
        )?
        .is_some())
    }

    // ── Cycles and motifs ───────────────────────────────────────────

    /// Compute a fundamental cycle basis of the graph.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(
    ///     &[(0,1), (1,2), (2,0)], false, None
    /// ).unwrap();
    /// let cycles = g.fundamental_cycles().unwrap();
    /// assert_eq!(cycles.len(), 1);
    /// ```
    pub fn fundamental_cycles(&self) -> IgraphResult<Vec<Vec<u32>>> {
        crate::algorithms::fundamental_cycles::fundamental_cycles(self, None, None)
    }

    /// Compute a minimum weight cycle basis.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(
    ///     &[(0,1), (1,2), (2,0), (1,3), (3,0)], false, None
    /// ).unwrap();
    /// let basis = g.minimum_cycle_basis().unwrap();
    /// assert_eq!(basis.len(), 2);
    /// ```
    pub fn minimum_cycle_basis(&self) -> IgraphResult<Vec<Vec<u32>>> {
        crate::algorithms::minimum_cycle_basis::minimum_cycle_basis(self, None, false)
    }

    // ── Cuts, covers, and sets ──────────────────────────────────────

    /// Find a minimum feedback arc set.
    ///
    /// Returns edges whose removal makes the graph acyclic.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,0)], true, None).unwrap();
    /// let fas = g.feedback_arc_set().unwrap();
    /// assert!(!fas.is_empty());
    /// ```
    pub fn feedback_arc_set(&self) -> IgraphResult<Vec<u32>> {
        crate::algorithms::feedback_arc_set::feedback_arc_set(
            self,
            None,
            crate::algorithms::feedback_arc_set::FasAlgorithm::EadesLinSmyth,
        )
    }

    /// Find the maximum cut of the graph.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(
    ///     &[(0,1), (1,2), (2,3)], false, None
    /// ).unwrap();
    /// let result = g.maximum_cut().unwrap();
    /// assert!(result.cut_value > 0);
    /// ```
    pub fn maximum_cut(&self) -> IgraphResult<crate::algorithms::max_cut::MaxCutResult> {
        crate::algorithms::max_cut::maximum_cut(self)
    }

    /// Find a minimum vertex cover.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2)], false, None).unwrap();
    /// let cover = g.minimum_vertex_cover().unwrap();
    /// assert!(cover.contains(&1));
    /// ```
    pub fn minimum_vertex_cover(&self) -> IgraphResult<Vec<u32>> {
        crate::algorithms::vertex_cover::minimum_vertex_cover(self)
    }

    /// Find a minimum edge cover.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,3)], false, None).unwrap();
    /// let cover = g.minimum_edge_cover().unwrap();
    /// assert!(!cover.is_empty());
    /// ```
    pub fn minimum_edge_cover(&self) -> IgraphResult<Vec<u32>> {
        crate::algorithms::edge_cover::minimum_edge_cover(self)
    }

    /// Find a maximum independent set.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2)], false, None).unwrap();
    /// let mis = g.maximum_independent_set().unwrap();
    /// assert_eq!(mis.len(), 2);
    /// ```
    pub fn maximum_independent_set(&self) -> IgraphResult<Vec<u32>> {
        crate::algorithms::independent_set::maximum_independent_set(self)
    }

    // ── Coloring ────────────────────────────────────────────────────

    /// Greedy vertex coloring.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(
    ///     &[(0,1), (1,2), (2,0)], false, None
    /// ).unwrap();
    /// let colors = g.vertex_coloring().unwrap();
    /// assert_eq!(colors.len(), 3);
    /// // Adjacent vertices must have different colors
    /// assert_ne!(colors[0], colors[1]);
    /// ```
    pub fn vertex_coloring(&self) -> IgraphResult<Vec<u32>> {
        crate::algorithms::coloring::vertex_coloring_greedy(
            self,
            crate::algorithms::coloring::GreedyColoringHeuristic::ColoredNeighbors,
        )
    }

    // ── Spanning trees ──────────────────────────────────────────────

    /// Sample a random spanning tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(
    ///     &[(0,1), (1,2), (2,0), (1,3)], false, None
    /// ).unwrap();
    /// let edges = g.random_spanning_tree(42).unwrap();
    /// assert_eq!(edges.len(), 3);
    /// ```
    pub fn random_spanning_tree(&self, seed: u64) -> IgraphResult<Vec<u32>> {
        crate::algorithms::spanning::random_spanning_tree::random_spanning_tree(self, None, seed)
    }

    // ── Community detection (extended) ──────────────────────────────

    /// Edge betweenness community detection.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(
    ///     &[(0,1), (1,2), (2,0), (3,4), (4,5), (5,3), (2,3)],
    ///     false, None
    /// ).unwrap();
    /// let result = g.edge_betweenness_community().unwrap();
    /// assert!(!result.membership.is_empty());
    /// ```
    pub fn edge_betweenness_community(
        &self,
    ) -> IgraphResult<crate::algorithms::community::edge_betweenness_community::EdgeBetweennessResult>
    {
        crate::algorithms::community::edge_betweenness_community::edge_betweenness_community(self)
    }

    // ── Isomorphism (canonical / BLISS) ─────────────────────────────

    /// Compute a canonical vertex permutation.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2)], false, None).unwrap();
    /// let perm = g.canonical_permutation().unwrap();
    /// assert_eq!(perm.len(), 3);
    /// ```
    pub fn canonical_permutation(&self) -> IgraphResult<Vec<u32>> {
        crate::algorithms::isomorphism::canonical::canonical_permutation::canonical_permutation(
            self, None,
        )
    }

    /// Count automorphisms of the graph.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// // K3 has 3! = 6 automorphisms
    /// let g = Graph::from_edges(
    ///     &[(0,1), (1,2), (2,0)], false, None
    /// ).unwrap();
    /// let count = g.count_automorphisms().unwrap();
    /// assert!((count - 6.0).abs() < 1e-10);
    /// ```
    pub fn count_automorphisms(&self) -> IgraphResult<f64> {
        crate::algorithms::isomorphism::canonical::count_automorphisms::count_automorphisms(
            self, None,
        )
    }

    /// Compute a generating set for the automorphism group.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(&[(0,1), (1,2), (2,0)], false, None).unwrap();
    /// let gens = g.automorphism_group().unwrap();
    /// assert!(!gens.is_empty());
    /// ```
    pub fn automorphism_group(&self) -> IgraphResult<Vec<Vec<u32>>> {
        crate::algorithms::isomorphism::canonical::automorphism_group::automorphism_group(
            self, None,
        )
    }

    // ── Epidemics ───────────────────────────────────────────────────

    /// Run a single SIR (Susceptible-Infected-Recovered) simulation.
    ///
    /// `beta` is the infection rate, `gamma` is the recovery rate.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(
    ///     &[(0,1), (1,2), (2,3), (3,4)], false, None
    /// ).unwrap();
    /// let result = g.sir(0.5, 0.1, 1, 42).unwrap();
    /// assert!(!result.is_empty());
    /// ```
    pub fn sir(
        &self,
        beta: f64,
        gamma: f64,
        no_sim: usize,
        seed: u64,
    ) -> IgraphResult<Vec<crate::algorithms::epidemics::Sir>> {
        crate::algorithms::epidemics::sir(self, beta, gamma, no_sim, seed)
    }

    // ── Spanner ─────────────────────────────────────────────────────

    /// Compute a graph spanner with the given stretch factor.
    ///
    /// Returns edge indices forming the spanner subgraph.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::Graph;
    ///
    /// let g = Graph::from_edges(
    ///     &[(0,1), (1,2), (2,0), (1,3)], false, None
    /// ).unwrap();
    /// let edges = g.spanner(3.0).unwrap();
    /// assert!(!edges.is_empty());
    /// ```
    pub fn spanner(&self, stretch: f64) -> IgraphResult<Vec<u32>> {
        crate::algorithms::paths::spanner::spanner(self, stretch, None)
    }
}

impl std::fmt::Display for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = if self.directed {
            "Directed"
        } else {
            "Undirected"
        };
        write!(
            f,
            "{kind} graph with {} vertices and {} edges",
            self.n,
            self.ecount()
        )
    }
}

/// Iterate over a graph's edges by reference.
///
/// Yields `(from, to)` pairs in edge-id order, enabling the idiomatic
/// `for (u, v) in &graph { ... }` pattern.
///
/// # Examples
///
/// ```
/// use rust_igraph::Graph;
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
///
/// let edges: Vec<_> = (&g).into_iter().collect();
/// assert_eq!(edges, vec![(0, 1), (1, 2)]);
/// ```
impl<'a> IntoIterator for &'a Graph {
    type Item = (VertexId, VertexId);
    type IntoIter = EdgeIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Construct an undirected graph from a slice of `(from, to)` edge pairs.
///
/// Vertex count is inferred from the maximum endpoint id (plus one).
/// This is a convenience for quick construction; for more control use
/// [`Graph::from_edges`] or [`GraphBuilder`](super::builder::GraphBuilder).
///
/// # Examples
///
/// ```
/// use rust_igraph::Graph;
///
/// let edges = vec![(0u32, 1), (1, 2), (2, 0)];
/// let g = Graph::try_from(edges.as_slice()).unwrap();
/// assert_eq!(g.vcount(), 3);
/// assert_eq!(g.ecount(), 3);
/// assert!(!g.is_directed());
/// ```
impl TryFrom<&[(VertexId, VertexId)]> for Graph {
    type Error = IgraphError;

    fn try_from(edges: &[(VertexId, VertexId)]) -> IgraphResult<Self> {
        let n = match edges.iter().flat_map(|&(u, v)| [u, v]).max() {
            Some(m) => m
                .checked_add(1)
                .ok_or_else(|| IgraphError::InvalidArgument("vertex id overflow".to_owned()))?,
            None => 0,
        };
        let mut g = Self::new(n, false)?;
        g.add_edges(edges.to_vec())?;
        Ok(g)
    }
}

/// Construct an undirected graph from a `Vec` of `(from, to)` edge pairs.
///
/// # Examples
///
/// ```
/// use rust_igraph::Graph;
///
/// let g = Graph::try_from(vec![(0u32, 1), (1, 2), (2, 3)]).unwrap();
/// assert_eq!(g.vcount(), 4);
/// assert_eq!(g.ecount(), 3);
/// ```
impl TryFrom<Vec<(VertexId, VertexId)>> for Graph {
    type Error = IgraphError;

    fn try_from(edges: Vec<(VertexId, VertexId)>) -> IgraphResult<Self> {
        let n = match edges.iter().flat_map(|&(u, v)| [u, v]).max() {
            Some(m) => m
                .checked_add(1)
                .ok_or_else(|| IgraphError::InvalidArgument("vertex id overflow".to_owned()))?,
            None => 0,
        };
        let mut g = Self::new(n, false)?;
        g.add_edges(edges)?;
        Ok(g)
    }
}

/// Collect edges from an iterator into an undirected graph.
///
/// Vertex count is inferred from the maximum endpoint id.
/// For directed graphs or explicit vertex counts, use [`Graph::from_edges`].
///
/// # Panics
///
/// Panics if a vertex id would overflow `u32::MAX`.
///
/// # Examples
///
/// ```
/// use rust_igraph::Graph;
///
/// let g: Graph = [(0u32, 1), (1, 2), (2, 0)].into_iter().collect();
/// assert_eq!(g.vcount(), 3);
/// assert_eq!(g.ecount(), 3);
/// ```
impl std::iter::FromIterator<(VertexId, VertexId)> for Graph {
    fn from_iter<I: IntoIterator<Item = (VertexId, VertexId)>>(iter: I) -> Self {
        let edges: Vec<(VertexId, VertexId)> = iter.into_iter().collect();
        Self::try_from(edges).expect("FromIterator: vertex id overflow or invalid edge")
    }
}

/// Extend a graph by adding edges from an iterator.
///
/// New vertices are automatically created as needed. This enables
/// patterns like `graph.extend(new_edges)`.
///
/// # Panics
///
/// Panics if an edge endpoint exceeds the current vertex count and
/// cannot be added.
///
/// # Examples
///
/// ```
/// use rust_igraph::Graph;
///
/// let mut g = Graph::with_vertices(3);
/// g.extend([(0u32, 1), (1, 2)]);
/// assert_eq!(g.ecount(), 2);
///
/// // Extending with a vertex beyond current count grows the graph
/// g.extend([(2u32, 5)]);
/// assert_eq!(g.vcount(), 6);
/// assert_eq!(g.ecount(), 3);
/// ```
impl Extend<(VertexId, VertexId)> for Graph {
    fn extend<I: IntoIterator<Item = (VertexId, VertexId)>>(&mut self, iter: I) {
        let edges: Vec<(VertexId, VertexId)> = iter.into_iter().collect();
        if edges.is_empty() {
            return;
        }
        let max_id = edges
            .iter()
            .flat_map(|&(u, v)| [u, v])
            .max()
            .expect("non-empty edges");
        if max_id >= self.n {
            self.add_vertices(max_id - self.n + 1)
                .expect("Extend: failed to add vertices");
        }
        self.add_edges(edges).expect("Extend: failed to add edges");
    }
}

/// Structural equality: two graphs are equal if they have the same
/// directedness, same vertex count, and the same sorted edge set.
///
/// This is *not* isomorphism — vertex ids must match exactly.
///
/// # Examples
///
/// ```
/// use rust_igraph::Graph;
///
/// let a = Graph::from_edges(&[(0,1), (1,2)], false, None).unwrap();
/// let b = Graph::from_edges(&[(1,2), (0,1)], false, None).unwrap();
/// assert_eq!(a, b); // same edges, different insertion order
///
/// let c = Graph::from_edges(&[(0,1), (1,2)], true, None).unwrap();
/// assert_ne!(a, c); // different directedness
/// ```
impl PartialEq for Graph {
    fn eq(&self, other: &Self) -> bool {
        if self.directed != other.directed || self.n != other.n || self.ecount() != other.ecount() {
            return false;
        }
        let mut self_edges: Vec<(VertexId, VertexId)> = self.iter().collect();
        let mut other_edges: Vec<(VertexId, VertexId)> = other.iter().collect();
        self_edges.sort_unstable();
        other_edges.sort_unstable();
        self_edges == other_edges
    }
}

impl Eq for Graph {}

/// Hash a graph by its structural content (directedness + vertex count +
/// sorted edge set).
///
/// This is consistent with the [`PartialEq`] impl: structurally equal
/// graphs produce the same hash.
impl std::hash::Hash for Graph {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.directed.hash(state);
        self.n.hash(state);
        let mut edges: Vec<(VertexId, VertexId)> = self.iter().collect();
        edges.sort_unstable();
        edges.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_counts() {
        let g = Graph::with_vertices(0);
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
        assert!(!g.is_directed());
    }

    #[test]
    fn new_directed_flag() {
        let g = Graph::new(3, true).unwrap();
        assert!(g.is_directed());
        let g = Graph::new(3, false).unwrap();
        assert!(!g.is_directed());
    }

    #[test]
    fn add_vertices_then_edges() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 2);
        assert_eq!(g.degree(1).unwrap(), 2);
        let mut nbrs = g.neighbors(1).unwrap();
        nbrs.sort_unstable();
        assert_eq!(nbrs, vec![0, 2]);
    }

    #[test]
    fn out_of_range_vertex_errors() {
        let mut g = Graph::with_vertices(2);
        let err = g.add_edge(0, 5).unwrap_err();
        assert!(matches!(err, IgraphError::VertexOutOfRange { id: 5, n: 2 }));
    }

    #[test]
    fn self_loop_counted_correctly() {
        let mut g = Graph::with_vertices(1);
        g.add_edge(0, 0).unwrap();
        assert_eq!(g.ecount(), 1);
        // Undirected self-loop: appears as both out and in, degree == 2.
        assert_eq!(g.degree(0).unwrap(), 2);
        let mut nbrs = g.neighbors(0).unwrap();
        nbrs.sort_unstable();
        assert_eq!(nbrs, vec![0, 0]);
    }

    #[test]
    fn parallel_edges() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        assert_eq!(g.ecount(), 2);
        assert_eq!(g.degree(0).unwrap(), 2);
        assert_eq!(g.degree(1).unwrap(), 2);
    }

    #[test]
    fn undirected_canonicalisation() {
        // Adding edges (1,0) and (0,1) — both stored canonically as (0,1).
        let mut g = Graph::with_vertices(2);
        g.add_edge(1, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        assert_eq!(g.ecount(), 2);
        // Both vertices see each other as a neighbour twice.
        let mut n0 = g.neighbors(0).unwrap();
        let mut n1 = g.neighbors(1).unwrap();
        n0.sort_unstable();
        n1.sort_unstable();
        assert_eq!(n0, vec![1, 1]);
        assert_eq!(n1, vec![0, 0]);
    }

    #[test]
    fn directed_neighbors_are_outgoing_only() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 0).unwrap();
        // Directed: neighbors(0) returns out-neighbours only.
        assert_eq!(g.neighbors(0).unwrap(), vec![1]);
        // Vertex 2 has out-edge to 0.
        assert_eq!(g.neighbors(2).unwrap(), vec![0]);
        // Vertex 1 has no out-edges.
        assert!(g.neighbors(1).unwrap().is_empty());
        // Degree counts both in and out for directed.
        assert_eq!(g.degree(0).unwrap(), 2); // out: 0->1, in: 2->0
        assert_eq!(g.degree(1).unwrap(), 1); // in: 0->1
        assert_eq!(g.degree(2).unwrap(), 1); // out: 2->0
    }

    #[test]
    fn add_edges_batch_then_rebuild() {
        let mut g = Graph::with_vertices(4);
        g.add_edges(vec![(0, 1), (0, 2), (1, 2), (2, 3)]).unwrap();
        assert_eq!(g.ecount(), 4);
        // Degrees: 0->{1,2} d=2; 1->{0,2} d=2; 2->{0,1,3} d=3; 3->{2} d=1.
        assert_eq!(g.degree(0).unwrap(), 2);
        assert_eq!(g.degree(1).unwrap(), 2);
        assert_eq!(g.degree(2).unwrap(), 3);
        assert_eq!(g.degree(3).unwrap(), 1);
    }

    #[test]
    fn clone_is_deep() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        let g2 = g.clone();
        // Mutate g; g2 must be unaffected.
        g.add_edge(0, 2).unwrap();
        assert_eq!(g.ecount(), 3);
        assert_eq!(g2.ecount(), 2);
    }

    #[test]
    fn os_invariant_is_monotone() {
        let mut g = Graph::with_vertices(5);
        g.add_edges(vec![(0, 1), (0, 2), (3, 4), (1, 2)]).unwrap();
        // os should be non-decreasing and end at ecount.
        for w in g.os.windows(2) {
            assert!(w[0] <= w[1]);
        }
        assert_eq!(g.os[0], 0);
        assert_eq!(*g.os.last().unwrap() as usize, g.ecount());
    }

    #[test]
    fn vertex_out_of_range_when_adding_edge() {
        let mut g = Graph::with_vertices(2);
        let e = g.add_edge(2, 0).unwrap_err();
        assert!(matches!(e, IgraphError::VertexOutOfRange { id: 2, n: 2 }));
        // Graph state must be unchanged after the failed add.
        assert_eq!(g.ecount(), 0);
    }

    // -------- ALGO-CORE-001b: edge-id helpers + incident --------

    #[test]
    fn edge_endpoints_round_trip() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0, 1), (2, 0), (1, 2)]).unwrap();
        // Directed: order preserved. edge_id == position in from/to.
        assert_eq!(g.edge(0).unwrap(), (0, 1));
        assert_eq!(g.edge(1).unwrap(), (2, 0));
        assert_eq!(g.edge(2).unwrap(), (1, 2));
        assert_eq!(g.edge_source(1).unwrap(), 2);
        assert_eq!(g.edge_target(1).unwrap(), 0);
    }

    #[test]
    fn edge_other_endpoint() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 2).unwrap();
        assert_eq!(g.edge_other(0, 0).unwrap(), 2);
        assert_eq!(g.edge_other(0, 2).unwrap(), 0);
        // Vertex not on the edge: error.
        let err = g.edge_other(0, 1).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn edge_out_of_range() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let err = g.edge(5).unwrap_err();
        assert!(matches!(err, IgraphError::EdgeOutOfRange { id: 5, m: 1 }));
    }

    #[test]
    fn incident_returns_edge_ids_matching_neighbors_order() {
        let mut g = Graph::with_vertices(4);
        g.add_edges(vec![(0, 1), (0, 2), (3, 0)]).unwrap();
        let eids = g.incident(0).unwrap();
        // Expect three incident edges; resolving back to neighbours
        // must equal `neighbors(0)` exactly (same iteration order).
        let resolved: Vec<u32> = eids.iter().map(|&e| g.edge_other(e, 0).unwrap()).collect();
        assert_eq!(resolved, g.neighbors(0).unwrap());
    }

    #[test]
    fn incident_self_loop_appears_twice_undirected() {
        let mut g = Graph::with_vertices(1);
        g.add_edge(0, 0).unwrap();
        let eids = g.incident(0).unwrap();
        // Undirected self-loop appears once on the out side and once on
        // the in side — same edge id, twice. Mirrors `neighbors`.
        assert_eq!(eids, vec![0, 0]);
        assert_eq!(g.degree(0).unwrap(), 2);
    }

    #[test]
    fn incident_directed_returns_outgoing_only() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0, 1), (2, 0)]).unwrap();
        // Directed `incident` mirrors directed `neighbors` (out only).
        assert_eq!(g.incident(0).unwrap(), vec![0]);
        assert_eq!(g.incident(2).unwrap(), vec![1]);
        assert!(g.incident(1).unwrap().is_empty());
    }

    #[test]
    fn get_eid_undirected_finds_edge_either_way() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap(); // edge 0
        g.add_edge(1, 2).unwrap(); // edge 1
        assert_eq!(g.get_eid(0, 1).unwrap(), 0);
        assert_eq!(g.get_eid(1, 0).unwrap(), 0);
        assert_eq!(g.get_eid(1, 2).unwrap(), 1);
        assert_eq!(g.get_eid(2, 1).unwrap(), 1);
    }

    #[test]
    fn get_eid_directed_respects_direction() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap(); // edge 0
        assert_eq!(g.get_eid(0, 1).unwrap(), 0);
        assert!(g.get_eid(1, 0).is_err()); // reverse direction has no edge
    }

    #[test]
    fn find_eid_returns_none_for_missing() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        assert_eq!(g.find_eid(0, 2).unwrap(), None);
        assert!(g.find_eid(0, 99).is_err()); // out-of-range vertex
    }

    #[test]
    fn get_eid_self_loop() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap(); // self-loop, edge 0
        g.add_edge(0, 1).unwrap(); // edge 1
        assert_eq!(g.get_eid(0, 0).unwrap(), 0);
        assert_eq!(g.get_eid(0, 1).unwrap(), 1);
    }

    #[test]
    fn get_all_eids_between_returns_all_parallel() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap(); // edge 0
        g.add_edge(0, 1).unwrap(); // edge 1
        g.add_edge(0, 1).unwrap(); // edge 2
        let eids = g.get_all_eids_between(0, 1).unwrap();
        assert_eq!(eids, vec![0, 1, 2]);
        // Reverse direction yields the same edges on undirected.
        let eids = g.get_all_eids_between(1, 0).unwrap();
        assert_eq!(eids, vec![0, 1, 2]);
    }

    #[test]
    fn get_all_eids_between_directed_one_way_only() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap(); // edge 0
        g.add_edge(0, 1).unwrap(); // edge 1 (parallel)
        assert_eq!(g.get_all_eids_between(0, 1).unwrap(), vec![0, 1]);
        // Reverse direction has no edges in directed graph.
        assert_eq!(g.get_all_eids_between(1, 0).unwrap(), Vec::<EdgeId>::new());
    }

    #[test]
    fn get_eid_returns_lowest_id_for_parallel() {
        // Spec: with multiple edges, get_eid always returns the same
        // edge id (matches upstream's "ignored multi-edges" guarantee).
        // Our impl returns the lowest from the bucket.
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap(); // edge 0
        g.add_edge(0, 1).unwrap(); // edge 1
        assert_eq!(g.get_eid(0, 1).unwrap(), 0);
    }

    // -------- ALGO-CORE-001c: delete_edges + delete_vertices --------

    #[test]
    fn delete_edges_empty_input_is_noop() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        g.delete_edges(&[]).unwrap();
        assert_eq!(g.ecount(), 2);
        assert_eq!(g.degree(1).unwrap(), 2);
    }

    #[test]
    fn delete_edges_single_edge_undirected() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0, 1), (1, 2), (0, 2)]).unwrap();
        // Remove edge id 1 (the (1,2) edge).
        g.delete_edges(&[1]).unwrap();
        assert_eq!(g.ecount(), 2);
        // Surviving edges renumbered to 0,1: (0,1) and (0,2).
        assert_eq!(g.find_eid(0, 1).unwrap(), Some(0));
        assert_eq!(g.find_eid(0, 2).unwrap(), Some(1));
        assert_eq!(g.find_eid(1, 2).unwrap(), None);
        // Degrees consistent post-rebuild.
        assert_eq!(g.degree(1).unwrap(), 1);
        assert_eq!(g.degree(2).unwrap(), 1);
    }

    #[test]
    fn delete_edges_duplicate_ids_tolerated() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        g.delete_edges(&[0, 0, 0]).unwrap();
        assert_eq!(g.ecount(), 1);
        assert_eq!(g.find_eid(1, 2).unwrap(), Some(0));
    }

    #[test]
    fn delete_edges_all_edges_leaves_isolated_vertices() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        g.delete_edges(&[0, 1]).unwrap();
        assert_eq!(g.ecount(), 0);
        assert_eq!(g.vcount(), 3);
        for v in 0..3 {
            assert_eq!(g.degree(v).unwrap(), 0);
        }
    }

    #[test]
    fn delete_edges_out_of_range_errors_and_preserves_state() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        let err = g.delete_edges(&[5]).unwrap_err();
        assert!(matches!(err, IgraphError::EdgeOutOfRange { id: 5, m: 2 }));
        // Graph unchanged.
        assert_eq!(g.ecount(), 2);
    }

    #[test]
    fn delete_edges_self_loop_directed() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edges(vec![(0, 0), (0, 1)]).unwrap();
        g.delete_edges(&[0]).unwrap(); // remove the self-loop
        assert_eq!(g.ecount(), 1);
        assert_eq!(g.degree(0).unwrap(), 1);
        assert_eq!(g.find_eid(0, 1).unwrap(), Some(0));
    }

    #[test]
    fn delete_vertices_empty_input_is_noop() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        g.delete_vertices(&[]).unwrap();
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 2);
    }

    #[test]
    fn delete_vertices_single_renumbers() {
        let mut g = Graph::with_vertices(4);
        g.add_edges(vec![(0, 1), (1, 2), (2, 3), (0, 3)]).unwrap();
        // Remove vertex 1: edges (0,1) and (1,2) go with it. (2,3),(0,3)
        // survive but get renumbered: 2 → 1, 3 → 2.
        g.delete_vertices(&[1]).unwrap();
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 2);
        // (2,3) → (1,2); (0,3) → (0,2).
        assert!(g.find_eid(1, 2).unwrap().is_some());
        assert!(g.find_eid(0, 2).unwrap().is_some());
        assert_eq!(g.find_eid(0, 1).unwrap(), None);
    }

    #[test]
    fn delete_vertices_duplicate_ids_tolerated() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        g.delete_vertices(&[1, 1, 1]).unwrap();
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn delete_vertices_all_yields_empty_graph() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        g.delete_vertices(&[0, 1, 2]).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn delete_vertices_out_of_range_errors_and_preserves_state() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        let err = g.delete_vertices(&[5]).unwrap_err();
        assert!(matches!(err, IgraphError::VertexOutOfRange { id: 5, n: 3 }));
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 2);
    }

    #[test]
    fn delete_vertices_map_returns_correct_mappings() {
        let mut g = Graph::with_vertices(5);
        g.add_edges(vec![(0, 1), (1, 2), (2, 3), (3, 4)]).unwrap();
        let (map, invmap) = g.delete_vertices_map(&[1, 3]).unwrap();
        // Removed: 1 and 3. Retained: 0 → 0, 2 → 1, 4 → 2.
        assert_eq!(map, vec![Some(0), None, Some(1), None, Some(2)]);
        assert_eq!(invmap, vec![0, 2, 4]);
        assert_eq!(g.vcount(), 3);
        // Only edges between retained vertices survive — none do here.
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn delete_vertices_directed_preserves_direction() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 0), (3, 0)]).unwrap();
        g.delete_vertices(&[3]).unwrap();
        assert_eq!(g.vcount(), 3);
        assert!(g.is_directed());
        // Surviving directed edges (3 → 0) gone; (0,1),(1,2),(2,0) keep direction.
        assert!(g.get_eid(0, 1).is_ok());
        assert!(g.get_eid(1, 0).is_err()); // wrong direction
    }

    #[test]
    fn delete_vertices_self_loop_on_removed_vertex() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0, 0), (0, 1), (1, 2)]).unwrap();
        g.delete_vertices(&[0]).unwrap();
        // Self-loop and edges to vertex 0 gone; only (1,2) → (0,1) survives.
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 1);
        assert!(g.find_eid(0, 1).unwrap().is_some());
    }

    #[test]
    fn delete_vertices_preserves_parallel_edges() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0, 1), (0, 1), (1, 2)]).unwrap();
        g.delete_vertices(&[2]).unwrap();
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 2); // both parallel (0,1) edges retained
        assert_eq!(g.degree(0).unwrap(), 2);
        assert_eq!(g.degree(1).unwrap(), 2);
    }

    #[test]
    fn add_edges_after_delete_works() {
        let mut g = Graph::with_vertices(4);
        g.add_edges(vec![(0, 1), (1, 2), (2, 3)]).unwrap();
        g.delete_vertices(&[0]).unwrap(); // now n=3, vertices 0,1,2
        // Add a new edge and check indexes still work.
        g.add_edge(0, 2).unwrap();
        assert_eq!(g.ecount(), 3);
        assert_eq!(g.degree(0).unwrap(), 2); // (0,1)+(0,2)
        assert!(g.find_eid(0, 2).unwrap().is_some());
    }

    #[test]
    fn from_adjacency_matrix_undirected_triangle() {
        let adj = vec![
            vec![0.0, 1.0, 1.0],
            vec![1.0, 0.0, 1.0],
            vec![1.0, 1.0, 0.0],
        ];
        let g = Graph::from_adjacency_matrix(&adj, false).unwrap();
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 3);
        assert!(!g.is_directed());
    }

    #[test]
    fn from_adjacency_matrix_directed() {
        let adj = vec![
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 1.0],
            vec![1.0, 0.0, 0.0],
        ];
        let g = Graph::from_adjacency_matrix(&adj, true).unwrap();
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 3);
        assert!(g.is_directed());
    }

    #[test]
    fn from_adjacency_matrix_with_self_loop() {
        let adj = vec![vec![1.0, 1.0], vec![1.0, 0.0]];
        let g = Graph::from_adjacency_matrix(&adj, false).unwrap();
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 2); // self-loop on 0 + edge 0-1
    }

    #[test]
    fn from_adjacency_matrix_empty() {
        let adj: Vec<Vec<f64>> = Vec::new();
        let g = Graph::from_adjacency_matrix(&adj, false).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn from_adjacency_matrix_non_square_error() {
        let adj = vec![vec![0.0, 1.0], vec![1.0, 0.0, 1.0]];
        assert!(Graph::from_adjacency_matrix(&adj, false).is_err());
    }

    #[test]
    fn from_adjacency_matrix_weighted_basic() {
        let adj = vec![
            vec![0.0, 2.5, 0.0],
            vec![2.5, 0.0, 1.0],
            vec![0.0, 1.0, 0.0],
        ];
        let (g, weights) = Graph::from_adjacency_matrix_weighted(&adj, false).unwrap();
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 2);
        assert_eq!(weights.len(), 2);
        assert!((weights[0] - 2.5).abs() < 1e-10);
        assert!((weights[1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn from_adjacency_matrix_weighted_directed() {
        let adj = vec![
            vec![0.0, 3.0, 0.0],
            vec![0.0, 0.0, 2.0],
            vec![1.5, 0.0, 0.0],
        ];
        let (g, weights) = Graph::from_adjacency_matrix_weighted(&adj, true).unwrap();
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 3);
        assert_eq!(weights.len(), 3);
        assert!((weights[0] - 3.0).abs() < 1e-10);
        assert!((weights[1] - 2.0).abs() < 1e-10);
        assert!((weights[2] - 1.5).abs() < 1e-10);
    }

    #[test]
    fn from_adjacency_matrix_multi_edges() {
        let adj = vec![vec![0.0, 3.0], vec![3.0, 0.0]];
        let g = Graph::from_adjacency_matrix(&adj, false).unwrap();
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 3); // 3 parallel edges
    }

    #[test]
    fn from_adjacency_list_undirected_triangle() {
        let adj = vec![vec![1, 2], vec![0, 2], vec![0, 1]];
        let g = Graph::from_adjacency_list(&adj, false).unwrap();
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 3);
        assert!(!g.is_directed());
    }

    #[test]
    fn from_adjacency_list_directed() {
        let adj = vec![vec![1, 2], vec![2], vec![]];
        let g = Graph::from_adjacency_list(&adj, true).unwrap();
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 3);
        assert!(g.is_directed());
    }

    #[test]
    fn from_adjacency_list_empty() {
        let adj: Vec<Vec<u32>> = Vec::new();
        let g = Graph::from_adjacency_list(&adj, false).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn from_adjacency_list_isolated_vertices() {
        let adj = vec![vec![], vec![], vec![]];
        let g = Graph::from_adjacency_list(&adj, false).unwrap();
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn from_adjacency_list_self_loop() {
        let adj = vec![vec![0, 1], vec![0]];
        let g = Graph::from_adjacency_list(&adj, false).unwrap();
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 2); // self-loop on 0 + edge 0-1
    }

    #[test]
    fn from_adjacency_list_out_of_range_error() {
        let adj = vec![vec![5]]; // only 1 vertex but references vertex 5
        assert!(Graph::from_adjacency_list(&adj, false).is_err());
    }

    #[test]
    fn neighbors_iter_matches_neighbors_undirected() {
        let g = Graph::from_edges(&[(0, 1), (0, 2), (1, 3), (2, 3), (3, 4)], false, None).unwrap();
        for v in 0..g.vcount() {
            let from_vec = g.neighbors(v).unwrap();
            let from_iter: Vec<VertexId> = g.neighbors_iter(v).unwrap().collect();
            assert_eq!(from_vec, from_iter, "mismatch at vertex {v}");
        }
    }

    #[test]
    fn neighbors_iter_matches_neighbors_directed() {
        let g = Graph::from_edges(&[(0, 1), (0, 2), (1, 3), (2, 3), (3, 4)], true, None).unwrap();
        for v in 0..g.vcount() {
            let from_vec = g.neighbors(v).unwrap();
            let from_iter: Vec<VertexId> = g.neighbors_iter(v).unwrap().collect();
            assert_eq!(from_vec, from_iter, "mismatch at vertex {v}");
        }
    }

    #[test]
    fn neighbors_iter_exact_size() {
        let g = Graph::from_edges(&[(0, 1), (0, 2), (0, 3)], false, None).unwrap();
        let iter = g.neighbors_iter(0).unwrap();
        assert_eq!(iter.len(), 3);
    }

    #[test]
    fn neighbors_iter_invalid_vertex() {
        let g = Graph::with_vertices(3);
        assert!(g.neighbors_iter(5).is_err());
    }
}

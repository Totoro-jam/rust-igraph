//! Fluent graph construction via the builder pattern.

use super::error::{IgraphError, IgraphResult};
use super::graph::{Graph, VertexId};

/// A fluent builder for constructing [`Graph`] instances.
///
/// `GraphBuilder` accumulates vertices and edges, then produces a
/// [`Graph`] in a single `build()` call. It validates structure only at
/// build time, so intermediate states need not be valid graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::GraphBuilder;
///
/// // Build a directed triangle.
/// let g = GraphBuilder::directed()
///     .vertices(3)
///     .edge(0, 1)
///     .edge(1, 2)
///     .edge(2, 0)
///     .build()
///     .unwrap();
/// assert_eq!(g.vcount(), 3);
/// assert_eq!(g.ecount(), 3);
/// assert!(g.is_directed());
/// ```
///
/// ```
/// use rust_igraph::GraphBuilder;
///
/// // Build from an edge list (vertex count inferred).
/// let g = GraphBuilder::undirected()
///     .edges(&[(0, 1), (1, 2), (2, 3), (3, 0)])
///     .build()
///     .unwrap();
/// assert_eq!(g.vcount(), 4);
/// assert_eq!(g.ecount(), 4);
/// ```
#[derive(Debug, Clone)]
pub struct GraphBuilder {
    directed: bool,
    n: Option<u32>,
    edge_list: Vec<(VertexId, VertexId)>,
}

impl GraphBuilder {
    /// Start building a directed graph.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::GraphBuilder;
    ///
    /// let g = GraphBuilder::directed()
    ///     .vertices(2)
    ///     .edge(0, 1)
    ///     .build()
    ///     .unwrap();
    /// assert!(g.is_directed());
    /// ```
    #[must_use]
    pub fn directed() -> Self {
        Self {
            directed: true,
            n: None,
            edge_list: Vec::new(),
        }
    }

    /// Start building an undirected graph.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::GraphBuilder;
    ///
    /// let g = GraphBuilder::undirected()
    ///     .vertices(5)
    ///     .build()
    ///     .unwrap();
    /// assert!(!g.is_directed());
    /// assert_eq!(g.vcount(), 5);
    /// ```
    #[must_use]
    pub fn undirected() -> Self {
        Self {
            directed: false,
            n: None,
            edge_list: Vec::new(),
        }
    }

    /// Set the vertex count explicitly.
    ///
    /// If not called, vertex count is inferred from the maximum vertex id
    /// in any added edge (plus one). Calling this with a value *smaller*
    /// than the inferred count causes `build()` to return an error.
    #[must_use]
    pub fn vertices(mut self, n: u32) -> Self {
        self.n = Some(n);
        self
    }

    /// Add a single edge.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::GraphBuilder;
    ///
    /// let g = GraphBuilder::undirected()
    ///     .edge(0, 1)
    ///     .edge(1, 2)
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(g.ecount(), 2);
    /// ```
    #[must_use]
    pub fn edge(mut self, from: VertexId, to: VertexId) -> Self {
        self.edge_list.push((from, to));
        self
    }

    /// Add multiple edges from a slice of `(from, to)` pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::GraphBuilder;
    ///
    /// let g = GraphBuilder::undirected()
    ///     .edges(&[(0, 1), (1, 2), (2, 0)])
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(g.ecount(), 3);
    /// ```
    #[must_use]
    pub fn edges(mut self, pairs: &[(VertexId, VertexId)]) -> Self {
        self.edge_list.extend_from_slice(pairs);
        self
    }

    /// Add a chain of edges forming a path: `v0 -> v1 -> v2 -> ... -> vk`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::GraphBuilder;
    ///
    /// let g = GraphBuilder::undirected()
    ///     .path(&[0, 1, 2, 3, 4])
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(g.ecount(), 4);
    /// ```
    #[must_use]
    pub fn path(mut self, vertices: &[VertexId]) -> Self {
        for w in vertices.windows(2) {
            self.edge_list.push((w[0], w[1]));
        }
        self
    }

    /// Add edges forming a cycle: `v0 -> v1 -> ... -> vk -> v0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::GraphBuilder;
    ///
    /// let g = GraphBuilder::undirected()
    ///     .cycle(&[0, 1, 2, 3])
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(g.ecount(), 4);
    /// assert!(g.has_edge(3, 0));
    /// ```
    #[must_use]
    pub fn cycle(mut self, vertices: &[VertexId]) -> Self {
        for w in vertices.windows(2) {
            self.edge_list.push((w[0], w[1]));
        }
        if vertices.len() >= 2 {
            self.edge_list
                .push((vertices[vertices.len() - 1], vertices[0]));
        }
        self
    }

    /// Add edges forming a complete subgraph (clique) on the given vertices.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::GraphBuilder;
    ///
    /// let g = GraphBuilder::undirected()
    ///     .clique(&[0, 1, 2, 3])
    ///     .build()
    ///     .unwrap();
    /// // K_4 has 4*3/2 = 6 edges.
    /// assert_eq!(g.ecount(), 6);
    /// ```
    #[must_use]
    pub fn clique(mut self, vertices: &[VertexId]) -> Self {
        for (i, &u) in vertices.iter().enumerate() {
            for &v in &vertices[i + 1..] {
                self.edge_list.push((u, v));
            }
        }
        self
    }

    /// Add edges connecting a center vertex to all given leaf vertices (star).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::GraphBuilder;
    ///
    /// let g = GraphBuilder::undirected()
    ///     .star(0, &[1, 2, 3, 4])
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(g.ecount(), 4);
    /// ```
    #[must_use]
    pub fn star(mut self, center: VertexId, leaves: &[VertexId]) -> Self {
        for &leaf in leaves {
            self.edge_list.push((center, leaf));
        }
        self
    }

    /// Consume the builder and produce a [`Graph`].
    ///
    /// # Errors
    ///
    /// Returns an error if the explicit vertex count is smaller than
    /// required by the edges.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::GraphBuilder;
    ///
    /// // Error: vertex count too small for edge (0, 5).
    /// let r = GraphBuilder::undirected()
    ///     .vertices(3)
    ///     .edge(0, 5)
    ///     .build();
    /// assert!(r.is_err());
    /// ```
    pub fn build(self) -> IgraphResult<Graph> {
        let inferred = match self.edge_list.iter().flat_map(|&(u, v)| [u, v]).max() {
            Some(m) => m
                .checked_add(1)
                .ok_or_else(|| IgraphError::InvalidArgument("vertex id overflow".to_owned()))?,
            None => 0,
        };

        let n = match self.n {
            Some(explicit) => {
                if explicit < inferred {
                    return Err(IgraphError::InvalidArgument(format!(
                        "vertex count {explicit} is too small for edges referencing vertex {}",
                        inferred - 1
                    )));
                }
                explicit
            }
            None => inferred,
        };

        let mut g = Graph::new(n, self.directed)?;
        g.add_edges(self.edge_list)?;
        Ok(g)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directed_triangle() {
        let g = GraphBuilder::directed()
            .vertices(3)
            .edge(0, 1)
            .edge(1, 2)
            .edge(2, 0)
            .build()
            .expect("build");
        assert!(g.is_directed());
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 3);
    }

    #[test]
    fn undirected_inferred_vertices() {
        let g = GraphBuilder::undirected()
            .edges(&[(0, 3), (1, 4)])
            .build()
            .expect("build");
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 2);
    }

    #[test]
    fn empty_graph() {
        let g = GraphBuilder::undirected()
            .vertices(10)
            .build()
            .expect("build");
        assert_eq!(g.vcount(), 10);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn zero_vertices_no_edges() {
        let g = GraphBuilder::undirected().build().expect("build");
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn path_helper() {
        let g = GraphBuilder::undirected()
            .path(&[0, 1, 2, 3])
            .build()
            .expect("build");
        assert_eq!(g.ecount(), 3);
        assert!(g.has_edge(0, 1));
        assert!(g.has_edge(2, 3));
    }

    #[test]
    fn cycle_helper() {
        let g = GraphBuilder::undirected()
            .cycle(&[0, 1, 2, 3, 4])
            .build()
            .expect("build");
        assert_eq!(g.ecount(), 5);
        assert!(g.has_edge(4, 0));
    }

    #[test]
    fn clique_helper() {
        let g = GraphBuilder::undirected()
            .clique(&[0, 1, 2, 3])
            .build()
            .expect("build");
        assert_eq!(g.ecount(), 6);
    }

    #[test]
    fn star_helper() {
        let g = GraphBuilder::undirected()
            .star(0, &[1, 2, 3])
            .build()
            .expect("build");
        assert_eq!(g.ecount(), 3);
        assert!(g.has_edge(0, 1));
        assert!(g.has_edge(0, 3));
    }

    #[test]
    fn vertex_count_too_small_errors() {
        let r = GraphBuilder::undirected().vertices(2).edge(0, 5).build();
        assert!(r.is_err());
    }

    #[test]
    fn chaining_multiple_patterns() {
        let g = GraphBuilder::undirected()
            .clique(&[0, 1, 2])
            .star(3, &[0, 1, 2])
            .build()
            .expect("build");
        assert_eq!(g.vcount(), 4);
        assert_eq!(g.ecount(), 6);
    }
}

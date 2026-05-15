//! Minimal `Graph` for the Phase 0 walking skeleton.
//!
//! Only undirected, unweighted graphs with `u32` vertex ids. Adjacency stored as
//! `Vec<Vec<u32>>`. This will be replaced by the full `igraph_t`-equivalent
//! structure in Phase 1 (see `ALGO-CORE-001`); the public surface kept here is
//! the strict subset BFS / EdgeList / oracle tests need to compile.

use crate::error::{IgraphError, IgraphResult};

/// Vertex id. Phase 0 fixes this to `u32`; Phase 1 may switch to a generic
/// integer type behind a feature flag. See plan §3.3.
pub type VertexId = u32;

/// Undirected, unweighted graph backed by an adjacency list.
#[derive(Debug, Clone, Default)]
pub struct Graph {
    n: u32,
    out_neighbors: Vec<Vec<VertexId>>,
    edge_count: usize,
}

impl Graph {
    /// Construct an empty graph on `n` vertices and no edges.
    pub fn with_vertices(n: u32) -> Self {
        Self {
            n,
            out_neighbors: vec![Vec::new(); n as usize],
            edge_count: 0,
        }
    }

    /// Number of vertices. Counterpart of `igraph_vcount()`.
    pub fn vcount(&self) -> u32 {
        self.n
    }

    /// Number of edges. Counterpart of `igraph_ecount()`.
    pub fn ecount(&self) -> usize {
        self.edge_count
    }

    /// Append `count` new isolated vertices, returning the id range
    /// `(first, last_inclusive)` of the new vertices.
    pub fn add_vertices(&mut self, count: u32) -> (VertexId, VertexId) {
        let first = self.n;
        self.n = self.n.checked_add(count).expect("vertex count overflow");
        self.out_neighbors.resize(self.n as usize, Vec::new());
        (first, self.n.saturating_sub(1))
    }

    /// Add a single undirected edge between `u` and `v`.
    ///
    /// Self-loops are allowed (added once); parallel edges are allowed.
    pub fn add_edge(&mut self, u: VertexId, v: VertexId) -> IgraphResult<()> {
        self.check_vertex(u)?;
        self.check_vertex(v)?;
        self.out_neighbors[u as usize].push(v);
        if u != v {
            self.out_neighbors[v as usize].push(u);
        }
        self.edge_count += 1;
        Ok(())
    }

    /// Add a sequence of undirected edges.
    pub fn add_edges<I>(&mut self, edges: I) -> IgraphResult<()>
    where
        I: IntoIterator<Item = (VertexId, VertexId)>,
    {
        for (u, v) in edges {
            self.add_edge(u, v)?;
        }
        Ok(())
    }

    /// Neighbors of `v`. Order is insertion order.
    pub fn neighbors(&self, v: VertexId) -> IgraphResult<&[VertexId]> {
        self.check_vertex(v)?;
        Ok(&self.out_neighbors[v as usize])
    }

    /// Degree of `v` (counting self-loops once each, parallel edges separately).
    pub fn degree(&self, v: VertexId) -> IgraphResult<usize> {
        self.check_vertex(v)?;
        Ok(self.out_neighbors[v as usize].len())
    }

    fn check_vertex(&self, v: VertexId) -> IgraphResult<()> {
        if v >= self.n {
            return Err(IgraphError::VertexOutOfRange { id: v, n: self.n });
        }
        Ok(())
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
    }

    #[test]
    fn add_vertices_then_edges() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 2);
        assert_eq!(g.degree(1).unwrap(), 2);
        assert_eq!(g.neighbors(1).unwrap(), &[0, 2]);
    }

    #[test]
    fn out_of_range_vertex_errors() {
        let mut g = Graph::with_vertices(2);
        let err = g.add_edge(0, 5).unwrap_err();
        assert!(matches!(err, IgraphError::VertexOutOfRange { id: 5, n: 2 }));
    }

    #[test]
    fn self_loop_counted_once_per_endpoint() {
        let mut g = Graph::with_vertices(1);
        g.add_edge(0, 0).unwrap();
        assert_eq!(g.ecount(), 1);
        assert_eq!(g.neighbors(0).unwrap(), &[0]);
    }
}

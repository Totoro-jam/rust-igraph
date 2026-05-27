//! `GraphDB` binary format reader (ALGO-IO-008).
//!
//! Reads graphs in the binary format used by the ARG Graph Database
//! for isomorphism testing. This is a read-only format.
//!
//! The file is composed of 16-bit little-endian words. The first word
//! is the number of vertices. Then for each vertex, the file contains
//! a word giving the number of outgoing edges, followed by a word for
//! each destination vertex (0-based).
//!
//! Reference: M. De Santo, P. Foggia, C. Sansone, M. Vento (2003).
//! A large database of graphs and its use for benchmarking graph
//! isomorphism algorithms. Pattern Recognition Letters, 24(8).
//!
//! Counterpart of `igraph_read_graph_graphdb`.

use std::io::Read;

use crate::core::{Graph, IgraphError, IgraphResult};

/// Read a graph from `GraphDB` binary format.
///
/// The format stores adjacency lists in packed little-endian 16-bit
/// words. Creates a directed or undirected graph depending on the
/// `directed` parameter.
///
/// # Examples
///
/// ```
/// use rust_igraph::read_graphdb;
///
/// // 2 vertices; vertex 0 has 1 edge to vertex 1; vertex 1 has 0 edges
/// let data: Vec<u8> = vec![
///     2, 0,  // n_vertices = 2
///     1, 0,  // vertex 0: 1 neighbor
///     1, 0,  // neighbor: vertex 1
///     0, 0,  // vertex 1: 0 neighbors
/// ];
/// let g = read_graphdb(&data[..], true).unwrap();
/// assert_eq!(g.vcount(), 2);
/// assert_eq!(g.ecount(), 1);
/// assert!(g.is_directed());
/// ```
pub fn read_graphdb<R: Read>(mut input: R, directed: bool) -> IgraphResult<Graph> {
    let n_vertices = read_u16_le(&mut input, "vertex count")?;

    let mut edges: Vec<(u32, u32)> = Vec::new();

    for v in 0..u32::from(n_vertices) {
        let degree = read_u16_le(&mut input, "edge count")?;

        for _ in 0..degree {
            let target = read_u16_le(&mut input, "target vertex")?;
            let target_u32 = u32::from(target);

            if target_u32 >= u32::from(n_vertices) {
                return Err(IgraphError::Parse {
                    line: 0,
                    message: format!(
                        "invalid vertex ID {target_u32} in graphdb file (n_vertices = {n_vertices})"
                    ),
                });
            }

            edges.push((v, target_u32));
        }
    }

    let mut graph = Graph::new(u32::from(n_vertices), directed)?;
    graph.add_edges(edges)?;

    Ok(graph)
}

fn read_u16_le<R: Read>(reader: &mut R, context: &str) -> IgraphResult<u16> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf).map_err(|e| {
        if e.kind() == std::io::ErrorKind::UnexpectedEof {
            IgraphError::Parse {
                line: 0,
                message: format!("unexpected end of file while reading {context}"),
            }
        } else {
            IgraphError::Io(e)
        }
    })?;
    Ok(u16::from_le_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_graphdb(n: u16, adj: &[Vec<u16>]) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&n.to_le_bytes());
        for neighbors in adj {
            #[allow(clippy::cast_possible_truncation)]
            let deg = neighbors.len() as u16;
            data.extend_from_slice(&deg.to_le_bytes());
            for &tgt in neighbors {
                data.extend_from_slice(&tgt.to_le_bytes());
            }
        }
        data
    }

    #[test]
    fn test_empty_graph() {
        let data = make_graphdb(0, &[]);
        let g = read_graphdb(&data[..], false).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn test_single_vertex_no_edges() {
        let data = make_graphdb(1, &[vec![]]);
        let g = read_graphdb(&data[..], false).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn test_directed_triangle() {
        let data = make_graphdb(3, &[vec![1], vec![2], vec![0]]);
        let g = read_graphdb(&data[..], true).unwrap();
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 3);
        assert!(g.is_directed());
    }

    #[test]
    fn test_undirected() {
        let data = make_graphdb(3, &[vec![1, 2], vec![0, 2], vec![0, 1]]);
        let g = read_graphdb(&data[..], false).unwrap();
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 6);
        assert!(!g.is_directed());
    }

    #[test]
    fn test_self_loop() {
        let data = make_graphdb(2, &[vec![0, 1], vec![]]);
        let g = read_graphdb(&data[..], true).unwrap();
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 2);
    }

    #[test]
    fn test_invalid_vertex_id() {
        let data = make_graphdb(2, &[vec![5], vec![]]);
        let result = read_graphdb(&data[..], true);
        assert!(result.is_err());
    }

    #[test]
    fn test_truncated_header() {
        let data = [2u8]; // Only 1 byte, need 2
        let result = read_graphdb(&data[..], true);
        assert!(result.is_err());
    }

    #[test]
    fn test_truncated_adjacency() {
        // 2 vertices, vertex 0 says 1 neighbor but no data follows
        let mut data = Vec::new();
        data.extend_from_slice(&2u16.to_le_bytes());
        data.extend_from_slice(&1u16.to_le_bytes());
        let result = read_graphdb(&data[..], true);
        assert!(result.is_err());
    }

    #[test]
    fn test_large_graph() {
        // 256 vertex complete-ish graph is too big to enumerate,
        // just test that 10 vertices with chain edges works
        let adj: Vec<Vec<u16>> = (0..10u16)
            .map(|v| if v < 9 { vec![v + 1] } else { vec![] })
            .collect();
        let data = make_graphdb(10, &adj);
        let g = read_graphdb(&data[..], true).unwrap();
        assert_eq!(g.vcount(), 10);
        assert_eq!(g.ecount(), 9);
    }

    #[test]
    fn test_multiple_edges_from_vertex() {
        // Star graph: vertex 0 connects to all others
        let data = make_graphdb(5, &[vec![1, 2, 3, 4], vec![], vec![], vec![], vec![]]);
        let g = read_graphdb(&data[..], true).unwrap();
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 4);
    }
}

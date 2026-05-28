//! Isomorphism class functions (ALGO-MO-004).
//!
//! Counterpart of `igraph_isoclass`, `igraph_isoclass_subgraph`, and
//! `igraph_isoclass_create` in `references/igraph/src/isomorphism/isoclasses.c`.
//!
//! Classifies small graphs into isomorphism classes via precomputed lookup
//! tables. Two graphs have the same isoclass iff they are isomorphic.
//!
//! Supported sizes:
//! - Directed: 3 and 4 vertices (16 and 218 isoclasses respectively)
//! - Undirected: 3, 4, and 5 vertices (4, 11, and 34 isoclasses)

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

mod tables;

/// Determines the isomorphism class of a graph.
///
/// The graph must have exactly 3 or 4 vertices (directed), or 3, 4, or 5
/// vertices (undirected). Multi-edges and self-loops are ignored.
///
/// Class 0 is always the empty graph; the highest class is the complete graph.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, isoclass};
///
/// // Empty directed graph on 3 vertices → class 0
/// let g = Graph::new(3, true).unwrap();
/// assert_eq!(isoclass(&g).unwrap(), 0);
///
/// // Complete undirected graph on 3 vertices (triangle) → class 3
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(0, 2).unwrap();
/// assert_eq!(isoclass(&g).unwrap(), 3);
/// ```
pub fn isoclass(graph: &Graph) -> IgraphResult<u32> {
    let n = graph.vcount();
    let directed = graph.is_directed();

    let (arr_idx, arr_code, mul) = get_tables(n, directed)?;

    let mut code: u32 = 0;
    let ecount = graph.ecount();
    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (src, tgt) = graph.edge(eid as u32)?;
        if src == tgt {
            continue;
        }
        let idx = mul * src + tgt;
        code |= arr_idx[idx as usize];
    }

    Ok(arr_code[code as usize].into())
}

/// Determines the isomorphism class of a subgraph induced by the given vertices.
///
/// `vids` must contain 3 or 4 distinct vertex IDs (directed), or 3, 4, or 5
/// distinct vertex IDs (undirected). Each vertex must exist in the graph.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, isoclass_subgraph};
///
/// // Build a 5-vertex graph with a triangle on vertices 0,1,2
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(3, 4).unwrap();
///
/// // Subgraph {0,1,2} is a triangle → class 3 for undirected 3-vertex
/// assert_eq!(isoclass_subgraph(&g, &[0, 1, 2]).unwrap(), 3);
/// // Subgraph {0,3,4} has one edge (3-4) → class 1
/// assert_eq!(isoclass_subgraph(&g, &[0, 3, 4]).unwrap(), 1);
/// ```
pub fn isoclass_subgraph(graph: &Graph, vids: &[VertexId]) -> IgraphResult<u32> {
    let subgraph_size = u32::try_from(vids.len())
        .map_err(|_| IgraphError::InvalidArgument("isoclass_subgraph: too many vertices".into()))?;
    let directed = graph.is_directed();

    let (arr_idx, arr_code, mul) = get_tables(subgraph_size, directed)?;

    let n = graph.vcount();
    for &v in vids {
        if v >= n {
            return Err(IgraphError::InvalidArgument(format!(
                "isoclass_subgraph: vertex {v} out of range (graph has {n} vertices)"
            )));
        }
    }

    let mut code: u32 = 0;

    for (i, &from) in vids.iter().enumerate() {
        let ecount = graph.ecount();
        for eid in 0..ecount {
            #[allow(clippy::cast_possible_truncation)]
            let (src, tgt) = graph.edge(eid as u32)?;
            if src != from || src == tgt {
                continue;
            }
            if let Some(to_pos) = vids.iter().position(|&v| v == tgt) {
                let idx = (mul as usize) * i + to_pos;
                code |= arr_idx[idx];
            }
        }
    }

    Ok(arr_code[code as usize].into())
}

/// Creates the canonical representative graph of the given isomorphism class.
///
/// The isomorphism class number must be in `[0, graph_count(size, directed) - 1]`.
///
/// Supports directed 3-4 and undirected 3-5 vertex graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{isoclass, isoclass_create};
///
/// // Create directed 3-vertex graph of class 5
/// let g = isoclass_create(3, 5, true).unwrap();
/// assert_eq!(g.vcount(), 3);
/// assert!(g.is_directed());
/// assert_eq!(isoclass(&g).unwrap(), 5);
///
/// // Create undirected 4-vertex graph of class 0 (empty graph)
/// let g = isoclass_create(4, 0, false).unwrap();
/// assert_eq!(g.ecount(), 0);
/// ```
pub fn isoclass_create(size: u32, number: u32, directed: bool) -> IgraphResult<Graph> {
    let (isographs, classedges, power) = get_create_tables(size, directed)?;

    let graph_count = isographs.len();
    if number as usize >= graph_count {
        return Err(IgraphError::InvalidArgument(format!(
            "isoclass_create: class {number} out of range (only {graph_count} \
             {} graphs on {size} vertices)",
            if directed { "directed" } else { "undirected" }
        )));
    }

    let mut code = isographs[number as usize];
    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();
    let mut current_power = power;
    let mut pos: usize = 0;

    while code > 0 {
        if code >= current_power {
            let src = VertexId::from(classedges[2 * pos]);
            let tgt = VertexId::from(classedges[2 * pos + 1]);
            edges.push((src, tgt));
            code -= current_power;
        }
        current_power /= 2;
        pos += 1;
    }

    let mut graph = Graph::new(size, directed)?;
    for (src, tgt) in edges {
        graph.add_edge(src, tgt)?;
    }

    Ok(graph)
}

fn get_tables(n: u32, directed: bool) -> IgraphResult<(&'static [u32], &'static [u8], u32)> {
    if directed {
        match n {
            3 => Ok((&tables::ISOCLASS_3_IDX, &tables::ISOCLASS2_3, 3)),
            4 => Ok((&tables::ISOCLASS_4_IDX, &tables::ISOCLASS2_4, 4)),
            _ => Err(IgraphError::InvalidArgument(format!(
                "isoclass: directed graphs require 3 or 4 vertices (got {n})"
            ))),
        }
    } else {
        match n {
            3 => Ok((&tables::ISOCLASS_3U_IDX, &tables::ISOCLASS2_3U, 3)),
            4 => Ok((&tables::ISOCLASS_4U_IDX, &tables::ISOCLASS2_4U, 4)),
            5 => Ok((&tables::ISOCLASS_5U_IDX, &tables::ISOCLASS2_5U, 5)),
            _ => Err(IgraphError::InvalidArgument(format!(
                "isoclass: undirected graphs require 3, 4, or 5 vertices (got {n})"
            ))),
        }
    }
}

fn get_create_tables(
    size: u32,
    directed: bool,
) -> IgraphResult<(&'static [u32], &'static [u8], u32)> {
    if directed {
        match size {
            3 => Ok((&tables::ISOGRAPHS_3, &tables::CLASSEDGES_3, 32)),
            4 => Ok((&tables::ISOGRAPHS_4, &tables::CLASSEDGES_4, 2048)),
            _ => Err(IgraphError::InvalidArgument(format!(
                "isoclass_create: directed graphs require size 3 or 4 (got {size})"
            ))),
        }
    } else {
        match size {
            3 => Ok((&tables::ISOGRAPHS_3U, &tables::CLASSEDGES_3U, 4)),
            4 => Ok((&tables::ISOGRAPHS_4U, &tables::CLASSEDGES_4U, 32)),
            5 => Ok((&tables::ISOGRAPHS_5U, &tables::CLASSEDGES_5U, 512)),
            _ => Err(IgraphError::InvalidArgument(format!(
                "isoclass_create: undirected graphs require size 3, 4, or 5 (got {size})"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isoclass_directed_3_empty() {
        let g = Graph::new(3, true).unwrap();
        assert_eq!(isoclass(&g).unwrap(), 0);
    }

    #[test]
    fn isoclass_directed_3_complete() {
        let mut g = Graph::new(3, true).unwrap();
        for i in 0..3u32 {
            for j in 0..3u32 {
                if i != j {
                    g.add_edge(i, j).unwrap();
                }
            }
        }
        assert_eq!(isoclass(&g).unwrap(), 15);
    }

    #[test]
    fn isoclass_undirected_3_triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        assert_eq!(isoclass(&g).unwrap(), 3);
    }

    #[test]
    fn isoclass_undirected_3_single_edge() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        assert_eq!(isoclass(&g).unwrap(), 1);
    }

    #[test]
    fn isoclass_undirected_4_empty() {
        let g = Graph::with_vertices(4);
        assert_eq!(isoclass(&g).unwrap(), 0);
    }

    #[test]
    fn isoclass_undirected_4_complete() {
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert_eq!(isoclass(&g).unwrap(), 10);
    }

    #[test]
    fn isoclass_undirected_5_empty() {
        let g = Graph::with_vertices(5);
        assert_eq!(isoclass(&g).unwrap(), 0);
    }

    #[test]
    fn isoclass_undirected_5_complete() {
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert_eq!(isoclass(&g).unwrap(), 33);
    }

    #[test]
    fn isoclass_create_roundtrip_directed_3() {
        for cls in 0..16u32 {
            let g = isoclass_create(3, cls, true).unwrap();
            assert_eq!(isoclass(&g).unwrap(), cls, "class {cls} roundtrip failed");
        }
    }

    #[test]
    fn isoclass_create_roundtrip_directed_4() {
        for cls in 0..218u32 {
            let g = isoclass_create(4, cls, true).unwrap();
            assert_eq!(isoclass(&g).unwrap(), cls, "class {cls} roundtrip failed");
        }
    }

    #[test]
    fn isoclass_create_roundtrip_undirected_3() {
        for cls in 0..4u32 {
            let g = isoclass_create(3, cls, false).unwrap();
            assert_eq!(isoclass(&g).unwrap(), cls, "class {cls} roundtrip failed");
        }
    }

    #[test]
    fn isoclass_create_roundtrip_undirected_4() {
        for cls in 0..11u32 {
            let g = isoclass_create(4, cls, false).unwrap();
            assert_eq!(isoclass(&g).unwrap(), cls, "class {cls} roundtrip failed");
        }
    }

    #[test]
    fn isoclass_create_roundtrip_undirected_5() {
        for cls in 0..34u32 {
            let g = isoclass_create(5, cls, false).unwrap();
            assert_eq!(isoclass(&g).unwrap(), cls, "class {cls} roundtrip failed");
        }
    }

    #[test]
    fn isoclass_create_out_of_range() {
        assert!(isoclass_create(3, 16, true).is_err());
        assert!(isoclass_create(4, 218, true).is_err());
        assert!(isoclass_create(3, 4, false).is_err());
        assert!(isoclass_create(4, 11, false).is_err());
        assert!(isoclass_create(5, 34, false).is_err());
    }

    #[test]
    fn isoclass_invalid_size() {
        let g = Graph::new(2, true).unwrap();
        assert!(isoclass(&g).is_err());
        let g = Graph::new(5, true).unwrap();
        assert!(isoclass(&g).is_err());
        let g = Graph::with_vertices(2);
        assert!(isoclass(&g).is_err());
        let g = Graph::with_vertices(6);
        assert!(isoclass(&g).is_err());
    }

    #[test]
    fn isoclass_self_loops_ignored() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let mut g2 = Graph::new(3, true).unwrap();
        g2.add_edge(0, 1).unwrap();
        assert_eq!(isoclass(&g).unwrap(), isoclass(&g2).unwrap());
    }

    #[test]
    fn isoclass_subgraph_basic() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(3, 4).unwrap();

        // Triangle subgraph
        assert_eq!(isoclass_subgraph(&g, &[0, 1, 2]).unwrap(), 3);
        // Single edge subgraph
        assert_eq!(isoclass_subgraph(&g, &[0, 3, 4]).unwrap(), 1);
        // Empty subgraph
        assert_eq!(isoclass_subgraph(&g, &[0, 3, 2]).unwrap(), 1);
    }

    #[test]
    fn isoclass_subgraph_invalid_vertex() {
        let g = Graph::with_vertices(3);
        assert!(isoclass_subgraph(&g, &[0, 1, 5]).is_err());
    }
}

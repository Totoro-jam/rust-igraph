//! LGL (Large Graph Layout) adjacency-list I/O (ALGO-IO-003).
//!
//! Reads and writes graphs in the `.lgl` format used by the Large Graph
//! Layout program. The format encodes an adjacency list where each hub
//! vertex is introduced by a `# vertexname` header, followed by lines
//! listing its neighbours (one per line), optionally with a weight.
//!
//! ```text
//! # vertex1
//! vertex2 weight
//! vertex3
//! # vertex2
//! vertex4 weight
//! ```
//!
//! The resulting graph is always **undirected**. Vertex names are mapped to
//! internal indices in first-occurrence order.
//!
//! Counterpart of `igraph_read_graph_lgl` / `igraph_write_graph_lgl`.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};

use crate::core::{Graph, IgraphError, IgraphResult};

/// Result of reading an LGL file: the graph plus optional metadata.
#[derive(Debug, Clone)]
pub struct LglGraph {
    /// The parsed graph (always undirected).
    pub graph: Graph,
    /// Vertex names in index order.
    pub names: Vec<String>,
    /// Edge weights (one per edge, in edge order). `None` if no edge had
    /// a weight specified. Missing weights default to 0.0.
    pub weights: Option<Vec<f64>>,
}

/// Read a graph from LGL (`.lgl`) format.
///
/// Lines starting with `# ` introduce a hub vertex. Subsequent lines
/// (until the next hub or EOF) list neighbours, optionally followed by
/// a weight. Empty lines are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::read_lgl;
///
/// let lgl = b"# Alice\nBob 1.0\nCarol\n# Bob\nCarol 2.5\n";
/// let result = read_lgl(&lgl[..]).unwrap();
/// assert_eq!(result.graph.vcount(), 3);
/// assert_eq!(result.graph.ecount(), 3);
/// assert_eq!(result.names, vec!["Alice", "Bob", "Carol"]);
/// ```
pub fn read_lgl<R: Read>(input: R) -> IgraphResult<LglGraph> {
    let reader = BufReader::new(input);
    let mut name_map: HashMap<String, u32> = HashMap::new();
    let mut names: Vec<String> = Vec::new();
    let mut edges: Vec<(u32, u32)> = Vec::new();
    let mut weights: Vec<f64> = Vec::new();
    let mut has_any_weight = false;

    let mut current_hub: Option<u32> = None;

    for (line_idx, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        if let Some(hub_name) = trimmed.strip_prefix('#') {
            let hub_name = hub_name.trim();
            if hub_name.is_empty() {
                return Err(IgraphError::Parse {
                    line: line_idx.wrapping_add(1),
                    message: "hub line '#' has no vertex name".into(),
                });
            }
            let hub_id = get_or_insert_name(hub_name, &mut name_map, &mut names);
            current_hub = Some(hub_id);
        } else {
            let hub_id = current_hub.ok_or_else(|| IgraphError::Parse {
                line: line_idx.wrapping_add(1),
                message: "neighbour line before any hub definition".into(),
            })?;

            let mut parts = trimmed.split_whitespace();
            let neighbour_name = parts.next().ok_or_else(|| IgraphError::Parse {
                line: line_idx.wrapping_add(1),
                message: "empty neighbour line".into(),
            })?;

            let neighbour_id = get_or_insert_name(neighbour_name, &mut name_map, &mut names);
            edges.push((hub_id, neighbour_id));

            let weight = if let Some(w_str) = parts.next() {
                let w = w_str.parse::<f64>().map_err(|e| IgraphError::Parse {
                    line: line_idx.wrapping_add(1),
                    message: format!("invalid weight '{w_str}': {e}"),
                })?;
                has_any_weight = true;
                w
            } else {
                0.0
            };
            weights.push(weight);
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    let n = names.len() as u32;
    let mut graph = Graph::with_vertices(n);
    graph.add_edges(edges)?;

    Ok(LglGraph {
        graph,
        names,
        weights: if has_any_weight { Some(weights) } else { None },
    })
}

/// Write a graph in LGL format.
///
/// Each vertex is written as a hub (`# name`) followed by its neighbours.
/// If `names` is provided, uses them as vertex labels; otherwise uses
/// numeric ids. If `weights` is provided (one per edge), appends the weight.
///
/// To avoid duplicate edges in the undirected representation, each edge is
/// emitted only once: under the endpoint with the smaller index.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, write_lgl};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
///
/// let names = vec!["A".to_string(), "B".to_string(), "C".to_string()];
/// let mut buf = Vec::new();
/// write_lgl(&g, Some(&names), None, &mut buf).unwrap();
/// let s = String::from_utf8(buf).unwrap();
/// assert!(s.contains("# A"));
/// assert!(s.contains("B"));
/// assert!(s.contains("C"));
/// ```
pub fn write_lgl<W: Write>(
    graph: &Graph,
    names: Option<&[String]>,
    weights: Option<&[f64]>,
    writer: &mut W,
) -> IgraphResult<()> {
    if let Some(n) = names {
        if n.len() != graph.vcount() as usize {
            return Err(IgraphError::InvalidArgument(format!(
                "names length {} does not match vcount {}",
                n.len(),
                graph.vcount()
            )));
        }
    }
    if let Some(w) = weights {
        if w.len() != graph.ecount() {
            return Err(IgraphError::InvalidArgument(format!(
                "weights length {} does not match ecount {}",
                w.len(),
                graph.ecount()
            )));
        }
    }

    // Build adjacency lists: for each vertex, collect (neighbour, edge_id).
    // Only emit each edge once — under the endpoint with the smaller index.
    let vcount = graph.vcount() as usize;
    let mut adj: Vec<Vec<(u32, usize)>> = vec![Vec::new(); vcount];

    for eid in 0..graph.ecount() {
        #[allow(clippy::cast_possible_truncation)]
        let (src, tgt) = graph.edge(eid as u32)?;
        // For self-loops, emit under the vertex itself
        let hub = src.min(tgt);
        let neighbour = src.max(tgt);
        adj[hub as usize].push((neighbour, eid));
    }

    for v in 0..vcount {
        if adj[v].is_empty() {
            continue;
        }

        let hub_name = match names {
            Some(n) => n[v].as_str().to_owned(),
            None => v.to_string(),
        };
        writeln!(writer, "# {hub_name}")?;

        for &(neighbour, eid) in &adj[v] {
            let nbr_name = match names {
                Some(n) => n[neighbour as usize].as_str().to_owned(),
                None => neighbour.to_string(),
            };
            match weights {
                Some(w) => writeln!(writer, "{nbr_name} {}", w[eid])?,
                None => writeln!(writer, "{nbr_name}")?,
            }
        }
    }

    Ok(())
}

fn get_or_insert_name(name: &str, map: &mut HashMap<String, u32>, names: &mut Vec<String>) -> u32 {
    if let Some(&id) = map.get(name) {
        id
    } else {
        #[allow(clippy::cast_possible_truncation)]
        let id = names.len() as u32;
        map.insert(name.to_owned(), id);
        names.push(name.to_owned());
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let result = read_lgl(&b""[..]).unwrap();
        assert_eq!(result.graph.vcount(), 0);
        assert_eq!(result.graph.ecount(), 0);
        assert!(result.names.is_empty());
        assert!(result.weights.is_none());
    }

    #[test]
    fn test_single_hub_no_neighbours() {
        let input = b"# Alice\n";
        let result = read_lgl(&input[..]).unwrap();
        assert_eq!(result.graph.vcount(), 1);
        assert_eq!(result.graph.ecount(), 0);
        assert_eq!(result.names, vec!["Alice"]);
    }

    #[test]
    fn test_basic_unweighted() {
        let input = b"# a\nb\nc\n# b\nc\n";
        let result = read_lgl(&input[..]).unwrap();
        assert_eq!(result.graph.vcount(), 3);
        assert_eq!(result.graph.ecount(), 3);
        assert_eq!(result.names, vec!["a", "b", "c"]);
        assert!(result.weights.is_none());
    }

    #[test]
    fn test_weighted() {
        let input = b"# x\ny 1.5\nz 2.0\n";
        let result = read_lgl(&input[..]).unwrap();
        assert_eq!(result.graph.ecount(), 2);
        let w = result.weights.unwrap();
        assert!((w[0] - 1.5).abs() < 1e-10);
        assert!((w[1] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_mixed_weights() {
        let input = b"# a\nb 3.0\nc\n";
        let result = read_lgl(&input[..]).unwrap();
        let w = result.weights.unwrap();
        assert!((w[0] - 3.0).abs() < 1e-10);
        assert!((w[1] - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_blank_lines_skipped() {
        let input = b"\n# a\n\nb\n\n";
        let result = read_lgl(&input[..]).unwrap();
        assert_eq!(result.graph.vcount(), 2);
        assert_eq!(result.graph.ecount(), 1);
    }

    #[test]
    fn test_multiple_hubs() {
        let input = b"# Alice\nBob\nCarol\n# Bob\nCarol\n";
        let result = read_lgl(&input[..]).unwrap();
        assert_eq!(result.graph.vcount(), 3);
        assert_eq!(result.graph.ecount(), 3);
    }

    #[test]
    fn test_self_loop() {
        let input = b"# a\na\n";
        let result = read_lgl(&input[..]).unwrap();
        assert_eq!(result.graph.vcount(), 1);
        assert_eq!(result.graph.ecount(), 1);
    }

    #[test]
    fn test_always_undirected() {
        let input = b"# x\ny\n# y\nx\n";
        let result = read_lgl(&input[..]).unwrap();
        assert!(!result.graph.is_directed());
    }

    #[test]
    fn test_negative_weight() {
        let input = b"# a\nb -1.5\n";
        let result = read_lgl(&input[..]).unwrap();
        let w = result.weights.unwrap();
        assert!((w[0] - (-1.5)).abs() < 1e-10);
    }

    #[test]
    fn test_scientific_notation_weight() {
        let input = b"# a\nb 2.5e-3\n";
        let result = read_lgl(&input[..]).unwrap();
        let w = result.weights.unwrap();
        assert!((w[0] - 0.0025).abs() < 1e-10);
    }

    #[test]
    fn test_invalid_weight_error() {
        let input = b"# a\nb notanumber\n";
        let result = read_lgl(&input[..]);
        assert!(result.is_err());
    }

    #[test]
    fn test_no_hub_before_neighbour_error() {
        let input = b"b\n";
        let result = read_lgl(&input[..]);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_hub_name_error() {
        let input = b"#\n";
        let result = read_lgl(&input[..]);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_unweighted() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();

        let names = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let mut buf = Vec::new();
        write_lgl(&g, Some(&names), None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("# A\n"));
        assert!(s.contains("B\n"));
        assert!(s.contains("C\n"));
    }

    #[test]
    fn test_write_weighted() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();

        let names = vec!["x".to_string(), "y".to_string()];
        let weights = vec![2.75];
        let mut buf = Vec::new();
        write_lgl(&g, Some(&names), Some(&weights), &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("# x\n"));
        assert!(s.contains("y 2.75"));
    }

    #[test]
    fn test_write_numeric_names() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();

        let mut buf = Vec::new();
        write_lgl(&g, None, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("# 0\n"));
        assert!(s.contains("1\n"));
    }

    #[test]
    fn test_round_trip() {
        let input = b"# Alice\nBob 1.0\nCarol 2.0\n# Bob\nCarol 3.0\n";
        let result = read_lgl(&input[..]).unwrap();

        let mut buf = Vec::new();
        write_lgl(
            &result.graph,
            Some(&result.names),
            result.weights.as_deref(),
            &mut buf,
        )
        .unwrap();

        let result2 = read_lgl(&buf[..]).unwrap();
        assert_eq!(result2.graph.vcount(), result.graph.vcount());
        assert_eq!(result2.graph.ecount(), result.graph.ecount());
    }

    #[test]
    fn test_write_names_length_mismatch() {
        let g = Graph::with_vertices(3);
        let names = vec!["A".to_string()];
        let mut buf = Vec::new();
        assert!(write_lgl(&g, Some(&names), None, &mut buf).is_err());
    }

    #[test]
    fn test_write_weights_length_mismatch() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let weights = vec![1.0, 2.0];
        let mut buf = Vec::new();
        assert!(write_lgl(&g, None, Some(&weights), &mut buf).is_err());
    }

    #[test]
    fn test_isolated_vertex_not_in_output() {
        // Isolated vertices with no edges don't appear in LGL output
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        // vertex 2 is isolated

        let names = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let mut buf = Vec::new();
        write_lgl(&g, Some(&names), None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("# A\n"));
        assert!(s.contains("B\n"));
        assert!(!s.contains('C'));
    }
}

//! NCOL (LGL edge list) I/O (ALGO-IO-002).
//!
//! Reads and writes graphs in the `.ncol` format used by the Large Graph
//! Layout (LGL) program. This is a symbolic weighted edge list: one edge
//! per line, two vertex names separated by whitespace, optionally followed
//! by an edge weight.
//!
//! The resulting graph is always **undirected**. Vertex names are mapped to
//! internal indices in first-occurrence order.
//!
//! Counterpart of `igraph_read_graph_ncol` / `igraph_write_graph_ncol`.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};

use crate::core::attributes::AttributeValue;
use crate::core::{Graph, IgraphError, IgraphResult};

/// Result of reading an NCOL file: the graph plus optional metadata.
#[derive(Debug, Clone)]
pub struct NcolGraph {
    /// The parsed graph (always undirected).
    pub graph: Graph,
    /// Vertex names in index order (name at position `i` is the name of
    /// vertex `i`).
    pub names: Vec<String>,
    /// Edge weights (one per edge, in edge order). `None` if no edge had
    /// a weight specified. If some edges have weights and some don't, the
    /// missing weights default to 0.0.
    pub weights: Option<Vec<f64>>,
}

/// Read a graph from NCOL (`.ncol`) format.
///
/// Each non-empty, non-comment line defines an edge:
/// ```text
/// vertex1 vertex2 [weight]
/// ```
///
/// Lines starting with `#` or `%` are comments. Vertex names are arbitrary
/// non-whitespace strings. The graph is always undirected.
///
/// A line with a single name creates an isolated vertex (igraph extension).
///
/// # Examples
///
/// ```
/// use rust_igraph::read_ncol;
///
/// let ncol = b"Alice Bob 1.0\nBob Carol\nAlice Carol 2.5\n";
/// let result = read_ncol(&ncol[..]).unwrap();
/// assert_eq!(result.graph.vcount(), 3);
/// assert_eq!(result.graph.ecount(), 3);
/// assert_eq!(result.names, vec!["Alice", "Bob", "Carol"]);
/// assert!(result.weights.is_some());
/// ```
pub fn read_ncol<R: Read>(input: R) -> IgraphResult<NcolGraph> {
    let reader = BufReader::new(input);
    let mut name_map: HashMap<String, u32> = HashMap::new();
    let mut names: Vec<String> = Vec::new();
    let mut edges: Vec<(u32, u32)> = Vec::new();
    let mut weights: Vec<f64> = Vec::new();
    let mut has_any_weight = false;

    for (line_idx, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('%') {
            continue;
        }

        let mut parts = trimmed.split_whitespace();

        let vertex_a = parts.next().ok_or_else(|| IgraphError::Parse {
            line: line_idx.wrapping_add(1),
            message: "empty edge line".into(),
        })?;

        let id1 = get_or_insert_name(vertex_a, &mut name_map, &mut names);

        // If there's no second token, this is an isolated vertex declaration
        let Some(vertex_b) = parts.next() else {
            continue;
        };

        let id2 = get_or_insert_name(vertex_b, &mut name_map, &mut names);
        edges.push((id1, id2));

        // Optional weight
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

    #[allow(clippy::cast_possible_truncation)]
    let n = names.len() as u32;
    let mut graph = Graph::with_vertices(n);
    graph.add_edges(edges)?;

    let has_any_label = names.iter().any(|l| !l.is_empty());
    if has_any_label {
        graph.set_vertex_attribute_all(
            "name",
            names
                .iter()
                .map(|l| AttributeValue::String(l.clone()))
                .collect(),
        )?;
    }
    if has_any_weight {
        graph.set_edge_attribute_all(
            "weight",
            weights
                .iter()
                .map(|&w| AttributeValue::Numeric(w))
                .collect(),
        )?;
    }

    Ok(NcolGraph {
        graph,
        names,
        weights: if has_any_weight { Some(weights) } else { None },
    })
}

/// Write a graph in NCOL format.
///
/// If `names` is provided, uses them as vertex labels; otherwise uses
/// numeric ids as strings. If `weights` is provided (one per edge),
/// appends the weight after each edge.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, write_ncol};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
///
/// let names = vec!["A".to_string(), "B".to_string(), "C".to_string()];
/// let mut buf = Vec::new();
/// write_ncol(&g, Some(&names), None, &mut buf).unwrap();
/// let s = String::from_utf8(buf).unwrap();
/// assert!(s.contains("A B"));
/// assert!(s.contains("B C"));
/// ```
pub fn write_ncol<W: Write>(
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

    for eid in 0..graph.ecount() {
        #[allow(clippy::cast_possible_truncation)]
        let (src, tgt) = graph.edge(eid as u32)?;

        let src_name = match names {
            Some(n) => n[src as usize].as_str().to_owned(),
            None => graph
                .vertex_attribute("name", src)
                .and_then(AttributeValue::as_str)
                .map_or_else(|| src.to_string(), str::to_owned),
        };
        let tgt_name = match names {
            Some(n) => n[tgt as usize].as_str().to_owned(),
            None => graph
                .vertex_attribute("name", tgt)
                .and_then(AttributeValue::as_str)
                .map_or_else(|| tgt.to_string(), str::to_owned),
        };

        #[allow(clippy::cast_possible_truncation)]
        let eid_u32 = eid as u32;
        match weights {
            Some(w) => writeln!(writer, "{src_name} {tgt_name} {}", w[eid])?,
            None => {
                if let Some(w) = graph
                    .edge_attribute("weight", eid_u32)
                    .and_then(AttributeValue::as_f64)
                {
                    writeln!(writer, "{src_name} {tgt_name} {w}")?;
                } else {
                    writeln!(writer, "{src_name} {tgt_name}")?;
                }
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
        let result = read_ncol(&b""[..]).unwrap();
        assert_eq!(result.graph.vcount(), 0);
        assert_eq!(result.graph.ecount(), 0);
        assert!(result.names.is_empty());
        assert!(result.weights.is_none());
    }

    #[test]
    fn test_basic_unweighted() {
        let input = b"a b\nb c\n";
        let result = read_ncol(&input[..]).unwrap();
        assert_eq!(result.graph.vcount(), 3);
        assert_eq!(result.graph.ecount(), 2);
        assert_eq!(result.names, vec!["a", "b", "c"]);
        assert!(result.weights.is_none());
    }

    #[test]
    fn test_weighted() {
        let input = b"a b 1.5\nb c 2.0\n";
        let result = read_ncol(&input[..]).unwrap();
        assert_eq!(result.graph.ecount(), 2);
        let w = result.weights.unwrap();
        assert!((w[0] - 1.5).abs() < 1e-10);
        assert!((w[1] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_mixed_weights() {
        let input = b"a b 3.0\nb c\n";
        let result = read_ncol(&input[..]).unwrap();
        let w = result.weights.unwrap();
        assert!((w[0] - 3.0).abs() < 1e-10);
        assert!((w[1] - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_comments_and_blanks() {
        let input = b"# comment\n\n% another comment\na b\n";
        let result = read_ncol(&input[..]).unwrap();
        assert_eq!(result.graph.vcount(), 2);
        assert_eq!(result.graph.ecount(), 1);
    }

    #[test]
    fn test_isolated_vertex() {
        let input = b"a b\nc\n";
        let result = read_ncol(&input[..]).unwrap();
        assert_eq!(result.graph.vcount(), 3);
        assert_eq!(result.graph.ecount(), 1);
        assert_eq!(result.names[2], "c");
    }

    #[test]
    fn test_self_loop() {
        let input = b"a a\n";
        let result = read_ncol(&input[..]).unwrap();
        assert_eq!(result.graph.vcount(), 1);
        assert_eq!(result.graph.ecount(), 1);
    }

    #[test]
    fn test_always_undirected() {
        let input = b"x y\ny x\n";
        let result = read_ncol(&input[..]).unwrap();
        assert!(!result.graph.is_directed());
        assert_eq!(result.graph.ecount(), 2); // multi-edge allowed
    }

    #[test]
    fn test_negative_weight() {
        let input = b"a b -1.5\n";
        let result = read_ncol(&input[..]).unwrap();
        let w = result.weights.unwrap();
        assert!((w[0] - (-1.5)).abs() < 1e-10);
    }

    #[test]
    fn test_scientific_notation_weight() {
        let input = b"a b 1.5e-3\n";
        let result = read_ncol(&input[..]).unwrap();
        let w = result.weights.unwrap();
        assert!((w[0] - 0.0015).abs() < 1e-10);
    }

    #[test]
    fn test_invalid_weight_error() {
        let input = b"a b notanumber\n";
        let result = read_ncol(&input[..]);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_unweighted() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let names = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let mut buf = Vec::new();
        write_ncol(&g, Some(&names), None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("A B\n"));
        assert!(s.contains("B C\n"));
    }

    #[test]
    fn test_write_weighted() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();

        let names = vec!["x".to_string(), "y".to_string()];
        let weights = vec![2.75];
        let mut buf = Vec::new();
        write_ncol(&g, Some(&names), Some(&weights), &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("x y 2.75"));
    }

    #[test]
    fn test_write_numeric_names() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();

        let mut buf = Vec::new();
        write_ncol(&g, None, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("0 1"));
    }

    #[test]
    fn test_round_trip() {
        let input = b"Alice Bob 1.0\nBob Carol 2.0\nAlice Carol 3.0\n";
        let result = read_ncol(&input[..]).unwrap();

        let mut buf = Vec::new();
        write_ncol(
            &result.graph,
            Some(&result.names),
            result.weights.as_deref(),
            &mut buf,
        )
        .unwrap();

        let result2 = read_ncol(&buf[..]).unwrap();
        assert_eq!(result2.graph.vcount(), result.graph.vcount());
        assert_eq!(result2.graph.ecount(), result.graph.ecount());
        assert_eq!(result2.names, result.names);
    }

    #[test]
    fn test_write_names_length_mismatch() {
        let g = Graph::with_vertices(3);
        let names = vec!["A".to_string()];
        let mut buf = Vec::new();
        assert!(write_ncol(&g, Some(&names), None, &mut buf).is_err());
    }

    #[test]
    fn test_write_weights_length_mismatch() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let weights = vec![1.0, 2.0]; // 2 weights but only 1 edge
        let mut buf = Vec::new();
        assert!(write_ncol(&g, None, Some(&weights), &mut buf).is_err());
    }

    // --- Attribute integration tests ---

    #[test]
    fn test_read_stores_name_attribute() {
        let input = b"Alice Bob 1.0\nBob Carol\n";
        let result = read_ncol(&input[..]).unwrap();
        let g = &result.graph;

        assert_eq!(
            g.vertex_attribute("name", 0)
                .and_then(AttributeValue::as_str),
            Some("Alice")
        );
        assert_eq!(
            g.vertex_attribute("name", 1)
                .and_then(AttributeValue::as_str),
            Some("Bob")
        );
        assert_eq!(
            g.vertex_attribute("name", 2)
                .and_then(AttributeValue::as_str),
            Some("Carol")
        );
    }

    #[test]
    fn test_read_stores_weight_attribute() {
        let input = b"a b 1.5\nb c 2.0\n";
        let result = read_ncol(&input[..]).unwrap();
        let g = &result.graph;

        let w0 = g
            .edge_attribute("weight", 0)
            .and_then(AttributeValue::as_f64)
            .unwrap();
        let w1 = g
            .edge_attribute("weight", 1)
            .and_then(AttributeValue::as_f64)
            .unwrap();
        assert!((w0 - 1.5).abs() < 1e-10);
        assert!((w1 - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_read_no_weight_attribute_when_unweighted() {
        let input = b"a b\nb c\n";
        let result = read_ncol(&input[..]).unwrap();
        assert!(result.graph.edge_attribute("weight", 0).is_none());
    }

    #[test]
    fn test_write_fallback_to_name_attribute() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.set_vertex_attribute("name", 0, AttributeValue::String("X".into()))
            .unwrap();
        g.set_vertex_attribute("name", 1, AttributeValue::String("Y".into()))
            .unwrap();

        let mut buf = Vec::new();
        write_ncol(&g, None, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("X Y"));
    }

    #[test]
    fn test_write_fallback_to_weight_attribute() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.set_edge_attribute("weight", 0, AttributeValue::Numeric(4.25))
            .unwrap();

        let mut buf = Vec::new();
        write_ncol(&g, None, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("4.25"));
    }

    #[test]
    fn test_roundtrip_via_attributes() {
        let input = b"Alice Bob 1.5\nBob Carol 2.5\n";
        let result = read_ncol(&input[..]).unwrap();

        // Write using attribute fallback (no explicit names/weights)
        let mut buf = Vec::new();
        write_ncol(&result.graph, None, None, &mut buf).unwrap();

        let result2 = read_ncol(&buf[..]).unwrap();
        assert_eq!(result2.names, vec!["Alice", "Bob", "Carol"]);
        let w = result2.weights.unwrap();
        assert!((w[0] - 1.5).abs() < 1e-10);
        assert!((w[1] - 2.5).abs() < 1e-10);
    }

    #[test]
    fn test_explicit_params_override_attributes() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.set_vertex_attribute("name", 0, AttributeValue::String("attr_A".into()))
            .unwrap();
        g.set_vertex_attribute("name", 1, AttributeValue::String("attr_B".into()))
            .unwrap();
        g.set_edge_attribute("weight", 0, AttributeValue::Numeric(9.0))
            .unwrap();

        let names = vec!["explicit_A".to_string(), "explicit_B".to_string()];
        let weights = vec![1.0];
        let mut buf = Vec::new();
        write_ncol(&g, Some(&names), Some(&weights), &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("explicit_A explicit_B 1"));
        assert!(!s.contains("attr_A"));
        assert!(!s.contains('9'));
    }
}

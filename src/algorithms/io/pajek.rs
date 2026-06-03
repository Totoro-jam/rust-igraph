//! Pajek (.net) I/O (ALGO-IO-005).
//!
//! Reads and writes graphs in a subset of the Pajek `.net` format.
//! The format has a `*Vertices N` header followed by optional vertex
//! labels, then `*Edges` (undirected) or `*Arcs` (directed) sections
//! listing edges with optional weights. Vertex labels are stored as the
//! `"name"` vertex attribute and edge weights as the `"weight"` edge
//! attribute via the [`Graph`] attribute system.
//!
//! ```text
//! *Vertices 4
//! 1 "Alice"
//! 2 "Bob"
//! 3 "Carol"
//! 4 "Dave"
//! *Edges
//! 1 2 1.0
//! 2 3
//! 3 4 2.5
//! ```
//!
//! Limitations:
//! - Only `.net` files (not `.paj` project files)
//! - No temporal networks
//! - Mixed directed/undirected not supported
//! - Bipartite mode not supported (vertex types ignored)
//!
//! Counterpart of `igraph_read_graph_pajek` / `igraph_write_graph_pajek`.

use std::io::{BufRead, BufReader, Read, Write};

use crate::core::attributes::AttributeValue;
use crate::core::{Graph, IgraphError, IgraphResult};

/// Result of reading a Pajek file.
#[derive(Debug, Clone)]
pub struct PajekGraph {
    /// The parsed graph.
    pub graph: Graph,
    /// Vertex labels (one per vertex, in index order). `None` if no
    /// vertex had a label.
    pub labels: Option<Vec<String>>,
    /// Edge weights (one per edge, in edge order). `None` if no edge
    /// had a weight.
    pub weights: Option<Vec<f64>>,
}

/// Read a graph from Pajek (`.net`) format.
///
/// Supports `*Vertices`, `*Edges` (undirected), and `*Arcs` (directed)
/// sections. Vertex IDs in the file are 1-based.
///
/// # Examples
///
/// ```
/// use rust_igraph::read_pajek;
///
/// let pajek = b"*Vertices 3\n1 \"A\"\n2 \"B\"\n3 \"C\"\n*Edges\n1 2\n2 3\n1 3\n";
/// let result = read_pajek(&pajek[..]).unwrap();
/// assert_eq!(result.graph.vcount(), 3);
/// assert_eq!(result.graph.ecount(), 3);
/// assert!(!result.graph.is_directed());
/// ```
#[allow(clippy::too_many_lines)]
pub fn read_pajek<R: Read>(input: R) -> IgraphResult<PajekGraph> {
    #[derive(PartialEq)]
    enum Section {
        None,
        Vertices,
        Edges,
    }

    let reader = BufReader::new(input);

    let mut n_vertices: Option<u32> = None;
    let mut directed = false;
    let mut labels: Vec<String> = Vec::new();
    let mut has_any_label = false;
    let mut edges: Vec<(u32, u32)> = Vec::new();
    let mut weights: Vec<f64> = Vec::new();
    let mut has_any_weight = false;
    let mut section = Section::None;

    for (line_idx, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('%') {
            continue;
        }

        let lower = trimmed.to_ascii_lowercase();

        // Section headers
        if lower.starts_with("*vertices") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() < 2 {
                return Err(IgraphError::Parse {
                    line: line_idx.wrapping_add(1),
                    message: "*Vertices line needs vertex count".into(),
                });
            }
            let n: u32 = parts[1].parse().map_err(|e| IgraphError::Parse {
                line: line_idx.wrapping_add(1),
                message: format!("invalid vertex count: {e}"),
            })?;
            n_vertices = Some(n);
            labels = vec![String::new(); n as usize];
            section = Section::Vertices;
            continue;
        }

        if lower.starts_with("*edges") || lower.starts_with("*edgeslist") {
            directed = false;
            section = Section::Edges;
            continue;
        }

        if lower.starts_with("*arcs") || lower.starts_with("*arcslist") {
            directed = true;
            section = Section::Edges;
            continue;
        }

        // Skip other section headers we don't handle
        if lower.starts_with('*') {
            section = Section::None;
            continue;
        }

        match section {
            Section::Vertices => {
                // Format: ID "label" [x y z] [other attrs...]
                // We only extract the ID and label
                let (vid, label) = parse_vertex_line(trimmed, line_idx)?;
                if let Some(n) = n_vertices {
                    if vid == 0 || vid > n {
                        return Err(IgraphError::Parse {
                            line: line_idx.wrapping_add(1),
                            message: format!("vertex id {vid} out of range [1, {n}]"),
                        });
                    }
                }
                if let Some(lbl) = label {
                    has_any_label = true;
                    let idx = (vid.wrapping_sub(1)) as usize;
                    if idx < labels.len() {
                        labels[idx] = lbl;
                    }
                }
            }
            Section::Edges => {
                // Format: FROM TO [weight] [other attrs...]
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() < 2 {
                    return Err(IgraphError::Parse {
                        line: line_idx.wrapping_add(1),
                        message: "edge line needs at least: FROM TO".into(),
                    });
                }
                let from: u32 = parts[0].parse().map_err(|e| IgraphError::Parse {
                    line: line_idx.wrapping_add(1),
                    message: format!("invalid source id: {e}"),
                })?;
                let to: u32 = parts[1].parse().map_err(|e| IgraphError::Parse {
                    line: line_idx.wrapping_add(1),
                    message: format!("invalid target id: {e}"),
                })?;

                if from == 0 || to == 0 {
                    return Err(IgraphError::Parse {
                        line: line_idx.wrapping_add(1),
                        message: "vertex IDs are 1-based in Pajek format".into(),
                    });
                }

                edges.push((from.wrapping_sub(1), to.wrapping_sub(1)));

                let weight = if parts.len() >= 3 {
                    match parts[2].parse::<f64>() {
                        Ok(w) => {
                            has_any_weight = true;
                            w
                        }
                        Err(_) => 0.0,
                    }
                } else {
                    0.0
                };
                weights.push(weight);
            }
            Section::None => {
                // Ignore lines in unknown sections
            }
        }
    }

    let n = n_vertices.ok_or_else(|| IgraphError::Parse {
        line: 0,
        message: "no *Vertices line found".into(),
    })?;

    let mut graph = Graph::new(n, directed)?;
    graph.add_edges(edges)?;

    // Store labels and weights as graph attributes for I/O interop
    if has_any_label {
        graph.set_vertex_attribute_all(
            "name",
            labels
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

    Ok(PajekGraph {
        graph,
        labels: if has_any_label { Some(labels) } else { None },
        weights: if has_any_weight { Some(weights) } else { None },
    })
}

/// Write a graph in Pajek (`.net`) format.
///
/// Writes `*Vertices` section (with optional labels), then `*Edges`
/// (undirected) or `*Arcs` (directed) section with optional weights.
///
/// When `labels` is `None`, falls back to the `"name"` vertex attribute
/// if present. When `weights` is `None`, falls back to the `"weight"`
/// edge attribute if present.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, write_pajek, AttributeValue};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.set_vertex_attribute_all("name", vec![
///     AttributeValue::String("A".into()),
///     AttributeValue::String("B".into()),
///     AttributeValue::String("C".into()),
/// ]).unwrap();
///
/// let mut buf = Vec::new();
/// write_pajek(&g, None, None, &mut buf).unwrap();
/// let s = String::from_utf8(buf).unwrap();
/// assert!(s.contains("*Vertices 3"));
/// assert!(s.contains("\"A\""));
/// assert!(s.contains("*Edges"));
/// ```
pub fn write_pajek<W: Write>(
    graph: &Graph,
    labels: Option<&[String]>,
    weights: Option<&[f64]>,
    writer: &mut W,
) -> IgraphResult<()> {
    if let Some(l) = labels {
        if l.len() != graph.vcount() as usize {
            return Err(IgraphError::InvalidArgument(format!(
                "labels length {} does not match vcount {}",
                l.len(),
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

    writeln!(writer, "*Vertices {}", graph.vcount())?;
    for v in 0..graph.vcount() {
        let label = if let Some(l) = labels {
            format!("\"{}\"", escape_pajek_string(&l[v as usize]))
        } else if let Some(name) = graph
            .vertex_attribute("name", v)
            .and_then(AttributeValue::as_str)
        {
            format!("\"{}\"", escape_pajek_string(name))
        } else {
            format!("\"{}\"", v + 1)
        };
        writeln!(writer, "{} {label}", v + 1)?;
    }

    if graph.is_directed() {
        writeln!(writer, "*Arcs")?;
    } else {
        writeln!(writer, "*Edges")?;
    }

    for eid in 0..graph.ecount() {
        #[allow(clippy::cast_possible_truncation)]
        let eid_u32 = eid as u32;
        let (from, to) = graph.edge(eid_u32)?;
        if let Some(w) = weights {
            writeln!(writer, "{} {} {}", from + 1, to + 1, w[eid])?;
        } else if let Some(w) = graph
            .edge_attribute("weight", eid_u32)
            .and_then(AttributeValue::as_f64)
        {
            writeln!(writer, "{} {} {w}", from + 1, to + 1)?;
        } else {
            writeln!(writer, "{} {}", from + 1, to + 1)?;
        }
    }

    Ok(())
}

fn parse_vertex_line(line: &str, line_idx: usize) -> IgraphResult<(u32, Option<String>)> {
    let mut chars = line.chars().peekable();

    // Skip leading whitespace
    while chars.peek().is_some_and(|c| c.is_whitespace()) {
        chars.next();
    }

    // Read vertex ID (digits)
    let mut id_str = String::new();
    while chars.peek().is_some_and(char::is_ascii_digit) {
        let Some(c) = chars.next() else { break };
        id_str.push(c);
    }

    if id_str.is_empty() {
        return Err(IgraphError::Parse {
            line: line_idx.wrapping_add(1),
            message: "expected vertex ID".into(),
        });
    }

    let vid: u32 = id_str.parse().map_err(|e| IgraphError::Parse {
        line: line_idx.wrapping_add(1),
        message: format!("invalid vertex id: {e}"),
    })?;

    // Skip whitespace
    while chars.peek().is_some_and(|c| c.is_whitespace()) {
        chars.next();
    }

    // Try to read a quoted label
    let label = if chars.peek() == Some(&'"') {
        chars.next(); // consume opening quote
        let mut lbl = String::new();
        loop {
            match chars.next() {
                Some('"') | None => break,
                Some('\\') => {
                    if let Some(c) = chars.next() {
                        lbl.push(c);
                    }
                }
                Some(c) => lbl.push(c),
            }
        }
        Some(lbl)
    } else {
        // Unquoted label: take next non-whitespace token
        let mut lbl = String::new();
        while chars.peek().is_some_and(|c| !c.is_whitespace()) {
            let Some(c) = chars.next() else { break };
            lbl.push(c);
        }
        if lbl.is_empty() { None } else { Some(lbl) }
    };

    Ok((vid, label))
}

fn escape_pajek_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_undirected() {
        let input = b"*Vertices 3\n1 \"A\"\n2 \"B\"\n3 \"C\"\n*Edges\n1 2\n2 3\n1 3\n";
        let result = read_pajek(&input[..]).unwrap();
        assert_eq!(result.graph.vcount(), 3);
        assert_eq!(result.graph.ecount(), 3);
        assert!(!result.graph.is_directed());
        let labels = result.labels.unwrap();
        assert_eq!(labels, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_directed_arcs() {
        let input = b"*Vertices 2\n*Arcs\n1 2\n2 1\n";
        let result = read_pajek(&input[..]).unwrap();
        assert!(result.graph.is_directed());
        assert_eq!(result.graph.ecount(), 2);
    }

    #[test]
    fn test_weighted_edges() {
        let input = b"*Vertices 3\n*Edges\n1 2 1.5\n2 3 2.5\n";
        let result = read_pajek(&input[..]).unwrap();
        let w = result.weights.unwrap();
        assert!((w[0] - 1.5).abs() < 1e-10);
        assert!((w[1] - 2.5).abs() < 1e-10);
    }

    #[test]
    fn test_no_labels() {
        let input = b"*Vertices 2\n*Edges\n1 2\n";
        let result = read_pajek(&input[..]).unwrap();
        assert!(result.labels.is_none());
    }

    #[test]
    fn test_no_weights() {
        let input = b"*Vertices 2\n*Edges\n1 2\n";
        let result = read_pajek(&input[..]).unwrap();
        assert!(result.weights.is_none());
    }

    #[test]
    fn test_mixed_weights() {
        let input = b"*Vertices 3\n*Edges\n1 2 3.0\n2 3\n";
        let result = read_pajek(&input[..]).unwrap();
        let w = result.weights.unwrap();
        assert!((w[0] - 3.0).abs() < 1e-10);
        assert!((w[1] - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_comment_lines_skipped() {
        let input = b"% comment\n*Vertices 2\n% another\n*Edges\n1 2\n";
        let result = read_pajek(&input[..]).unwrap();
        assert_eq!(result.graph.vcount(), 2);
        assert_eq!(result.graph.ecount(), 1);
    }

    #[test]
    fn test_case_insensitive_sections() {
        let input = b"*vertices 2\n*edges\n1 2\n";
        let result = read_pajek(&input[..]).unwrap();
        assert_eq!(result.graph.vcount(), 2);
    }

    #[test]
    fn test_no_vertices_error() {
        let input = b"*Edges\n1 2\n";
        let result = read_pajek(&input[..]);
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_vertex_id_error() {
        let input = b"*Vertices 2\n*Edges\n0 1\n";
        let result = read_pajek(&input[..]);
        assert!(result.is_err());
    }

    #[test]
    fn test_vertex_id_out_of_range_error() {
        let input = b"*Vertices 2\n1 \"A\"\n5 \"E\"\n*Edges\n1 2\n";
        let result = read_pajek(&input[..]);
        assert!(result.is_err());
    }

    #[test]
    fn test_self_loop() {
        let input = b"*Vertices 2\n*Edges\n1 1\n";
        let result = read_pajek(&input[..]).unwrap();
        assert_eq!(result.graph.ecount(), 1);
    }

    #[test]
    fn test_empty_graph() {
        let input = b"*Vertices 5\n*Edges\n";
        let result = read_pajek(&input[..]).unwrap();
        assert_eq!(result.graph.vcount(), 5);
        assert_eq!(result.graph.ecount(), 0);
    }

    #[test]
    fn test_quoted_label_with_escape() {
        let input = b"*Vertices 1\n1 \"hello \\\"world\\\"\"\n*Edges\n";
        let result = read_pajek(&input[..]).unwrap();
        let labels = result.labels.unwrap();
        assert_eq!(labels[0], "hello \"world\"");
    }

    #[test]
    fn test_write_undirected() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let labels = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let mut buf = Vec::new();
        write_pajek(&g, Some(&labels), None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("*Vertices 3"));
        assert!(s.contains("1 \"A\""));
        assert!(s.contains("2 \"B\""));
        assert!(s.contains("3 \"C\""));
        assert!(s.contains("*Edges"));
        assert!(s.contains("1 2\n"));
        assert!(s.contains("2 3\n"));
    }

    #[test]
    fn test_write_directed() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();

        let mut buf = Vec::new();
        write_pajek(&g, None, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("*Arcs"));
    }

    #[test]
    fn test_write_weighted() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();

        let weights = vec![4.5];
        let mut buf = Vec::new();
        write_pajek(&g, None, Some(&weights), &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("1 2 4.5"));
    }

    #[test]
    fn test_write_labels_mismatch_error() {
        let g = Graph::with_vertices(3);
        let labels = vec!["A".to_string()];
        let mut buf = Vec::new();
        assert!(write_pajek(&g, Some(&labels), None, &mut buf).is_err());
    }

    #[test]
    fn test_write_weights_mismatch_error() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let weights = vec![1.0, 2.0];
        let mut buf = Vec::new();
        assert!(write_pajek(&g, None, Some(&weights), &mut buf).is_err());
    }

    #[test]
    fn test_round_trip() {
        let input = b"*Vertices 4\n1 \"Alice\"\n2 \"Bob\"\n3 \"Carol\"\n4 \"Dave\"\n*Edges\n1 2 1.0\n2 3 2.0\n3 4 3.0\n";
        let result = read_pajek(&input[..]).unwrap();

        let mut buf = Vec::new();
        write_pajek(
            &result.graph,
            result.labels.as_deref(),
            result.weights.as_deref(),
            &mut buf,
        )
        .unwrap();

        let result2 = read_pajek(&buf[..]).unwrap();
        assert_eq!(result2.graph.vcount(), result.graph.vcount());
        assert_eq!(result2.graph.ecount(), result.graph.ecount());
        assert_eq!(result2.labels, result.labels);
    }

    #[test]
    fn test_blank_lines_skipped() {
        let input = b"\n*Vertices 2\n\n1 \"X\"\n\n*Edges\n\n1 2\n\n";
        let result = read_pajek(&input[..]).unwrap();
        assert_eq!(result.graph.vcount(), 2);
        assert_eq!(result.graph.ecount(), 1);
    }

    // --- attribute integration tests ---

    #[test]
    fn read_stores_name_attribute() {
        let input = b"*Vertices 3\n1 \"Alice\"\n2 \"Bob\"\n3 \"Carol\"\n*Edges\n1 2\n";
        let result = read_pajek(&input[..]).unwrap();
        assert_eq!(
            result
                .graph
                .vertex_attribute("name", 0)
                .and_then(AttributeValue::as_str),
            Some("Alice"),
        );
        assert_eq!(
            result
                .graph
                .vertex_attribute("name", 2)
                .and_then(AttributeValue::as_str),
            Some("Carol"),
        );
    }

    #[test]
    fn read_stores_weight_attribute() {
        let input = b"*Vertices 3\n*Edges\n1 2 1.5\n2 3 2.5\n";
        let result = read_pajek(&input[..]).unwrap();
        assert_eq!(
            result
                .graph
                .edge_attribute("weight", 0)
                .and_then(AttributeValue::as_f64),
            Some(1.5),
        );
        assert_eq!(
            result
                .graph
                .edge_attribute("weight", 1)
                .and_then(AttributeValue::as_f64),
            Some(2.5),
        );
    }

    #[test]
    fn read_no_labels_no_name_attribute() {
        let input = b"*Vertices 2\n*Edges\n1 2\n";
        let result = read_pajek(&input[..]).unwrap();
        assert!(!result.graph.has_vertex_attribute("name"));
    }

    #[test]
    fn write_uses_name_attribute_fallback() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.set_vertex_attribute_all(
            "name",
            vec![
                AttributeValue::String("X".into()),
                AttributeValue::String("Y".into()),
            ],
        )
        .unwrap();

        let mut buf = Vec::new();
        write_pajek(&g, None, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("\"X\""));
        assert!(s.contains("\"Y\""));
    }

    #[test]
    fn write_uses_weight_attribute_fallback() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.set_edge_attribute_all("weight", vec![AttributeValue::Numeric(4.25)])
            .unwrap();

        let mut buf = Vec::new();
        write_pajek(&g, None, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("1 2 4.25"));
    }

    #[test]
    fn roundtrip_with_attributes() {
        let input = b"*Vertices 3\n1 \"Alice\"\n2 \"Bob\"\n3 \"Carol\"\n*Edges\n1 2 1.5\n2 3 2.0\n";
        let result = read_pajek(&input[..]).unwrap();

        // Write using attribute fallback (no explicit labels/weights)
        let mut buf = Vec::new();
        write_pajek(&result.graph, None, None, &mut buf).unwrap();

        let result2 = read_pajek(&buf[..]).unwrap();
        assert_eq!(result2.graph.vcount(), 3);
        assert_eq!(result2.graph.ecount(), 2);
        assert_eq!(
            result2
                .graph
                .vertex_attribute("name", 0)
                .and_then(AttributeValue::as_str),
            Some("Alice"),
        );
        assert_eq!(
            result2
                .graph
                .edge_attribute("weight", 0)
                .and_then(AttributeValue::as_f64),
            Some(1.5),
        );
    }
}

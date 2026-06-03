//! LEDA native graph format reader and writer (ALGO-IO-007 / IO-012).
//!
//! The LEDA native graph format is:
//!
//! ```text
//! LEDA.GRAPH
//! string
//! void
//! -2
//! # Vertices
//! 3
//! |{Alice}|
//! |{Bob}|
//! |{Carol}|
//! # Edges
//! 2
//! 1 2 0 |{}|
//! 2 3 0 |{}|
//! ```
//!
//! The header lines specify vertex/edge attribute types (`void`, `string`,
//! `double`). The directedness flag is `-1` for directed, `-2` for
//! undirected. Each edge line has: source target reversal label.
//!
//! Counterpart of `igraph_write_graph_leda` / `igraph_read_graph_leda`.

use std::io::{BufRead, BufReader, Read, Write};

use crate::core::attributes::AttributeValue;
use crate::core::{Graph, IgraphError, IgraphResult};

/// Result of reading a LEDA file.
#[derive(Debug, Clone)]
pub struct LedaGraph {
    /// The parsed graph.
    pub graph: Graph,
    /// Vertex labels (if the vertex attribute type is `string`).
    pub labels: Option<Vec<String>>,
    /// Edge weights (if the edge attribute type is `double`).
    pub weights: Option<Vec<f64>>,
}

/// Read a graph from LEDA native graph format.
///
/// Parses the header (`LEDA.GRAPH`), attribute type declarations,
/// directedness flag, vertex section, and edge section. Vertex IDs in
/// the file are 1-based and converted to 0-based internally.
///
/// # Examples
///
/// ```
/// use rust_igraph::read_leda;
///
/// let input = b"LEDA.GRAPH\nvoid\nvoid\n-2\n# Vertices\n3\n|{}|\n|{}|\n|{}|\n# Edges\n2\n1 2 0 |{}|\n2 3 0 |{}|\n";
/// let result = read_leda(&input[..]).unwrap();
/// assert_eq!(result.graph.vcount(), 3);
/// assert_eq!(result.graph.ecount(), 2);
/// assert!(!result.graph.is_directed());
/// ```
pub fn read_leda<R: Read>(input: R) -> IgraphResult<LedaGraph> {
    let reader = BufReader::new(input);
    let mut content_lines: Vec<String> = Vec::new();
    for line_result in reader.lines() {
        let line = line_result?;
        let trimmed = line.trim().to_string();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            content_lines.push(trimmed);
        }
    }

    let get = |i: usize, msg: &str| -> IgraphResult<&str> {
        content_lines
            .get(i)
            .map(String::as_str)
            .ok_or_else(|| leda_parse_err(i, msg))
    };

    if get(0, "empty LEDA file")? != "LEDA.GRAPH" {
        return Err(leda_parse_err(0, "LEDA file must start with 'LEDA.GRAPH'"));
    }
    let has_labels = get(1, "missing vertex attribute type")?.eq_ignore_ascii_case("string");
    let has_weights = get(2, "missing edge attribute type")?.eq_ignore_ascii_case("double");
    let directed = match get(3, "missing directedness flag")? {
        "-1" => true,
        "-2" => false,
        other => {
            return Err(leda_parse_err(
                3,
                &format!("invalid directedness flag '{other}', expected -1 or -2"),
            ));
        }
    };
    let n: u32 = get(4, "missing vertex count")?
        .parse()
        .map_err(|e| leda_parse_err(4, &format!("invalid vertex count: {e}")))?;

    let mut pos = 5;
    let mut labels: Vec<String> = Vec::with_capacity(n as usize);
    for _ in 0..n {
        let vline = get(pos, "unexpected end in vertex section")?;
        let label = extract_leda_label(vline)
            .ok_or_else(|| leda_parse_err(pos, &format!("invalid vertex entry: {vline}")))?;
        labels.push(label);
        pos += 1;
    }

    let parsed = parse_leda_edges(&content_lines, pos, n, has_weights)?;

    let mut graph = Graph::new(n, directed)?;
    graph.add_edges(parsed.edges)?;

    if has_labels {
        graph.set_vertex_attribute_all(
            "name",
            labels
                .iter()
                .map(|l| AttributeValue::String(l.clone()))
                .collect(),
        )?;
    }
    if has_weights {
        graph.set_edge_attribute_all(
            "weight",
            parsed
                .weights
                .iter()
                .map(|&w| AttributeValue::Numeric(w))
                .collect(),
        )?;
    }

    Ok(LedaGraph {
        graph,
        labels: if has_labels { Some(labels) } else { None },
        weights: if has_weights {
            Some(parsed.weights)
        } else {
            None
        },
    })
}

struct LedaEdges {
    edges: Vec<(u32, u32)>,
    weights: Vec<f64>,
}

fn parse_leda_edges(
    lines: &[String],
    start: usize,
    n: u32,
    has_weights: bool,
) -> IgraphResult<LedaEdges> {
    let m_str = lines
        .get(start)
        .ok_or_else(|| leda_parse_err(start, "missing edge count"))?;
    let m: usize = m_str
        .parse()
        .map_err(|e| leda_parse_err(start, &format!("invalid edge count: {e}")))?;

    let mut edges: Vec<(u32, u32)> = Vec::with_capacity(m);
    let mut weights: Vec<f64> = Vec::with_capacity(m);
    for i in 0..m {
        let pos = start + 1 + i;
        let eline = lines
            .get(pos)
            .ok_or_else(|| leda_parse_err(pos, "unexpected end in edge section"))?;
        let tokens: Vec<&str> = eline.splitn(4, ' ').collect();
        if tokens.len() < 3 {
            return Err(leda_parse_err(
                pos,
                &format!("edge line needs at least 3 fields: {eline}"),
            ));
        }
        let from: u32 = tokens[0]
            .parse()
            .map_err(|e| leda_parse_err(pos, &format!("invalid source id: {e}")))?;
        let to: u32 = tokens[1]
            .parse()
            .map_err(|e| leda_parse_err(pos, &format!("invalid target id: {e}")))?;
        if from == 0 || to == 0 || from > n || to > n {
            return Err(leda_parse_err(
                pos,
                &format!("vertex ID out of range: {from}->{to} (n={n})"),
            ));
        }
        edges.push((from - 1, to - 1));

        if has_weights {
            let label = tokens
                .get(3)
                .and_then(|s| extract_leda_label(s))
                .unwrap_or_default();
            let w: f64 = if label.is_empty() {
                0.0
            } else {
                label
                    .parse()
                    .map_err(|e| leda_parse_err(pos, &format!("invalid edge weight: {e}")))?
            };
            weights.push(w);
        }
    }
    Ok(LedaEdges { edges, weights })
}

fn leda_parse_err(line: usize, msg: &str) -> IgraphError {
    IgraphError::Parse {
        line,
        message: msg.to_string(),
    }
}

fn extract_leda_label(s: &str) -> Option<String> {
    let trimmed = s.trim();
    let inner = trimmed.strip_prefix("|{")?.strip_suffix("}|")?;
    Some(inner.to_string())
}

/// Write a graph in LEDA native graph format.
///
/// Vertex labels are written as string attributes if provided.
/// Edge weights are written as double attributes if provided.
/// The `reversal` field for edges is always 0 (no reverse edge pointer).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, write_leda};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
///
/// let labels = vec!["A".to_string(), "B".to_string(), "C".to_string()];
/// let mut buf = Vec::new();
/// write_leda(&g, Some(&labels), None, &mut buf).unwrap();
/// let s = String::from_utf8(buf).unwrap();
/// assert!(s.contains("LEDA.GRAPH"));
/// assert!(s.contains("|{A}|"));
/// ```
pub fn write_leda<W: Write>(
    graph: &Graph,
    vertex_labels: Option<&[String]>,
    edge_weights: Option<&[f64]>,
    writer: &mut W,
) -> IgraphResult<()> {
    if let Some(l) = vertex_labels {
        if l.len() != graph.vcount() as usize {
            return Err(IgraphError::InvalidArgument(format!(
                "vertex_labels length {} does not match vcount {}",
                l.len(),
                graph.vcount()
            )));
        }
        for (i, lbl) in l.iter().enumerate() {
            if lbl.contains('\n') {
                return Err(IgraphError::InvalidArgument(format!(
                    "vertex label at index {i} contains a newline character"
                )));
            }
        }
    }
    if let Some(w) = edge_weights {
        if w.len() != graph.ecount() {
            return Err(IgraphError::InvalidArgument(format!(
                "edge_weights length {} does not match ecount {}",
                w.len(),
                graph.ecount()
            )));
        }
    }

    let has_attr_labels =
        vertex_labels.is_none() && graph.vertex_attribute_names().contains(&"name");
    let has_attr_weights =
        edge_weights.is_none() && graph.edge_attribute_names().contains(&"weight");

    // Header
    writeln!(writer, "LEDA.GRAPH")?;

    // Vertex attribute type
    if vertex_labels.is_some() || has_attr_labels {
        writeln!(writer, "string")?;
    } else {
        writeln!(writer, "void")?;
    }

    // Edge attribute type
    if edge_weights.is_some() || has_attr_weights {
        writeln!(writer, "double")?;
    } else {
        writeln!(writer, "void")?;
    }

    // Directedness: -1 = directed, -2 = undirected
    if graph.is_directed() {
        writeln!(writer, "-1")?;
    } else {
        writeln!(writer, "-2")?;
    }

    // Vertices section
    writeln!(writer, "# Vertices")?;
    writeln!(writer, "{}", graph.vcount())?;

    for v in 0..graph.vcount() {
        match vertex_labels {
            Some(labels) => writeln!(writer, "|{{{}}}|", labels[v as usize])?,
            None => {
                if has_attr_labels {
                    let label = graph
                        .vertex_attribute("name", v)
                        .and_then(AttributeValue::as_str)
                        .unwrap_or("");
                    writeln!(writer, "|{{{label}}}|")?;
                } else {
                    writeln!(writer, "|{{}}|")?;
                }
            }
        }
    }

    // Edges section
    writeln!(writer, "# Edges")?;
    writeln!(writer, "{}", graph.ecount())?;

    for eid in 0..graph.ecount() {
        #[allow(clippy::cast_possible_truncation)]
        let (from, to) = graph.edge(eid as u32)?;

        if let Some(w) = edge_weights {
            writeln!(writer, "{} {} 0 |{{{}}}|", from + 1, to + 1, w[eid])?;
        } else {
            #[allow(clippy::cast_possible_truncation)]
            let eid_u32 = eid as u32;
            if let Some(w) = graph
                .edge_attribute("weight", eid_u32)
                .and_then(AttributeValue::as_f64)
            {
                writeln!(writer, "{} {} 0 |{{{w}}}|", from + 1, to + 1)?;
            } else {
                writeln!(writer, "{} {} 0 |{{}}|", from + 1, to + 1)?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_undirected() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let mut buf = Vec::new();
        write_leda(&g, None, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();

        assert!(s.starts_with("LEDA.GRAPH\n"));
        assert!(s.contains("void\nvoid\n-2\n"));
        assert!(s.contains("# Vertices\n3\n"));
        assert!(s.contains("|{}|\n|{}|\n|{}|\n"));
        assert!(s.contains("# Edges\n2\n"));
        assert!(s.contains("1 2 0 |{}|\n"));
        assert!(s.contains("2 3 0 |{}|\n"));
    }

    #[test]
    fn test_directed() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();

        let mut buf = Vec::new();
        write_leda(&g, None, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();

        assert!(s.contains("-1\n"));
    }

    #[test]
    fn test_with_labels() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();

        let labels = vec!["Alice".to_string(), "Bob".to_string(), "Carol".to_string()];
        let mut buf = Vec::new();
        write_leda(&g, Some(&labels), None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();

        assert!(s.contains("string\nvoid\n"));
        assert!(s.contains("|{Alice}|\n"));
        assert!(s.contains("|{Bob}|\n"));
        assert!(s.contains("|{Carol}|\n"));
    }

    #[test]
    fn test_with_weights() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();

        let weights = vec![3.5];
        let mut buf = Vec::new();
        write_leda(&g, None, Some(&weights), &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();

        assert!(s.contains("void\ndouble\n"));
        assert!(s.contains("1 2 0 |{3.5}|\n"));
    }

    #[test]
    fn test_with_labels_and_weights() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();

        let labels = vec!["X".to_string(), "Y".to_string()];
        let weights = vec![1.25];
        let mut buf = Vec::new();
        write_leda(&g, Some(&labels), Some(&weights), &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();

        assert!(s.contains("string\ndouble\n"));
        assert!(s.contains("|{X}|\n"));
        assert!(s.contains("|{Y}|\n"));
        assert!(s.contains("1 2 0 |{1.25}|\n"));
    }

    #[test]
    fn test_empty_graph() {
        let g = Graph::with_vertices(0);

        let mut buf = Vec::new();
        write_leda(&g, None, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();

        assert!(s.contains("# Vertices\n0\n"));
        assert!(s.contains("# Edges\n0\n"));
    }

    #[test]
    fn test_no_edges() {
        let g = Graph::with_vertices(3);

        let mut buf = Vec::new();
        write_leda(&g, None, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();

        assert!(s.contains("# Vertices\n3\n"));
        assert!(s.contains("# Edges\n0\n"));
    }

    #[test]
    fn test_label_mismatch_error() {
        let g = Graph::with_vertices(3);
        let labels = vec!["A".to_string()];
        let mut buf = Vec::new();
        assert!(write_leda(&g, Some(&labels), None, &mut buf).is_err());
    }

    #[test]
    fn test_weight_mismatch_error() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let weights = vec![1.0, 2.0];
        let mut buf = Vec::new();
        assert!(write_leda(&g, None, Some(&weights), &mut buf).is_err());
    }

    #[test]
    fn test_newline_in_label_error() {
        let g = Graph::with_vertices(2);
        let labels = vec!["hello\nworld".to_string(), "ok".to_string()];
        let mut buf = Vec::new();
        assert!(write_leda(&g, Some(&labels), None, &mut buf).is_err());
    }

    #[test]
    fn test_self_loop() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();

        let mut buf = Vec::new();
        write_leda(&g, None, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();

        assert!(s.contains("1 1 0 |{}|\n"));
    }

    #[test]
    fn test_one_based_vertex_ids() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(2, 3).unwrap();

        let mut buf = Vec::new();
        write_leda(&g, None, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();

        assert!(s.contains("3 4 0 |{}|\n"));
    }

    // --- read_leda tests ---

    #[test]
    fn test_read_basic_undirected() {
        let input = b"LEDA.GRAPH\nvoid\nvoid\n-2\n# Vertices\n3\n|{}|\n|{}|\n|{}|\n# Edges\n2\n1 2 0 |{}|\n2 3 0 |{}|\n";
        let result = read_leda(&input[..]).unwrap();
        assert_eq!(result.graph.vcount(), 3);
        assert_eq!(result.graph.ecount(), 2);
        assert!(!result.graph.is_directed());
        assert!(result.labels.is_none());
        assert!(result.weights.is_none());
    }

    #[test]
    fn test_read_directed() {
        let input =
            b"LEDA.GRAPH\nvoid\nvoid\n-1\n# Vertices\n2\n|{}|\n|{}|\n# Edges\n1\n1 2 0 |{}|\n";
        let result = read_leda(&input[..]).unwrap();
        assert!(result.graph.is_directed());
        assert_eq!(result.graph.ecount(), 1);
    }

    #[test]
    fn test_read_with_labels() {
        let input = b"LEDA.GRAPH\nstring\nvoid\n-2\n# Vertices\n3\n|{Alice}|\n|{Bob}|\n|{Carol}|\n# Edges\n1\n1 2 0 |{}|\n";
        let result = read_leda(&input[..]).unwrap();
        let labels = result.labels.unwrap();
        assert_eq!(labels, vec!["Alice", "Bob", "Carol"]);
    }

    #[test]
    fn test_read_with_weights() {
        let input =
            b"LEDA.GRAPH\nvoid\ndouble\n-2\n# Vertices\n2\n|{}|\n|{}|\n# Edges\n1\n1 2 0 |{3.5}|\n";
        let result = read_leda(&input[..]).unwrap();
        let w = result.weights.unwrap();
        assert!((w[0] - 3.5).abs() < 1e-10);
    }

    #[test]
    fn test_read_with_labels_and_weights() {
        let input = b"LEDA.GRAPH\nstring\ndouble\n-2\n# Vertices\n2\n|{X}|\n|{Y}|\n# Edges\n1\n1 2 0 |{1.25}|\n";
        let result = read_leda(&input[..]).unwrap();
        let labels = result.labels.unwrap();
        assert_eq!(labels, vec!["X", "Y"]);
        let w = result.weights.unwrap();
        assert!((w[0] - 1.25).abs() < 1e-10);
    }

    #[test]
    fn test_read_empty_graph() {
        let input = b"LEDA.GRAPH\nvoid\nvoid\n-2\n# Vertices\n0\n# Edges\n0\n";
        let result = read_leda(&input[..]).unwrap();
        assert_eq!(result.graph.vcount(), 0);
        assert_eq!(result.graph.ecount(), 0);
    }

    #[test]
    fn test_read_no_edges() {
        let input = b"LEDA.GRAPH\nvoid\nvoid\n-2\n# Vertices\n3\n|{}|\n|{}|\n|{}|\n# Edges\n0\n";
        let result = read_leda(&input[..]).unwrap();
        assert_eq!(result.graph.vcount(), 3);
        assert_eq!(result.graph.ecount(), 0);
    }

    #[test]
    fn test_read_self_loop() {
        let input =
            b"LEDA.GRAPH\nvoid\nvoid\n-2\n# Vertices\n2\n|{}|\n|{}|\n# Edges\n1\n1 1 0 |{}|\n";
        let result = read_leda(&input[..]).unwrap();
        assert_eq!(result.graph.ecount(), 1);
        let (from, to) = result.graph.edge(0).unwrap();
        assert_eq!(from, 0);
        assert_eq!(to, 0);
    }

    #[test]
    fn test_read_bad_header() {
        let input = b"NOT_LEDA\nvoid\nvoid\n-2\n";
        assert!(read_leda(&input[..]).is_err());
    }

    #[test]
    fn test_read_bad_directedness() {
        let input = b"LEDA.GRAPH\nvoid\nvoid\n-3\n# Vertices\n0\n# Edges\n0\n";
        assert!(read_leda(&input[..]).is_err());
    }

    #[test]
    fn test_read_vertex_id_out_of_range() {
        let input =
            b"LEDA.GRAPH\nvoid\nvoid\n-2\n# Vertices\n2\n|{}|\n|{}|\n# Edges\n1\n1 5 0 |{}|\n";
        assert!(read_leda(&input[..]).is_err());
    }

    #[test]
    fn test_roundtrip_undirected() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();

        let mut buf = Vec::new();
        write_leda(&g, None, None, &mut buf).unwrap();
        let result = read_leda(&buf[..]).unwrap();

        assert_eq!(result.graph.vcount(), g.vcount());
        assert_eq!(result.graph.ecount(), g.ecount());
        assert_eq!(result.graph.is_directed(), g.is_directed());
    }

    #[test]
    fn test_roundtrip_directed() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let mut buf = Vec::new();
        write_leda(&g, None, None, &mut buf).unwrap();
        let result = read_leda(&buf[..]).unwrap();

        assert_eq!(result.graph.vcount(), g.vcount());
        assert_eq!(result.graph.ecount(), g.ecount());
        assert!(result.graph.is_directed());
    }

    #[test]
    fn test_roundtrip_with_labels() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let labels = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let mut buf = Vec::new();
        write_leda(&g, Some(&labels), None, &mut buf).unwrap();
        let result = read_leda(&buf[..]).unwrap();

        assert_eq!(result.labels.unwrap(), labels);
    }

    #[test]
    fn test_roundtrip_with_weights() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();

        let weights = vec![2.75];
        let mut buf = Vec::new();
        write_leda(&g, None, Some(&weights), &mut buf).unwrap();
        let result = read_leda(&buf[..]).unwrap();

        let w = result.weights.unwrap();
        assert!((w[0] - 2.75).abs() < 1e-10);
    }

    #[test]
    fn test_read_empty_label() {
        let input = b"LEDA.GRAPH\nstring\nvoid\n-2\n# Vertices\n2\n|{}|\n|{hello}|\n# Edges\n0\n";
        let result = read_leda(&input[..]).unwrap();
        let labels = result.labels.unwrap();
        assert_eq!(labels[0], "");
        assert_eq!(labels[1], "hello");
    }

    // --- Attribute integration tests ---

    #[test]
    fn test_read_stores_label_attribute() {
        let input =
            b"LEDA.GRAPH\nstring\nvoid\n-2\n# Vertices\n2\n|{Alice}|\n|{Bob}|\n# Edges\n1\n1 2 0 |{}|\n";
        let result = read_leda(&input[..]).unwrap();
        assert_eq!(
            result
                .graph
                .vertex_attribute("name", 0)
                .and_then(AttributeValue::as_str),
            Some("Alice")
        );
        assert_eq!(
            result
                .graph
                .vertex_attribute("name", 1)
                .and_then(AttributeValue::as_str),
            Some("Bob")
        );
    }

    #[test]
    fn test_read_stores_weight_attribute() {
        let input =
            b"LEDA.GRAPH\nvoid\ndouble\n-2\n# Vertices\n2\n|{}|\n|{}|\n# Edges\n1\n1 2 0 |{4.5}|\n";
        let result = read_leda(&input[..]).unwrap();
        let w = result
            .graph
            .edge_attribute("weight", 0)
            .and_then(AttributeValue::as_f64)
            .unwrap();
        assert!((w - 4.5).abs() < 1e-10);
    }

    #[test]
    fn test_read_no_label_attribute_when_void() {
        let input = b"LEDA.GRAPH\nvoid\nvoid\n-2\n# Vertices\n2\n|{}|\n|{}|\n# Edges\n0\n";
        let result = read_leda(&input[..]).unwrap();
        assert!(result.graph.vertex_attribute("name", 0).is_none());
    }

    #[test]
    fn test_write_fallback_to_label_attribute() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.set_vertex_attribute("name", 0, AttributeValue::String("X".into()))
            .unwrap();
        g.set_vertex_attribute("name", 1, AttributeValue::String("Y".into()))
            .unwrap();

        let mut buf = Vec::new();
        write_leda(&g, None, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("string"));
        assert!(s.contains("|{X}|"));
        assert!(s.contains("|{Y}|"));
    }

    #[test]
    fn test_write_fallback_to_weight_attribute() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.set_edge_attribute("weight", 0, AttributeValue::Numeric(7.5))
            .unwrap();

        let mut buf = Vec::new();
        write_leda(&g, None, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("double"));
        assert!(s.contains("|{7.5}|"));
    }

    #[test]
    fn test_roundtrip_via_attributes() {
        let input = b"LEDA.GRAPH\nstring\ndouble\n-2\n# Vertices\n2\n|{Alice}|\n|{Bob}|\n# Edges\n1\n1 2 0 |{1.5}|\n";
        let result = read_leda(&input[..]).unwrap();

        let mut buf = Vec::new();
        write_leda(&result.graph, None, None, &mut buf).unwrap();

        let result2 = read_leda(&buf[..]).unwrap();
        assert_eq!(result2.graph.vcount(), 2);
        assert_eq!(result2.graph.ecount(), 1);
        assert!(result2.labels.is_some());
        assert!(result2.weights.is_some());
        let labels = result2.labels.unwrap();
        assert_eq!(labels, vec!["Alice", "Bob"]);
    }

    #[test]
    fn test_explicit_params_override_attributes() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.set_vertex_attribute("name", 0, AttributeValue::String("attr_A".into()))
            .unwrap();
        g.set_vertex_attribute("name", 1, AttributeValue::String("attr_B".into()))
            .unwrap();

        let labels = vec!["explicit_A".to_string(), "explicit_B".to_string()];
        let mut buf = Vec::new();
        write_leda(&g, Some(&labels), None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("|{explicit_A}|"));
        assert!(!s.contains("attr_A"));
    }
}

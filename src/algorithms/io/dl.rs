//! UCINET DL format reader and writer (ALGO-IO-009 / IO-013).
//!
//! Reads and writes graphs in the DL format used by UCINET. Three data
//! representations are supported for reading:
//!
//! - **fullmatrix**: adjacency matrix of 0/1 values
//! - **edgelist1**: pairs of 1-based vertex IDs with optional weights
//! - **nodelist1**: source vertex followed by its neighbors (1-based)
//!
//! Writing always uses **edgelist1** format.
//!
//! ```text
//! DL n=5
//! format = edgelist1
//! data:
//! 1 2
//! 1 3
//! 2 3
//! ```
//!
//! Vertex labels can be provided via `labels:` or `labels embedded:`.
//!
//! Counterpart of `igraph_read_graph_dl` / `igraph_write_graph_dl`.

use std::io::{BufRead, BufReader, Read, Write};

use crate::core::{Graph, IgraphError, IgraphResult};

/// Result of reading a DL file.
#[derive(Debug, Clone)]
pub struct DlGraph {
    /// The parsed graph.
    pub graph: Graph,
    /// Vertex labels (if provided).
    pub labels: Option<Vec<String>>,
    /// Edge weights (if provided, only for edgelist1 format).
    pub weights: Option<Vec<f64>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum DlFormat {
    FullMatrix,
    EdgeList1,
    NodeList1,
}

/// Read a graph from UCINET DL format.
///
/// Supports fullmatrix, edgelist1, and nodelist1 data formats.
/// Case-insensitive keywords. Vertex IDs in edgelist1 and nodelist1
/// are 1-based.
///
/// # Examples
///
/// ```
/// use rust_igraph::read_dl;
///
/// let input = b"DL n=3\nformat = edgelist1\ndata:\n1 2\n2 3\n1 3\n";
/// let result = read_dl(&input[..], true).unwrap();
/// assert_eq!(result.graph.vcount(), 3);
/// assert_eq!(result.graph.ecount(), 3);
/// ```
#[allow(clippy::too_many_lines)]
pub fn read_dl<R: Read>(input: R, directed: bool) -> IgraphResult<DlGraph> {
    let reader = BufReader::new(input);
    let mut lines: Vec<String> = Vec::new();

    for line_result in reader.lines() {
        let line = line_result?;
        lines.push(line);
    }

    let mut pos = 0;
    let mut n_vertices: Option<u32> = None;
    let mut format = DlFormat::FullMatrix;
    let mut labels: Vec<String> = Vec::new();
    let mut labels_embedded = false;

    // Parse header: DL n=N
    pos = skip_empty(&lines, pos);
    if pos >= lines.len() {
        return Err(parse_err(0, "empty DL file"));
    }

    let header_line = lines[pos].trim().to_ascii_lowercase();
    if !header_line.starts_with("dl") {
        return Err(parse_err(pos + 1, "DL file must start with 'DL'"));
    }

    // Extract n= from header line or subsequent lines
    let header_rest = &lines[pos].trim()[2..];
    if let Some(n) = extract_n(header_rest) {
        n_vertices = Some(n);
    }
    pos += 1;

    // Parse directives until DATA:
    loop {
        pos = skip_empty(&lines, pos);
        if pos >= lines.len() {
            break;
        }

        let trimmed = lines[pos].trim();
        let lower = trimmed.to_ascii_lowercase();

        if lower.starts_with("data:") || lower == "data" {
            pos += 1;
            break;
        }

        if lower.starts_with("n=") || lower.starts_with("n =") {
            if n_vertices.is_none() {
                let val =
                    extract_n(trimmed).ok_or_else(|| parse_err(pos + 1, "invalid n= value"))?;
                n_vertices = Some(val);
            }
            pos += 1;
            continue;
        }

        if lower.starts_with("format") {
            if lower.contains("fullmatrix") || lower.contains("full matrix") {
                format = DlFormat::FullMatrix;
            } else if lower.contains("edgelist1")
                || lower.contains("edge list1")
                || lower.contains("edgelist 1")
            {
                format = DlFormat::EdgeList1;
            } else if lower.contains("nodelist1")
                || lower.contains("node list1")
                || lower.contains("nodelist 1")
            {
                format = DlFormat::NodeList1;
            }
            pos += 1;
            continue;
        }

        if lower.starts_with("labels embedded") {
            labels_embedded = true;
            pos += 1;
            continue;
        }

        if lower.starts_with("labels:") || lower == "labels" {
            pos += 1;
            // Read label lines until next keyword or data:
            while pos < lines.len() {
                let lbl_line = lines[pos].trim();
                let lbl_lower = lbl_line.to_ascii_lowercase();
                if lbl_lower.starts_with("data")
                    || lbl_lower.starts_with("format")
                    || lbl_lower.starts_with("labels")
                {
                    break;
                }
                if !lbl_line.is_empty() {
                    // Labels can be comma or whitespace separated on one line
                    for token in split_labels(lbl_line) {
                        labels.push(token);
                    }
                }
                pos += 1;
            }
            continue;
        }

        // Try to extract n= from lines like "N = 5"
        if let Some(n) = extract_n(trimmed) {
            if n_vertices.is_none() {
                n_vertices = Some(n);
            }
            pos += 1;
            continue;
        }

        pos += 1;
    }

    let n = n_vertices.ok_or_else(|| parse_err(0, "no vertex count (n=) found in DL file"))?;

    let mut edges: Vec<(u32, u32)> = Vec::new();
    let mut weights: Vec<f64> = Vec::new();
    let mut has_weights = false;

    match format {
        DlFormat::FullMatrix => {
            if labels_embedded {
                // First row is label row, then labeled rows
                pos = skip_empty(&lines, pos);
                if pos < lines.len() {
                    let header_labels = split_labels(lines[pos].trim());
                    if labels.is_empty() {
                        labels = header_labels;
                    }
                    pos += 1;
                }
                let mut row = 0u32;
                while pos < lines.len() && row < n {
                    let trimmed = lines[pos].trim();
                    if trimmed.is_empty() {
                        pos += 1;
                        continue;
                    }
                    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
                    if tokens.is_empty() {
                        pos += 1;
                        continue;
                    }
                    // First token is the row label (skip), rest are 0/1
                    let data_start = 1;
                    for (col, &val) in tokens.iter().skip(data_start).enumerate() {
                        if val == "1" {
                            #[allow(clippy::cast_possible_truncation)]
                            edges.push((row, col as u32));
                        }
                    }
                    row += 1;
                    pos += 1;
                }
            } else {
                let mut row = 0u32;
                while pos < lines.len() && row < n {
                    let trimmed = lines[pos].trim();
                    if trimmed.is_empty() {
                        pos += 1;
                        continue;
                    }
                    for (col, ch) in trimmed.split_whitespace().enumerate() {
                        if ch == "1" {
                            #[allow(clippy::cast_possible_truncation)]
                            edges.push((row, col as u32));
                        }
                    }
                    row += 1;
                    pos += 1;
                }
            }
        }
        DlFormat::EdgeList1 => {
            if labels_embedded {
                while pos < lines.len() {
                    let trimmed = lines[pos].trim();
                    if trimmed.is_empty() {
                        pos += 1;
                        continue;
                    }
                    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
                    if tokens.len() < 2 {
                        pos += 1;
                        continue;
                    }
                    let from_label = tokens[0].to_string();
                    let to_label = tokens[1].to_string();
                    let from_id = get_or_add_label(&mut labels, &from_label);
                    let to_id = get_or_add_label(&mut labels, &to_label);
                    edges.push((from_id, to_id));
                    if tokens.len() >= 3 {
                        if let Ok(w) = tokens[2].parse::<f64>() {
                            has_weights = true;
                            weights.push(w);
                        } else {
                            weights.push(0.0);
                        }
                    } else {
                        weights.push(0.0);
                    }
                    pos += 1;
                }
            } else {
                while pos < lines.len() {
                    let trimmed = lines[pos].trim();
                    if trimmed.is_empty() {
                        pos += 1;
                        continue;
                    }
                    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
                    if tokens.len() < 2 {
                        pos += 1;
                        continue;
                    }
                    let from: u32 = tokens[0]
                        .parse()
                        .map_err(|e| parse_err(pos + 1, &format!("invalid source id: {e}")))?;
                    let to: u32 = tokens[1]
                        .parse()
                        .map_err(|e| parse_err(pos + 1, &format!("invalid target id: {e}")))?;
                    if from == 0 || to == 0 || from > n || to > n {
                        return Err(parse_err(
                            pos + 1,
                            &format!("vertex ID out of range: {from} {to} (n={n})"),
                        ));
                    }
                    edges.push((from - 1, to - 1));
                    if tokens.len() >= 3 {
                        if let Ok(w) = tokens[2].parse::<f64>() {
                            has_weights = true;
                            weights.push(w);
                        } else {
                            weights.push(0.0);
                        }
                    } else {
                        weights.push(0.0);
                    }
                    pos += 1;
                }
            }
        }
        DlFormat::NodeList1 => {
            if labels_embedded {
                while pos < lines.len() {
                    let trimmed = lines[pos].trim();
                    if trimmed.is_empty() {
                        pos += 1;
                        continue;
                    }
                    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
                    if tokens.is_empty() {
                        pos += 1;
                        continue;
                    }
                    let from_label = tokens[0].to_string();
                    let from_id = get_or_add_label(&mut labels, &from_label);
                    for &tok in &tokens[1..] {
                        let to_label = tok.to_string();
                        let to_id = get_or_add_label(&mut labels, &to_label);
                        edges.push((from_id, to_id));
                    }
                    pos += 1;
                }
            } else {
                while pos < lines.len() {
                    let trimmed = lines[pos].trim();
                    if trimmed.is_empty() {
                        pos += 1;
                        continue;
                    }
                    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
                    if tokens.is_empty() {
                        pos += 1;
                        continue;
                    }
                    let from: u32 = tokens[0]
                        .parse()
                        .map_err(|e| parse_err(pos + 1, &format!("invalid source id: {e}")))?;
                    if from == 0 || from > n {
                        return Err(parse_err(
                            pos + 1,
                            &format!("source vertex ID out of range: {from} (n={n})"),
                        ));
                    }
                    for &tok in &tokens[1..] {
                        let to: u32 = tok
                            .parse()
                            .map_err(|e| parse_err(pos + 1, &format!("invalid target id: {e}")))?;
                        if to == 0 || to > n {
                            return Err(parse_err(
                                pos + 1,
                                &format!("target vertex ID out of range: {to} (n={n})"),
                            ));
                        }
                        edges.push((from - 1, to - 1));
                    }
                    pos += 1;
                }
            }
        }
    }

    let mut graph = Graph::new(n, directed)?;
    graph.add_edges(edges)?;

    let final_labels = if labels.is_empty() {
        None
    } else {
        Some(labels)
    };
    let final_weights = if has_weights { Some(weights) } else { None };

    Ok(DlGraph {
        graph,
        labels: final_labels,
        weights: final_weights,
    })
}

fn parse_err(line: usize, msg: &str) -> IgraphError {
    IgraphError::Parse {
        line,
        message: msg.to_string(),
    }
}

fn skip_empty(lines: &[String], mut pos: usize) -> usize {
    while pos < lines.len() && lines[pos].trim().is_empty() {
        pos += 1;
    }
    pos
}

fn extract_n(s: &str) -> Option<u32> {
    let lower = s.to_ascii_lowercase();
    // Look for n=N or n = N pattern
    for part in lower.split([',', ';']) {
        let trimmed = part.trim();
        if let Some(after_n) = trimmed.strip_prefix('n') {
            let rest = after_n.trim();
            if let Some(stripped) = rest.strip_prefix('=') {
                if let Ok(val) = stripped.trim().parse::<u32>() {
                    return Some(val);
                }
            }
        }
    }
    None
}

fn split_labels(s: &str) -> Vec<String> {
    if s.contains(',') {
        s.split(',')
            .map(|t| t.trim().trim_matches('"').to_string())
            .filter(|t| !t.is_empty())
            .collect()
    } else {
        s.split_whitespace()
            .map(|t| t.trim_matches('"').to_string())
            .collect()
    }
}

fn get_or_add_label(labels: &mut Vec<String>, name: &str) -> u32 {
    for (i, lbl) in labels.iter().enumerate() {
        if lbl == name {
            #[allow(clippy::cast_possible_truncation)]
            return i as u32;
        }
    }
    labels.push(name.to_string());
    #[allow(clippy::cast_possible_truncation)]
    let id = (labels.len() - 1) as u32;
    id
}

/// Write a graph in UCINET DL edgelist1 format.
///
/// Uses the `edgelist1` representation with 1-based vertex IDs. Vertex
/// labels are emitted as a `labels:` section if provided. Edge weights
/// appear as a third field on each edge line.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, write_dl};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
///
/// let mut buf = Vec::new();
/// write_dl(&g, None, None, &mut buf).unwrap();
/// let s = String::from_utf8(buf).unwrap();
/// assert!(s.contains("DL n=3"));
/// assert!(s.contains("format = edgelist1"));
/// ```
pub fn write_dl<W: Write>(
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

    writeln!(writer, "DL n={}", graph.vcount())?;
    writeln!(writer, "format = edgelist1")?;

    if let Some(labels) = vertex_labels {
        writeln!(writer, "labels:")?;
        let joined: Vec<&str> = labels.iter().map(String::as_str).collect();
        writeln!(writer, "{}", joined.join(","))?;
    }

    writeln!(writer, "data:")?;

    for eid in 0..graph.ecount() {
        #[allow(clippy::cast_possible_truncation)]
        let (from, to) = graph.edge(eid as u32)?;

        match edge_weights {
            Some(w) => writeln!(writer, "{} {} {}", from + 1, to + 1, w[eid])?,
            None => writeln!(writer, "{} {}", from + 1, to + 1)?,
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edgelist1_basic() {
        let input = b"DL n=3\nformat = edgelist1\ndata:\n1 2\n2 3\n1 3\n";
        let result = read_dl(&input[..], true).unwrap();
        assert_eq!(result.graph.vcount(), 3);
        assert_eq!(result.graph.ecount(), 3);
        assert!(result.graph.is_directed());
    }

    #[test]
    fn test_edgelist1_undirected() {
        let input = b"DL n=3\nformat = edgelist1\ndata:\n1 2\n2 3\n";
        let result = read_dl(&input[..], false).unwrap();
        assert_eq!(result.graph.vcount(), 3);
        assert_eq!(result.graph.ecount(), 2);
        assert!(!result.graph.is_directed());
    }

    #[test]
    fn test_edgelist1_with_weights() {
        let input = b"DL n=3\nformat = edgelist1\ndata:\n1 2 1.5\n2 3 2.5\n";
        let result = read_dl(&input[..], true).unwrap();
        let w = result.weights.unwrap();
        assert!((w[0] - 1.5).abs() < 1e-10);
        assert!((w[1] - 2.5).abs() < 1e-10);
    }

    #[test]
    fn test_edgelist1_with_labels() {
        let input = b"DL n=3\nformat = edgelist1\nlabels:\nA,B,C\ndata:\n1 2\n2 3\n";
        let result = read_dl(&input[..], true).unwrap();
        let labels = result.labels.unwrap();
        assert_eq!(labels, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_edgelist1_labels_embedded() {
        let input = b"DL n=3\nformat = edgelist1\nlabels embedded:\ndata:\nAlice Bob\nBob Carol\n";
        let result = read_dl(&input[..], true).unwrap();
        assert_eq!(result.graph.ecount(), 2);
        let labels = result.labels.unwrap();
        assert_eq!(labels[0], "Alice");
        assert_eq!(labels[1], "Bob");
        assert_eq!(labels[2], "Carol");
    }

    #[test]
    fn test_fullmatrix_basic() {
        let input = b"DL n=3\nformat = fullmatrix\ndata:\n0 1 0\n0 0 1\n1 0 0\n";
        let result = read_dl(&input[..], true).unwrap();
        assert_eq!(result.graph.vcount(), 3);
        assert_eq!(result.graph.ecount(), 3);
    }

    #[test]
    fn test_fullmatrix_default_format() {
        // No format line = default fullmatrix
        let input = b"DL n=3\ndata:\n0 1 1\n1 0 0\n0 1 0\n";
        let result = read_dl(&input[..], true).unwrap();
        assert_eq!(result.graph.vcount(), 3);
        assert_eq!(result.graph.ecount(), 4);
    }

    #[test]
    fn test_fullmatrix_labels_embedded() {
        let input = b"DL n=3\nlabels embedded:\ndata:\nA B C\nA 0 1 0\nB 0 0 1\nC 1 0 0\n";
        let result = read_dl(&input[..], true).unwrap();
        assert_eq!(result.graph.vcount(), 3);
        assert_eq!(result.graph.ecount(), 3);
        let labels = result.labels.unwrap();
        assert_eq!(labels, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_nodelist1_basic() {
        let input = b"DL n=4\nformat = nodelist1\ndata:\n1 2 3\n2 3\n3 4\n";
        let result = read_dl(&input[..], true).unwrap();
        assert_eq!(result.graph.vcount(), 4);
        assert_eq!(result.graph.ecount(), 4);
    }

    #[test]
    fn test_nodelist1_labels_embedded() {
        let input = b"DL n=3\nformat = nodelist1\nlabels embedded:\ndata:\nA B C\nB C\n";
        let result = read_dl(&input[..], true).unwrap();
        assert_eq!(result.graph.ecount(), 3);
        let labels = result.labels.unwrap();
        assert_eq!(labels[0], "A");
    }

    #[test]
    fn test_case_insensitive() {
        let input = b"dl N=2\nFORMAT = EDGELIST1\nDATA:\n1 2\n";
        let result = read_dl(&input[..], true).unwrap();
        assert_eq!(result.graph.vcount(), 2);
        assert_eq!(result.graph.ecount(), 1);
    }

    #[test]
    fn test_empty_graph() {
        let input = b"DL n=5\nformat = edgelist1\ndata:\n";
        let result = read_dl(&input[..], true).unwrap();
        assert_eq!(result.graph.vcount(), 5);
        assert_eq!(result.graph.ecount(), 0);
    }

    #[test]
    fn test_no_dl_header_error() {
        let input = b"n=3\ndata:\n1 2\n";
        let result = read_dl(&input[..], true);
        assert!(result.is_err());
    }

    #[test]
    fn test_no_n_error() {
        let input = b"DL\nformat = edgelist1\ndata:\n1 2\n";
        let result = read_dl(&input[..], true);
        assert!(result.is_err());
    }

    #[test]
    fn test_vertex_id_out_of_range() {
        let input = b"DL n=2\nformat = edgelist1\ndata:\n1 5\n";
        let result = read_dl(&input[..], true);
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_vertex_id_error() {
        let input = b"DL n=2\nformat = edgelist1\ndata:\n0 1\n";
        let result = read_dl(&input[..], true);
        assert!(result.is_err());
    }

    #[test]
    fn test_n_on_same_line() {
        let input = b"DL n=4\nformat=edgelist1\ndata:\n1 2\n3 4\n";
        let result = read_dl(&input[..], true).unwrap();
        assert_eq!(result.graph.vcount(), 4);
        assert_eq!(result.graph.ecount(), 2);
    }

    #[test]
    fn test_labels_whitespace_separated() {
        let input = b"DL n=3\nformat = edgelist1\nlabels:\nAlpha Beta Gamma\ndata:\n1 2\n";
        let result = read_dl(&input[..], true).unwrap();
        let labels = result.labels.unwrap();
        assert_eq!(labels, vec!["Alpha", "Beta", "Gamma"]);
    }

    // --- write_dl tests ---

    #[test]
    fn test_write_basic_directed() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let mut buf = Vec::new();
        write_dl(&g, None, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();

        assert!(s.contains("DL n=3"));
        assert!(s.contains("format = edgelist1"));
        assert!(s.contains("data:"));
        assert!(s.contains("1 2\n"));
        assert!(s.contains("2 3\n"));
    }

    #[test]
    fn test_write_with_labels() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();

        let labels = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let mut buf = Vec::new();
        write_dl(&g, Some(&labels), None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();

        assert!(s.contains("labels:"));
        assert!(s.contains("A,B,C"));
    }

    #[test]
    fn test_write_with_weights() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();

        let weights = vec![3.5];
        let mut buf = Vec::new();
        write_dl(&g, None, Some(&weights), &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();

        assert!(s.contains("1 2 3.5\n"));
    }

    #[test]
    fn test_write_empty_graph() {
        let g = Graph::with_vertices(0);

        let mut buf = Vec::new();
        write_dl(&g, None, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();

        assert!(s.contains("DL n=0"));
        assert!(s.contains("data:"));
    }

    #[test]
    fn test_write_no_edges() {
        let g = Graph::with_vertices(5);

        let mut buf = Vec::new();
        write_dl(&g, None, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();

        assert!(s.contains("DL n=5"));
        let after_data = s.split("data:\n").nth(1).unwrap();
        assert!(after_data.trim().is_empty());
    }

    #[test]
    fn test_write_label_mismatch_error() {
        let g = Graph::with_vertices(3);
        let labels = vec!["A".to_string()];
        let mut buf = Vec::new();
        assert!(write_dl(&g, Some(&labels), None, &mut buf).is_err());
    }

    #[test]
    fn test_write_weight_mismatch_error() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let weights = vec![1.0, 2.0];
        let mut buf = Vec::new();
        assert!(write_dl(&g, None, Some(&weights), &mut buf).is_err());
    }

    #[test]
    fn test_roundtrip_directed() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();

        let mut buf = Vec::new();
        write_dl(&g, None, None, &mut buf).unwrap();
        let result = read_dl(&buf[..], true).unwrap();

        assert_eq!(result.graph.vcount(), g.vcount());
        assert_eq!(result.graph.ecount(), g.ecount());
        assert!(result.graph.is_directed());
    }

    #[test]
    fn test_roundtrip_undirected() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let mut buf = Vec::new();
        write_dl(&g, None, None, &mut buf).unwrap();
        let result = read_dl(&buf[..], false).unwrap();

        assert_eq!(result.graph.vcount(), g.vcount());
        assert_eq!(result.graph.ecount(), g.ecount());
        assert!(!result.graph.is_directed());
    }

    #[test]
    fn test_roundtrip_with_labels() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let labels = vec!["X".to_string(), "Y".to_string(), "Z".to_string()];
        let mut buf = Vec::new();
        write_dl(&g, Some(&labels), None, &mut buf).unwrap();
        let result = read_dl(&buf[..], false).unwrap();

        assert_eq!(result.labels.unwrap(), labels);
    }

    #[test]
    fn test_roundtrip_with_weights() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();

        let weights = vec![2.75];
        let mut buf = Vec::new();
        write_dl(&g, None, Some(&weights), &mut buf).unwrap();
        let result = read_dl(&buf[..], false).unwrap();

        let w = result.weights.unwrap();
        assert!((w[0] - 2.75).abs() < 1e-10);
    }

    #[test]
    fn test_roundtrip_with_labels_and_weights() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let labels = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let weights = vec![1.5, 2.5];
        let mut buf = Vec::new();
        write_dl(&g, Some(&labels), Some(&weights), &mut buf).unwrap();
        let result = read_dl(&buf[..], true).unwrap();

        assert_eq!(result.graph.vcount(), 3);
        assert_eq!(result.graph.ecount(), 2);
        assert_eq!(result.labels.unwrap(), labels);
        let w = result.weights.unwrap();
        assert!((w[0] - 1.5).abs() < 1e-10);
        assert!((w[1] - 2.5).abs() < 1e-10);
    }

    #[test]
    fn test_write_self_loop() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();

        let mut buf = Vec::new();
        write_dl(&g, None, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();

        assert!(s.contains("1 1\n"));
    }

    #[test]
    fn test_write_one_based_ids() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(2, 3).unwrap();

        let mut buf = Vec::new();
        write_dl(&g, None, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();

        assert!(s.contains("3 4\n"));
    }
}

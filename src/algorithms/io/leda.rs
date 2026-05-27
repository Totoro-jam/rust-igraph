//! LEDA native graph format writer (ALGO-IO-007).
//!
//! Writes graphs in the LEDA native graph format. This is a write-only
//! format in igraph and in our implementation. The format is:
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
//! Counterpart of `igraph_write_graph_leda`.

use std::io::Write;

use crate::core::{Graph, IgraphError, IgraphResult};

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

    // Header
    writeln!(writer, "LEDA.GRAPH")?;

    // Vertex attribute type
    if vertex_labels.is_some() {
        writeln!(writer, "string")?;
    } else {
        writeln!(writer, "void")?;
    }

    // Edge attribute type
    if edge_weights.is_some() {
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
            None => writeln!(writer, "|{{}}|")?,
        }
    }

    // Edges section
    writeln!(writer, "# Edges")?;
    writeln!(writer, "{}", graph.ecount())?;

    for eid in 0..graph.ecount() {
        #[allow(clippy::cast_possible_truncation)]
        let (from, to) = graph.edge(eid as u32)?;

        match edge_weights {
            Some(w) => writeln!(writer, "{} {} 0 |{{{}}}|", from + 1, to + 1, w[eid])?,
            None => writeln!(writer, "{} {} 0 |{{}}|", from + 1, to + 1)?,
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
}

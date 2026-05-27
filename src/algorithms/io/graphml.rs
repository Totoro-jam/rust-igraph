//! `GraphML` writer (ALGO-IO-011).
//!
//! Writes graphs in the `GraphML` XML format. This is a write-only
//! implementation since reading `GraphML` requires a full XML parser.
//!
//! ```text
//! <?xml version="1.0" encoding="UTF-8"?>
//! <graphml xmlns="http://graphml.graphdrawing.org/xmlns">
//!   <graph id="G" edgedefault="undirected">
//!     <node id="n0"/>
//!     <node id="n1"/>
//!     <edge source="n0" target="n1"/>
//!   </graph>
//! </graphml>
//! ```
//!
//! Counterpart of `igraph_write_graph_graphml`.

use std::io::Write;

use crate::core::{Graph, IgraphError, IgraphResult};

/// Write a graph in `GraphML` format.
///
/// Outputs valid `GraphML` XML with node and edge elements. If `labels`
/// is provided, uses them as node IDs; otherwise uses `n0`, `n1`, etc.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, write_graphml};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
///
/// let mut buf = Vec::new();
/// write_graphml(&g, None, &mut buf).unwrap();
/// let s = String::from_utf8(buf).unwrap();
/// assert!(s.contains("<graphml"));
/// assert!(s.contains("edgedefault=\"undirected\""));
/// assert!(s.contains("<node id=\"n0\""));
/// ```
pub fn write_graphml<W: Write>(
    graph: &Graph,
    labels: Option<&[String]>,
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

    let edge_default = if graph.is_directed() {
        "directed"
    } else {
        "undirected"
    };

    writeln!(writer, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
    writeln!(
        writer,
        "<graphml xmlns=\"http://graphml.graphdrawing.org/xmlns\">"
    )?;
    writeln!(writer, "  <graph id=\"G\" edgedefault=\"{edge_default}\">")?;

    // Nodes
    for v in 0..graph.vcount() {
        let node_id = vertex_id(v, labels);
        writeln!(writer, "    <node id=\"{}\"/>", xml_escape(&node_id))?;
    }

    // Edges
    for eid in 0..graph.ecount() {
        #[allow(clippy::cast_possible_truncation)]
        let (from, to) = graph.edge(eid as u32)?;
        let src_id = vertex_id(from, labels);
        let tgt_id = vertex_id(to, labels);
        writeln!(
            writer,
            "    <edge source=\"{}\" target=\"{}\"/>",
            xml_escape(&src_id),
            xml_escape(&tgt_id)
        )?;
    }

    writeln!(writer, "  </graph>")?;
    writeln!(writer, "</graphml>")?;

    Ok(())
}

fn vertex_id(v: u32, labels: Option<&[String]>) -> String {
    match labels {
        Some(l) => l[v as usize].clone(),
        None => format!("n{v}"),
    }
}

fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
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
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let mut buf = Vec::new();
        write_graphml(&g, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();

        assert!(s.contains("<?xml version=\"1.0\""));
        assert!(s.contains("edgedefault=\"undirected\""));
        assert!(s.contains("<node id=\"n0\"/>"));
        assert!(s.contains("<node id=\"n1\"/>"));
        assert!(s.contains("<node id=\"n2\"/>"));
        assert!(s.contains("<edge source=\"n0\" target=\"n1\"/>"));
        assert!(s.contains("<edge source=\"n1\" target=\"n2\"/>"));
        assert!(s.contains("</graphml>"));
    }

    #[test]
    fn test_directed() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();

        let mut buf = Vec::new();
        write_graphml(&g, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();

        assert!(s.contains("edgedefault=\"directed\""));
    }

    #[test]
    fn test_with_labels() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();

        let labels = vec!["Alice".to_string(), "Bob".to_string()];
        let mut buf = Vec::new();
        write_graphml(&g, Some(&labels), &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();

        assert!(s.contains("<node id=\"Alice\"/>"));
        assert!(s.contains("<node id=\"Bob\"/>"));
        assert!(s.contains("<edge source=\"Alice\" target=\"Bob\"/>"));
    }

    #[test]
    fn test_xml_escaping() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();

        let labels = vec!["A&B".to_string(), "C<D".to_string()];
        let mut buf = Vec::new();
        write_graphml(&g, Some(&labels), &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();

        assert!(s.contains("<node id=\"A&amp;B\"/>"));
        assert!(s.contains("<node id=\"C&lt;D\"/>"));
    }

    #[test]
    fn test_empty_graph() {
        let g = Graph::with_vertices(0);

        let mut buf = Vec::new();
        write_graphml(&g, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();

        assert!(s.contains("<graph id=\"G\""));
        assert!(s.contains("</graph>"));
    }

    #[test]
    fn test_no_edges() {
        let g = Graph::with_vertices(3);

        let mut buf = Vec::new();
        write_graphml(&g, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();

        assert!(s.contains("<node id=\"n0\"/>"));
        assert!(s.contains("<node id=\"n1\"/>"));
        assert!(s.contains("<node id=\"n2\"/>"));
        assert!(!s.contains("<edge"));
    }

    #[test]
    fn test_self_loop() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();

        let mut buf = Vec::new();
        write_graphml(&g, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();

        assert!(s.contains("<edge source=\"n0\" target=\"n0\"/>"));
    }

    #[test]
    fn test_labels_mismatch_error() {
        let g = Graph::with_vertices(3);
        let labels = vec!["A".to_string()];
        let mut buf = Vec::new();
        assert!(write_graphml(&g, Some(&labels), &mut buf).is_err());
    }
}

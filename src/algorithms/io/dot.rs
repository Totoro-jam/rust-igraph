//! DOT (Graphviz) writer (ALGO-IO-006).
//!
//! Writes graphs in the DOT language used by the Graphviz suite of
//! tools. This is a write-only format (igraph does not read DOT files).
//!
//! ```text
//! graph {
//!   0 -- 1;
//!   1 -- 2;
//! }
//! ```
//!
//! For directed graphs:
//! ```text
//! digraph {
//!   0 -> 1;
//!   1 -> 2;
//! }
//! ```
//!
//! Counterpart of `igraph_write_graph_dot`.

use std::io::Write;

use crate::core::{Graph, IgraphError, IgraphResult};

/// Write a graph in DOT (Graphviz) format.
///
/// If `labels` is provided, uses them as vertex labels; otherwise uses
/// numeric ids. Isolated vertices are listed explicitly.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, write_dot};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
///
/// let mut buf = Vec::new();
/// write_dot(&g, None, &mut buf).unwrap();
/// let s = String::from_utf8(buf).unwrap();
/// assert!(s.contains("graph {"));
/// assert!(s.contains("--"));
/// ```
pub fn write_dot<W: Write>(
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

    let edge_op = if graph.is_directed() { "->" } else { "--" };
    let graph_type = if graph.is_directed() {
        "digraph"
    } else {
        "graph"
    };

    writeln!(writer, "{graph_type} {{")?;

    // Track which vertices appear in edges
    let mut has_edge = vec![false; graph.vcount() as usize];

    for eid in 0..graph.ecount() {
        #[allow(clippy::cast_possible_truncation)]
        let (src, tgt) = graph.edge(eid as u32)?;
        has_edge[src as usize] = true;
        has_edge[tgt as usize] = true;

        let src_label = vertex_label(src, labels);
        let tgt_label = vertex_label(tgt, labels);
        writeln!(writer, "  {src_label} {edge_op} {tgt_label};")?;
    }

    // Emit isolated vertices
    for v in 0..graph.vcount() {
        if !has_edge[v as usize] {
            let lbl = vertex_label(v, labels);
            writeln!(writer, "  {lbl};")?;
        }
    }

    writeln!(writer, "}}")?;

    Ok(())
}

fn vertex_label(v: u32, labels: Option<&[String]>) -> String {
    match labels {
        Some(l) => dot_escape(&l[v as usize]),
        None => v.to_string(),
    }
}

fn dot_escape(s: &str) -> String {
    // Check if the string is a simple identifier (alphanumeric + underscore, not starting with digit)
    let is_simple = !s.is_empty()
        && !s.as_bytes()[0].is_ascii_digit()
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');

    // Also check if it's a DOT reserved word
    let reserved = matches!(
        s.to_ascii_lowercase().as_str(),
        "graph" | "digraph" | "node" | "edge" | "strict" | "subgraph"
    );

    if is_simple && !reserved {
        s.to_owned()
    } else {
        // Quote the string
        let mut out = String::with_capacity(s.len() + 2);
        out.push('"');
        for c in s.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                _ => out.push(c),
            }
        }
        out.push('"');
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undirected_basic() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let mut buf = Vec::new();
        write_dot(&g, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.starts_with("graph {\n"));
        assert!(s.contains("0 -- 1;"));
        assert!(s.contains("1 -- 2;"));
        assert!(s.ends_with("}\n"));
    }

    #[test]
    fn test_directed_basic() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let mut buf = Vec::new();
        write_dot(&g, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.starts_with("digraph {\n"));
        assert!(s.contains("0 -> 1;"));
        assert!(s.contains("1 -> 2;"));
    }

    #[test]
    fn test_with_labels() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();

        let labels = vec!["Alice".to_string(), "Bob".to_string(), "Carol".to_string()];
        let mut buf = Vec::new();
        write_dot(&g, Some(&labels), &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("Alice -- Bob;"));
        // Carol is isolated
        assert!(s.contains("Carol;"));
    }

    #[test]
    fn test_isolated_vertices() {
        let g = Graph::with_vertices(3);

        let mut buf = Vec::new();
        write_dot(&g, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("  0;\n"));
        assert!(s.contains("  1;\n"));
        assert!(s.contains("  2;\n"));
    }

    #[test]
    fn test_empty_graph() {
        let g = Graph::with_vertices(0);

        let mut buf = Vec::new();
        write_dot(&g, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert_eq!(s, "graph {\n}\n");
    }

    #[test]
    fn test_reserved_word_label_escaped() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();

        let labels = vec!["graph".to_string(), "node".to_string()];
        let mut buf = Vec::new();
        write_dot(&g, Some(&labels), &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("\"graph\" -- \"node\";"));
    }

    #[test]
    fn test_label_with_spaces_escaped() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();

        let labels = vec!["hello world".to_string(), "foo bar".to_string()];
        let mut buf = Vec::new();
        write_dot(&g, Some(&labels), &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("\"hello world\" -- \"foo bar\";"));
    }

    #[test]
    fn test_label_with_quotes_escaped() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();

        let labels = vec!["say \"hi\"".to_string(), "ok".to_string()];
        let mut buf = Vec::new();
        write_dot(&g, Some(&labels), &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("\"say \\\"hi\\\"\" -- ok;"));
    }

    #[test]
    fn test_label_starting_with_digit() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();

        let labels = vec!["123abc".to_string(), "valid_name".to_string()];
        let mut buf = Vec::new();
        write_dot(&g, Some(&labels), &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("\"123abc\" -- valid_name;"));
    }

    #[test]
    fn test_self_loop() {
        let mut g = Graph::with_vertices(1);
        g.add_edge(0, 0).unwrap();

        let mut buf = Vec::new();
        write_dot(&g, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("0 -- 0;"));
    }

    #[test]
    fn test_labels_mismatch_error() {
        let g = Graph::with_vertices(3);
        let labels = vec!["A".to_string()];
        let mut buf = Vec::new();
        assert!(write_dot(&g, Some(&labels), &mut buf).is_err());
    }
}

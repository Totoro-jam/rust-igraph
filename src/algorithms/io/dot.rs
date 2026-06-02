//! DOT (Graphviz) reader / writer (ALGO-IO-006, ALGO-IO-013).
//!
//! Reads and writes graphs in the DOT language used by the Graphviz suite
//! of tools. The reader handles the structural subset: `graph`/`digraph`
//! keyword, node declarations, and edge statements (`--` / `->`). Graph,
//! node, and edge attributes (inside `[...]`) are silently skipped.
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
//! Counterparts of `igraph_read_graph_dot` (new) and `igraph_write_graph_dot`.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};

use crate::core::{Graph, IgraphError, IgraphResult};

/// Result of parsing a DOT file.
pub struct DotGraph {
    /// The parsed graph.
    pub graph: Graph,
    /// Node labels in vertex-index order. Numeric-only labels are preserved
    /// as strings (e.g. `"0"`, `"1"`).
    pub labels: Vec<String>,
}

/// Read a graph from DOT (Graphviz) format.
///
/// Parses `graph { ... }` (undirected) or `digraph { ... }` (directed)
/// blocks. Recognizes edge statements using `--` or `->` and node
/// declarations. Attribute blocks (`[...]`) are skipped. Subgraphs are
/// treated as flat (nodes are merged into the top-level graph).
///
/// Node identifiers may be bare identifiers, numbers, or double-quoted
/// strings. They are assigned internal vertex indices in first-appearance
/// order and returned in [`DotGraph::labels`].
///
/// # Errors
///
/// Returns an error if the input lacks a `graph`/`digraph` keyword or
/// contains `--` edges in a `digraph` (or `->` in a `graph`).
///
/// # Examples
///
/// ```
/// use rust_igraph::read_dot;
///
/// let dot = "graph {\n  Alice -- Bob;\n  Bob -- Carol;\n}";
/// let result = read_dot(dot.as_bytes()).unwrap();
/// assert_eq!(result.graph.vcount(), 3);
/// assert_eq!(result.graph.ecount(), 2);
/// assert!(!result.graph.is_directed());
/// assert_eq!(result.labels, vec!["Alice", "Bob", "Carol"]);
/// ```
pub fn read_dot<R: Read>(input: R) -> IgraphResult<DotGraph> {
    let reader = BufReader::new(input);
    let mut lines_buf: Vec<String> = Vec::new();
    for line in reader.lines() {
        lines_buf.push(line?);
    }
    let content = lines_buf.join("\n");

    // Strip C/C++ style comments.
    let content = strip_comments(&content);

    // Determine directed/undirected from the keyword.
    let directed = detect_directed(&content)?;

    // Find the body between the first '{' and its matching '}'.
    let body_start = content.find('{').ok_or_else(|| IgraphError::Parse {
        line: 0,
        message: "no opening '{' found in DOT input".to_owned(),
    })? + 1;
    let body_end = content.rfind('}').ok_or_else(|| IgraphError::Parse {
        line: 0,
        message: "no closing '}' found in DOT input".to_owned(),
    })?;
    let body = &content[body_start..body_end];

    // Tokenize the body into statements (split on ';' and newlines).
    let mut node_ids: Vec<String> = Vec::new();
    let mut node_map: HashMap<String, u32> = HashMap::new();
    let mut edges: Vec<(u32, u32)> = Vec::new();

    let edge_op = if directed { "->" } else { "--" };

    let ensure_node =
        |name: &str, ids: &mut Vec<String>, map: &mut HashMap<String, u32>| -> IgraphResult<u32> {
            if let Some(&idx) = map.get(name) {
                return Ok(idx);
            }
            let idx = u32::try_from(ids.len())
                .map_err(|_| IgraphError::InvalidArgument("too many nodes for u32".to_owned()))?;
            map.insert(name.to_owned(), idx);
            ids.push(name.to_owned());
            Ok(idx)
        };

    for raw_stmt in body.split(';') {
        let stmt = raw_stmt.trim();
        if stmt.is_empty() {
            continue;
        }

        // Skip attribute-only statements and subgraph keywords.
        let lower = stmt.to_ascii_lowercase();
        if lower.starts_with("graph ")
            || lower.starts_with("node ")
            || lower.starts_with("edge ")
            || lower.starts_with("subgraph ")
            || lower.starts_with('{')
            || lower.starts_with('}')
        {
            // Check if it's actually a "graph [...]" attribute statement.
            if !stmt.contains(edge_op) {
                continue;
            }
        }

        // Try to parse as an edge statement.
        if stmt.contains(edge_op) {
            let parts = split_edge_statement(stmt, edge_op);
            if parts.len() >= 2 {
                for pair in parts.windows(2) {
                    let src_name = clean_node_id(pair[0]);
                    let tgt_name = clean_node_id(pair[1]);
                    if src_name.is_empty() || tgt_name.is_empty() {
                        continue;
                    }
                    let src = ensure_node(&src_name, &mut node_ids, &mut node_map)?;
                    let tgt = ensure_node(&tgt_name, &mut node_ids, &mut node_map)?;
                    edges.push((src, tgt));
                }
                continue;
            }
        }

        // Otherwise it's a node declaration.
        let node_name = clean_node_id(stmt);
        if !node_name.is_empty() {
            ensure_node(&node_name, &mut node_ids, &mut node_map)?;
        }
    }

    // Build graph.
    let n = u32::try_from(node_ids.len())
        .map_err(|_| IgraphError::InvalidArgument("too many nodes for u32".to_owned()))?;
    let mut graph = Graph::new(n, directed)?;
    for (src, tgt) in edges {
        graph.add_edge(src, tgt)?;
    }

    Ok(DotGraph {
        graph,
        labels: node_ids,
    })
}

/// Strip C-style (`/* ... */`) and C++-style (`// ...`) comments.
fn strip_comments(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            // Block comment — skip until */
            i += 2;
            while i + 1 < len && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i += 2; // skip */
        } else if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            // Line comment — skip until newline.
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
        } else if bytes[i] == b'"' {
            // Quoted string — preserve verbatim.
            out.push('"');
            i += 1;
            while i < len && bytes[i] != b'"' {
                if bytes[i] == b'\\' && i + 1 < len {
                    out.push(bytes[i] as char);
                    i += 1;
                }
                out.push(bytes[i] as char);
                i += 1;
            }
            if i < len {
                out.push('"');
                i += 1;
            }
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

/// Detect whether the DOT file is `digraph` (directed) or `graph` (undirected).
fn detect_directed(content: &str) -> IgraphResult<bool> {
    // Look for the first occurrence of "digraph" or "graph" as a keyword.
    let lower = content.to_ascii_lowercase();
    let di_pos = lower.find("digraph");
    let g_pos = lower.find("graph");
    match (di_pos, g_pos) {
        (Some(dp), Some(gp)) => {
            if dp <= gp {
                Ok(true)
            } else {
                // "graph" appears first, but check it's not inside "digraph"
                Ok(false)
            }
        }
        (Some(_), None) => Ok(true),
        (None, Some(_)) => Ok(false),
        (None, None) => Err(IgraphError::Parse {
            line: 0,
            message: "no 'graph' or 'digraph' keyword found".to_owned(),
        }),
    }
}

/// Split an edge statement by the edge operator, handling chains like
/// `a -> b -> c`.
fn split_edge_statement<'a>(stmt: &'a str, op: &str) -> Vec<&'a str> {
    stmt.split(op).map(str::trim).collect()
}

/// Clean up a node identifier: strip attribute brackets, trim whitespace,
/// and remove surrounding double quotes.
fn clean_node_id(raw: &str) -> String {
    let mut s = raw.trim();
    // Strip trailing attribute block [...]
    if let Some(bracket) = s.find('[') {
        s = s[..bracket].trim();
    }
    // Strip trailing braces (subgraph artifacts)
    if let Some(brace) = s.find('{') {
        s = s[..brace].trim();
    }
    if let Some(brace) = s.find('}') {
        s = s[..brace].trim();
    }
    // Remove surrounding quotes.
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        let inner = &s[1..s.len() - 1];
        return inner.replace("\\\"", "\"").replace("\\\\", "\\");
    }
    s.to_owned()
}

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

    // --- read_dot tests ---

    #[test]
    fn read_undirected_basic() {
        let dot = "graph {\n  0 -- 1;\n  1 -- 2;\n}";
        let result = read_dot(dot.as_bytes()).unwrap();
        assert_eq!(result.graph.vcount(), 3);
        assert_eq!(result.graph.ecount(), 2);
        assert!(!result.graph.is_directed());
    }

    #[test]
    fn read_directed_basic() {
        let dot = "digraph {\n  a -> b;\n  b -> c;\n}";
        let result = read_dot(dot.as_bytes()).unwrap();
        assert_eq!(result.graph.vcount(), 3);
        assert_eq!(result.graph.ecount(), 2);
        assert!(result.graph.is_directed());
        assert_eq!(result.labels, vec!["a", "b", "c"]);
    }

    #[test]
    fn read_with_labels() {
        let dot = "graph {\n  Alice -- Bob;\n  Bob -- Carol;\n}";
        let result = read_dot(dot.as_bytes()).unwrap();
        assert_eq!(result.labels, vec!["Alice", "Bob", "Carol"]);
        assert_eq!(result.graph.ecount(), 2);
    }

    #[test]
    fn read_quoted_labels() {
        let dot = r#"graph { "hello world" -- "foo bar"; }"#;
        let result = read_dot(dot.as_bytes()).unwrap();
        assert_eq!(result.labels, vec!["hello world", "foo bar"]);
        assert_eq!(result.graph.ecount(), 1);
    }

    #[test]
    fn read_with_attributes_ignored() {
        let dot = r#"graph {
  node [shape=circle];
  edge [color=red];
  a [label="Node A"];
  b [label="Node B"];
  a -- b [weight=1.5];
}"#;
        let result = read_dot(dot.as_bytes()).unwrap();
        assert_eq!(result.graph.vcount(), 2);
        assert_eq!(result.graph.ecount(), 1);
    }

    #[test]
    fn read_edge_chain() {
        let dot = "digraph { a -> b -> c -> d; }";
        let result = read_dot(dot.as_bytes()).unwrap();
        assert_eq!(result.graph.vcount(), 4);
        assert_eq!(result.graph.ecount(), 3);
    }

    #[test]
    fn read_isolated_nodes() {
        let dot = "graph {\n  x;\n  y;\n  z;\n  x -- y;\n}";
        let result = read_dot(dot.as_bytes()).unwrap();
        assert_eq!(result.graph.vcount(), 3);
        assert_eq!(result.graph.ecount(), 1);
    }

    #[test]
    fn read_with_comments() {
        let dot = "// comment\ngraph {\n  /* block comment */\n  a -- b;\n}";
        let result = read_dot(dot.as_bytes()).unwrap();
        assert_eq!(result.graph.vcount(), 2);
        assert_eq!(result.graph.ecount(), 1);
    }

    #[test]
    fn read_empty_graph() {
        let dot = "graph { }";
        let result = read_dot(dot.as_bytes()).unwrap();
        assert_eq!(result.graph.vcount(), 0);
        assert_eq!(result.graph.ecount(), 0);
    }

    #[test]
    fn read_no_keyword_is_error() {
        let dot = "{ a -- b; }";
        assert!(read_dot(dot.as_bytes()).is_err());
    }

    #[test]
    fn read_self_loop() {
        let dot = "graph { a -- a; }";
        let result = read_dot(dot.as_bytes()).unwrap();
        assert_eq!(result.graph.vcount(), 1);
        assert_eq!(result.graph.ecount(), 1);
    }

    #[test]
    fn roundtrip_dot() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();

        let mut buf = Vec::new();
        write_dot(&g, None, &mut buf).unwrap();

        let result = read_dot(buf.as_slice()).unwrap();
        assert_eq!(result.graph.vcount(), 3);
        assert_eq!(result.graph.ecount(), 3);
        assert!(result.graph.is_directed());
    }

    #[test]
    fn read_strict_keyword() {
        let dot = "strict graph {\n  a -- b;\n  a -- b;\n}";
        let result = read_dot(dot.as_bytes()).unwrap();
        assert!(!result.graph.is_directed());
        assert_eq!(result.graph.vcount(), 2);
    }
}

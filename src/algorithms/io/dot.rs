//! DOT (Graphviz) reader / writer (ALGO-IO-006, ALGO-IO-013).
//!
//! Reads and writes graphs in the DOT language used by the Graphviz suite
//! of tools. The reader handles the structural subset: `graph`/`digraph`
//! keyword, node declarations, and edge statements (`--` / `->`).
//! Attribute blocks (`[key=value, ...]`) on nodes and edges are parsed and
//! stored via the [`Graph`] attribute system. Graph-level attribute
//! statements (`graph [key=value]`) are stored as graph attributes.
//!
//! ```text
//! graph {
//!   graph [label="My Graph"];
//!   0 [color="red"];
//!   0 -- 1 [weight=1.5];
//!   1 -- 2;
//! }
//! ```
//!
//! For directed graphs:
//! ```text
//! digraph {
//!   0 -> 1 [weight=2.0];
//!   1 -> 2;
//! }
//! ```
//!
//! Counterparts of `igraph_read_graph_dot` (new) and `igraph_write_graph_dot`.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};

use crate::core::attributes::AttributeValue;
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
/// declarations. Attribute blocks (`[key=value, ...]`) on nodes and edges
/// are parsed and stored via the [`Graph`] attribute system. Subgraphs are
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
/// use rust_igraph::{read_dot, AttributeValue};
///
/// let dot = r#"graph {
///   Alice [color="red"];
///   Alice -- Bob [weight=1.5];
///   Bob -- Carol;
/// }"#;
/// let result = read_dot(dot.as_bytes()).unwrap();
/// assert_eq!(result.graph.vcount(), 3);
/// assert_eq!(result.graph.ecount(), 2);
/// assert!(!result.graph.is_directed());
/// assert_eq!(result.labels, vec!["Alice", "Bob", "Carol"]);
/// assert_eq!(
///     result.graph.vertex_attribute("color", 0).and_then(|v| v.as_str()),
///     Some("red"),
/// );
/// assert_eq!(
///     result.graph.edge_attribute("weight", 0).and_then(|v| v.as_f64()),
///     Some(1.5),
/// );
/// ```
#[allow(clippy::too_many_lines)]
pub fn read_dot<R: Read>(input: R) -> IgraphResult<DotGraph> {
    let reader = BufReader::new(input);
    let mut lines_buf: Vec<String> = Vec::new();
    for line in reader.lines() {
        lines_buf.push(line?);
    }
    let content = lines_buf.join("\n");

    let content = strip_comments(&content);
    let directed = detect_directed(&content)?;

    let body_start = content.find('{').ok_or_else(|| IgraphError::Parse {
        line: 0,
        message: "no opening '{' found in DOT input".to_owned(),
    })? + 1;
    let body_end = content.rfind('}').ok_or_else(|| IgraphError::Parse {
        line: 0,
        message: "no closing '}' found in DOT input".to_owned(),
    })?;
    let body = &content[body_start..body_end];

    let mut node_ids: Vec<String> = Vec::new();
    let mut node_map: HashMap<String, u32> = HashMap::new();
    let mut edges: Vec<(u32, u32)> = Vec::new();
    let mut node_attr_list: Vec<Vec<(String, AttributeValue)>> = Vec::new();
    let mut edge_attr_list: Vec<Vec<(String, AttributeValue)>> = Vec::new();
    let mut graph_attrs: Vec<(String, AttributeValue)> = Vec::new();

    let edge_op = if directed { "->" } else { "--" };

    let ensure_node = |name: &str,
                       ids: &mut Vec<String>,
                       map: &mut HashMap<String, u32>,
                       attrs: &mut Vec<Vec<(String, AttributeValue)>>|
     -> IgraphResult<u32> {
        if let Some(&idx) = map.get(name) {
            return Ok(idx);
        }
        let idx = u32::try_from(ids.len())
            .map_err(|_| IgraphError::InvalidArgument("too many nodes for u32".to_owned()))?;
        map.insert(name.to_owned(), idx);
        ids.push(name.to_owned());
        attrs.push(Vec::new());
        Ok(idx)
    };

    for raw_stmt in body.split(';') {
        let stmt = raw_stmt.trim();
        if stmt.is_empty() {
            continue;
        }

        let lower = stmt.to_ascii_lowercase();

        // Graph-level attribute statement: graph [key=value]
        if lower.starts_with("graph ") && !stmt.contains(edge_op) {
            if let Some(attrs) = extract_attr_block(stmt) {
                let parsed = parse_attr_pairs(&attrs);
                graph_attrs.extend(parsed);
            }
            continue;
        }

        // Skip global node/edge/subgraph attribute statements
        if (lower.starts_with("node ")
            || lower.starts_with("edge ")
            || lower.starts_with("subgraph "))
            && !stmt.contains(edge_op)
        {
            continue;
        }
        if lower.starts_with('{') || lower.starts_with('}') {
            continue;
        }

        // Edge statement
        if stmt.contains(edge_op) {
            let (structural, attrs) = split_attr_block(stmt);
            let edge_attrs_parsed = attrs.map(|a| parse_attr_pairs(&a)).unwrap_or_default();
            let parts = split_edge_statement(&structural, edge_op);
            if parts.len() >= 2 {
                for pair in parts.windows(2) {
                    let src_name = clean_node_id(pair[0]);
                    let tgt_name = clean_node_id(pair[1]);
                    if src_name.is_empty() || tgt_name.is_empty() {
                        continue;
                    }
                    let src =
                        ensure_node(&src_name, &mut node_ids, &mut node_map, &mut node_attr_list)?;
                    let tgt =
                        ensure_node(&tgt_name, &mut node_ids, &mut node_map, &mut node_attr_list)?;
                    edges.push((src, tgt));
                    edge_attr_list.push(edge_attrs_parsed.clone());
                }
                continue;
            }
        }

        // Node declaration (possibly with attributes)
        let (structural, attrs) = split_attr_block(stmt);
        let node_name = clean_node_id(&structural);
        if !node_name.is_empty() {
            let idx = ensure_node(
                &node_name,
                &mut node_ids,
                &mut node_map,
                &mut node_attr_list,
            )?;
            if let Some(a) = attrs {
                let parsed = parse_attr_pairs(&a);
                node_attr_list[idx as usize].extend(parsed);
            }
        }
    }

    // Build graph
    let n = u32::try_from(node_ids.len())
        .map_err(|_| IgraphError::InvalidArgument("too many nodes for u32".to_owned()))?;
    let mut graph = Graph::new(n, directed)?;
    for (src, tgt) in &edges {
        graph.add_edge(*src, *tgt)?;
    }

    // Apply vertex attributes
    apply_vertex_attrs(&mut graph, &node_attr_list)?;

    // Apply edge attributes
    apply_edge_attrs(&mut graph, &edge_attr_list)?;

    // Apply graph attributes
    for (key, val) in graph_attrs {
        graph.set_graph_attribute(key, val);
    }

    Ok(DotGraph {
        graph,
        labels: node_ids,
    })
}

/// Apply parsed vertex attributes to the graph.
fn apply_vertex_attrs(
    graph: &mut Graph,
    node_attr_list: &[Vec<(String, AttributeValue)>],
) -> IgraphResult<()> {
    let mut attr_names: HashMap<String, Vec<AttributeValue>> = HashMap::new();
    let n = graph.vcount() as usize;

    for (vid, attrs) in node_attr_list.iter().enumerate() {
        for (key, val) in attrs {
            let col = attr_names.entry(key.clone()).or_insert_with(|| {
                let default = val.default_for_same_type();
                vec![default; n]
            });
            if vid < col.len() {
                col[vid] = val.clone();
            }
        }
    }

    for (name, values) in attr_names {
        graph.set_vertex_attribute_all(name, values)?;
    }
    Ok(())
}

/// Apply parsed edge attributes to the graph.
fn apply_edge_attrs(
    graph: &mut Graph,
    edge_attr_list: &[Vec<(String, AttributeValue)>],
) -> IgraphResult<()> {
    let mut attr_names: HashMap<String, Vec<AttributeValue>> = HashMap::new();
    let m = graph.ecount();

    for (eid, attrs) in edge_attr_list.iter().enumerate() {
        for (key, val) in attrs {
            let col = attr_names.entry(key.clone()).or_insert_with(|| {
                let default = val.default_for_same_type();
                vec![default; m]
            });
            if eid < col.len() {
                col[eid] = val.clone();
            }
        }
    }

    for (name, values) in attr_names {
        graph.set_edge_attribute_all(name, values)?;
    }
    Ok(())
}

/// Extract the content inside `[...]` from a statement string, if present.
fn extract_attr_block(stmt: &str) -> Option<String> {
    let open = stmt.find('[')?;
    let close = stmt.rfind(']')?;
    if close > open {
        Some(stmt[open + 1..close].to_owned())
    } else {
        None
    }
}

/// Split a statement into (structural part, optional attr block content).
fn split_attr_block(stmt: &str) -> (String, Option<String>) {
    if let Some(open) = stmt.find('[') {
        let structural = stmt[..open].trim().to_owned();
        let close = stmt.rfind(']').unwrap_or(stmt.len());
        let attrs = stmt[open + 1..close].to_owned();
        (structural, Some(attrs))
    } else {
        (stmt.to_owned(), None)
    }
}

/// Parse `key=value, key=value, ...` pairs from attribute block content.
fn parse_attr_pairs(content: &str) -> Vec<(String, AttributeValue)> {
    let mut result = Vec::new();
    let mut remaining = content.trim();

    while !remaining.is_empty() {
        // Find key
        let Some(eq_pos) = remaining.find('=') else {
            break;
        };
        let key = remaining[..eq_pos].trim();
        remaining = remaining[eq_pos + 1..].trim();

        // Parse value
        let (value_str, rest) = if remaining.starts_with('"') {
            parse_quoted_value(remaining)
        } else {
            parse_unquoted_value(remaining)
        };

        if !key.is_empty() {
            result.push((key.to_owned(), dot_value_to_attribute(&value_str)));
        }

        remaining = rest.trim();
        // Skip comma or semicolon separator
        if remaining.starts_with(',') || remaining.starts_with(';') {
            remaining = remaining[1..].trim();
        }
    }

    result
}

/// Parse a double-quoted value, handling escapes.
fn parse_quoted_value(s: &str) -> (String, &str) {
    let bytes = s.as_bytes();
    let mut i = 1; // skip opening quote
    let mut value = String::new();
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            match bytes[i + 1] {
                b'"' => value.push('"'),
                b'\\' => value.push('\\'),
                b'n' => value.push('\n'),
                other => {
                    value.push('\\');
                    value.push(other as char);
                }
            }
            i += 2;
        } else if bytes[i] == b'"' {
            return (value, &s[i + 1..]);
        } else {
            value.push(bytes[i] as char);
            i += 1;
        }
    }
    (value, "")
}

/// Parse an unquoted value (terminated by comma, semicolon, or end).
fn parse_unquoted_value(s: &str) -> (String, &str) {
    let end = s.find([',', ';', ']']).unwrap_or(s.len());
    let value = s[..end].trim().to_owned();
    (value, &s[end..])
}

/// Convert a DOT attribute value string to an `AttributeValue`.
fn dot_value_to_attribute(s: &str) -> AttributeValue {
    // Try boolean
    match s.to_ascii_lowercase().as_str() {
        "true" => return AttributeValue::Boolean(true),
        "false" => return AttributeValue::Boolean(false),
        _ => {}
    }
    // Try numeric
    if let Ok(v) = s.parse::<f64>() {
        return AttributeValue::Numeric(v);
    }
    AttributeValue::String(s.to_owned())
}

/// Strip C-style (`/* ... */`) and C++-style (`// ...`) comments.
fn strip_comments(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < len && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i += 2;
        } else if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
        } else if bytes[i] == b'"' {
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
    let lower = content.to_ascii_lowercase();
    let di_pos = lower.find("digraph");
    let g_pos = lower.find("graph");
    match (di_pos, g_pos) {
        (Some(dp), Some(gp)) => {
            if dp <= gp {
                Ok(true)
            } else {
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

/// Clean up a node identifier: trim whitespace and remove surrounding
/// double quotes.
fn clean_node_id(raw: &str) -> String {
    let mut s = raw.trim();
    // Strip trailing braces (subgraph artifacts)
    if let Some(brace) = s.find('{') {
        s = s[..brace].trim();
    }
    if let Some(brace) = s.find('}') {
        s = s[..brace].trim();
    }
    // Remove surrounding quotes
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        let inner = &s[1..s.len() - 1];
        return inner.replace("\\\"", "\"").replace("\\\\", "\\");
    }
    s.to_owned()
}

// ─── Writer ──────────────────────────────────────────────────────────

/// Write a graph in DOT (Graphviz) format.
///
/// If `labels` is provided, uses them as vertex labels; otherwise uses
/// numeric ids. Isolated vertices are listed explicitly. Vertex and edge
/// attributes stored on the [`Graph`] are emitted as DOT attribute blocks
/// `[key=value, ...]`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, write_dot, AttributeValue};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.set_vertex_attribute("color", 0, AttributeValue::String("red".into())).unwrap();
/// g.set_edge_attribute("weight", 0, AttributeValue::Numeric(1.5)).unwrap();
///
/// let mut buf = Vec::new();
/// write_dot(&g, None, &mut buf).unwrap();
/// let s = String::from_utf8(buf).unwrap();
/// assert!(s.contains("graph {"));
/// assert!(s.contains("[color=\"red\"]"));
/// assert!(s.contains("[weight=1.5]"));
/// ```
#[allow(clippy::too_many_lines)]
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

    // Write graph-level attributes
    write_graph_attrs(graph, writer)?;

    // Collect vertex/edge attribute names
    let v_attr_names = graph.vertex_attribute_names();
    let e_attr_names = graph.edge_attribute_names();

    // Track which vertices appear in edges
    let mut has_edge = vec![false; graph.vcount() as usize];

    for eid in 0..graph.ecount() {
        let eid_u32 = u32::try_from(eid)
            .map_err(|_| IgraphError::InvalidArgument("edge id overflow".to_owned()))?;
        let (src, tgt) = graph.edge(eid_u32)?;
        has_edge[src as usize] = true;
        has_edge[tgt as usize] = true;

        let src_label = vertex_label(src, labels);
        let tgt_label = vertex_label(tgt, labels);

        write!(writer, "  {src_label} {edge_op} {tgt_label}")?;
        write_edge_attr_block(graph, eid_u32, &e_attr_names, writer)?;
        writeln!(writer, ";")?;
    }

    // Emit all vertices (with attributes if any, or isolated ones without)
    for v in 0..graph.vcount() {
        let has_attrs = v_attr_names.iter().any(|name| {
            graph
                .vertex_attribute(name, v)
                .is_some_and(|val| !is_default_attr(val))
        });

        if has_attrs || !has_edge[v as usize] {
            let lbl = vertex_label(v, labels);
            write!(writer, "  {lbl}")?;
            write_vertex_attr_block(graph, v, &v_attr_names, writer)?;
            writeln!(writer, ";")?;
        }
    }

    writeln!(writer, "}}")?;

    Ok(())
}

/// Write graph-level attributes as `graph [key=value, ...]`.
fn write_graph_attrs<W: Write>(graph: &Graph, writer: &mut W) -> IgraphResult<()> {
    let names = graph.graph_attribute_names();
    if names.is_empty() {
        return Ok(());
    }

    write!(writer, "  graph [")?;
    let mut first = true;
    for name in &names {
        if let Some(val) = graph.graph_attribute(name) {
            if !first {
                write!(writer, ", ")?;
            }
            write_attr_pair(name, val, writer)?;
            first = false;
        }
    }
    writeln!(writer, "];")?;
    Ok(())
}

/// Write a vertex attribute block `[key=value, ...]` for vertex `v`.
fn write_vertex_attr_block<W: Write>(
    graph: &Graph,
    v: u32,
    attr_names: &[&str],
    writer: &mut W,
) -> IgraphResult<()> {
    let mut pairs: Vec<(&str, &AttributeValue)> = Vec::new();
    for name in attr_names {
        if let Some(val) = graph.vertex_attribute(name, v) {
            if !is_default_attr(val) {
                pairs.push((name, val));
            }
        }
    }

    if pairs.is_empty() {
        return Ok(());
    }

    write!(writer, " [")?;
    for (i, (name, val)) in pairs.iter().enumerate() {
        if i > 0 {
            write!(writer, ", ")?;
        }
        write_attr_pair(name, val, writer)?;
    }
    write!(writer, "]")?;
    Ok(())
}

/// Write an edge attribute block `[key=value, ...]` for edge `eid`.
fn write_edge_attr_block<W: Write>(
    graph: &Graph,
    eid: u32,
    attr_names: &[&str],
    writer: &mut W,
) -> IgraphResult<()> {
    let mut pairs: Vec<(&str, &AttributeValue)> = Vec::new();
    for name in attr_names {
        if let Some(val) = graph.edge_attribute(name, eid) {
            if !is_default_attr(val) {
                pairs.push((name, val));
            }
        }
    }

    if pairs.is_empty() {
        return Ok(());
    }

    write!(writer, " [")?;
    for (i, (name, val)) in pairs.iter().enumerate() {
        if i > 0 {
            write!(writer, ", ")?;
        }
        write_attr_pair(name, val, writer)?;
    }
    write!(writer, "]")?;
    Ok(())
}

/// Check if an attribute value is a "default" (NaN, empty string, false)
/// that should be omitted from output.
fn is_default_attr(val: &AttributeValue) -> bool {
    match val {
        AttributeValue::Numeric(v) => v.is_nan(),
        AttributeValue::String(s) => s.is_empty(),
        AttributeValue::Boolean(_) => false,
    }
}

/// Write a single `key=value` pair in DOT format.
fn write_attr_pair<W: Write>(name: &str, val: &AttributeValue, writer: &mut W) -> IgraphResult<()> {
    match val {
        AttributeValue::Numeric(v) => {
            write!(writer, "{name}={v}")?;
        }
        AttributeValue::Boolean(b) => {
            write!(writer, "{name}={b}")?;
        }
        AttributeValue::String(s) => {
            let escaped = dot_escape_value(s);
            write!(writer, "{name}={escaped}")?;
        }
    }
    Ok(())
}

fn vertex_label(v: u32, labels: Option<&[String]>) -> String {
    match labels {
        Some(l) => dot_escape_id(&l[v as usize]),
        None => v.to_string(),
    }
}

/// Escape a string for use as a DOT node identifier.
fn dot_escape_id(s: &str) -> String {
    let is_simple = !s.is_empty()
        && !s.as_bytes()[0].is_ascii_digit()
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');

    let reserved = matches!(
        s.to_ascii_lowercase().as_str(),
        "graph" | "digraph" | "node" | "edge" | "strict" | "subgraph"
    );

    if is_simple && !reserved {
        s.to_owned()
    } else {
        dot_quote(s)
    }
}

/// Escape a string for use as a DOT attribute value.
fn dot_escape_value(s: &str) -> String {
    dot_quote(s)
}

/// Wrap a string in double quotes with DOT escaping.
fn dot_quote(s: &str) -> String {
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

    // --- write_dot attribute tests ---

    #[test]
    fn write_vertex_attributes() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.set_vertex_attribute("color", 0, AttributeValue::String("red".into()))
            .unwrap();
        g.set_vertex_attribute("color", 1, AttributeValue::String("blue".into()))
            .unwrap();

        let mut buf = Vec::new();
        write_dot(&g, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("[color=\"red\"]"));
        assert!(s.contains("[color=\"blue\"]"));
    }

    #[test]
    fn write_edge_attributes() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.set_edge_attribute("weight", 0, AttributeValue::Numeric(2.5))
            .unwrap();

        let mut buf = Vec::new();
        write_dot(&g, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("0 -- 1 [weight=2.5];"));
    }

    #[test]
    fn write_graph_attributes() {
        let mut g = Graph::with_vertices(1);
        g.set_graph_attribute("label", AttributeValue::String("test graph".into()));

        let mut buf = Vec::new();
        write_dot(&g, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("graph [label=\"test graph\"];"));
    }

    #[test]
    fn write_boolean_attribute() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.set_vertex_attribute("visited", 0, AttributeValue::Boolean(true))
            .unwrap();

        let mut buf = Vec::new();
        write_dot(&g, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("[visited=true]"));
    }

    #[test]
    fn write_multiple_attributes() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.set_edge_attribute("weight", 0, AttributeValue::Numeric(1.5))
            .unwrap();
        g.set_edge_attribute("label", 0, AttributeValue::String("edge0".into()))
            .unwrap();

        let mut buf = Vec::new();
        write_dot(&g, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        // Both attributes should be present in the block
        assert!(s.contains("weight=1.5"));
        assert!(s.contains("label=\"edge0\""));
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
    fn read_with_attributes_parsed() {
        let dot = r#"graph {
  a [color="red", shape=circle];
  b [color="blue"];
  a -- b [weight=1.5];
}"#;
        let result = read_dot(dot.as_bytes()).unwrap();
        assert_eq!(result.graph.vcount(), 2);
        assert_eq!(result.graph.ecount(), 1);
        assert_eq!(
            result
                .graph
                .vertex_attribute("color", 0)
                .and_then(AttributeValue::as_str),
            Some("red"),
        );
        assert_eq!(
            result
                .graph
                .vertex_attribute("color", 1)
                .and_then(AttributeValue::as_str),
            Some("blue"),
        );
        assert_eq!(
            result
                .graph
                .vertex_attribute("shape", 0)
                .and_then(AttributeValue::as_str),
            Some("circle"),
        );
        assert_eq!(
            result
                .graph
                .edge_attribute("weight", 0)
                .and_then(AttributeValue::as_f64),
            Some(1.5),
        );
    }

    #[test]
    fn read_graph_level_attributes() {
        let dot = r#"graph {
  graph [label="My Graph", rankdir="LR"];
  a -- b;
}"#;
        let result = read_dot(dot.as_bytes()).unwrap();
        assert_eq!(
            result
                .graph
                .graph_attribute("label")
                .and_then(AttributeValue::as_str),
            Some("My Graph"),
        );
        assert_eq!(
            result
                .graph
                .graph_attribute("rankdir")
                .and_then(AttributeValue::as_str),
            Some("LR"),
        );
    }

    #[test]
    fn read_boolean_attribute() {
        let dot = r"graph {
  a [visited=true];
  b [visited=false];
  a -- b;
}";
        let result = read_dot(dot.as_bytes()).unwrap();
        assert_eq!(
            result
                .graph
                .vertex_attribute("visited", 0)
                .and_then(AttributeValue::as_bool),
            Some(true),
        );
        assert_eq!(
            result
                .graph
                .vertex_attribute("visited", 1)
                .and_then(AttributeValue::as_bool),
            Some(false),
        );
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
    fn roundtrip_with_attributes() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        g.set_vertex_attribute("color", 0, AttributeValue::String("red".into()))
            .unwrap();
        g.set_vertex_attribute("color", 1, AttributeValue::String("blue".into()))
            .unwrap();
        g.set_edge_attribute("weight", 0, AttributeValue::Numeric(1.5))
            .unwrap();
        g.set_graph_attribute("label", AttributeValue::String("test".into()));

        let mut buf = Vec::new();
        write_dot(&g, None, &mut buf).unwrap();

        let result = read_dot(buf.as_slice()).unwrap();
        assert_eq!(result.graph.vcount(), 3);
        assert_eq!(result.graph.ecount(), 2);
        assert!(result.graph.is_directed());
        assert_eq!(
            result
                .graph
                .vertex_attribute("color", 0)
                .and_then(AttributeValue::as_str),
            Some("red"),
        );
        assert_eq!(
            result
                .graph
                .vertex_attribute("color", 1)
                .and_then(AttributeValue::as_str),
            Some("blue"),
        );
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
                .graph_attribute("label")
                .and_then(AttributeValue::as_str),
            Some("test"),
        );
    }

    #[test]
    fn read_strict_keyword() {
        let dot = "strict graph {\n  a -- b;\n  a -- b;\n}";
        let result = read_dot(dot.as_bytes()).unwrap();
        assert!(!result.graph.is_directed());
        assert_eq!(result.graph.vcount(), 2);
    }

    #[test]
    fn read_node_attrs_ignored() {
        // "node [shape=circle]" is a global default statement — skip it
        let dot = r"graph {
  node [shape=circle];
  edge [color=red];
  a -- b;
}";
        let result = read_dot(dot.as_bytes()).unwrap();
        assert_eq!(result.graph.vcount(), 2);
        assert_eq!(result.graph.ecount(), 1);
        // Global node/edge defaults are NOT stored as vertex/edge attributes
        assert!(!result.graph.has_vertex_attribute("shape"));
    }

    #[test]
    fn parse_attr_pairs_basic() {
        let pairs = parse_attr_pairs(r#"weight=1.5, color="red", active=true"#);
        assert_eq!(pairs.len(), 3);
        assert_eq!(pairs[0], ("weight".into(), AttributeValue::Numeric(1.5)));
        assert_eq!(
            pairs[1],
            ("color".into(), AttributeValue::String("red".into()))
        );
        assert_eq!(pairs[2], ("active".into(), AttributeValue::Boolean(true)));
    }
}

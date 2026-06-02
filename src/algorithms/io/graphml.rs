//! `GraphML` reader / writer (ALGO-IO-011, ALGO-IO-012).
//!
//! Reads and writes graphs in the `GraphML` XML format. The reader uses a
//! lightweight custom XML parser (no external dependencies) that handles the
//! structural subset needed to recover graph topology: `<node id="..."/>` and
//! `<edge source="..." target="..."/>` elements within a `<graph>` block.
//! Non-structural attributes (keys, data elements) are silently skipped.
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
//! Counterparts of `igraph_read_graph_graphml` and `igraph_write_graph_graphml`.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};

use crate::core::{Graph, IgraphError, IgraphResult};

/// Result of parsing a `GraphML` file.
pub struct GraphmlGraph {
    /// The parsed graph.
    pub graph: Graph,
    /// Node labels (the `id` attributes from the `GraphML` `<node>` elements),
    /// in vertex-index order. Useful for mapping back to the original naming.
    pub labels: Vec<String>,
}

/// Read a graph from `GraphML` format.
///
/// Parses the first `<graph>` element in the input. Extracts
/// `edgedefault="directed"|"undirected"`, `<node id="..."/>` entries, and
/// `<edge source="..." target="..."/>` entries. All other elements (keys,
/// data, descriptions, ports, hyperedges) are silently skipped.
///
/// Node ids are arbitrary strings; vertices are assigned internal indices
/// in the order they first appear. The original string ids are returned in
/// [`GraphmlGraph::labels`].
///
/// # Errors
///
/// Returns an error if the input is not valid enough to extract a graph
/// (missing `<graph>` element, edges referencing undeclared nodes, etc.).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, read_graphml, write_graphml};
///
/// let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
/// <graphml xmlns="http://graphml.graphdrawing.org/xmlns">
///   <graph id="G" edgedefault="undirected">
///     <node id="Alice"/>
///     <node id="Bob"/>
///     <node id="Carol"/>
///     <edge source="Alice" target="Bob"/>
///     <edge source="Bob" target="Carol"/>
///   </graph>
/// </graphml>"#;
///
/// let result = read_graphml(xml.as_bytes()).unwrap();
/// assert_eq!(result.graph.vcount(), 3);
/// assert_eq!(result.graph.ecount(), 2);
/// assert!(!result.graph.is_directed());
/// assert_eq!(result.labels, vec!["Alice", "Bob", "Carol"]);
/// ```
pub fn read_graphml<R: Read>(input: R) -> IgraphResult<GraphmlGraph> {
    let reader = BufReader::new(input);
    let mut content = String::new();
    for line in reader.lines() {
        content.push_str(&line?);
        content.push('\n');
    }

    let parse_err = |msg: String| IgraphError::Parse {
        line: 0,
        message: msg,
    };

    // Find the <graph ...> element (not <graphml>).
    let graph_start = find_graph_element(&content)
        .ok_or_else(|| parse_err("no <graph> element found".to_owned()))?;
    let graph_end = content[graph_start..]
        .find("</graph>")
        .or_else(|| content[graph_start..].find("/>"))
        .ok_or_else(|| parse_err("unclosed <graph> element".to_owned()))?;
    let graph_section = &content[graph_start..graph_start + graph_end + 8];

    // Parse edgedefault attribute.
    let directed = parse_edgedefault(graph_section);

    // Parse nodes.
    let mut node_ids: Vec<String> = Vec::new();
    let mut node_map: HashMap<String, u32> = HashMap::new();
    let mut pos = 0;
    while let Some(node_start) = graph_section[pos..].find("<node") {
        let abs_start = pos + node_start;
        let tag_end = find_tag_end(&graph_section[abs_start..])
            .ok_or_else(|| parse_err("unclosed <node> tag".to_owned()))?;
        let tag = &graph_section[abs_start..abs_start + tag_end];
        if let Some(id) = extract_attr(tag, "id") {
            let idx = u32::try_from(node_ids.len())
                .map_err(|_| IgraphError::InvalidArgument("too many nodes for u32".to_owned()))?;
            node_map.insert(id.clone(), idx);
            node_ids.push(id);
        }
        pos = abs_start + tag_end;
    }

    // Build graph.
    let n = u32::try_from(node_ids.len())
        .map_err(|_| IgraphError::InvalidArgument("too many nodes for u32".to_owned()))?;
    let mut graph = Graph::new(n, directed)?;

    // Parse edges.
    pos = 0;
    while let Some(edge_start) = graph_section[pos..].find("<edge") {
        let abs_start = pos + edge_start;
        let tag_end = find_tag_end(&graph_section[abs_start..])
            .ok_or_else(|| parse_err("unclosed <edge> tag".to_owned()))?;
        let tag = &graph_section[abs_start..abs_start + tag_end];
        let source = extract_attr(tag, "source")
            .ok_or_else(|| parse_err("edge missing source attribute".to_owned()))?;
        let target = extract_attr(tag, "target")
            .ok_or_else(|| parse_err("edge missing target attribute".to_owned()))?;

        let src_idx = *node_map
            .get(&source)
            .ok_or_else(|| parse_err(format!("edge references unknown node \"{source}\"")))?;
        let tgt_idx = *node_map
            .get(&target)
            .ok_or_else(|| parse_err(format!("edge references unknown node \"{target}\"")))?;
        graph.add_edge(src_idx, tgt_idx)?;
        pos = abs_start + tag_end;
    }

    Ok(GraphmlGraph {
        graph,
        labels: node_ids,
    })
}

/// Find the start of a `<graph` element that is NOT `<graphml`.
fn find_graph_element(s: &str) -> Option<usize> {
    let mut search_from = 0;
    while let Some(pos) = s[search_from..].find("<graph") {
        let abs = search_from + pos;
        let after = abs + 6; // length of "<graph"
        match s.as_bytes().get(after) {
            // `<graph>` or `<graph ` — a valid graph element start.
            Some(b' ' | b'>' | b'/' | b'\t' | b'\n' | b'\r') | None => return Some(abs),
            // `<graphml` or similar — skip and keep searching.
            _ => search_from = abs + 1,
        }
    }
    None
}

/// Parse the `edgedefault` attribute from a `<graph ...>` opening tag.
fn parse_edgedefault(graph_tag: &str) -> bool {
    if let Some(pos) = graph_tag.find("edgedefault") {
        let rest = &graph_tag[pos..];
        if let Some(q1) = rest.find('"') {
            let after_q = &rest[q1 + 1..];
            if let Some(q2) = after_q.find('"') {
                let val = &after_q[..q2];
                return val == "directed";
            }
        }
        // Try single quotes.
        if let Some(q1) = rest.find('\'') {
            let after_q = &rest[q1 + 1..];
            if let Some(q2) = after_q.find('\'') {
                let val = &after_q[..q2];
                return val == "directed";
            }
        }
    }
    false
}

/// Find the end of an XML tag starting at position 0 (returns position after
/// `>` or `/>` including trailing `>`).
fn find_tag_end(s: &str) -> Option<usize> {
    // Handle potential nested content; just find the first '>'.
    s.find('>').map(|p| p + 1)
}

/// Extract the value of a named attribute from an XML tag string.
fn extract_attr(tag: &str, attr_name: &str) -> Option<String> {
    // Look for `attr_name="value"` or `attr_name='value'`.
    let needle = format!("{attr_name}=");
    let pos = tag.find(&needle)?;
    let rest = &tag[pos + needle.len()..];
    let quote = rest.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let after = &rest[1..];
    let end = after.find(quote)?;
    let raw = &after[..end];
    Some(xml_unescape(raw))
}

/// Unescape basic XML entities.
fn xml_unescape(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

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

    // --- read_graphml tests ---

    #[test]
    fn read_basic_undirected() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<graphml xmlns="http://graphml.graphdrawing.org/xmlns">
  <graph id="G" edgedefault="undirected">
    <node id="n0"/>
    <node id="n1"/>
    <node id="n2"/>
    <edge source="n0" target="n1"/>
    <edge source="n1" target="n2"/>
  </graph>
</graphml>"#;
        let result = read_graphml(xml.as_bytes()).unwrap();
        assert_eq!(result.graph.vcount(), 3);
        assert_eq!(result.graph.ecount(), 2);
        assert!(!result.graph.is_directed());
        assert_eq!(result.labels, vec!["n0", "n1", "n2"]);
    }

    #[test]
    fn read_directed() {
        let xml = r#"<graphml>
  <graph edgedefault="directed">
    <node id="a"/>
    <node id="b"/>
    <edge source="a" target="b"/>
  </graph>
</graphml>"#;
        let result = read_graphml(xml.as_bytes()).unwrap();
        assert!(result.graph.is_directed());
        assert_eq!(result.graph.vcount(), 2);
        assert_eq!(result.graph.ecount(), 1);
    }

    #[test]
    fn read_with_string_labels() {
        let xml = r#"<graphml>
  <graph edgedefault="undirected">
    <node id="Alice"/>
    <node id="Bob"/>
    <node id="Carol"/>
    <edge source="Alice" target="Bob"/>
    <edge source="Bob" target="Carol"/>
    <edge source="Carol" target="Alice"/>
  </graph>
</graphml>"#;
        let result = read_graphml(xml.as_bytes()).unwrap();
        assert_eq!(result.graph.vcount(), 3);
        assert_eq!(result.graph.ecount(), 3);
        assert_eq!(result.labels, vec!["Alice", "Bob", "Carol"]);
    }

    #[test]
    fn read_empty_graph() {
        let xml = r#"<graphml><graph edgedefault="undirected"></graph></graphml>"#;
        let result = read_graphml(xml.as_bytes()).unwrap();
        assert_eq!(result.graph.vcount(), 0);
        assert_eq!(result.graph.ecount(), 0);
    }

    #[test]
    fn read_nodes_only() {
        let xml = r#"<graphml>
  <graph edgedefault="undirected">
    <node id="x"/>
    <node id="y"/>
    <node id="z"/>
  </graph>
</graphml>"#;
        let result = read_graphml(xml.as_bytes()).unwrap();
        assert_eq!(result.graph.vcount(), 3);
        assert_eq!(result.graph.ecount(), 0);
    }

    #[test]
    fn read_xml_entity_unescaping() {
        let xml = r#"<graphml>
  <graph edgedefault="undirected">
    <node id="A&amp;B"/>
    <node id="C&lt;D"/>
    <edge source="A&amp;B" target="C&lt;D"/>
  </graph>
</graphml>"#;
        let result = read_graphml(xml.as_bytes()).unwrap();
        assert_eq!(result.labels, vec!["A&B", "C<D"]);
        assert_eq!(result.graph.ecount(), 1);
    }

    #[test]
    fn read_unknown_node_in_edge_is_error() {
        let xml = r#"<graphml>
  <graph edgedefault="undirected">
    <node id="a"/>
    <edge source="a" target="b"/>
  </graph>
</graphml>"#;
        assert!(read_graphml(xml.as_bytes()).is_err());
    }

    #[test]
    fn read_no_graph_element_is_error() {
        let xml = r#"<graphml><key id="k"/></graphml>"#;
        assert!(read_graphml(xml.as_bytes()).is_err());
    }

    #[test]
    fn roundtrip_write_then_read() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();

        let mut buf = Vec::new();
        write_graphml(&g, None, &mut buf).unwrap();

        let result = read_graphml(buf.as_slice()).unwrap();
        assert_eq!(result.graph.vcount(), 4);
        assert_eq!(result.graph.ecount(), 4);
        assert!(result.graph.is_directed());
    }

    #[test]
    fn roundtrip_with_labels() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let labels = vec!["X".to_string(), "Y".to_string(), "Z".to_string()];
        let mut buf = Vec::new();
        write_graphml(&g, Some(&labels), &mut buf).unwrap();

        let result = read_graphml(buf.as_slice()).unwrap();
        assert_eq!(result.graph.vcount(), 3);
        assert_eq!(result.graph.ecount(), 2);
        assert_eq!(result.labels, vec!["X", "Y", "Z"]);
    }

    #[test]
    fn read_ignores_extra_elements() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<graphml xmlns="http://graphml.graphdrawing.org/xmlns">
  <key id="d0" for="node" attr.name="label" attr.type="string"/>
  <graph id="G" edgedefault="undirected">
    <node id="n0">
      <data key="d0">Alice</data>
    </node>
    <node id="n1">
      <data key="d0">Bob</data>
    </node>
    <edge source="n0" target="n1">
      <data key="d1">1.5</data>
    </edge>
  </graph>
</graphml>"#;
        let result = read_graphml(xml.as_bytes()).unwrap();
        assert_eq!(result.graph.vcount(), 2);
        assert_eq!(result.graph.ecount(), 1);
    }
}

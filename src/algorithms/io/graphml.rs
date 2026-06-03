//! `GraphML` reader / writer (ALGO-IO-011, ALGO-IO-012).
//!
//! Reads and writes graphs in the `GraphML` XML format. The reader uses a
//! lightweight custom XML parser (no external dependencies) that handles
//! both structural elements (`<node>`, `<edge>`) and attribute data
//! (`<key>` definitions and `<data>` elements).
//!
//! Counterparts of `igraph_read_graph_graphml` and `igraph_write_graph_graphml`.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};

use crate::core::attributes::AttributeValue;
use crate::core::{Graph, IgraphError, IgraphResult};

/// Result of parsing a `GraphML` file.
pub struct GraphmlGraph {
    /// The parsed graph (with attributes populated).
    pub graph: Graph,
    /// Node labels (the `id` attributes from the `GraphML` `<node>` elements),
    /// in vertex-index order.
    pub labels: Vec<String>,
}

#[derive(Debug, Clone)]
struct KeyDef {
    attr_name: String,
    attr_type: String,
}

fn parse_attr_value(raw: &str, attr_type: &str) -> AttributeValue {
    match attr_type {
        "int" | "long" | "float" | "double" => {
            if let Ok(v) = raw.parse::<f64>() {
                AttributeValue::Numeric(v)
            } else {
                AttributeValue::String(raw.to_owned())
            }
        }
        "boolean" => {
            let v = matches!(raw, "true" | "1" | "yes");
            AttributeValue::Boolean(v)
        }
        _ => AttributeValue::String(raw.to_owned()),
    }
}

/// Read a graph from `GraphML` format.
///
/// Parses the first `<graph>` element in the input. Extracts structural
/// elements (`<node>`, `<edge>`) and attribute data (`<key>` definitions
/// and `<data>` elements). Attributes are stored on the returned graph
/// using the [`Graph`] attribute system.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, read_graphml, write_graphml, AttributeValue};
///
/// let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
/// <graphml xmlns="http://graphml.graphdrawing.org/xmlns">
///   <key id="d0" for="node" attr.name="label" attr.type="string"/>
///   <key id="d1" for="edge" attr.name="weight" attr.type="double"/>
///   <graph id="G" edgedefault="undirected">
///     <node id="Alice"><data key="d0">Alice</data></node>
///     <node id="Bob"><data key="d0">Bob</data></node>
///     <edge source="Alice" target="Bob"><data key="d1">1.5</data></edge>
///   </graph>
/// </graphml>"#;
///
/// let result = read_graphml(xml.as_bytes()).unwrap();
/// assert_eq!(result.graph.vcount(), 2);
/// assert_eq!(result.graph.ecount(), 1);
/// assert_eq!(
///     result.graph.vertex_attribute("label", 0).and_then(|v| v.as_str()),
///     Some("Alice"),
/// );
/// assert_eq!(
///     result.graph.edge_attribute("weight", 0).and_then(|v| v.as_f64()),
///     Some(1.5),
/// );
/// ```
#[allow(clippy::too_many_lines)]
pub fn read_graphml<R: Read>(input: R) -> IgraphResult<GraphmlGraph> {
    let reader = BufReader::new(input);
    let mut content = String::new();
    for line in reader.lines() {
        content.push_str(&line?);
        content.push('\n');
    }

    let keys = parse_key_definitions(&content)?;
    let graph_section = extract_graph_section(&content)?;
    let directed = parse_edgedefault(graph_section);

    let parsed = parse_nodes(graph_section)?;

    let n = u32::try_from(parsed.ids.len())
        .map_err(|_| IgraphError::InvalidArgument("too many nodes for u32".to_owned()))?;
    let mut graph = Graph::new(n, directed)?;

    if !parsed.ids.is_empty() {
        let label_vals: Vec<AttributeValue> = parsed
            .ids
            .iter()
            .map(|s| AttributeValue::String(s.clone()))
            .collect();
        graph.set_vertex_attribute_all("name", label_vals)?;
    }

    apply_node_attributes(&mut graph, &parsed.data, &keys)?;

    let edge_data_list = parse_and_add_edges(&mut graph, graph_section, &parsed.map)?;

    apply_edge_attributes(&mut graph, &edge_data_list, &keys)?;
    apply_graph_attributes(&mut graph, graph_section, &keys);

    Ok(GraphmlGraph {
        graph,
        labels: parsed.ids,
    })
}

fn parse_key_definitions(content: &str) -> IgraphResult<HashMap<String, KeyDef>> {
    let mut keys: HashMap<String, KeyDef> = HashMap::new();
    let mut pos = 0;
    while let Some(key_start) = content[pos..].find("<key") {
        let abs_start = pos + key_start;
        let after = abs_start + 4;
        match content.as_bytes().get(after) {
            Some(b' ' | b'>' | b'/' | b'\t' | b'\n' | b'\r') | None => {}
            _ => {
                pos = abs_start + 1;
                continue;
            }
        }
        let tag_end = find_tag_end(&content[abs_start..]).ok_or_else(|| IgraphError::Parse {
            line: 0,
            message: "unclosed <key> tag".to_owned(),
        })?;
        let tag = &content[abs_start..abs_start + tag_end];
        if let Some(id) = extract_attr(tag, "id") {
            let attr_name = extract_attr(tag, "attr.name").unwrap_or_else(|| id.clone());
            let attr_type = extract_attr(tag, "attr.type").unwrap_or_else(|| "string".to_owned());
            keys.insert(
                id,
                KeyDef {
                    attr_name,
                    attr_type,
                },
            );
        }
        pos = abs_start + tag_end;
    }
    Ok(keys)
}

fn extract_graph_section(content: &str) -> IgraphResult<&str> {
    let graph_start = find_graph_element(content).ok_or_else(|| IgraphError::Parse {
        line: 0,
        message: "no <graph> element found".to_owned(),
    })?;
    let graph_end = content[graph_start..]
        .find("</graph>")
        .or_else(|| content[graph_start..].find("/>"))
        .ok_or_else(|| IgraphError::Parse {
            line: 0,
            message: "unclosed <graph> element".to_owned(),
        })?;
    Ok(&content[graph_start..graph_start + graph_end + 8])
}

struct ParsedNodes {
    ids: Vec<String>,
    map: HashMap<String, u32>,
    data: Vec<Vec<(String, String)>>,
}

fn parse_nodes(graph_section: &str) -> IgraphResult<ParsedNodes> {
    let mut node_ids: Vec<String> = Vec::new();
    let mut node_map: HashMap<String, u32> = HashMap::new();
    let mut node_data: Vec<Vec<(String, String)>> = Vec::new();

    let mut pos = 0;
    while let Some(node_start) = graph_section[pos..].find("<node") {
        let abs_start = pos + node_start;
        let after = abs_start + 5;
        match graph_section.as_bytes().get(after) {
            Some(b' ' | b'>' | b'/' | b'\t' | b'\n' | b'\r') | None => {}
            _ => {
                pos = abs_start + 1;
                continue;
            }
        }

        let tag_end_pos =
            find_tag_end(&graph_section[abs_start..]).ok_or_else(|| IgraphError::Parse {
                line: 0,
                message: "unclosed <node> tag".to_owned(),
            })?;
        let tag = &graph_section[abs_start..abs_start + tag_end_pos];

        if let Some(id) = extract_attr(tag, "id") {
            let idx = u32::try_from(node_ids.len())
                .map_err(|_| IgraphError::InvalidArgument("too many nodes for u32".to_owned()))?;
            node_map.insert(id.clone(), idx);
            node_ids.push(id);

            let node_block_end;
            if tag.ends_with("/>") {
                node_block_end = abs_start + tag_end_pos;
                node_data.push(Vec::new());
            } else if let Some(close) = graph_section[abs_start..].find("</node>") {
                node_block_end = abs_start + close + 7;
                let block = &graph_section[abs_start + tag_end_pos..abs_start + close];
                node_data.push(extract_data_elements(block));
            } else {
                node_block_end = abs_start + tag_end_pos;
                node_data.push(Vec::new());
            }
            pos = node_block_end;
        } else {
            pos = abs_start + tag_end_pos;
        }
    }
    Ok(ParsedNodes {
        ids: node_ids,
        map: node_map,
        data: node_data,
    })
}

fn apply_node_attributes(
    graph: &mut Graph,
    node_data: &[Vec<(String, String)>],
    keys: &HashMap<String, KeyDef>,
) -> IgraphResult<()> {
    for (vid, data_items) in node_data.iter().enumerate() {
        for (key_id, raw_val) in data_items {
            if let Some(key_def) = keys.get(key_id) {
                let value = parse_attr_value(raw_val, &key_def.attr_type);
                let vid_u32 =
                    u32::try_from(vid).map_err(|_| IgraphError::Internal("vertex id overflow"))?;
                graph.set_vertex_attribute(&key_def.attr_name, vid_u32, value)?;
            }
        }
    }
    Ok(())
}

fn parse_and_add_edges(
    graph: &mut Graph,
    graph_section: &str,
    node_map: &HashMap<String, u32>,
) -> IgraphResult<Vec<Vec<(String, String)>>> {
    let mut edge_data_list: Vec<Vec<(String, String)>> = Vec::new();
    let mut pos = 0;
    while let Some(edge_start) = graph_section[pos..].find("<edge") {
        let abs_start = pos + edge_start;
        let tag_end_pos =
            find_tag_end(&graph_section[abs_start..]).ok_or_else(|| IgraphError::Parse {
                line: 0,
                message: "unclosed <edge> tag".to_owned(),
            })?;
        let tag = &graph_section[abs_start..abs_start + tag_end_pos];
        let source = extract_attr(tag, "source").ok_or_else(|| IgraphError::Parse {
            line: 0,
            message: "edge missing source attribute".to_owned(),
        })?;
        let target = extract_attr(tag, "target").ok_or_else(|| IgraphError::Parse {
            line: 0,
            message: "edge missing target attribute".to_owned(),
        })?;

        let src_idx = *node_map.get(&source).ok_or_else(|| IgraphError::Parse {
            line: 0,
            message: format!("edge references unknown node \"{source}\""),
        })?;
        let tgt_idx = *node_map.get(&target).ok_or_else(|| IgraphError::Parse {
            line: 0,
            message: format!("edge references unknown node \"{target}\""),
        })?;
        graph.add_edge(src_idx, tgt_idx)?;

        if tag.ends_with("/>") {
            edge_data_list.push(Vec::new());
        } else if let Some(close) = graph_section[abs_start..].find("</edge>") {
            let block = &graph_section[abs_start + tag_end_pos..abs_start + close];
            edge_data_list.push(extract_data_elements(block));
            pos = abs_start + close + 7;
            continue;
        } else {
            edge_data_list.push(Vec::new());
        }
        pos = abs_start + tag_end_pos;
    }
    Ok(edge_data_list)
}

fn apply_edge_attributes(
    graph: &mut Graph,
    edge_data_list: &[Vec<(String, String)>],
    keys: &HashMap<String, KeyDef>,
) -> IgraphResult<()> {
    for (eid, data_items) in edge_data_list.iter().enumerate() {
        for (key_id, raw_val) in data_items {
            if let Some(key_def) = keys.get(key_id) {
                let value = parse_attr_value(raw_val, &key_def.attr_type);
                let eid_u32 =
                    u32::try_from(eid).map_err(|_| IgraphError::Internal("edge id overflow"))?;
                graph.set_edge_attribute(&key_def.attr_name, eid_u32, value)?;
            }
        }
    }
    Ok(())
}

fn apply_graph_attributes(graph: &mut Graph, graph_section: &str, keys: &HashMap<String, KeyDef>) {
    let graph_tag_end = find_tag_end(graph_section).unwrap_or(graph_section.len());
    let graph_inner = &graph_section[graph_tag_end..];
    let first_child = graph_inner
        .find("<node")
        .unwrap_or(graph_inner.len())
        .min(graph_inner.find("<edge").unwrap_or(graph_inner.len()));
    let preamble = &graph_inner[..first_child];
    for (key_id, raw_val) in extract_data_elements(preamble) {
        if let Some(key_def) = keys.get(&key_id) {
            let value = parse_attr_value(&raw_val, &key_def.attr_type);
            graph.set_graph_attribute(&key_def.attr_name, value);
        }
    }
}

fn extract_data_elements(block: &str) -> Vec<(String, String)> {
    let mut result = Vec::new();
    let mut pos = 0;
    while let Some(data_start) = block[pos..].find("<data") {
        let abs_start = pos + data_start;
        let Some(tag_end) = find_tag_end(&block[abs_start..]) else {
            break;
        };
        let tag = &block[abs_start..abs_start + tag_end];

        if let Some(key_id) = extract_attr(tag, "key") {
            if tag.ends_with("/>") {
                result.push((key_id, String::new()));
                pos = abs_start + tag_end;
            } else if let Some(close) = block[abs_start..].find("</data>") {
                let content = &block[abs_start + tag_end..abs_start + close];
                let unescaped = xml_unescape(content.trim());
                result.push((key_id, unescaped));
                pos = abs_start + close + 7;
            } else {
                pos = abs_start + tag_end;
            }
        } else {
            pos = abs_start + tag_end;
        }
    }
    result
}

/// Find the start of a `<graph` element that is NOT `<graphml`.
fn find_graph_element(s: &str) -> Option<usize> {
    let mut search_from = 0;
    while let Some(pos) = s[search_from..].find("<graph") {
        let abs = search_from + pos;
        let after = abs + 6;
        match s.as_bytes().get(after) {
            Some(b' ' | b'>' | b'/' | b'\t' | b'\n' | b'\r') | None => return Some(abs),
            _ => search_from = abs + 1,
        }
    }
    None
}

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

fn find_tag_end(s: &str) -> Option<usize> {
    s.find('>').map(|p| p + 1)
}

fn extract_attr(tag: &str, attr_name: &str) -> Option<String> {
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

fn xml_unescape(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

/// Write a graph in `GraphML` format, including attributes.
///
/// If `labels` is provided, uses them as node IDs; otherwise uses `n0`, `n1`,
/// etc. Graph, vertex, and edge attributes from the attribute system are
/// written as `<key>` definitions and `<data>` elements.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, write_graphml, AttributeValue};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.set_vertex_attribute("label", 0, "Alice".into()).unwrap();
/// g.set_vertex_attribute("label", 1, "Bob".into()).unwrap();
/// g.set_vertex_attribute("label", 2, "Carol".into()).unwrap();
/// g.set_edge_attribute("weight", 0, 1.5.into()).unwrap();
///
/// let mut buf = Vec::new();
/// write_graphml(&g, None, &mut buf).unwrap();
/// let s = String::from_utf8(buf).unwrap();
/// assert!(s.contains("attr.name=\"label\""));
/// assert!(s.contains("attr.name=\"weight\""));
/// assert!(s.contains("<data key="));
/// ```
#[allow(clippy::too_many_lines)]
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

    let graph_attr_names = graph.graph_attribute_names();
    let vertex_attr_names = graph.vertex_attribute_names();
    let edge_attr_names = graph.edge_attribute_names();

    let mut key_counter = 0u32;
    let graph_key_map = write_graph_keys(graph, &graph_attr_names, &mut key_counter, writer)?;
    let vertex_key_map = write_vertex_keys(graph, &vertex_attr_names, &mut key_counter, writer)?;
    let edge_key_map = write_edge_keys(graph, &edge_attr_names, &mut key_counter, writer)?;

    writeln!(writer, "  <graph id=\"G\" edgedefault=\"{edge_default}\">")?;

    write_graph_data(graph, &graph_attr_names, &graph_key_map, writer)?;
    write_nodes(graph, &vertex_attr_names, &vertex_key_map, labels, writer)?;
    write_edges(graph, &edge_attr_names, &edge_key_map, labels, writer)?;

    writeln!(writer, "  </graph>")?;
    writeln!(writer, "</graphml>")?;

    Ok(())
}

fn write_graph_keys<W: Write>(
    graph: &Graph,
    names: &[&str],
    counter: &mut u32,
    writer: &mut W,
) -> IgraphResult<HashMap<String, String>> {
    let mut map = HashMap::new();
    for &name in names {
        let key_id = format!("g{counter}");
        *counter = counter.wrapping_add(1);
        let attr_type = graph.graph_attribute(name).map_or("string", attr_type_str);
        writeln!(
            writer,
            "  <key id=\"{key_id}\" for=\"graph\" attr.name=\"{}\" attr.type=\"{attr_type}\"/>",
            xml_escape(name)
        )?;
        map.insert(name.to_owned(), key_id);
    }
    Ok(map)
}

fn write_vertex_keys<W: Write>(
    graph: &Graph,
    names: &[&str],
    counter: &mut u32,
    writer: &mut W,
) -> IgraphResult<HashMap<String, String>> {
    let mut map = HashMap::new();
    for &name in names {
        let key_id = format!("v{counter}");
        *counter = counter.wrapping_add(1);
        let attr_type = graph
            .vertex_attributes(name)
            .and_then(|v| v.first())
            .map_or("string", attr_type_str);
        writeln!(
            writer,
            "  <key id=\"{key_id}\" for=\"node\" attr.name=\"{}\" attr.type=\"{attr_type}\"/>",
            xml_escape(name)
        )?;
        map.insert(name.to_owned(), key_id);
    }
    Ok(map)
}

fn write_edge_keys<W: Write>(
    graph: &Graph,
    names: &[&str],
    counter: &mut u32,
    writer: &mut W,
) -> IgraphResult<HashMap<String, String>> {
    let mut map = HashMap::new();
    for &name in names {
        let key_id = format!("e{counter}");
        *counter = counter.wrapping_add(1);
        let attr_type = graph
            .edge_attributes(name)
            .and_then(|v| v.first())
            .map_or("string", attr_type_str);
        writeln!(
            writer,
            "  <key id=\"{key_id}\" for=\"edge\" attr.name=\"{}\" attr.type=\"{attr_type}\"/>",
            xml_escape(name)
        )?;
        map.insert(name.to_owned(), key_id);
    }
    Ok(map)
}

fn write_graph_data<W: Write>(
    graph: &Graph,
    names: &[&str],
    key_map: &HashMap<String, String>,
    writer: &mut W,
) -> IgraphResult<()> {
    for &name in names {
        if let (Some(key_id), Some(val)) = (key_map.get(name), graph.graph_attribute(name)) {
            writeln!(
                writer,
                "    <data key=\"{key_id}\">{}</data>",
                xml_escape(&val.to_string())
            )?;
        }
    }
    Ok(())
}

fn write_nodes<W: Write>(
    graph: &Graph,
    attr_names: &[&str],
    key_map: &HashMap<String, String>,
    labels: Option<&[String]>,
    writer: &mut W,
) -> IgraphResult<()> {
    for v in 0..graph.vcount() {
        let node_id = vertex_id(v, labels);
        let has_data = attr_names
            .iter()
            .any(|name| graph.vertex_attribute(name, v).is_some());
        if has_data {
            writeln!(writer, "    <node id=\"{}\">", xml_escape(&node_id))?;
            for &name in attr_names {
                if let (Some(key_id), Some(val)) =
                    (key_map.get(name), graph.vertex_attribute(name, v))
                {
                    writeln!(
                        writer,
                        "      <data key=\"{key_id}\">{}</data>",
                        xml_escape(&val.to_string())
                    )?;
                }
            }
            writeln!(writer, "    </node>")?;
        } else {
            writeln!(writer, "    <node id=\"{}\"/>", xml_escape(&node_id))?;
        }
    }
    Ok(())
}

fn write_edges<W: Write>(
    graph: &Graph,
    attr_names: &[&str],
    key_map: &HashMap<String, String>,
    labels: Option<&[String]>,
    writer: &mut W,
) -> IgraphResult<()> {
    for eid in 0..graph.ecount() {
        let eid_u32 = u32::try_from(eid).map_err(|_| IgraphError::Internal("edge id overflow"))?;
        let (from, to) = graph.edge(eid_u32)?;
        let src_id = vertex_id(from, labels);
        let tgt_id = vertex_id(to, labels);
        let has_data = attr_names
            .iter()
            .any(|name| graph.edge_attribute(name, eid_u32).is_some());
        if has_data {
            writeln!(
                writer,
                "    <edge source=\"{}\" target=\"{}\">",
                xml_escape(&src_id),
                xml_escape(&tgt_id)
            )?;
            for &name in attr_names {
                if let (Some(key_id), Some(val)) =
                    (key_map.get(name), graph.edge_attribute(name, eid_u32))
                {
                    writeln!(
                        writer,
                        "      <data key=\"{key_id}\">{}</data>",
                        xml_escape(&val.to_string())
                    )?;
                }
            }
            writeln!(writer, "    </edge>")?;
        } else {
            writeln!(
                writer,
                "    <edge source=\"{}\" target=\"{}\"/>",
                xml_escape(&src_id),
                xml_escape(&tgt_id)
            )?;
        }
    }
    Ok(())
}

fn attr_type_str(val: &AttributeValue) -> &'static str {
    match val {
        AttributeValue::Numeric(_) => "double",
        AttributeValue::Boolean(_) => "boolean",
        AttributeValue::String(_) => "string",
    }
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
        assert!(s.contains("n0"));
        assert!(s.contains("n1"));
        assert!(s.contains("n2"));
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

        assert!(s.contains("Alice"));
        assert!(s.contains("Bob"));
    }

    #[test]
    fn test_xml_escaping() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();

        let labels = vec!["A&B".to_string(), "C<D".to_string()];
        let mut buf = Vec::new();
        write_graphml(&g, Some(&labels), &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();

        assert!(s.contains("A&amp;B"));
        assert!(s.contains("C&lt;D"));
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
    fn test_self_loop() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();

        let mut buf = Vec::new();
        write_graphml(&g, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();

        assert!(s.contains("source=\"n0\" target=\"n0\""));
    }

    #[test]
    fn test_labels_mismatch_error() {
        let g = Graph::with_vertices(3);
        let labels = vec!["A".to_string()];
        let mut buf = Vec::new();
        assert!(write_graphml(&g, Some(&labels), &mut buf).is_err());
    }

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
    }

    #[test]
    fn read_empty_graph() {
        let xml = r#"<graphml><graph edgedefault="undirected"></graph></graphml>"#;
        let result = read_graphml(xml.as_bytes()).unwrap();
        assert_eq!(result.graph.vcount(), 0);
        assert_eq!(result.graph.ecount(), 0);
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
    fn read_with_attributes() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<graphml xmlns="http://graphml.graphdrawing.org/xmlns">
  <key id="d0" for="node" attr.name="label" attr.type="string"/>
  <key id="d1" for="edge" attr.name="weight" attr.type="double"/>
  <key id="d2" for="graph" attr.name="name" attr.type="string"/>
  <graph id="G" edgedefault="undirected">
    <data key="d2">TestGraph</data>
    <node id="n0">
      <data key="d0">Alice</data>
    </node>
    <node id="n1">
      <data key="d0">Bob</data>
    </node>
    <edge source="n0" target="n1">
      <data key="d1">2.5</data>
    </edge>
  </graph>
</graphml>"#;
        let result = read_graphml(xml.as_bytes()).unwrap();
        assert_eq!(result.graph.vcount(), 2);
        assert_eq!(result.graph.ecount(), 1);

        assert_eq!(
            result
                .graph
                .vertex_attribute("label", 0)
                .and_then(|v| v.as_str()),
            Some("Alice"),
        );
        assert_eq!(
            result
                .graph
                .vertex_attribute("label", 1)
                .and_then(|v| v.as_str()),
            Some("Bob"),
        );

        let w = result
            .graph
            .edge_attribute("weight", 0)
            .and_then(AttributeValue::as_f64);
        assert!((w.unwrap() - 2.5).abs() < f64::EPSILON);

        assert_eq!(
            result
                .graph
                .graph_attribute("name")
                .and_then(|v| v.as_str()),
            Some("TestGraph"),
        );
    }

    #[test]
    fn write_with_attributes() {
        let mut g = Graph::from_edges(&[(0, 1)], false, None).unwrap();
        g.set_vertex_attribute("color", 0, "red".into()).unwrap();
        g.set_vertex_attribute("color", 1, "blue".into()).unwrap();
        g.set_edge_attribute("weight", 0, 3.0.into()).unwrap();
        g.set_graph_attribute("title", "test".into());

        let mut buf = Vec::new();
        write_graphml(&g, None, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();

        assert!(s.contains("attr.name=\"color\""));
        assert!(s.contains("attr.name=\"weight\""));
        assert!(s.contains("attr.name=\"title\""));
        assert!(s.contains("<data key="));
        assert!(s.contains("red"));
        assert!(s.contains("blue"));
        assert!(s.contains("test"));
    }

    #[test]
    fn roundtrip_with_attributes() {
        let mut g = Graph::from_edges(&[(0, 1), (1, 2)], false, None).unwrap();
        g.set_vertex_attribute("label", 0, "A".into()).unwrap();
        g.set_vertex_attribute("label", 1, "B".into()).unwrap();
        g.set_vertex_attribute("label", 2, "C".into()).unwrap();
        g.set_edge_attribute("weight", 0, 1.5.into()).unwrap();
        g.set_edge_attribute("weight", 1, 2.5.into()).unwrap();
        g.set_graph_attribute("name", "roundtrip_test".into());

        let mut buf = Vec::new();
        write_graphml(&g, None, &mut buf).unwrap();

        let result = read_graphml(buf.as_slice()).unwrap();
        assert_eq!(result.graph.vcount(), 3);
        assert_eq!(result.graph.ecount(), 2);
        assert_eq!(
            result
                .graph
                .vertex_attribute("label", 0)
                .and_then(|v| v.as_str()),
            Some("A"),
        );
        assert_eq!(
            result
                .graph
                .graph_attribute("name")
                .and_then(|v| v.as_str()),
            Some("roundtrip_test"),
        );
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
        assert_eq!(
            result
                .graph
                .vertex_attribute("label", 0)
                .and_then(|v| v.as_str()),
            Some("Alice"),
        );
    }

    #[test]
    fn read_boolean_attribute() {
        let xml = r#"<graphml>
  <key id="d0" for="node" attr.name="active" attr.type="boolean"/>
  <graph edgedefault="undirected">
    <node id="n0"><data key="d0">true</data></node>
    <node id="n1"><data key="d0">false</data></node>
  </graph>
</graphml>"#;
        let result = read_graphml(xml.as_bytes()).unwrap();
        assert_eq!(
            result
                .graph
                .vertex_attribute("active", 0)
                .and_then(AttributeValue::as_bool),
            Some(true),
        );
        assert_eq!(
            result
                .graph
                .vertex_attribute("active", 1)
                .and_then(AttributeValue::as_bool),
            Some(false),
        );
    }
}

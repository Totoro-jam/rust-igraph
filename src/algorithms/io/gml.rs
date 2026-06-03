//! GML (Graph Modelling Language) I/O (ALGO-IO-001).
//!
//! Reads and writes graphs in GML format. GML is a hierarchical key-value
//! format widely used in network science tools.
//!
//! Reference: <https://web.archive.org/web/20190207140002/http://www.fim.uni-passau.de/index.php?id=17297&L=1>
//!
//! Structural keys (`directed`, `id`, `source`, `target`) drive graph
//! construction. All other keys inside `node` and `edge` blocks are stored
//! as vertex/edge attributes via the [`Graph`] attribute system. Graph-level
//! keys (e.g. `Creator`) are stored as graph attributes.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};

use crate::core::attributes::AttributeValue;
use crate::core::{Graph, IgraphError, IgraphResult};

/// Read a graph from GML format.
///
/// Parses the first `graph [ ... ]` block. Extracts `directed` (0 or 1,
/// default 0), `node [ id N ]` entries, and `edge [ source N target N ]`
/// entries. Non-structural keys within node/edge blocks (e.g. `label`,
/// `weight`) are stored as vertex/edge attributes.
///
/// Node ids need not be contiguous or start at 0 — the reader maps them
/// to internal vertex indices in the order they appear.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, read_gml, AttributeValue};
///
/// let gml = b"graph [
///   directed 0
///   node [ id 1 label \"Alice\" ]
///   node [ id 2 label \"Bob\" ]
///   edge [ source 1 target 2 weight 1.5 ]
/// ]";
/// let g = read_gml(&gml[..]).unwrap();
/// assert_eq!(g.vcount(), 2);
/// assert_eq!(g.ecount(), 1);
/// assert!(!g.is_directed());
/// assert_eq!(
///     g.vertex_attribute("label", 0).and_then(|v| v.as_str()),
///     Some("Alice"),
/// );
/// assert_eq!(
///     g.edge_attribute("weight", 0).and_then(|v| v.as_f64()),
///     Some(1.5),
/// );
/// ```
pub fn read_gml<R: Read>(input: R) -> IgraphResult<Graph> {
    let reader = BufReader::new(input);
    let tokens = tokenize(reader)?;
    parse_graph(&tokens)
}

/// Write a graph in GML format, including attributes.
///
/// Produces a valid GML file with `graph [ directed 0/1 node [ id N ... ]
/// ... edge [ source N target N ... ] ... ]`. Graph, vertex, and edge
/// attributes are written as additional key-value pairs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, write_gml, read_gml, AttributeValue};
///
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.set_vertex_attribute("label", 0, "A".into()).unwrap();
/// g.set_edge_attribute("weight", 0, 2.5.into()).unwrap();
///
/// let mut buf = Vec::new();
/// write_gml(&g, &mut buf).unwrap();
/// let s = String::from_utf8(buf).unwrap();
/// assert!(s.contains("directed 1"));
/// assert!(s.contains("label \"A\""));
/// assert!(s.contains("weight 2.5"));
///
/// // Round-trip
/// let g2 = read_gml(s.as_bytes()).unwrap();
/// assert_eq!(g2.vcount(), 3);
/// assert_eq!(g2.ecount(), 2);
/// assert!(g2.is_directed());
/// ```
pub fn write_gml<W: Write>(graph: &Graph, writer: &mut W) -> IgraphResult<()> {
    writeln!(writer, "graph")?;
    writeln!(writer, "[")?;
    writeln!(writer, "  directed {}", i32::from(graph.is_directed()))?;

    write_graph_attrs(graph, writer)?;
    write_node_blocks(graph, writer)?;
    write_edge_blocks(graph, writer)?;

    writeln!(writer, "]")?;
    Ok(())
}

fn write_graph_attrs<W: Write>(graph: &Graph, writer: &mut W) -> IgraphResult<()> {
    for &name in &graph.graph_attribute_names() {
        if let Some(val) = graph.graph_attribute(name) {
            write!(writer, "  {name} ")?;
            write_gml_value(val, writer)?;
            writeln!(writer)?;
        }
    }
    Ok(())
}

fn write_node_blocks<W: Write>(graph: &Graph, writer: &mut W) -> IgraphResult<()> {
    let attr_names = graph.vertex_attribute_names();
    for v in 0..graph.vcount() {
        writeln!(writer, "  node")?;
        writeln!(writer, "  [")?;
        writeln!(writer, "    id {v}")?;
        for &name in &attr_names {
            if let Some(val) = graph.vertex_attribute(name, v) {
                write!(writer, "    {name} ")?;
                write_gml_value(val, writer)?;
                writeln!(writer)?;
            }
        }
        writeln!(writer, "  ]")?;
    }
    Ok(())
}

fn write_edge_blocks<W: Write>(graph: &Graph, writer: &mut W) -> IgraphResult<()> {
    let attr_names = graph.edge_attribute_names();
    for eid in 0..graph.ecount() {
        let eid_u32 = u32::try_from(eid).map_err(|_| IgraphError::Internal("edge id overflow"))?;
        let (src, tgt) = graph.edge(eid_u32)?;
        writeln!(writer, "  edge")?;
        writeln!(writer, "  [")?;
        writeln!(writer, "    source {src}")?;
        writeln!(writer, "    target {tgt}")?;
        for &name in &attr_names {
            if let Some(val) = graph.edge_attribute(name, eid_u32) {
                write!(writer, "    {name} ")?;
                write_gml_value(val, writer)?;
                writeln!(writer)?;
            }
        }
        writeln!(writer, "  ]")?;
    }
    Ok(())
}

fn write_gml_value<W: Write>(val: &AttributeValue, writer: &mut W) -> IgraphResult<()> {
    match val {
        AttributeValue::Numeric(v) => {
            write!(writer, "{v}")?;
        }
        AttributeValue::Boolean(b) => {
            write!(writer, "{}", i32::from(*b))?;
        }
        AttributeValue::String(s) => {
            write!(writer, "\"{}\"", gml_escape(s))?;
        }
    }
    Ok(())
}

fn gml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("&quot;"),
            '\\' => out.push_str("\\\\"),
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

// --- Tokenizer ---

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Key(String),
    Integer(i64),
    Float(f64),
    Str(String),
    Open,  // [
    Close, // ]
}

fn tokenize<R: BufRead>(reader: R) -> IgraphResult<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut line_no: usize = 0;

    for line_result in reader.lines() {
        line_no = line_no.wrapping_add(1);
        let line = line_result?;
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        let mut chars = trimmed.chars().peekable();
        while let Some(&ch) = chars.peek() {
            match ch {
                ' ' | '\t' | '\r' => {
                    chars.next();
                }
                '[' => {
                    tokens.push(Token::Open);
                    chars.next();
                }
                ']' => {
                    tokens.push(Token::Close);
                    chars.next();
                }
                '"' => {
                    chars.next();
                    let s = read_quoted_string(&mut chars, line_no)?;
                    let decoded = decode_entities(&s);
                    tokens.push(Token::Str(decoded));
                }
                '#' => break,
                _ => {
                    let word = read_word(&mut chars);
                    if word.is_empty() {
                        return Err(IgraphError::Parse {
                            line: line_no,
                            message: format!("unexpected character '{ch}'"),
                        });
                    }
                    if let Ok(i) = word.parse::<i64>() {
                        tokens.push(Token::Integer(i));
                    } else if let Ok(f) = parse_gml_float(&word) {
                        tokens.push(Token::Float(f));
                    } else {
                        tokens.push(Token::Key(word));
                    }
                }
            }
        }
    }

    Ok(tokens)
}

fn read_quoted_string(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    line_no: usize,
) -> IgraphResult<String> {
    let mut s = String::new();
    let mut escaped = false;
    loop {
        match chars.next() {
            None => {
                return Err(IgraphError::Parse {
                    line: line_no,
                    message: "unterminated string".into(),
                });
            }
            Some('\\') if !escaped => {
                escaped = true;
            }
            Some('"') if !escaped => break,
            Some(c) => {
                if escaped {
                    match c {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        '\\' => s.push('\\'),
                        '"' => s.push('"'),
                        _ => {
                            s.push('\\');
                            s.push(c);
                        }
                    }
                    escaped = false;
                } else {
                    s.push(c);
                }
            }
        }
    }
    Ok(s)
}

fn read_word(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut word = String::new();
    while let Some(&c) = chars.peek() {
        if c == ' ' || c == '\t' || c == '[' || c == ']' || c == '"' {
            break;
        }
        word.push(c);
        chars.next();
    }
    word
}

fn parse_gml_float(s: &str) -> Result<f64, ()> {
    let lower = s.to_ascii_lowercase();
    match lower.as_str() {
        "inf" | "+inf" => Ok(f64::INFINITY),
        "-inf" => Ok(f64::NEG_INFINITY),
        "nan" => Ok(f64::NAN),
        _ => s.parse::<f64>().map_err(|_| ()),
    }
}

fn decode_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

// --- Parser ---

struct NodeAttrs {
    id: i64,
    attrs: Vec<(String, AttributeValue)>,
}

struct EdgeAttrs {
    source: i64,
    target: i64,
    attrs: Vec<(String, AttributeValue)>,
}

fn parse_graph(tokens: &[Token]) -> IgraphResult<Graph> {
    let mut pos = 0;

    // Collect top-level (graph-level) attributes before "graph" keyword.
    let mut graph_attrs: Vec<(String, AttributeValue)> = Vec::new();

    while pos < tokens.len() {
        match &tokens[pos] {
            Token::Key(k) if k == "graph" => {
                pos += 1;
                break;
            }
            Token::Key(k) => {
                let key = k.clone();
                pos += 1;
                if let Some((val, new_pos)) = try_read_simple_value(tokens, pos) {
                    graph_attrs.push((key, val));
                    pos = new_pos;
                } else {
                    pos = skip_value(tokens, pos);
                }
            }
            _ => {
                pos += 1;
            }
        }
    }

    if pos >= tokens.len() {
        return Err(IgraphError::Parse {
            line: 0,
            message: "no 'graph' block found in GML input".into(),
        });
    }
    if tokens[pos] != Token::Open {
        return Err(IgraphError::Parse {
            line: 0,
            message: "expected '[' after 'graph'".into(),
        });
    }
    pos += 1;

    let mut directed = false;
    let mut nodes: Vec<NodeAttrs> = Vec::new();
    let mut edges: Vec<EdgeAttrs> = Vec::new();

    while pos < tokens.len() && tokens[pos] != Token::Close {
        match &tokens[pos] {
            Token::Key(k) => {
                let key = k.clone();
                pos += 1;

                match key.as_str() {
                    "directed" => {
                        if let Some(Token::Integer(v)) = tokens.get(pos) {
                            directed = *v != 0;
                            pos += 1;
                        } else {
                            pos = skip_value(tokens, pos);
                        }
                    }
                    "node" if pos < tokens.len() && tokens[pos] == Token::Open => {
                        pos += 1;
                        let (node, new_pos) = parse_node(tokens, pos)?;
                        nodes.push(node);
                        pos = new_pos;
                    }
                    "edge" if pos < tokens.len() && tokens[pos] == Token::Open => {
                        pos += 1;
                        let (edge, new_pos) = parse_edge(tokens, pos)?;
                        edges.push(edge);
                        pos = new_pos;
                    }
                    _ => {
                        if let Some((val, new_pos)) = try_read_simple_value(tokens, pos) {
                            graph_attrs.push((key, val));
                            pos = new_pos;
                        } else {
                            pos = skip_value(tokens, pos);
                        }
                    }
                }
            }
            _ => {
                return Err(IgraphError::Parse {
                    line: 0,
                    message: format!("unexpected token in graph block: {:?}", tokens[pos]),
                });
            }
        }
    }

    build_graph(directed, &nodes, &edges, &graph_attrs)
}

fn try_read_simple_value(tokens: &[Token], pos: usize) -> Option<(AttributeValue, usize)> {
    match tokens.get(pos)? {
        #[allow(clippy::cast_precision_loss)]
        Token::Integer(i) => Some((AttributeValue::Numeric(*i as f64), pos + 1)),
        Token::Float(f) => Some((AttributeValue::Numeric(*f), pos + 1)),
        Token::Str(s) => Some((AttributeValue::String(s.clone()), pos + 1)),
        _ => None,
    }
}

fn parse_node(tokens: &[Token], mut pos: usize) -> IgraphResult<(NodeAttrs, usize)> {
    let mut node_id: Option<i64> = None;
    let mut attrs: Vec<(String, AttributeValue)> = Vec::new();

    while pos < tokens.len() && tokens[pos] != Token::Close {
        match &tokens[pos] {
            Token::Key(k) => {
                let key = k.clone();
                pos += 1;
                if key == "id" {
                    if let Some(Token::Integer(v)) = tokens.get(pos) {
                        node_id = Some(*v);
                        pos += 1;
                    } else {
                        pos = skip_value(tokens, pos);
                    }
                } else if let Some((val, new_pos)) = try_read_simple_value(tokens, pos) {
                    attrs.push((key, val));
                    pos = new_pos;
                } else {
                    pos = skip_value(tokens, pos);
                }
            }
            _ => {
                return Err(IgraphError::Parse {
                    line: 0,
                    message: format!("unexpected token in node block: {:?}", tokens[pos]),
                });
            }
        }
    }

    if pos < tokens.len() && tokens[pos] == Token::Close {
        pos += 1;
    }

    let id = node_id.ok_or_else(|| IgraphError::Parse {
        line: 0,
        message: "node without 'id' field".into(),
    })?;

    Ok((NodeAttrs { id, attrs }, pos))
}

fn parse_edge(tokens: &[Token], mut pos: usize) -> IgraphResult<(EdgeAttrs, usize)> {
    let mut source: Option<i64> = None;
    let mut target: Option<i64> = None;
    let mut attrs: Vec<(String, AttributeValue)> = Vec::new();

    while pos < tokens.len() && tokens[pos] != Token::Close {
        match &tokens[pos] {
            Token::Key(k) => {
                let key = k.clone();
                pos += 1;
                match key.as_str() {
                    "source" => {
                        if let Some(Token::Integer(v)) = tokens.get(pos) {
                            source = Some(*v);
                            pos += 1;
                        } else {
                            pos = skip_value(tokens, pos);
                        }
                    }
                    "target" => {
                        if let Some(Token::Integer(v)) = tokens.get(pos) {
                            target = Some(*v);
                            pos += 1;
                        } else {
                            pos = skip_value(tokens, pos);
                        }
                    }
                    _ => {
                        if let Some((val, new_pos)) = try_read_simple_value(tokens, pos) {
                            attrs.push((key, val));
                            pos = new_pos;
                        } else {
                            pos = skip_value(tokens, pos);
                        }
                    }
                }
            }
            _ => {
                return Err(IgraphError::Parse {
                    line: 0,
                    message: format!("unexpected token in edge block: {:?}", tokens[pos]),
                });
            }
        }
    }

    if pos < tokens.len() && tokens[pos] == Token::Close {
        pos += 1;
    }

    let src = source.ok_or_else(|| IgraphError::Parse {
        line: 0,
        message: "edge without 'source' field".into(),
    })?;
    let tgt = target.ok_or_else(|| IgraphError::Parse {
        line: 0,
        message: "edge without 'target' field".into(),
    })?;

    Ok((
        EdgeAttrs {
            source: src,
            target: tgt,
            attrs,
        },
        pos,
    ))
}

fn skip_value(tokens: &[Token], mut pos: usize) -> usize {
    if pos >= tokens.len() {
        return pos;
    }
    match &tokens[pos] {
        Token::Integer(_) | Token::Float(_) | Token::Str(_) | Token::Key(_) => pos + 1,
        Token::Open => {
            pos += 1;
            let mut depth: u32 = 1;
            while pos < tokens.len() && depth > 0 {
                match &tokens[pos] {
                    Token::Open => depth = depth.saturating_add(1),
                    Token::Close => depth = depth.saturating_sub(1),
                    _ => {}
                }
                pos += 1;
            }
            pos
        }
        Token::Close => pos,
    }
}

fn build_graph(
    directed: bool,
    nodes: &[NodeAttrs],
    edges: &[EdgeAttrs],
    graph_attrs: &[(String, AttributeValue)],
) -> IgraphResult<Graph> {
    let mut id_map: HashMap<i64, u32> = HashMap::with_capacity(nodes.len());
    for (idx, node) in nodes.iter().enumerate() {
        let internal_id =
            u32::try_from(idx).map_err(|_| IgraphError::Internal("node index overflow"))?;
        if id_map.insert(node.id, internal_id).is_some() {
            return Err(IgraphError::Parse {
                line: 0,
                message: format!("duplicate node id {}", node.id),
            });
        }
    }

    let n = u32::try_from(nodes.len()).map_err(|_| IgraphError::Internal("node count overflow"))?;
    let mut graph = Graph::new(n, directed)?;

    // Apply graph-level attributes.
    for (key, val) in graph_attrs {
        graph.set_graph_attribute(key, val.clone());
    }

    // Apply vertex attributes.
    for (idx, node) in nodes.iter().enumerate() {
        let vid = u32::try_from(idx).map_err(|_| IgraphError::Internal("vertex id overflow"))?;
        for (key, val) in &node.attrs {
            graph.set_vertex_attribute(key, vid, val.clone())?;
        }
    }

    // Add edges and apply edge attributes.
    for (eid_idx, edge) in edges.iter().enumerate() {
        let src = id_map.get(&edge.source).ok_or_else(|| IgraphError::Parse {
            line: 0,
            message: format!("edge references unknown node id {}", edge.source),
        })?;
        let tgt = id_map.get(&edge.target).ok_or_else(|| IgraphError::Parse {
            line: 0,
            message: format!("edge references unknown node id {}", edge.target),
        })?;
        graph.add_edge(*src, *tgt)?;

        let eid = u32::try_from(eid_idx).map_err(|_| IgraphError::Internal("edge id overflow"))?;
        for (key, val) in &edge.attrs {
            graph.set_edge_attribute(key, eid, val.clone())?;
        }
    }

    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let gml = b"graph [ ]";
        let g = read_gml(&gml[..]).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
        assert!(!g.is_directed());
    }

    #[test]
    fn test_directed_flag() {
        let gml = b"graph [ directed 1 node [ id 0 ] node [ id 1 ] edge [ source 0 target 1 ] ]";
        let g = read_gml(&gml[..]).unwrap();
        assert!(g.is_directed());
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 1);
    }

    #[test]
    fn test_undirected_default() {
        let gml = b"graph [ node [ id 0 ] node [ id 1 ] edge [ source 0 target 1 ] ]";
        let g = read_gml(&gml[..]).unwrap();
        assert!(!g.is_directed());
    }

    #[test]
    fn test_non_contiguous_ids() {
        let gml = b"graph [\n  node [ id 10 ]\n  node [ id 20 ]\n  node [ id 30 ]\n  edge [ source 10 target 30 ]\n]";
        let g = read_gml(&gml[..]).unwrap();
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 1);
        let (src, tgt) = g.edge(0).unwrap();
        assert_eq!(src, 0);
        assert_eq!(tgt, 2);
    }

    #[test]
    fn test_multiline_with_attributes() {
        let gml = b"Creator \"test\"\ngraph\n[\n  directed 0\n  node\n  [\n    id 0\n    label \"A\"\n  ]\n  node\n  [\n    id 1\n    label \"B\"\n  ]\n  edge\n  [\n    source 0\n    target 1\n    weight 1.5\n  ]\n]\n";
        let g = read_gml(&gml[..]).unwrap();
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 1);
        assert_eq!(
            g.vertex_attribute("label", 0).and_then(|v| v.as_str()),
            Some("A"),
        );
        assert_eq!(
            g.vertex_attribute("label", 1).and_then(|v| v.as_str()),
            Some("B"),
        );
        let w = g
            .edge_attribute("weight", 0)
            .and_then(AttributeValue::as_f64);
        assert!((w.unwrap() - 1.5).abs() < f64::EPSILON);
        assert_eq!(
            g.graph_attribute("Creator").and_then(|v| v.as_str()),
            Some("test"),
        );
    }

    #[test]
    fn test_self_loop() {
        let gml = b"graph [ node [ id 0 ] edge [ source 0 target 0 ] ]";
        let g = read_gml(&gml[..]).unwrap();
        assert_eq!(g.ecount(), 1);
        let (s, t) = g.edge(0).unwrap();
        assert_eq!(s, t);
    }

    #[test]
    fn test_multiple_edges() {
        let gml = b"graph [ node [ id 0 ] node [ id 1 ] edge [ source 0 target 1 ] edge [ source 0 target 1 ] ]";
        let g = read_gml(&gml[..]).unwrap();
        assert_eq!(g.ecount(), 2);
    }

    #[test]
    fn test_error_missing_graph() {
        let gml = b"node [ id 0 ]";
        let result = read_gml(&gml[..]);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_duplicate_node_id() {
        let gml = b"graph [ node [ id 0 ] node [ id 0 ] ]";
        let result = read_gml(&gml[..]);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_unknown_edge_target() {
        let gml = b"graph [ node [ id 0 ] edge [ source 0 target 99 ] ]";
        let result = read_gml(&gml[..]);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_node_without_id() {
        let gml = b"graph [ node [ label \"x\" ] ]";
        let result = read_gml(&gml[..]);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_edge_without_source() {
        let gml = b"graph [ node [ id 0 ] node [ id 1 ] edge [ target 1 ] ]";
        let result = read_gml(&gml[..]);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_read_round_trip_undirected() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();

        let mut buf = Vec::new();
        write_gml(&g, &mut buf).unwrap();

        let g2 = read_gml(&buf[..]).unwrap();
        assert_eq!(g2.vcount(), 4);
        assert_eq!(g2.ecount(), 4);
        assert!(!g2.is_directed());
    }

    #[test]
    fn test_write_read_round_trip_directed() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();

        let mut buf = Vec::new();
        write_gml(&g, &mut buf).unwrap();

        let g2 = read_gml(&buf[..]).unwrap();
        assert_eq!(g2.vcount(), 3);
        assert_eq!(g2.ecount(), 3);
        assert!(g2.is_directed());

        for eid in 0..3u32 {
            let (s1, t1) = g.edge(eid).unwrap();
            let (s2, t2) = g2.edge(eid).unwrap();
            assert_eq!(s1, s2);
            assert_eq!(t1, t2);
        }
    }

    #[test]
    fn test_write_empty() {
        let g = Graph::with_vertices(0);
        let mut buf = Vec::new();
        write_gml(&g, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("graph"));
        assert!(s.contains("directed 0"));
    }

    #[test]
    fn test_string_with_entities() {
        let gml = b"graph [ node [ id 0 label \"a&amp;b\" ] ]";
        let g = read_gml(&gml[..]).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(
            g.vertex_attribute("label", 0).and_then(|v| v.as_str()),
            Some("a&b"),
        );
    }

    #[test]
    fn test_top_level_creator_as_graph_attr() {
        let gml = b"Creator \"Someone\"\nVersion 1\ngraph [ node [ id 0 ] ]";
        let g = read_gml(&gml[..]).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(
            g.graph_attribute("Creator").and_then(|v| v.as_str()),
            Some("Someone"),
        );
    }

    #[test]
    fn test_negative_node_id() {
        let gml = b"graph [ node [ id -5 ] node [ id -3 ] edge [ source -5 target -3 ] ]";
        let g = read_gml(&gml[..]).unwrap();
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 1);
    }

    #[test]
    fn test_lesmis_style_header() {
        let gml = b"Creator \"Mark Newman on Fri Jul 21 12:44:53 2006\"\ngraph\n[\n  node\n  [\n    id 0\n    label \"Myriel\"\n  ]\n  node\n  [\n    id 1\n    label \"Napoleon\"\n  ]\n  edge\n  [\n    source 0\n    target 1\n    value 1\n  ]\n]\n";
        let g = read_gml(&gml[..]).unwrap();
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 1);
        assert_eq!(
            g.vertex_attribute("label", 0).and_then(|v| v.as_str()),
            Some("Myriel"),
        );
        assert_eq!(
            g.vertex_attribute("label", 1).and_then(|v| v.as_str()),
            Some("Napoleon"),
        );
        let v = g
            .edge_attribute("value", 0)
            .and_then(AttributeValue::as_f64);
        assert!((v.unwrap() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn roundtrip_with_attributes() {
        let mut g = Graph::from_edges(&[(0, 1), (1, 2)], false, None).unwrap();
        g.set_vertex_attribute("label", 0, "Alice".into()).unwrap();
        g.set_vertex_attribute("label", 1, "Bob".into()).unwrap();
        g.set_vertex_attribute("label", 2, "Carol".into()).unwrap();
        g.set_edge_attribute("weight", 0, 1.5.into()).unwrap();
        g.set_edge_attribute("weight", 1, 2.5.into()).unwrap();
        g.set_graph_attribute("name", "test_graph".into());

        let mut buf = Vec::new();
        write_gml(&g, &mut buf).unwrap();

        let g2 = read_gml(&buf[..]).unwrap();
        assert_eq!(g2.vcount(), 3);
        assert_eq!(g2.ecount(), 2);
        assert_eq!(
            g2.vertex_attribute("label", 0).and_then(|v| v.as_str()),
            Some("Alice"),
        );
        assert_eq!(
            g2.vertex_attribute("label", 2).and_then(|v| v.as_str()),
            Some("Carol"),
        );
        let w = g2
            .edge_attribute("weight", 0)
            .and_then(AttributeValue::as_f64);
        assert!((w.unwrap() - 1.5).abs() < f64::EPSILON);
        assert_eq!(
            g2.graph_attribute("name").and_then(|v| v.as_str()),
            Some("test_graph"),
        );
    }

    #[test]
    fn write_boolean_attribute() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.set_vertex_attribute("active", 0, true.into()).unwrap();
        g.set_vertex_attribute("active", 1, false.into()).unwrap();

        let mut buf = Vec::new();
        write_gml(&g, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("active 1"));
        assert!(s.contains("active 0"));
    }
}

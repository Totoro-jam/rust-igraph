//! GML (Graph Modelling Language) I/O (ALGO-IO-001).
//!
//! Reads and writes graphs in GML format. GML is a hierarchical key-value
//! format widely used in network science tools.
//!
//! Reference: <https://web.archive.org/web/20190207140002/http://www.fim.uni-passau.de/index.php?id=17297&L=1>
//!
//! We implement the structural subset: `graph [ directed 0/1  node [ id N ]
//! edge [ source N target N ] ]`. Non-structural attributes (labels, weights,
//! coordinates) are silently skipped during reading and not emitted during
//! writing. This matches the igraph C `igraph_read_graph_gml` behaviour for
//! graphs without an attribute handler installed.

use std::io::{BufRead, BufReader, Read, Write};

use crate::core::{Graph, IgraphError, IgraphResult};

/// Read a graph from GML format.
///
/// Parses the first `graph [ ... ]` block. Extracts `directed` (0 or 1,
/// default 0), `node [ id N ]` entries, and `edge [ source N target N ]`
/// entries. All other keys/values are skipped.
///
/// Node ids need not be contiguous or start at 0 — the reader maps them
/// to internal vertex indices in the order they appear.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, read_gml};
///
/// let gml = b"graph [\n  directed 0\n  node [ id 1 ]\n  node [ id 2 ]\n  node [ id 3 ]\n  edge [ source 1 target 2 ]\n  edge [ source 2 target 3 ]\n]";
/// let g = read_gml(&gml[..]).unwrap();
/// assert_eq!(g.vcount(), 3);
/// assert_eq!(g.ecount(), 2);
/// assert!(!g.is_directed());
/// ```
pub fn read_gml<R: Read>(input: R) -> IgraphResult<Graph> {
    let reader = BufReader::new(input);
    let tokens = tokenize(reader)?;
    parse_graph(&tokens)
}

/// Write a graph in GML format.
///
/// Produces a valid GML file with `graph [ directed 0/1 node [ id N ] ...
/// edge [ source N target N ] ... ]`. Node ids are the internal vertex
/// indices (0-based).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, write_gml, read_gml};
///
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
///
/// let mut buf = Vec::new();
/// write_gml(&g, &mut buf).unwrap();
/// let s = String::from_utf8(buf).unwrap();
/// assert!(s.contains("directed 1"));
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

    for v in 0..graph.vcount() {
        writeln!(writer, "  node")?;
        writeln!(writer, "  [")?;
        writeln!(writer, "    id {v}")?;
        writeln!(writer, "  ]")?;
    }

    for eid in 0..graph.ecount() {
        #[allow(clippy::cast_possible_truncation)]
        let (src, tgt) = graph.edge(eid as u32)?;
        writeln!(writer, "  edge")?;
        writeln!(writer, "  [")?;
        writeln!(writer, "    source {src}")?;
        writeln!(writer, "    target {tgt}")?;
        writeln!(writer, "  ]")?;
    }

    writeln!(writer, "]")?;
    Ok(())
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

        // Skip comment lines (igraph C treats "comment" as a key, but
        // lines starting with # are sometimes used informally)
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
                    chars.next(); // consume opening quote
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
                    // Decode basic XML entities
                    let decoded = decode_entities(&s);
                    tokens.push(Token::Str(decoded));
                }
                '#' => break, // rest of line is comment
                _ => {
                    // Collect a word (key or number)
                    let mut word = String::new();
                    while let Some(&c) = chars.peek() {
                        if c == ' ' || c == '\t' || c == '[' || c == ']' || c == '"' {
                            break;
                        }
                        word.push(c);
                        chars.next();
                    }

                    if word.is_empty() {
                        return Err(IgraphError::Parse {
                            line: line_no,
                            message: format!("unexpected character '{ch}'"),
                        });
                    }

                    // Try integer first, then float, otherwise it's a key
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

fn parse_graph(tokens: &[Token]) -> IgraphResult<Graph> {
    // Find "graph" "[" ... "]"
    let mut pos = 0;

    // Skip top-level keys until we find "graph"
    while pos < tokens.len() {
        match &tokens[pos] {
            Token::Key(k) if k == "graph" => {
                pos += 1;
                break;
            }
            Token::Key(_) => {
                pos += 1;
                // Skip the value (could be a number, string, or nested block)
                pos = skip_value(tokens, pos);
            }
            _ => {
                pos += 1;
            }
        }
    }

    // Expect "["
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
    let mut node_ids: Vec<i64> = Vec::new();
    let mut edges: Vec<(i64, i64)> = Vec::new();

    // Parse inside graph block
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
                        let (node_id, new_pos) = parse_node(tokens, pos)?;
                        node_ids.push(node_id);
                        pos = new_pos;
                    }
                    "edge" if pos < tokens.len() && tokens[pos] == Token::Open => {
                        pos += 1;
                        let (edge, new_pos) = parse_edge(tokens, pos)?;
                        edges.push(edge);
                        pos = new_pos;
                    }
                    _ => {
                        pos = skip_value(tokens, pos);
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

    // Skip closing "]"
    if pos < tokens.len() && tokens[pos] == Token::Close {
        // consumed
    }

    // Build the graph
    build_graph(directed, &node_ids, &edges)
}

fn parse_node(tokens: &[Token], mut pos: usize) -> IgraphResult<(i64, usize)> {
    let mut node_id: Option<i64> = None;

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

    // Skip closing "]"
    if pos < tokens.len() && tokens[pos] == Token::Close {
        pos += 1;
    }

    let id = node_id.ok_or_else(|| IgraphError::Parse {
        line: 0,
        message: "node without 'id' field".into(),
    })?;

    Ok((id, pos))
}

fn parse_edge(tokens: &[Token], mut pos: usize) -> IgraphResult<((i64, i64), usize)> {
    let mut source: Option<i64> = None;
    let mut target: Option<i64> = None;

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
                        pos = skip_value(tokens, pos);
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

    // Skip closing "]"
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

    Ok(((src, tgt), pos))
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

fn build_graph(directed: bool, node_ids: &[i64], edges: &[(i64, i64)]) -> IgraphResult<Graph> {
    use std::collections::HashMap;

    // Map external GML ids to internal 0-based indices
    let mut id_map: HashMap<i64, u32> = HashMap::with_capacity(node_ids.len());
    for (idx, &gml_id) in node_ids.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        let internal_id = idx as u32;
        if id_map.insert(gml_id, internal_id).is_some() {
            return Err(IgraphError::Parse {
                line: 0,
                message: format!("duplicate node id {gml_id}"),
            });
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    let n = node_ids.len() as u32;
    let mut graph = Graph::new(n, directed)?;

    let mut edge_list: Vec<(u32, u32)> = Vec::with_capacity(edges.len());
    for &(src_gml, tgt_gml) in edges {
        let src = id_map.get(&src_gml).ok_or_else(|| IgraphError::Parse {
            line: 0,
            message: format!("edge references unknown node id {src_gml}"),
        })?;
        let tgt = id_map.get(&tgt_gml).ok_or_else(|| IgraphError::Parse {
            line: 0,
            message: format!("edge references unknown node id {tgt_gml}"),
        })?;
        edge_list.push((*src, *tgt));
    }

    graph.add_edges(edge_list)?;
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
        // node 10 -> internal 0, node 30 -> internal 2
        let (src, tgt) = g.edge(0).unwrap();
        assert_eq!(src, 0);
        assert_eq!(tgt, 2);
    }

    #[test]
    fn test_multiline() {
        let gml = b"Creator \"test\"\ngraph\n[\n  directed 0\n  node\n  [\n    id 0\n    label \"A\"\n  ]\n  node\n  [\n    id 1\n    label \"B\"\n  ]\n  edge\n  [\n    source 0\n    target 1\n    weight 1.5\n  ]\n]\n";
        let g = read_gml(&gml[..]).unwrap();
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 1);
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
    }

    #[test]
    fn test_top_level_creator_ignored() {
        let gml = b"Creator \"Someone\"\nVersion 1\ngraph [ node [ id 0 ] ]";
        let g = read_gml(&gml[..]).unwrap();
        assert_eq!(g.vcount(), 1);
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
    }
}

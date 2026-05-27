//! Plain edge-list I/O.
//!
//! Format: one edge per line, `"u v"` (whitespace-separated `u32` vertex ids).
//! Lines starting with `#` and blank lines are ignored. The graph is sized to
//! `max(u, v) + 1` vertices.
//!
//! Counterparts of `igraph_read_graph_edgelist` / `igraph_write_graph_edgelist`.

use std::io::{BufRead, BufReader, Read, Write};

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Read an edge list from any [`Read`] into a fresh [`Graph`].
pub fn read_edgelist<R: Read>(input: R) -> IgraphResult<Graph> {
    let reader = BufReader::new(input);
    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();
    let mut max_id: VertexId = 0;
    let mut had_any = false;

    for (line_no, line) in reader.lines().enumerate() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let u = parse_id(parts.next(), line_no + 1)?;
        let v = parse_id(parts.next(), line_no + 1)?;
        if parts.next().is_some() {
            return Err(IgraphError::Parse {
                line: line_no + 1,
                message: "expected exactly two vertex ids".into(),
            });
        }
        max_id = max_id.max(u).max(v);
        edges.push((u, v));
        had_any = true;
    }

    let n = if had_any {
        max_id.checked_add(1).ok_or(IgraphError::Internal(
            "vertex id u32::MAX cannot be sized into n",
        ))?
    } else {
        0
    };
    let mut g = Graph::with_vertices(n);
    g.add_edges(edges)?;
    Ok(g)
}

fn parse_id(token: Option<&str>, line: usize) -> IgraphResult<VertexId> {
    let s = token.ok_or_else(|| IgraphError::Parse {
        line,
        message: "missing vertex id".into(),
    })?;
    s.parse::<VertexId>().map_err(|e| IgraphError::Parse {
        line,
        message: format!("invalid vertex id `{s}`: {e}"),
    })
}

/// Write a graph as an edge list.
///
/// Outputs one line per edge: `"source target\n"` using 0-based vertex
/// IDs. Isolated vertices are not represented in the output.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, write_edgelist};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
///
/// let mut buf = Vec::new();
/// write_edgelist(&g, &mut buf).unwrap();
/// let s = String::from_utf8(buf).unwrap();
/// assert_eq!(s, "0 1\n1 2\n");
/// ```
pub fn write_edgelist<W: Write>(graph: &Graph, writer: &mut W) -> IgraphResult<()> {
    for eid in 0..graph.ecount() {
        #[allow(clippy::cast_possible_truncation)]
        let (from, to) = graph.edge(eid as u32)?;
        writeln!(writer, "{from} {to}")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_gives_empty_graph() {
        let g = read_edgelist(&b""[..]).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn comments_and_blanks_ignored() {
        let input = b"# hi\n\n0 1\n   2 3   \n";
        let g = read_edgelist(&input[..]).unwrap();
        assert_eq!(g.vcount(), 4);
        assert_eq!(g.ecount(), 2);
    }

    #[test]
    fn malformed_line_errors_with_line_number() {
        let input = b"0 1\nbroken\n";
        let err = read_edgelist(&input[..]).unwrap_err();
        match err {
            IgraphError::Parse { line, .. } => assert_eq!(line, 2),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn write_basic() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let mut buf = Vec::new();
        write_edgelist(&g, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert_eq!(s, "0 1\n1 2\n");
    }

    #[test]
    fn write_empty_graph() {
        let g = Graph::with_vertices(5);
        let mut buf = Vec::new();
        write_edgelist(&g, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert_eq!(s, "");
    }

    #[test]
    fn write_directed() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 0).unwrap();

        let mut buf = Vec::new();
        write_edgelist(&g, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert_eq!(s, "0 1\n2 0\n");
    }

    #[test]
    fn round_trip() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(1, 3).unwrap();

        let mut buf = Vec::new();
        write_edgelist(&g, &mut buf).unwrap();

        let g2 = read_edgelist(&buf[..]).unwrap();
        assert_eq!(g2.vcount(), g.vcount());
        assert_eq!(g2.ecount(), g.ecount());
    }
}

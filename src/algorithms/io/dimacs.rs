//! DIMACS flow-problem I/O (ALGO-IO-004).
//!
//! Reads and writes graphs in the DIMACS network flow format. This is a
//! line-oriented ASCII format with the following line types:
//!
//! - `c ...` — comment (ignored)
//! - `p max|edge N M` — problem line: type, vertex count, edge count
//! - `n ID s|t` — (max problems) designate source/target vertex
//! - `n ID LABEL` — (edge problems) assign integer label to vertex
//! - `a FROM TO CAP` — (max problems) arc with capacity
//! - `e FROM TO` — (edge problems) undirected edge
//!
//! Vertex IDs in the file are 1-based; internally they are 0-based.
//!
//! Counterpart of `igraph_read_graph_dimacs_flow` / `igraph_write_graph_dimacs_flow`.

use std::io::{BufRead, BufReader, Read, Write};

use crate::core::{Graph, IgraphError, IgraphResult};

/// The type of DIMACS problem read from the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DimacsProblem {
    /// Maximum-flow problem (`p max N M`).
    Max,
    /// Edge-coloring/clique problem (`p edge N M`).
    Edge,
}

/// Result of reading a DIMACS file.
#[derive(Debug, Clone)]
pub struct DimacsGraph {
    /// The parsed graph.
    pub graph: Graph,
    /// Problem type.
    pub problem: DimacsProblem,
    /// Source vertex (0-based). Only meaningful for `Max` problems.
    pub source: Option<u32>,
    /// Target vertex (0-based). Only meaningful for `Max` problems.
    pub target: Option<u32>,
    /// Edge capacities (one per edge, in edge order). Only present for
    /// `Max` problems.
    pub capacity: Option<Vec<f64>>,
    /// Vertex labels (indexed by vertex id). Only present for `Edge`
    /// problems when at least one `n` line appears.
    pub labels: Option<Vec<i64>>,
}

/// Read a graph from DIMACS flow/edge format.
///
/// # Examples
///
/// ```
/// use rust_igraph::read_dimacs;
///
/// let dimacs = b"c example\np max 4 5\nn 1 s\nn 4 t\na 1 2 10\na 1 3 8\na 2 3 5\na 2 4 7\na 3 4 10\n";
/// let result = read_dimacs(&dimacs[..], true).unwrap();
/// assert_eq!(result.graph.vcount(), 4);
/// assert_eq!(result.graph.ecount(), 5);
/// assert_eq!(result.source, Some(0));
/// assert_eq!(result.target, Some(3));
/// ```
#[allow(clippy::too_many_lines)]
pub fn read_dimacs<R: Read>(input: R, directed: bool) -> IgraphResult<DimacsGraph> {
    let reader = BufReader::new(input);

    let mut problem_type: Option<DimacsProblem> = None;
    let mut n_vertices: Option<u32> = None;
    let mut source: Option<u32> = None;
    let mut target: Option<u32> = None;
    let mut edges: Vec<(u32, u32)> = Vec::new();
    let mut capacities: Vec<f64> = Vec::new();
    let mut labels: Option<Vec<i64>> = None;

    for (line_idx, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        let first_char = trimmed.as_bytes()[0];

        match first_char {
            b'c' => {
                // Comment line — skip
            }
            b'p' => {
                if problem_type.is_some() {
                    return Err(IgraphError::Parse {
                        line: line_idx.wrapping_add(1),
                        message: "duplicate problem line".into(),
                    });
                }
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() < 4 {
                    return Err(IgraphError::Parse {
                        line: line_idx.wrapping_add(1),
                        message: "problem line needs: p type nodes edges".into(),
                    });
                }
                let parts = parts.as_slice();
                let ptype = match parts[1] {
                    "max" => DimacsProblem::Max,
                    "edge" | "col" => DimacsProblem::Edge,
                    other => {
                        return Err(IgraphError::Parse {
                            line: line_idx.wrapping_add(1),
                            message: format!(
                                "unknown problem type '{other}', expected 'max' or 'edge'"
                            ),
                        });
                    }
                };
                let n: u32 = parts[2].parse().map_err(|e| IgraphError::Parse {
                    line: line_idx.wrapping_add(1),
                    message: format!("invalid vertex count: {e}"),
                })?;
                let _m: u32 = parts[3].parse().map_err(|e| IgraphError::Parse {
                    line: line_idx.wrapping_add(1),
                    message: format!("invalid edge count: {e}"),
                })?;

                problem_type = Some(ptype);
                n_vertices = Some(n);

                if ptype == DimacsProblem::Edge {
                    // Initialize labels: default = vertex_id + 1 (1-based)
                    let mut lbl = Vec::with_capacity(n as usize);
                    for i in 0..n {
                        lbl.push(i64::from(i) + 1);
                    }
                    labels = Some(lbl);
                }
            }
            b'n' => {
                let ptype = problem_type.ok_or_else(|| IgraphError::Parse {
                    line: line_idx.wrapping_add(1),
                    message: "'n' line before problem line".into(),
                })?;
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() < 3 {
                    return Err(IgraphError::Parse {
                        line: line_idx.wrapping_add(1),
                        message: "node line needs: n ID type_or_label".into(),
                    });
                }
                let parts = parts.as_slice();
                let vid: u32 = parts[1].parse().map_err(|e| IgraphError::Parse {
                    line: line_idx.wrapping_add(1),
                    message: format!("invalid vertex id: {e}"),
                })?;
                if vid == 0 {
                    return Err(IgraphError::Parse {
                        line: line_idx.wrapping_add(1),
                        message: "vertex IDs are 1-based in DIMACS".into(),
                    });
                }
                let vid0 = vid.wrapping_sub(1); // 0-based

                match ptype {
                    DimacsProblem::Max => match parts[2] {
                        "s" => {
                            source = Some(vid0);
                        }
                        "t" => {
                            target = Some(vid0);
                        }
                        other => {
                            return Err(IgraphError::Parse {
                                line: line_idx.wrapping_add(1),
                                message: format!(
                                    "invalid node type '{other}', expected 's' or 't'"
                                ),
                            });
                        }
                    },
                    DimacsProblem::Edge => {
                        let lbl: i64 = parts[2].parse().map_err(|e| IgraphError::Parse {
                            line: line_idx.wrapping_add(1),
                            message: format!("invalid label: {e}"),
                        })?;
                        if let Some(ref mut lab_vec) = labels {
                            if (vid0 as usize) < lab_vec.len() {
                                lab_vec[vid0 as usize] = lbl;
                            }
                        }
                    }
                }
            }
            b'a' => {
                if problem_type != Some(DimacsProblem::Max) {
                    return Err(IgraphError::Parse {
                        line: line_idx.wrapping_add(1),
                        message: "'a' lines only allowed in max problems".into(),
                    });
                }
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() < 4 {
                    return Err(IgraphError::Parse {
                        line: line_idx.wrapping_add(1),
                        message: "arc line needs: a FROM TO CAPACITY".into(),
                    });
                }
                let from: u32 = parts[1].parse().map_err(|e| IgraphError::Parse {
                    line: line_idx.wrapping_add(1),
                    message: format!("invalid source id: {e}"),
                })?;
                let to: u32 = parts[2].parse().map_err(|e| IgraphError::Parse {
                    line: line_idx.wrapping_add(1),
                    message: format!("invalid target id: {e}"),
                })?;
                let cap: f64 = parts[3].parse().map_err(|e| IgraphError::Parse {
                    line: line_idx.wrapping_add(1),
                    message: format!("invalid capacity: {e}"),
                })?;

                if from == 0 || to == 0 {
                    return Err(IgraphError::Parse {
                        line: line_idx.wrapping_add(1),
                        message: "vertex IDs are 1-based in DIMACS".into(),
                    });
                }
                edges.push((from.wrapping_sub(1), to.wrapping_sub(1)));
                capacities.push(cap);
            }
            b'e' => {
                if problem_type != Some(DimacsProblem::Edge) {
                    return Err(IgraphError::Parse {
                        line: line_idx.wrapping_add(1),
                        message: "'e' lines only allowed in edge problems".into(),
                    });
                }
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() < 3 {
                    return Err(IgraphError::Parse {
                        line: line_idx.wrapping_add(1),
                        message: "edge line needs: e FROM TO".into(),
                    });
                }
                let from: u32 = parts[1].parse().map_err(|e| IgraphError::Parse {
                    line: line_idx.wrapping_add(1),
                    message: format!("invalid source id: {e}"),
                })?;
                let to: u32 = parts[2].parse().map_err(|e| IgraphError::Parse {
                    line: line_idx.wrapping_add(1),
                    message: format!("invalid target id: {e}"),
                })?;

                if from == 0 || to == 0 {
                    return Err(IgraphError::Parse {
                        line: line_idx.wrapping_add(1),
                        message: "vertex IDs are 1-based in DIMACS".into(),
                    });
                }
                edges.push((from.wrapping_sub(1), to.wrapping_sub(1)));
            }
            _ => {
                return Err(IgraphError::Parse {
                    line: line_idx.wrapping_add(1),
                    message: format!(
                        "unknown line type '{}'",
                        trimmed.chars().next().unwrap_or('?')
                    ),
                });
            }
        }
    }

    let n = n_vertices.ok_or_else(|| IgraphError::Parse {
        line: 0,
        message: "no problem line found in DIMACS file".into(),
    })?;

    let ptype = problem_type.unwrap_or(DimacsProblem::Edge);

    let mut graph = Graph::new(n, directed)?;
    graph.add_edges(edges)?;

    Ok(DimacsGraph {
        graph,
        problem: ptype,
        source,
        target,
        capacity: if ptype == DimacsProblem::Max {
            Some(capacities)
        } else {
            None
        },
        labels: if ptype == DimacsProblem::Edge {
            labels
        } else {
            None
        },
    })
}

/// Write a graph in DIMACS max-flow format.
///
/// Writes the problem line, source/target node lines, and arc lines
/// with capacity values.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, write_dimacs_flow};
///
/// let mut g = Graph::new(4, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
///
/// let capacity = vec![10.0, 5.0, 8.0];
/// let mut buf = Vec::new();
/// write_dimacs_flow(&g, 0, 3, &capacity, &mut buf).unwrap();
/// let s = String::from_utf8(buf).unwrap();
/// assert!(s.contains("p max 4 3"));
/// assert!(s.contains("n 1 s"));
/// assert!(s.contains("n 4 t"));
/// ```
pub fn write_dimacs_flow<W: Write>(
    graph: &Graph,
    source: u32,
    target: u32,
    capacity: &[f64],
    writer: &mut W,
) -> IgraphResult<()> {
    if capacity.len() != graph.ecount() {
        return Err(IgraphError::InvalidArgument(format!(
            "capacity length {} does not match ecount {}",
            capacity.len(),
            graph.ecount()
        )));
    }
    if source >= graph.vcount() {
        return Err(IgraphError::InvalidArgument(format!(
            "source {} out of range (vcount = {})",
            source,
            graph.vcount()
        )));
    }
    if target >= graph.vcount() {
        return Err(IgraphError::InvalidArgument(format!(
            "target {} out of range (vcount = {})",
            target,
            graph.vcount()
        )));
    }

    writeln!(writer, "c created by rust-igraph")?;
    writeln!(writer, "p max {} {}", graph.vcount(), graph.ecount())?;
    writeln!(writer, "n {} s", source + 1)?;
    writeln!(writer, "n {} t", target + 1)?;

    for (eid, &cap) in capacity.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        let (from, to) = graph.edge(eid as u32)?;
        writeln!(writer, "a {} {} {cap}", from + 1, to + 1)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_file_error() {
        let result = read_dimacs(&b""[..], true);
        assert!(result.is_err());
    }

    #[test]
    fn test_max_problem_basic() {
        let input = b"c comment\np max 4 3\nn 1 s\nn 4 t\na 1 2 10\na 2 3 5\na 3 4 8\n";
        let result = read_dimacs(&input[..], true).unwrap();
        assert_eq!(result.graph.vcount(), 4);
        assert_eq!(result.graph.ecount(), 3);
        assert_eq!(result.problem, DimacsProblem::Max);
        assert_eq!(result.source, Some(0));
        assert_eq!(result.target, Some(3));
        let cap = result.capacity.unwrap();
        assert!((cap[0] - 10.0).abs() < 1e-10);
        assert!((cap[1] - 5.0).abs() < 1e-10);
        assert!((cap[2] - 8.0).abs() < 1e-10);
    }

    #[test]
    fn test_edge_problem_basic() {
        let input = b"p edge 5 4\ne 1 2\ne 2 3\ne 3 4\ne 4 5\n";
        let result = read_dimacs(&input[..], false).unwrap();
        assert_eq!(result.graph.vcount(), 5);
        assert_eq!(result.graph.ecount(), 4);
        assert_eq!(result.problem, DimacsProblem::Edge);
        assert!(result.capacity.is_none());
        assert!(!result.graph.is_directed());
    }

    #[test]
    fn test_edge_problem_with_labels() {
        let input = b"p edge 3 2\nn 1 100\nn 3 300\ne 1 2\ne 2 3\n";
        let result = read_dimacs(&input[..], false).unwrap();
        let lab = result.labels.unwrap();
        assert_eq!(lab[0], 100);
        assert_eq!(lab[1], 2); // default = 1-based id
        assert_eq!(lab[2], 300);
    }

    #[test]
    fn test_comments_ignored() {
        let input = b"c first comment\nc second comment\np max 2 1\nn 1 s\nn 2 t\na 1 2 5\n";
        let result = read_dimacs(&input[..], true).unwrap();
        assert_eq!(result.graph.vcount(), 2);
        assert_eq!(result.graph.ecount(), 1);
    }

    #[test]
    fn test_directed_graph() {
        let input = b"p max 3 2\nn 1 s\nn 3 t\na 1 2 10\na 2 3 20\n";
        let result = read_dimacs(&input[..], true).unwrap();
        assert!(result.graph.is_directed());
    }

    #[test]
    fn test_zero_capacity() {
        let input = b"p max 2 1\nn 1 s\nn 2 t\na 1 2 0\n";
        let result = read_dimacs(&input[..], true).unwrap();
        let cap = result.capacity.unwrap();
        assert!((cap[0] - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_float_capacity() {
        let input = b"p max 2 1\nn 1 s\nn 2 t\na 1 2 7.25\n";
        let result = read_dimacs(&input[..], true).unwrap();
        let cap = result.capacity.unwrap();
        assert!((cap[0] - 7.25).abs() < 1e-10);
    }

    #[test]
    fn test_duplicate_problem_line_error() {
        let input = b"p max 2 1\np max 3 2\n";
        let result = read_dimacs(&input[..], true);
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_problem_type_error() {
        let input = b"p foo 2 1\n";
        let result = read_dimacs(&input[..], true);
        assert!(result.is_err());
    }

    #[test]
    fn test_arc_in_edge_problem_error() {
        let input = b"p edge 2 1\na 1 2 5\n";
        let result = read_dimacs(&input[..], false);
        assert!(result.is_err());
    }

    #[test]
    fn test_edge_in_max_problem_error() {
        let input = b"p max 2 1\ne 1 2\n";
        let result = read_dimacs(&input[..], true);
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_vertex_id_error() {
        let input = b"p max 2 1\na 0 1 5\n";
        let result = read_dimacs(&input[..], true);
        assert!(result.is_err());
    }

    #[test]
    fn test_col_alias_for_edge() {
        let input = b"p col 3 2\ne 1 2\ne 2 3\n";
        let result = read_dimacs(&input[..], false).unwrap();
        assert_eq!(result.problem, DimacsProblem::Edge);
        assert_eq!(result.graph.ecount(), 2);
    }

    #[test]
    fn test_write_max_flow() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let capacity = vec![10.0, 20.0];
        let mut buf = Vec::new();
        write_dimacs_flow(&g, 0, 2, &capacity, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("p max 3 2"));
        assert!(s.contains("n 1 s"));
        assert!(s.contains("n 3 t"));
        assert!(s.contains("a 1 2 10"));
        assert!(s.contains("a 2 3 20"));
    }

    #[test]
    fn test_write_capacity_mismatch_error() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        let capacity = vec![1.0, 2.0]; // 2 capacities but 1 edge
        let mut buf = Vec::new();
        assert!(write_dimacs_flow(&g, 0, 1, &capacity, &mut buf).is_err());
    }

    #[test]
    fn test_write_source_out_of_range_error() {
        let g = Graph::new(2, true).unwrap();
        let mut buf = Vec::new();
        assert!(write_dimacs_flow(&g, 5, 1, &[], &mut buf).is_err());
    }

    #[test]
    fn test_round_trip() {
        let input = b"p max 4 4\nn 1 s\nn 4 t\na 1 2 10\na 1 3 8\na 2 4 7\na 3 4 9\n";
        let result = read_dimacs(&input[..], true).unwrap();

        let mut buf = Vec::new();
        write_dimacs_flow(
            &result.graph,
            result.source.unwrap(),
            result.target.unwrap(),
            result.capacity.as_deref().unwrap(),
            &mut buf,
        )
        .unwrap();

        let result2 = read_dimacs(&buf[..], true).unwrap();
        assert_eq!(result2.graph.vcount(), result.graph.vcount());
        assert_eq!(result2.graph.ecount(), result.graph.ecount());
        assert_eq!(result2.source, result.source);
        assert_eq!(result2.target, result.target);
    }

    #[test]
    fn test_blank_lines_skipped() {
        let input = b"\n\np max 2 1\n\nn 1 s\nn 2 t\n\na 1 2 5\n\n";
        let result = read_dimacs(&input[..], true).unwrap();
        assert_eq!(result.graph.ecount(), 1);
    }
}

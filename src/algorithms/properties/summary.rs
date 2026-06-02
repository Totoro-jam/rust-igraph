//! Graph summary — a quick one-call overview of structural properties.

use std::fmt::Write as FmtWrite;

use crate::algorithms::connectivity::components::connected_components;
use crate::algorithms::connectivity::is_connected::{ConnectednessMode, is_connected};
use crate::algorithms::properties::basic::density;
use crate::algorithms::properties::degree::{DegreeMode, max_degree, min_degree};
use crate::algorithms::properties::multiplicity::{has_loop, has_multiple};
use crate::core::{Graph, IgraphResult};

/// Result of [`graph_summary`] containing precomputed graph statistics.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct GraphSummary {
    /// Whether the graph is directed.
    pub directed: bool,
    /// Number of vertices.
    pub vcount: u32,
    /// Number of edges.
    pub ecount: usize,
    /// Whether the graph is connected (weakly, for directed graphs).
    pub connected: bool,
    /// Number of (weakly) connected components.
    pub component_count: u32,
    /// Edge density, or `None` for trivial graphs (0 or 1 vertex).
    pub density: Option<f64>,
    /// Minimum degree across all vertices.
    pub min_degree: u32,
    /// Maximum degree across all vertices.
    pub max_degree: u32,
    /// Whether the graph has self-loops.
    pub has_loops: bool,
    /// Whether the graph has multi-edges (parallel edges).
    pub has_multiple: bool,
}

impl std::fmt::Display for GraphSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = if self.directed {
            "Directed"
        } else {
            "Undirected"
        };
        write!(
            f,
            "{kind} graph: {v} vertices, {e} edges",
            v = self.vcount,
            e = self.ecount
        )?;

        if self.vcount > 0 {
            let conn = if self.connected {
                "connected"
            } else {
                "disconnected"
            };
            write!(f, ", {conn}")?;
            if !self.connected {
                write!(f, " ({} components)", self.component_count)?;
            }
        }

        if let Some(d) = self.density {
            write!(f, ", density={d:.4}")?;
        }

        if self.vcount > 0 {
            write!(f, ", degree=[{}, {}]", self.min_degree, self.max_degree)?;
        }

        let mut flags = Vec::new();
        if self.has_loops {
            flags.push("loops");
        }
        if self.has_multiple {
            flags.push("multi");
        }
        if !flags.is_empty() {
            write!(f, " [{}]", flags.join(", "))?;
        }

        Ok(())
    }
}

/// Compute a quick structural summary of a graph.
///
/// Returns a [`GraphSummary`] struct with key properties pre-computed:
/// vertex/edge counts, connectivity, density, degree range, and whether
/// the graph has self-loops or multi-edges. The struct implements
/// [`Display`](std::fmt::Display) for pretty-printing.
///
/// This is the Rust counterpart of python-igraph's `Graph.summary()`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, graph_summary};
///
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
///
/// let s = graph_summary(&g).unwrap();
/// assert_eq!(s.vcount, 5);
/// assert_eq!(s.ecount, 4);
/// assert!(s.connected);
/// assert_eq!(s.component_count, 1);
/// assert_eq!(s.min_degree, 1);
/// assert_eq!(s.max_degree, 2);
/// assert!(!s.has_loops);
/// assert!(!s.has_multiple);
/// println!("{s}");
/// ```
///
/// ```
/// use rust_igraph::{Graph, graph_summary};
///
/// let g = Graph::with_vertices(0);
/// let s = graph_summary(&g).unwrap();
/// assert_eq!(s.vcount, 0);
/// assert_eq!(s.ecount, 0);
/// assert!(s.connected); // vacuously
/// ```
pub fn graph_summary(graph: &Graph) -> IgraphResult<GraphSummary> {
    let vcount = graph.vcount();
    let ecount = graph.ecount();
    let directed = graph.is_directed();

    let (connected, component_count) = if vcount == 0 {
        (true, 0)
    } else {
        let conn = is_connected(graph, ConnectednessMode::Weak)?;
        let cc = connected_components(graph)?;
        (conn, cc.count)
    };

    let dens = density(graph)?;

    let (min_deg, max_deg) = if vcount == 0 {
        (0, 0)
    } else {
        let mn = min_degree(graph, DegreeMode::All)?;
        let mx = max_degree(graph, DegreeMode::All)?;
        (mn, mx)
    };

    let loops = has_loop(graph)?;
    let multi = has_multiple(graph)?;

    Ok(GraphSummary {
        directed,
        vcount,
        ecount,
        connected,
        component_count,
        density: dens,
        min_degree: min_deg,
        max_degree: max_deg,
        has_loops: loops,
        has_multiple: multi,
    })
}

/// Format a detailed multi-line summary of a graph.
///
/// Similar to [`graph_summary`] but returns a multi-line string with
/// additional detail, formatted for terminal/log output.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, graph_summary_string};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 0).unwrap();
///
/// let s = graph_summary_string(&g).unwrap();
/// assert!(s.contains("Vertices: 4"));
/// assert!(s.contains("Edges: 4"));
/// assert!(s.contains("Connected: yes"));
/// ```
pub fn graph_summary_string(graph: &Graph) -> IgraphResult<String> {
    let s = graph_summary(graph)?;
    let mut out = String::new();

    let kind = if s.directed { "Directed" } else { "Undirected" };
    let _ = writeln!(out, "--- Graph Summary ---");
    let _ = writeln!(out, "Type: {kind}");
    let _ = writeln!(out, "Vertices: {}", s.vcount);
    let _ = writeln!(out, "Edges: {}", s.ecount);

    let conn_str = if s.connected { "yes" } else { "no" };
    let _ = writeln!(out, "Connected: {conn_str}");
    if !s.connected {
        let _ = writeln!(out, "Components: {}", s.component_count);
    }

    if let Some(d) = s.density {
        let _ = writeln!(out, "Density: {d:.6}");
    }

    if s.vcount > 0 {
        let _ = writeln!(out, "Degree range: [{}, {}]", s.min_degree, s.max_degree);
    }

    let simple = !s.has_loops && !s.has_multiple;
    let _ = writeln!(out, "Simple: {}", if simple { "yes" } else { "no" });
    if s.has_loops {
        let _ = writeln!(out, "  Self-loops: yes");
    }
    if s.has_multiple {
        let _ = writeln!(out, "  Multi-edges: yes");
    }

    let _ = write!(out, "--------------------");
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_empty_graph() {
        let g = Graph::with_vertices(0);
        let s = graph_summary(&g).unwrap();
        assert_eq!(s.vcount, 0);
        assert_eq!(s.ecount, 0);
        assert!(s.connected);
        assert_eq!(s.component_count, 0);
        assert_eq!(s.density, None);
        assert_eq!(s.min_degree, 0);
        assert_eq!(s.max_degree, 0);
        assert!(!s.has_loops);
        assert!(!s.has_multiple);
    }

    #[test]
    fn summary_path_graph() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();

        let s = graph_summary(&g).unwrap();
        assert!(!s.directed);
        assert_eq!(s.vcount, 5);
        assert_eq!(s.ecount, 4);
        assert!(s.connected);
        assert_eq!(s.component_count, 1);
        assert_eq!(s.min_degree, 1);
        assert_eq!(s.max_degree, 2);
        assert!(!s.has_loops);
        assert!(!s.has_multiple);
    }

    #[test]
    fn summary_disconnected_graph() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(3, 4).unwrap();
        // vertex 5 is isolated

        let s = graph_summary(&g).unwrap();
        assert!(!s.connected);
        assert_eq!(s.component_count, 3);
    }

    #[test]
    fn summary_directed_graph() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();

        let s = graph_summary(&g).unwrap();
        assert!(s.directed);
        assert!(s.connected);
        assert_eq!(s.ecount, 3);
    }

    #[test]
    fn summary_with_loops() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 1).unwrap(); // self-loop

        let s = graph_summary(&g).unwrap();
        assert!(s.has_loops);
    }

    #[test]
    fn summary_with_multiedges() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap(); // parallel edge

        let s = graph_summary(&g).unwrap();
        assert!(s.has_multiple);
    }

    #[test]
    fn summary_display_format() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();

        let s = graph_summary(&g).unwrap();
        let display = format!("{s}");
        assert!(display.contains("Undirected graph"));
        assert!(display.contains("4 vertices"));
        assert!(display.contains("4 edges"));
        assert!(display.contains("connected"));
        assert!(display.contains("density="));
    }

    #[test]
    fn summary_string_format() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let out = graph_summary_string(&g).unwrap();
        assert!(out.contains("Vertices: 3"));
        assert!(out.contains("Edges: 2"));
        assert!(out.contains("Connected: yes"));
        assert!(out.contains("Simple: yes"));
    }

    #[test]
    fn summary_singleton() {
        let g = Graph::with_vertices(1);
        let s = graph_summary(&g).unwrap();
        assert_eq!(s.vcount, 1);
        assert_eq!(s.ecount, 0);
        assert!(s.connected);
        assert_eq!(s.component_count, 1);
        assert_eq!(s.density, None);
        assert_eq!(s.min_degree, 0);
        assert_eq!(s.max_degree, 0);
    }
}

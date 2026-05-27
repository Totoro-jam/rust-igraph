//! Large Graph Layout (ALGO-LO-011).
//!
//! BFS-layered initial placement followed by Fruchterman-Reingold style
//! force-directed refinement. Designed for large graphs where global
//! force-directed algorithms are too slow.
//!
//! Reference: igraph C `igraph_layout_lgl()` in `src/layout/large_graph.c`.

use std::collections::VecDeque;

use crate::core::{Graph, IgraphError, IgraphResult};

/// Parameters for the LGL layout algorithm.
#[derive(Debug, Clone)]
pub struct LglParams {
    /// Maximum number of force-directed iterations. Default: 150.
    pub maxit: u32,
    /// Maximum displacement per iteration (per vertex). Default: vertex count.
    pub maxdelta: Option<f64>,
    /// Area parameter for force computation. Default: n².
    pub area: Option<f64>,
    /// Cooling exponent. Default: 1.5.
    pub coolexp: f64,
    /// Repulsion cancellation radius. Default: area × n.
    pub repulserad: Option<f64>,
    /// Cell size for grid acceleration. Default: area^0.25.
    pub cellsize: Option<f64>,
    /// Root vertex for BFS layering. Default: vertex with highest degree.
    pub root: Option<u32>,
}

impl Default for LglParams {
    fn default() -> Self {
        Self {
            maxit: 150,
            maxdelta: None,
            area: None,
            coolexp: 1.5,
            repulserad: None,
            cellsize: None,
            root: None,
        }
    }
}

/// Compute the Large Graph Layout (LGL).
///
/// Places vertices using BFS layering from a root vertex followed by
/// Fruchterman-Reingold style force-directed refinement with grid-based
/// repulsion acceleration.
///
/// # Arguments
///
/// * `graph` — input graph (treated as undirected).
/// * `params` — algorithm parameters (all have sensible defaults).
///
/// Returns `[x, y]` positions for each vertex.
///
/// # Errors
///
/// Returns `InvalidArgument` if root is out of range or if the graph
/// is empty.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, layout_lgl, LglParams};
///
/// let mut g = Graph::with_vertices(6);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(4, 5).unwrap();
///
/// let params = LglParams::default();
/// let pos = layout_lgl(&g, &params).unwrap();
/// assert_eq!(pos.len(), 6);
/// assert!(pos.iter().all(|p| p[0].is_finite() && p[1].is_finite()));
/// ```
pub fn layout_lgl(graph: &Graph, params: &LglParams) -> IgraphResult<Vec<[f64; 2]>> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(Vec::new());
    }
    if n == 1 {
        return Ok(vec![[0.0, 0.0]]);
    }

    let root = if let Some(r) = params.root {
        if r >= graph.vcount() {
            return Err(IgraphError::InvalidArgument(format!(
                "root vertex {} out of range (vcount={})",
                r,
                graph.vcount()
            )));
        }
        r as usize
    } else {
        highest_degree_vertex(graph)
    };

    let maxdelta = params.maxdelta.unwrap_or(n as f64);
    let area = params.area.unwrap_or((n * n) as f64);
    let coolexp = params.coolexp;
    let repulserad = params.repulserad.unwrap_or(area * n as f64);
    let cellsize = params.cellsize.unwrap_or(area.sqrt().sqrt());

    let frk = (area / n as f64).sqrt();

    // Build adjacency list
    let adj = build_adjacency(graph, n);

    // BFS layering from root
    let (layers, parent) = bfs_layers(n, root, &adj);

    // Initial placement: layer by layer on circles around parents
    let mut pos = vec![[0.0_f64; 2]; n];
    let mut placed = vec![false; n];
    let mut rng = SplitMix64::new(42);

    // Place root at origin
    pos[root] = [0.0, 0.0];
    placed[root] = true;

    // Place each subsequent layer
    for layer in layers.iter().skip(1) {
        for &v in layer {
            let p = parent[v];
            if p == usize::MAX {
                pos[v] = [rng.next_uniform() - 0.5, rng.next_uniform() - 0.5];
                placed[v] = true;
                continue;
            }
            // Place on a circle around parent
            let angle = rng.next_uniform() * std::f64::consts::TAU;
            let radius = frk * 0.5;
            pos[v] = [
                pos[p][0] + radius * angle.cos(),
                pos[p][1] + radius * angle.sin(),
            ];
            placed[v] = true;
        }
    }

    // Place any unreached vertices (disconnected components)
    for v in 0..n {
        if !placed[v] {
            pos[v] = [
                (rng.next_uniform() - 0.5) * frk * 2.0,
                (rng.next_uniform() - 0.5) * frk * 2.0,
            ];
        }
    }

    // Force-directed refinement with grid acceleration
    let mut disp = vec![[0.0_f64; 2]; n];

    for iter in 0..params.maxit {
        // Cooling schedule
        let temp = maxdelta * (-coolexp * (iter as f64) / (params.maxit as f64)).exp();

        // Clear displacements
        disp.fill([0.0, 0.0]);

        // Repulsion using grid
        apply_repulsion_grid(n, &pos, &mut disp, frk, repulserad, cellsize);

        // Attraction along edges
        for v in 0..n {
            for &u in &adj[v] {
                if u <= v {
                    continue;
                }
                let dx = pos[v][0] - pos[u][0];
                let dy = pos[v][1] - pos[u][1];
                let dist = (dx * dx + dy * dy).sqrt();
                if dist == 0.0 {
                    continue;
                }
                // Attraction: dist²/frk
                let force = dist * dist / frk;
                let fx = force * dx / dist;
                let fy = force * dy / dist;
                disp[v][0] -= fx;
                disp[v][1] -= fy;
                disp[u][0] += fx;
                disp[u][1] += fy;
            }
        }

        // Apply displacement with temperature limit
        for v in 0..n {
            let dx = disp[v][0];
            let dy = disp[v][1];
            let len = (dx * dx + dy * dy).sqrt();
            if len > 0.0 {
                let scale = temp.min(len) / len;
                pos[v][0] += dx * scale;
                pos[v][1] += dy * scale;
            }
        }
    }

    Ok(pos)
}

fn highest_degree_vertex(graph: &Graph) -> usize {
    let n = graph.vcount();
    let mut best = 0_usize;
    let mut best_deg = 0_usize;
    for v in 0..n {
        if let Ok(d) = graph.degree(v) {
            if d > best_deg {
                best_deg = d;
                best = v as usize;
            }
        }
    }
    best
}

fn build_adjacency(graph: &Graph, n: usize) -> Vec<Vec<usize>> {
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for eid in 0..graph.ecount() as u32 {
        if let Ok((src, tgt)) = graph.edge(eid) {
            let s = src as usize;
            let t = tgt as usize;
            adj[s].push(t);
            if s != t {
                adj[t].push(s);
            }
        }
    }
    adj
}

fn bfs_layers(n: usize, root: usize, adj: &[Vec<usize>]) -> (Vec<Vec<usize>>, Vec<usize>) {
    let mut visited = vec![false; n];
    let mut parent = vec![usize::MAX; n];
    let mut layers: Vec<Vec<usize>> = Vec::new();

    let mut queue = VecDeque::new();
    queue.push_back(root);
    visited[root] = true;

    let mut current_layer = vec![root];
    let mut next_layer = Vec::new();

    layers.push(current_layer.clone());

    loop {
        next_layer.clear();
        for &v in &current_layer {
            for &u in &adj[v] {
                if !visited[u] {
                    visited[u] = true;
                    parent[u] = v;
                    next_layer.push(u);
                }
            }
        }
        if next_layer.is_empty() {
            break;
        }
        layers.push(next_layer.clone());
        current_layer.clone_from(&next_layer);
    }

    (layers, parent)
}

fn apply_repulsion_grid(
    n: usize,
    pos: &[[f64; 2]],
    disp: &mut [[f64; 2]],
    frk: f64,
    repulserad: f64,
    cellsize: f64,
) {
    if n <= 100 {
        // Brute-force for small graphs
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = pos[i][0] - pos[j][0];
                let dy = pos[i][1] - pos[j][1];
                let dist_sq = dx * dx + dy * dy;
                let dist = dist_sq.sqrt();
                if dist == 0.0 || dist_sq >= repulserad {
                    continue;
                }
                // Repulsion: frk² * (1/dist - dist²/repulserad)
                let force = frk * frk * (1.0 / dist - dist_sq / repulserad);
                let fx = force * dx / dist;
                let fy = force * dy / dist;
                disp[i][0] += fx;
                disp[i][1] += fy;
                disp[j][0] -= fx;
                disp[j][1] -= fy;
            }
        }
    } else {
        // Grid-based acceleration
        grid_repulsion(n, pos, disp, frk, repulserad, cellsize);
    }
}

fn grid_repulsion(
    n: usize,
    pos: &[[f64; 2]],
    disp: &mut [[f64; 2]],
    frk: f64,
    repulserad: f64,
    cellsize: f64,
) {
    if cellsize <= 0.0 {
        return;
    }

    // Find bounding box
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for p in pos.iter().take(n) {
        if p[0] < min_x {
            min_x = p[0];
        }
        if p[0] > max_x {
            max_x = p[0];
        }
        if p[1] < min_y {
            min_y = p[1];
        }
        if p[1] > max_y {
            max_y = p[1];
        }
    }

    let width = max_x - min_x;
    let height = max_y - min_y;
    if width <= 0.0 && height <= 0.0 {
        return;
    }

    let cols = ((width / cellsize).ceil() as usize).max(1);
    let rows = ((height / cellsize).ceil() as usize).max(1);

    // Cap grid size to prevent memory explosion
    let max_cells = 10_000;
    if cols.saturating_mul(rows) > max_cells {
        // Fall back to brute-force subset: only check nearby pairs
        brute_force_repulsion(n, pos, disp, frk, repulserad);
        return;
    }

    // Assign vertices to cells
    let mut grid: Vec<Vec<usize>> = vec![Vec::new(); cols * rows];
    let mut cell_of = vec![0_usize; n];

    for v in 0..n {
        let cx = ((pos[v][0] - min_x) / cellsize).floor() as usize;
        let cy = ((pos[v][1] - min_y) / cellsize).floor() as usize;
        let cx = cx.min(cols - 1);
        let cy = cy.min(rows - 1);
        let cell = cy * cols + cx;
        grid[cell].push(v);
        cell_of[v] = cell;
    }

    // For each vertex, check its cell and 8 neighbors
    for v in 0..n {
        let cell = cell_of[v];
        let cy = cell / cols;
        let cx = cell % cols;

        let row_start = if cy > 0 { cy - 1 } else { 0 };
        let row_end = (cy + 1).min(rows - 1);
        let col_start = if cx > 0 { cx - 1 } else { 0 };
        let col_end = (cx + 1).min(cols - 1);

        for ry in row_start..=row_end {
            for rx in col_start..=col_end {
                let neighbor_cell = ry * cols + rx;
                for &u in &grid[neighbor_cell] {
                    if u <= v {
                        continue;
                    }
                    let dx = pos[v][0] - pos[u][0];
                    let dy = pos[v][1] - pos[u][1];
                    let dist_sq = dx * dx + dy * dy;
                    let dist = dist_sq.sqrt();
                    if dist == 0.0 || dist_sq >= repulserad {
                        continue;
                    }
                    let force = frk * frk * (1.0 / dist - dist_sq / repulserad);
                    let fx = force * dx / dist;
                    let fy = force * dy / dist;
                    disp[v][0] += fx;
                    disp[v][1] += fy;
                    disp[u][0] -= fx;
                    disp[u][1] -= fy;
                }
            }
        }
    }
}

fn brute_force_repulsion(
    n: usize,
    pos: &[[f64; 2]],
    disp: &mut [[f64; 2]],
    frk: f64,
    repulserad: f64,
) {
    for i in 0..n {
        for j in (i + 1)..n {
            let dx = pos[i][0] - pos[j][0];
            let dy = pos[i][1] - pos[j][1];
            let dist_sq = dx * dx + dy * dy;
            let dist = dist_sq.sqrt();
            if dist == 0.0 || dist_sq >= repulserad {
                continue;
            }
            let force = frk * frk * (1.0 / dist - dist_sq / repulserad);
            let fx = force * dx / dist;
            let fy = force * dy / dist;
            disp[i][0] += fx;
            disp[i][1] += fy;
            disp[j][0] -= fx;
            disp[j][1] -= fy;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Internal RNG
// ═══════════════════════════════════════════════════════════════════

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn next_uniform(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / ((1u64 << 53) as f64)
    }
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lgl_empty() {
        let g = Graph::with_vertices(0);
        let params = LglParams::default();
        let pos = layout_lgl(&g, &params).unwrap();
        assert!(pos.is_empty());
    }

    #[test]
    fn test_lgl_single() {
        let g = Graph::with_vertices(1);
        let params = LglParams::default();
        let pos = layout_lgl(&g, &params).unwrap();
        assert_eq!(pos.len(), 1);
        assert!(pos[0][0].abs() < 1e-10 && pos[0][1].abs() < 1e-10);
    }

    #[test]
    fn test_lgl_two_vertices() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let params = LglParams::default();
        let pos = layout_lgl(&g, &params).unwrap();
        assert_eq!(pos.len(), 2);
        assert!(pos[0][0].is_finite() && pos[1][0].is_finite());
        let dx = pos[0][0] - pos[1][0];
        let dy = pos[0][1] - pos[1][1];
        assert!((dx * dx + dy * dy).sqrt() > 0.01);
    }

    #[test]
    fn test_lgl_path() {
        let mut g = Graph::with_vertices(6);
        for i in 0..5 {
            g.add_edge(i, i + 1).unwrap();
        }
        let params = LglParams::default();
        let pos = layout_lgl(&g, &params).unwrap();
        assert_eq!(pos.len(), 6);
        for p in &pos {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }

    #[test]
    fn test_lgl_cycle() {
        let mut g = Graph::with_vertices(8);
        for i in 0..8 {
            g.add_edge(i, (i + 1) % 8).unwrap();
        }
        let params = LglParams::default();
        let pos = layout_lgl(&g, &params).unwrap();
        assert_eq!(pos.len(), 8);
        for p in &pos {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }

    #[test]
    fn test_lgl_complete() {
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        let params = LglParams::default();
        let pos = layout_lgl(&g, &params).unwrap();
        assert_eq!(pos.len(), 5);
        for p in &pos {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }

    #[test]
    fn test_lgl_with_root() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        let params = LglParams {
            root: Some(2),
            ..LglParams::default()
        };
        let pos = layout_lgl(&g, &params).unwrap();
        assert_eq!(pos.len(), 5);
        for p in &pos {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }

    #[test]
    fn test_lgl_root_out_of_range() {
        let g = Graph::with_vertices(3);
        let params = LglParams {
            root: Some(5),
            ..LglParams::default()
        };
        assert!(layout_lgl(&g, &params).is_err());
    }

    #[test]
    fn test_lgl_disconnected() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        let params = LglParams::default();
        let pos = layout_lgl(&g, &params).unwrap();
        assert_eq!(pos.len(), 6);
        for p in &pos {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }

    #[test]
    fn test_lgl_deterministic() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        let params = LglParams::default();
        let pos1 = layout_lgl(&g, &params).unwrap();
        let pos2 = layout_lgl(&g, &params).unwrap();
        for i in 0..5 {
            assert!((pos1[i][0] - pos2[i][0]).abs() < 1e-10);
            assert!((pos1[i][1] - pos2[i][1]).abs() < 1e-10);
        }
    }

    #[test]
    fn test_lgl_custom_params() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        let params = LglParams {
            maxit: 50,
            coolexp: 2.0,
            ..LglParams::default()
        };
        let pos = layout_lgl(&g, &params).unwrap();
        assert_eq!(pos.len(), 5);
        for p in &pos {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }

    #[test]
    fn test_lgl_star_topology() {
        let mut g = Graph::with_vertices(7);
        for i in 1..7 {
            g.add_edge(0, i).unwrap();
        }
        let params = LglParams {
            root: Some(0),
            ..LglParams::default()
        };
        let pos = layout_lgl(&g, &params).unwrap();
        assert_eq!(pos.len(), 7);
        for p in &pos {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }
}

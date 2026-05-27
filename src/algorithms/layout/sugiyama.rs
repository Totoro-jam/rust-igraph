//! Sugiyama layered layout (ALGO-LO-005).
//!
//! Hierarchical layout for directed (acyclic) graphs. Reference:
//! K. Sugiyama, S. Tagawa, M. Toda, "Methods for Visual Understanding
//! of Hierarchical Systems", IEEE Trans. SMC 11(2), 1981.
//!
//! Coordinate assignment uses the Brandes-Köpf method:
//! U. Brandes, B. Köpf, "Fast and Simple Horizontal Coordinate
//! Assignment", LNCS 2265:31-44, 2002.

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};
use std::collections::VecDeque;

/// Result of a Sugiyama layout computation.
#[derive(Debug, Clone)]
pub struct SugiyamaResult {
    /// Positions `[x, y]` for each vertex in the original graph (indices 0..vcount).
    pub positions: Vec<[f64; 2]>,
    /// Positions of dummy vertices added for long edges (indices vcount..).
    pub dummy_positions: Vec<[f64; 2]>,
    /// The extended graph containing both original and dummy vertices.
    /// Each edge spans exactly one layer and points downward.
    pub extended_graph: Graph,
    /// For each edge in `extended_graph`, the index of the corresponding
    /// original edge. Multiple extended edges map to the same original edge
    /// when dummy nodes split a long edge.
    pub extended_to_orig_eids: Vec<u32>,
}

/// Parameters for the Sugiyama layout.
#[derive(Debug, Clone)]
pub struct SugiyamaParams {
    /// Preferred horizontal gap between vertices in the same layer.
    pub hgap: f64,
    /// Vertical distance between layers.
    pub vgap: f64,
    /// Maximum number of iterations for crossing minimization.
    pub maxiter: u32,
}

impl Default for SugiyamaParams {
    fn default() -> Self {
        Self {
            hgap: 1.0,
            vgap: 1.0,
            maxiter: 100,
        }
    }
}

/// Compute the Sugiyama hierarchical layout.
///
/// Designed for directed acyclic graphs where vertices are assigned to
/// layers. Vertices on the same layer share the same Y coordinate. The
/// algorithm minimizes edge crossings via the barycenter heuristic and
/// assigns X coordinates using the Brandes-Köpf method.
///
/// For graphs with cycles, a DFS-based feedback arc set is computed and
/// those edges are reversed before layering.
///
/// If `layers` is `None`, layer assignment is computed automatically using
/// longest-path layering.
///
/// # Arguments
///
/// * `graph` — input graph (directed preferred; undirected is treated as
///   all edges pointing both ways).
/// * `layers` — optional per-vertex layer assignment (0-indexed). If
///   provided, must have length equal to `graph.vcount()`.
/// * `params` — layout parameters (gaps, iterations).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, layout_sugiyama, SugiyamaParams};
///
/// // Simple DAG: 0 -> 1, 0 -> 2, 1 -> 3, 2 -> 3
/// let mut g = Graph::new(4, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// g.add_edge(2, 3).unwrap();
///
/// let result = layout_sugiyama(&g, None, &SugiyamaParams::default()).unwrap();
/// assert_eq!(result.positions.len(), 4);
/// // Vertex 0 should be at top (y=0), vertex 3 at bottom (y=2)
/// assert!((result.positions[0][1]).abs() < 1e-10);
/// assert!((result.positions[3][1] - 2.0).abs() < 1e-10);
/// ```
pub fn layout_sugiyama(
    graph: &Graph,
    layers: Option<&[u32]>,
    params: &SugiyamaParams,
) -> IgraphResult<SugiyamaResult> {
    let n = graph.vcount() as usize;

    if n == 0 {
        return Ok(SugiyamaResult {
            positions: Vec::new(),
            dummy_positions: Vec::new(),
            extended_graph: Graph::new(0, graph.is_directed())?,
            extended_to_orig_eids: Vec::new(),
        });
    }

    // Step 1: Determine layer assignment
    let mut layer_assign = if let Some(l) = layers {
        if l.len() != n {
            return Err(IgraphError::InvalidArgument(
                "layer vector length must equal vertex count".into(),
            ));
        }
        l.to_vec()
    } else {
        compute_layers(graph)?
    };

    // Normalize layers: eliminate empty layers and make contiguous 0..k
    let layer_to_y = normalize_layers(&mut layer_assign, params.vgap);

    // Step 2: Build extended graph with dummy nodes for long edges
    let ext = build_extended_graph(graph, &layer_assign)?;

    // Step 3: Compute layout for each connected component
    let mut positions = vec![[0.0_f64; 2]; n];
    let mut dummy_positions: Vec<[f64; 2]> = Vec::new();

    // For simplicity, process all vertices together (single-component path)
    let total_nodes = ext.node_count;
    let ext_layers = ext.layers.clone();

    // Build layering structure
    let num_layers = ext_layers.iter().copied().max().unwrap_or(0) as usize + 1;
    let mut layering: Vec<Vec<usize>> = vec![Vec::new(); num_layers];
    for (v, &lyr) in ext_layers.iter().enumerate() {
        layering[lyr as usize].push(v);
    }

    // Assign initial Y coordinates
    let mut layout_y = vec![0.0_f64; total_nodes];
    for (v, &lyr) in ext_layers.iter().enumerate() {
        let y_idx = lyr as usize;
        layout_y[v] = if y_idx < layer_to_y.len() {
            layer_to_y[y_idx]
        } else {
            f64::from(lyr) * params.vgap
        };
    }

    // Step 4: Order nodes horizontally (crossing minimization)
    let mut layout_x = vec![0.0_f64; total_nodes];
    order_horizontally(
        &ext.edges,
        total_nodes,
        &layering,
        &mut layout_x,
        params.maxiter,
    );

    // Step 5: Assign horizontal coordinates (Brandes-Köpf simplified)
    place_horizontally(
        &ext.edges,
        total_nodes,
        &layering,
        &mut layout_x,
        &ext_layers,
        params.hgap,
        n,
    );

    // Collect results
    for v in 0..n {
        positions[v] = [layout_x[v], layout_y[v]];
    }
    for v in n..total_nodes {
        dummy_positions.push([layout_x[v], layout_y[v]]);
    }

    // Build the actual extended Graph
    let mut ext_graph = Graph::new(total_nodes as u32, graph.is_directed())?;
    for &(src, tgt) in &ext.edges {
        ext_graph.add_edge(src as VertexId, tgt as VertexId)?;
    }

    Ok(SugiyamaResult {
        positions,
        dummy_positions,
        extended_graph: ext_graph,
        extended_to_orig_eids: ext.edge_to_orig,
    })
}

// ═══════════════════════════════════════════════════════════════════
// Layer assignment
// ═══════════════════════════════════════════════════════════════════

fn compute_layers(graph: &Graph) -> IgraphResult<Vec<u32>> {
    let n = graph.vcount() as usize;
    let directed = graph.is_directed();

    // For directed graphs: use longest-path layering after removing cycles
    // For undirected: use BFS-based layering from a source vertex
    if directed {
        longest_path_layering(graph)
    } else {
        // BFS from vertex 0 (or highest-degree vertex)
        let mut layers = vec![0u32; n];
        let mut visited = vec![false; n];
        let mut queue = VecDeque::new();

        let root = select_highest_degree(graph);
        visited[root] = true;
        queue.push_back(root);

        while let Some(v) = queue.pop_front() {
            let neighbors = all_neighbors(graph, v as VertexId);
            for nei in neighbors {
                let nei_idx = nei as usize;
                if !visited[nei_idx] {
                    visited[nei_idx] = true;
                    layers[nei_idx] = layers[v]
                        .checked_add(1)
                        .ok_or_else(|| IgraphError::InvalidArgument("layer overflow".into()))?;
                    queue.push_back(nei_idx);
                }
            }
        }

        // Handle disconnected vertices
        for i in 0..n {
            if !visited[i] {
                layers[i] = 0;
            }
        }

        Ok(layers)
    }
}

fn longest_path_layering(graph: &Graph) -> IgraphResult<Vec<u32>> {
    let n = graph.vcount() as usize;
    let ecount = graph.ecount();

    // First, find a feedback arc set via DFS and compute a topological order
    // ignoring back edges
    let mut in_degree = vec![0u32; n];
    let mut adj_out: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut back_edges = vec![false; ecount];

    // DFS to find back edges
    let mut color = vec![0u8; n]; // 0=white, 1=gray, 2=black
    let mut stack: Vec<(usize, usize)> = Vec::new(); // (vertex, neighbor_index)

    for start in 0..n {
        if color[start] != 0 {
            continue;
        }
        stack.push((start, 0));
        color[start] = 1;

        while let Some((v, idx)) = stack.last_mut() {
            let v_id = *v;
            let out_edges = out_edges_of(graph, v_id as VertexId);
            if *idx >= out_edges.len() {
                color[v_id] = 2;
                stack.pop();
            } else {
                let (eid, tgt) = out_edges[*idx];
                *idx += 1;
                let tgt_idx = tgt as usize;
                if color[tgt_idx] == 1 {
                    // Back edge — mark for removal
                    back_edges[eid] = true;
                } else if color[tgt_idx] == 0 {
                    color[tgt_idx] = 1;
                    stack.push((tgt_idx, 0));
                }
            }
        }
    }

    // Build DAG adjacency (ignoring back edges)
    for eid in 0..ecount {
        if back_edges[eid] {
            continue;
        }
        if let Ok((src, tgt)) = graph.edge(eid as u32) {
            let src_idx = src as usize;
            let tgt_idx = tgt as usize;
            adj_out[src_idx].push(tgt_idx);
            in_degree[tgt_idx] = in_degree[tgt_idx]
                .checked_add(1)
                .ok_or_else(|| IgraphError::InvalidArgument("in-degree overflow".into()))?;
        }
    }

    // Longest-path layering on the DAG
    // layer[v] = max(layer[pred] + 1) for all predecessors
    let mut layers = vec![0u32; n];
    let mut queue: VecDeque<usize> = VecDeque::new();
    let mut remaining_in = in_degree.clone();

    // Start from sources (in_degree == 0)
    for v in 0..n {
        if remaining_in[v] == 0 {
            queue.push_back(v);
        }
    }

    while let Some(v) = queue.pop_front() {
        for &w in &adj_out[v] {
            let new_layer = layers[v]
                .checked_add(1)
                .ok_or_else(|| IgraphError::InvalidArgument("layer overflow".into()))?;
            if new_layer > layers[w] {
                layers[w] = new_layer;
            }
            remaining_in[w] -= 1;
            if remaining_in[w] == 0 {
                queue.push_back(w);
            }
        }
    }

    // If there are unreachable vertices (isolated or in unprocessed cycles),
    // assign them layer 0
    Ok(layers)
}

fn normalize_layers(layers: &mut [u32], vgap: f64) -> Vec<f64> {
    if layers.is_empty() {
        return Vec::new();
    }

    // Sort unique layer values and create contiguous mapping
    let mut unique: Vec<u32> = layers.to_vec();
    unique.sort_unstable();
    unique.dedup();

    let mut layer_to_y = Vec::with_capacity(unique.len());
    for (new_idx, &old_val) in unique.iter().enumerate() {
        layer_to_y.push(f64::from(old_val) * vgap);
        for l in layers.iter_mut() {
            if *l == old_val {
                *l = new_idx as u32;
            }
        }
    }

    // Re-derive layer_to_y based on new contiguous indices
    let num_layers = unique.len();
    let mut result = vec![0.0; num_layers];
    for i in 0..num_layers {
        result[i] = i as f64 * vgap;
    }
    result
}

// ═══════════════════════════════════════════════════════════════════
// Extended graph (dummy nodes for long edges)
// ═══════════════════════════════════════════════════════════════════

struct ExtendedGraph {
    node_count: usize,
    edges: Vec<(usize, usize)>,
    edge_to_orig: Vec<u32>,
    layers: Vec<u32>,
}

fn build_extended_graph(graph: &Graph, layers: &[u32]) -> IgraphResult<ExtendedGraph> {
    let n = graph.vcount() as usize;
    let ecount = graph.ecount() as u32;
    let mut next_node = n;
    let mut edges: Vec<(usize, usize)> = Vec::new();
    let mut edge_to_orig: Vec<u32> = Vec::new();
    let mut node_layers: Vec<u32> = layers.to_vec();

    for eid in 0..ecount {
        let (src, tgt) = graph.edge(eid)?;
        let src_idx = src as usize;
        let tgt_idx = tgt as usize;
        let src_layer = layers[src_idx];
        let tgt_layer = layers[tgt_idx];

        if src_layer == tgt_layer {
            // Same-layer edge: include directly
            edges.push((src_idx, tgt_idx));
            edge_to_orig.push(eid);
        } else {
            // Determine direction: edge should go from lower to higher layer
            let (from, to, from_layer, to_layer) = if src_layer < tgt_layer {
                (src_idx, tgt_idx, src_layer, tgt_layer)
            } else {
                (tgt_idx, src_idx, tgt_layer, src_layer)
            };

            if to_layer - from_layer == 1 {
                // Spans exactly one layer — no dummy needed
                edges.push((from, to));
                edge_to_orig.push(eid);
            } else {
                // Insert dummy nodes
                let mut prev = from;
                for lyr in (from_layer + 1)..to_layer {
                    let dummy = next_node;
                    next_node += 1;
                    node_layers.push(lyr);
                    edges.push((prev, dummy));
                    edge_to_orig.push(eid);
                    prev = dummy;
                }
                edges.push((prev, to));
                edge_to_orig.push(eid);
            }
        }
    }

    Ok(ExtendedGraph {
        node_count: next_node,
        edges,
        edge_to_orig,
        layers: node_layers,
    })
}

// ═══════════════════════════════════════════════════════════════════
// Crossing minimization (barycenter heuristic)
// ═══════════════════════════════════════════════════════════════════

fn order_horizontally(
    edges: &[(usize, usize)],
    node_count: usize,
    layering: &[Vec<usize>],
    layout_x: &mut [f64],
    maxiter: u32,
) {
    let num_layers = layering.len();
    if num_layers <= 1 {
        // Single layer — just assign sequential positions
        for (i, &v) in layering
            .first()
            .map_or(&[][..], |l| l.as_slice())
            .iter()
            .enumerate()
        {
            layout_x[v] = i as f64;
        }
        return;
    }

    // Build adjacency: for each node, predecessors (in higher layer) and successors
    let mut predecessors: Vec<Vec<usize>> = vec![Vec::new(); node_count];
    let mut successors: Vec<Vec<usize>> = vec![Vec::new(); node_count];
    for &(src, tgt) in edges {
        successors[src].push(tgt);
        predecessors[tgt].push(src);
    }

    // Initial ordering: sequential within each layer
    let mut layer_order: Vec<Vec<usize>> = layering.to_vec();
    for (layer_idx, layer) in layer_order.iter().enumerate() {
        for (pos, &v) in layer.iter().enumerate() {
            layout_x[v] = pos as f64;
        }
        let _ = layer_idx;
    }

    // Iterate: sweep down then up, reordering by barycenters
    let mut changed = true;
    let mut iter = 0u32;
    while changed && iter < maxiter {
        changed = false;

        // Sweep downward: for each layer (top to bottom), sort by
        // barycenter of predecessors
        for layer_idx in 1..num_layers {
            let layer = &layer_order[layer_idx];
            let layer_size = layer.len();
            if layer_size == 0 {
                continue;
            }

            let mut barycenters: Vec<(f64, usize)> = Vec::with_capacity(layer_size);
            for &v in layer {
                let preds = &predecessors[v];
                let bc = if preds.is_empty() {
                    layout_x[v]
                } else {
                    let sum: f64 = preds.iter().map(|&p| layout_x[p]).sum();
                    sum / preds.len() as f64
                };
                barycenters.push((bc, v));
            }

            barycenters.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

            let new_order: Vec<usize> = barycenters.iter().map(|&(_, v)| v).collect();
            if new_order != layer_order[layer_idx] {
                changed = true;
                for (pos, &v) in new_order.iter().enumerate() {
                    layout_x[v] = pos as f64;
                }
                layer_order[layer_idx] = new_order;
            }
        }

        // Sweep upward: for each layer (bottom to top), sort by
        // barycenter of successors
        for layer_idx in (0..num_layers.saturating_sub(1)).rev() {
            let layer = &layer_order[layer_idx];
            let layer_size = layer.len();
            if layer_size == 0 {
                continue;
            }

            let mut barycenters: Vec<(f64, usize)> = Vec::with_capacity(layer_size);
            for &v in layer {
                let succs = &successors[v];
                let bc = if succs.is_empty() {
                    layout_x[v]
                } else {
                    let sum: f64 = succs.iter().map(|&s| layout_x[s]).sum();
                    sum / succs.len() as f64
                };
                barycenters.push((bc, v));
            }

            barycenters.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

            let new_order: Vec<usize> = barycenters.iter().map(|&(_, v)| v).collect();
            if new_order != layer_order[layer_idx] {
                changed = true;
                for (pos, &v) in new_order.iter().enumerate() {
                    layout_x[v] = pos as f64;
                }
                layer_order[layer_idx] = new_order;
            }
        }

        iter += 1;
    }
}

// ═══════════════════════════════════════════════════════════════════
// Horizontal coordinate assignment (simplified Brandes-Köpf)
// ═══════════════════════════════════════════════════════════════════

fn place_horizontally(
    edges: &[(usize, usize)],
    node_count: usize,
    layering: &[Vec<usize>],
    layout_x: &mut [f64],
    _node_layers: &[u32],
    hgap: f64,
    _no_of_real_nodes: usize,
) {
    // Build predecessor list
    let mut predecessors: Vec<Vec<usize>> = vec![Vec::new(); node_count];
    for &(src, tgt) in edges {
        predecessors[tgt].push(src);
    }

    // Run 4 alignments (up-left, up-right, down-left, down-right) and
    // take median X for each vertex
    let mut all_xs: [Vec<f64>; 4] = [
        vec![0.0; node_count],
        vec![0.0; node_count],
        vec![0.0; node_count],
        vec![0.0; node_count],
    ];

    for combo in 0..4u8 {
        let reverse = combo / 2 == 1;
        let align_right = combo % 2 == 1;

        // Vertical alignment
        let mut roots = (0..node_count).collect::<Vec<_>>();
        let mut align = (0..node_count).collect::<Vec<_>>();

        vertical_alignment(
            edges,
            node_count,
            layering,
            layout_x,
            reverse,
            align_right,
            &mut roots,
            &mut align,
            &predecessors,
        );

        // Horizontal compaction
        horizontal_compaction(
            node_count,
            layering,
            &roots,
            &align,
            hgap,
            &mut all_xs[combo as usize],
        );
    }

    // Align the 4 coordinate sets and take the median
    let mut mins = [0.0_f64; 4];
    let mut maxs = [0.0_f64; 4];
    for i in 0..4 {
        mins[i] = all_xs[i].iter().copied().fold(f64::INFINITY, f64::min);
        maxs[i] = all_xs[i].iter().copied().fold(f64::NEG_INFINITY, f64::max);
    }

    // Find narrowest alignment
    let mut best = 0;
    let mut best_width = f64::INFINITY;
    for i in 0..4 {
        let w = maxs[i] - mins[i];
        if w < best_width {
            best_width = w;
            best = i;
        }
    }

    // Shift alignments to match
    for i in 0..4 {
        if i == best {
            continue;
        }
        let diff = if i % 2 == 0 {
            mins[best] - mins[i]
        } else {
            maxs[best] - maxs[i]
        };
        for x in &mut all_xs[i] {
            *x += diff;
        }
    }

    // Take median of 4 values for each vertex
    for v in 0..node_count {
        let mut vals = [all_xs[0][v], all_xs[1][v], all_xs[2][v], all_xs[3][v]];
        vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        layout_x[v] = (vals[1] + vals[2]) * 0.5;
    }
}

fn vertical_alignment(
    edges: &[(usize, usize)],
    node_count: usize,
    layering: &[Vec<usize>],
    layout_x: &[f64],
    reverse: bool,
    align_right: bool,
    roots: &mut [usize],
    align: &mut [usize],
    predecessors: &[Vec<usize>],
) {
    let num_layers = layering.len();

    // Build successors
    let mut successors: Vec<Vec<usize>> = vec![Vec::new(); node_count];
    for &(src, tgt) in edges {
        successors[src].push(tgt);
    }

    // Initialize: each vertex is its own root and aligned to itself
    for i in 0..node_count {
        roots[i] = i;
        align[i] = i;
    }

    // Process layers
    let layer_range: Vec<usize> = if reverse {
        (0..num_layers.saturating_sub(1)).rev().collect()
    } else {
        (1..num_layers).collect()
    };

    for &layer_idx in &layer_range {
        let layer = &layering[layer_idx];
        let mut r: i64 = if align_right { i64::MAX } else { -1 };

        let vertex_iter: Vec<usize> = if align_right {
            layer.iter().copied().rev().collect()
        } else {
            layer.clone()
        };

        for vertex in vertex_iter {
            if align[vertex] != vertex {
                continue;
            }

            // Get upper/lower neighbors depending on direction
            let neighbors: &Vec<usize> = if reverse {
                &successors[vertex]
            } else {
                &predecessors[vertex]
            };

            if neighbors.is_empty() {
                continue;
            }

            // Sort neighbors by their X position
            let mut sorted_neis: Vec<usize> = neighbors.clone();
            sorted_neis.sort_by(|&a, &b| {
                layout_x[a]
                    .partial_cmp(&layout_x[b])
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            // Find median neighbor(s)
            let n_neis = sorted_neis.len();
            let medians = if n_neis == 1 {
                vec![sorted_neis[0]]
            } else if n_neis % 2 == 1 {
                vec![sorted_neis[n_neis / 2]]
            } else if align_right {
                vec![sorted_neis[n_neis / 2], sorted_neis[n_neis / 2 - 1]]
            } else {
                vec![sorted_neis[n_neis / 2 - 1], sorted_neis[n_neis / 2]]
            };

            // Try to align with median
            for median in medians {
                if align[vertex] != vertex {
                    break;
                }
                let pos = layout_x[median] as i64;
                if (align_right && r > pos) || (!align_right && r < pos) {
                    align[median] = vertex;
                    roots[vertex] = roots[median];
                    align[vertex] = roots[median];
                    r = pos;
                }
            }
        }
    }
}

fn horizontal_compaction(
    node_count: usize,
    layering: &[Vec<usize>],
    roots: &[usize],
    align: &[usize],
    hgap: f64,
    xs: &mut [f64],
) {
    // Build vertex_to_the_left
    let mut vertex_to_left: Vec<usize> = (0..node_count).collect();
    for layer in layering {
        if layer.len() <= 1 {
            continue;
        }
        for i in 1..layer.len() {
            vertex_to_left[layer[i]] = layer[i - 1];
        }
    }

    let mut sinks: Vec<usize> = (0..node_count).collect();
    let mut shifts = vec![f64::INFINITY; node_count];

    // Initialize xs to -1 (unplaced)
    for x in xs.iter_mut() {
        *x = -1.0;
    }

    // Place blocks starting from roots
    for i in 0..node_count {
        if roots[i] == i {
            place_block(
                i,
                &vertex_to_left,
                roots,
                align,
                &mut sinks,
                &mut shifts,
                hgap,
                xs,
            );
        }
    }

    // Calculate absolute coordinates
    let old_xs = xs.to_vec();
    for i in 0..node_count {
        let root = roots[i];
        xs[i] = old_xs[root];
        let shift = shifts[sinks[root]];
        if shift < f64::INFINITY {
            xs[i] += shift;
        }
    }
}

fn place_block(
    v: usize,
    vertex_to_left: &[usize],
    roots: &[usize],
    align: &[usize],
    sinks: &mut [usize],
    shifts: &mut [f64],
    hgap: f64,
    xs: &mut [f64],
) {
    if xs[v] >= 0.0 {
        return;
    }

    xs[v] = 0.0;

    let mut w = v;
    loop {
        let u_left = vertex_to_left[w];
        if u_left != w {
            let u = roots[u_left];
            place_block(u, vertex_to_left, roots, align, sinks, shifts, hgap, xs);

            let u_sink = sinks[u];
            let v_sink = sinks[v];
            if v_sink == v {
                sinks[v] = u_sink;
            }

            let current_v_sink = sinks[v];
            if current_v_sink != u_sink {
                let candidate = xs[v] - xs[u] - hgap;
                if shifts[u_sink] > candidate {
                    shifts[u_sink] = candidate;
                }
            } else if xs[v] < xs[u] + hgap {
                xs[v] = xs[u] + hgap;
            }
        }

        w = align[w];
        if w == v {
            break;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════

fn select_highest_degree(graph: &Graph) -> usize {
    let n = graph.vcount() as usize;
    if n == 0 {
        return 0;
    }
    let mut best = 0;
    let mut best_deg = 0;
    for v in 0..n {
        if let Ok(d) = graph.degree(v as VertexId) {
            if d > best_deg {
                best_deg = d;
                best = v;
            }
        }
    }
    best
}

fn all_neighbors(graph: &Graph, v: VertexId) -> Vec<VertexId> {
    let mut result = Vec::new();
    let ecount = graph.ecount();
    for eid in 0..ecount as u32 {
        if let Ok((src, tgt)) = graph.edge(eid) {
            if src == v && tgt != v {
                result.push(tgt);
            } else if tgt == v && src != v {
                result.push(src);
            }
        }
    }
    result
}

fn out_edges_of(graph: &Graph, v: VertexId) -> Vec<(usize, VertexId)> {
    let mut result = Vec::new();
    let ecount = graph.ecount();
    for eid in 0..ecount as u32 {
        if let Ok((src, tgt)) = graph.edge(eid) {
            if graph.is_directed() {
                if src == v && tgt != v {
                    result.push((eid as usize, tgt));
                }
            } else if src == v && tgt != v {
                result.push((eid as usize, tgt));
            } else if tgt == v && src != v {
                result.push((eid as usize, src));
            }
        }
    }
    result
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_dag() -> Graph {
        // 0 -> 1, 0 -> 2, 1 -> 3, 2 -> 3
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        g
    }

    #[test]
    fn test_sugiyama_basic_dag() {
        let g = simple_dag();
        let result = layout_sugiyama(&g, None, &SugiyamaParams::default()).unwrap();
        assert_eq!(result.positions.len(), 4);
        // Vertex 0 at top layer
        assert!(result.positions[0][1].abs() < 1e-10);
        // Vertex 3 at bottom layer (layer 2)
        assert!((result.positions[3][1] - 2.0).abs() < 1e-10);
        // Vertices 1 and 2 at middle layer
        assert!((result.positions[1][1] - 1.0).abs() < 1e-10);
        assert!((result.positions[2][1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_sugiyama_empty() {
        let g = Graph::new(0, true).unwrap();
        let result = layout_sugiyama(&g, None, &SugiyamaParams::default()).unwrap();
        assert!(result.positions.is_empty());
    }

    #[test]
    fn test_sugiyama_single_vertex() {
        let g = Graph::new(1, true).unwrap();
        let result = layout_sugiyama(&g, None, &SugiyamaParams::default()).unwrap();
        assert_eq!(result.positions.len(), 1);
        assert!(result.positions[0][0].abs() < 1e-10);
        assert!(result.positions[0][1].abs() < 1e-10);
    }

    #[test]
    fn test_sugiyama_linear_chain() {
        // 0 -> 1 -> 2 -> 3
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let result = layout_sugiyama(&g, None, &SugiyamaParams::default()).unwrap();
        // Each vertex should be on a successive layer
        for i in 0..4 {
            assert!(
                (result.positions[i][1] - i as f64).abs() < 1e-10,
                "vertex {i} y={}, expected {i}",
                result.positions[i][1]
            );
        }
    }

    #[test]
    fn test_sugiyama_with_layers() {
        let g = simple_dag();
        let layers = vec![0, 1, 1, 2];
        let result = layout_sugiyama(&g, Some(&layers), &SugiyamaParams::default()).unwrap();
        assert_eq!(result.positions.len(), 4);
        assert!(result.positions[0][1].abs() < 1e-10);
        assert!((result.positions[3][1] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_sugiyama_invalid_layers() {
        let g = simple_dag();
        let layers = vec![0, 1]; // Wrong length
        let result = layout_sugiyama(&g, Some(&layers), &SugiyamaParams::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_sugiyama_long_edge_dummies() {
        // 0 -> 1, 0 -> 2, with layers [0, 2, 1]
        // Edge 0->1 spans 2 layers, needs a dummy
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        let layers = vec![0, 2, 1];
        let result = layout_sugiyama(&g, Some(&layers), &SugiyamaParams::default()).unwrap();
        assert_eq!(result.positions.len(), 3);
        // Should have one dummy for the 0->1 edge
        assert_eq!(result.dummy_positions.len(), 1);
        // Dummy should be at layer 1
        assert!((result.dummy_positions[0][1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_sugiyama_diamond_symmetry() {
        let g = simple_dag();
        let result = layout_sugiyama(&g, None, &SugiyamaParams::default()).unwrap();
        // Vertices 1 and 2 should be symmetric around vertex 0's x
        let mid_x = result.positions[0][0];
        let left = result.positions[1][0] - mid_x;
        let right = result.positions[2][0] - mid_x;
        assert!(
            (left + right).abs() < 1e-10,
            "not symmetric: left={left}, right={right}"
        );
    }

    #[test]
    fn test_sugiyama_no_overlap_same_layer() {
        let g = simple_dag();
        let result = layout_sugiyama(&g, None, &SugiyamaParams::default()).unwrap();
        // Vertices 1 and 2 are on the same layer — should not overlap
        let x1 = result.positions[1][0];
        let x2 = result.positions[2][0];
        assert!((x1 - x2).abs() >= 1.0 - 1e-10, "overlap: x1={x1}, x2={x2}");
    }

    #[test]
    fn test_sugiyama_undirected() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let result = layout_sugiyama(&g, None, &SugiyamaParams::default()).unwrap();
        assert_eq!(result.positions.len(), 4);
        // All positions should be finite
        for p in &result.positions {
            assert!(p[0].is_finite());
            assert!(p[1].is_finite());
        }
    }

    #[test]
    fn test_sugiyama_with_cycle() {
        // 0 -> 1 -> 2 -> 0 (cycle)
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let result = layout_sugiyama(&g, None, &SugiyamaParams::default()).unwrap();
        assert_eq!(result.positions.len(), 3);
        // Should produce valid layout despite cycle
        for p in &result.positions {
            assert!(p[0].is_finite());
            assert!(p[1].is_finite());
        }
    }

    #[test]
    fn test_sugiyama_custom_gaps() {
        let g = simple_dag();
        let params = SugiyamaParams {
            hgap: 2.0,
            vgap: 3.0,
            maxiter: 50,
        };
        let result = layout_sugiyama(&g, None, &params).unwrap();
        // Vertex 3 should be at y = 2 * vgap = 6.0
        assert!((result.positions[3][1] - 6.0).abs() < 1e-10);
        // Vertices 1 and 2 should be at least hgap=2.0 apart
        let x1 = result.positions[1][0];
        let x2 = result.positions[2][0];
        assert!(
            (x1 - x2).abs() >= 2.0 - 1e-10,
            "gap too small: |{x1} - {x2}| = {}",
            (x1 - x2).abs()
        );
    }

    #[test]
    fn test_sugiyama_extended_graph() {
        let g = simple_dag();
        let result = layout_sugiyama(&g, None, &SugiyamaParams::default()).unwrap();
        // Extended graph should have at least as many vertices as original
        assert!(result.extended_graph.vcount() >= g.vcount());
        // Each extended edge should map to a valid original edge
        for &orig_eid in &result.extended_to_orig_eids {
            assert!(orig_eid < g.ecount() as u32);
        }
    }
}

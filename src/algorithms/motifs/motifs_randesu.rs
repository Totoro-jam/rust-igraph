//! Motif census via ESU enumeration (ALGO-MO-005).
//!
//! Counterpart of `igraph_motifs_randesu()`, `igraph_motifs_randesu_no()`,
//! and `igraph_motifs_randesu_callback()` in
//! `references/igraph/src/misc/motifs.c`.
//!
//! Finds all connected induced subgraphs of a given size (the "motifs")
//! and classifies them by isomorphism class using the ESU algorithm
//! (Wernicke & Rasche, Bioinformatics 2006).
//!
//! Supported sizes:
//! - Directed: 3 and 4 vertices (16 and 218 isoclasses)
//! - Undirected: 3, 4, and 5 vertices (4, 11, and 34 isoclasses)

use super::isoclass::tables;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Motif census: count motifs of each isomorphism class.
///
/// Returns a histogram where `hist[c]` is the number of connected induced
/// subgraphs of the given `size` whose isomorphism class is `c`.
/// Disconnected isomorphism classes are reported as `f64::NAN`.
///
/// # Arguments
///
/// * `graph` — the input graph.
/// * `size` — motif size (3 or 4 for directed; 3, 4, or 5 for undirected).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, motifs_randesu};
///
/// // Triangle: one 3-vertex motif of class 3 (complete undirected)
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(0, 2).unwrap();
/// let hist = motifs_randesu(&g, 3).unwrap();
/// assert!((hist[3] - 1.0).abs() < 1e-10);
/// ```
pub fn motifs_randesu(graph: &Graph, size: u32) -> IgraphResult<Vec<f64>> {
    let directed = graph.is_directed();
    let histlen = hist_length(size, directed)?;

    let mut hist = vec![0.0_f64; histlen];

    motifs_randesu_callback(graph, size, |_vids, isoclass| {
        hist[isoclass as usize] += 1.0;
        Ok(true)
    })?;

    let not_connected = not_connected_classes(size, directed);
    for &cls in &not_connected {
        hist[cls] = f64::NAN;
    }

    Ok(hist)
}

/// Count the total number of connected induced subgraphs of a given size.
///
/// Unlike [`motifs_randesu`], this does not classify by isomorphism class
/// and supports arbitrary motif sizes (≥ 3).
///
/// # Arguments
///
/// * `graph` — the input graph.
/// * `size` — motif size (must be ≥ 3).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, motifs_randesu_no};
///
/// // Complete graph K4 has 4 triangles
/// let mut g = Graph::with_vertices(4);
/// for u in 0..4u32 {
///     for v in (u + 1)..4 {
///         g.add_edge(u, v).unwrap();
///     }
/// }
/// assert!((motifs_randesu_no(&g, 3).unwrap() - 4.0).abs() < 1e-10);
/// ```
pub fn motifs_randesu_no(graph: &Graph, size: u32) -> IgraphResult<f64> {
    if size < 3 {
        return Err(IgraphError::InvalidArgument(format!(
            "motifs_randesu_no: size must be at least 3 (got {size})"
        )));
    }

    let n = graph.vcount();
    if n == 0 {
        return Ok(0.0);
    }

    let all_neis = build_all_neighbors(graph)?;
    let mut added = vec![0_i32; n as usize];
    let mut count: f64 = 0.0;

    for parent in 0..n {
        let mut vids: Vec<VertexId> = Vec::new();
        let mut adjverts: Vec<(VertexId, VertexId)> = Vec::new();
        let mut stack: Vec<(VertexId, VertexId, u32)> = Vec::new();

        vids.push(parent);
        added[parent as usize] += 1;
        let mut level: u32 = 1;

        for &nei in &all_neis[parent as usize] {
            if added[nei as usize] == 0 && nei > parent {
                adjverts.push((nei, parent));
            }
            added[nei as usize] += 1;
        }

        while level > 1 || !adjverts.is_empty() {
            if level == size - 1 {
                #[allow(clippy::cast_precision_loss)]
                {
                    count += adjverts.len() as f64;
                }
            }

            if level < size - 1 && !adjverts.is_empty() {
                let (nei, neiparent) = adjverts.pop().unwrap_or((0, 0));

                vids.push(nei);
                added[nei as usize] += 1;
                level += 1;

                stack.push((neiparent, nei, level));

                for &nei2 in &all_neis[nei as usize] {
                    if added[nei2 as usize] == 0 && nei2 > parent {
                        adjverts.push((nei2, nei));
                    }
                    added[nei2 as usize] += 1;
                }
            } else {
                while let Some(&(_, _, stack_level)) = stack.last() {
                    if level == stack_level - 1 {
                        let (neiparent, nei, _) = stack.pop().unwrap_or((0, 0, 0));
                        adjverts.push((nei, neiparent));
                    } else {
                        break;
                    }
                }

                if let Some(nei) = vids.pop() {
                    added[nei as usize] -= 1;
                    level -= 1;
                    for &n2 in &all_neis[nei as usize] {
                        added[n2 as usize] -= 1;
                    }
                    while adjverts.last().is_some_and(|&(_, p)| p == nei) {
                        adjverts.pop();
                    }
                }
            }
        }

        added[parent as usize] -= 1;
        for &nei in &all_neis[parent as usize] {
            added[nei as usize] -= 1;
        }
    }

    Ok(count)
}

/// Enumerate all connected induced subgraphs of the given size and call
/// `callback(vids, isoclass)` for each one.
///
/// The callback returns `Ok(true)` to continue or `Ok(false)` to stop.
///
/// # Arguments
///
/// * `graph` — the input graph.
/// * `size` — motif size (3 or 4 for directed; 3, 4, or 5 for undirected).
/// * `callback` — called with (vertex ids, isomorphism class) for each motif.
pub fn motifs_randesu_callback<F>(graph: &Graph, size: u32, mut callback: F) -> IgraphResult<()>
where
    F: FnMut(&[VertexId], u32) -> IgraphResult<bool>,
{
    let directed = graph.is_directed();
    let (arr_idx, arr_code, mul) = get_isoclass_tables(size, directed)?;

    let n = graph.vcount();
    if n == 0 {
        return Ok(());
    }

    let all_neis = build_all_neighbors(graph)?;
    let out_neis = build_out_neighbors(graph)?;
    let mut added = vec![0_i32; n as usize];
    let mut subg = vec![0_u32; n as usize];

    for parent in 0..n {
        let mut vids: Vec<VertexId> = Vec::new();
        let mut adjverts: Vec<(VertexId, VertexId)> = Vec::new();
        let mut stack: Vec<(VertexId, VertexId, u32)> = Vec::new();

        vids.push(parent);
        subg[parent as usize] = 1;
        added[parent as usize] += 1;
        let mut level: u32 = 1;

        for &nei in &all_neis[parent as usize] {
            if added[nei as usize] == 0 && nei > parent {
                adjverts.push((nei, parent));
            }
            added[nei as usize] += 1;
        }

        let mut terminate = false;

        while level > 1 || !adjverts.is_empty() {
            if level == size - 1 {
                for &(last, _) in &adjverts {
                    vids.push(last);
                    subg[last as usize] = size;

                    let code = compute_isoclass_code(&vids, size, &out_neis, &subg, arr_idx, mul);
                    let isoclass = u32::from(arr_code[code as usize]);

                    match callback(&vids, isoclass) {
                        Ok(true) => {}
                        Ok(false) => {
                            vids.pop();
                            subg[last as usize] = 0;
                            terminate = true;
                            break;
                        }
                        Err(e) => {
                            vids.pop();
                            subg[last as usize] = 0;
                            return Err(e);
                        }
                    }

                    vids.pop();
                    subg[last as usize] = 0;
                }
            }

            if terminate {
                break;
            }

            if level < size - 1 && !adjverts.is_empty() {
                let (nei, neiparent) = adjverts.pop().unwrap_or((0, 0));

                vids.push(nei);
                subg[nei as usize] = level + 1;
                added[nei as usize] += 1;
                level += 1;

                stack.push((neiparent, nei, level));

                for &nei2 in &all_neis[nei as usize] {
                    if added[nei2 as usize] == 0 && nei2 > parent {
                        adjverts.push((nei2, nei));
                    }
                    added[nei2 as usize] += 1;
                }
            } else {
                while let Some(&(_, _, stack_level)) = stack.last() {
                    if level == stack_level - 1 {
                        let (neiparent, nei, _) = stack.pop().unwrap_or((0, 0, 0));
                        adjverts.push((nei, neiparent));
                    } else {
                        break;
                    }
                }

                if let Some(nei) = vids.pop() {
                    subg[nei as usize] = 0;
                    added[nei as usize] -= 1;
                    level -= 1;
                    for &n2 in &all_neis[nei as usize] {
                        added[n2 as usize] -= 1;
                    }
                    while adjverts.last().is_some_and(|&(_, p)| p == nei) {
                        adjverts.pop();
                    }
                }
            }
        }

        if terminate {
            break;
        }

        added[parent as usize] -= 1;
        subg[parent as usize] = 0;
        for &nei in &all_neis[parent as usize] {
            added[nei as usize] -= 1;
        }
    }

    Ok(())
}

fn compute_isoclass_code(
    vids: &[VertexId],
    size: u32,
    out_neis: &[Vec<VertexId>],
    subg: &[u32],
    arr_idx: &[u32],
    mul: u32,
) -> u32 {
    let mut code: u32 = 0;
    for k in 0..size {
        let from = vids[k as usize];
        for &nei in &out_neis[from as usize] {
            let nei_subg = subg[nei as usize];
            if nei_subg != 0 && k != nei_subg - 1 {
                let idx = mul * k + (nei_subg - 1);
                code |= arr_idx[idx as usize];
            }
        }
    }
    code
}

fn get_isoclass_tables(
    size: u32,
    directed: bool,
) -> IgraphResult<(&'static [u32], &'static [u8], u32)> {
    if directed {
        match size {
            3 => Ok((&tables::ISOCLASS_3_IDX, &tables::ISOCLASS2_3, 3)),
            4 => Ok((&tables::ISOCLASS_4_IDX, &tables::ISOCLASS2_4, 4)),
            _ => Err(IgraphError::InvalidArgument(format!(
                "motifs_randesu: directed graphs support size 3 or 4 (got {size})"
            ))),
        }
    } else {
        match size {
            3 => Ok((&tables::ISOCLASS_3U_IDX, &tables::ISOCLASS2_3U, 3)),
            4 => Ok((&tables::ISOCLASS_4U_IDX, &tables::ISOCLASS2_4U, 4)),
            5 => Ok((&tables::ISOCLASS_5U_IDX, &tables::ISOCLASS2_5U, 5)),
            _ => Err(IgraphError::InvalidArgument(format!(
                "motifs_randesu: undirected graphs support size 3, 4, or 5 (got {size})"
            ))),
        }
    }
}

fn hist_length(size: u32, directed: bool) -> IgraphResult<usize> {
    if directed {
        match size {
            3 => Ok(16),
            4 => Ok(218),
            _ => Err(IgraphError::InvalidArgument(format!(
                "motifs_randesu: directed graphs support size 3 or 4 (got {size})"
            ))),
        }
    } else {
        match size {
            3 => Ok(4),
            4 => Ok(11),
            5 => Ok(34),
            _ => Err(IgraphError::InvalidArgument(format!(
                "motifs_randesu: undirected graphs support size 3, 4, or 5 (got {size})"
            ))),
        }
    }
}

fn not_connected_classes(size: u32, directed: bool) -> Vec<usize> {
    if size == 3 {
        if directed { vec![0, 1, 3] } else { vec![0, 1] }
    } else if size == 4 {
        if directed {
            vec![
                0, 1, 2, 4, 5, 6, 9, 10, 11, 15, 22, 23, 27, 28, 33, 34, 39, 62, 120,
            ]
        } else {
            vec![0, 1, 2, 3, 5]
        }
    } else if size == 5 {
        vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 19]
    } else {
        vec![]
    }
}

fn build_all_neighbors(graph: &Graph) -> IgraphResult<Vec<Vec<VertexId>>> {
    let n = graph.vcount() as usize;
    let ecount = graph.ecount();
    let mut adj: Vec<Vec<VertexId>> = vec![Vec::new(); n];

    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (src, tgt) = graph.edge(eid as u32)?;
        if src == tgt {
            continue;
        }
        adj[src as usize].push(tgt);
        adj[tgt as usize].push(src);
    }

    for neighbors in &mut adj {
        neighbors.sort_unstable();
        neighbors.dedup();
    }

    Ok(adj)
}

fn build_out_neighbors(graph: &Graph) -> IgraphResult<Vec<Vec<VertexId>>> {
    let n = graph.vcount() as usize;
    let ecount = graph.ecount();
    let mut adj: Vec<Vec<VertexId>> = vec![Vec::new(); n];

    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (src, tgt) = graph.edge(eid as u32)?;
        if src == tgt {
            continue;
        }
        adj[src as usize].push(tgt);
        if !graph.is_directed() {
            adj[tgt as usize].push(src);
        }
    }

    for neighbors in &mut adj {
        neighbors.sort_unstable();
    }

    Ok(adj)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn motifs_randesu_empty_graph() {
        let g = Graph::with_vertices(0);
        let hist = motifs_randesu(&g, 3).unwrap();
        assert_eq!(hist.len(), 4);
        assert!(hist[0].is_nan());
        assert!(hist[1].is_nan());
        assert!((hist[2]).abs() < 1e-10);
        assert!((hist[3]).abs() < 1e-10);
    }

    #[test]
    fn motifs_randesu_triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        let hist = motifs_randesu(&g, 3).unwrap();
        assert!((hist[3] - 1.0).abs() < 1e-10);
        assert!((hist[2]).abs() < 1e-10);
    }

    #[test]
    fn motifs_randesu_path_3() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let hist = motifs_randesu(&g, 3).unwrap();
        assert!((hist[2] - 1.0).abs() < 1e-10);
        assert!((hist[3]).abs() < 1e-10);
    }

    #[test]
    fn motifs_randesu_k4_size3() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let hist = motifs_randesu(&g, 3).unwrap();
        assert!((hist[3] - 4.0).abs() < 1e-10);
    }

    #[test]
    fn motifs_randesu_k4_size4() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let hist = motifs_randesu(&g, 4).unwrap();
        assert_eq!(hist.len(), 11);
        assert!((hist[10] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn motifs_randesu_star_4() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        let hist = motifs_randesu(&g, 3).unwrap();
        assert!((hist[2] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn motifs_randesu_no_triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        let count = motifs_randesu_no(&g, 3).unwrap();
        assert!((count - 1.0).abs() < 1e-10);
    }

    #[test]
    fn motifs_randesu_no_k4() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let count = motifs_randesu_no(&g, 3).unwrap();
        assert!((count - 4.0).abs() < 1e-10);
    }

    #[test]
    fn motifs_randesu_no_k5_size3() {
        let mut g = Graph::with_vertices(5);
        for u in 0..5u32 {
            for v in (u + 1)..5 {
                g.add_edge(u, v).unwrap();
            }
        }
        let count = motifs_randesu_no(&g, 3).unwrap();
        // C(5,3) = 10 triangles
        assert!((count - 10.0).abs() < 1e-10);
    }

    #[test]
    fn motifs_randesu_no_k5_size4() {
        let mut g = Graph::with_vertices(5);
        for u in 0..5u32 {
            for v in (u + 1)..5 {
                g.add_edge(u, v).unwrap();
            }
        }
        let count = motifs_randesu_no(&g, 4).unwrap();
        // C(5,4) = 5
        assert!((count - 5.0).abs() < 1e-10);
    }

    #[test]
    fn motifs_randesu_directed_3_cycle() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let hist = motifs_randesu(&g, 3).unwrap();
        assert_eq!(hist.len(), 16);
        // 3-cycle in directed graph: isoclass 7
        let total: f64 = hist.iter().filter(|x| !x.is_nan()).sum();
        assert!((total - 1.0).abs() < 1e-10);
    }

    #[test]
    fn motifs_randesu_directed_mutual() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(1, 2).unwrap();
        let hist = motifs_randesu(&g, 3).unwrap();
        let total: f64 = hist.iter().filter(|x| !x.is_nan()).sum();
        assert!((total - 1.0).abs() < 1e-10);
    }

    #[test]
    fn motifs_randesu_callback_early_stop() {
        let mut g = Graph::with_vertices(5);
        for u in 0..5u32 {
            for v in (u + 1)..5 {
                g.add_edge(u, v).unwrap();
            }
        }
        let mut count = 0;
        motifs_randesu_callback(&g, 3, |_vids, _cls| {
            count += 1;
            Ok(count < 3)
        })
        .unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn motifs_randesu_no_empty() {
        let g = Graph::with_vertices(0);
        assert!((motifs_randesu_no(&g, 3).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn motifs_randesu_no_small_graph() {
        let g = Graph::with_vertices(2);
        assert!((motifs_randesu_no(&g, 3).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn motifs_randesu_invalid_size() {
        let g = Graph::with_vertices(3);
        assert!(motifs_randesu(&g, 2).is_err());
        assert!(motifs_randesu_no(&g, 2).is_err());
    }

    #[test]
    fn motifs_randesu_no_path_5_size3() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        let count = motifs_randesu_no(&g, 3).unwrap();
        // Connected induced subgraphs of size 3 on a path of 5:
        // {0,1,2}, {1,2,3}, {2,3,4} = 3
        assert!((count - 3.0).abs() < 1e-10);
    }

    #[test]
    fn motifs_randesu_hist_matches_no() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();

        let hist = motifs_randesu(&g, 3).unwrap();
        let total_from_hist: f64 = hist.iter().filter(|x| !x.is_nan()).sum();
        let total_from_no = motifs_randesu_no(&g, 3).unwrap();
        assert!(
            (total_from_hist - total_from_no).abs() < 1e-10,
            "hist sum {total_from_hist} != no {total_from_no}"
        );
    }

    #[test]
    fn motifs_randesu_size5_k5() {
        let mut g = Graph::with_vertices(5);
        for u in 0..5u32 {
            for v in (u + 1)..5 {
                g.add_edge(u, v).unwrap();
            }
        }
        let hist = motifs_randesu(&g, 5).unwrap();
        assert_eq!(hist.len(), 34);
        // K5: one motif of class 33 (complete graph)
        assert!((hist[33] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn motifs_randesu_directed_4() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        let hist = motifs_randesu(&g, 4).unwrap();
        assert_eq!(hist.len(), 218);
        let total: f64 = hist.iter().filter(|x| !x.is_nan()).sum();
        assert!((total - 1.0).abs() < 1e-10);
    }

    #[test]
    fn motifs_randesu_self_loops_ignored() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        let hist = motifs_randesu(&g, 3).unwrap();
        assert!((hist[3] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn motifs_randesu_disconnected_graph() {
        // Two isolated edges: 0-1 and 2-3
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let hist = motifs_randesu(&g, 3).unwrap();
        // Only connected subgraphs counted; none exist
        let total: f64 = hist.iter().filter(|x| !x.is_nan()).sum();
        assert!((total).abs() < 1e-10);
    }
}

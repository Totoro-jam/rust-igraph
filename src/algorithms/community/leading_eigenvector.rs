//! Leading eigenvector community detection (ALGO-CO-017).
//!
//! Newman's leading eigenvector method: recursively bisect the graph
//! by the sign of the leading eigenvector of the modularity matrix
//! **B** = **A** − **k** **k**^T / (2m), restricted to each community.
//!
//! Reference: MEJ Newman, "Finding community structure in networks
//! using the eigenvectors of matrices", Phys Rev E 74:036104 (2006).
//!
//! Counterpart of `igraph_community_leading_eigenvector()` from
//! `references/igraph/src/community/leading_eigenvector.c:331`.

use std::collections::VecDeque;

use crate::algorithms::community::lanczos::lanczos_largest;
use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult};

/// Result of the leading eigenvector community detection.
#[derive(Debug, Clone)]
pub struct LeadingEigenvectorResult {
    /// Community assignment for each vertex.
    pub membership: Vec<u32>,
    /// Eigenvalue found at each split step (positive means a split
    /// occurred; non-positive means the split was rejected).
    pub eigenvalues: Vec<f64>,
    /// Merges matrix (same format as dendrogram merges): each entry
    /// `(a, b)` means communities `a` and `b` were merged (in reverse
    /// of the split order). Use with `le_community_to_membership`.
    pub merges: Vec<(u32, u32)>,
    /// Final modularity of the partition.
    pub modularity: f64,
}

/// Detect communities using Newman's leading eigenvector method.
///
/// Repeatedly bisects communities by the sign of the leading
/// eigenvector of the restricted modularity matrix. Stops when no
/// further split increases modularity, or after `steps` splits.
///
/// Edge directions are ignored (the graph is treated as undirected).
///
/// `steps`: maximum number of split steps. Pass `None` for
/// `vcount - 1` (exhaustive).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, leading_eigenvector};
///
/// // Barbell graph: two triangles connected by a bridge
/// let mut g = Graph::with_vertices(6);
/// // Triangle 1
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(0, 2).unwrap();
/// // Triangle 2
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(4, 5).unwrap();
/// g.add_edge(3, 5).unwrap();
/// // Bridge
/// g.add_edge(2, 3).unwrap();
///
/// let result = leading_eigenvector(&g, None, None).unwrap();
/// // Should find 2 communities (one per triangle)
/// assert_eq!(result.membership[0], result.membership[1]);
/// assert_eq!(result.membership[3], result.membership[4]);
/// assert_ne!(result.membership[0], result.membership[3]);
/// ```
#[allow(clippy::too_many_lines)]
pub fn leading_eigenvector(
    graph: &Graph,
    weights: Option<&[f64]>,
    steps: Option<u32>,
) -> IgraphResult<LeadingEigenvectorResult> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(LeadingEigenvectorResult {
            membership: Vec::new(),
            eigenvalues: Vec::new(),
            merges: Vec::new(),
            modularity: 0.0,
        });
    }

    let ecount = graph.ecount();
    if let Some(w) = weights {
        if w.len() != ecount {
            return Err(IgraphError::InvalidArgument(format!(
                "weights length ({}) differs from edge count ({ecount})",
                w.len()
            )));
        }
        for &wv in w {
            if !wv.is_finite() {
                return Err(IgraphError::InvalidArgument(
                    "edge weights must be finite".to_string(),
                ));
            }
        }
    }

    let max_steps = match steps {
        Some(s) => s as usize,
        None => {
            if n > 0 {
                n - 1
            } else {
                0
            }
        }
    };

    // Build adjacency list (treat as undirected: include both directions)
    let adj = build_adjacency(graph);

    // Degree / strength per vertex
    let (deg_or_strength, two_m) = compute_degrees(graph, weights, &adj);

    if two_m <= 0.0 {
        return Ok(LeadingEigenvectorResult {
            membership: vec![0; n],
            eigenvalues: Vec::new(),
            merges: Vec::new(),
            modularity: 0.0,
        });
    }

    // Start from weakly connected components
    let cc = crate::algorithms::connectivity::components::connected_components(graph)?;
    let mut membership: Vec<u32> = cc.membership.clone();
    let mut communities = cc.count;

    let mut eigenvalues_out = Vec::new();
    let mut merges_out = Vec::new();

    // Queue of communities to try splitting (depth-first via stack)
    let mut to_split: VecDeque<u32> = VecDeque::new();
    for c in 0..communities {
        let size = membership.iter().filter(|&&m| m == c).count();
        if size > 2 {
            to_split.push_back(c);
        }
    }

    // Record initial component merges (mirrors upstream)
    for c in 1..communities {
        merges_out.push((c - 1, c));
        eigenvalues_out.push(f64::NAN);
    }
    let mut steps_taken = (communities as usize).saturating_sub(1);

    let mut rng = SplitMix64::new(42);

    while let Some(comm) = to_split.pop_back() {
        if steps_taken >= max_steps {
            break;
        }

        // Collect vertex indices in this community
        let idx: Vec<usize> = (0..n).filter(|&i| membership[i] == comm).collect();
        let size = idx.len();

        steps_taken += 1;

        if size <= 2 {
            continue;
        }

        // Build reverse mapping: vertex -> position in community
        let mut idx2 = vec![0usize; n];
        for (pos, &v) in idx.iter().enumerate() {
            idx2[v] = pos;
        }

        // The modularity matrix-vector product for this community
        let matvec = |x: &[f64], y: &mut [f64]| {
            modularity_matvec(
                graph,
                weights,
                &adj,
                &deg_or_strength,
                two_m,
                &membership,
                comm,
                &idx,
                &idx2,
                x,
                y,
            );
        };

        // Random start vector: alternating ±1 with small perturbation
        let mut start_vec = vec![0.0_f64; size];
        for (i, sv) in start_vec.iter_mut().enumerate() {
            let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
            let perturb = (rng.gen_unit() - 0.5) * 0.2;
            *sv = sign + perturb;
        }
        // Shuffle
        for i in (1..size).rev() {
            let j = rng.gen_index(i + 1);
            start_vec.swap(i, j);
        }

        let result = lanczos_largest(size, &matvec, &mut start_vec, 10000);

        // Clean up small numerical noise
        let eigenvalue = if result.eigenvalue.abs() < 1e-8 {
            0.0
        } else {
            result.eigenvalue
        };

        let mut eigvec = result.eigenvector;
        for v in &mut eigvec {
            if v.abs() < 1e-8 {
                *v = 0.0;
            }
        }

        // Normalize sign: first nonzero element should be positive
        if let Some(first_nonzero) = eigvec.iter().find(|&&v| v != 0.0) {
            if *first_nonzero < 0.0 {
                for v in &mut eigvec {
                    *v = -*v;
                }
            }
        }

        eigenvalues_out.push(eigenvalue);

        if eigenvalue <= 0.0 {
            continue;
        }

        // Count vertices in each side of the split
        let neg_count = eigvec.iter().filter(|&&v| v < 0.0).count();
        if neg_count == 0 || neg_count == size {
            continue;
        }

        // Verify that the split actually increases modularity
        // by computing x^T B x for the sign vector
        let mut sign_vec = vec![0.0_f64; size];
        for (i, &v) in eigvec.iter().enumerate() {
            sign_vec[i] = if v < 0.0 { -1.0 } else { 1.0 };
        }
        let mut bx = vec![0.0_f64; size];
        matvec(&sign_vec, &mut bx);
        let mod_increase: f64 = sign_vec.iter().zip(bx.iter()).map(|(s, b)| s * b).sum();
        if mod_increase <= 1e-8 {
            continue;
        }

        // Perform the split
        let new_comm = communities;
        communities += 1;

        for (j, &v) in eigvec.iter().enumerate() {
            if v < 0.0 {
                membership[idx[j]] = new_comm as u32;
            }
        }

        merges_out.push((comm, new_comm as u32));

        // Queue sub-communities for further splitting
        let pos_count = size - neg_count;
        if neg_count > 1 {
            to_split.push_back(new_comm as u32);
        }
        if pos_count > 1 {
            to_split.push_back(comm);
        }
    }

    // Compute final modularity
    let mod_val = compute_modularity(graph, weights, &membership, &deg_or_strength, two_m);

    Ok(LeadingEigenvectorResult {
        membership,
        eigenvalues: eigenvalues_out,
        merges: merges_out,
        modularity: mod_val,
    })
}

/// Weighted leading eigenvector community detection (convenience wrapper).
///
/// Equivalent to `leading_eigenvector(graph, Some(weights), steps)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, leading_eigenvector_weighted};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(0, 3).unwrap();
/// let w = vec![1.0; 4];
/// let r = leading_eigenvector_weighted(&g, &w, None).unwrap();
/// assert_eq!(r.membership.len(), 4);
/// ```
pub fn leading_eigenvector_weighted(
    graph: &Graph,
    weights: &[f64],
    steps: Option<u32>,
) -> IgraphResult<LeadingEigenvectorResult> {
    leading_eigenvector(graph, Some(weights), steps)
}

// ─── Internal helpers ────────────────────────────────────────────

type AdjList = Vec<Vec<(usize, usize)>>; // adj[v] = [(neighbor, edge_id), ...]

fn build_adjacency(graph: &Graph) -> AdjList {
    let n = graph.vcount() as usize;
    let mut adj: AdjList = vec![Vec::new(); n];
    for eid in 0..graph.ecount() {
        #[allow(clippy::cast_possible_truncation)]
        let eid32 = eid as u32;
        let Ok(s) = graph.edge_source(eid32) else {
            continue;
        };
        let Ok(t) = graph.edge_target(eid32) else {
            continue;
        };
        let s = s as usize;
        let t = t as usize;
        adj[s].push((t, eid));
        if s != t {
            adj[t].push((s, eid));
        }
    }
    adj
}

#[allow(clippy::cast_precision_loss)]
fn compute_degrees(graph: &Graph, weights: Option<&[f64]>, adj: &AdjList) -> (Vec<f64>, f64) {
    let n = graph.vcount() as usize;
    let mut deg = vec![0.0_f64; n];
    let mut total = 0.0_f64;

    match weights {
        None => {
            for v in 0..n {
                let d = adj[v].len() as f64;
                deg[v] = d;
                total += d;
            }
        }
        Some(w) => {
            for v in 0..n {
                let mut s = 0.0_f64;
                for &(_, eid) in &adj[v] {
                    s += w[eid];
                }
                deg[v] = s;
                total += s;
            }
        }
    }

    (deg, total)
}

/// Compute `y = B_comm * x` where `B_comm` is the modularity matrix
/// restricted to community `comm`.
///
/// `B_ij = A_ij - k_i * k_j / (2m)`, restricted to vertices in `comm`,
/// with diagonal correction to make `B_comm` have zero row sums within
/// the community (the "generalized modularity matrix" from Newman 2006).
#[allow(clippy::too_many_arguments)]
fn modularity_matvec(
    _graph: &Graph,
    weights: Option<&[f64]>,
    adj: &AdjList,
    deg: &[f64],
    two_m: f64,
    membership: &[u32],
    comm: u32,
    idx: &[usize],
    idx2: &[usize],
    x: &[f64],
    y: &mut [f64],
) {
    let size = idx.len();
    let inv_2m = 1.0 / two_m;
    let mut tmp = vec![0.0_f64; size];

    // Step 1: Ax (adjacency restricted to community)
    for j in 0..size {
        let v = idx[j];
        y[j] = 0.0;
        tmp[j] = 0.0;
        for &(nei, eid) in &adj[v] {
            if membership[nei] == comm {
                let w = match weights {
                    Some(wt) => wt[eid],
                    None => 1.0,
                };
                y[j] += x[idx2[nei]] * w;
                tmp[j] += w;
            }
        }
    }

    // Step 2: k^T x / (2m)
    let mut ktx = 0.0_f64;
    let mut ktx2 = 0.0_f64;
    for j in 0..size {
        let v = idx[j];
        ktx += x[j] * deg[v];
        ktx2 += deg[v];
    }
    ktx *= inv_2m;
    ktx2 *= inv_2m;

    // Step 3: Bx = Ax - k * (k^T x) / (2m)
    for j in 0..size {
        let v = idx[j];
        y[j] -= ktx * deg[v];
        tmp[j] -= ktx2 * deg[v];
    }

    // Step 4: diagonal correction -d_ij * sum_l B_il
    // (ensures B_comm has zero row sums within the community)
    for j in 0..size {
        y[j] -= tmp[j] * x[j];
    }
}

fn compute_modularity(
    graph: &Graph,
    weights: Option<&[f64]>,
    membership: &[u32],
    deg: &[f64],
    two_m: f64,
) -> f64 {
    if two_m <= 0.0 {
        return 0.0;
    }
    let inv_2m = 1.0 / two_m;
    let mut q = 0.0_f64;

    for eid in 0..graph.ecount() {
        #[allow(clippy::cast_possible_truncation)]
        let eid32 = eid as u32;
        let Ok(s) = graph.edge_source(eid32) else {
            continue;
        };
        let Ok(t) = graph.edge_target(eid32) else {
            continue;
        };
        let s = s as usize;
        let t = t as usize;
        if membership[s] == membership[t] {
            let w = match weights {
                Some(wt) => wt[eid],
                None => 1.0,
            };
            if s == t {
                q += w - deg[s] * deg[t] * inv_2m;
            } else {
                q += 2.0 * (w - deg[s] * deg[t] * inv_2m);
            }
        }
    }

    q * inv_2m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let r = leading_eigenvector(&g, None, None).unwrap();
        assert!(r.membership.is_empty());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        let r = leading_eigenvector(&g, None, None).unwrap();
        assert_eq!(r.membership, vec![0]);
    }

    #[test]
    fn two_components() {
        // Two disconnected edges
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let r = leading_eigenvector(&g, None, None).unwrap();
        assert_eq!(r.membership[0], r.membership[1]);
        assert_eq!(r.membership[2], r.membership[3]);
        assert_ne!(r.membership[0], r.membership[2]);
    }

    #[test]
    fn barbell_finds_two_communities() {
        let mut g = Graph::with_vertices(6);
        // Triangle 1
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        // Triangle 2
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(3, 5).unwrap();
        // Bridge
        g.add_edge(2, 3).unwrap();

        let r = leading_eigenvector(&g, None, None).unwrap();
        // Same community within each triangle
        assert_eq!(r.membership[0], r.membership[1]);
        assert_eq!(r.membership[0], r.membership[2]);
        assert_eq!(r.membership[3], r.membership[4]);
        assert_eq!(r.membership[3], r.membership[5]);
        // Different communities across triangles
        assert_ne!(r.membership[0], r.membership[3]);
    }

    #[test]
    fn complete_graph_no_split() {
        // K5: no meaningful split possible
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        let r = leading_eigenvector(&g, None, None).unwrap();
        // All vertices should be in the same community
        let c = r.membership[0];
        for &m in &r.membership {
            assert_eq!(m, c, "K5 should not be split");
        }
    }

    #[test]
    fn weighted_barbell() {
        let mut g = Graph::with_vertices(6);
        // Triangle 1 with heavy edges
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        // Triangle 2 with heavy edges
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(3, 5).unwrap();
        // Weak bridge
        g.add_edge(2, 3).unwrap();

        let weights = vec![5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 0.1];
        let r = leading_eigenvector(&g, Some(&weights), None).unwrap();
        assert_eq!(r.membership[0], r.membership[1]);
        assert_eq!(r.membership[3], r.membership[4]);
        assert_ne!(r.membership[0], r.membership[3]);
    }

    #[test]
    fn steps_limit() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(3, 5).unwrap();
        g.add_edge(2, 3).unwrap();

        // With 0 steps, should get initial component assignment
        let r = leading_eigenvector(&g, None, Some(0)).unwrap();
        // All in one component initially
        let c = r.membership[0];
        for &m in &r.membership {
            assert_eq!(m, c);
        }
    }

    #[test]
    fn modularity_is_positive_for_good_split() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(3, 5).unwrap();
        g.add_edge(2, 3).unwrap();

        let r = leading_eigenvector(&g, None, None).unwrap();
        assert!(
            r.modularity > 0.0,
            "modularity should be positive for barbell, got {}",
            r.modularity
        );
    }
}

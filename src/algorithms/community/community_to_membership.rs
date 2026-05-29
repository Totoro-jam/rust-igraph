//! ALGO-CM-013 — `community_to_membership` (dendrogram → membership).
//!
//! Cut a binary dendrogram (the same `merges` matrix produced by
//! [`crate::walktrap`], [`crate::fast_greedy_modularity`],
//! [`crate::edge_betweenness_community`], …) at a chosen number of
//! merges. Returns a membership vector (per-vertex cluster id) and a
//! cluster-size vector.
//!
//! Mirrors `igraph_community_to_membership` from
//! `references/igraph/src/community/community_misc.c`. The C function
//! traverses the requested `steps` from the bottom up, assigning a
//! cluster id to every leaf reachable from each top-level surviving
//! dendrogram node; leaves never touched stay as singleton clusters.
//!
//! Also provides [`le_community_to_membership`], the generalized form
//! whose dendrogram leaves are arbitrary initial clusters rather than
//! individual vertices (mirrors `igraph_le_community_to_membership`).

use crate::core::error::{IgraphError, IgraphResult};

/// Result of [`community_to_membership`]: per-vertex cluster ids
/// densified to `0..k`, plus the size of each cluster in the same
/// order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommunityToMembershipResult {
    /// `membership[v] ∈ [0, k)` — the cluster id of vertex `v` after
    /// `steps` merges have been applied.
    pub membership: Vec<u32>,
    /// `csize[c]` — the number of vertices in cluster `c`.
    pub csize: Vec<u32>,
}

/// Cut a binary dendrogram at `steps` merges, returning the resulting
/// membership and cluster-size vectors.
///
/// `merges[i] = [c1, c2]` is the `i`-th merge: it combines dendrogram
/// nodes `c1` and `c2` into the new node with id `nodes + i`. Leaves
/// are numbered `0..nodes`; nodes `≥ nodes` are internal merges from
/// earlier rows.
///
/// # Errors
/// - `InvalidArgument` if `steps > merges.len()`.
/// - `InvalidArgument` if a merge entry would be merged twice.
/// - `InvalidArgument` if a merge entry references a dendrogram node
///   that does not exist (`≥ nodes + i` at row `i`).
///
/// # Examples
/// ```
/// use rust_igraph::community_to_membership;
///
/// // Four leaves, one merge: {0,1} | {2} | {3}.
/// let r = community_to_membership(&[[0, 1]], 4, 1).unwrap();
/// assert_eq!(r.membership, vec![0, 0, 1, 2]);
/// assert_eq!(r.csize, vec![2, 1, 1]);
/// ```
pub fn community_to_membership(
    merges: &[[u32; 2]],
    nodes: u32,
    steps: u32,
) -> IgraphResult<CommunityToMembershipResult> {
    let n = nodes as usize;
    let rows = merges.len();
    let steps_usize = steps as usize;

    if steps_usize > rows {
        return Err(IgraphError::InvalidArgument(format!(
            "Number of steps is greater than number of rows in merges matrix: \
             found {steps} steps, {rows} rows."
        )));
    }

    // `tmp[i]` is the 1-based cluster id assigned to the supercluster
    // formed at merge row `i` (0 = unassigned). Mirrors the C version's
    // 1-based scheme so the final densification pass can detect "never
    // touched" leaves via 0.
    let mut membership = vec![0u32; n];
    // The maximum dendrogram-node id we may encounter is `n + steps - 1`
    // (row `steps - 1` produces id `n + steps - 1`). The C version
    // allocates `steps + nodes` slots — same here.
    let total_nodes = n
        .checked_add(steps_usize)
        .ok_or_else(|| IgraphError::InvalidArgument("dendrogram size overflows usize".into()))?;
    let mut already_merged = vec![false; total_nodes];
    let mut tmp = vec![0u32; steps_usize];

    let mut found: u32 = 0;

    // Walk the requested steps from the top down so that the cluster
    // id assigned at row `i` cascades to every leaf merged into it.
    for i in (0..steps_usize).rev() {
        let [c1, c2] = merges[i];
        let limit = u32::try_from(n + i).map_err(|_| {
            IgraphError::InvalidArgument(format!(
                "Merges matrix row {i} dendrogram node id overflows u32."
            ))
        })?;
        if c1 >= limit {
            return Err(IgraphError::InvalidArgument(format!(
                "Merges matrix row {i}: child {c1} exceeds the maximum valid \
                 dendrogram node id ({} = nodes + i).",
                limit - 1
            )));
        }
        if c2 >= limit {
            return Err(IgraphError::InvalidArgument(format!(
                "Merges matrix row {i}: child {c2} exceeds the maximum valid \
                 dendrogram node id ({} = nodes + i).",
                limit - 1
            )));
        }
        let c1_idx = c1 as usize;
        let c2_idx = c2 as usize;
        if already_merged[c1_idx] {
            return Err(IgraphError::InvalidArgument(format!(
                "Merges matrix contains multiple merges of cluster {c1}."
            )));
        }
        already_merged[c1_idx] = true;
        if already_merged[c2_idx] {
            return Err(IgraphError::InvalidArgument(format!(
                "Merges matrix contains multiple merges of cluster {c2}."
            )));
        }
        already_merged[c2_idx] = true;

        // New supercluster? Allocate a fresh 1-based id.
        if tmp[i] == 0 {
            found += 1;
            tmp[i] = found;
        }
        let cid_one_based = tmp[i];

        for &child in &[c1, c2] {
            if (child as usize) < n {
                membership[child as usize] = cid_one_based;
            } else {
                // Internal node — propagate the supercluster id down.
                tmp[child as usize - n] = cid_one_based;
            }
        }
    }

    // Densify: leaves with cid == 0 stay as singletons; every other leaf
    // becomes cid - 1.
    let total_clusters_estimate = (n - usize::min(n, 2 * steps_usize)) + found as usize;
    let mut csize: Vec<u32> = vec![0; found as usize];
    csize.reserve(total_clusters_estimate);

    for slot in &mut membership {
        if *slot == 0 {
            *slot = found;
            csize.push(1);
            found += 1;
        } else {
            let cid = *slot - 1;
            *slot = cid;
            csize[cid as usize] += 1;
        }
    }

    Ok(CommunityToMembershipResult { membership, csize })
}

/// Cut an incomplete dendrogram after `steps` merges, starting from an
/// initial (non-singleton) cluster assignment.
///
/// This is the more general form of [`community_to_membership`]: instead
/// of assuming each dendrogram leaf is a single vertex, the leaves are
/// the `m` contiguous clusters described by `membership` (so
/// `membership[v] ∈ [0, m)`). The `merges` matrix then merges those
/// clusters, and `steps` of those merges are applied. The output
/// `membership` reassigns every vertex to its surviving cluster, and
/// `csize` gives the vertex count of each.
///
/// This dendrogram shape is produced by divisive detectors that stop
/// before splitting the graph down to individual vertices, such as
/// igraph's leading-eigenvector method.
///
/// Mirrors `igraph_le_community_to_membership` from
/// `references/igraph/src/community/leading_eigenvector.c`.
///
/// # Errors
/// - `InvalidArgument` if `membership` is empty, or `steps >= m` where
///   `m` is the number of initial clusters.
/// - `InvalidArgument` if `membership` is not a contiguous `0..m`
///   labelling (some cluster index in that range is unused).
/// - Any error propagated from [`community_to_membership`] when applying
///   the merges to the `m` initial clusters.
///
/// # Examples
/// ```
/// use rust_igraph::le_community_to_membership;
///
/// // Six vertices in three initial clusters: {0,1}=0, {2,3}=1, {4,5}=2.
/// // One merge combines clusters 0 and 1.
/// let membership = [0, 0, 1, 1, 2, 2];
/// let r = le_community_to_membership(&[[0, 1]], 1, &membership).unwrap();
/// // Clusters 0 and 1 fuse; cluster 2 stays separate.
/// assert_eq!(r.membership, vec![0, 0, 0, 0, 1, 1]);
/// assert_eq!(r.csize, vec![4, 2]);
/// ```
pub fn le_community_to_membership(
    merges: &[[u32; 2]],
    steps: u32,
    membership: &[u32],
) -> IgraphResult<CommunityToMembershipResult> {
    let no_of_nodes = membership.len();

    if no_of_nodes == 0 {
        return Err(IgraphError::InvalidArgument(
            "le_community_to_membership: membership vector must not be empty.".into(),
        ));
    }

    let components = membership
        .iter()
        .copied()
        .max()
        .map_or(0u32, |m| m.saturating_add(1));

    if components as usize > no_of_nodes {
        return Err(IgraphError::InvalidArgument(format!(
            "Invalid membership vector: number of components ({components}) must not be \
             greater than the number of nodes ({no_of_nodes})."
        )));
    }

    if steps >= components {
        return Err(IgraphError::InvalidArgument(format!(
            "Number of steps ({steps}) must be smaller than number of components ({components})."
        )));
    }

    // Verify the initial assignment uses every cluster id in `0..components`
    // (no empty cluster), mirroring the C reference's validation.
    let mut seen = vec![false; components as usize];
    for &c in membership {
        seen[c as usize] = true;
    }
    if let Some(empty) = seen.iter().position(|&s| !s) {
        return Err(IgraphError::InvalidArgument(format!(
            "Invalid membership vector, empty cluster found at id {empty}."
        )));
    }

    // Merge the `components` initial clusters as if they were leaves.
    let comp = community_to_membership(merges, components, steps)?;

    // Remap each vertex from its old cluster to its new merged cluster.
    let new_cluster_count = comp.csize.len();
    let mut new_membership = vec![0u32; no_of_nodes];
    let mut csize = vec![0u32; new_cluster_count];
    for (i, &old_cluster) in membership.iter().enumerate() {
        let new_cluster = comp.membership[old_cluster as usize];
        new_membership[i] = new_cluster;
        csize[new_cluster as usize] += 1;
    }

    Ok(CommunityToMembershipResult {
        membership: new_membership,
        csize,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_steps_yields_singletons() {
        let r = community_to_membership(&[], 4, 0).unwrap();
        assert_eq!(r.membership, vec![0, 1, 2, 3]);
        assert_eq!(r.csize, vec![1, 1, 1, 1]);
    }

    #[test]
    fn full_collapse_to_one_cluster() {
        // Binary balanced dendrogram on 4 leaves:
        //   merges[0] = [0, 1] -> 4
        //   merges[1] = [2, 3] -> 5
        //   merges[2] = [4, 5] -> 6
        let merges = vec![[0u32, 1], [2, 3], [4, 5]];
        let r = community_to_membership(&merges, 4, 3).unwrap();
        assert_eq!(r.membership, vec![0, 0, 0, 0]);
        assert_eq!(r.csize, vec![4]);
    }

    #[test]
    fn intermediate_cut_two_clusters() {
        // Same dendrogram cut at 2 steps: {0,1} and {2,3} survive.
        let merges = vec![[0u32, 1], [2, 3], [4, 5]];
        let r = community_to_membership(&merges, 4, 2).unwrap();
        // The C reference numbers clusters in the order it first
        // touches them while walking merges from the top down — so the
        // most recent merge (row 1 = [2, 3]) gets cluster 0, the
        // earlier merge (row 0 = [0, 1]) gets cluster 1.
        assert_eq!(r.membership, vec![1, 1, 0, 0]);
        assert_eq!(r.csize, vec![2, 2]);
    }

    #[test]
    fn one_merge_with_untouched_leaves() {
        // 5 leaves, one merge of {0, 2} -> cluster A; 1, 3, 4 stay
        // singletons.
        let r = community_to_membership(&[[0u32, 2]], 5, 1).unwrap();
        // Touched leaves come first (cluster 0); untouched leaves get
        // ids in encounter order: 1 -> 1, 3 -> 2, 4 -> 3.
        assert_eq!(r.membership, vec![0, 1, 0, 2, 3]);
        assert_eq!(r.csize, vec![2, 1, 1, 1]);
    }

    #[test]
    fn chained_merges_three_levels() {
        // Linear chain: ((0,1)->4, (4,2)->5, (5,3)->6) on 4 leaves.
        // Cut at 3 merges -> one cluster of all 4 leaves.
        let merges = vec![[0u32, 1], [4, 2], [5, 3]];
        let r = community_to_membership(&merges, 4, 3).unwrap();
        assert_eq!(r.membership, vec![0, 0, 0, 0]);
        assert_eq!(r.csize, vec![4]);
    }

    #[test]
    fn empty_graph_zero_steps() {
        let r = community_to_membership(&[], 0, 0).unwrap();
        assert!(r.membership.is_empty());
        assert!(r.csize.is_empty());
    }

    #[test]
    fn err_steps_exceeds_rows() {
        let err = community_to_membership(&[[0u32, 1]], 4, 2).unwrap_err();
        match err {
            IgraphError::InvalidArgument(m) => assert!(m.contains("greater than")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn err_double_merge_of_same_cluster() {
        // Bogus dendrogram: leaf 0 is "merged" twice.
        let merges = vec![[0u32, 1], [0, 2]];
        let err = community_to_membership(&merges, 4, 2).unwrap_err();
        match err {
            IgraphError::InvalidArgument(m) => assert!(m.contains("multiple merges")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn err_merge_entry_references_future_node() {
        // Row 0 may only reference leaves 0..n (id < n + 0 = n). Here
        // it points at n + 0 = 4, which is the node row 0 itself
        // creates — invalid.
        let merges = vec![[0u32, 4]];
        let err = community_to_membership(&merges, 4, 1).unwrap_err();
        match err {
            IgraphError::InvalidArgument(m) => assert!(m.contains("exceeds")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn partial_dendrogram_with_fewer_steps_than_rows() {
        // Three merges given but only cut at 1.
        let merges = vec![[0u32, 1], [2, 3], [4, 5]];
        let r = community_to_membership(&merges, 4, 1).unwrap();
        assert_eq!(r.membership, vec![0, 0, 1, 2]);
        assert_eq!(r.csize, vec![2, 1, 1]);
    }

    #[test]
    fn matches_walktrap_dendrogram_on_two_k4_bridge() {
        // End-to-end: drive Walktrap on two K4s joined by a bridge,
        // then re-run the dendrogram through community_to_membership
        // at the same step count and check the partition matches.
        use crate::Graph;
        use crate::walktrap;

        let mut g = Graph::with_vertices(8);
        for u in 0u32..4 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        for u in 4u32..8 {
            for v in (u + 1)..8 {
                g.add_edge(u, v).unwrap();
            }
        }
        g.add_edge(3, 4).unwrap(); // bridge

        let r = walktrap(&g).unwrap();
        assert_eq!(r.nb_clusters, 2);

        // Best-Q cut is at step `n - nb_clusters`.
        let step = 8u32 - r.nb_clusters;
        let cut = community_to_membership(&r.merges, 8, step).unwrap();
        assert_eq!(cut.membership.len(), 8);
        // The two K4s should be in distinct clusters under the cut.
        for u in 0u32..3 {
            assert_eq!(
                cut.membership[u as usize],
                cut.membership[(u + 1) as usize],
                "vertices {} and {} should share a cluster",
                u,
                u + 1
            );
        }
        for u in 4u32..7 {
            assert_eq!(cut.membership[u as usize], cut.membership[(u + 1) as usize]);
        }
        assert_ne!(cut.membership[0], cut.membership[7]);
        assert_eq!(cut.csize.iter().sum::<u32>(), 8);
        assert_eq!(cut.csize.len(), 2);
    }

    // ── le_community_to_membership tests ────────────────────────────

    #[test]
    fn le_zero_steps_keeps_initial_clusters() {
        // Three initial clusters, no merges applied.
        let membership = [0u32, 0, 1, 1, 2, 2];
        let r = le_community_to_membership(&[], 0, &membership).unwrap();
        assert_eq!(r.membership, vec![0, 0, 1, 1, 2, 2]);
        assert_eq!(r.csize, vec![2, 2, 2]);
    }

    #[test]
    fn le_one_merge_fuses_two_clusters() {
        // {0,1}=0, {2,3}=1, {4,5}=2; merge clusters 0 and 1.
        let membership = [0u32, 0, 1, 1, 2, 2];
        let r = le_community_to_membership(&[[0, 1]], 1, &membership).unwrap();
        assert_eq!(r.membership, vec![0, 0, 0, 0, 1, 1]);
        assert_eq!(r.csize, vec![4, 2]);
    }

    #[test]
    fn le_full_collapse() {
        // Three clusters, two merges: ((0,1)->3, (3,2)->4) -> all one.
        let membership = [0u32, 0, 1, 2, 2, 2];
        let merges = [[0u32, 1], [3, 2]];
        let r = le_community_to_membership(&merges, 2, &membership).unwrap();
        assert_eq!(r.membership, vec![0, 0, 0, 0, 0, 0]);
        assert_eq!(r.csize, vec![6]);
    }

    #[test]
    fn le_csize_sums_to_node_count() {
        let membership = [0u32, 1, 2, 3, 0, 1, 2, 3];
        // Merge clusters 0,1 -> 4; leave 2 and 3 alone (1 step).
        let r = le_community_to_membership(&[[0, 1]], 1, &membership).unwrap();
        assert_eq!(r.csize.iter().sum::<u32>(), 8);
        assert_eq!(r.csize.len(), 3); // 4 components - 1 step
        for &c in &r.membership {
            assert!((c as usize) < r.csize.len());
        }
    }

    #[test]
    fn le_err_empty_membership() {
        let err = le_community_to_membership(&[], 0, &[]).unwrap_err();
        match err {
            IgraphError::InvalidArgument(m) => assert!(m.contains("must not be empty")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn le_err_steps_not_smaller_than_components() {
        // Two components, asking for 2 steps is invalid.
        let membership = [0u32, 0, 1, 1];
        let err = le_community_to_membership(&[[0, 1]], 2, &membership).unwrap_err();
        match err {
            IgraphError::InvalidArgument(m) => assert!(m.contains("must be smaller")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn le_err_empty_cluster() {
        // Cluster id 1 is skipped: ids present are {0, 2}.
        let membership = [0u32, 0, 2, 2];
        let err = le_community_to_membership(&[[0, 1]], 1, &membership).unwrap_err();
        match err {
            IgraphError::InvalidArgument(m) => assert!(m.contains("empty cluster")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn le_matches_singleton_case() {
        // When every initial cluster is a singleton, the result must
        // equal community_to_membership on the same dendrogram.
        let membership = [0u32, 1, 2, 3];
        let merges = [[0u32, 1], [2, 3]];
        let le = le_community_to_membership(&merges, 2, &membership).unwrap();
        let plain = community_to_membership(&merges, 4, 2).unwrap();
        assert_eq!(le.membership, plain.membership);
        assert_eq!(le.csize, plain.csize);
    }

    #[cfg(all(test, feature = "proptest-harness"))]
    mod prop {
        use super::*;
        use proptest::prelude::*;

        // Generate a valid binary dendrogram on `n` leaves with at
        // most `n - 1` merge rows. Use a SplitMix64-style PRNG to pick
        // a random pair of live nodes at each step.
        prop_compose! {
            fn arb_dendrogram()(n in 2u32..=10u32, seed in any::<u64>())
                -> (u32, Vec<[u32; 2]>)
            {
                let mut rng: u64 = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
                let mut live: Vec<u32> = (0..n).collect();
                let mut merges: Vec<[u32; 2]> = Vec::with_capacity((n as usize).saturating_sub(1));
                let mut next_id: u32 = n;
                while live.len() >= 2 {
                    rng = rng.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1);
                    let i = (rng >> 32) as usize % live.len();
                    rng = rng.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1);
                    let mut j = (rng >> 32) as usize % live.len();
                    if j == i { j = (j + 1) % live.len(); }
                    let (i, j) = if i < j { (i, j) } else { (j, i) };
                    let c2 = live.remove(j);
                    let c1 = live.remove(i);
                    merges.push([c1, c2]);
                    live.push(next_id);
                    next_id += 1;
                }
                (n, merges)
            }
        }

        proptest! {
            #![proptest_config(ProptestConfig { cases: 80, ..ProptestConfig::default() })]

            // Cutting at 0 steps reproduces the singleton partition.
            #[test]
            fn zero_steps_singletons((n, merges) in arb_dendrogram()) {
                let r = community_to_membership(&merges, n, 0).unwrap();
                let expected: Vec<u32> = (0..n).collect();
                prop_assert_eq!(r.membership, expected);
                prop_assert_eq!(r.csize, vec![1u32; n as usize]);
            }

            // Cutting at the full step count collapses to one cluster.
            #[test]
            fn full_collapse((n, merges) in arb_dendrogram()) {
                let steps = u32::try_from(merges.len()).unwrap();
                let r = community_to_membership(&merges, n, steps).unwrap();
                let expected = vec![0u32; n as usize];
                prop_assert_eq!(r.membership, expected);
                prop_assert_eq!(r.csize, vec![n]);
            }

            // For every cut: csize sums to n, csize.len() == n - steps,
            // and every membership entry is < csize.len().
            #[test]
            fn cut_invariants((n, merges) in arb_dendrogram(), step_frac in 0u32..=100) {
                let rows = u32::try_from(merges.len()).unwrap();
                let steps = (rows * step_frac) / 100;
                let r = community_to_membership(&merges, n, steps).unwrap();
                let expected_k = n - steps;
                prop_assert_eq!(r.csize.len() as u32, expected_k);
                prop_assert_eq!(r.csize.iter().sum::<u32>(), n);
                for &c in &r.membership {
                    prop_assert!(c < expected_k);
                }
                for c in 0..expected_k {
                    let count = r.membership.iter().filter(|&&x| x == c).count() as u32;
                    prop_assert_eq!(count, r.csize[c as usize]);
                }
            }
        }
    }
}

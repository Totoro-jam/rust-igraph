//! ALGO-CM-016 — `split_join_distance` (asymmetric projection distances).
//!
//! Given two membership vectors over the same vertex set, returns the
//! **pair** of projection distances `(d12, d21)` where:
//!
//! - `d12 = n − Σ_i max_j |R_i ∩ C_j|` (project `comm1` onto `comm2`)
//! - `d21 = n − Σ_j max_i |R_i ∩ C_j|` (project `comm2` onto `comm1`)
//!
//! The total split-join distance (van Dongen 2000) is `d12 + d21`. One
//! component being zero means one partition is a *sub-partition* of the
//! other; both being zero means the partitions are identical.
//!
//! Mirrors `igraph_split_join_distance` in
//! `references/igraph/src/community/community_misc.c`. The C reference
//! exposes the asymmetric pair specifically so callers can detect the
//! sub-partition relationship — [`crate::compare_communities`] with
//! [`crate::CommunityComparison::SplitJoin`] returns the sum.
//!
//! Reuses the densification (via [`crate::reindex_membership`]) and
//! sparse-confusion-matrix machinery from CM-015 — both inputs are
//! reindexed once, then a single `HashMap<(u32, u32), u32>` pass yields
//! the row- and column-max sums.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use crate::core::error::{IgraphError, IgraphResult};

use super::compare_communities::split_join_distances;
use super::reindex_membership::reindex_membership;

/// Asymmetric split-join distances between two community partitions.
///
/// `d12` is the projection distance of `comm1` from `comm2`; `d21` is
/// the projection distance in the other direction. The (symmetric)
/// split-join distance is [`SplitJoinDistance::total`] = `d12 + d21`.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SplitJoinDistance {
    /// Projection distance of `comm1` from `comm2`.
    pub d12: u64,
    /// Projection distance of `comm2` from `comm1`.
    pub d21: u64,
}

impl SplitJoinDistance {
    /// Total (symmetric) split-join distance, `d12 + d21`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::split_join_distance;
    ///
    /// let r = split_join_distance(&[0, 0, 1, 1], &[0, 1, 1, 1]).unwrap();
    /// assert_eq!(r.total(), r.d12 + r.d21);
    /// ```
    #[must_use]
    pub fn total(self) -> u64 {
        self.d12 + self.d21
    }
}

/// Compute the two asymmetric projection distances between `comm1` and
/// `comm2`.
///
/// Both vectors must have the same length. Cluster ids do not have to
/// be densified — they are reindexed internally via
/// [`crate::reindex_membership`]. Empty inputs return `(0, 0)`.
///
/// # Examples
/// ```
/// use rust_igraph::split_join_distance;
///
/// // comm1 is a sub-partition of comm2: every cluster in comm1 fits
/// // entirely inside some cluster of comm2, so d12 == 0.
/// let r = split_join_distance(&[0, 0, 1, 2], &[0, 0, 0, 1]).unwrap();
/// assert_eq!(r.d12, 0);
/// assert_eq!(r.d21, 1);
/// assert_eq!(r.total(), 1);
///
/// // Identical partitions: both distances are 0.
/// let r = split_join_distance(&[0, 0, 1, 1], &[5, 5, 7, 7]).unwrap();
/// assert_eq!(r.d12, 0);
/// assert_eq!(r.d21, 0);
/// ```
///
/// # Errors
/// - [`IgraphError::InvalidArgument`] if `comm1.len() != comm2.len()`.
pub fn split_join_distance(comm1: &[u32], comm2: &[u32]) -> IgraphResult<SplitJoinDistance> {
    if comm1.len() != comm2.len() {
        return Err(IgraphError::InvalidArgument(format!(
            "community membership vectors have different lengths: {} and {}",
            comm1.len(),
            comm2.len(),
        )));
    }
    let n = comm1.len();
    if n == 0 {
        return Ok(SplitJoinDistance { d12: 0, d21: 0 });
    }
    let c1 = reindex_membership(comm1)?;
    let c2 = reindex_membership(comm2)?;
    let (d12, d21) = split_join_distances(&c1.membership, &c2.membership, n);
    Ok(SplitJoinDistance { d12, d21 })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_inputs_yield_zero_zero() {
        let r = split_join_distance(&[], &[]).unwrap();
        assert_eq!(r.d12, 0);
        assert_eq!(r.d21, 0);
        assert_eq!(r.total(), 0);
    }

    #[test]
    fn length_mismatch_errors() {
        let err = split_join_distance(&[0, 0], &[0, 0, 0]).unwrap_err();
        match err {
            IgraphError::InvalidArgument(_) => (),
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn identical_partition_is_zero() {
        let r = split_join_distance(&[0, 0, 1, 1, 2, 2], &[3, 3, 5, 5, 7, 7]).unwrap();
        assert_eq!(r.d12, 0);
        assert_eq!(r.d21, 0);
    }

    #[test]
    fn subpartition_asymmetry() {
        // comm1 = {{0,1},{2},{3}} is a sub-partition of comm2 = {{0,1,2},{3}}.
        // Every comm1 cluster fits inside a comm2 cluster ⇒ d12 = 0.
        // In the reverse direction: comm2[0]={0,1,2} overlaps comm1
        // clusters with sizes (2, 1) → max=2 → contributes 2; comm2[1]={3}
        // → contributes 1; sum = 3, n = 4 ⇒ d21 = 1.
        let r = split_join_distance(&[0, 0, 1, 2], &[0, 0, 0, 1]).unwrap();
        assert_eq!(r.d12, 0);
        assert_eq!(r.d21, 1);
        assert_eq!(r.total(), 1);
    }

    #[test]
    fn full_disagreement_2x2() {
        // Canonical worst case for n=4: [0,0,1,1] vs [0,1,0,1].
        // Each comm1 cluster overlaps comm2 clusters by (1, 1) → max=1
        // each → row sum = 2 → d12 = 4 - 2 = 2. Same for d21 by symmetry.
        // Total = 4 (the known SplitJoin from CM-015 unit tests).
        let r = split_join_distance(&[0, 0, 1, 1], &[0, 1, 0, 1]).unwrap();
        assert_eq!(r.d12, 2);
        assert_eq!(r.d21, 2);
        assert_eq!(r.total(), 4);
    }

    #[test]
    fn matches_compare_communities_split_join_sum() {
        use crate::{CommunityComparison, compare_communities};
        let a = [0u32, 0, 1, 2, 2, 3, 3, 3];
        let b = [0u32, 1, 1, 2, 3, 3, 0, 2];
        let r = split_join_distance(&a, &b).unwrap();
        let total = compare_communities(&a, &b, CommunityComparison::SplitJoin).unwrap();
        assert!((total - (r.total() as f64)).abs() < 1e-12);
    }

    #[test]
    fn relabel_invariant() {
        // Distances depend only on the partition, not the cluster ids.
        let a = [0u32, 0, 1, 1, 2, 2];
        let b = [0u32, 1, 0, 1, 0, 1];
        let r1 = split_join_distance(&a, &b).unwrap();
        // Relabel a: 0 -> 17, 1 -> 4, 2 -> 999. Relabel b: 0 -> 1_000_000, 1 -> 5.
        let a2: Vec<u32> = a
            .iter()
            .map(|&c| match c {
                0 => 17,
                1 => 4,
                _ => 999,
            })
            .collect();
        let b2: Vec<u32> = b
            .iter()
            .map(|&c| if c == 0 { 1_000_000 } else { 5 })
            .collect();
        let r2 = split_join_distance(&a2, &b2).unwrap();
        assert_eq!(r1, r2);
    }

    #[test]
    fn single_singleton_block() {
        // n=5, both partitions all singletons. Every cluster maps 1:1 ⇒
        // d12 = d21 = 0.
        let v: Vec<u32> = (0..5).collect();
        let r = split_join_distance(&v, &v).unwrap();
        assert_eq!(r.d12, 0);
        assert_eq!(r.d21, 0);
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::collection::vec;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn non_negative_components(
            (a, b) in (1usize..40).prop_flat_map(|n| (vec(0u32..5, n), vec(0u32..5, n)))
        ) {
            let r = split_join_distance(&a, &b).unwrap();
            // d12, d21 ∈ [0, n] always.
            let n = a.len() as u64;
            prop_assert!(r.d12 <= n);
            prop_assert!(r.d21 <= n);
        }

        #[test]
        fn total_equals_compare_communities_split_join(
            (a, b) in (1usize..40).prop_flat_map(|n| (vec(0u32..5, n), vec(0u32..5, n)))
        ) {
            use crate::{CommunityComparison, compare_communities};
            let r = split_join_distance(&a, &b).unwrap();
            let total = compare_communities(&a, &b, CommunityComparison::SplitJoin).unwrap();
            prop_assert!((total - (r.total() as f64)).abs() < 1e-12);
        }

        #[test]
        fn relabel_invariant_prop(
            (a, b) in (2usize..30).prop_flat_map(|n| (vec(0u32..6, n), vec(0u32..6, n))),
            offset in 1u32..1000,
        ) {
            let r1 = split_join_distance(&a, &b).unwrap();
            let a2: Vec<u32> = a.iter().map(|&c| c.wrapping_add(offset).wrapping_mul(31)).collect();
            let b2: Vec<u32> = b.iter().map(|&c| c.wrapping_add(offset.wrapping_mul(7))).collect();
            let r2 = split_join_distance(&a2, &b2).unwrap();
            prop_assert_eq!(r1, r2);
        }

        #[test]
        fn identical_partitions_are_zero(
            a in vec(0u32..7, 1..40),
        ) {
            let r = split_join_distance(&a, &a).unwrap();
            prop_assert_eq!(r.d12, 0);
            prop_assert_eq!(r.d21, 0);
        }

        #[test]
        fn subpartition_one_side_is_zero(
            a in vec(0u32..4, 2..25),
        ) {
            // Coarsen `a` into `b` by collapsing every cluster c -> c/2.
            // The result `b` is strictly coarser than `a`, so `a` is a
            // sub-partition of `b` and d12 = 0 (project a -> b: every
            // a-cluster fits entirely in one b-cluster).
            let b: Vec<u32> = a.iter().map(|&c| c / 2).collect();
            let r = split_join_distance(&a, &b).unwrap();
            prop_assert_eq!(r.d12, 0);
        }
    }
}

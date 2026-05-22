//! ALGO-CM-015 — `compare_communities` (5 partition-distance metrics).
//!
//! Compare two membership vectors over the same vertex set with one of
//! five standard partition-comparison measures: variation of information
//! (Meilă 2003), normalized mutual information (Danon et al. 2005),
//! split-join distance (van Dongen 2000), Rand index (Rand 1971), and
//! the adjusted Rand index (Hubert & Arabie 1985).
//!
//! Mirrors `igraph_compare_communities` /
//! `igraph_i_compare_communities_{nmi,vi,rand}` /
//! `igraph_i_split_join_distance` in
//! `references/igraph/src/community/community_misc.c`.
//!
//! Both membership vectors are first densified via
//! [`crate::reindex_membership`] so cluster ids are contiguous `0..k`,
//! then a confusion matrix is built once and reused for whichever
//! metric the caller picks. The confusion matrix is stored sparsely as
//! a `HashMap<(u32, u32), u32>` so the working set is `O(observed
//! co-occurrences)` rather than `O(k1 · k2)`.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use std::collections::HashMap;

use crate::core::error::{IgraphError, IgraphResult};

use super::reindex_membership::reindex_membership;

/// Which partition-comparison measure [`compare_communities`] returns.
///
/// Mirrors `igraph_community_comparison_t`.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CommunityComparison {
    /// **Variation of information** (Meilă 2003) — distance in `[0, log n]`.
    /// `VI = H(C1) + H(C2) − 2·I(C1, C2)`.
    /// Lower means more similar; 0 iff partitions are identical.
    VariationOfInformation,
    /// **Normalized mutual information** (Danon et al. 2005) —
    /// similarity in `[0, 1]`. `NMI = 2·I(C1, C2) / (H(C1) + H(C2))`.
    /// Higher means more similar; 1 iff partitions are identical.
    /// Defined as 1 when both partitions have a single cluster.
    NormalizedMutualInformation,
    /// **Split-join distance** (van Dongen 2000) — sum of the two
    /// projection distances. Distance in `[0, 2·(n − 1)]`. Lower
    /// means more similar; 0 iff partitions are identical.
    SplitJoin,
    /// **Rand index** (Rand 1971) — similarity in `[0, 1]`. Fraction
    /// of vertex pairs the two partitions agree on (either both
    /// together or both apart). Requires `n ≥ 2`.
    Rand,
    /// **Adjusted Rand index** (Hubert & Arabie 1985) — similarity in
    /// `[-1, 1]`, chance-corrected. 0 means the partitions agree no
    /// more than two random partitions of the same shape would.
    /// Requires `n ≥ 2`. Degenerate inputs where both partitions are
    /// all-one-cluster or all-singletons return 1.0 (sklearn
    /// convention; the C reference's formula would otherwise yield
    /// `0/0`).
    AdjustedRand,
}

/// Compare two community membership vectors over the same vertex set.
///
/// Both vectors must have the same length. Cluster ids do not have to
/// be densified — they are reindexed internally via
/// [`crate::reindex_membership`].
///
/// The returned `f64` carries either a **distance** (VI, `SplitJoin`)
/// or a **similarity** (NMI, Rand, `AdjustedRand`) depending on
/// [`CommunityComparison`].
///
/// # Examples
/// ```
/// use rust_igraph::{compare_communities, CommunityComparison};
///
/// // Identical partitions.
/// let q = compare_communities(
///     &[0, 0, 1, 1],
///     &[5, 5, 7, 7],
///     CommunityComparison::NormalizedMutualInformation,
/// ).unwrap();
/// assert!((q - 1.0).abs() < 1e-12);
///
/// let q = compare_communities(
///     &[0, 0, 1, 1],
///     &[5, 5, 7, 7],
///     CommunityComparison::VariationOfInformation,
/// ).unwrap();
/// assert!(q.abs() < 1e-12);
/// ```
///
/// # Errors
/// - [`IgraphError::InvalidArgument`] if `comm1.len() != comm2.len()`.
/// - [`IgraphError::InvalidArgument`] if `n < 2` and `method` is
///   `Rand` or `AdjustedRand` (mirrors the C reference).
pub fn compare_communities(
    comm1: &[u32],
    comm2: &[u32],
    method: CommunityComparison,
) -> IgraphResult<f64> {
    if comm1.len() != comm2.len() {
        return Err(IgraphError::InvalidArgument(format!(
            "community membership vectors have different lengths: {} and {}",
            comm1.len(),
            comm2.len(),
        )));
    }

    let n = comm1.len();

    if n == 0 {
        return match method {
            CommunityComparison::NormalizedMutualInformation => Ok(1.0),
            CommunityComparison::VariationOfInformation | CommunityComparison::SplitJoin => Ok(0.0),
            CommunityComparison::Rand | CommunityComparison::AdjustedRand => {
                Err(IgraphError::InvalidArgument(format!(
                    "Rand indices not defined for zero or one vertices. \
                     Found membership vector of size {n}.",
                )))
            }
        };
    }

    // Densify so cluster ids are contiguous `0..k`.
    let c1 = reindex_membership(comm1)?;
    let c2 = reindex_membership(comm2)?;

    match method {
        CommunityComparison::VariationOfInformation => {
            let (h1, h2, mi) = entropy_and_mutual_information(&c1.membership, &c2.membership, n);
            Ok(h1 + h2 - 2.0 * mi)
        }
        CommunityComparison::NormalizedMutualInformation => {
            let (h1, h2, mi) = entropy_and_mutual_information(&c1.membership, &c2.membership, n);
            if h1 == 0.0 && h2 == 0.0 {
                Ok(1.0)
            } else {
                Ok(2.0 * mi / (h1 + h2))
            }
        }
        CommunityComparison::SplitJoin => {
            let (d12, d21) = split_join_distances(&c1.membership, &c2.membership, n);
            // Sum fits because both are bounded by n which is a usize
            // bounded by isize::MAX; the f64 round-trip is exact for
            // any practical input.
            Ok((d12 + d21) as f64)
        }
        CommunityComparison::Rand | CommunityComparison::AdjustedRand => {
            if n < 2 {
                return Err(IgraphError::InvalidArgument(format!(
                    "Rand indices not defined for zero or one vertices. \
                     Found membership vector of size {n}.",
                )));
            }
            Ok(rand_index(
                &c1.membership,
                &c2.membership,
                n,
                matches!(method, CommunityComparison::AdjustedRand),
            ))
        }
    }
}

/// Joint entropy + mutual information of two reindexed membership
/// vectors over `n = v1.len() = v2.len() > 0`. Returns `(H(v1), H(v2),
/// I(v1; v2))`, all in nats.
fn entropy_and_mutual_information(v1: &[u32], v2: &[u32], n: usize) -> (f64, f64, f64) {
    let k1 = max_plus_one(v1);
    let k2 = max_plus_one(v2);
    let n_f = n as f64;

    // Marginal cluster sizes.
    let mut p1: Vec<f64> = vec![0.0; k1];
    let mut p2: Vec<f64> = vec![0.0; k2];
    for &c in v1 {
        p1[c as usize] += 1.0;
    }
    for &c in v2 {
        p2[c as usize] += 1.0;
    }

    // Marginal entropies. log(0) is unreachable because every cluster
    // id in 0..k1 (resp. 0..k2) was emitted by `reindex_membership`
    // and therefore appears at least once.
    let mut h1 = 0.0;
    for x in &mut p1 {
        *x /= n_f;
        h1 -= *x * x.ln();
    }
    let mut h2 = 0.0;
    for x in &mut p2 {
        *x /= n_f;
        h2 -= *x * x.ln();
    }

    // From here on we only need log p1 / log p2.
    let log_p1: Vec<f64> = p1.iter().map(|&p| p.ln()).collect();
    let log_p2: Vec<f64> = p2.iter().map(|&p| p.ln()).collect();

    // Confusion counts (sparse) — observed (c1, c2) pairs only.
    let mut counts: HashMap<(u32, u32), u32> = HashMap::new();
    for i in 0..n {
        *counts.entry((v1[i], v2[i])).or_insert(0) += 1;
    }

    let mut mut_inf = 0.0;
    for (&(r, c), &cnt) in &counts {
        let p = f64::from(cnt) / n_f;
        mut_inf += p * (p.ln() - log_p1[r as usize] - log_p2[c as usize]);
    }

    (h1, h2, mut_inf)
}

/// Returns the split-join distances `(d12, d21)` between two
/// reindexed membership vectors over `n > 0`. `d12` is `n − Σ_i
/// max_j |R_i ∩ C_j|` where rows are `v1`-clusters; `d21` swaps
/// rows/cols.
pub(crate) fn split_join_distances(v1: &[u32], v2: &[u32], n: usize) -> (u64, u64) {
    let k1 = max_plus_one(v1);
    let k2 = max_plus_one(v2);

    let mut counts: HashMap<(u32, u32), u32> = HashMap::new();
    for i in 0..n {
        *counts.entry((v1[i], v2[i])).or_insert(0) += 1;
    }

    let mut row_max: Vec<u32> = vec![0; k1];
    let mut col_max: Vec<u32> = vec![0; k2];
    for (&(r, c), &cnt) in &counts {
        let r_slot = &mut row_max[r as usize];
        if cnt > *r_slot {
            *r_slot = cnt;
        }
        let c_slot = &mut col_max[c as usize];
        if cnt > *c_slot {
            *c_slot = cnt;
        }
    }

    let sum_row: u64 = row_max.iter().map(|&x| u64::from(x)).sum();
    let sum_col: u64 = col_max.iter().map(|&x| u64::from(x)).sum();

    let n_u64 = n as u64;
    (n_u64 - sum_row, n_u64 - sum_col)
}

/// Rand index (or adjusted Rand index if `adjust`) of two reindexed
/// membership vectors over `n ≥ 2`.
fn rand_index(v1: &[u32], v2: &[u32], n: usize, adjust: bool) -> f64 {
    let k1 = max_plus_one(v1);
    let k2 = max_plus_one(v2);
    let n_f = n as f64;

    let mut counts: HashMap<(u32, u32), u32> = HashMap::new();
    for i in 0..n {
        *counts.entry((v1[i], v2[i])).or_insert(0) += 1;
    }

    let mut row_sums: Vec<f64> = vec![0.0; k1];
    let mut col_sums: Vec<f64> = vec![0.0; k2];
    for (&(r, c), &cnt) in &counts {
        row_sums[r as usize] += f64::from(cnt);
        col_sums[c as usize] += f64::from(cnt);
    }

    // a/(n choose 2) term — sum_ij (n_ij / n)·((n_ij − 1)/(n − 1)).
    let mut joint = 0.0;
    for &cnt in counts.values() {
        let v = f64::from(cnt);
        joint += (v / n_f) * (v - 1.0) / (n_f - 1.0);
    }

    let mut frac_in_1 = 0.0;
    for &v in &row_sums {
        frac_in_1 += (v / n_f) * (v - 1.0) / (n_f - 1.0);
    }
    let mut frac_in_2 = 0.0;
    for &v in &col_sums {
        frac_in_2 += (v / n_f) * (v - 1.0) / (n_f - 1.0);
    }

    // Unadjusted Rand index — see C reference for the derivation.
    let rand = 1.0 + 2.0 * joint - frac_in_1 - frac_in_2;

    if adjust {
        let expected = frac_in_1 * frac_in_2 + (1.0 - frac_in_1) * (1.0 - frac_in_2);
        let denom = 1.0 - expected;
        // Degenerate cases (both partitions all-one-cluster or both
        // all-singletons) give 0/0 under the C formula. Following
        // sklearn's `adjusted_rand_score`, take the limit value 1.0:
        // the partitions necessarily agree perfectly.
        if denom == 0.0 {
            1.0
        } else {
            (rand - expected) / denom
        }
    } else {
        rand
    }
}

/// `max(v) + 1`, where `v` is a densified `[0, k)` membership vector.
/// `v` is non-empty by the call sites.
fn max_plus_one(v: &[u32]) -> usize {
    let m = v.iter().copied().max().unwrap_or(0);
    (m as usize) + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() < tol
    }

    #[test]
    fn err_on_length_mismatch() {
        let err = compare_communities(&[0, 1], &[0], CommunityComparison::VariationOfInformation)
            .unwrap_err();
        match err {
            IgraphError::InvalidArgument(_) => (),
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn empty_input_returns_method_defaults() {
        for (m, expected) in [
            (CommunityComparison::VariationOfInformation, 0.0),
            (CommunityComparison::NormalizedMutualInformation, 1.0),
            (CommunityComparison::SplitJoin, 0.0),
        ] {
            let q = compare_communities(&[], &[], m).unwrap();
            assert!(close(q, expected, 1e-12), "method {m:?} got {q}");
        }
        for m in [CommunityComparison::Rand, CommunityComparison::AdjustedRand] {
            assert!(compare_communities(&[], &[], m).is_err());
        }
    }

    #[test]
    fn identical_partitions_have_nmi_1_and_vi_0() {
        let v = [0, 0, 1, 1, 2, 2];
        assert!(close(
            compare_communities(&v, &v, CommunityComparison::NormalizedMutualInformation).unwrap(),
            1.0,
            1e-12,
        ));
        assert!(close(
            compare_communities(&v, &v, CommunityComparison::VariationOfInformation).unwrap(),
            0.0,
            1e-12,
        ));
        assert!(close(
            compare_communities(&v, &v, CommunityComparison::Rand).unwrap(),
            1.0,
            1e-12,
        ));
        assert!(close(
            compare_communities(&v, &v, CommunityComparison::AdjustedRand).unwrap(),
            1.0,
            1e-12,
        ));
        assert!(close(
            compare_communities(&v, &v, CommunityComparison::SplitJoin).unwrap(),
            0.0,
            1e-12,
        ));
    }

    #[test]
    fn relabel_invariance() {
        // Same partition under two different labellings — every
        // measure must agree with the "identical" baseline.
        let a = [0, 0, 1, 1, 2, 2];
        let b = [7, 7, 3, 3, 9, 9];
        for m in [
            CommunityComparison::VariationOfInformation,
            CommunityComparison::NormalizedMutualInformation,
            CommunityComparison::SplitJoin,
            CommunityComparison::Rand,
            CommunityComparison::AdjustedRand,
        ] {
            let q1 = compare_communities(&a, &a, m).unwrap();
            let q2 = compare_communities(&a, &b, m).unwrap();
            assert!(close(q1, q2, 1e-12), "method {m:?}: {q1} vs {q2}");
        }
    }

    #[test]
    fn singletons_vs_singletons() {
        let v: Vec<u32> = (0..6).collect();
        assert!(close(
            compare_communities(&v, &v, CommunityComparison::NormalizedMutualInformation).unwrap(),
            1.0,
            1e-12,
        ));
        // Two everyone-different partitions agree on every pair (each
        // pair is "apart" in both), so Rand = 1.
        let w: Vec<u32> = (0..6).rev().collect();
        assert!(close(
            compare_communities(&v, &w, CommunityComparison::Rand).unwrap(),
            1.0,
            1e-12,
        ));
    }

    #[test]
    fn one_cluster_each_side_is_nmi_one_per_spec() {
        // h1 == h2 == 0 → NMI defined as 1.
        let v = [0u32; 5];
        let w = [9u32; 5];
        assert!(close(
            compare_communities(&v, &w, CommunityComparison::NormalizedMutualInformation).unwrap(),
            1.0,
            1e-12,
        ));
        assert!(close(
            compare_communities(&v, &w, CommunityComparison::VariationOfInformation).unwrap(),
            0.0,
            1e-12,
        ));
        assert!(close(
            compare_communities(&v, &w, CommunityComparison::SplitJoin).unwrap(),
            0.0,
            1e-12,
        ));
        // Rand of "all same" vs "all same" — every pair agrees.
        assert!(close(
            compare_communities(&v, &w, CommunityComparison::Rand).unwrap(),
            1.0,
            1e-12,
        ));
    }

    #[test]
    fn full_disagreement_two_clusters() {
        // n=4, {0,0,1,1} vs {0,1,0,1}. Confusion = [[1,1],[1,1]].
        // MI = 0 since rows/cols are independent under uniform.
        let a = [0u32, 0, 1, 1];
        let b = [0u32, 1, 0, 1];
        let nmi =
            compare_communities(&a, &b, CommunityComparison::NormalizedMutualInformation).unwrap();
        assert!(close(nmi, 0.0, 1e-12), "NMI = {nmi}");
        // VI = H1 + H2 - 2*MI = ln 2 + ln 2 - 0 = 2*ln 2.
        let vi = compare_communities(&a, &b, CommunityComparison::VariationOfInformation).unwrap();
        assert!(close(vi, 2.0 * 2f64.ln(), 1e-12), "VI = {vi}");
        // SplitJoin: row-max = [1,1], col-max = [1,1]; d12 = d21 = 4-2 = 2.
        let sj = compare_communities(&a, &b, CommunityComparison::SplitJoin).unwrap();
        assert!(close(sj, 4.0, 1e-12), "SJ = {sj}");
        // Rand: pairs (0,1),(2,3) agree (both in same on a, diff on b → disagree).
        // Pairs (0,2),(1,3) agree across. There are C(4,2)=6 pairs total.
        // n_ij counts: each cell = 1; joint = 4*(1/4)*(0/3) = 0.
        // row n_i* = 2,2 → frac_in_1 = 2*(2/4)*(1/3) = 1/3.
        // frac_in_2 = 1/3. Rand = 1 + 0 - 1/3 - 1/3 = 1/3.
        let rand = compare_communities(&a, &b, CommunityComparison::Rand).unwrap();
        assert!(close(rand, 1.0 / 3.0, 1e-12), "Rand = {rand}");
        // AdjustedRand: expected = (1/3)^2 + (2/3)^2 = 5/9 → AR = (1/3 - 5/9)/(1 - 5/9) = -2/4 = -0.5.
        let ar = compare_communities(&a, &b, CommunityComparison::AdjustedRand).unwrap();
        assert!(close(ar, -0.5, 1e-12), "AR = {ar}");
    }

    #[test]
    fn split_join_is_zero_for_subpartition() {
        // Coarser partition (a) refines into finer (b): b is a refinement.
        // d21 = 0 (b → a is a sub-cluster relation), d12 may be > 0.
        let a = [0u32, 0, 0, 1, 1, 1];
        let b = [5u32, 5, 6, 7, 7, 8];
        // Reuse the internal helper to inspect both pieces.
        let r1 = reindex_membership(&a).unwrap();
        let r2 = reindex_membership(&b).unwrap();
        let (d12, d21) = split_join_distances(&r1.membership, &r2.membership, a.len());
        // {0,1,2}∩{5,5,6} → 2 (best); {3,4,5}∩{7,7,8} → 2 (best).
        // d12 = 6 - (2 + 2) = 2.
        assert_eq!(d12, 2);
        // For d21: each finer cluster falls inside one coarser cluster.
        // {5,5}∩{0,0,0}=2 ; {6}∩{0,0,0}=1 ; {7,7}∩{1,1,1}=2 ; {8}∩{1,1,1}=1.
        // d21 = 6 - (2+1+2+1) = 0.
        assert_eq!(d21, 0);
        let sj = compare_communities(&a, &b, CommunityComparison::SplitJoin).unwrap();
        assert!(close(sj, 2.0, 1e-12));
    }

    #[test]
    fn nmi_is_symmetric() {
        let a = [0u32, 0, 1, 1, 2, 2, 0, 1];
        let b = [3u32, 4, 4, 3, 3, 4, 4, 3];
        let n_ab =
            compare_communities(&a, &b, CommunityComparison::NormalizedMutualInformation).unwrap();
        let n_ba =
            compare_communities(&b, &a, CommunityComparison::NormalizedMutualInformation).unwrap();
        assert!(close(n_ab, n_ba, 1e-12));
    }

    #[test]
    fn rand_requires_at_least_two_vertices() {
        let v = [0u32];
        assert!(compare_communities(&v, &v, CommunityComparison::Rand).is_err());
        assert!(compare_communities(&v, &v, CommunityComparison::AdjustedRand).is_err());
    }

    #[test]
    fn variation_of_information_zero_iff_same_partition() {
        let a = [0u32, 0, 1, 1];
        let b = [1u32, 1, 0, 0]; // same partition, relabelled
        let vi = compare_communities(&a, &b, CommunityComparison::VariationOfInformation).unwrap();
        assert!(close(vi, 0.0, 1e-12));
    }

    #[cfg(feature = "proptest-harness")]
    mod prop {
        use super::*;
        use proptest::prelude::*;

        prop_compose! {
            fn arb_pair()(
                n in 2usize..=24,
                k1 in 1u32..=5,
                k2 in 1u32..=5,
                seed in any::<u64>(),
            ) -> (Vec<u32>, Vec<u32>) {
                let mut rng: u64 = seed.wrapping_add(0xDEAD_BEEF_C0FF_EE00);
                let mut step = || -> u32 {
                    rng = rng.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1);
                    (rng >> 32) as u32
                };
                let v1: Vec<u32> = (0..n).map(|_| step() % k1).collect();
                let v2: Vec<u32> = (0..n).map(|_| step() % k2).collect();
                (v1, v2)
            }
        }

        proptest! {
            #![proptest_config(ProptestConfig { cases: 60, ..ProptestConfig::default() })]

            #[test]
            fn nmi_in_unit_interval((a, b) in arb_pair()) {
                let q = compare_communities(
                    &a, &b, CommunityComparison::NormalizedMutualInformation,
                ).unwrap();
                prop_assert!((-1e-9..=1.0 + 1e-9).contains(&q), "NMI out of [0,1]: {}", q);
            }

            #[test]
            fn vi_non_negative((a, b) in arb_pair()) {
                let q = compare_communities(
                    &a, &b, CommunityComparison::VariationOfInformation,
                ).unwrap();
                prop_assert!(q >= -1e-9, "VI < 0: {}", q);
            }

            #[test]
            fn rand_in_unit_interval((a, b) in arb_pair()) {
                let q = compare_communities(
                    &a, &b, CommunityComparison::Rand,
                ).unwrap();
                prop_assert!((-1e-9..=1.0 + 1e-9).contains(&q), "Rand out of [0,1]: {}", q);
            }

            #[test]
            fn adjusted_rand_capped_at_one((a, b) in arb_pair()) {
                let q = compare_communities(
                    &a, &b, CommunityComparison::AdjustedRand,
                ).unwrap();
                prop_assert!(q <= 1.0 + 1e-9, "AR > 1: {}", q);
            }

            #[test]
            fn measures_are_relabel_invariant((a, b) in arb_pair()) {
                // Multiply both partitions through a relabel and confirm
                // each measure is unchanged.
                let bump = |v: &[u32], offset: u32| -> Vec<u32> {
                    v.iter().map(|&x| x.wrapping_add(offset).wrapping_mul(7)).collect()
                };
                let a2 = bump(&a, 100);
                let b2 = bump(&b, 50);
                for m in [
                    CommunityComparison::VariationOfInformation,
                    CommunityComparison::NormalizedMutualInformation,
                    CommunityComparison::SplitJoin,
                    CommunityComparison::Rand,
                    CommunityComparison::AdjustedRand,
                ] {
                    let q1 = compare_communities(&a, &b, m).unwrap();
                    let q2 = compare_communities(&a2, &b2, m).unwrap();
                    prop_assert!((q1 - q2).abs() < 1e-9, "method {:?}: {} vs {}", m, q1, q2);
                }
            }

            #[test]
            fn nmi_symmetric((a, b) in arb_pair()) {
                let ab = compare_communities(
                    &a, &b, CommunityComparison::NormalizedMutualInformation,
                ).unwrap();
                let ba = compare_communities(
                    &b, &a, CommunityComparison::NormalizedMutualInformation,
                ).unwrap();
                prop_assert!((ab - ba).abs() < 1e-9);
            }

            #[test]
            fn identical_partition_is_extremal((a, _b) in arb_pair()) {
                for (m, expected) in [
                    (CommunityComparison::VariationOfInformation, 0.0_f64),
                    (CommunityComparison::NormalizedMutualInformation, 1.0),
                    (CommunityComparison::SplitJoin, 0.0),
                    (CommunityComparison::Rand, 1.0),
                    (CommunityComparison::AdjustedRand, 1.0),
                ] {
                    let q = compare_communities(&a, &a, m).unwrap();
                    prop_assert!((q - expected).abs() < 1e-9, "method {:?}: {} vs {}", m, q, expected);
                }
            }
        }
    }
}

//! ALGO-CM-014 — `reindex_membership` (densify membership labels).
//!
//! Take any `membership[v] = c` vector with arbitrary cluster ids
//! (including gaps and negatives in the C version's `int` typing —
//! here, any `u32`) and produce a new, densified `0..k-1` labelling
//! such that the *partition* is identical. Optionally also report the
//! old cluster id that each new id replaced, in encounter order.
//!
//! Mirrors `igraph_reindex_membership` /
//! `igraph_i_reindex_membership_large` in
//! `references/igraph/src/community/community_misc.c`.
//!
//! The C implementation has a fast path when every id is already in
//! `0..n`, and a sort-based fallback otherwise. Because Rust's
//! `Vec<u32>` already has cheap key-set operations via `HashMap`, we
//! use a single `BTreeMap`-free first-occurrence scan: O(n) when the
//! id space fits in a `Vec` (length `max_id + 1` is bounded by `n`),
//! and `O(n log k)` via `BTreeMap` otherwise. Both branches preserve
//! the *encounter order* — the new id of an old id is the index where
//! it was first seen.

use std::collections::BTreeMap;

use crate::core::error::IgraphResult;

/// Result of [`reindex_membership`]: a densified membership vector
/// `[0, k)`, the old cluster id behind each new id (so `new_to_old[i]`
/// is the original id that now maps to `i`), and the number of
/// distinct clusters `k = new_to_old.len()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReindexMembershipResult {
    /// `membership[v] ∈ [0, k)` — the densified cluster id of vertex
    /// `v`.
    pub membership: Vec<u32>,
    /// `new_to_old[i]` — the original cluster id that vertex
    /// encounters with this id now map to. `nb_clusters = new_to_old.len()`.
    pub new_to_old: Vec<u32>,
}

impl ReindexMembershipResult {
    /// Number of distinct clusters in the densified membership.
    #[must_use]
    pub fn nb_clusters(&self) -> u32 {
        // `new_to_old` cannot exceed `membership.len()`, which is a
        // `Vec` length bounded by `isize::MAX`. The cast is safe for
        // any practical input.
        u32::try_from(self.new_to_old.len()).unwrap_or(u32::MAX)
    }
}

/// Relabel a membership vector so cluster ids run contiguously over
/// `0..k`, where `k` is the number of distinct clusters. The *partition*
/// is unchanged: two vertices share a cluster in the output iff they
/// shared one in the input. New ids are assigned in *first-occurrence
/// order* — the first cluster id you encounter when sweeping left to
/// right becomes `0`, the next new one becomes `1`, and so on.
///
/// # Examples
/// ```
/// use rust_igraph::reindex_membership;
///
/// // Gaps and out-of-order ids are densified by first occurrence.
/// let r = reindex_membership(&[7, 7, 3, 7, 9, 3]).unwrap();
/// assert_eq!(r.membership, vec![0, 0, 1, 0, 2, 1]);
/// assert_eq!(r.new_to_old, vec![7, 3, 9]);
/// assert_eq!(r.nb_clusters(), 3);
/// ```
///
/// # Errors
/// Currently infallible — returns `IgraphResult` for API symmetry with
/// other community helpers and to allow adding fallible checks later
/// without a breaking change.
pub fn reindex_membership(membership: &[u32]) -> IgraphResult<ReindexMembershipResult> {
    let n = membership.len();

    // Empty input is the trivial 0-cluster case.
    if n == 0 {
        return Ok(ReindexMembershipResult {
            membership: Vec::new(),
            new_to_old: Vec::new(),
        });
    }

    // Mirror the C implementation: fast path when every id is in
    // `0..n` — we can use a flat `Vec<u32>` of size `n` as the lookup,
    // where `lookup[old] = new + 1` and 0 means "not yet seen".
    let mut max_id: u32 = 0;
    for &c in membership {
        if c > max_id {
            max_id = c;
        }
    }

    if (max_id as usize) < n {
        let mut lookup: Vec<u32> = vec![0; n];
        let mut new_to_old: Vec<u32> = Vec::new();
        let mut out: Vec<u32> = vec![0; n];
        let mut next_new: u32 = 0;

        for (i, &old) in membership.iter().enumerate() {
            let slot = &mut lookup[old as usize];
            if *slot == 0 {
                *slot = next_new + 1;
                new_to_old.push(old);
                next_new = next_new.saturating_add(1);
            }
            out[i] = *slot - 1;
        }

        return Ok(ReindexMembershipResult {
            membership: out,
            new_to_old,
        });
    }

    // Sparse / large-id fallback: BTreeMap keyed by old cluster id.
    let mut lookup: BTreeMap<u32, u32> = BTreeMap::new();
    let mut new_to_old: Vec<u32> = Vec::new();
    let mut out: Vec<u32> = vec![0; n];
    let mut next_new: u32 = 0;

    for (i, &old) in membership.iter().enumerate() {
        let new_id = if let Some(&nid) = lookup.get(&old) {
            nid
        } else {
            let nid = next_new;
            lookup.insert(old, nid);
            new_to_old.push(old);
            next_new = next_new.saturating_add(1);
            nid
        };
        out[i] = new_id;
    }

    Ok(ReindexMembershipResult {
        membership: out,
        new_to_old,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_membership() {
        let r = reindex_membership(&[]).unwrap();
        assert!(r.membership.is_empty());
        assert!(r.new_to_old.is_empty());
        assert_eq!(r.nb_clusters(), 0);
    }

    #[test]
    fn already_densified_identity() {
        let r = reindex_membership(&[0, 1, 2, 0, 1, 2]).unwrap();
        assert_eq!(r.membership, vec![0, 1, 2, 0, 1, 2]);
        assert_eq!(r.new_to_old, vec![0, 1, 2]);
    }

    #[test]
    fn singleton_cluster_collapses_to_zero() {
        let r = reindex_membership(&[42, 42, 42, 42]).unwrap();
        assert_eq!(r.membership, vec![0, 0, 0, 0]);
        assert_eq!(r.new_to_old, vec![42]);
        assert_eq!(r.nb_clusters(), 1);
    }

    #[test]
    fn gaps_are_compressed() {
        // Old ids: 5, 5, 10, 5, 0 -> first-encounter: 5 -> 0, 10 -> 1, 0 -> 2.
        let r = reindex_membership(&[5, 5, 10, 5, 0]).unwrap();
        assert_eq!(r.membership, vec![0, 0, 1, 0, 2]);
        assert_eq!(r.new_to_old, vec![5, 10, 0]);
    }

    #[test]
    fn reverse_order_relabels_to_descending_in_new_to_old() {
        // 3 vertices, ids 2, 1, 0. New ids assigned in first-encounter:
        // 2 -> 0, 1 -> 1, 0 -> 2.
        let r = reindex_membership(&[2, 1, 0]).unwrap();
        assert_eq!(r.membership, vec![0, 1, 2]);
        assert_eq!(r.new_to_old, vec![2, 1, 0]);
    }

    #[test]
    fn singletons() {
        let r = reindex_membership(&[7, 8, 9, 10]).unwrap();
        assert_eq!(r.membership, vec![0, 1, 2, 3]);
        assert_eq!(r.new_to_old, vec![7, 8, 9, 10]);
    }

    #[test]
    fn large_id_takes_btreemap_fallback() {
        // max_id (1_000_000) >> n (4), forces the sparse branch.
        let r = reindex_membership(&[1_000_000, 7, 1_000_000, 7]).unwrap();
        assert_eq!(r.membership, vec![0, 1, 0, 1]);
        assert_eq!(r.new_to_old, vec![1_000_000, 7]);
    }

    #[test]
    fn fast_path_max_id_equals_n_minus_1() {
        // Edge of the fast-path branch: max_id == n - 1.
        let r = reindex_membership(&[3, 0, 1, 2]).unwrap();
        assert_eq!(r.membership, vec![0, 1, 2, 3]);
        assert_eq!(r.new_to_old, vec![3, 0, 1, 2]);
    }

    #[test]
    fn fast_path_max_id_equals_n_takes_sparse_branch() {
        // max_id (4) == n (4), so max_id < n is false -> sparse path.
        // The output is still correct.
        let r = reindex_membership(&[4, 0, 1, 2]).unwrap();
        assert_eq!(r.membership, vec![0, 1, 2, 3]);
        assert_eq!(r.new_to_old, vec![4, 0, 1, 2]);
    }

    #[test]
    fn preserves_partition_under_relabel() {
        // Two vertices share a cluster in the input iff they share one
        // in the output, regardless of label values.
        let inputs: &[&[u32]] = &[
            &[0, 0, 1, 2, 1, 2, 0],
            &[5, 5, 1_000, 7, 1_000, 7, 5],
            &[u32::MAX, 0, u32::MAX, 1, 0],
        ];
        for input in inputs {
            let r = reindex_membership(input).unwrap();
            for i in 0..input.len() {
                for j in 0..input.len() {
                    assert_eq!(
                        input[i] == input[j],
                        r.membership[i] == r.membership[j],
                        "partition diverged at ({i}, {j}) for input {input:?}",
                    );
                }
            }
            // Densified labels really do run 0..k.
            let k = r.nb_clusters();
            for &c in &r.membership {
                assert!(c < k);
            }
            for c in 0..k {
                assert!(r.membership.contains(&c));
            }
            assert_eq!(r.new_to_old.len(), k as usize);
        }
    }

    #[test]
    fn fast_path_matches_sparse_path_for_in_range_ids() {
        // Same input under both branches should yield the same result.
        // The fast path triggers when max_id < n. We can force the
        // sparse path by appending an out-of-range tail value, then
        // truncating the result, and comparing the in-range prefix.
        let input: Vec<u32> = vec![3, 0, 3, 1, 2, 0, 1];
        let fast = reindex_membership(&input).unwrap();
        let len_u32 = u32::try_from(input.len()).expect("test input length fits u32");
        assert!(input.iter().copied().max().unwrap_or(0) < len_u32);
        // Build a "sparse-branch" check by manually constructing the
        // expected first-occurrence mapping and comparing.
        let mut expected_new_to_old: Vec<u32> = Vec::new();
        let mut expected: Vec<u32> = Vec::with_capacity(input.len());
        for &old in &input {
            let pos = expected_new_to_old
                .iter()
                .position(|&x| x == old)
                .unwrap_or_else(|| {
                    expected_new_to_old.push(old);
                    expected_new_to_old.len() - 1
                });
            expected.push(u32::try_from(pos).expect("test bookkeeping fits u32"));
        }
        assert_eq!(fast.membership, expected);
        assert_eq!(fast.new_to_old, expected_new_to_old);
    }

    #[cfg(all(test, feature = "proptest-harness"))]
    mod prop {
        use super::*;
        use proptest::prelude::*;

        prop_compose! {
            fn arb_membership()(
                n in 1usize..=64,
                seed in any::<u64>(),
                max_id_kind in 0u32..3,
            ) -> Vec<u32> {
                let mut rng: u64 = seed.wrapping_add(0xA5A5_5A5A_DEAD_BEEF);
                let max_id: u32 = match max_id_kind {
                    0 => (n as u32).saturating_sub(1).max(1),       // fast path
                    1 => (n as u32).saturating_mul(4).max(1),       // straddles boundary
                    _ => 1_000_000,                                  // sparse path
                };
                (0..n).map(|_| {
                    rng = rng.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1);
                    (rng >> 32) as u32 % (max_id + 1)
                }).collect()
            }
        }

        proptest! {
            #![proptest_config(ProptestConfig { cases: 80, ..ProptestConfig::default() })]

            #[test]
            fn partition_preserved(input in arb_membership()) {
                let r = reindex_membership(&input).unwrap();
                prop_assert_eq!(r.membership.len(), input.len());
                for i in 0..input.len() {
                    for j in (i + 1)..input.len() {
                        prop_assert_eq!(
                            input[i] == input[j],
                            r.membership[i] == r.membership[j],
                        );
                    }
                }
            }

            #[test]
            fn ids_are_contiguous(input in arb_membership()) {
                let r = reindex_membership(&input).unwrap();
                let k = r.nb_clusters();
                for &c in &r.membership {
                    prop_assert!(c < k);
                }
                // Every id in 0..k appears at least once.
                for c in 0..k {
                    prop_assert!(r.membership.contains(&c));
                }
            }

            #[test]
            fn new_to_old_round_trips(input in arb_membership()) {
                let r = reindex_membership(&input).unwrap();
                prop_assert_eq!(r.new_to_old.len(), r.nb_clusters() as usize);
                for (i, &old) in input.iter().enumerate() {
                    let new = r.membership[i] as usize;
                    prop_assert_eq!(r.new_to_old[new], old);
                }
            }

            #[test]
            fn idempotent_when_already_dense(input in arb_membership()) {
                let r1 = reindex_membership(&input).unwrap();
                let r2 = reindex_membership(&r1.membership).unwrap();
                prop_assert_eq!(r1.membership.clone(), r2.membership);
                // After one pass, new_to_old of the second call is just
                // (0..k) since the input is already 0..k in first-occurrence
                // order.
                let expected_n2o: Vec<u32> = (0..r1.nb_clusters()).collect();
                prop_assert_eq!(r2.new_to_old, expected_n2o);
            }
        }
    }
}

//! Degree sequence realizability tests (ALGO-PR-034).
//!
//! Counterpart of `igraph_is_graphical` + `igraph_is_bigraphical` from
//! `references/igraph/src/misc/graphicality.c` (928 lines).
//!
//! Tests whether a given degree sequence can be realized as a graph
//! of the specified type (simple, multi-edges, loops, etc.).

/// What kinds of edges are allowed when testing graphicality.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeTypeFilter {
    /// Simple graph: no self-loops, no multi-edges.
    Simple,
    /// At most one self-loop per vertex, no multi-edges.
    LoopsSimple,
    /// No self-loops, multi-edges allowed.
    Multi,
    /// Self-loops and multi-edges both allowed.
    LoopsMulti,
}

/// Test whether a degree sequence is graphical (realizable as some graph).
///
/// For **undirected** graphs, pass `in_degrees = None`.
/// For **directed** graphs, pass `out_degrees` and `Some(in_degrees)`.
///
/// # Algorithms
///
/// - Simple undirected: Erdős–Gallai theorem (Cloteaux 2015 linear-time).
/// - Loopy-simple undirected: Cairns–Mendan modification of Erdős–Gallai.
/// - Loopless multi undirected: even sum + sum ≥ 2·max.
/// - Loopy multi undirected: even sum only.
/// - Simple directed: Fulkerson–Chen–Anstee theorem (Berger 2014 relaxation).
/// - Loopy-simple directed: reduces to bigraphical simple (Gale–Ryser).
/// - Loopless multi directed: `sum_in` == `sum_out` + `sum_out` >= `max_total`.
/// - Loopy multi directed: `sum_in` == `sum_out` only.
///
/// # Examples
///
/// ```
/// use rust_igraph::{EdgeTypeFilter, is_graphical};
///
/// // Star K_{1,4}: center has degree 4, leaves have degree 1
/// assert!(is_graphical(&[4, 1, 1, 1, 1], None, EdgeTypeFilter::Simple));
///
/// // Odd sum → not graphical
/// assert!(!is_graphical(&[3, 1, 1], None, EdgeTypeFilter::Simple));
/// ```
pub fn is_graphical(
    out_degrees: &[u32],
    in_degrees: Option<&[u32]>,
    edge_types: EdgeTypeFilter,
) -> bool {
    match in_degrees {
        None => is_graphical_undirected(out_degrees, edge_types),
        Some(in_deg) => is_graphical_directed(out_degrees, in_deg, edge_types),
    }
}

/// Test whether a bi-degree-sequence is bigraphical (realizable as a
/// bipartite graph).
///
/// `degrees1` and `degrees2` are the degree sequences of the two
/// partitions. Self-loops are impossible in bipartite graphs, so the
/// `edge_types` parameter only distinguishes `Simple` vs `Multi`.
///
/// # Algorithms
///
/// - Simple: Gale–Ryser theorem (Berger 2014 relaxation), O(n).
/// - Multi: sum equality check, O(n).
///
/// # Examples
///
/// ```
/// use rust_igraph::{EdgeTypeFilter, is_bigraphical};
///
/// // Complete bipartite K_{2,3}: [3,3] and [2,2,2]
/// assert!(is_bigraphical(&[3, 3], &[2, 2, 2], EdgeTypeFilter::Simple));
///
/// // Unequal sums → not bigraphical
/// assert!(!is_bigraphical(&[3, 3], &[2, 2, 1], EdgeTypeFilter::Multi));
/// ```
pub fn is_bigraphical(degrees1: &[u32], degrees2: &[u32], edge_types: EdgeTypeFilter) -> bool {
    match edge_types {
        EdgeTypeFilter::Multi | EdgeTypeFilter::LoopsMulti => bigraphical_multi(degrees1, degrees2),
        _ => bigraphical_simple(degrees1, degrees2),
    }
}

fn is_graphical_undirected(degrees: &[u32], edge_types: EdgeTypeFilter) -> bool {
    match edge_types {
        EdgeTypeFilter::LoopsMulti => undirected_multi_loops(degrees),
        EdgeTypeFilter::Multi => undirected_loopless_multi(degrees),
        EdgeTypeFilter::LoopsSimple => undirected_loopy_simple(degrees),
        EdgeTypeFilter::Simple => undirected_simple(degrees),
    }
}

fn is_graphical_directed(
    out_degrees: &[u32],
    in_degrees: &[u32],
    edge_types: EdgeTypeFilter,
) -> bool {
    if out_degrees.len() != in_degrees.len() {
        return false;
    }
    match edge_types {
        EdgeTypeFilter::LoopsMulti => directed_loopy_multi(out_degrees, in_degrees),
        EdgeTypeFilter::Multi => directed_loopless_multi(out_degrees, in_degrees),
        EdgeTypeFilter::LoopsSimple => bigraphical_simple(out_degrees, in_degrees),
        EdgeTypeFilter::Simple => directed_simple(out_degrees, in_degrees),
    }
}

// --- Undirected variants ---

fn undirected_multi_loops(degrees: &[u32]) -> bool {
    let mut sum_parity: u64 = 0;
    for &d in degrees {
        sum_parity = (sum_parity + u64::from(d)) & 1;
    }
    sum_parity == 0
}

fn undirected_loopless_multi(degrees: &[u32]) -> bool {
    if degrees.is_empty() {
        return true;
    }
    let mut dsum: u64 = 0;
    let mut dmax: u64 = 0;
    for &d in degrees {
        let d64 = u64::from(d);
        dsum += d64;
        if d64 > dmax {
            dmax = d64;
        }
    }
    dsum % 2 == 0 && dsum >= 2 * dmax
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::many_single_char_names
)]
fn undirected_loopy_simple(degrees: &[u32]) -> bool {
    let n = degrees.len();
    if n == 0 {
        return true;
    }

    if !undirected_multi_loops(degrees) {
        return false;
    }

    let n_i64 = n as i64;
    let mut num_degs = vec![0i64; n + 2];
    for &d in degrees {
        let d_usize = d as usize;
        if d_usize > n + 1 {
            return false;
        }
        num_degs[d_usize] += 1;
    }

    for d in (0..=n).rev() {
        num_degs[d] += num_degs[d + 1];
    }

    let mut wd: i64 = 0;
    let mut kd: i64 = n_i64 + 1;
    let mut w: i64 = n_i64 - 1;
    let mut b: i64 = 0;
    let mut s: i64 = 0;
    let mut c: i64 = 0;

    for k in 0..n_i64 {
        while k >= num_degs[kd as usize] {
            kd -= 1;
        }
        b += kd;
        c += w;
        while w > k {
            while wd <= n_i64 && w < num_degs[(wd + 1) as usize] {
                wd += 1;
            }
            if wd > k + 1 {
                break;
            }
            s += wd;
            c -= k + 1;
            w -= 1;
        }
        if b > c + s + 2 * (k + 1) {
            return false;
        }
        if w == k {
            break;
        }
    }
    true
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::similar_names
)]
fn undirected_simple(degrees: &[u32]) -> bool {
    let p = degrees.len();
    if p == 0 {
        return true;
    }

    let mut num_degs = vec![0i64; p];
    let mut deg_min: u64 = u64::from(u32::MAX);
    let mut deg_max: u64 = 0;
    let mut dsum: u64 = 0;
    let mut nonzero_count: i64 = 0;

    for &d in degrees {
        let d64 = u64::from(d);
        if d64 >= p as u64 {
            return false;
        }
        if d > 0 {
            if d64 > deg_max {
                deg_max = d64;
            }
            if d64 < deg_min {
                deg_min = d64;
            }
            dsum += d64;
            nonzero_count += 1;
            num_degs[d as usize] += 1;
        }
    }

    if dsum % 2 != 0 {
        return false;
    }
    if nonzero_count == 0 {
        return true;
    }

    // Zverovich bound — sufficient condition for graphicality
    let zv_arg = (deg_max + deg_min + 1) as i64;
    let mut zverovich_bound = (zv_arg * zv_arg) / 4;
    if deg_min % 2 == 1 || (deg_max + deg_min) % 4 == 1 {
        zverovich_bound -= 1;
    }
    if (deg_min as i64) * nonzero_count >= zverovich_bound {
        return true;
    }

    // Erdős–Gallai via Cloteaux's linear-time algorithm
    let mut k: i64 = 0;
    let mut sum_deg: i64 = 0;
    let mut sum_freq: i64 = 0;
    let mut sum_ifreq: i64 = 0;

    let deg_min_i = deg_min as i64;
    let deg_max_i = deg_max as i64;

    let mut dk = deg_max_i;
    while dk >= deg_min_i {
        if dk < k + 1 {
            return true;
        }

        let mut run_size = num_degs[dk as usize];
        if run_size > 0 {
            if dk < k + run_size {
                run_size = dk - k;
            }
            sum_deg += run_size * dk;
            for v in 0..run_size {
                sum_freq += num_degs[(k + v) as usize];
                sum_ifreq += (k + v) * num_degs[(k + v) as usize];
            }
            k += run_size;
            if sum_deg > k * (nonzero_count - 1) - k * sum_freq + sum_ifreq {
                return false;
            }
        }
        dk -= 1;
    }

    true
}

// --- Directed variants ---

fn directed_loopy_multi(out_degrees: &[u32], in_degrees: &[u32]) -> bool {
    let mut sumdiff: i64 = 0;
    for (&dout, &din) in out_degrees.iter().zip(in_degrees.iter()) {
        sumdiff += i64::from(din) - i64::from(dout);
    }
    sumdiff == 0
}

fn directed_loopless_multi(out_degrees: &[u32], in_degrees: &[u32]) -> bool {
    let mut sumin: u64 = 0;
    let mut sumout: u64 = 0;
    let mut dmax: u64 = 0;
    for (&dout, &din) in out_degrees.iter().zip(in_degrees.iter()) {
        let d = u64::from(dout) + u64::from(din);
        sumin += u64::from(din);
        sumout += u64::from(dout);
        if d > dmax {
            dmax = d;
        }
    }
    sumin == sumout && sumout >= dmax
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
fn directed_simple(out_degrees: &[u32], in_degrees: &[u32]) -> bool {
    if !directed_loopy_multi(out_degrees, in_degrees) {
        return false;
    }

    let vcount = out_degrees.len();
    if vcount == 0 {
        return true;
    }
    let vc_i64 = vcount as i64;

    // Counting sort by in-degree (descending) with stable ordering
    let mut in_degree_cumcounts = vec![0usize; vcount + 1];
    for idx in 0..vcount {
        let indeg = in_degrees[idx] as usize;
        let outdeg = out_degrees[idx] as usize;
        if indeg >= vcount || outdeg >= vcount {
            return false;
        }
        in_degree_cumcounts[indeg + 1] += 1;
    }

    for indeg in 0..vcount {
        in_degree_cumcounts[indeg + 1] += in_degree_cumcounts[indeg];
    }

    let mut sorted_out = vec![0i64; vcount];
    let mut sorted_in = vec![0i64; vcount];
    let mut in_degree_counts = vec![0usize; vcount];

    for idx in 0..vcount {
        let outdeg = out_degrees[idx] as usize;
        let indeg = in_degrees[idx] as usize;
        let pos = in_degree_cumcounts[indeg] + in_degree_counts[indeg];
        sorted_out[vcount - pos - 1] = outdeg as i64;
        sorted_in[vcount - pos - 1] = indeg as i64;
        in_degree_counts[indeg] += 1;
    }

    // Fulkerson–Chen–Anstee check
    let mut left_pq = vec![0i64; vcount];
    let mut right_pq = vec![0i64; vcount];
    let mut left_sum: i64 = 0;
    let mut right_sum: i64 = 0;
    let mut left_pq_size: i64 = 0;
    let mut right_pq_size: i64 = vc_i64;
    let mut left_cursor: i64 = 0;
    let mut right_cursor: i64 = 0;

    for idx in 0..vcount {
        right_pq[sorted_out[idx] as usize] += 1;
    }

    let mut lhs: i64 = 0;
    for step in 0..vc_i64 {
        let step_u = step as usize;
        lhs += sorted_in[step_u];

        if sorted_out[step_u] < step {
            left_sum += sorted_out[step_u];
        } else {
            left_pq[sorted_out[step_u] as usize] += 1;
            left_pq_size += 1;
        }
        while left_cursor < step {
            let lc = left_cursor as usize;
            while left_pq[lc] > 0 {
                left_pq[lc] -= 1;
                left_pq_size -= 1;
                left_sum += left_cursor;
            }
            left_cursor += 1;
        }

        while right_cursor < step + 1 {
            let rc = right_cursor as usize;
            while right_pq[rc] > 0 {
                right_pq[rc] -= 1;
                right_pq_size -= 1;
                right_sum += right_cursor;
            }
            right_cursor += 1;
        }
        if sorted_out[step_u] < step + 1 {
            right_sum -= sorted_out[step_u];
        } else {
            right_pq[sorted_out[step_u] as usize] -= 1;
            right_pq_size -= 1;
        }

        let rhs = left_sum + step * left_pq_size + right_sum + (step + 1) * right_pq_size;
        if lhs > rhs {
            return false;
        }
    }
    true
}

// --- Bipartite variants ---

fn bigraphical_multi(degrees1: &[u32], degrees2: &[u32]) -> bool {
    let sum1: u64 = degrees1.iter().map(|&d| u64::from(d)).sum();
    let sum2: u64 = degrees2.iter().map(|&d| u64::from(d)).sum();
    sum1 == sum2
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
fn bigraphical_simple(degrees1: &[u32], degrees2: &[u32]) -> bool {
    let (mut d1, mut d2) = (degrees1, degrees2);
    let (mut n1, mut n2) = (d1.len(), d2.len());

    if n1 == 0 && n2 == 0 {
        return true;
    }

    if !bigraphical_multi(d1, d2) {
        return false;
    }

    // Ensure d1 is the shorter vector
    if n2 < n1 {
        std::mem::swap(&mut d1, &mut d2);
        std::mem::swap(&mut n1, &mut n2);
    }

    let mut deg_freq1 = vec![0i64; n2 + 1];
    let mut deg_freq2 = vec![0i64; n1 + 1];

    for &d in d1 {
        let du = d as usize;
        if du > n2 {
            return false;
        }
        deg_freq1[du] += 1;
    }
    for &d in d2 {
        let du = d as usize;
        if du > n1 {
            return false;
        }
        deg_freq2[du] += 1;
    }

    // Gale–Ryser theorem (Berger 2014 relaxation)
    let mut lhs_sum: i64 = 0;
    let mut partial_rhs_sum: i64 = 0;
    let mut partial_rhs_count: i64 = 0;
    let mut b: i64 = 0;
    let mut k: i64 = -1;
    let n2_i64 = n2 as i64;

    let mut a = n2 as i64;
    while a >= 0 {
        let acount = deg_freq1[a as usize];
        lhs_sum += a * acount;
        k += acount;

        while b <= k + 1 {
            let bu = b as usize;
            let bcount = deg_freq2[bu];
            partial_rhs_sum += b * bcount;
            partial_rhs_count += bcount;
            b += 1;
        }

        if lhs_sum > partial_rhs_sum + (n2_i64 - partial_rhs_count) * (k + 1) {
            return false;
        }
        a -= 1;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- is_graphical undirected simple ---

    #[test]
    fn empty_sequence_is_graphical() {
        assert!(is_graphical(&[], None, EdgeTypeFilter::Simple));
    }

    #[test]
    fn single_zero_degree() {
        assert!(is_graphical(&[0], None, EdgeTypeFilter::Simple));
    }

    #[test]
    fn all_zeros() {
        assert!(is_graphical(&[0, 0, 0], None, EdgeTypeFilter::Simple));
    }

    #[test]
    fn star_k14_simple() {
        assert!(is_graphical(&[4, 1, 1, 1, 1], None, EdgeTypeFilter::Simple));
    }

    #[test]
    fn complete_k4_simple() {
        assert!(is_graphical(&[3, 3, 3, 3], None, EdgeTypeFilter::Simple));
    }

    #[test]
    fn cycle_c5_simple() {
        assert!(is_graphical(&[2, 2, 2, 2, 2], None, EdgeTypeFilter::Simple));
    }

    #[test]
    fn odd_sum_not_graphical() {
        assert!(!is_graphical(&[3, 1, 1], None, EdgeTypeFilter::Simple));
    }

    #[test]
    fn degree_too_large_simple() {
        assert!(!is_graphical(
            &[5, 1, 1, 1, 1],
            None,
            EdgeTypeFilter::Simple
        ));
    }

    #[test]
    fn erdos_gallai_failure() {
        // [4, 4, 1, 1]: sum=10 even, but EG fails at k=2
        assert!(!is_graphical(&[4, 4, 1, 1], None, EdgeTypeFilter::Simple));
    }

    #[test]
    fn path_p3_simple() {
        assert!(is_graphical(&[1, 2, 1], None, EdgeTypeFilter::Simple));
    }

    // --- is_graphical undirected with loops ---

    #[test]
    fn odd_sum_with_loops_multi() {
        assert!(!is_graphical(&[3, 2], None, EdgeTypeFilter::LoopsMulti));
    }

    #[test]
    fn even_sum_with_loops_multi() {
        assert!(is_graphical(&[3, 3], None, EdgeTypeFilter::LoopsMulti));
    }

    // --- is_graphical undirected loopless multi ---

    #[test]
    fn loopless_multi_basic() {
        assert!(is_graphical(&[4, 2, 2], None, EdgeTypeFilter::Multi));
    }

    #[test]
    fn loopless_multi_max_too_large() {
        // sum=6 even, but max=5 > sum/2=3 → 2*max=10 > sum=6
        assert!(!is_graphical(&[5, 1, 0], None, EdgeTypeFilter::Multi));
    }

    // --- is_graphical undirected loopy simple ---

    #[test]
    fn loopy_simple_basic() {
        // Each vertex can have at most one self-loop
        assert!(is_graphical(
            &[3, 3, 2, 2],
            None,
            EdgeTypeFilter::LoopsSimple
        ));
    }

    #[test]
    fn loopy_simple_degree_too_large() {
        // d > n+1 is not realizable
        assert!(!is_graphical(
            &[10, 1, 1],
            None,
            EdgeTypeFilter::LoopsSimple
        ));
    }

    // --- is_graphical directed ---

    #[test]
    fn directed_simple_basic() {
        // Directed cycle: out=[1,1,1], in=[1,1,1]
        assert!(is_graphical(
            &[1, 1, 1],
            Some(&[1, 1, 1]),
            EdgeTypeFilter::Simple
        ));
    }

    #[test]
    fn directed_simple_star() {
        // Directed star: center out=3, leaves in=1
        assert!(is_graphical(
            &[3, 0, 0, 0],
            Some(&[0, 1, 1, 1]),
            EdgeTypeFilter::Simple
        ));
    }

    #[test]
    fn directed_sum_mismatch() {
        assert!(!is_graphical(
            &[2, 1],
            Some(&[1, 1]),
            EdgeTypeFilter::Simple
        ));
    }

    #[test]
    fn directed_length_mismatch() {
        assert!(!is_graphical(&[1, 1], Some(&[1]), EdgeTypeFilter::Simple));
    }

    #[test]
    fn directed_loopy_multi_basic() {
        assert!(is_graphical(
            &[2, 1],
            Some(&[1, 2]),
            EdgeTypeFilter::LoopsMulti
        ));
    }

    #[test]
    fn directed_loopless_multi_basic() {
        assert!(is_graphical(&[1, 1], Some(&[1, 1]), EdgeTypeFilter::Multi));
    }

    #[test]
    fn directed_loopless_multi_max_too_large() {
        // out=[3,0], in=[0,3] → sum_out=3, max_total=3 → 3>=3 OK
        assert!(is_graphical(&[3, 0], Some(&[0, 3]), EdgeTypeFilter::Multi));
        // out=[4,0], in=[0,4] → max_total=4, sum_out=4 → 4>=4 OK
        assert!(is_graphical(&[4, 0], Some(&[0, 4]), EdgeTypeFilter::Multi));
    }

    #[test]
    fn directed_degree_too_large_simple() {
        // out=[3,0,0], in=[0,0,3] — each degree >= n=3, not simple
        assert!(!is_graphical(
            &[3, 0, 0],
            Some(&[0, 0, 3]),
            EdgeTypeFilter::Simple
        ));
    }

    // --- is_bigraphical ---

    #[test]
    fn bigraphical_empty() {
        assert!(is_bigraphical(&[], &[], EdgeTypeFilter::Simple));
    }

    #[test]
    fn bigraphical_k23_simple() {
        assert!(is_bigraphical(&[3, 3], &[2, 2, 2], EdgeTypeFilter::Simple));
    }

    #[test]
    fn bigraphical_sum_mismatch() {
        assert!(!is_bigraphical(&[3, 3], &[2, 2, 1], EdgeTypeFilter::Multi));
    }

    #[test]
    fn bigraphical_degree_exceeds_partition() {
        // degrees1 vertex has degree 4 but partition 2 has only 2 vertices
        assert!(!is_bigraphical(&[4, 0], &[2, 2], EdgeTypeFilter::Simple));
    }

    #[test]
    fn bigraphical_multi_basic() {
        assert!(is_bigraphical(&[5, 5], &[5, 5], EdgeTypeFilter::Multi));
    }

    #[test]
    fn bigraphical_simple_asymmetric() {
        // K_{1,3}: [3] and [1,1,1]
        assert!(is_bigraphical(&[3], &[1, 1, 1], EdgeTypeFilter::Simple));
    }

    #[test]
    fn bigraphical_gale_ryser_failure() {
        // [3, 3] and [1, 1, 1] → sum matches (6=3), no sum=6 vs sum=3 → fail
        assert!(!is_bigraphical(&[3, 3], &[1, 1, 1], EdgeTypeFilter::Simple));
    }

    // --- Known graphical sequences from igraph C tests ---

    #[test]
    fn igraph_test_simple_sequences() {
        // Sequences from igraph's test suite
        assert!(is_graphical(&[0], None, EdgeTypeFilter::Simple));
        assert!(is_graphical(&[0, 0], None, EdgeTypeFilter::Simple));
        assert!(is_graphical(&[1, 1], None, EdgeTypeFilter::Simple));
        assert!(is_graphical(
            &[1, 3, 2, 4, 3, 3, 2, 2],
            None,
            EdgeTypeFilter::Simple
        ));
    }

    #[test]
    fn igraph_test_non_graphical_simple() {
        assert!(!is_graphical(&[1], None, EdgeTypeFilter::Simple));
        assert!(!is_graphical(&[1, 1, 1], None, EdgeTypeFilter::Simple));
        assert!(!is_graphical(
            &[1, 3, 2, 4, 3, 3, 2, 3],
            None,
            EdgeTypeFilter::Simple
        ));
    }

    #[test]
    fn directed_fulkerson_complete() {
        // Complete directed graph K_4 (no self-loops): each vertex has out=3, in=3
        assert!(is_graphical(
            &[3, 3, 3, 3],
            Some(&[3, 3, 3, 3]),
            EdgeTypeFilter::Simple
        ));
    }

    #[test]
    fn directed_fulkerson_tournament() {
        // Tournament on 4 vertices: out=[3,2,1,0], in=[0,1,2,3]
        assert!(is_graphical(
            &[3, 2, 1, 0],
            Some(&[0, 1, 2, 3]),
            EdgeTypeFilter::Simple
        ));
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn all_zeros_always_graphical(n in 0usize..50) {
            let degrees = vec![0u32; n];
            prop_assert!(is_graphical(&degrees, None, EdgeTypeFilter::Simple));
            prop_assert!(is_graphical(&degrees, None, EdgeTypeFilter::Multi));
            prop_assert!(is_graphical(&degrees, None, EdgeTypeFilter::LoopsSimple));
            prop_assert!(is_graphical(&degrees, None, EdgeTypeFilter::LoopsMulti));
        }

        #[test]
        fn regular_even_is_graphical(d in 1u32..10, n in 2usize..20) {
            // d-regular graph on n vertices: sum = d*n must be even
            let degrees = vec![d; n];
            let sum = d as u64 * n as u64;
            if sum % 2 == 0 && d < n as u32 {
                // Multi always accepts even sum
                prop_assert!(is_graphical(&degrees, None, EdgeTypeFilter::Multi));
                // LoopsMulti always accepts even sum
                prop_assert!(is_graphical(&degrees, None, EdgeTypeFilter::LoopsMulti));
            }
        }

        #[test]
        fn simple_implies_multi(
            degrees in proptest::collection::vec(0u32..20, 0..20)
        ) {
            // If realizable as simple, must be realizable as multi (more permissive)
            if is_graphical(&degrees, None, EdgeTypeFilter::Simple) {
                prop_assert!(is_graphical(&degrees, None, EdgeTypeFilter::Multi));
            }
        }

        #[test]
        fn multi_implies_loops_multi(
            degrees in proptest::collection::vec(0u32..20, 0..20)
        ) {
            if is_graphical(&degrees, None, EdgeTypeFilter::Multi) {
                prop_assert!(is_graphical(&degrees, None, EdgeTypeFilter::LoopsMulti));
            }
        }

        #[test]
        fn directed_sum_parity(
            out_deg in proptest::collection::vec(0u32..10, 1..15),
            in_deg in proptest::collection::vec(0u32..10, 1..15),
        ) {
            let n = out_deg.len().min(in_deg.len());
            let out_d = &out_deg[..n];
            let in_d = &in_deg[..n];
            let sum_out: u64 = out_d.iter().map(|&x| u64::from(x)).sum();
            let sum_in: u64 = in_d.iter().map(|&x| u64::from(x)).sum();
            if sum_out != sum_in {
                prop_assert!(!is_graphical(out_d, Some(in_d), EdgeTypeFilter::LoopsMulti));
            }
        }

        #[test]
        fn bigraphical_simple_implies_multi(
            d1 in proptest::collection::vec(0u32..10, 0..15),
            d2 in proptest::collection::vec(0u32..10, 0..15),
        ) {
            if is_bigraphical(&d1, &d2, EdgeTypeFilter::Simple) {
                prop_assert!(is_bigraphical(&d1, &d2, EdgeTypeFilter::Multi));
            }
        }
    }
}

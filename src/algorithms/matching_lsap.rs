//! Linear Sum Assignment Problem (ALGO-MA-005) — Hungarian method.
//!
//! Counterpart of `igraph_solve_lsap` in
//! `references/igraph/src/internal/lsap.c:664`.
//!
//! Solves the balanced assignment problem: given an n×n cost matrix,
//! find a permutation p such that Σ C\[i\]\[p\[i\]\] is minimized.
//!
//! ## Algorithm
//!
//! Classical Hungarian method (Kuhn-Munkres): O(n³).
//! 1. Subtract row and column minima (preprocessing).
//! 2. Greedily assign zeros.
//! 3. Iteratively cover rows/columns and reduce until all rows assigned.

use crate::core::{IgraphError, IgraphResult};

/// Solve a balanced linear sum assignment problem (Hungarian method).
///
/// Given an n×n cost matrix (stored row-major as a flat slice of length n²),
/// find an assignment of each row to exactly one column that minimizes the
/// total cost.
///
/// # Arguments
///
/// * `costs` — flat row-major n×n cost matrix (length must equal `n * n`).
/// * `n` — size of the problem (number of rows = number of columns).
///
/// # Returns
///
/// A vector `p` of length `n` where `p[i]` is the column assigned to row `i`.
///
/// # Errors
///
/// Returns an error if `costs.len() != n * n`, or if `n` is zero and costs
/// is non-empty, or if costs contain NaN.
///
/// # Examples
///
/// ```
/// use rust_igraph::solve_lsap;
///
/// // 3×3 cost matrix:
/// // [1, 2, 3]
/// // [2, 4, 6]
/// // [3, 6, 9]
/// // Optimal: row 0→col 2 (3), row 1→col 1 (4), row 2→col 0 (3) = 10
/// // OR: row 0→col 0 (1), row 1→col 1 (4), row 2→col 2 (9) = 14
/// // Actually optimal: 0→2(3), 1→0(2), 2→1(6) = 11
/// // Let's use a simpler example:
/// let costs = vec![
///     10.0, 5.0, 13.0,
///      3.0, 7.0,  2.0,
///      6.0, 8.0, 12.0,
/// ];
/// let p = solve_lsap(&costs, 3).unwrap();
/// // Verify it's a valid permutation
/// let mut used = vec![false; 3];
/// for &col in &p {
///     assert!(!used[col as usize]);
///     used[col as usize] = true;
/// }
/// ```
pub fn solve_lsap(costs: &[f64], n: usize) -> IgraphResult<Vec<u32>> {
    if n == 0 {
        if costs.is_empty() {
            return Ok(Vec::new());
        }
        return Err(IgraphError::InvalidArgument(
            "solve_lsap: n=0 but costs is non-empty".into(),
        ));
    }

    let expected_len = n
        .checked_mul(n)
        .ok_or_else(|| IgraphError::InvalidArgument("solve_lsap: n*n overflows".into()))?;
    if costs.len() != expected_len {
        return Err(IgraphError::InvalidArgument(format!(
            "solve_lsap: costs length {} != n*n = {}",
            costs.len(),
            expected_len
        )));
    }

    for (i, &v) in costs.iter().enumerate() {
        if v.is_nan() {
            return Err(IgraphError::InvalidArgument(format!(
                "solve_lsap: costs[{i}] is NaN"
            )));
        }
    }

    let assignment = hungarian(costs, n);
    Ok(assignment)
}

fn hungarian(costs: &[f64], n: usize) -> Vec<u32> {
    // Build reduced cost matrix (1-indexed internally for clarity)
    let mut c = vec![vec![0.0_f64; n + 1]; n + 1];
    for i in 1..=n {
        for j in 1..=n {
            c[i][j] = costs[(i - 1) * n + (j - 1)];
        }
    }

    preprocess(&mut c, n);

    // s[i] = column assigned to row i (0 = unassigned)
    let mut s = vec![0_usize; n + 1];
    // f[j] = row assigned to column j (0 = unassigned)
    let mut f = vec![0_usize; n + 1];
    let mut na = 0_usize;

    preassign(&c, n, &mut s, &mut f, &mut na);

    while na < n {
        let mut ri = vec![false; n + 1]; // covered rows
        let mut ci = vec![false; n + 1]; // covered columns

        if cover(&mut c, n, &mut s, &mut f, &mut na, &mut ri, &mut ci) {
            reduce(&mut c, n, &ri, &ci);
        }
    }

    // Convert to 0-based u32
    (1..=n)
        .map(|i| u32::try_from(s[i] - 1).unwrap_or(0))
        .collect()
}

#[allow(clippy::needless_range_loop)]
fn preprocess(c: &mut [Vec<f64>], n: usize) {
    // Subtract row minima
    for i in 1..=n {
        let mut min = c[i][1];
        for j in 2..=n {
            if c[i][j] < min {
                min = c[i][j];
            }
        }
        for j in 1..=n {
            c[i][j] -= min;
        }
    }

    // Subtract column minima
    for j in 1..=n {
        let mut min = c[1][j];
        for i in 2..=n {
            if c[i][j] < min {
                min = c[i][j];
            }
        }
        for i in 1..=n {
            c[i][j] -= min;
        }
    }
}

#[allow(clippy::needless_range_loop)]
fn preassign(c: &[Vec<f64>], n: usize, s: &mut [usize], f: &mut [usize], na: &mut usize) {
    *na = 0;
    let mut row_assigned = vec![false; n + 1];
    let mut col_assigned = vec![false; n + 1];

    // Count zeros in each row and column
    let mut rz = vec![0_usize; n + 1];
    let mut cz = vec![0_usize; n + 1];

    for i in 1..=n {
        for j in 1..=n {
            if c[i][j] == 0.0 {
                rz[i] += 1;
                cz[j] += 1;
            }
        }
    }

    loop {
        // Find unassigned row with fewest zeros > 0
        let mut best_row = 0;
        let mut best_count = usize::MAX;
        for i in 1..=n {
            if !row_assigned[i] && rz[i] > 0 && rz[i] < best_count {
                best_count = rz[i];
                best_row = i;
            }
        }
        if best_row == 0 {
            break;
        }

        // Find unassigned column in that row with fewest zeros
        let mut best_col = 0;
        let mut best_col_count = usize::MAX;
        for j in 1..=n {
            if c[best_row][j] == 0.0 && !col_assigned[j] && cz[j] < best_col_count {
                best_col_count = cz[j];
                best_col = j;
            }
        }

        if best_col != 0 {
            *na += 1;
            s[best_row] = best_col;
            f[best_col] = best_row;
            row_assigned[best_row] = true;
            col_assigned[best_col] = true;

            // Adjust zero counts for column best_col
            for i in 1..=n {
                if c[i][best_col] == 0.0 {
                    rz[i] = rz[i].saturating_sub(1);
                }
            }
            cz[best_col] = 0;
        } else {
            // No available column, mark row as having no usable zeros
            rz[best_row] = 0;
        }
    }
}

/// Attempt to extend the assignment. Returns true if reduction is needed.
#[allow(clippy::needless_range_loop, clippy::many_single_char_names)]
fn cover(
    c: &mut [Vec<f64>],
    n: usize,
    s: &mut [usize],
    f: &mut [usize],
    na: &mut usize,
    ri: &mut [bool],
    ci: &mut [bool],
) -> bool {
    // Reset cover indices
    let mut mr = vec![false; n + 1]; // marked rows
    for i in 1..=n {
        if s[i] == 0 {
            ri[i] = false; // uncovered
            mr[i] = true; // marked
        } else {
            ri[i] = true; // covered
        }
        ci[i] = false; // uncovered
    }

    loop {
        // Find a marked row
        let mut r = 0;
        for i in 1..=n {
            if mr[i] {
                r = i;
                break;
            }
        }
        if r == 0 {
            break;
        }

        // Look for uncovered zero in row r
        let mut found_augment = false;
        for j in 1..=n {
            if c[r][j] == 0.0 && !ci[j] {
                if f[j] != 0 {
                    // Column j is assigned to row f[j]: uncover that row
                    ri[f[j]] = false;
                    mr[f[j]] = true;
                    ci[j] = true;
                } else {
                    // Augmenting path found
                    if s[r] == 0 {
                        *na += 1;
                    }
                    // Unassign old column of row r
                    let old_col = s[r];
                    if old_col != 0 {
                        f[old_col] = 0;
                    }
                    f[j] = r;
                    s[r] = j;
                    found_augment = true;
                    break;
                }
            }
        }

        if found_augment {
            return false;
        }
        mr[r] = false;
    }

    true
}

#[allow(clippy::needless_range_loop)]
fn reduce(c: &mut [Vec<f64>], n: usize, ri: &[bool], ci: &[bool]) {
    // Find minimum uncovered element
    let mut min = f64::MAX;
    for i in 1..=n {
        if ri[i] {
            continue;
        }
        for j in 1..=n {
            if ci[j] {
                continue;
            }
            if c[i][j] < min {
                min = c[i][j];
            }
        }
    }

    // Subtract min from uncovered, add to doubly-covered
    for i in 1..=n {
        for j in 1..=n {
            if !ri[i] && !ci[j] {
                c[i][j] -= min;
            } else if ri[i] && ci[j] {
                c[i][j] += min;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_valid_permutation(p: &[u32], n: usize) -> bool {
        if p.len() != n {
            return false;
        }
        let mut used = vec![false; n];
        for &col in p {
            let c = col as usize;
            if c >= n || used[c] {
                return false;
            }
            used[c] = true;
        }
        true
    }

    fn assignment_cost(costs: &[f64], n: usize, p: &[u32]) -> f64 {
        (0..n).map(|i| costs[i * n + p[i] as usize]).sum()
    }

    #[test]
    fn lsap_empty() {
        let p = solve_lsap(&[], 0).unwrap();
        assert!(p.is_empty());
    }

    #[test]
    fn lsap_1x1() {
        let p = solve_lsap(&[42.0], 1).unwrap();
        assert_eq!(p, vec![0]);
    }

    #[test]
    fn lsap_2x2_identity() {
        // [1, 100]
        // [100, 1]
        // Optimal: (0,0) + (1,1) = 2
        let costs = vec![1.0, 100.0, 100.0, 1.0];
        let p = solve_lsap(&costs, 2).unwrap();
        assert!(is_valid_permutation(&p, 2));
        let cost = assignment_cost(&costs, 2, &p);
        assert!((cost - 2.0).abs() < 1e-10);
    }

    #[test]
    fn lsap_2x2_swap() {
        // [100, 1]
        // [1, 100]
        // Optimal: (0,1) + (1,0) = 2
        let costs = vec![100.0, 1.0, 1.0, 100.0];
        let p = solve_lsap(&costs, 2).unwrap();
        assert!(is_valid_permutation(&p, 2));
        let cost = assignment_cost(&costs, 2, &p);
        assert!((cost - 2.0).abs() < 1e-10);
    }

    #[test]
    fn lsap_3x3() {
        // Classic example:
        // [82, 83, 69]
        // [77, 37, 49]
        // [11, 69, 5]
        // Optimal: 0→2(69), 1→1(37), 2→0(11) = 117
        let costs = vec![82.0, 83.0, 69.0, 77.0, 37.0, 49.0, 11.0, 69.0, 5.0];
        let p = solve_lsap(&costs, 3).unwrap();
        assert!(is_valid_permutation(&p, 3));
        let cost = assignment_cost(&costs, 3, &p);
        // Verify against known optimum
        assert!((cost - 117.0).abs() < 1e-10);
    }

    #[test]
    fn lsap_4x4() {
        // [10, 5, 13, 15]
        // [ 3, 9, 18,  3]
        // [13, 6,  12, 14]
        // [12, 8, 14,  9]
        // Optimal: 0→1(5), 1→3(3), 2→2(12), 3→0(12) = 32
        // OR 0→1(5), 1→0(3), 2→2(12), 3→3(9) = 29
        let costs = vec![
            10.0, 5.0, 13.0, 15.0, 3.0, 9.0, 18.0, 3.0, 13.0, 6.0, 12.0, 14.0, 12.0, 8.0, 14.0, 9.0,
        ];
        let p = solve_lsap(&costs, 4).unwrap();
        assert!(is_valid_permutation(&p, 4));
        let cost = assignment_cost(&costs, 4, &p);
        // Check all possible assignments to find the minimum
        let min_cost = brute_force_min_cost(&costs, 4);
        assert!(
            (cost - min_cost).abs() < 1e-10,
            "Hungarian cost {cost} != brute force min {min_cost}"
        );
    }

    #[test]
    fn lsap_uniform() {
        // All costs equal: any permutation is optimal
        let costs = vec![5.0; 9];
        let p = solve_lsap(&costs, 3).unwrap();
        assert!(is_valid_permutation(&p, 3));
        let cost = assignment_cost(&costs, 3, &p);
        assert!((cost - 15.0).abs() < 1e-10);
    }

    #[test]
    fn lsap_diagonal() {
        // Diagonal is cheapest
        let n = 5;
        let mut costs = vec![100.0; n * n];
        for i in 0..n {
            costs[i * n + i] = 1.0;
        }
        let p = solve_lsap(&costs, n).unwrap();
        assert!(is_valid_permutation(&p, n));
        let cost = assignment_cost(&costs, n, &p);
        assert!((cost - 5.0).abs() < 1e-10);
    }

    #[test]
    fn lsap_invalid_size() {
        assert!(solve_lsap(&[1.0, 2.0], 2).is_err());
    }

    #[test]
    fn lsap_nan_cost() {
        assert!(solve_lsap(&[f64::NAN, 1.0, 1.0, 1.0], 2).is_err());
    }

    fn brute_force_min_cost(costs: &[f64], n: usize) -> f64 {
        let mut perm: Vec<usize> = (0..n).collect();
        let mut min_cost = f64::MAX;
        loop {
            let cost: f64 = (0..n).map(|i| costs[i * n + perm[i]]).sum();
            if cost < min_cost {
                min_cost = cost;
            }
            if !next_permutation(&mut perm) {
                break;
            }
        }
        min_cost
    }

    fn next_permutation(arr: &mut [usize]) -> bool {
        let n = arr.len();
        if n < 2 {
            return false;
        }
        let mut i = n - 1;
        while i > 0 && arr[i - 1] >= arr[i] {
            i -= 1;
        }
        if i == 0 {
            return false;
        }
        let mut j = n - 1;
        while arr[j] <= arr[i - 1] {
            j -= 1;
        }
        arr.swap(i - 1, j);
        arr[i..].reverse();
        true
    }
}

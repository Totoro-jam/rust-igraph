//! Random sampling without replacement (ALGO-NG-001).
//!
//! Counterpart of `igraph_random_sample()` from
//! `references/igraph/src/random/random.c`.
//!
//! Generates an increasing (sorted) random sequence of integers from a
//! given interval using Vitter's Algorithm D (1987), with Algorithm A
//! as a fallback for the tail. Expected time complexity is O(length).

use crate::core::IgraphResult;
use crate::core::error::IgraphError;
use crate::core::rng::SplitMix64;

/// Generate an increasing random sequence of integers from `[l, h]`.
///
/// Returns a sorted `Vec<i64>` of `length` distinct integers sampled
/// uniformly without replacement from the inclusive interval `[l, h]`.
/// The algorithm is Vitter's Algorithm D (1987), which runs in expected
/// O(`length`) time regardless of the interval size.
///
/// # Arguments
///
/// * `l` — lower bound of the sampling interval (inclusive).
/// * `h` — upper bound of the sampling interval (inclusive).
/// * `length` — number of integers to sample.
/// * `seed` — seed for the internal PRNG (deterministic for a given seed).
///
/// # Errors
///
/// Returns `InvalidArgument` if:
/// - `l > h` (empty interval).
/// - `length` exceeds the number of integers in `[l, h]`.
///
/// # Examples
///
/// ```
/// use rust_igraph::random_sample;
///
/// // Sample 5 integers from [1, 100]
/// let sample = random_sample(1, 100, 5, 42).unwrap();
/// assert_eq!(sample.len(), 5);
/// // Result is sorted ascending
/// for w in sample.windows(2) {
///     assert!(w[0] < w[1]);
/// }
/// // All values in range
/// for &v in &sample {
///     assert!(v >= 1 && v <= 100);
/// }
/// ```
pub fn random_sample(l: i64, h: i64, length: usize, seed: u64) -> IgraphResult<Vec<i64>> {
    if l > h {
        return Err(IgraphError::InvalidArgument(
            "random_sample: lower limit is greater than upper limit".to_string(),
        ));
    }

    let pool_size = (h - l).checked_add(1).ok_or_else(|| {
        IgraphError::InvalidArgument("random_sample: interval overflows".to_string())
    })?;
    let pool_size_u = u64::try_from(pool_size).map_err(|_| {
        IgraphError::InvalidArgument("random_sample: interval overflows u64".to_string())
    })?;

    if (length as u64) > pool_size_u {
        return Err(IgraphError::InvalidArgument(
            "random_sample: sample size exceeds size of candidate pool".to_string(),
        ));
    }

    if l == h {
        return Ok(vec![l]);
    }
    if length == 0 {
        return Ok(Vec::new());
    }
    let length_u64 = length as u64;
    if length_u64 == pool_size_u {
        return Ok((l..=h).collect());
    }

    let mut rng = SplitMix64::new(seed);
    let mut result = Vec::with_capacity(length);

    let mut n_real = length as f64;
    #[allow(clippy::float_cmp)]
    let n_inv = if n_real == 0.0 { 0.0 } else { 1.0 / n_real };
    let mut big_n = pool_size_u;
    let mut big_n_real = big_n as f64;
    let mut vprime = rng.gen_unit().powf(n_inv);
    let mut cur = l - 1;
    let mut n_remaining = length as u64;
    let mut qu1 = big_n.wrapping_sub(n_remaining).wrapping_add(1);
    let mut qu1_real = big_n_real - n_real + 1.0;
    let neg_alpha_inv: f64 = -13.0;
    let mut threshold = (neg_alpha_inv * n_real).abs();

    while n_remaining > 1 && threshold < big_n_real {
        let nmin1inv = 1.0 / (-1.0 + n_real);

        let skip;
        loop {
            let mut x;
            loop {
                x = big_n_real * (1.0 - vprime);
                let s_candidate = x as u64;
                if s_candidate < qu1 {
                    break;
                }
                vprime = rng.gen_unit().powf(n_inv);
            }

            let s = x as u64;
            let u = rng.gen_unit();
            let neg_s_real = -(s as f64);

            let y1 = (u * big_n_real / qu1_real).powf(nmin1inv);
            vprime = y1 * (1.0 - x / big_n_real) * (qu1_real / (neg_s_real + qu1_real));
            if vprime <= 1.0 {
                skip = s;
                break;
            }

            let mut y2 = 1.0_f64;
            let mut top = big_n_real - 1.0;
            let (mut bottom, limit);
            if n_remaining - 1 > s {
                bottom = big_n_real - n_real;
                limit = big_n - s;
            } else {
                bottom = big_n_real - 1.0 + neg_s_real;
                limit = qu1;
            }

            let mut t = big_n - 1;
            while t >= limit {
                y2 = (y2 * top) / bottom;
                top -= 1.0;
                bottom -= 1.0;
                t -= 1;
            }

            if big_n_real / (big_n_real - x) >= y1 * y2.powf(nmin1inv) {
                vprime = rng.gen_unit().powf(nmin1inv);
                skip = s;
                break;
            }
            vprime = rng.gen_unit().powf(n_inv);
        }

        cur = cur.checked_add(skip as i64 + 1).ok_or_else(|| {
            IgraphError::InvalidArgument("random_sample: overflow in position".to_string())
        })?;
        result.push(cur);

        big_n -= skip + 1;
        big_n_real -= skip as f64 + 1.0;
        n_remaining -= 1;
        n_real -= 1.0;
        qu1 -= skip;
        qu1_real -= skip as f64;
        threshold -= neg_alpha_inv;
    }

    if n_remaining > 1 {
        algorithm_a(&mut rng, &mut result, cur + 1, h, n_remaining);
    } else {
        let s = (big_n_real * vprime) as i64;
        cur = cur.checked_add(s + 1).ok_or_else(|| {
            IgraphError::InvalidArgument("random_sample: overflow in final position".to_string())
        })?;
        result.push(cur);
    }

    Ok(result)
}

/// Vitter's Algorithm A — simple sequential sampling fallback.
fn algorithm_a(rng: &mut SplitMix64, result: &mut Vec<i64>, l: i64, h: i64, length: u64) {
    let mut big_n = (h - l + 1) as f64;
    let mut n = length;
    let mut cur = l - 1;

    while n >= 2 {
        let v = rng.gen_unit();
        let mut s: i64 = 1;
        let mut quot = (big_n - n as f64) / big_n;
        while quot > v {
            s += 1;
            big_n -= 1.0;
            quot = (quot * (big_n - n as f64)) / big_n;
        }
        cur += s;
        result.push(cur);
        big_n -= 1.0;
        n -= 1;
    }

    let s = (big_n * rng.gen_unit()).trunc() as i64;
    cur += s + 1;
    result.push(cur);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_sample() {
        let result = random_sample(1, 100, 0, 42).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn single_element_interval() {
        let result = random_sample(5, 5, 1, 42).unwrap();
        assert_eq!(result, vec![5]);
    }

    #[test]
    fn full_interval() {
        let result = random_sample(10, 15, 6, 42).unwrap();
        assert_eq!(result, vec![10, 11, 12, 13, 14, 15]);
    }

    #[test]
    fn error_l_greater_than_h() {
        assert!(random_sample(10, 5, 1, 42).is_err());
    }

    #[test]
    fn error_length_exceeds_pool() {
        assert!(random_sample(1, 5, 10, 42).is_err());
    }

    #[test]
    fn result_is_sorted_ascending() {
        let result = random_sample(0, 1_000_000, 100, 12345).unwrap();
        assert_eq!(result.len(), 100);
        for w in result.windows(2) {
            assert!(w[0] < w[1], "not sorted: {} >= {}", w[0], w[1]);
        }
    }

    #[test]
    fn all_values_in_range() {
        let result = random_sample(-50, 50, 30, 99).unwrap();
        assert_eq!(result.len(), 30);
        for &v in &result {
            assert!(v >= -50 && v <= 50, "value {v} out of range");
        }
    }

    #[test]
    fn no_duplicates() {
        let result = random_sample(1, 1000, 200, 777).unwrap();
        let mut deduped = result.clone();
        deduped.dedup();
        assert_eq!(result.len(), deduped.len());
    }

    #[test]
    fn deterministic_same_seed() {
        let a = random_sample(0, 999, 50, 42).unwrap();
        let b = random_sample(0, 999, 50, 42).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn different_seeds_differ() {
        let a = random_sample(0, 999, 50, 1).unwrap();
        let b = random_sample(0, 999, 50, 2).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn negative_range() {
        let result = random_sample(-100, -1, 10, 42).unwrap();
        assert_eq!(result.len(), 10);
        for &v in &result {
            assert!(v >= -100 && v <= -1);
        }
        for w in result.windows(2) {
            assert!(w[0] < w[1]);
        }
    }

    #[test]
    fn large_interval_small_sample() {
        let result = random_sample(0, 1_000_000_000, 10, 42).unwrap();
        assert_eq!(result.len(), 10);
        for w in result.windows(2) {
            assert!(w[0] < w[1]);
        }
    }

    #[test]
    fn sample_size_one() {
        let result = random_sample(1, 100, 1, 42).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0] >= 1 && result[0] <= 100);
    }

    #[test]
    fn sample_size_two() {
        let result = random_sample(1, 100, 2, 42).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result[0] < result[1]);
    }

    #[test]
    fn sample_nearly_full() {
        let result = random_sample(1, 20, 19, 42).unwrap();
        assert_eq!(result.len(), 19);
        for w in result.windows(2) {
            assert!(w[0] < w[1]);
        }
        for &v in &result {
            assert!(v >= 1 && v <= 20);
        }
    }

    #[test]
    fn statistical_uniformity() {
        // Over many runs with different seeds, each value in [0, 9] should
        // appear roughly equally often when sampling 1 from [0, 9].
        let mut counts = [0u32; 10];
        for seed in 0..10_000u64 {
            let result = random_sample(0, 9, 1, seed).unwrap();
            counts[result[0] as usize] += 1;
        }
        for (i, &c) in counts.iter().enumerate() {
            let expected = 1000.0;
            let deviation = (c as f64 - expected).abs() / expected;
            assert!(
                deviation < 0.1,
                "value {i} appeared {c} times (expected ~{expected})"
            );
        }
    }
}

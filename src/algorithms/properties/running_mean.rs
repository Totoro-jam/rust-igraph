//! Running mean (ALGO-PR-037).
//!
//! Counterpart of `igraph_running_mean()` from
//! `references/igraph/src/misc/other.c`.
//!
//! Computes the running mean of a data vector with a given bin width.

use crate::core::{IgraphError, IgraphResult};

/// Compute the running mean of a data vector.
///
/// For a data vector of length `n` and bin width `w`, returns a vector
/// of length `n - w + 1` where entry `i` is the mean of
/// `data[i..i+w]`.
///
/// # Errors
///
/// - `InvalidArgument` if `binwidth < 1`.
/// - `InvalidArgument` if `data.len() < binwidth`.
///
/// # Examples
///
/// ```
/// use rust_igraph::running_mean;
///
/// let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
/// let result = running_mean(&data, 3).unwrap();
/// assert_eq!(result.len(), 3);
/// assert!((result[0] - 2.0).abs() < 1e-10); // mean(1,2,3)
/// assert!((result[1] - 3.0).abs() < 1e-10); // mean(2,3,4)
/// assert!((result[2] - 4.0).abs() < 1e-10); // mean(3,4,5)
/// ```
pub fn running_mean(data: &[f64], binwidth: usize) -> IgraphResult<Vec<f64>> {
    if binwidth < 1 {
        return Err(IgraphError::InvalidArgument(
            "running_mean: binwidth must be at least 1".into(),
        ));
    }

    if data.len() < binwidth {
        return Err(IgraphError::InvalidArgument(format!(
            "running_mean: data length {} is smaller than binwidth {binwidth}",
            data.len()
        )));
    }

    let result_len = data.len() - binwidth + 1;
    let mut result: Vec<f64> = Vec::with_capacity(result_len);

    #[allow(clippy::cast_precision_loss)]
    let bw_f64 = binwidth as f64;

    let mut sum: f64 = data[..binwidth].iter().sum();
    result.push(sum / bw_f64);

    for i in 1..result_len {
        sum -= data[i - 1];
        sum += data[i + binwidth - 1];
        result.push(sum / bw_f64);
    }

    Ok(result)
}

/// Expand a vertex path into consecutive pairs for edge lookup.
///
/// Converts `[v0, v1, v2, v3]` into `[(v0, v1), (v1, v2), (v2, v3)]`.
/// Useful for converting a path of vertex IDs into pairs that can be
/// passed to edge-lookup functions.
///
/// Returns an empty vector if the path has fewer than 2 elements.
///
/// # Examples
///
/// ```
/// use rust_igraph::expand_path_to_pairs;
///
/// let path = vec![0, 1, 2, 3];
/// let pairs = expand_path_to_pairs(&path);
/// assert_eq!(pairs, vec![(0, 1), (1, 2), (2, 3)]);
/// ```
pub fn expand_path_to_pairs(path: &[u32]) -> Vec<(u32, u32)> {
    if path.len() < 2 {
        return Vec::new();
    }
    path.windows(2).map(|w| (w[0], w[1])).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn running_mean_basic() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let r = running_mean(&data, 3).unwrap();
        assert_eq!(r.len(), 3);
        assert!((r[0] - 2.0).abs() < 1e-10);
        assert!((r[1] - 3.0).abs() < 1e-10);
        assert!((r[2] - 4.0).abs() < 1e-10);
    }

    #[test]
    fn running_mean_binwidth_1() {
        let data = vec![5.0, 3.0, 7.0];
        let r = running_mean(&data, 1).unwrap();
        assert_eq!(r.len(), 3);
        assert!((r[0] - 5.0).abs() < 1e-10);
        assert!((r[1] - 3.0).abs() < 1e-10);
        assert!((r[2] - 7.0).abs() < 1e-10);
    }

    #[test]
    fn running_mean_binwidth_equals_len() {
        let data = vec![1.0, 2.0, 3.0];
        let r = running_mean(&data, 3).unwrap();
        assert_eq!(r.len(), 1);
        assert!((r[0] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn running_mean_binwidth_too_large() {
        let data = vec![1.0, 2.0];
        let err = running_mean(&data, 3).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn running_mean_binwidth_zero() {
        let data = vec![1.0, 2.0, 3.0];
        let err = running_mean(&data, 0).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn running_mean_constant() {
        let data = vec![4.0; 10];
        let r = running_mean(&data, 5).unwrap();
        assert_eq!(r.len(), 6);
        for v in &r {
            assert!((*v - 4.0).abs() < 1e-10);
        }
    }

    #[test]
    fn expand_pairs_basic() {
        let pairs = expand_path_to_pairs(&[0, 1, 2, 3]);
        assert_eq!(pairs, vec![(0, 1), (1, 2), (2, 3)]);
    }

    #[test]
    fn expand_pairs_single_edge() {
        let pairs = expand_path_to_pairs(&[5, 10]);
        assert_eq!(pairs, vec![(5, 10)]);
    }

    #[test]
    fn expand_pairs_empty() {
        let pairs = expand_path_to_pairs(&[]);
        assert!(pairs.is_empty());
    }

    #[test]
    fn expand_pairs_single_vertex() {
        let pairs = expand_path_to_pairs(&[7]);
        assert!(pairs.is_empty());
    }

    #[test]
    fn expand_pairs_cycle() {
        let pairs = expand_path_to_pairs(&[0, 1, 2, 0]);
        assert_eq!(pairs, vec![(0, 1), (1, 2), (2, 0)]);
    }
}

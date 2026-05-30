//! Power-law fitting (ALGO-PR-019).
//!
//! Pure-Rust port of `igraph_power_law_fit()` from
//! `references/igraph/src/misc/power_law_fit.c`, which wraps the bundled
//! `plfit` library (`references/igraph/vendor/plfit/`). Implements the
//! Clauset–Shalizi–Newman maximum-likelihood method for fitting a
//! power-law tail `P(X = x) ∝ x^(-alpha)` for `x >= xmin`.
//!
//! Two regimes, selected exactly as upstream:
//! - **continuous**: closed-form MLE `alpha = 1 + m / Σ log(x_i / xmin)`.
//! - **discrete**: `alpha` maximises the zeta log-likelihood
//!   `-alpha·Σ log(x_i) - m·ln ζ(alpha, xmin)` (upstream uses L-BFGS; we
//!   use a derivative-free golden-section search on this strictly concave
//!   objective, which converges to the same maximiser).
//!
//! When `xmin < 0` the optimal `xmin` is chosen to minimise the
//! Kolmogorov–Smirnov statistic `D`, faithfully reproducing the
//! deterministic continuous stratified-sampling search and the discrete
//! unique-value scan. The p-value is **not** computed (upstream sets
//! `PLFIT_P_VALUE_SKIP` in the default fit), matching `igraph`.
//!
//! The Hurwitz zeta `ζ(s, q) = Σ_{k>=0} (k + q)^(-s)` is evaluated by the
//! Euler–Maclaurin summation used in `vendor/plfit/hzeta.c`.

#![allow(
    // Sample counts and ranks are well below 2^52, so the f64 casts are exact.
    clippy::cast_precision_loss,
    // Integrality and exact-cutoff tests are intentional bit-exact comparisons,
    // matching the upstream plfit logic.
    clippy::float_cmp,
    // The Euler–Maclaurin coefficients are transcribed verbatim from
    // vendor/plfit/hzeta.c; the surplus digits keep the port visually faithful.
    clippy::excessive_precision,
    // Faithful numeric ports: terse single-letter names (a/b/c/d, s/q, m/n) and
    // the coefficient-indexing loop mirror the C source and read clearest as-is.
    clippy::many_single_char_names,
    clippy::needless_range_loop
)]

use crate::core::{IgraphError, IgraphResult};

/// Result of fitting a power-law distribution to a sample.
///
/// Mirrors the externally observable fields of `igraph_plfit_result_t`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PowerLawFitResult {
    /// `true` if a continuous model was fitted, `false` for discrete.
    pub continuous: bool,
    /// The exponent of the fitted power-law (the scaling parameter).
    pub alpha: f64,
    /// The lower cutoff `xmin`: the power-law holds for `x >= xmin`.
    pub xmin: f64,
    /// Log-likelihood of the fitted parameters on the sample.
    pub log_likelihood: f64,
    /// Kolmogorov–Smirnov test statistic between fit and sample.
    pub ks_statistic: f64,
}

// ---------------------------------------------------------------------------
// Hurwitz zeta via Euler–Maclaurin (port of hsl_sf_hzeta_e, hzeta.c).
// ---------------------------------------------------------------------------

const EM_SHIFT: usize = 10;
const EM_ORDER: usize = 32;
const DBL_EPSILON: f64 = 2.220_446_049_250_313_1e-16;
const LOG_DBL_MIN: f64 = -7.083_964_185_322_640_8e2;
const LOG_DBL_MAX: f64 = 7.097_827_128_933_839_7e2;

/// `B_{2j} / (2j)!` Euler–Maclaurin coefficients (`coeffs[0] = 1`).
const EM_COEFFS: [f64; EM_ORDER + 1] = [
    1.0,
    1.0 / 12.0,
    -1.0 / 720.0,
    1.0 / 30240.0,
    -1.0 / 1_209_600.0,
    1.0 / 47_900_160.0,
    -691.0 / 1_307_674_368_000.0,
    1.0 / 74_724_249_600.0,
    -3.389_680_296_322_582_8e-13,
    8.586_062_056_277_844_5e-15,
    -2.174_868_698_558_061_9e-16,
    5.509_002_828_360_230e-18,
    -1.395_446_468_581_252_3e-19,
    3.534_707_039_629_467_5e-21,
    -8.953_517_427_037_547e-23,
    2.267_952_452_337_683e-24,
    -5.744_790_668_872_202_5e-26,
    1.455_172_475_614_865e-27,
    -3.685_994_940_665_31e-29,
    9.336_734_257_095_045e-31,
    -2.365_022_415_700_63e-32,
    5.990_671_762_482_134e-34,
    -1.517_454_884_468_290_3e-35,
    3.843_758_125_454_188e-37,
    -9.736_353_072_646_691e-39,
    2.466_247_044_200_681e-40,
    -6.247_076_741_820_744e-42,
    1.582_403_024_464_491_4e-43,
    -4.008_273_685_948_936e-45,
    1.015_307_585_556_955_6e-46,
    -2.571_804_158_241_871_7e-48,
    6.514_456_035_233_815e-50,
    -1.650_130_990_689_652_4e-51,
];

/// Hurwitz zeta `ζ(s, q) = Σ_{k>=0} (k + q)^(-s)` for `s > 1`, `q > 0`.
fn hzeta(s: f64, q: f64) -> IgraphResult<f64> {
    if s <= 1.0 || q <= 0.0 {
        return Err(IgraphError::InvalidArgument(
            "power_law_fit: Hurwitz zeta requires s > 1 and q > 0".into(),
        ));
    }

    let ln_term0 = -s * q.ln();
    if ln_term0 < LOG_DBL_MIN + 1.0 {
        return Err(IgraphError::InvalidArgument(
            "power_law_fit: underflow while evaluating Hurwitz zeta".into(),
        ));
    }
    if ln_term0 > LOG_DBL_MAX - 1.0 {
        return Err(IgraphError::InvalidArgument(
            "power_law_fit: overflow while evaluating Hurwitz zeta".into(),
        ));
    }

    let max_bits = 54.0;
    // Fast paths for large s / small q (q >= 1 in our callers, so these
    // are inert there, but kept for faithfulness on general input).
    if ((max_bits < s) && (q < 1.0)) || ((0.5 * max_bits < s) && (q < 0.25)) {
        return Ok(q.powf(-s));
    }
    if (0.5 * max_bits < s) && (q < 1.0) {
        let a0 = q.powf(-s);
        let p1 = (q / (1.0 + q)).powf(s);
        let p2 = (q / (2.0 + q)).powf(s);
        return Ok(a0 * (1.0 + p1 + p2));
    }

    // Euler–Maclaurin summation. Terms are accumulated, then re-summed in
    // reverse for floating-point accuracy, exactly as in hzeta.c.
    let qshift = EM_SHIFT as f64 + q;
    let inv_qshift = 1.0 / qshift;
    let sqr_inv_qshift = inv_qshift * inv_qshift;
    let inv_sm1 = 1.0 / (s - 1.0);
    let pmax = qshift.powf(-s);

    let mut terms: Vec<f64> = Vec::with_capacity(EM_SHIFT + EM_ORDER + 2);

    for j in 0..EM_SHIFT {
        terms.push((j as f64 + q).powf(-s));
    }
    terms.push(0.5 * pmax);
    terms.push(pmax * qshift * inv_sm1);

    let mut tscp = s;
    let mut scp = tscp;
    let mut pcp = pmax * inv_qshift;
    let mut ratio = scp * pcp;
    let mut last_j = EM_ORDER;
    let mut ans: f64 = terms.iter().sum();

    for j in 1..=EM_ORDER {
        let delta = EM_COEFFS[j] * ratio;
        terms.push(delta);
        ans += delta;
        tscp += 1.0;
        scp *= tscp;
        tscp += 1.0;
        scp *= tscp;
        pcp *= sqr_inv_qshift;
        ratio = scp * pcp;
        if (delta / ans).abs() < 0.5 * DBL_EPSILON {
            last_j = j;
            break;
        }
    }
    let _ = last_j;

    let mut acc = 0.0;
    for &t in terms.iter().rev() {
        acc += t;
    }
    Ok(acc)
}

/// `ln ζ(s, q)`. For the moderate `(alpha, xmin)` range that arises in
/// power-law fitting, `ln(hzeta)` is fully accurate.
fn lnhzeta(s: f64, q: f64) -> IgraphResult<f64> {
    Ok(hzeta(s, q)?.ln())
}

// ---------------------------------------------------------------------------
// Continuous case.
// ---------------------------------------------------------------------------

/// MLE of `alpha` on a sorted slice whose elements are all `>= xmin`.
fn estimate_alpha_continuous(cut: &[f64], xmin: f64) -> IgraphResult<f64> {
    if cut.is_empty() {
        return Err(IgraphError::InvalidArgument(
            "power_law_fit: no data point was larger than xmin".into(),
        ));
    }
    let logsum: f64 = cut.iter().map(|&x| (x / xmin).ln()).sum();
    Ok(1.0 + (cut.len() as f64) / logsum)
}

/// KS statistic for the continuous fit on a sorted cut slice.
fn ks_continuous(cut: &[f64], alpha: f64, xmin: f64) -> f64 {
    let n = cut.len() as f64;
    let mut d_max = 0.0_f64;
    for (m, &x) in cut.iter().enumerate() {
        let d = (1.0 - (xmin / x).powf(alpha - 1.0) - (m as f64) / n).abs();
        if d > d_max {
            d_max = d;
        }
    }
    d_max
}

/// Continuous log-likelihood on a sorted cut slice.
fn log_likelihood_continuous(cut: &[f64], alpha: f64, xmin: f64) -> f64 {
    let m = cut.len() as f64;
    let logsum: f64 = cut.iter().map(|&x| (x / xmin).ln()).sum();
    let c = (alpha - 1.0) / xmin;
    -alpha * logsum + c.ln() * m
}

fn finite_size_correction(alpha: f64, n: usize) -> f64 {
    let nf = n as f64;
    alpha * (nf - 1.0) / nf + 1.0 / nf
}

/// One xmin probe for the continuous search: returns `(alpha, D)`.
fn eval_continuous_probe(sorted: &[f64], start: usize) -> IgraphResult<(f64, f64)> {
    let cut = &sorted[start..];
    let xmin = sorted[start];
    let alpha = estimate_alpha_continuous(cut, xmin)?;
    let d = ks_continuous(cut, alpha, xmin);
    Ok((alpha, d))
}

/// Deterministic linear scan over candidate start indices; the last probe
/// is excluded (matches `i < num_probes - 1` upstream). Returns the best
/// `(start_index, alpha, xmin, D)`.
fn linear_scan_continuous(
    sorted: &[f64],
    probe_starts: &[usize],
) -> IgraphResult<Option<(usize, f64, f64)>> {
    let mut best: Option<(usize, f64, f64)> = None;
    let mut best_d = f64::MAX;
    if probe_starts.len() < 2 {
        return Ok(None);
    }
    for &start in &probe_starts[..probe_starts.len() - 1] {
        let (alpha, d) = eval_continuous_probe(sorted, start)?;
        if d < best_d {
            best_d = d;
            best = Some((start, alpha, d));
        }
    }
    Ok(best)
}

/// Indices in `sorted` where each distinct value first appears.
fn unique_starts(sorted: &[f64]) -> Vec<usize> {
    let mut starts = Vec::new();
    let mut prev = f64::NAN;
    for (i, &x) in sorted.iter().enumerate() {
        if i == 0 || x != prev {
            starts.push(i);
            prev = x;
        }
    }
    starts
}

fn fit_continuous_search(sorted: &[f64]) -> IgraphResult<PowerLawFitResult> {
    let n = sorted.len();
    let uniques = unique_starts(sorted);
    let num_uniques = uniques.len();

    // Stratified sampling (PLFIT_STRATIFIED_SAMPLING), deterministic.
    let mut best: Option<(usize, f64, f64)> = None;
    if num_uniques >= 50 {
        let subdivision = 10usize;
        let num_strata = num_uniques / subdivision;
        let strata: Vec<usize> = (0..num_strata).map(|i| uniques[i * subdivision]).collect();
        if let Some((bstart, balpha, bd)) = linear_scan_continuous(sorted, &strata)? {
            // Locate the winning stratum, then scan the window around it.
            let si = (0..num_strata)
                .find(|&i| sorted[strata[i]] == sorted[bstart])
                .unwrap_or(0);
            let lo = if si > 0 { (si - 1) * subdivision } else { 0 };
            let mut count = 0usize;
            if si != 0 {
                count += subdivision;
            }
            if si != num_strata - 1 {
                count += subdivision;
            }
            if count > 0 {
                let hi = (lo + count).min(num_uniques);
                let window = &uniques[lo..hi];
                best = linear_scan_continuous(sorted, window)?;
            }
            if best.is_none() {
                best = Some((bstart, balpha, bd));
            }
        }
    }

    // Fallback: full linear scan over all unique values.
    if best.is_none() {
        best = linear_scan_continuous(sorted, &uniques)?;
    }

    let (start, alpha, d) = best.ok_or_else(|| {
        IgraphError::InvalidArgument("power_law_fit: could not fit continuous power-law".into())
    })?;
    let xmin = sorted[start];
    let best_n = n - start;
    let cut = &sorted[start..];

    let final_alpha = if n < 50 {
        finite_size_correction(alpha, best_n)
    } else {
        alpha
    };
    let l = log_likelihood_continuous(cut, final_alpha, xmin);

    Ok(PowerLawFitResult {
        continuous: true,
        alpha: final_alpha,
        xmin,
        log_likelihood: l,
        ks_statistic: d,
    })
}

fn fit_continuous_fixed(sorted: &[f64], xmin: f64) -> IgraphResult<PowerLawFitResult> {
    if xmin <= 0.0 {
        return Err(IgraphError::InvalidArgument(
            "power_law_fit: xmin must be greater than zero".into(),
        ));
    }
    let start = sorted.partition_point(|&x| x < xmin);
    let cut = &sorted[start..];
    let alpha = estimate_alpha_continuous(cut, xmin)?;
    let d = ks_continuous(cut, alpha, xmin);
    let total_n = sorted.len();
    let final_alpha = if total_n < 50 {
        finite_size_correction(alpha, cut.len())
    } else {
        alpha
    };
    let l = log_likelihood_continuous(cut, final_alpha, xmin);
    Ok(PowerLawFitResult {
        continuous: true,
        alpha: final_alpha,
        xmin,
        log_likelihood: l,
        ks_statistic: d,
    })
}

// ---------------------------------------------------------------------------
// Discrete case.
// ---------------------------------------------------------------------------

/// Maximise the discrete log-likelihood over `alpha > 1` by golden-section
/// search on the strictly concave objective. Replaces upstream L-BFGS.
fn estimate_alpha_discrete(cut: &[f64], xmin: f64) -> IgraphResult<f64> {
    if cut.is_empty() {
        return Err(IgraphError::InvalidArgument(
            "power_law_fit: no data point was larger than xmin".into(),
        ));
    }
    let logsum: f64 = cut.iter().map(|&x| x.ln()).sum();
    let m = cut.len() as f64;

    // Negative log-likelihood to minimise.
    let nll = |alpha: f64| -> IgraphResult<f64> { Ok(alpha * logsum + m * lnhzeta(alpha, xmin)?) };

    let inv_phi = (5.0_f64.sqrt() - 1.0) / 2.0; // 1/φ
    let mut a = 1.0 + 1e-8;
    let mut b = 64.0;
    let mut c = b - inv_phi * (b - a);
    let mut d = a + inv_phi * (b - a);
    let mut fc = nll(c)?;
    let mut fd = nll(d)?;

    for _ in 0..200 {
        if (b - a).abs() < 1e-11 {
            break;
        }
        if fc < fd {
            b = d;
            d = c;
            fd = fc;
            c = b - inv_phi * (b - a);
            fc = nll(c)?;
        } else {
            a = c;
            c = d;
            fc = fd;
            d = a + inv_phi * (b - a);
            fd = nll(d)?;
        }
    }
    Ok(f64::midpoint(a, b))
}

/// KS statistic for the discrete fit on a sorted cut slice.
fn ks_discrete(cut: &[f64], alpha: f64, xmin: f64) -> IgraphResult<f64> {
    let n = cut.len() as f64;
    let lnz_xmin = lnhzeta(alpha, xmin)?;
    let mut d_max = 0.0_f64;
    let mut i = 0usize;
    let mut m = 0usize;
    while i < cut.len() {
        let x = cut[i];
        let d = ((lnhzeta(alpha, x)? - lnz_xmin).exp_m1() + (m as f64) / n).abs();
        if d > d_max {
            d_max = d;
        }
        while i < cut.len() && cut[i] == x {
            i += 1;
            m += 1;
        }
    }
    Ok(d_max)
}

/// Discrete log-likelihood on a sorted cut slice.
fn log_likelihood_discrete(cut: &[f64], alpha: f64, xmin: f64) -> IgraphResult<f64> {
    let logsum: f64 = cut.iter().map(|&x| x.ln()).sum();
    let m = cut.len() as f64;
    Ok(-alpha * logsum - m * lnhzeta(alpha, xmin)?)
}

fn fit_discrete_fixed(sorted: &[f64], xmin: f64) -> IgraphResult<PowerLawFitResult> {
    if xmin < 1.0 {
        return Err(IgraphError::InvalidArgument(
            "power_law_fit: xmin must be at least 1".into(),
        ));
    }
    let start = sorted.partition_point(|&x| x < xmin);
    let cut = &sorted[start..];
    let alpha = estimate_alpha_discrete(cut, xmin)?;
    let d = ks_discrete(cut, alpha, xmin)?;
    let total_n = sorted.len();
    let final_alpha = if total_n < 50 {
        finite_size_correction(alpha, cut.len())
    } else {
        alpha
    };
    let l = log_likelihood_discrete(cut, final_alpha, xmin)?;
    Ok(PowerLawFitResult {
        continuous: false,
        alpha: final_alpha,
        xmin,
        log_likelihood: l,
        ks_statistic: d,
    })
}

fn fit_discrete_search(sorted: &[f64]) -> IgraphResult<PowerLawFitResult> {
    let n = sorted.len();
    // Skip values < 1 (invalid as discrete xmin candidates).
    let first = sorted.partition_point(|&x| x < 1.0);
    if first >= n {
        return Err(IgraphError::InvalidArgument(
            "power_law_fit: no data point was at least 1".into(),
        ));
    }

    // Candidate xmin values: the distinct values from `first`, but the top
    // two distinct values are excluded so at least three remain (mirrors
    // the `end_xmin` trimming in plfit_discrete).
    let uniques = unique_starts(&sorted[first..]);
    // Map back to absolute indices.
    let abs_uniques: Vec<usize> = uniques.iter().map(|&u| u + first).collect();
    let num_candidates = abs_uniques.len().saturating_sub(2);

    let mut best: Option<(usize, f64, f64)> = None;
    let mut best_d = f64::MAX;
    for &start in &abs_uniques[..num_candidates] {
        let cut = &sorted[start..];
        let xmin = sorted[start];
        let alpha = estimate_alpha_discrete(cut, xmin)?;
        let d = ks_discrete(cut, alpha, xmin)?;
        if d < best_d {
            best_d = d;
            best = Some((start, alpha, d));
        }
    }

    let (start, alpha, d) = best.ok_or_else(|| {
        IgraphError::InvalidArgument("power_law_fit: could not fit discrete power-law".into())
    })?;
    let xmin = sorted[start];
    let cut = &sorted[start..];
    let final_alpha = if n < 50 {
        finite_size_correction(alpha, cut.len())
    } else {
        alpha
    };
    let l = log_likelihood_discrete(cut, final_alpha, xmin)?;
    Ok(PowerLawFitResult {
        continuous: false,
        alpha: final_alpha,
        xmin,
        log_likelihood: l,
        ks_statistic: d,
    })
}

// ---------------------------------------------------------------------------
// Public entry point.
// ---------------------------------------------------------------------------

/// Fit a power-law distribution to a sample of numbers.
///
/// Counterpart of `igraph_power_law_fit()`. `P(X = x)` is assumed
/// proportional to `x^(-alpha)` for `x >= xmin`.
///
/// - If `xmin >= 0`, that cutoff is fixed and only `alpha` is fitted.
/// - If `xmin < 0`, the optimal cutoff is found by minimising the
///   Kolmogorov–Smirnov statistic.
/// - If `force_continuous` is `false` and every sample is an integer, a
///   discrete model is fitted; otherwise a continuous one. Setting
///   `force_continuous = true` always fits the continuous model.
///
/// The p-value is not computed (matching `igraph`'s default).
///
/// # Errors
///
/// - `InvalidArgument` if `data` is empty, if a fixed `xmin` is out of
///   range (`<= 0` continuous, `< 1` discrete), or if no sample reaches
///   `xmin`.
///
/// # Examples
///
/// ```
/// use rust_igraph::power_law_fit;
///
/// // A continuous power-law tail; fit alpha with a fixed cutoff.
/// let data = vec![1.0, 2.0, 2.0, 3.0, 5.0, 8.0, 13.0, 21.0, 34.0, 55.0];
/// let fit = power_law_fit(&data, 1.0, true).unwrap();
/// assert!(fit.continuous);
/// assert!(fit.alpha > 1.0);
/// assert_eq!(fit.xmin, 1.0);
/// ```
pub fn power_law_fit(
    data: &[f64],
    xmin: f64,
    force_continuous: bool,
) -> IgraphResult<PowerLawFitResult> {
    if data.is_empty() {
        return Err(IgraphError::InvalidArgument(
            "power_law_fit: no data points".into(),
        ));
    }

    // Decide discrete vs continuous exactly as upstream.
    let discrete = if force_continuous {
        false
    } else {
        data.iter().all(|&x| x.trunc() == x)
    };

    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    if discrete {
        if xmin >= 0.0 {
            fit_discrete_fixed(&sorted, xmin)
        } else {
            fit_discrete_search(&sorted)
        }
    } else if xmin >= 0.0 {
        fit_continuous_fixed(&sorted, xmin)
    } else {
        fit_continuous_search(&sorted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-9;

    #[test]
    fn hzeta_matches_riemann_zeta() {
        // ζ(s, 1) is the Riemann zeta. Known closed forms:
        //   ζ(2) = π²/6, ζ(4) = π⁴/90.
        let z2 = hzeta(2.0, 1.0).expect("zeta(2,1)");
        let z4 = hzeta(4.0, 1.0).expect("zeta(4,1)");
        assert!((z2 - std::f64::consts::PI.powi(2) / 6.0).abs() < 1e-12);
        assert!((z4 - std::f64::consts::PI.powi(4) / 90.0).abs() < 1e-12);
    }

    #[test]
    fn hzeta_hurwitz_shift_identity() {
        // ζ(s, 2) = ζ(s, 1) - 1, since the q=2 series drops the k=0 term.
        let s = 3.5;
        let a = hzeta(s, 1.0).expect("zeta(s,1)");
        let b = hzeta(s, 2.0).expect("zeta(s,2)");
        assert!((a - b - 1.0).abs() < 1e-12);
    }

    #[test]
    fn hzeta_rejects_bad_args() {
        assert!(hzeta(1.0, 1.0).is_err());
        assert!(hzeta(2.0, 0.0).is_err());
    }

    #[test]
    fn continuous_closed_form_alpha() {
        // With a fixed xmin and n >= 50 (no finite-size correction), alpha is
        // the exact closed-form MLE 1 + n / Σ log(x_i / xmin).
        let data: Vec<f64> = (0..60).map(|i| 1.0 + f64::from(i)).collect();
        let xmin = 1.0;
        let cut: Vec<f64> = data.iter().copied().filter(|&x| x >= xmin).collect();
        let logsum: f64 = cut.iter().map(|&x| (x / xmin).ln()).sum();
        let expected = 1.0 + (cut.len() as f64) / logsum;
        let fit = power_law_fit(&data, xmin, true).expect("fit");
        assert!(fit.continuous);
        assert!((fit.alpha - expected).abs() < TOL);
        assert_eq!(fit.xmin, 1.0);
    }

    #[test]
    fn finite_size_correction_applies_below_50() {
        // n < 50 triggers the (n-1)/n alpha rescale on the cut count.
        let data: Vec<f64> = (0..10).map(|i| 1.0 + f64::from(i)).collect();
        let xmin = 1.0;
        let cut: Vec<f64> = data.iter().copied().filter(|&x| x >= xmin).collect();
        let logsum: f64 = cut.iter().map(|&x| (x / xmin).ln()).sum();
        let raw = 1.0 + (cut.len() as f64) / logsum;
        let m = cut.len() as f64;
        let corrected = raw * (m - 1.0) / m + 1.0 / m;
        let fit = power_law_fit(&data, xmin, true).expect("fit");
        assert!((fit.alpha - corrected).abs() < TOL);
    }

    #[test]
    fn discrete_detected_for_integer_data() {
        let data = vec![1.0, 1.0, 2.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let fit = power_law_fit(&data, 1.0, false).expect("fit");
        assert!(!fit.continuous);
        assert!(fit.alpha > 1.0);
    }

    #[test]
    fn force_continuous_overrides_integer_detection() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let fit = power_law_fit(&data, 1.0, true).expect("fit");
        assert!(fit.continuous);
    }

    #[test]
    fn non_integer_forces_continuous() {
        let data = vec![1.5, 2.0, 3.0, 4.0, 5.5];
        let fit = power_law_fit(&data, 1.0, false).expect("fit");
        assert!(fit.continuous);
    }

    #[test]
    fn empty_data_errors() {
        assert!(power_law_fit(&[], -1.0, false).is_err());
    }

    #[test]
    fn continuous_xmin_must_be_positive() {
        let data = vec![1.5, 2.0, 3.0];
        assert!(power_law_fit(&data, 0.0, true).is_err());
    }

    #[test]
    fn discrete_xmin_must_be_at_least_one() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        assert!(power_law_fit(&data, 0.5, false).is_err());
    }

    #[test]
    fn ks_statistic_in_unit_interval() {
        let data: Vec<f64> = (1..=100).map(f64::from).collect();
        let fit = power_law_fit(&data, -1.0, true).expect("fit");
        assert!(fit.ks_statistic >= 0.0 && fit.ks_statistic <= 1.0);
    }
}

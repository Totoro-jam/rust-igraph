//! Tiny dependency-free PRNG shared by every rust-igraph AWU that
//! needs deterministic randomness.
//!
//! We deliberately hand-roll [`SplitMix64`] (Steele, Lea, Flood 2014)
//! instead of pulling in `rand`. The constraint comes from the project
//! charter: no new dependencies unless an ADR approves them, and the
//! algorithms here only need a reproducible uniform-bit stream — not the
//! full `rand` zoo.
//!
//! `SplitMix64` is a 64-bit Weyl-step + xorshift-multiply mixer with a
//! state period of `2^64`. It is *not* cryptographic but it is more
//! than adequate for stochastic graph generators (and is in fact the
//! seeding PRNG used by both `xoshiro256**` and Java's `SplittableRandom`).
//!
//! In addition to raw uniform bits the type exposes the three
//! distributions every generator AWU asks for:
//!
//! * [`gen_index`](SplitMix64::gen_index) — uniform `usize ∈ [0, bound)`
//! * [`gen_unit`](SplitMix64::gen_unit) — uniform `f64 ∈ [0, 1)`
//! * [`gen_geom`](SplitMix64::gen_geom) — geometric `≥ 0` with success
//!   probability `p`, matching `RNG_GEOM` from upstream igraph.

/// Minimal `SplitMix64` PRNG.
///
/// State is a single `u64` Weyl counter. Each call advances the
/// counter by the fixed Weyl increment `0x9E37_79B9_7F4A_7C15`
/// (`floor(2^64 / φ)`, where `φ` is the golden ratio) and then runs
/// the avalanche mix.
#[derive(Debug, Clone)]
pub struct SplitMix64(u64);

impl SplitMix64 {
    /// Constructs a new PRNG with the given 64-bit seed.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// Returns the next uniform `u64`.
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform integer in `[0, bound)`. Panics if `bound == 0`.
    ///
    /// Uses straight modulo reduction; the bias is `< 2^-32` for any
    /// `bound ≤ 2^32`, which covers every realistic graph index.
    pub fn gen_index(&mut self, bound: usize) -> usize {
        assert!(bound > 0, "gen_index bound must be positive");
        let bound_u64 = u64::try_from(bound).expect("bound fits in u64 on supported targets");
        let r = self.next_u64() % bound_u64;
        usize::try_from(r).expect("modulus result fits in usize by construction")
    }

    /// Uniform `f64` in `[0, 1)`, using the top 53 mantissa bits.
    pub fn gen_unit(&mut self) -> f64 {
        let bits = self.next_u64() >> 11; // [0, 2^53)
        #[allow(clippy::cast_precision_loss)]
        let numerator = bits as f64;
        numerator * (1.0_f64 / 9_007_199_254_740_992.0_f64)
    }

    /// Geometric distribution: number of failures before the first
    /// success when each trial succeeds with probability `p`.
    ///
    /// Returns `f64` to match upstream's `RNG_GEOM`, which uses a real
    /// value so that callers walking pair-index space can detect the
    /// sentinel "gap ≥ remaining" condition without overflow even when
    /// `p` is tiny.
    ///
    /// * `p ∈ (0, 1]` — required. Caller validates.
    /// * Inverse-CDF form: `floor(log(1-U) / log(1-p))`.
    pub fn gen_geom(&mut self, p: f64) -> f64 {
        debug_assert!(p > 0.0 && p <= 1.0, "gen_geom requires p in (0, 1]");
        if p >= 1.0 {
            return 0.0;
        }
        let u = self.gen_unit();
        // 1 - U is in (0, 1]. log(1) = 0, which only happens when
        // U == 0 — then we return 0, consistent with "first trial
        // succeeded".
        let one_minus_u = 1.0 - u;
        #[allow(clippy::float_cmp)]
        let is_one = one_minus_u == 1.0;
        if is_one {
            return 0.0;
        }
        let log_one_minus_p = (1.0 - p).ln();
        #[allow(clippy::float_cmp)]
        if log_one_minus_p == 0.0 {
            // p extremely close to 0; the geometric mean is huge, but
            // we can't divide by zero. Return f64::INFINITY so callers
            // walking [0, maxedges) terminate immediately.
            return f64::INFINITY;
        }
        (one_minus_u.ln() / log_one_minus_p).floor()
    }

    /// Standard normal (Gaussian) variate via Box-Muller transform.
    ///
    /// Returns a draw from N(0, 1). Uses the basic form: generate two
    /// independent U(0,1) draws, produce two independent N(0,1) draws,
    /// cache the second for the next call.
    pub fn gen_normal(&mut self) -> f64 {
        let u1 = self.gen_unit();
        let u2 = self.gen_unit();
        let r = (-2.0 * (1.0 - u1).ln()).sqrt();
        r * (std::f64::consts::TAU * u2).cos()
    }

    /// Gamma(shape, 1) variate via Marsaglia-Tsang's method (2000).
    ///
    /// For `shape >= 1` uses the direct method. For `0 < shape < 1`
    /// uses `Gamma(shape+1, 1) * U^(1/shape)` where U ~ U(0,1).
    ///
    /// Caller must ensure `shape > 0`.
    #[allow(clippy::many_single_char_names)]
    pub fn gen_gamma(&mut self, shape: f64) -> f64 {
        debug_assert!(shape > 0.0, "gen_gamma requires shape > 0");

        if shape < 1.0 {
            let g = self.gen_gamma(shape + 1.0);
            let u = self.gen_unit();
            return g * u.powf(1.0 / shape);
        }

        let d = shape - 1.0 / 3.0;
        let c = 1.0 / (9.0 * d).sqrt();

        loop {
            let x = self.gen_normal();
            let v_base = 1.0 + c * x;
            if v_base <= 0.0 {
                continue;
            }
            let v = v_base * v_base * v_base;
            let u = self.gen_unit();
            let x2 = x * x;

            if u < 1.0 - 0.0331 * x2 * x2 {
                return d * v;
            }
            if u.ln() < 0.5 * x2 + d * (1.0 - v + v.ln()) {
                return d * v;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SplitMix64;

    #[test]
    fn determinism_same_seed_same_stream() {
        let mut a = SplitMix64::new(42);
        let mut b = SplitMix64::new(42);
        for _ in 0..1024 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_differ() {
        let mut a = SplitMix64::new(1);
        let mut b = SplitMix64::new(2);
        // First word should differ.
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn gen_index_in_range() {
        let mut r = SplitMix64::new(0x1234);
        for _ in 0..10_000 {
            let v = r.gen_index(100);
            assert!(v < 100);
        }
    }

    #[test]
    fn gen_unit_in_unit_interval() {
        let mut r = SplitMix64::new(99);
        for _ in 0..10_000 {
            let u = r.gen_unit();
            assert!((0.0..1.0).contains(&u), "u={u}");
        }
    }

    #[test]
    fn gen_geom_p1_returns_zero() {
        let mut r = SplitMix64::new(7);
        for _ in 0..32 {
            #[allow(clippy::float_cmp)]
            let v = r.gen_geom(1.0);
            #[allow(clippy::float_cmp)]
            {
                assert_eq!(v, 0.0);
            }
        }
    }

    #[test]
    fn gen_normal_mean_and_variance() {
        let n: i32 = 100_000;
        let mut r = SplitMix64::new(42);
        let samples: Vec<f64> = (0..n).map(|_| r.gen_normal()).collect();
        let nf = f64::from(n);
        let mean = samples.iter().sum::<f64>() / nf;
        let variance = samples
            .iter()
            .map(|&x| (x - mean) * (x - mean))
            .sum::<f64>()
            / nf;
        assert!(mean.abs() < 0.02, "normal mean should be ~0, got {mean}");
        assert!(
            (variance - 1.0).abs() < 0.05,
            "normal variance should be ~1, got {variance}"
        );
    }

    #[test]
    fn gen_gamma_mean_matches_shape() {
        // E[Gamma(shape, 1)] = shape
        for &shape in &[0.5, 1.0, 2.0, 5.0] {
            let n: i32 = 50_000;
            let mut r = SplitMix64::new(99);
            let sum: f64 = (0..n).map(|_| r.gen_gamma(shape)).sum();
            let mean = sum / f64::from(n);
            let rel_err = (mean - shape).abs() / shape;
            assert!(
                rel_err < 0.05,
                "gamma({shape}) mean={mean}, expected ~{shape}, rel_err={rel_err}"
            );
        }
    }

    #[test]
    fn gen_gamma_all_positive() {
        let mut r = SplitMix64::new(777);
        for _ in 0..10_000 {
            let v = r.gen_gamma(0.3);
            assert!(v > 0.0, "gamma sample must be positive, got {v}");
        }
    }

    #[test]
    fn gen_geom_mean_matches_distribution() {
        // E[Geom(p)] = (1-p)/p when counting failures before first success.
        // We use a generous tolerance because the sample mean for moderate
        // n only converges slowly.
        let p = 0.1;
        let expected_mean = (1.0 - p) / p; // 9.0
        let n = 100_000;
        let mut r = SplitMix64::new(123_456);
        let sum: f64 = (0..n).map(|_| r.gen_geom(p)).sum();
        let mean = sum / f64::from(n);
        // 3% tolerance — empirically the sample mean for n=1e5 lands
        // well within this band.
        assert!(
            (mean - expected_mean).abs() / expected_mean < 0.03,
            "mean={mean}, expected≈{expected_mean}"
        );
    }
}

//! Random graph generators (`ALGO-GN-*`).
//!
//! Each generator takes a `seed: u64` and runs against the shared
//! [`crate::core::rng::SplitMix64`] PRNG, so callers get fully
//! reproducible graphs without pulling in a `rand` dependency.

pub mod barabasi;
pub mod erdos_renyi;
pub mod growing_random;

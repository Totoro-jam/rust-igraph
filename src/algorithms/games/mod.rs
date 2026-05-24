//! Random graph generators (`ALGO-GN-*`).
//!
//! Each generator takes a `seed: u64` and runs against the shared
//! [`crate::core::rng::SplitMix64`] PRNG, so callers get fully
//! reproducible graphs without pulling in a `rand` dependency.

pub mod barabasi;
pub mod barabasi_aging;
pub mod barabasi_psumtree;
pub mod callaway_traits;
pub mod chung_lu;
pub mod cited_type;
pub mod erdos_renyi;
pub mod establishment;
pub mod forestfire;
pub mod grg;
pub mod growing_random;
pub mod hsbm;
pub mod islands;
pub mod k_regular;
pub mod lastcit;
pub mod preference;
pub mod recent_degree;
pub mod sbm;
pub mod static_fitness;
pub mod tree;
pub mod watts;

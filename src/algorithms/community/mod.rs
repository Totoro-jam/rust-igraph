//! Community-detection algorithms (ALGO-CO-*). Phase 1: `modularity`
//! (Newman-Girvan modularity of a partition). Phase 4: `louvain`
//! multilevel community detection, `leiden` (Traag-Waltman-van Eck 2019),
//! `label_propagation` (Raghavan-Albert-Kumara 2007 +
//! Traag-Šubelj 2023 fast variant), `fluid_communities`
//! (Parés et al. 2017), `edge_betweenness_community` (Girvan-Newman 2002),
//! `fast_greedy_modularity` (Clauset-Newman-Moore 2004),
//! `walktrap` (Pons-Latapy 2005). Helpers: `community_to_membership`
//! (cut a dendrogram at `k` merges), `reindex_membership` (densify a
//! membership vector to `0..k-1` by first occurrence).

// `pub(crate)` so the inner module name doesn't double-list with the
// function re-export in rustdoc.
pub(crate) mod community_to_membership;
pub(crate) mod compare_communities;
pub(crate) mod edge_betweenness_community;
pub(crate) mod edge_betweenness_community_weighted;
pub(crate) mod fast_greedy_modularity;
pub(crate) mod fluid_communities;
pub(crate) mod label_propagation;
pub(crate) mod leiden;
pub(crate) mod louvain;
pub(crate) mod modularity;
pub(crate) mod reindex_membership;
pub(crate) mod walktrap;

pub use community_to_membership::{CommunityToMembershipResult, community_to_membership};
pub use compare_communities::{CommunityComparison, compare_communities};
pub use edge_betweenness_community::{EdgeBetweennessResult, edge_betweenness_community};
pub use edge_betweenness_community_weighted::edge_betweenness_community_weighted;
pub use fast_greedy_modularity::{
    FastGreedyResult, fast_greedy_modularity, fast_greedy_modularity_weighted,
};
pub use fluid_communities::{
    FLUID_DEFAULT_MAX_ITERATIONS, FluidOptions, FluidResult, fluid_communities,
    fluid_communities_with_options,
};
pub use label_propagation::{
    LpaOptions, LpaResult, LpaVariant, label_propagation, label_propagation_weighted,
    label_propagation_with_options,
};
pub use leiden::{
    LEIDEN_DEFAULT_BETA, LEIDEN_DEFAULT_ITERATIONS, LeidenObjective, LeidenOptions, LeidenResult,
    leiden, leiden_weighted, leiden_with_options,
};
pub use louvain::{LouvainResult, louvain, louvain_weighted, louvain_with_options};
pub use modularity::{modularity, modularity_directed, modularity_weighted};
pub use reindex_membership::{ReindexMembershipResult, reindex_membership};
pub use walktrap::{
    WALKTRAP_DEFAULT_STEPS, WalktrapOptions, WalktrapResult, walktrap, walktrap_weighted,
    walktrap_with_options,
};

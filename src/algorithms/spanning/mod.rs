//! Spanning-tree algorithms (ALGO-MST-*, ALGO-RST-*).
//!
//! Mirrors the upstream `igraph` C file
//! [`references/igraph/src/misc/spanning_trees.c`](https://github.com/igraph/igraph/blob/main/src/misc/spanning_trees.c)
//! which groups the deterministic minimum-spanning-tree variants
//! (BFS-unweighted, Prim, Kruskal, automatic dispatch) and the
//! loop-erased random-walk (LERW) random spanning tree.
//!
//! Currently hosts:
//! - `mst` (`ALGO-MST-001`): [`minimum_spanning_tree`] — Prim / Kruskal /
//!   Unweighted / Automatic with a `Vec<EdgeId>` return type.
//! - `random_spanning_tree` (`ALGO-RST-001`): [`random_spanning_tree`] —
//!   uniform spanning tree/forest via loop-erased random walk.

pub(crate) mod mst;
pub(crate) mod random_spanning_tree;

pub use mst::{MstAlgorithm, minimum_spanning_tree};
pub use random_spanning_tree::random_spanning_tree;

//! Graph traversal. Phase 0 ships BFS; ALGO-TR-002 adds DFS;
//! ALGO-TR-001 adds the multi-output BFS variant `bfs_tree`.

// `pub(crate)` so the inner module names don't double-list with the
// function re-exports in rustdoc.
pub(crate) mod bfs;
pub(crate) mod dfs;

pub use bfs::{BfsMode, BfsSimple, BfsTree, bfs, bfs_simple, bfs_tree};
pub use dfs::{DfsTree, dfs, dfs_tree};

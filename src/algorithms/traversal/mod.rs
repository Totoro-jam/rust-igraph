//! Graph traversal. Phase 0 ships BFS; ALGO-TR-002 adds DFS;
//! ALGO-TR-001 adds the multi-output BFS variant `bfs_tree`.

pub mod bfs;
pub mod dfs;

pub use bfs::{BfsTree, bfs, bfs_tree};
pub use dfs::dfs;

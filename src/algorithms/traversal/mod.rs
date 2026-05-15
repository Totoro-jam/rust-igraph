//! Graph traversal. Phase 0 ships BFS; ALGO-TR-002 adds DFS.

pub mod bfs;
pub mod dfs;

pub use bfs::bfs;
pub use dfs::dfs;

//! Error type for rust-igraph.
//!
//! Mirrors the most-used variants of `igraph_error_t` from the C core, with room to grow.
//! Numeric error codes from `include/igraph_error.h` are kept as a separate enum so
//! existing igraph-C tests can be reproduced precisely.

use thiserror::Error;

/// All errors returned from rust-igraph.
///
/// Variants are added as algorithms land; the initial set covers the cases the
/// walking-skeleton (Phase 0) needs.
#[derive(Debug, Error)]
pub enum IgraphError {
    /// Argument was outside its accepted range or otherwise invalid.
    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    /// A vertex id referred to a vertex that does not exist.
    #[error("vertex {id} out of range (graph has {n} vertices)")]
    VertexOutOfRange { id: u32, n: u32 },

    /// An edge id referred to an edge that does not exist.
    #[error("edge {id} out of range (graph has {m} edges)")]
    EdgeOutOfRange { id: u32, m: u32 },

    /// An I/O failure while reading or writing graph data.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Parsing a graph file failed.
    #[error("parse error at line {line}: {message}")]
    Parse { line: usize, message: String },

    /// Operation is not supported (e.g. unweighted-only path requested on weighted graph).
    #[error("unsupported: {0}")]
    Unsupported(&'static str),

    /// Numeric algorithm failed to converge within iteration budget.
    #[error("did not converge after {iters} iterations (last residual {residual})")]
    DidNotConverge { iters: usize, residual: f64 },

    /// Catch-all for unexpected internal failures (bugs).
    #[error("internal error: {0}")]
    Internal(&'static str),
}

/// Convenience alias for `Result<T, IgraphError>`.
pub type IgraphResult<T> = std::result::Result<T, IgraphError>;

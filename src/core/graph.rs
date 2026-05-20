//! `Graph` — pure-Rust port of `igraph_t`.
//!
//! Storage is the **indexed edge list** that upstream igraph uses (see
//! `references/igraph/include/igraph_datatype.h:105-116`):
//!
//! - `from[e]`, `to[e]` — canonical edge list. Edge `e` runs from
//!   `from[e]` to `to[e]`; `|from| == |to| == ecount`.
//! - `oi[i]` — edge ids ordered by `from` (and then `to`).
//! - `ii[i]` — edge ids ordered by `to` (and then `from`).
//! - `os[v]..os[v+1]` — slice of `oi` covering vertex `v`'s out-edges.
//! - `is[v]..is[v+1]` — slice of `ii` covering vertex `v`'s in-edges.
//!
//! For undirected graphs the edge list is canonicalised so `from[e] <= to[e]`
//! (matching upstream igraph's invariant in `type_indexededgelist.c:282-288`).
//! The doubled in/out indexing makes `neighbors()` symmetric for undirected
//! graphs without storing each edge twice.
//!
//! ALGO-CORE-001a (Phase 1, this file): struct + `new`/`with_vertices` +
//! `add_vertices`/`add_edge`/`add_edges` + `vcount`/`ecount`/`is_directed` +
//! `neighbors`/`degree` + `Clone`.
//!
//! Follow-up AWUs:
//! - 001b: `incident`, edge-id helpers.
//! - 001c: `delete_vertices`/`delete_edges`.
//! - 001d: `edge`/`edges`/`get_eid`/`get_eids`/`get_all_eids_between`.
//! - 001e: property cache, `is_same_graph`.
//!
//! Attribute system → ALGO-AT-* (out of scope here).

use super::error::{IgraphError, IgraphResult};

/// Vertex id. The Phase-0 ADR-0007 fixes this to `u32`; `Option<VertexId>`
/// is the idiomatic "no vertex" sentinel (igraph C uses `-1`).
pub type VertexId = u32;

/// Edge id. Same width as [`VertexId`]; an edge id is its position in
/// `from`/`to`.
pub type EdgeId = u32;

/// Counterpart of `igraph_t` (see `references/igraph/include/igraph_datatype.h`).
///
/// Phase-0 callers (`bfs`, `read_edgelist`, oracle tests) only depended on
/// `with_vertices`, `add_edge`, `add_edges`, `vcount`, `ecount`, `neighbors`,
/// `degree` — those signatures are preserved here, so existing call sites
/// compile unchanged. New for Phase 1: `new` (with `directed` flag),
/// `is_directed`.
#[derive(Debug, Clone, Default)]
pub struct Graph {
    /// Vertex count. Redundant with the highest used id; mirrors `igraph_t::n`.
    n: u32,
    /// Whether the graph is directed.
    directed: bool,
    /// Source endpoints, one per edge.
    from: Vec<VertexId>,
    /// Target endpoints, one per edge.
    to: Vec<VertexId>,
    /// Edge ids in `from`-major order.
    oi: Vec<EdgeId>,
    /// Edge ids in `to`-major order.
    ii: Vec<EdgeId>,
    /// `os[v]..os[v+1]` is the slice of `oi` for vertex `v`'s out-edges.
    /// Length is `n + 1`; `os[0] == 0`, `os[n] == ecount`.
    os: Vec<u32>,
    /// `is[v]..is[v+1]` for incoming. Same shape as `os`.
    is: Vec<u32>,
}

impl Graph {
    /// Construct an empty graph on `n` vertices.
    ///
    /// Counterpart of `igraph_empty()`; `directed` defaults to `false` if
    /// you use [`Graph::with_vertices`] instead.
    pub fn new(n: u32, directed: bool) -> IgraphResult<Self> {
        let mut g = Self {
            n: 0,
            directed,
            from: Vec::new(),
            to: Vec::new(),
            oi: Vec::new(),
            ii: Vec::new(),
            os: vec![0],
            is: vec![0],
        };
        g.add_vertices(n)?;
        Ok(g)
    }

    /// Construct an empty *undirected* graph on `n` vertices.
    ///
    /// Equivalent to `Graph::new(n, false).unwrap()` for the success path.
    /// Phase-0 compatibility shim — exact signature preserved.
    pub fn with_vertices(n: u32) -> Self {
        Self::new(n, false).expect("with_vertices: n cannot exceed u32::MAX")
    }

    /// Number of vertices. Counterpart of `igraph_vcount()`.
    #[must_use]
    pub fn vcount(&self) -> u32 {
        self.n
    }

    /// Number of edges. Counterpart of `igraph_ecount()`.
    #[must_use]
    pub fn ecount(&self) -> usize {
        self.from.len()
    }

    /// `true` if the graph is directed. Counterpart of `igraph_is_directed()`.
    #[must_use]
    pub fn is_directed(&self) -> bool {
        self.directed
    }

    /// Append `nv` isolated vertices, returning the inclusive id range
    /// `(first, last)` of the new vertices. If `nv == 0` returns
    /// `(self.n, self.n)` and does nothing.
    ///
    /// Counterpart of `igraph_add_vertices()`.
    pub fn add_vertices(&mut self, nv: u32) -> IgraphResult<(VertexId, VertexId)> {
        let new_n = self
            .n
            .checked_add(nv)
            .ok_or(IgraphError::Internal("vertex count overflow"))?;
        let first = self.n;
        // os/is grow by `nv` entries, all initialised to ecount.
        let ec = u32::try_from(self.ecount())
            .map_err(|_| IgraphError::Internal("edge count exceeds u32::MAX"))?;
        for _ in 0..nv {
            self.os.push(ec);
            self.is.push(ec);
        }
        self.n = new_n;
        Ok((first, new_n.saturating_sub(1)))
    }

    /// Add a single edge from `u` to `v`.
    ///
    /// Self-loops and parallel edges are allowed. For undirected graphs the
    /// edge is canonicalised so the stored `from <= to`.
    pub fn add_edge(&mut self, u: VertexId, v: VertexId) -> IgraphResult<()> {
        self.add_edges(std::iter::once((u, v)))
    }

    /// Add a sequence of edges. After all edges are appended, the indexes
    /// (`oi` / `ii` / `os` / `is`) are rebuilt in one pass — counterpart of
    /// `igraph_add_edges` (`type_indexededgelist.c:254-367`).
    pub fn add_edges<I>(&mut self, edges: I) -> IgraphResult<()>
    where
        I: IntoIterator<Item = (VertexId, VertexId)>,
    {
        for (u, v) in edges {
            self.check_vertex(u)?;
            self.check_vertex(v)?;
            // Undirected canonicalisation: store the smaller endpoint as `from`.
            if !self.directed && u > v {
                self.from.push(v);
                self.to.push(u);
            } else {
                self.from.push(u);
                self.to.push(v);
            }
        }
        self.rebuild_indexes()?;
        Ok(())
    }

    /// Out-edge neighbour iterator for vertex `v`.
    ///
    /// For undirected graphs this returns *all* neighbours (since the
    /// indexing tracks both endpoints symmetrically). Order is the upstream
    /// igraph order — edges are visited in `oi` order, then `ii` order, with
    /// duplicates suppressed when the same edge is incident on both.
    ///
    /// Counterpart of `igraph_neighbors(graph, _, vid, IGRAPH_ALL, ...)`.
    pub fn neighbors(&self, v: VertexId) -> IgraphResult<Vec<VertexId>> {
        self.check_vertex(v)?;
        let v_idx = v as usize;
        if self.directed {
            // Directed: only outgoing neighbours; oi sorted by (from, to)
            // so the out-neighbour list is already sorted ascending.
            let out_range = self.os[v_idx] as usize..self.os[v_idx + 1] as usize;
            let out: Vec<VertexId> = self.oi[out_range]
                .iter()
                .map(|&e| self.to[e as usize])
                .collect();
            Ok(out)
        } else {
            // Undirected: merge the two already-sorted sublists from oi
            // (out-side, ascending in `to`) and ii (in-side, ascending
            // in `from`) into one ascending neighbour list. Matches
            // upstream `igraph_neighbors(_, _, _, IGRAPH_ALL)` and
            // python-igraph's `Graph.neighbors(v)` exactly.
            let out_start = self.os[v_idx] as usize;
            let out_end = self.os[v_idx + 1] as usize;
            let in_start = self.is[v_idx] as usize;
            let in_end = self.is[v_idx + 1] as usize;
            let mut out = Vec::with_capacity((out_end - out_start) + (in_end - in_start));
            let mut out_idx = out_start;
            let mut in_idx = in_start;
            while out_idx < out_end && in_idx < in_end {
                let a = self.to[self.oi[out_idx] as usize];
                let b = self.from[self.ii[in_idx] as usize];
                if a <= b {
                    out.push(a);
                    out_idx += 1;
                } else {
                    out.push(b);
                    in_idx += 1;
                }
            }
            while out_idx < out_end {
                out.push(self.to[self.oi[out_idx] as usize]);
                out_idx += 1;
            }
            while in_idx < in_end {
                out.push(self.from[self.ii[in_idx] as usize]);
                in_idx += 1;
            }
            Ok(out)
        }
    }

    /// Degree of vertex `v` — number of edges incident to it.
    ///
    /// On undirected graphs every edge counts once except a self-loop which
    /// counts twice (matches upstream igraph's `IGRAPH_LOOPS = TWICE` default
    /// at `type_indexededgelist.c:1162`).
    ///
    /// Counterpart of `igraph_degree_1(_, _, _, IGRAPH_ALL, IGRAPH_LOOPS_TWICE)`.
    pub fn degree(&self, v: VertexId) -> IgraphResult<usize> {
        self.check_vertex(v)?;
        let v_idx = v as usize;
        let out = (self.os[v_idx + 1] - self.os[v_idx]) as usize;
        if self.directed {
            let in_count = (self.is[v_idx + 1] - self.is[v_idx]) as usize;
            Ok(out + in_count)
        } else {
            // Undirected: out + in. Self-loops appear in both (they are
            // stored once with `from == to` but indexed in both `os` and
            // `is` slots), so naturally count twice.
            let in_count = (self.is[v_idx + 1] - self.is[v_idx]) as usize;
            Ok(out + in_count)
        }
    }

    // ---------------------------------------------------------------
    // ALGO-CORE-001b: edge-id helpers + incident edges.
    // ---------------------------------------------------------------

    /// Source endpoint of edge `eid`. Counterpart of `IGRAPH_FROM`
    /// (`igraph_interface.h:115`).
    pub fn edge_source(&self, eid: EdgeId) -> IgraphResult<VertexId> {
        self.check_edge(eid)?;
        Ok(self.from[eid as usize])
    }

    /// Target endpoint of edge `eid`. Counterpart of `IGRAPH_TO`
    /// (`igraph_interface.h:128`).
    pub fn edge_target(&self, eid: EdgeId) -> IgraphResult<VertexId> {
        self.check_edge(eid)?;
        Ok(self.to[eid as usize])
    }

    /// Both endpoints of edge `eid`, ordered as `(from, to)`. Counterpart
    /// of `igraph_edge` (`igraph_interface.h:71`).
    pub fn edge(&self, eid: EdgeId) -> IgraphResult<(VertexId, VertexId)> {
        self.check_edge(eid)?;
        let i = eid as usize;
        Ok((self.from[i], self.to[i]))
    }

    /// The other endpoint of `eid` given one endpoint `vid`. Counterpart
    /// of `IGRAPH_OTHER` (`igraph_interface.h:145`). Errors if `vid` is
    /// not actually an endpoint of `eid`.
    pub fn edge_other(&self, eid: EdgeId, vid: VertexId) -> IgraphResult<VertexId> {
        let (u, v) = self.edge(eid)?;
        if vid == u {
            Ok(v)
        } else if vid == v {
            Ok(u)
        } else {
            Err(IgraphError::InvalidArgument(format!(
                "vertex {vid} is not an endpoint of edge {eid} ({u}, {v})"
            )))
        }
    }

    /// Edge ids incident to vertex `v`, in the same iteration order as
    /// [`Graph::neighbors`].
    ///
    /// For undirected graphs returns the union of out-side (`oi`) and
    /// in-side (`ii`) edges — every edge incident to `v` once, except
    /// self-loops which appear twice (matching `igraph_neighbors` /
    /// `igraph_degree`'s `IGRAPH_LOOPS_TWICE` default at
    /// `type_indexededgelist.c:1162`).
    ///
    /// For directed graphs returns out-edges only, mirroring this AWU's
    /// `neighbors()` choice. (The full mode-aware variant lands later
    /// alongside `igraph_neighbors(mode = IN/OUT/ALL)`.)
    ///
    /// Counterpart of `igraph_incident(_, _, v, IGRAPH_ALL, IGRAPH_LOOPS_TWICE)`
    /// for undirected; `IGRAPH_OUT` mode for directed.
    pub fn incident(&self, v: VertexId) -> IgraphResult<Vec<EdgeId>> {
        self.check_vertex(v)?;
        let v_idx = v as usize;
        let out_range = self.os[v_idx] as usize..self.os[v_idx + 1] as usize;
        if self.directed {
            Ok(self.oi[out_range].to_vec())
        } else {
            let in_range = self.is[v_idx] as usize..self.is[v_idx + 1] as usize;
            let mut out = Vec::with_capacity(out_range.len() + in_range.len());
            out.extend_from_slice(&self.oi[out_range]);
            out.extend_from_slice(&self.ii[in_range]);
            Ok(out)
        }
    }

    /// Companion to [`incident`](Self::incident): returns *only* the
    /// edges incoming to `v` for directed graphs. For undirected
    /// graphs the result is identical to `incident` (every edge is
    /// bidirectional).
    ///
    /// Counterpart of `igraph_incident(_, _, v, IGRAPH_IN, IGRAPH_LOOPS_TWICE)`.
    pub(crate) fn incident_in(&self, v: VertexId) -> IgraphResult<Vec<EdgeId>> {
        self.check_vertex(v)?;
        let v_idx = v as usize;
        if self.directed {
            let in_range = self.is[v_idx] as usize..self.is[v_idx + 1] as usize;
            Ok(self.ii[in_range].to_vec())
        } else {
            self.incident(v)
        }
    }

    /// Edge id between `from` and `to`, if any.
    ///
    /// On undirected graphs `(u, v)` and `(v, u)` are equivalent.
    /// On directed graphs the search follows the edge direction
    /// `from -> to`. Returns [`crate::IgraphError::InvalidArgument`]
    /// when no such edge exists; for the "no error, return None" variant
    /// use [`Self::find_eid`].
    ///
    /// Counterpart of
    /// `igraph_get_eid(_, _, from, to, /*directed=*/true, /*error=*/true)`
    /// from `references/igraph/src/graph/type_indexededgelist.c:1522-1555`.
    /// Phase-1 minimal slice: linear scan across the from-bucket; the
    /// upstream binary-search optimisation lands in a perf pass.
    pub fn get_eid(&self, from: VertexId, to: VertexId) -> IgraphResult<EdgeId> {
        self.find_eid(from, to)?
            .ok_or_else(|| IgraphError::InvalidArgument(format!("no edge between {from} and {to}")))
    }

    /// Edge id between `from` and `to`, or `None` if not connected.
    ///
    /// Same semantics as [`Self::get_eid`] but no-error variant
    /// matching upstream's `error=false` mode. When parallel edges
    /// exist, returns the lowest edge id (matching upstream's
    /// "always returns the same edge ID" guarantee).
    pub fn find_eid(&self, from: VertexId, to: VertexId) -> IgraphResult<Option<EdgeId>> {
        self.check_vertex(from)?;
        self.check_vertex(to)?;
        if self.directed {
            // Search out-bucket of `from` for `to[e] == to`.
            let range = self.os[from as usize] as usize..self.os[from as usize + 1] as usize;
            for &e in &self.oi[range] {
                if self.to[e as usize] == to {
                    return Ok(Some(e));
                }
            }
            Ok(None)
        } else {
            // Undirected: edges canonicalised so `from[e] <= to[e]`.
            // Search the bucket of the smaller endpoint for the larger.
            let (lo, hi) = if from <= to { (from, to) } else { (to, from) };
            let range = self.os[lo as usize] as usize..self.os[lo as usize + 1] as usize;
            for &e in &self.oi[range] {
                if self.to[e as usize] == hi {
                    return Ok(Some(e));
                }
            }
            Ok(None)
        }
    }

    /// All edge ids between `from` and `to`, including parallel edges
    /// and (for self-loops) the loop edge once.
    ///
    /// Counterpart of
    /// `igraph_get_all_eids_between()` from
    /// `references/igraph/src/graph/type_indexededgelist.c:~1700`.
    /// On undirected graphs `(u, v)` and `(v, u)` are equivalent. The
    /// returned vector is sorted ascending by edge id.
    pub fn get_all_eids_between(&self, from: VertexId, to: VertexId) -> IgraphResult<Vec<EdgeId>> {
        self.check_vertex(from)?;
        self.check_vertex(to)?;
        let mut out = Vec::new();
        if self.directed {
            let range = self.os[from as usize] as usize..self.os[from as usize + 1] as usize;
            for &e in &self.oi[range] {
                if self.to[e as usize] == to {
                    out.push(e);
                }
            }
        } else {
            let (lo, hi) = if from <= to { (from, to) } else { (to, from) };
            let range = self.os[lo as usize] as usize..self.os[lo as usize + 1] as usize;
            for &e in &self.oi[range] {
                if self.to[e as usize] == hi {
                    out.push(e);
                }
            }
        }
        out.sort_unstable();
        Ok(out)
    }

    /// Out-neighbours of `v` (always — directed or undirected). Each
    /// edge contributes one entry, in `oi[os[v]..os[v+1]]` order
    /// (lex by `(from, to)`). Self-loops appear once.
    ///
    /// Internal helper used by direction-aware algorithms (e.g.
    /// strongly connected components). The full mode-aware public
    /// surface ships with the next `igraph_neighbors` AWU.
    pub(crate) fn out_neighbors_vec(&self, v: VertexId) -> IgraphResult<Vec<VertexId>> {
        self.check_vertex(v)?;
        let v_idx = v as usize;
        let range = self.os[v_idx] as usize..self.os[v_idx + 1] as usize;
        Ok(self.oi[range]
            .iter()
            .map(|&e| self.to[e as usize])
            .collect())
    }

    /// In-neighbours of `v` (always — directed or undirected). Each
    /// edge contributes one entry, in `ii[is[v]..is[v+1]]` order
    /// (lex by `(to, from)`). Self-loops appear once.
    ///
    /// Companion to [`out_neighbors_vec`](Self::out_neighbors_vec); see
    /// its doc for context on visibility.
    pub(crate) fn in_neighbors_vec(&self, v: VertexId) -> IgraphResult<Vec<VertexId>> {
        self.check_vertex(v)?;
        let v_idx = v as usize;
        let range = self.is[v_idx] as usize..self.is[v_idx + 1] as usize;
        Ok(self.ii[range]
            .iter()
            .map(|&e| self.from[e as usize])
            .collect())
    }

    // ---------------------------------------------------------------
    // ALGO-CORE-001c: delete_edges + delete_vertices + delete_vertices_map.
    // ---------------------------------------------------------------

    /// Remove the given edges from the graph.
    ///
    /// `edges` may contain the same id more than once — the second and
    /// later occurrences are no-ops. Remaining edges keep their
    /// pairwise relative order but are renumbered so edge ids stay
    /// contiguous starting at 0. Returns
    /// [`IgraphError::EdgeOutOfRange`] if any id is `>= ecount()`; on
    /// error the graph is left unchanged.
    ///
    /// Counterpart of `igraph_delete_edges`
    /// (`references/igraph/src/graph/type_indexededgelist.c:500`).
    pub fn delete_edges(&mut self, edges: &[EdgeId]) -> IgraphResult<()> {
        let m = self.ecount();
        let m_u32 = u32::try_from(m).unwrap_or(u32::MAX);

        // Validate up front so a bad id leaves graph state untouched.
        for &eid in edges {
            if (eid as usize) >= m {
                return Err(IgraphError::EdgeOutOfRange { id: eid, m: m_u32 });
            }
        }
        if edges.is_empty() {
            return Ok(());
        }

        let mut remove = vec![false; m];
        for &eid in edges {
            remove[eid as usize] = true;
        }

        let mut new_from: Vec<VertexId> = Vec::with_capacity(m);
        let mut new_to: Vec<VertexId> = Vec::with_capacity(m);
        for (e, &is_removed) in remove.iter().enumerate() {
            if !is_removed {
                new_from.push(self.from[e]);
                new_to.push(self.to[e]);
            }
        }
        self.from = new_from;
        self.to = new_to;
        self.rebuild_indexes()
    }

    /// Remove the given vertices and all their incident edges.
    ///
    /// `vertices` may repeat ids freely. Surviving vertices get
    /// renumbered so the new id space is `0..new_vcount` in their
    /// previous relative order. Returns
    /// [`IgraphError::VertexOutOfRange`] if any id is `>= vcount()`;
    /// on error the graph is left unchanged.
    ///
    /// Counterpart of `igraph_delete_vertices`
    /// (`references/igraph/src/graph/type_indexededgelist.c:540`).
    pub fn delete_vertices(&mut self, vertices: &[VertexId]) -> IgraphResult<()> {
        self.delete_vertices_map(vertices).map(|_| ())
    }

    /// Like [`delete_vertices`](Self::delete_vertices), but also returns
    /// the old↔new vertex id mappings.
    ///
    /// Returns `(map, invmap)` where:
    /// - `map[old_id] == Some(new_id)` if the vertex was retained, else
    ///   `None`. Length is the *original* vertex count.
    /// - `invmap[new_id] == old_id`. Length is the *new* vertex count.
    ///
    /// Counterpart of `igraph_delete_vertices_map`
    /// (`references/igraph/src/graph/type_indexededgelist.c:645`).
    pub fn delete_vertices_map(
        &mut self,
        vertices: &[VertexId],
    ) -> IgraphResult<(Vec<Option<VertexId>>, Vec<VertexId>)> {
        let n_u32 = self.n;
        let n = n_u32 as usize;

        // Validate first.
        for &vid in vertices {
            if vid >= n_u32 {
                return Err(IgraphError::VertexOutOfRange { id: vid, n: n_u32 });
            }
        }

        let mut remove = vec![false; n];
        for &vid in vertices {
            remove[vid as usize] = true;
        }

        // Build map (old → new) and invmap (new → old).
        let mut map: Vec<Option<VertexId>> = vec![None; n];
        let mut invmap: Vec<VertexId> = Vec::new();
        let mut next_new: u32 = 0;
        for (i, &is_removed) in remove.iter().enumerate() {
            if !is_removed {
                let i_u32 = u32::try_from(i)
                    .map_err(|_| IgraphError::Internal("vertex index exceeds u32::MAX"))?;
                map[i] = Some(next_new);
                invmap.push(i_u32);
                next_new = next_new
                    .checked_add(1)
                    .ok_or(IgraphError::Internal("new vertex count overflow"))?;
            }
        }

        // Filter edges: keep only those with both endpoints retained,
        // renumber endpoints via `map`.
        let m = self.ecount();
        let mut new_from: Vec<VertexId> = Vec::with_capacity(m);
        let mut new_to: Vec<VertexId> = Vec::with_capacity(m);
        for (u, v) in self.from.iter().zip(self.to.iter()) {
            if let (Some(nu), Some(nv)) = (map[*u as usize], map[*v as usize]) {
                new_from.push(nu);
                new_to.push(nv);
            }
        }

        self.n = next_new;
        self.from = new_from;
        self.to = new_to;
        self.rebuild_indexes()?;

        Ok((map, invmap))
    }

    fn check_vertex(&self, v: VertexId) -> IgraphResult<()> {
        if v >= self.n {
            return Err(IgraphError::VertexOutOfRange { id: v, n: self.n });
        }
        Ok(())
    }

    fn check_edge(&self, eid: EdgeId) -> IgraphResult<()> {
        let m = self.ecount();
        let m_u32 = u32::try_from(m).unwrap_or(u32::MAX);
        if (eid as usize) >= m {
            return Err(IgraphError::EdgeOutOfRange { id: eid, m: m_u32 });
        }
        Ok(())
    }

    /// Recompute `oi`, `ii`, `os`, `is` from `from`/`to`. Called after
    /// any structural change.
    ///
    /// Each side does a stable lexicographic sort: `oi` orders edges by
    /// `(from[e], to[e])`, `ii` by `(to[e], from[e])`. Time complexity
    /// is `O(|V| + |E| log |E|)` (Rust stable sort) — same asymptotic
    /// as upstream's `igraph_vector_int_pair_order`.
    ///
    /// The within-bucket secondary sort matches upstream igraph; without
    /// it, `neighbors(v)` for an unsorted-edge-input graph diverges from
    /// `python-igraph`'s output and breaks DFS order parity. (Counted
    /// for an oracle-test failure during ALGO-TR-002 — see
    /// `tests/oracle.rs::dfs_small_synthetic_matches_python_igraph`.)
    ///
    /// Counterpart of `igraph_i_create_start_vectors` + the
    /// `igraph_vector_int_pair_order` calls in
    /// `type_indexededgelist.c:309-336`.
    fn rebuild_indexes(&mut self) -> IgraphResult<()> {
        let m = self.ecount();
        let n = self.n as usize;

        // Build (primary_key, secondary_key, edge_id) tuples for each
        // side, sort them lexicographically, then extract edge ids and
        // the offset array.

        // ---- Out-side: sort by (from, to). ----
        let mut tuples: Vec<(VertexId, VertexId, u32)> = (0..m)
            .map(|e| {
                Ok::<_, IgraphError>((
                    self.from[e],
                    self.to[e],
                    u32::try_from(e)
                        .map_err(|_| IgraphError::Internal("edge id exceeds u32::MAX"))?,
                ))
            })
            .collect::<Result<_, _>>()?;
        tuples.sort_unstable_by_key(|a| (a.0, a.1));
        self.oi = tuples.iter().map(|t| t.2).collect();
        // os[v] = number of entries with primary_key < v.
        self.os = vec![0u32; n + 1];
        for &(u, _, _) in &tuples {
            self.os[u as usize + 1] = self.os[u as usize + 1]
                .checked_add(1)
                .ok_or(IgraphError::Internal("degree overflow in rebuild_indexes"))?;
        }
        for i in 1..=n {
            self.os[i] = self.os[i]
                .checked_add(self.os[i - 1])
                .ok_or(IgraphError::Internal("offset overflow in rebuild_indexes"))?;
        }

        // ---- In-side: sort by (to, from). ----
        let mut tuples: Vec<(VertexId, VertexId, u32)> = (0..m)
            .map(|e| {
                Ok::<_, IgraphError>((
                    self.to[e],
                    self.from[e],
                    u32::try_from(e)
                        .map_err(|_| IgraphError::Internal("edge id exceeds u32::MAX"))?,
                ))
            })
            .collect::<Result<_, _>>()?;
        tuples.sort_unstable_by_key(|a| (a.0, a.1));
        self.ii = tuples.iter().map(|t| t.2).collect();
        self.is = vec![0u32; n + 1];
        for &(v, _, _) in &tuples {
            self.is[v as usize + 1] = self.is[v as usize + 1]
                .checked_add(1)
                .ok_or(IgraphError::Internal("degree overflow in rebuild_indexes"))?;
        }
        for i in 1..=n {
            self.is[i] = self.is[i]
                .checked_add(self.is[i - 1])
                .ok_or(IgraphError::Internal("offset overflow in rebuild_indexes"))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_counts() {
        let g = Graph::with_vertices(0);
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
        assert!(!g.is_directed());
    }

    #[test]
    fn new_directed_flag() {
        let g = Graph::new(3, true).unwrap();
        assert!(g.is_directed());
        let g = Graph::new(3, false).unwrap();
        assert!(!g.is_directed());
    }

    #[test]
    fn add_vertices_then_edges() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 2);
        assert_eq!(g.degree(1).unwrap(), 2);
        let mut nbrs = g.neighbors(1).unwrap();
        nbrs.sort_unstable();
        assert_eq!(nbrs, vec![0, 2]);
    }

    #[test]
    fn out_of_range_vertex_errors() {
        let mut g = Graph::with_vertices(2);
        let err = g.add_edge(0, 5).unwrap_err();
        assert!(matches!(err, IgraphError::VertexOutOfRange { id: 5, n: 2 }));
    }

    #[test]
    fn self_loop_counted_correctly() {
        let mut g = Graph::with_vertices(1);
        g.add_edge(0, 0).unwrap();
        assert_eq!(g.ecount(), 1);
        // Undirected self-loop: appears as both out and in, degree == 2.
        assert_eq!(g.degree(0).unwrap(), 2);
        let mut nbrs = g.neighbors(0).unwrap();
        nbrs.sort_unstable();
        assert_eq!(nbrs, vec![0, 0]);
    }

    #[test]
    fn parallel_edges() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        assert_eq!(g.ecount(), 2);
        assert_eq!(g.degree(0).unwrap(), 2);
        assert_eq!(g.degree(1).unwrap(), 2);
    }

    #[test]
    fn undirected_canonicalisation() {
        // Adding edges (1,0) and (0,1) — both stored canonically as (0,1).
        let mut g = Graph::with_vertices(2);
        g.add_edge(1, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        assert_eq!(g.ecount(), 2);
        // Both vertices see each other as a neighbour twice.
        let mut n0 = g.neighbors(0).unwrap();
        let mut n1 = g.neighbors(1).unwrap();
        n0.sort_unstable();
        n1.sort_unstable();
        assert_eq!(n0, vec![1, 1]);
        assert_eq!(n1, vec![0, 0]);
    }

    #[test]
    fn directed_neighbors_are_outgoing_only() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 0).unwrap();
        // Directed: neighbors(0) returns out-neighbours only.
        assert_eq!(g.neighbors(0).unwrap(), vec![1]);
        // Vertex 2 has out-edge to 0.
        assert_eq!(g.neighbors(2).unwrap(), vec![0]);
        // Vertex 1 has no out-edges.
        assert!(g.neighbors(1).unwrap().is_empty());
        // Degree counts both in and out for directed.
        assert_eq!(g.degree(0).unwrap(), 2); // out: 0->1, in: 2->0
        assert_eq!(g.degree(1).unwrap(), 1); // in: 0->1
        assert_eq!(g.degree(2).unwrap(), 1); // out: 2->0
    }

    #[test]
    fn add_edges_batch_then_rebuild() {
        let mut g = Graph::with_vertices(4);
        g.add_edges(vec![(0, 1), (0, 2), (1, 2), (2, 3)]).unwrap();
        assert_eq!(g.ecount(), 4);
        // Degrees: 0->{1,2} d=2; 1->{0,2} d=2; 2->{0,1,3} d=3; 3->{2} d=1.
        assert_eq!(g.degree(0).unwrap(), 2);
        assert_eq!(g.degree(1).unwrap(), 2);
        assert_eq!(g.degree(2).unwrap(), 3);
        assert_eq!(g.degree(3).unwrap(), 1);
    }

    #[test]
    fn clone_is_deep() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        let g2 = g.clone();
        // Mutate g; g2 must be unaffected.
        g.add_edge(0, 2).unwrap();
        assert_eq!(g.ecount(), 3);
        assert_eq!(g2.ecount(), 2);
    }

    #[test]
    fn os_invariant_is_monotone() {
        let mut g = Graph::with_vertices(5);
        g.add_edges(vec![(0, 1), (0, 2), (3, 4), (1, 2)]).unwrap();
        // os should be non-decreasing and end at ecount.
        for w in g.os.windows(2) {
            assert!(w[0] <= w[1]);
        }
        assert_eq!(g.os[0], 0);
        assert_eq!(*g.os.last().unwrap() as usize, g.ecount());
    }

    #[test]
    fn vertex_out_of_range_when_adding_edge() {
        let mut g = Graph::with_vertices(2);
        let e = g.add_edge(2, 0).unwrap_err();
        assert!(matches!(e, IgraphError::VertexOutOfRange { id: 2, n: 2 }));
        // Graph state must be unchanged after the failed add.
        assert_eq!(g.ecount(), 0);
    }

    // -------- ALGO-CORE-001b: edge-id helpers + incident --------

    #[test]
    fn edge_endpoints_round_trip() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0, 1), (2, 0), (1, 2)]).unwrap();
        // Directed: order preserved. edge_id == position in from/to.
        assert_eq!(g.edge(0).unwrap(), (0, 1));
        assert_eq!(g.edge(1).unwrap(), (2, 0));
        assert_eq!(g.edge(2).unwrap(), (1, 2));
        assert_eq!(g.edge_source(1).unwrap(), 2);
        assert_eq!(g.edge_target(1).unwrap(), 0);
    }

    #[test]
    fn edge_other_endpoint() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 2).unwrap();
        assert_eq!(g.edge_other(0, 0).unwrap(), 2);
        assert_eq!(g.edge_other(0, 2).unwrap(), 0);
        // Vertex not on the edge: error.
        let err = g.edge_other(0, 1).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn edge_out_of_range() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let err = g.edge(5).unwrap_err();
        assert!(matches!(err, IgraphError::EdgeOutOfRange { id: 5, m: 1 }));
    }

    #[test]
    fn incident_returns_edge_ids_matching_neighbors_order() {
        let mut g = Graph::with_vertices(4);
        g.add_edges(vec![(0, 1), (0, 2), (3, 0)]).unwrap();
        let eids = g.incident(0).unwrap();
        // Expect three incident edges; resolving back to neighbours
        // must equal `neighbors(0)` exactly (same iteration order).
        let resolved: Vec<u32> = eids.iter().map(|&e| g.edge_other(e, 0).unwrap()).collect();
        assert_eq!(resolved, g.neighbors(0).unwrap());
    }

    #[test]
    fn incident_self_loop_appears_twice_undirected() {
        let mut g = Graph::with_vertices(1);
        g.add_edge(0, 0).unwrap();
        let eids = g.incident(0).unwrap();
        // Undirected self-loop appears once on the out side and once on
        // the in side — same edge id, twice. Mirrors `neighbors`.
        assert_eq!(eids, vec![0, 0]);
        assert_eq!(g.degree(0).unwrap(), 2);
    }

    #[test]
    fn incident_directed_returns_outgoing_only() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0, 1), (2, 0)]).unwrap();
        // Directed `incident` mirrors directed `neighbors` (out only).
        assert_eq!(g.incident(0).unwrap(), vec![0]);
        assert_eq!(g.incident(2).unwrap(), vec![1]);
        assert!(g.incident(1).unwrap().is_empty());
    }

    #[test]
    fn get_eid_undirected_finds_edge_either_way() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap(); // edge 0
        g.add_edge(1, 2).unwrap(); // edge 1
        assert_eq!(g.get_eid(0, 1).unwrap(), 0);
        assert_eq!(g.get_eid(1, 0).unwrap(), 0);
        assert_eq!(g.get_eid(1, 2).unwrap(), 1);
        assert_eq!(g.get_eid(2, 1).unwrap(), 1);
    }

    #[test]
    fn get_eid_directed_respects_direction() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap(); // edge 0
        assert_eq!(g.get_eid(0, 1).unwrap(), 0);
        assert!(g.get_eid(1, 0).is_err()); // reverse direction has no edge
    }

    #[test]
    fn find_eid_returns_none_for_missing() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        assert_eq!(g.find_eid(0, 2).unwrap(), None);
        assert!(g.find_eid(0, 99).is_err()); // out-of-range vertex
    }

    #[test]
    fn get_eid_self_loop() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap(); // self-loop, edge 0
        g.add_edge(0, 1).unwrap(); // edge 1
        assert_eq!(g.get_eid(0, 0).unwrap(), 0);
        assert_eq!(g.get_eid(0, 1).unwrap(), 1);
    }

    #[test]
    fn get_all_eids_between_returns_all_parallel() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap(); // edge 0
        g.add_edge(0, 1).unwrap(); // edge 1
        g.add_edge(0, 1).unwrap(); // edge 2
        let eids = g.get_all_eids_between(0, 1).unwrap();
        assert_eq!(eids, vec![0, 1, 2]);
        // Reverse direction yields the same edges on undirected.
        let eids = g.get_all_eids_between(1, 0).unwrap();
        assert_eq!(eids, vec![0, 1, 2]);
    }

    #[test]
    fn get_all_eids_between_directed_one_way_only() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap(); // edge 0
        g.add_edge(0, 1).unwrap(); // edge 1 (parallel)
        assert_eq!(g.get_all_eids_between(0, 1).unwrap(), vec![0, 1]);
        // Reverse direction has no edges in directed graph.
        assert_eq!(g.get_all_eids_between(1, 0).unwrap(), Vec::<EdgeId>::new());
    }

    #[test]
    fn get_eid_returns_lowest_id_for_parallel() {
        // Spec: with multiple edges, get_eid always returns the same
        // edge id (matches upstream's "ignored multi-edges" guarantee).
        // Our impl returns the lowest from the bucket.
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap(); // edge 0
        g.add_edge(0, 1).unwrap(); // edge 1
        assert_eq!(g.get_eid(0, 1).unwrap(), 0);
    }

    // -------- ALGO-CORE-001c: delete_edges + delete_vertices --------

    #[test]
    fn delete_edges_empty_input_is_noop() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        g.delete_edges(&[]).unwrap();
        assert_eq!(g.ecount(), 2);
        assert_eq!(g.degree(1).unwrap(), 2);
    }

    #[test]
    fn delete_edges_single_edge_undirected() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0, 1), (1, 2), (0, 2)]).unwrap();
        // Remove edge id 1 (the (1,2) edge).
        g.delete_edges(&[1]).unwrap();
        assert_eq!(g.ecount(), 2);
        // Surviving edges renumbered to 0,1: (0,1) and (0,2).
        assert_eq!(g.find_eid(0, 1).unwrap(), Some(0));
        assert_eq!(g.find_eid(0, 2).unwrap(), Some(1));
        assert_eq!(g.find_eid(1, 2).unwrap(), None);
        // Degrees consistent post-rebuild.
        assert_eq!(g.degree(1).unwrap(), 1);
        assert_eq!(g.degree(2).unwrap(), 1);
    }

    #[test]
    fn delete_edges_duplicate_ids_tolerated() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        g.delete_edges(&[0, 0, 0]).unwrap();
        assert_eq!(g.ecount(), 1);
        assert_eq!(g.find_eid(1, 2).unwrap(), Some(0));
    }

    #[test]
    fn delete_edges_all_edges_leaves_isolated_vertices() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        g.delete_edges(&[0, 1]).unwrap();
        assert_eq!(g.ecount(), 0);
        assert_eq!(g.vcount(), 3);
        for v in 0..3 {
            assert_eq!(g.degree(v).unwrap(), 0);
        }
    }

    #[test]
    fn delete_edges_out_of_range_errors_and_preserves_state() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        let err = g.delete_edges(&[5]).unwrap_err();
        assert!(matches!(err, IgraphError::EdgeOutOfRange { id: 5, m: 2 }));
        // Graph unchanged.
        assert_eq!(g.ecount(), 2);
    }

    #[test]
    fn delete_edges_self_loop_directed() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edges(vec![(0, 0), (0, 1)]).unwrap();
        g.delete_edges(&[0]).unwrap(); // remove the self-loop
        assert_eq!(g.ecount(), 1);
        assert_eq!(g.degree(0).unwrap(), 1);
        assert_eq!(g.find_eid(0, 1).unwrap(), Some(0));
    }

    #[test]
    fn delete_vertices_empty_input_is_noop() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        g.delete_vertices(&[]).unwrap();
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 2);
    }

    #[test]
    fn delete_vertices_single_renumbers() {
        let mut g = Graph::with_vertices(4);
        g.add_edges(vec![(0, 1), (1, 2), (2, 3), (0, 3)]).unwrap();
        // Remove vertex 1: edges (0,1) and (1,2) go with it. (2,3),(0,3)
        // survive but get renumbered: 2 → 1, 3 → 2.
        g.delete_vertices(&[1]).unwrap();
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 2);
        // (2,3) → (1,2); (0,3) → (0,2).
        assert!(g.find_eid(1, 2).unwrap().is_some());
        assert!(g.find_eid(0, 2).unwrap().is_some());
        assert_eq!(g.find_eid(0, 1).unwrap(), None);
    }

    #[test]
    fn delete_vertices_duplicate_ids_tolerated() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        g.delete_vertices(&[1, 1, 1]).unwrap();
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn delete_vertices_all_yields_empty_graph() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        g.delete_vertices(&[0, 1, 2]).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn delete_vertices_out_of_range_errors_and_preserves_state() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        let err = g.delete_vertices(&[5]).unwrap_err();
        assert!(matches!(err, IgraphError::VertexOutOfRange { id: 5, n: 3 }));
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 2);
    }

    #[test]
    fn delete_vertices_map_returns_correct_mappings() {
        let mut g = Graph::with_vertices(5);
        g.add_edges(vec![(0, 1), (1, 2), (2, 3), (3, 4)]).unwrap();
        let (map, invmap) = g.delete_vertices_map(&[1, 3]).unwrap();
        // Removed: 1 and 3. Retained: 0 → 0, 2 → 1, 4 → 2.
        assert_eq!(map, vec![Some(0), None, Some(1), None, Some(2)]);
        assert_eq!(invmap, vec![0, 2, 4]);
        assert_eq!(g.vcount(), 3);
        // Only edges between retained vertices survive — none do here.
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn delete_vertices_directed_preserves_direction() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 0), (3, 0)]).unwrap();
        g.delete_vertices(&[3]).unwrap();
        assert_eq!(g.vcount(), 3);
        assert!(g.is_directed());
        // Surviving directed edges (3 → 0) gone; (0,1),(1,2),(2,0) keep direction.
        assert!(g.get_eid(0, 1).is_ok());
        assert!(g.get_eid(1, 0).is_err()); // wrong direction
    }

    #[test]
    fn delete_vertices_self_loop_on_removed_vertex() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0, 0), (0, 1), (1, 2)]).unwrap();
        g.delete_vertices(&[0]).unwrap();
        // Self-loop and edges to vertex 0 gone; only (1,2) → (0,1) survives.
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 1);
        assert!(g.find_eid(0, 1).unwrap().is_some());
    }

    #[test]
    fn delete_vertices_preserves_parallel_edges() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0, 1), (0, 1), (1, 2)]).unwrap();
        g.delete_vertices(&[2]).unwrap();
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 2); // both parallel (0,1) edges retained
        assert_eq!(g.degree(0).unwrap(), 2);
        assert_eq!(g.degree(1).unwrap(), 2);
    }

    #[test]
    fn add_edges_after_delete_works() {
        let mut g = Graph::with_vertices(4);
        g.add_edges(vec![(0, 1), (1, 2), (2, 3)]).unwrap();
        g.delete_vertices(&[0]).unwrap(); // now n=3, vertices 0,1,2
        // Add a new edge and check indexes still work.
        g.add_edge(0, 2).unwrap();
        assert_eq!(g.ecount(), 3);
        assert_eq!(g.degree(0).unwrap(), 2); // (0,1)+(0,2)
        assert!(g.find_eid(0, 2).unwrap().is_some());
    }
}

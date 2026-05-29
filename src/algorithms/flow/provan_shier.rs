//! Provan-Shier (s,t)-cut enumeration scaffolding shared by
//! [`all_st_cuts`](super::all_st_cuts) (ALGO-FL-031) and
//! [`all_st_mincuts`](super::all_st_mincuts) (ALGO-FL-032).
//!
//! Counterpart of the `igraph_marked_queue_int_t` / `igraph_estack_t`
//! containers plus `igraph_provan_shier_list` /
//! `igraph_i_provan_shier_list_recursive` from
//! `references/igraph/src/flow/st-cuts.c`. The binary-search recursion over
//! *closed* vertex sets `S` is identical for both cut listers; only the
//! `pivot` callback differs (dominator-tree based for all cuts, max-flow /
//! active-set based for minimum cuts), so igraph factors it out and we mirror
//! that here.

use crate::core::{IgraphResult, VertexId};

/// A batch stack with O(1) membership test (`igraph_marked_queue_int_t`).
///
/// Elements are pushed individually but removed a whole batch at a time.
/// `start_batch` records a batch boundary; `pop_back_batch` rewinds to the
/// most recent boundary, clearing membership for everything popped.
pub(crate) struct MarkedQueue {
    queue: Vec<i64>,
    member: Vec<bool>,
    size: usize,
}

impl MarkedQueue {
    pub(crate) fn new(n: usize) -> Self {
        Self {
            queue: Vec::new(),
            member: vec![false; n],
            size: 0,
        }
    }

    pub(crate) fn iselement(&self, e: VertexId) -> bool {
        self.member[e as usize]
    }

    pub(crate) fn size(&self) -> usize {
        self.size
    }

    pub(crate) fn push(&mut self, e: VertexId) {
        if !self.member[e as usize] {
            self.queue.push(i64::from(e));
            self.member[e as usize] = true;
            self.size += 1;
        }
    }

    pub(crate) fn start_batch(&mut self) {
        self.queue.push(-1);
    }

    pub(crate) fn pop_back_batch(&mut self) {
        while let Some(top) = self.queue.pop() {
            if top == -1 {
                break;
            }
            if let Ok(idx) = usize::try_from(top) {
                self.member[idx] = false;
                self.size -= 1;
            }
        }
    }

    /// Snapshot of the live elements in insertion order (markers skipped).
    pub(crate) fn as_vector(&self) -> Vec<VertexId> {
        // Markers are stored as `-1`, which `VertexId::try_from` rejects, so
        // `filter_map` drops them without a separate predicate.
        self.queue
            .iter()
            .filter_map(|&e| VertexId::try_from(e).ok())
            .collect()
    }
}

/// A stack with O(1) membership test (`igraph_estack_t`).
pub(crate) struct EStack {
    stack: Vec<VertexId>,
    isin: Vec<bool>,
}

impl EStack {
    pub(crate) fn new(n: usize) -> Self {
        Self {
            stack: Vec::new(),
            isin: vec![false; n],
        }
    }

    pub(crate) fn iselement(&self, e: VertexId) -> bool {
        self.isin[e as usize]
    }

    pub(crate) fn push(&mut self, e: VertexId) {
        if !self.isin[e as usize] {
            self.stack.push(e);
            self.isin[e as usize] = true;
        }
    }

    pub(crate) fn pop(&mut self) {
        if let Some(e) = self.stack.pop() {
            self.isin[e as usize] = false;
        }
    }
}

/// A pivot callback: given the current `S` / `T` it returns `(v, I(S,v))`.
/// An empty `I(S,v)` marks `S` as a leaf; otherwise the recursion branches
/// right (add `I(S,v)` to `S`) and left (forbid `v`).
///
/// `source` / `target` are forwarded verbatim from
/// [`provan_shier_list`] (the all-cuts pivot uses both; the min-cuts pivot
/// ignores them).
pub(crate) trait Pivot {
    fn pivot(
        &mut self,
        s: &MarkedQueue,
        t: &EStack,
        source: VertexId,
        target: VertexId,
    ) -> IgraphResult<(VertexId, Vec<VertexId>)>;
}

/// Enumerate every closed set produced by the Provan-Shier search tree
/// (`igraph_provan_shier_list`). Returns the source-side vertex sets in the
/// post-reversal order igraph uses, one per `(s,t)` cut.
pub(crate) fn provan_shier_list<P: Pivot>(
    n: usize,
    source: VertexId,
    target: VertexId,
    pivot: &mut P,
) -> IgraphResult<Vec<Vec<VertexId>>> {
    let mut s = MarkedQueue::new(n);
    let mut t = EStack::new(n);
    let mut result: Vec<Vec<VertexId>> = Vec::new();
    recurse(n, &mut s, &mut t, source, target, &mut result, pivot)?;
    // The recursion appends leaf-first; igraph reverses to keep the historical
    // ordering compatible with versions before 0.10.3.
    result.reverse();
    Ok(result)
}

/// `igraph_i_provan_shier_list_recursive`.
fn recurse<P: Pivot>(
    n: usize,
    s: &mut MarkedQueue,
    t: &mut EStack,
    source: VertexId,
    target: VertexId,
    result: &mut Vec<Vec<VertexId>>,
    pivot: &mut P,
) -> IgraphResult<()> {
    let (v, isv) = pivot.pivot(s, t, source, target)?;

    if isv.is_empty() {
        let sz = s.size();
        if sz != 0 && sz != n {
            result.push(s.as_vector());
        }
        return Ok(());
    }

    // Down-right: add I(S,v) to S.
    s.start_batch();
    for &x in &isv {
        s.push(x);
    }
    recurse(n, s, t, source, target, result, pivot)?;
    s.pop_back_batch();

    // Down-left: forbid v.
    t.push(v);
    recurse(n, s, t, source, target, result, pivot)?;
    t.pop();

    Ok(())
}

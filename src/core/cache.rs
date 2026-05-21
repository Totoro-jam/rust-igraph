//! Per-graph property cache (ALGO-CORE-001f).
//!
//! Counterpart of `igraph_i_property_cache_t` from
//! `references/igraph/src/graph/caching.{h,c}` and the
//! `igraph_i_property_cache_*` developer-facing API in
//! `references/igraph/include/igraph_interface.h:89-94`.
//!
//! Stores the value of a small set of boolean graph properties (e.g.
//! `is_dag`, `is_forest`, `has_loop`) so that repeated calls don't
//! recompute. The cache lives on `Graph` itself and is invalidated on
//! any structural mutation; selective invalidation (e.g. *adding* an
//! edge cannot make a connected graph disconnected) mirrors upstream's
//! `igraph_i_property_cache_invalidate_conditionally`.
//!
//! ## Interior mutability
//!
//! igraph C updates the cache through a `const igraph_t *` pointer —
//! computing a property is not considered "modification" of the graph.
//! We mirror this with `Cell<u32>`: a property compute function takes
//! `&Graph` and may still populate the cache. Single-threaded for now;
//! future multi-thread support would swap to `AtomicU32`.
//!
//! ## Storage layout
//!
//! Two `u32`s, bit `i` corresponding to [`CachedProperty`] discriminant
//! `i`:
//!
//! - `known` — bit set iff property `i` has a cached value;
//! - `values` — bit set iff cached value of `i` is `true`.
//!
//! Reading `values` is only meaningful when the matching `known` bit
//! is set; `get` therefore returns `Option<bool>` rather than a raw
//! pair. This matches the C struct in
//! `references/igraph/src/graph/caching.h:29-34` exactly (one bit
//! per property in `known`, one byte per property in C `value[]` —
//! we bit-pack since Rust gives us free integer arithmetic).

use std::cell::Cell;

/// The set of boolean graph properties that may be cached.
///
/// Discriminant values **must stay stable** — they are used as bit
/// indices into the cache's bitfield. Mirrors the
/// `igraph_cached_property_t` enum at
/// `references/igraph/include/igraph_datatype.h:31-64`; the order is
/// preserved so the C-side reasoning (e.g. "adding an edge keeps
/// `IS_DAG` if cached false") translates by index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CachedProperty {
    /// Has at least one self-loop.
    HasLoop = 0,
    /// Has at least one multi-edge (direction-aware: `(a,b)` + `(b,a)`
    /// in a *directed* graph does **not** count as a multi-edge).
    HasMulti = 1,
    /// Has at least one reciprocal edge pair (directed graphs only).
    HasMutual = 2,
    /// Weakly connected.
    IsWeaklyConnected = 3,
    /// Strongly connected (directed graphs only).
    IsStronglyConnected = 4,
    /// Directed acyclic graph (directed graphs only).
    IsDag = 5,
    /// Forest — acyclic when edge directions are ignored.
    IsForest = 6,
}

/// Total number of cached property slots. Must equal
/// `CachedProperty::all().len()`. We keep a constant rather than
/// reflecting it dynamically so bitmask compile-time checks are easy.
pub(crate) const PROP_COUNT: u8 = 7;

impl CachedProperty {
    #[inline]
    fn bit(self) -> u32 {
        1u32 << (self as u8)
    }
}

/// Boolean property cache, embedded in [`Graph`](crate::Graph).
///
/// Field-private; users interact through `Graph::cache_*` helpers.
/// Cloned by deep copy — the cached values carry over so a `Graph::clone`
/// doesn't need to recompute properties on the new copy.
#[derive(Debug, Clone, Default)]
pub(crate) struct PropertyCache {
    /// Bit `i` set iff property `i` has a cached value.
    known: Cell<u32>,
    /// Bit `i` set iff cached value of property `i` is `true`. Bits
    /// where `known` is 0 are meaningless.
    values: Cell<u32>,
}

impl PropertyCache {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn has(&self, prop: CachedProperty) -> bool {
        self.known.get() & prop.bit() != 0
    }

    pub(crate) fn get(&self, prop: CachedProperty) -> Option<bool> {
        if self.has(prop) {
            Some(self.values.get() & prop.bit() != 0)
        } else {
            None
        }
    }

    /// Store `value` for `prop`, overwriting any previous cached value.
    ///
    /// Counterpart of `igraph_i_property_cache_set_bool`.
    pub(crate) fn set(&self, prop: CachedProperty, value: bool) {
        let bit = prop.bit();
        self.known.set(self.known.get() | bit);
        let cur = self.values.get();
        self.values.set(if value { cur | bit } else { cur & !bit });
    }

    /// Drop the cached value of `prop` (no-op if not cached).
    ///
    /// Counterpart of `igraph_i_property_cache_invalidate`.
    pub(crate) fn invalidate(&self, prop: CachedProperty) {
        self.known.set(self.known.get() & !prop.bit());
    }

    /// Drop every cached value.
    ///
    /// Counterpart of `igraph_i_property_cache_invalidate_all`.
    pub(crate) fn invalidate_all(&self) {
        self.known.set(0);
    }

    /// Selective invalidation: drop every cached value **except** those
    /// listed in `keep_always`, plus those listed in `keep_when_false` /
    /// `keep_when_true` whose currently-cached value matches.
    ///
    /// Mask convention: bit `i` corresponds to
    /// `1u32 << (CachedProperty::X as u8)`. Use [`CachedProperty::bit`]
    /// when constructing masks externally — but this is `pub(crate)`, so
    /// in practice callers go through the higher-level helpers below
    /// (`invalidate_after_add_edge`, etc.).
    ///
    /// Counterpart of `igraph_i_property_cache_invalidate_conditionally`.
    pub(crate) fn invalidate_conditionally(
        &self,
        keep_always: u32,
        keep_when_false: u32,
        keep_when_true: u32,
    ) {
        let mut invalidate = !keep_always;
        let known = self.known.get();
        let values = self.values.get();
        let maybe_keep = known & invalidate & (keep_when_false | keep_when_true);
        if maybe_keep != 0 {
            for i in 0..PROP_COUNT {
                let mask = 1u32 << i;
                if maybe_keep & mask != 0 {
                    let cached_value = values & mask != 0;
                    if (keep_when_false & mask != 0 && !cached_value)
                        || (keep_when_true & mask != 0 && cached_value)
                    {
                        invalidate &= !mask;
                    }
                }
            }
        }
        self.known.set(known & !invalidate);
    }
}

// ----- pre-baked invalidation policies ---------------------------------------

/// After [`Graph::add_vertices`](crate::Graph::add_vertices): adding
/// isolated vertices cannot affect any edge-level property
/// (`HasLoop`, `HasMulti`, `HasMutual`, `IsDag`, `IsForest` are all
/// preserved). It can only **break** connectivity (a known-disconnected
/// graph stays disconnected; a known-connected graph may become
/// disconnected).
pub(crate) fn invalidate_after_add_vertices(cache: &PropertyCache) {
    use CachedProperty::{
        HasLoop, HasMulti, HasMutual, IsDag, IsForest, IsStronglyConnected, IsWeaklyConnected,
    };
    cache.invalidate_conditionally(
        HasLoop.bit() | HasMulti.bit() | HasMutual.bit() | IsDag.bit() | IsForest.bit(),
        IsWeaklyConnected.bit() | IsStronglyConnected.bit(),
        0,
    );
}

/// After [`Graph::add_edges`](crate::Graph::add_edges): mirrors the
/// C policy at `type_indexededgelist.c:341-364`.
///
/// - Adding an edge **never** disconnects a connected graph and
///   **never** removes existing self-loops / multi-edges / reciprocal
///   pairs → keep `IS_*_CONNECTED`, `HAS_LOOP`, `HAS_MULTI`,
///   `HAS_MUTUAL` if currently cached **true**.
/// - Adding an edge **may** turn a DAG into a non-DAG, or a forest
///   into a non-forest → keep `IS_DAG`, `IS_FOREST` only if currently
///   cached **false**.
pub(crate) fn invalidate_after_add_edges(cache: &PropertyCache) {
    use CachedProperty::{
        HasLoop, HasMulti, HasMutual, IsDag, IsForest, IsStronglyConnected, IsWeaklyConnected,
    };
    cache.invalidate_conditionally(
        0,
        IsDag.bit() | IsForest.bit(),
        IsWeaklyConnected.bit()
            | IsStronglyConnected.bit()
            | HasLoop.bit()
            | HasMulti.bit()
            | HasMutual.bit(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_cache_misses() {
        let c = PropertyCache::new();
        assert!(!c.has(CachedProperty::IsDag));
        assert_eq!(c.get(CachedProperty::IsDag), None);
    }

    #[test]
    fn set_then_get_round_trips() {
        let c = PropertyCache::new();
        c.set(CachedProperty::IsDag, true);
        c.set(CachedProperty::IsForest, false);
        assert_eq!(c.get(CachedProperty::IsDag), Some(true));
        assert_eq!(c.get(CachedProperty::IsForest), Some(false));
        assert_eq!(c.get(CachedProperty::HasLoop), None);
    }

    #[test]
    fn overwrite_changes_value() {
        let c = PropertyCache::new();
        c.set(CachedProperty::HasLoop, true);
        assert_eq!(c.get(CachedProperty::HasLoop), Some(true));
        c.set(CachedProperty::HasLoop, false);
        assert_eq!(c.get(CachedProperty::HasLoop), Some(false));
    }

    #[test]
    fn invalidate_one_keeps_others() {
        let c = PropertyCache::new();
        c.set(CachedProperty::HasLoop, true);
        c.set(CachedProperty::IsDag, false);
        c.invalidate(CachedProperty::HasLoop);
        assert_eq!(c.get(CachedProperty::HasLoop), None);
        assert_eq!(c.get(CachedProperty::IsDag), Some(false));
    }

    #[test]
    fn invalidate_all_clears_everything() {
        let c = PropertyCache::new();
        c.set(CachedProperty::HasLoop, true);
        c.set(CachedProperty::IsDag, false);
        c.invalidate_all();
        for p in [
            CachedProperty::HasLoop,
            CachedProperty::IsDag,
            CachedProperty::IsForest,
        ] {
            assert_eq!(c.get(p), None);
        }
    }

    #[test]
    fn invalidate_conditionally_keep_always() {
        let c = PropertyCache::new();
        c.set(CachedProperty::IsDag, true);
        c.set(CachedProperty::HasLoop, false);
        c.invalidate_conditionally(CachedProperty::IsDag.bit(), 0, 0);
        assert_eq!(c.get(CachedProperty::IsDag), Some(true));
        assert_eq!(c.get(CachedProperty::HasLoop), None);
    }

    #[test]
    fn invalidate_conditionally_keep_when_false() {
        let c = PropertyCache::new();
        c.set(CachedProperty::IsDag, false);
        c.invalidate_conditionally(0, CachedProperty::IsDag.bit(), 0);
        assert_eq!(c.get(CachedProperty::IsDag), Some(false));

        // Same mask but value cached as true — should now drop.
        let c = PropertyCache::new();
        c.set(CachedProperty::IsDag, true);
        c.invalidate_conditionally(0, CachedProperty::IsDag.bit(), 0);
        assert_eq!(c.get(CachedProperty::IsDag), None);
    }

    #[test]
    fn invalidate_conditionally_keep_when_true() {
        let c = PropertyCache::new();
        c.set(CachedProperty::HasLoop, true);
        c.invalidate_conditionally(0, 0, CachedProperty::HasLoop.bit());
        assert_eq!(c.get(CachedProperty::HasLoop), Some(true));

        let c = PropertyCache::new();
        c.set(CachedProperty::HasLoop, false);
        c.invalidate_conditionally(0, 0, CachedProperty::HasLoop.bit());
        assert_eq!(c.get(CachedProperty::HasLoop), None);
    }

    #[test]
    fn add_edges_policy_keeps_dag_false_and_loop_true() {
        let c = PropertyCache::new();
        c.set(CachedProperty::IsDag, false);
        c.set(CachedProperty::HasLoop, true);
        c.set(CachedProperty::IsForest, true); // should drop after add_edges
        c.set(CachedProperty::HasMulti, false); // should drop
        invalidate_after_add_edges(&c);
        assert_eq!(c.get(CachedProperty::IsDag), Some(false));
        assert_eq!(c.get(CachedProperty::HasLoop), Some(true));
        assert_eq!(c.get(CachedProperty::IsForest), None);
        assert_eq!(c.get(CachedProperty::HasMulti), None);
    }

    #[test]
    fn add_vertices_policy_keeps_edge_props_unconditionally() {
        let c = PropertyCache::new();
        c.set(CachedProperty::HasLoop, false);
        c.set(CachedProperty::HasMulti, true);
        c.set(CachedProperty::HasMutual, true);
        c.set(CachedProperty::IsDag, true);
        c.set(CachedProperty::IsForest, false);
        c.set(CachedProperty::IsWeaklyConnected, true); // should drop
        invalidate_after_add_vertices(&c);
        assert_eq!(c.get(CachedProperty::HasLoop), Some(false));
        assert_eq!(c.get(CachedProperty::HasMulti), Some(true));
        assert_eq!(c.get(CachedProperty::HasMutual), Some(true));
        assert_eq!(c.get(CachedProperty::IsDag), Some(true));
        assert_eq!(c.get(CachedProperty::IsForest), Some(false));
        assert_eq!(c.get(CachedProperty::IsWeaklyConnected), None);
    }

    #[test]
    fn clone_preserves_cache() {
        let c = PropertyCache::new();
        c.set(CachedProperty::IsDag, true);
        let c2 = c.clone();
        assert_eq!(c2.get(CachedProperty::IsDag), Some(true));
    }
}

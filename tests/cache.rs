//! Integration tests for `Graph` boolean property cache (CORE-001f).
//!
//! These tests cross the public API surface — they confirm that
//! - compute functions populate the cache,
//! - cache survives `Clone`,
//! - mutations (`add_vertices` / `add_edges` / `delete_*`) invalidate
//!   per the policy in `core::cache::invalidate_after_*`,
//! - selective invalidation correctly preserves values that the
//!   mutation cannot affect.

use rust_igraph::DijkstraMode;
use rust_igraph::core::cache::CachedProperty;
use rust_igraph::{Graph, has_loop, has_multiple, is_dag, is_forest};

#[test]
fn is_dag_populates_cache_on_first_call() {
    let mut g = Graph::new(3, true).unwrap();
    g.add_edges(vec![(0u32, 1u32), (1, 2)]).unwrap();
    assert_eq!(g.cache_get(CachedProperty::IsDag), None);
    let result = is_dag(&g);
    assert!(result);
    assert_eq!(g.cache_get(CachedProperty::IsDag), Some(true));
}

#[test]
fn is_dag_cache_serves_subsequent_calls() {
    let mut g = Graph::new(3, true).unwrap();
    g.add_edges(vec![(0u32, 1u32), (1, 2)]).unwrap();
    let _ = is_dag(&g);
    // Poison the cache to a value that contradicts reality; if the
    // function reads from the cache, it must return that wrong value.
    g.cache_set(CachedProperty::IsDag, false);
    assert!(!is_dag(&g));
}

#[test]
fn add_edges_invalidates_dag_only_when_cached_true() {
    let mut g = Graph::new(3, true).unwrap();
    g.add_edges(vec![(0u32, 1u32), (1, 2)]).unwrap();
    let _ = is_dag(&g); // populates IsDag = true
    assert_eq!(g.cache_get(CachedProperty::IsDag), Some(true));

    // Adding an edge could break DAG-ness, so cache must drop.
    g.add_edge(2, 0).unwrap();
    assert_eq!(g.cache_get(CachedProperty::IsDag), None);
    assert!(!is_dag(&g));
    assert_eq!(g.cache_get(CachedProperty::IsDag), Some(false));

    // Adding another edge to a known-non-DAG graph cannot make it
    // a DAG again, so cache stays at false.
    g.add_edge(0, 1).unwrap();
    assert_eq!(g.cache_get(CachedProperty::IsDag), Some(false));
}

#[test]
fn add_vertices_keeps_dag_cache() {
    let mut g = Graph::new(3, true).unwrap();
    g.add_edges(vec![(0u32, 1u32), (1, 2)]).unwrap();
    let _ = is_dag(&g);
    assert_eq!(g.cache_get(CachedProperty::IsDag), Some(true));
    g.add_vertices(2).unwrap();
    // Adding isolated vertices cannot break DAG-ness.
    assert_eq!(g.cache_get(CachedProperty::IsDag), Some(true));
}

#[test]
fn delete_edges_invalidates_all() {
    let mut g = Graph::new(3, true).unwrap();
    g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 0)]).unwrap();
    let _ = is_dag(&g);
    assert_eq!(g.cache_get(CachedProperty::IsDag), Some(false));
    g.delete_edges(&[2]).unwrap();
    assert_eq!(g.cache_get(CachedProperty::IsDag), None);
    assert!(is_dag(&g));
}

#[test]
fn delete_vertices_invalidates_all() {
    let mut g = Graph::new(3, true).unwrap();
    g.add_edges(vec![(0u32, 1u32), (1, 2)]).unwrap();
    let _ = is_dag(&g);
    assert_eq!(g.cache_get(CachedProperty::IsDag), Some(true));
    g.delete_vertices(&[2]).unwrap();
    assert_eq!(g.cache_get(CachedProperty::IsDag), None);
}

#[test]
fn has_loop_populates_and_serves_cache() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    assert_eq!(g.cache_get(CachedProperty::HasLoop), None);
    assert!(!has_loop(&g).unwrap());
    assert_eq!(g.cache_get(CachedProperty::HasLoop), Some(false));

    g.add_edge(1, 1).unwrap();
    // Add-edge keeps HasLoop iff cached true. Cached as false → drop.
    assert_eq!(g.cache_get(CachedProperty::HasLoop), None);
    assert!(has_loop(&g).unwrap());
    assert_eq!(g.cache_get(CachedProperty::HasLoop), Some(true));

    // Add another edge — HasLoop=true is preserved (loop can't disappear by add).
    g.add_edge(0, 2).unwrap();
    assert_eq!(g.cache_get(CachedProperty::HasLoop), Some(true));
}

#[test]
fn has_multiple_cache_hit_miss() {
    let mut g = Graph::with_vertices(3);
    g.add_edges(vec![(0u32, 1u32), (1, 2)]).unwrap();
    assert!(!has_multiple(&g).unwrap());
    assert_eq!(g.cache_get(CachedProperty::HasMulti), Some(false));

    // Add a parallel edge — HasMulti=false drops, recomputed to true.
    g.add_edge(0, 1).unwrap();
    assert_eq!(g.cache_get(CachedProperty::HasMulti), None);
    assert!(has_multiple(&g).unwrap());
    assert_eq!(g.cache_get(CachedProperty::HasMulti), Some(true));
}

#[test]
fn is_forest_populates_cache_in_all_mode() {
    let mut g = Graph::with_vertices(4);
    g.add_edges(vec![(0u32, 1u32), (2, 3)]).unwrap();
    let _ = is_forest(&g, DijkstraMode::All).unwrap();
    assert_eq!(g.cache_get(CachedProperty::IsForest), Some(true));

    // Add an edge closing a cycle — drops IsForest=true, recomputes false.
    g.add_edges(vec![(1u32, 2u32), (3, 0)]).unwrap();
    assert_eq!(g.cache_get(CachedProperty::IsForest), None);
    assert!(is_forest(&g, DijkstraMode::All).unwrap().is_none());
    assert_eq!(g.cache_get(CachedProperty::IsForest), Some(false));
}

#[test]
fn directed_is_forest_only_caches_in_all_mode() {
    let mut g = Graph::new(3, true).unwrap();
    g.add_edges(vec![(0u32, 2u32), (1, 2)]).unwrap();
    // Out mode: not a forest (in-degree 2 at vertex 2).
    assert!(is_forest(&g, DijkstraMode::Out).unwrap().is_none());
    // But Out-mode result should NOT have populated IsForest cache,
    // because IsForest semantics are "underlying-undirected-forest".
    assert_eq!(g.cache_get(CachedProperty::IsForest), None);

    // All mode: IS a forest (path 0-2-1 ignoring direction).
    let _ = is_forest(&g, DijkstraMode::All).unwrap();
    assert_eq!(g.cache_get(CachedProperty::IsForest), Some(true));
}

#[test]
fn clone_preserves_cache() {
    let mut g = Graph::new(3, true).unwrap();
    g.add_edges(vec![(0u32, 1u32), (1, 2)]).unwrap();
    let _ = is_dag(&g);
    let g2 = g.clone();
    assert_eq!(g2.cache_get(CachedProperty::IsDag), Some(true));
}

#[test]
fn cache_invalidate_one_keeps_others() {
    let mut g = Graph::new(3, true).unwrap();
    g.add_edges(vec![(0u32, 1u32), (1, 2)]).unwrap();
    let _ = is_dag(&g);
    let _ = has_loop(&g).unwrap();
    g.cache_invalidate(CachedProperty::IsDag);
    assert_eq!(g.cache_get(CachedProperty::IsDag), None);
    assert_eq!(g.cache_get(CachedProperty::HasLoop), Some(false));
}

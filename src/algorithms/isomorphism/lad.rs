//! LAD subgraph isomorphism (`ALGO-ISO-020`).
//!
//! Pure-Rust port of `igraph_subisomorphic_lad` and the AllDifferent-based
//! constraint-propagation engine behind it, from
//! `references/igraph/src/isomorphism/lad.c` (Christine Solnon's LADv1,
//! adapted by igraph).
//!
//! LAD models subgraph isomorphism as a constraint satisfaction problem: each
//! pattern vertex `u` owns a *domain* `D(u)` of candidate target vertices, and
//! an AllDifferent constraint forbids two pattern vertices from mapping to the
//! same target vertex. Filtering interleaves three propagators —
//! `checkLAD` (each `(u,v)` must admit an `adj(u)`-covering matching),
//! `ensureGACallDiff` (generalized arc consistency for AllDifferent via a
//! residual-graph SCC analysis) and forward-checking on edges — driven to a
//! fixpoint, then a branch-and-propagate search instantiates the smallest
//! remaining domain.
//!
//! Like upstream, multi-edges are rejected; self-loops are allowed. Directed
//! and undirected graphs are both supported (the pattern and target must agree
//! on directedness). The optional `domains` argument restricts each pattern
//! vertex to an explicit candidate list (e.g. to emulate vertex colours).

// The LAD engine is a faithful port of a pointer/index-heavy C algorithm that
// juggles `usize` slice indices, `u32` [`Graph`] ids and `i64` `-1`-sentinel
// slots. Every value is bounded by the vertex/edge count, which fits `u32` by
// the [`Graph`] contract, so the conversions cannot truncate, wrap or lose
// sign in practice.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    // Faithful port: the C source uses terse, deliberately similar index names
    // (u/v/d/gp/gt, nb_v/nb_dv, list_v/list_dv) that are clearest kept as-is.
    clippy::many_single_char_names,
    clippy::similar_names,
    // Prose mentions algorithm concepts (AllDifferent, LADv1, ensureGACallDiff).
    clippy::doc_markdown,
    // Mirrors the C control flow `if (a != b) { ... } else { ... }`.
    clippy::if_not_else,
    // `Solver` collects the C out-params (firstSol/iso/want_map/want_maps).
    clippy::struct_excessive_bools
)]

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult};

/// Result of a first-solution LAD subgraph-isomorphism search
/// ([`subisomorphic_lad`]).
#[derive(Debug, Clone)]
pub struct LadSubisomorphism {
    /// Whether `pattern` is isomorphic to a subgraph of `target`.
    pub iso: bool,
    /// For each pattern vertex (in id order), the matched target vertex.
    /// Empty when no embedding exists.
    pub map: Vec<u32>,
}

/// igraph's `Tgraph`: out-adjacency view with loops-once, no multi-edges.
struct Tgraph {
    nb_vertices: i64,
    nb_succ: Vec<i64>,
    succ: Vec<Vec<i64>>,
    is_edge: Vec<bool>,
}

impl Tgraph {
    #[inline]
    fn is_edge(&self, i: i64, j: i64) -> bool {
        self.is_edge[(i * self.nb_vertices + j) as usize]
    }
}

/// Build the LAD `Tgraph` from a [`Graph`], reproducing
/// `igraph_i_lad_createGraph` (out-adjacency, `IGRAPH_LOOPS_ONCE`,
/// multi-edges rejected).
fn create_graph(g: &Graph) -> IgraphResult<Tgraph> {
    let n = i64::from(g.vcount());
    let directed = g.is_directed();
    let mut succ: Vec<Vec<i64>> = vec![Vec::new(); n as usize];
    for eid in 0..g.ecount() {
        let (a, b) = g.edge(eid as EdgeId)?;
        let (a, b) = (i64::from(a), i64::from(b));
        if directed {
            succ[a as usize].push(b);
        } else if a == b {
            succ[a as usize].push(a); // self-loop counted once
        } else {
            succ[a as usize].push(b);
            succ[b as usize].push(a);
        }
    }
    let mut is_edge = vec![false; (n * n) as usize];
    let mut nb_succ = vec![0i64; n as usize];
    for i in 0..n as usize {
        succ[i].sort_unstable();
        for w in 0..succ[i].len() {
            if w > 0 && succ[i][w] == succ[i][w - 1] {
                return Err(IgraphError::InvalidArgument(
                    "LAD functions do not support graphs with multi-edges.".into(),
                ));
            }
            let j = succ[i][w];
            is_edge[i * n as usize + j as usize] = true;
        }
        nb_succ[i] = succ[i].len() as i64;
    }
    Ok(Tgraph {
        nb_vertices: n,
        nb_succ,
        succ,
        is_edge,
    })
}

/// igraph's `Tdomain`: per-pattern-vertex candidate domains plus the global
/// AllDifferent matching and the `checkLAD` matching cache.
struct Tdomain {
    nb_t: i64,
    nb_val: Vec<i64>,
    first_val: Vec<i64>,
    val: Vec<i64>,
    pos_in_val: Vec<i64>,
    first_match: Vec<i64>,
    matching: Vec<i64>,
    next_out_to_filter: i64,
    last_in_to_filter: i64,
    to_filter: Vec<i64>,
    marked_to_filter: Vec<bool>,
    global_matching_p: Vec<i64>,
    global_matching_t: Vec<i64>,
}

impl Tdomain {
    #[inline]
    fn pos(&self, u: i64, v: i64) -> i64 {
        self.pos_in_val[(u * self.nb_t + v) as usize]
    }
    #[inline]
    fn set_pos(&mut self, u: i64, v: i64, p: i64) {
        let nb_t = self.nb_t;
        self.pos_in_val[(u * nb_t + v) as usize] = p;
    }
    #[inline]
    fn first_match(&self, u: i64, v: i64) -> i64 {
        self.first_match[(u * self.nb_t + v) as usize]
    }
    #[inline]
    fn is_in_d(&self, u: i64, v: i64) -> bool {
        self.pos(u, v) < self.first_val[u as usize] + self.nb_val[u as usize]
    }
    #[inline]
    fn to_filter_empty(&self) -> bool {
        self.next_out_to_filter < 0
    }
    fn reset_to_filter(&mut self) {
        for m in &mut self.marked_to_filter {
            *m = false;
        }
        self.next_out_to_filter = -1;
    }
    fn next_to_filter(&mut self, size: i64) -> i64 {
        let u = self.to_filter[self.next_out_to_filter as usize];
        self.marked_to_filter[u as usize] = false;
        if self.next_out_to_filter == self.last_in_to_filter {
            self.next_out_to_filter = -1;
        } else if self.next_out_to_filter == size - 1 {
            self.next_out_to_filter = 0;
        } else {
            self.next_out_to_filter += 1;
        }
        u
    }
    fn add_to_filter(&mut self, u: i64, size: i64) {
        if self.marked_to_filter[u as usize] {
            return;
        }
        self.marked_to_filter[u as usize] = true;
        if self.next_out_to_filter < 0 {
            self.last_in_to_filter = 0;
            self.next_out_to_filter = 0;
        } else if self.last_in_to_filter == size - 1 {
            self.last_in_to_filter = 0;
        } else {
            self.last_in_to_filter += 1;
        }
        self.to_filter[self.last_in_to_filter as usize] = u;
    }
}

/// Find an augmenting path from pattern vertex `u` in the bipartite graph
/// `{(u,v): v in D(u)} ∪ {(v,u): globalMatchingP[u]=v}`; update the global
/// matching and return whether one was found
/// (`igraph_i_lad_augmentingPath`).
fn augmenting_path(u: i64, d: &mut Tdomain, nb_v: i64) -> bool {
    let nb_v = nb_v as usize;
    let mut fifo = vec![0i64; nb_v];
    let mut pred = vec![0i64; nb_v];
    let mut marked = vec![false; nb_v];
    let mut next_in = 0usize;
    let mut next_out = 0usize;

    let fv = d.first_val[u as usize];
    for i in 0..d.nb_val[u as usize] {
        let v = d.val[(fv + i) as usize];
        if d.global_matching_t[v as usize] < 0 {
            d.global_matching_p[u as usize] = v;
            d.global_matching_t[v as usize] = u;
            return true;
        }
        pred[v as usize] = u;
        fifo[next_in] = v;
        next_in += 1;
        marked[v as usize] = true;
    }
    while next_out < next_in {
        let u2 = d.global_matching_t[fifo[next_out] as usize];
        next_out += 1;
        let fv2 = d.first_val[u2 as usize];
        for i in 0..d.nb_val[u2 as usize] {
            let v = d.val[(fv2 + i) as usize];
            if d.global_matching_t[v as usize] < 0 {
                let mut u2m = u2;
                let mut vv = v;
                while u2m != u {
                    let v2 = d.global_matching_p[u2m as usize];
                    d.global_matching_p[u2m as usize] = vv;
                    d.global_matching_t[vv as usize] = u2m;
                    vv = v2;
                    u2m = pred[vv as usize];
                }
                d.global_matching_p[u as usize] = vv;
                d.global_matching_t[vv as usize] = u;
                return true;
            }
            if !marked[v as usize] {
                pred[v as usize] = u2;
                fifo[next_in] = v;
                next_in += 1;
                marked[v as usize] = true;
            }
        }
    }
    false
}

/// Remove every value but `v` from `D(u)`, queue `u`'s successors, and repair
/// the global matching (`igraph_i_lad_removeAllValuesButOne`). Returns
/// `false` if an inconsistency wrt the global AllDifferent is detected.
fn remove_all_values_but_one(u: i64, v: i64, d: &mut Tdomain, gp: &Tgraph, gt: &Tgraph) -> bool {
    for j in 0..gp.succ[u as usize].len() {
        let s = gp.succ[u as usize][j];
        d.add_to_filter(s, gp.nb_vertices);
    }
    let old_pos = d.pos(u, v);
    let new_pos = d.first_val[u as usize];
    d.val[old_pos as usize] = d.val[new_pos as usize];
    d.val[new_pos as usize] = v;
    let a = d.val[new_pos as usize];
    let b = d.val[old_pos as usize];
    d.set_pos(u, a, new_pos);
    d.set_pos(u, b, old_pos);
    d.nb_val[u as usize] = 1;
    if d.global_matching_p[u as usize] != v {
        let old = d.global_matching_p[u as usize];
        d.global_matching_t[old as usize] = -1;
        d.global_matching_p[u as usize] = -1;
        augmenting_path(u, d, gt.nb_vertices)
    } else {
        true
    }
}

/// Remove value `v` from `D(u)`, queue `u`'s successors, and repair the
/// global matching (`igraph_i_lad_removeValue`). Returns `false` on a global
/// AllDifferent inconsistency.
fn remove_value(u: i64, v: i64, d: &mut Tdomain, gp: &Tgraph, gt: &Tgraph) -> bool {
    for j in 0..gp.succ[u as usize].len() {
        let s = gp.succ[u as usize][j];
        d.add_to_filter(s, gp.nb_vertices);
    }
    let old_pos = d.pos(u, v);
    d.nb_val[u as usize] -= 1;
    let new_pos = d.first_val[u as usize] + d.nb_val[u as usize];
    d.val[old_pos as usize] = d.val[new_pos as usize];
    d.val[new_pos as usize] = v;
    let a = d.val[old_pos as usize];
    let b = d.val[new_pos as usize];
    d.set_pos(u, a, old_pos);
    d.set_pos(u, b, new_pos);
    if d.global_matching_p[u as usize] == v {
        d.global_matching_p[u as usize] = -1;
        d.global_matching_t[v as usize] = -1;
        augmenting_path(u, d, gt.nb_vertices)
    } else {
        true
    }
}

/// Match every vertex in `stack` to the single value of its domain and forward
/// -check the others wrt edges and AllDifferent
/// (`igraph_i_lad_matchVertices`). Returns `true` if an inconsistency is
/// detected (the C `invalid` out-param).
fn match_vertices(
    mut stack: Vec<i64>,
    induced: bool,
    d: &mut Tdomain,
    gp: &Tgraph,
    gt: &Tgraph,
) -> bool {
    while let Some(u) = stack.pop() {
        let v = d.val[d.first_val[u as usize] as usize];
        for u2 in 0..gp.nb_vertices {
            if u == u2 {
                continue;
            }
            let old_nb_val = d.nb_val[u2 as usize];
            if d.is_in_d(u2, v) && !remove_value(u2, v, d, gp, gt) {
                return true;
            }
            if gp.is_edge(u, u2) {
                let mut j = d.first_val[u2 as usize];
                while j < d.first_val[u2 as usize] + d.nb_val[u2 as usize] {
                    let val_j = d.val[j as usize];
                    if gt.is_edge(v, val_j) {
                        j += 1;
                    } else if !remove_value(u2, val_j, d, gp, gt) {
                        return true;
                    }
                }
            } else if induced {
                if d.nb_val[u2 as usize] < gt.nb_succ[v as usize] {
                    let mut j = d.first_val[u2 as usize];
                    while j < d.first_val[u2 as usize] + d.nb_val[u2 as usize] {
                        let val_j = d.val[j as usize];
                        if !gt.is_edge(v, val_j) {
                            j += 1;
                        } else if !remove_value(u2, val_j, d, gp, gt) {
                            return true;
                        }
                    }
                } else {
                    for j in 0..gt.nb_succ[v as usize] {
                        let v2 = gt.succ[v as usize][j as usize];
                        if d.is_in_d(u2, v2) && !remove_value(u2, v2, d, gp, gt) {
                            return true;
                        }
                    }
                }
            }
            if d.nb_val[u2 as usize] == 0 {
                return true;
            }
            if d.nb_val[u2 as usize] == 1 && old_nb_val > 1 {
                stack.push(u2);
            }
        }
    }
    false
}

/// Single-vertex convenience wrapper over [`match_vertices`]
/// (`igraph_i_lad_matchVertex`). Returns `true` on success (no
/// inconsistency).
fn match_vertex(u: i64, induced: bool, d: &mut Tdomain, gp: &Tgraph, gt: &Tgraph) -> bool {
    !match_vertices(vec![u], induced, d, gp, gt)
}

/// `igraph_i_lad_compare`: true iff every element of `mu` can be matched to a
/// distinct element of `mv` that is `>=` it.
fn compare(mu: &mut [i64], mv: &mut [i64]) -> bool {
    mu.sort_unstable();
    mv.sort_unstable();
    let mut i = mv.len() as i64 - 1;
    for j in (0..mu.len()).rev() {
        if mu[j] > mv[i as usize] {
            return false;
        }
        i -= 1;
    }
    true
}

/// Initialize the domains (`igraph_i_lad_initDomains`). `v in D(u)` iff the
/// optional `domains[u]` mask admits it and the neighbour-degree multiset of
/// `u` can be matched into that of `v`. Returns the domain `D` and whether
/// some domain ended up empty.
#[allow(clippy::too_many_lines)]
fn init_domains(domains: Option<&[Vec<u32>]>, gp: &Tgraph, gt: &Tgraph) -> (Tdomain, bool) {
    let nb_p = gp.nb_vertices;
    let nb_t = gt.nb_vertices;
    let mut val: Vec<i64> = Vec::new();
    let mut nb_val = vec![0i64; nb_p as usize];
    let mut first_val = vec![0i64; nb_p as usize];
    let mut pos_in_val = vec![0i64; (nb_p * nb_t) as usize];
    let mut first_match = vec![0i64; (nb_p * nb_t) as usize];
    let global_matching_p = vec![-1i64; nb_p as usize];
    let global_matching_t = vec![-1i64; nb_t as usize];
    let mut to_filter = vec![0i64; nb_p as usize];
    let mut marked_to_filter = vec![false; nb_p as usize];

    let mut val_size: i64 = 0;
    let mut matching_size: i64 = 0;
    let mut dom = vec![false; nb_t as usize];

    for u in 0..nb_p {
        if let Some(doms) = domains {
            dom.fill(false);
            for &v in &doms[u as usize] {
                dom[v as usize] = true;
            }
        }
        marked_to_filter[u as usize] = true;
        to_filter[u as usize] = u;
        nb_val[u as usize] = 0;
        first_val[u as usize] = val_size;
        for v in 0..nb_t {
            if domains.is_some() && !dom[v as usize] {
                pos_in_val[(u * nb_t + v) as usize] = first_val[u as usize] + nb_t;
            } else {
                first_match[(u * nb_t + v) as usize] = matching_size;
                matching_size += gp.nb_succ[u as usize];
                if gp.nb_succ[u as usize] <= gt.nb_succ[v as usize] {
                    let mut mu: Vec<i64> = gp.succ[u as usize]
                        .iter()
                        .map(|&w| gp.nb_succ[w as usize])
                        .collect();
                    let mut mv: Vec<i64> = gt.succ[v as usize]
                        .iter()
                        .map(|&w| gt.nb_succ[w as usize])
                        .collect();
                    if compare(&mut mu, &mut mv) {
                        val.push(v);
                        nb_val[u as usize] += 1;
                        pos_in_val[(u * nb_t + v) as usize] = val_size;
                        val_size += 1;
                    } else {
                        pos_in_val[(u * nb_t + v) as usize] = first_val[u as usize] + nb_t;
                    }
                } else {
                    pos_in_val[(u * nb_t + v) as usize] = first_val[u as usize] + nb_t;
                }
            }
        }
        if nb_val[u as usize] == 0 {
            let d = Tdomain {
                nb_t,
                nb_val,
                first_val,
                val: Vec::new(),
                pos_in_val,
                first_match,
                matching: Vec::new(),
                next_out_to_filter: -1,
                last_in_to_filter: 0,
                to_filter,
                marked_to_filter,
                global_matching_p,
                global_matching_t,
            };
            return (d, true);
        }
    }

    let matching = vec![-1i64; matching_size as usize];
    let d = Tdomain {
        nb_t,
        nb_val,
        first_val,
        val,
        pos_in_val,
        first_match,
        matching,
        next_out_to_filter: 0,
        last_in_to_filter: nb_p - 1,
        to_filter,
        marked_to_filter,
        global_matching_p,
        global_matching_t,
    };
    (d, false)
}

const WHITE: i64 = 0;
const GREY: i64 = 1;
const BLACK: i64 = 2;
const TO_BE_DELETED: i64 = 3;
const DELETED: i64 = 4;

fn add_to_delete(u: i64, list: &mut [i64], nb: &mut i64, marked: &mut [i64]) {
    if marked[u as usize] < TO_BE_DELETED {
        list[*nb as usize] = u;
        *nb += 1;
        marked[u as usize] = TO_BE_DELETED;
    }
}

/// Hopcroft-Karp style maximum bipartite matching covering `U`
/// (`igraph_i_lad_updateMatching`). `matched_with_u[u]` is updated in place;
/// returns `true` if no covering matching exists (the C `invalid` out-param).
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn update_matching(
    size_of_u: i64,
    size_of_v: i64,
    degree: &[i64],
    first_adj: &[i64],
    adj: &[i64],
    matched_with_u: &mut [i64],
) -> bool {
    if size_of_u > size_of_v {
        return true;
    }
    let su = size_of_u as usize;
    let sv = size_of_v as usize;
    let mut matched_with_v = vec![-1i64; sv];
    let mut nb_pred = vec![0i64; sv];
    let mut pred = vec![0i64; sv * su];
    let mut nb_succ = vec![0i64; su];
    let mut succ = vec![0i64; su * sv];
    let mut list_v = vec![0i64; sv];
    let mut list_u = vec![0i64; su];
    let mut list_dv = vec![0i64; sv];
    let mut list_du = vec![0i64; su];
    let mut marked_v = vec![0i64; sv];
    let mut marked_u = vec![0i64; su];
    let mut unmatched = vec![0i64; su];
    let mut pos_in_unmatched = vec![0i64; su];
    let mut path: Vec<i64> = Vec::new();
    let mut nb_unmatched: i64 = 0;

    for u in 0..su {
        if matched_with_u[u] >= 0 {
            matched_with_v[matched_with_u[u] as usize] = u as i64;
        } else {
            pos_in_unmatched[u] = nb_unmatched;
            unmatched[nb_unmatched as usize] = u as i64;
            nb_unmatched += 1;
        }
    }

    // greedy: match unmatched U with any free V neighbour
    let mut j: i64 = 0;
    while j < nb_unmatched {
        let u = unmatched[j as usize];
        let start = first_adj[u as usize];
        let end = start + degree[u as usize];
        let mut i = start;
        while i < end && matched_with_v[adj[i as usize] as usize] >= 0 {
            i += 1;
        }
        if i == end {
            j += 1;
        } else {
            let v = adj[i as usize];
            matched_with_u[u as usize] = v;
            matched_with_v[v as usize] = u;
            nb_unmatched -= 1;
            unmatched[j as usize] = unmatched[nb_unmatched as usize];
            pos_in_unmatched[unmatched[j as usize] as usize] = j;
        }
    }

    while nb_unmatched > 0 {
        // step 1: build the layered DAG
        marked_u.fill(WHITE);
        nb_succ.fill(0);
        marked_v.fill(WHITE);
        nb_pred.fill(0);
        let mut nb_v: i64 = 0;
        for jj in 0..nb_unmatched {
            let u = unmatched[jj as usize];
            marked_u[u as usize] = BLACK;
            let start = first_adj[u as usize];
            let end = start + degree[u as usize];
            for i in start..end {
                let v = adj[i as usize];
                pred[(v * size_of_u + nb_pred[v as usize]) as usize] = u;
                nb_pred[v as usize] += 1;
                succ[(u * size_of_v + nb_succ[u as usize]) as usize] = v;
                nb_succ[u as usize] += 1;
                if marked_v[v as usize] == WHITE {
                    marked_v[v as usize] = GREY;
                    list_v[nb_v as usize] = v;
                    nb_v += 1;
                }
            }
        }
        let mut stop = false;
        while !stop && nb_v > 0 {
            let mut nb_u: i64 = 0;
            for i in 0..nb_v {
                let v = list_v[i as usize];
                marked_v[v as usize] = BLACK;
                let u = matched_with_v[v as usize];
                if marked_u[u as usize] == WHITE {
                    marked_u[u as usize] = GREY;
                    list_u[nb_u as usize] = u;
                    nb_u += 1;
                }
            }
            nb_v = 0;
            for jj in 0..nb_u {
                let u = list_u[jj as usize];
                marked_u[u as usize] = BLACK;
                let start = first_adj[u as usize];
                let end = start + degree[u as usize];
                for i in start..end {
                    let v = adj[i as usize];
                    if marked_v[v as usize] != BLACK {
                        pred[(v * size_of_u + nb_pred[v as usize]) as usize] = u;
                        nb_pred[v as usize] += 1;
                        succ[(u * size_of_v + nb_succ[u as usize]) as usize] = v;
                        nb_succ[u as usize] += 1;
                        if marked_v[v as usize] == WHITE {
                            marked_v[v as usize] = GREY;
                            list_v[nb_v as usize] = v;
                            nb_v += 1;
                        }
                        if matched_with_v[v as usize] == -1 {
                            stop = true;
                        }
                    }
                }
            }
        }
        if nb_v == 0 {
            return true;
        }

        // step 2: extract vertex-disjoint augmenting paths
        for k in 0..nb_v {
            let mut v = list_v[k as usize];
            if matched_with_v[v as usize] == -1 && nb_pred[v as usize] > 0 {
                path.clear();
                path.push(v);
                let mut nb_dv: i64 = 0;
                let mut nb_du: i64 = 0;
                add_to_delete(v, &mut list_dv, &mut nb_dv, &mut marked_v);
                let mut u;
                loop {
                    u = pred[(v * size_of_u) as usize];
                    path.push(u);
                    add_to_delete(u, &mut list_du, &mut nb_du, &mut marked_u);
                    if matched_with_u[u as usize] != -1 {
                        v = matched_with_u[u as usize];
                        path.push(v);
                        add_to_delete(v, &mut list_dv, &mut nb_dv, &mut marked_v);
                    }
                    if matched_with_u[u as usize] == -1 {
                        break;
                    }
                }

                while nb_dv > 0 || nb_du > 0 {
                    while nb_dv > 0 {
                        nb_dv -= 1;
                        let vv = list_dv[nb_dv as usize];
                        marked_v[vv as usize] = DELETED;
                        let uu = matched_with_v[vv as usize];
                        if uu != -1 {
                            add_to_delete(uu, &mut list_du, &mut nb_du, &mut marked_u);
                        }
                        for i in 0..nb_pred[vv as usize] {
                            let up = pred[(vv * size_of_u + i) as usize];
                            let mut jj: i64 = 0;
                            while jj < nb_succ[up as usize]
                                && vv != succ[(up * size_of_v + jj) as usize]
                            {
                                jj += 1;
                            }
                            nb_succ[up as usize] -= 1;
                            succ[(up * size_of_v + jj) as usize] =
                                succ[(up * size_of_v + nb_succ[up as usize]) as usize];
                            if nb_succ[up as usize] == 0 {
                                add_to_delete(up, &mut list_du, &mut nb_du, &mut marked_u);
                            }
                        }
                    }
                    while nb_du > 0 {
                        nb_du -= 1;
                        let uu = list_du[nb_du as usize];
                        marked_u[uu as usize] = DELETED;
                        let vv = matched_with_u[uu as usize];
                        if vv != -1 {
                            add_to_delete(vv, &mut list_dv, &mut nb_dv, &mut marked_v);
                        }
                        for i in 0..nb_succ[uu as usize] {
                            let vp = succ[(uu * size_of_v + i) as usize];
                            let mut jj: i64 = 0;
                            while jj < nb_pred[vp as usize]
                                && uu != pred[(vp * size_of_u + jj) as usize]
                            {
                                jj += 1;
                            }
                            nb_pred[vp as usize] -= 1;
                            pred[(vp * size_of_u + jj) as usize] =
                                pred[(vp * size_of_u + nb_pred[vp as usize]) as usize];
                            if nb_pred[vp as usize] == 0 {
                                add_to_delete(vp, &mut list_dv, &mut nb_dv, &mut marked_v);
                            }
                        }
                    }
                }
                let last = path[path.len() - 1];
                let i = pos_in_unmatched[last as usize];
                nb_unmatched -= 1;
                unmatched[i as usize] = unmatched[nb_unmatched as usize];
                pos_in_unmatched[unmatched[i as usize] as usize] = i;
                while path.len() > 1 {
                    let pu = path.pop().unwrap_or(-1);
                    let pv = path.pop().unwrap_or(-1);
                    matched_with_u[pu as usize] = pv;
                    matched_with_v[pv as usize] = pu;
                }
            }
        }
    }
    false
}

/// Depth-first numbering used by [`scc`] (`igraph_i_lad_DFS`).
#[allow(clippy::too_many_arguments)]
fn dfs(
    nb_u: i64,
    u: i64,
    marked: &mut [bool],
    nb_succ: &[i64],
    succ: &[i64],
    matched_with_u: &[i64],
    order: &mut [i64],
    nb: &mut i64,
) {
    let v = matched_with_u[u as usize];
    marked[u as usize] = true;
    if v >= 0 {
        for i in 0..nb_succ[v as usize] {
            let w = succ[(v * nb_u + i) as usize];
            if !marked[w as usize] {
                dfs(nb_u, w, marked, nb_succ, succ, matched_with_u, order, nb);
            }
        }
    }
    order[*nb as usize] = u;
    *nb -= 1;
}

/// Tarjan/Kosaraju-style strongly-connected-component labelling of the
/// residual bipartite graph (`igraph_i_lad_SCC`). Fills `num_v`/`num_u` with
/// matching component ids.
#[allow(clippy::too_many_arguments)]
fn scc(
    nb_u: i64,
    nb_v: i64,
    num_v: &mut [i64],
    num_u: &mut [i64],
    nb_succ: &[i64],
    succ: &[i64],
    nb_pred: &[i64],
    pred: &[i64],
    matched_with_u: &[i64],
    matched_with_v: &[i64],
) {
    let mut order = vec![0i64; nb_u as usize];
    let mut marked = vec![false; nb_u as usize];
    let mut fifo = vec![0i64; nb_v as usize];
    let mut nb = nb_u - 1;
    for u in 0..nb_u {
        if !marked[u as usize] {
            dfs(
                nb_u,
                u,
                &mut marked,
                nb_succ,
                succ,
                matched_with_u,
                &mut order,
                &mut nb,
            );
        }
    }
    let mut nb_scc: i64 = 0;
    for x in num_u.iter_mut() {
        *x = -1;
    }
    for x in num_v.iter_mut() {
        *x = -1;
    }
    for i in 0..nb_u {
        let u0 = order[i as usize];
        let v0 = matched_with_u[u0 as usize];
        if v0 == -1 {
            continue;
        }
        if num_v[v0 as usize] == -1 {
            nb_scc += 1;
            let mut k: i64 = 1;
            fifo[0] = v0;
            num_v[v0 as usize] = nb_scc;
            while k > 0 {
                k -= 1;
                let v = fifo[k as usize];
                let u = matched_with_v[v as usize];
                if u != -1 {
                    num_u[u as usize] = nb_scc;
                    for j in 0..nb_pred[u as usize] {
                        let vp = pred[(u * nb_v + j) as usize];
                        if num_v[vp as usize] == -1 {
                            num_v[vp as usize] = nb_scc;
                            fifo[k as usize] = vp;
                            k += 1;
                        }
                    }
                }
            }
        }
    }
}

/// Enforce generalized arc consistency for the global AllDifferent constraint
/// (`igraph_i_lad_ensureGACallDiff`). Returns `true` if an inconsistency is
/// found (the C `invalid` out-param).
#[allow(clippy::too_many_lines)]
fn ensure_gac_all_diff(induced: bool, gp: &Tgraph, gt: &Tgraph, d: &mut Tdomain) -> bool {
    let nb_p = gp.nb_vertices;
    let nb_t = gt.nb_vertices;
    let mut nb_pred = vec![0i64; nb_p as usize];
    let mut pred = vec![0i64; (nb_p * nb_t) as usize];
    let mut nb_succ = vec![0i64; nb_t as usize];
    let mut succ = vec![0i64; (nb_t * nb_p) as usize];
    let mut num_v = vec![0i64; nb_t as usize];
    let mut num_u = vec![0i64; nb_p as usize];
    let mut used = vec![false; (nb_p * nb_t) as usize];
    let mut list = vec![0i64; nb_t as usize];
    let mut nb: i64 = 0;

    for u in 0..nb_p {
        let fv = d.first_val[u as usize];
        for i in 0..d.nb_val[u as usize] {
            let v = d.val[(fv + i) as usize];
            used[(u * nb_t + v) as usize] = false;
            if v != d.global_matching_p[u as usize] {
                pred[(u * nb_t + nb_pred[u as usize]) as usize] = v;
                nb_pred[u as usize] += 1;
                succ[(v * nb_p + nb_succ[v as usize]) as usize] = u;
                nb_succ[v as usize] += 1;
            }
        }
    }

    // mark edges of alternating paths from free target vertices
    for v in 0..nb_t {
        if d.global_matching_t[v as usize] < 0 {
            list[nb as usize] = v;
            nb += 1;
            num_v[v as usize] = 1;
        }
    }
    while nb > 0 {
        nb -= 1;
        let v = list[nb as usize];
        for i in 0..nb_succ[v as usize] {
            let u = succ[(v * nb_p + i) as usize];
            used[(u * nb_t + v) as usize] = true;
            if num_u[u as usize] == 0 {
                num_u[u as usize] = 1;
                let w = d.global_matching_p[u as usize];
                used[(u * nb_t + w) as usize] = true;
                if num_v[w as usize] == 0 {
                    list[nb as usize] = w;
                    nb += 1;
                    num_v[w as usize] = 1;
                }
            }
        }
    }

    scc(
        nb_p,
        nb_t,
        &mut num_v,
        &mut num_u,
        &nb_succ,
        &succ,
        &nb_pred,
        &pred,
        &d.global_matching_p,
        &d.global_matching_t,
    );

    let mut to_match: Vec<i64> = Vec::new();
    for u in 0..nb_p {
        let old_nb_val = d.nb_val[u as usize];
        // NOTE: upstream uses a `for (i=0; i<nbVal[u]; i++)` loop here and
        // advances `i` unconditionally even after a removal (unlike
        // `match_vertices`); the GAC removal set is fixed by the SCC labelling
        // computed above, so a swapped-in tail value is intentionally not
        // re-examined in this pass.
        let mut i: i64 = 0;
        while i < d.nb_val[u as usize] {
            let v = d.val[(d.first_val[u as usize] + i) as usize];
            if !used[(u * nb_t + v) as usize]
                && num_v[v as usize] != num_u[u as usize]
                && d.global_matching_p[u as usize] != v
                && !remove_value(u, v, d, gp, gt)
            {
                return true;
            }
            i += 1;
        }
        if d.nb_val[u as usize] == 0 {
            return true;
        }
        if old_nb_val > 1 && d.nb_val[u as usize] == 1 {
            to_match.push(u);
        }
    }
    match_vertices(to_match, induced, d, gp, gt)
}

/// Check whether `G_(u,v)` has an `adj(u)`-covering matching, updating the
/// cached matching for `(u,v)` (`igraph_i_lad_checkLAD`).
#[allow(clippy::too_many_lines)]
fn check_lad(u: i64, v: i64, d: &mut Tdomain, gp: &Tgraph, gt: &Tgraph) -> bool {
    let nb_succ_u = gp.nb_succ[u as usize];
    let fm = d.first_match(u, v);

    // special case: u has a single neighbour => no Hopcroft-Karp needed
    if nb_succ_u == 1 {
        let u2 = gp.succ[u as usize][0];
        let v2 = d.matching[fm as usize];
        if v2 != -1 && d.is_in_d(u2, v2) {
            return true;
        }
        let start = d.first_val[u2 as usize];
        let end = start + d.nb_val[u2 as usize];
        for i in start..end {
            if gt.is_edge(v, d.val[i as usize]) {
                d.matching[fm as usize] = d.val[i as usize];
                return true;
            }
        }
        return false;
    }

    // general case
    let mut nb_matched = 0i64;
    for i in 0..nb_succ_u {
        let u2 = gp.succ[u as usize][i as usize];
        let v2 = d.matching[(fm + i) as usize];
        if v2 != -1 && d.is_in_d(u2, v2) {
            nb_matched += 1;
        }
    }
    if nb_matched == nb_succ_u {
        return true;
    }

    let nb_t = gt.nb_vertices;
    let mut num = vec![-1i64; nb_t as usize];
    let mut num_inv = vec![0i64; nb_t as usize];
    let mut nb_comp = vec![0i64; nb_succ_u as usize];
    let mut first_comp = vec![0i64; nb_succ_u as usize];
    let mut comp = vec![0i64; (nb_succ_u * nb_t) as usize];
    let mut matched_with_u = vec![0i64; nb_succ_u as usize];
    let mut nb_num = 0i64;
    let mut pos_in_comp = 0i64;

    for i in 0..nb_succ_u {
        let u2 = gp.succ[u as usize][i as usize];
        nb_comp[i as usize] = 0;
        first_comp[i as usize] = pos_in_comp;
        if d.nb_val[u2 as usize] > gt.nb_succ[v as usize] {
            let start = d.first_val[u2 as usize];
            let end = start + d.nb_val[u2 as usize];
            for j in start..end {
                let v2 = d.val[j as usize];
                if gt.is_edge(v, v2) {
                    if num[v2 as usize] < 0 {
                        num[v2 as usize] = nb_num;
                        num_inv[nb_num as usize] = v2;
                        nb_num += 1;
                    }
                    comp[pos_in_comp as usize] = num[v2 as usize];
                    pos_in_comp += 1;
                    nb_comp[i as usize] += 1;
                }
            }
        } else {
            for j in 0..gt.nb_succ[v as usize] {
                let v2 = gt.succ[v as usize][j as usize];
                if d.is_in_d(u2, v2) {
                    if num[v2 as usize] < 0 {
                        num[v2 as usize] = nb_num;
                        num_inv[nb_num as usize] = v2;
                        nb_num += 1;
                    }
                    comp[pos_in_comp as usize] = num[v2 as usize];
                    pos_in_comp += 1;
                    nb_comp[i as usize] += 1;
                }
            }
        }
        if nb_comp[i as usize] == 0 {
            return false;
        }
        let v2 = d.matching[(fm + i) as usize];
        matched_with_u[i as usize] = if v2 != -1 && d.is_in_d(u2, v2) {
            num[v2 as usize]
        } else {
            -1
        };
    }

    let invalid = update_matching(
        nb_succ_u,
        nb_num,
        &nb_comp,
        &first_comp,
        &comp,
        &mut matched_with_u,
    );
    if invalid {
        return false;
    }
    for i in 0..nb_succ_u {
        d.matching[(fm + i) as usize] = num_inv[matched_with_u[i as usize] as usize];
    }
    true
}

/// Filter all queued domains wrt LAD and AllDifferent to a fixpoint
/// (`igraph_i_lad_filter`). Returns `false` if some domain becomes empty.
fn filter(induced: bool, d: &mut Tdomain, gp: &Tgraph, gt: &Tgraph) -> bool {
    while !d.to_filter_empty() {
        while !d.to_filter_empty() {
            let u = d.next_to_filter(gp.nb_vertices);
            let old_nb_val = d.nb_val[u as usize];
            let mut i = d.first_val[u as usize];
            while i < d.first_val[u as usize] + d.nb_val[u as usize] {
                let v = d.val[i as usize];
                if check_lad(u, v, d, gp, gt) {
                    i += 1;
                } else if !remove_value(u, v, d, gp, gt) {
                    return false;
                }
            }
            if d.nb_val[u as usize] == 1 && old_nb_val > 1 && !match_vertex(u, induced, d, gp, gt) {
                return false;
            }
            if d.nb_val[u as usize] == 0 {
                return false;
            }
        }
        if ensure_gac_all_diff(induced, gp, gt, d) {
            return false;
        }
    }
    true
}

/// Accumulators threaded through the recursive [`solve`] search.
struct Solver {
    first_sol: bool,
    iso: bool,
    map: Vec<i64>,
    maps: Vec<Vec<i64>>,
    want_map: bool,
    want_maps: bool,
    nb_sol: i64,
}

/// Branch-and-propagate search (`igraph_i_lad_solve`): instantiate the
/// smallest non-singleton domain and recurse, collecting solutions per the
/// [`Solver`] configuration.
fn solve(induced: bool, d: &mut Tdomain, gp: &Tgraph, gt: &Tgraph, s: &mut Solver) {
    let nb_p = gp.nb_vertices;
    if !filter(induced, d, gp, gt) {
        d.reset_to_filter();
        return;
    }

    let mut nb_val_snapshot = vec![0i64; nb_p as usize];
    let mut global_snapshot = vec![0i64; nb_p as usize];
    let mut min_dom = -1i64;
    for u in 0..nb_p {
        nb_val_snapshot[u as usize] = d.nb_val[u as usize];
        if d.nb_val[u as usize] > 1
            && (min_dom < 0 || d.nb_val[u as usize] < d.nb_val[min_dom as usize])
        {
            min_dom = u;
        }
        global_snapshot[u as usize] = d.global_matching_p[u as usize];
    }

    if min_dom == -1 {
        // every vertex is fixed => solution found
        s.iso = true;
        s.nb_sol += 1;
        if s.want_map && s.map.is_empty() {
            s.map = (0..nb_p)
                .map(|u| d.val[d.first_val[u as usize] as usize])
                .collect();
        }
        if s.want_maps {
            let m: Vec<i64> = (0..nb_p)
                .map(|u| d.val[d.first_val[u as usize] as usize])
                .collect();
            s.maps.push(m);
        }
        d.reset_to_filter();
        return;
    }

    let nb_min = nb_val_snapshot[min_dom as usize];
    let mut branch_vals = vec![0i64; nb_min as usize];
    for i in 0..nb_min {
        branch_vals[i as usize] = d.val[(d.first_val[min_dom as usize] + i) as usize];
    }

    let mut i = 0i64;
    while i < nb_min && (!s.first_sol || s.nb_sol == 0) {
        let v = branch_vals[i as usize];
        let ok = remove_all_values_but_one(min_dom, v, d, gp, gt)
            && match_vertex(min_dom, induced, d, gp, gt);
        if ok {
            solve(induced, d, gp, gt, s);
        } else {
            d.reset_to_filter();
        }
        // restore domain sizes and the global AllDifferent matching
        for x in &mut d.global_matching_t {
            *x = -1;
        }
        for u in 0..nb_p {
            d.nb_val[u as usize] = nb_val_snapshot[u as usize];
            d.global_matching_p[u as usize] = global_snapshot[u as usize];
            d.global_matching_t[global_snapshot[u as usize] as usize] = u;
        }
        i += 1;
    }
}

/// Shared engine behind the two public wrappers. `first_sol` stops at the first
/// embedding; otherwise every embedding is enumerated. Returns
/// `(iso, map, maps)` with pattern→target vertex ids.
fn lad_engine(
    pattern: &Graph,
    target: &Graph,
    domains: Option<&[Vec<u32>]>,
    induced: bool,
    first_sol: bool,
) -> IgraphResult<(bool, Vec<u32>, Vec<Vec<u32>>)> {
    if pattern.is_directed() != target.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "Cannot search for a directed pattern in an undirected target or vice versa.".into(),
        ));
    }

    let want_maps = !first_sol;
    let n_pat = pattern.vcount();

    // null pattern: trivially isomorphic to the empty subgraph
    if n_pat == 0 {
        let maps = if want_maps {
            vec![Vec::new()]
        } else {
            Vec::new()
        };
        return Ok((true, Vec::new(), maps));
    }

    if let Some(doms) = domains {
        if doms.len() != n_pat as usize {
            return Err(IgraphError::InvalidArgument(format!(
                "Domain list length ({}) must match the number of pattern vertices ({}).",
                doms.len(),
                n_pat
            )));
        }
        let n_t = target.vcount();
        for d in doms {
            for &v in d {
                if v >= n_t {
                    return Err(IgraphError::InvalidArgument(format!(
                        "Domain contains target vertex id {v} out of range (target has {n_t} vertices)."
                    )));
                }
            }
        }
    }

    let gp = create_graph(pattern)?;
    let gt = create_graph(target)?;

    if gp.nb_vertices > gt.nb_vertices {
        return Ok((false, Vec::new(), Vec::new()));
    }

    let (mut d, invalid_domain) = init_domains(domains, &gp, &gt);
    if invalid_domain {
        return Ok((false, Vec::new(), Vec::new()));
    }

    // global AllDifferent matching over the full domains
    let nb_p = gp.nb_vertices;
    let mut gmp = std::mem::take(&mut d.global_matching_p);
    let invalid = update_matching(
        gp.nb_vertices,
        gt.nb_vertices,
        &d.nb_val,
        &d.first_val,
        &d.val,
        &mut gmp,
    );
    d.global_matching_p = gmp;
    if invalid {
        return Ok((false, Vec::new(), Vec::new()));
    }

    if ensure_gac_all_diff(induced, &gp, &gt, &mut d) {
        return Ok((false, Vec::new(), Vec::new()));
    }

    for u in 0..nb_p {
        let v = d.global_matching_p[u as usize];
        d.global_matching_t[v as usize] = u;
    }

    let mut to_match: Vec<i64> = Vec::new();
    for u in 0..nb_p {
        if d.nb_val[u as usize] == 1 {
            to_match.push(u);
        }
    }
    if match_vertices(to_match, induced, &mut d, &gp, &gt) {
        return Ok((false, Vec::new(), Vec::new()));
    }

    let mut solver = Solver {
        first_sol,
        iso: false,
        map: Vec::new(),
        maps: Vec::new(),
        want_map: !want_maps,
        want_maps,
        nb_sol: 0,
    };
    solve(induced, &mut d, &gp, &gt, &mut solver);

    let map_u32: Vec<u32> = solver.map.iter().map(|&x| x as u32).collect();
    let maps_u32: Vec<Vec<u32>> = solver
        .maps
        .iter()
        .map(|m| m.iter().map(|&x| x as u32).collect())
        .collect();
    Ok((solver.iso, map_u32, maps_u32))
}

/// Decide whether `pattern` is isomorphic to a subgraph of `target` using the
/// LAD algorithm, returning the first embedding found.
///
/// `pattern` and `target` must agree on directedness. Multi-edges are
/// rejected; self-loops are allowed. The optional `domains` restricts each
/// pattern vertex (in id order) to an explicit list of compatible target
/// vertices — useful for colour-constrained matching. When `induced` is
/// `true`, non-edges of the pattern must map to non-edges of the target
/// (induced subgraph); when `false`, only pattern edges must be preserved
/// (monomorphism).
///
/// Port of `igraph_subisomorphic_lad` with `maps == NULL` (first-solution
/// mode).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, subisomorphic_lad};
///
/// // A triangle pattern embeds into K4.
/// let mut target = Graph::new(4, false)?;
/// for (u, v) in [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)] {
///     target.add_edge(u, v)?;
/// }
/// let mut pattern = Graph::new(3, false)?;
/// for (u, v) in [(0, 1), (1, 2), (2, 0)] {
///     pattern.add_edge(u, v)?;
/// }
/// let res = subisomorphic_lad(&pattern, &target, None, false)?;
/// assert!(res.iso);
/// assert_eq!(res.map.len(), 3);
/// # Ok::<(), rust_igraph::IgraphError>(())
/// ```
pub fn subisomorphic_lad(
    pattern: &Graph,
    target: &Graph,
    domains: Option<&[Vec<u32>]>,
    induced: bool,
) -> IgraphResult<LadSubisomorphism> {
    let (iso, map, _) = lad_engine(pattern, target, domains, induced, true)?;
    Ok(LadSubisomorphism { iso, map })
}

/// Enumerate *all* subgraph isomorphisms from `pattern` into `target` with the
/// LAD algorithm. Each returned vector maps pattern vertex `i` (by index) to
/// its target vertex.
///
/// Arguments mirror [`subisomorphic_lad`]. A null pattern yields a single
/// empty mapping. Port of `igraph_subisomorphic_lad` with a non-null `maps`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, get_subisomorphisms_lad};
///
/// // A single edge embeds into a triangle in six ordered ways.
/// let mut target = Graph::new(3, false)?;
/// for (u, v) in [(0, 1), (1, 2), (2, 0)] {
///     target.add_edge(u, v)?;
/// }
/// let mut pattern = Graph::new(2, false)?;
/// pattern.add_edge(0, 1)?;
/// let maps = get_subisomorphisms_lad(&pattern, &target, None, false)?;
/// assert_eq!(maps.len(), 6);
/// # Ok::<(), rust_igraph::IgraphError>(())
/// ```
pub fn get_subisomorphisms_lad(
    pattern: &Graph,
    target: &Graph,
    domains: Option<&[Vec<u32>]>,
    induced: bool,
) -> IgraphResult<Vec<Vec<u32>>> {
    let (_, _, maps) = lad_engine(pattern, target, domains, induced, false)?;
    Ok(maps)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn undirected(n: u32, edges: &[(u32, u32)]) -> Graph {
        let mut g = Graph::new(n, false).expect("graph init");
        for &(a, b) in edges {
            g.add_edge(a, b).expect("edge in range");
        }
        g
    }

    /// The graphs from `igraph_subisomorphic_lad.out`.
    fn example_graphs() -> (Graph, Graph) {
        let target = undirected(
            9,
            &[
                (0, 1),
                (0, 4),
                (0, 6),
                (1, 4),
                (1, 2),
                (2, 3),
                (3, 4),
                (3, 5),
                (3, 7),
                (3, 8),
                (4, 5),
                (4, 6),
                (5, 6),
                (5, 8),
                (7, 8),
            ],
        );
        let pattern = undirected(5, &[(0, 1), (0, 4), (1, 4), (1, 2), (2, 3), (3, 4)]);
        (pattern, target)
    }

    fn sorted(mut maps: Vec<Vec<u32>>) -> Vec<Vec<u32>> {
        maps.sort();
        maps
    }

    /// Verify `map` is a valid (monomorphic) embedding of `pattern` in `target`.
    fn is_valid_embedding(map: &[u32], pattern: &Graph, target: &Graph) -> bool {
        if map.len() != pattern.vcount() as usize {
            return false;
        }
        // injective
        let mut seen = std::collections::HashSet::new();
        if !map.iter().all(|&v| seen.insert(v)) {
            return false;
        }
        for eid in 0..pattern.ecount() {
            let (a, b) = pattern.edge(eid as EdgeId).expect("pattern edge");
            if target
                .find_eid(map[a as usize], map[b as usize])
                .expect("valid target vertices")
                .is_none()
            {
                return false;
            }
        }
        true
    }

    #[test]
    fn example_oracle_monomorphism() {
        let (pattern, target) = example_graphs();
        let maps = get_subisomorphisms_lad(&pattern, &target, None, false).expect("lad");
        assert_eq!(maps.len(), 20, "monomorphism count from the C example");
        for m in &maps {
            assert!(is_valid_embedding(m, &pattern, &target));
        }
    }

    #[test]
    fn example_oracle_induced() {
        let (pattern, target) = example_graphs();
        let maps = get_subisomorphisms_lad(&pattern, &target, None, true).expect("lad");
        let expected = sorted(vec![
            vec![0, 1, 2, 3, 4],
            vec![5, 3, 2, 1, 4],
            vec![5, 4, 1, 2, 3],
            vec![0, 4, 3, 2, 1],
        ]);
        assert_eq!(
            sorted(maps),
            expected,
            "induced embeddings from the C example"
        );
    }

    #[test]
    fn example_oracle_domains() {
        let (pattern, target) = example_graphs();
        let domains = vec![
            vec![0u32, 2, 8],
            vec![4, 5, 6, 7],
            vec![1, 3, 5, 6, 7, 8],
            vec![0, 2, 8],
            vec![1, 3, 7, 8],
        ];
        let maps = get_subisomorphisms_lad(&pattern, &target, Some(&domains), false).expect("lad");
        assert_eq!(
            maps,
            vec![vec![0u32, 4, 3, 2, 1]],
            "domain-restricted embedding"
        );
    }

    #[test]
    fn first_solution_mode() {
        let (pattern, target) = example_graphs();
        let res = subisomorphic_lad(&pattern, &target, None, false).expect("lad");
        assert!(res.iso);
        assert!(is_valid_embedding(&res.map, &pattern, &target));
    }

    #[test]
    fn null_pattern_is_trivially_iso() {
        let target = undirected(3, &[(0, 1), (1, 2)]);
        let empty = Graph::new(0, false).expect("graph init");
        let res = subisomorphic_lad(&empty, &target, None, false).expect("lad");
        assert!(res.iso);
        assert!(res.map.is_empty());
        let maps = get_subisomorphisms_lad(&empty, &target, None, false).expect("lad");
        assert_eq!(maps, vec![Vec::<u32>::new()]);
    }

    #[test]
    fn pattern_larger_than_target() {
        let pattern = undirected(4, &[(0, 1), (1, 2), (2, 3)]);
        let target = undirected(3, &[(0, 1), (1, 2)]);
        let res = subisomorphic_lad(&pattern, &target, None, false).expect("lad");
        assert!(!res.iso);
        assert!(res.map.is_empty());
    }

    #[test]
    fn triangle_not_in_path() {
        let triangle = undirected(3, &[(0, 1), (1, 2), (2, 0)]);
        let path = undirected(3, &[(0, 1), (1, 2)]);
        let res = subisomorphic_lad(&triangle, &path, None, false).expect("lad");
        assert!(!res.iso);
    }

    #[test]
    fn edge_into_triangle_has_six_maps() {
        let target = undirected(3, &[(0, 1), (1, 2), (2, 0)]);
        let pattern = undirected(2, &[(0, 1)]);
        let maps = get_subisomorphisms_lad(&pattern, &target, None, false).expect("lad");
        assert_eq!(maps.len(), 6);
    }

    #[test]
    fn multi_edge_rejected() {
        let mut pattern = Graph::new(2, false).expect("graph init");
        pattern.add_edge(0, 1).expect("edge");
        pattern.add_edge(0, 1).expect("edge");
        let target = undirected(3, &[(0, 1), (1, 2), (2, 0)]);
        assert!(subisomorphic_lad(&pattern, &target, None, false).is_err());
    }

    #[test]
    fn directedness_mismatch_rejected() {
        let pattern = Graph::new(2, true).expect("graph init");
        let target = undirected(3, &[(0, 1), (1, 2)]);
        assert!(subisomorphic_lad(&pattern, &target, None, false).is_err());
    }
}

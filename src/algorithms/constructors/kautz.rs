//! Kautz graph constructor (ALGO-CN-013).
//!
//! Counterpart of `igraph_kautz()` in
//! `references/igraph/src/constructors/kautz.c:61-210`.
//!
//! The Kautz graph `K(m, n)` is the **directed** graph whose vertices are
//! length-`n+1` strings over an alphabet of `m+1` letters, with the
//! constraint that no two consecutive letters in the string may be
//! equal. There is an arc from `v = (a_0, …, a_n)` to
//! `w = (a_1, …, a_n, b)` for every alphabet letter `b ≠ a_n`. Each
//! vertex therefore has out-degree exactly `m` and in-degree exactly `m`;
//! the total arc count is `m · (m+1) · m^n`.
//!
//! Vertex count is `(m+1) · m^n`. Vertices are encoded as base-`(m+1)`
//! integers in `[0, (m+1)^(n+1))`, but only those whose digit sequence
//! avoids consecutive repeats are valid — we build two index tables
//! (`index1`, `index2`) to map between the sparse string codes and the
//! dense `[0, vcount)` vertex ids, just like the upstream C source.
//!
//! Degenerate forms (match upstream order in `kautz.c` lines 81-86):
//!
//! * `n == 0` → `(m+1)`-vertex directed complete graph with no
//!   self-loops (`K_{m+1}` directed). For `m == 0 ∧ n == 0` this
//!   collapses to a singleton. The C source delegates to
//!   `igraph_full`; we inline the emission to avoid pulling in a
//!   not-yet-ported full-graph helper.
//! * `m == 0 ∧ n ≥ 1` → empty 0-vertex directed graph (no valid string
//!   survives the consecutive-distinct constraint when only one symbol
//!   exists).
//!
//! Time complexity: `O(|V| + |E|)` — both the index-table construction
//! and the arc emission are linear in the output.

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Build the Kautz graph `K(m, n)`.
///
/// The result is **always directed** (matches `IGRAPH_DIRECTED` in the C
/// constructor). For `n ≥ 1` every vertex has out-degree and in-degree
/// equal to `m`.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — `(m+1)^(n+1)` overflows `u32`,
///   so the sparse string-code space cannot be addressed with the
///   graph's `u32` vertex id width. The check uses [`u32::checked_pow`]
///   so the failure mode is deterministic.
/// * [`IgraphError::InvalidArgument`] — vcount `(m+1) · m^n` or ecount
///   `m · vcount` overflows `usize` so the index tables / edge buffer
///   cannot be sized safely.
///
/// # Examples
///
/// ```
/// use rust_igraph::kautz;
///
/// // K(2, 1) — directed graph on 6 vertices; every vertex has
/// // out-degree 2 and in-degree 2 → 12 arcs total.
/// let g = kautz(2, 1).unwrap();
/// assert_eq!(g.vcount(), 6);
/// assert_eq!(g.ecount(), 12);
/// assert!(g.is_directed());
/// ```
pub fn kautz(m: u32, n: u32) -> IgraphResult<Graph> {
    // n == 0 → directed K_{m+1} with no loops. m == 0 ∧ n == 0
    // collapses to a singleton (inner loop emits no arcs).
    if n == 0 {
        return kautz_n_zero(m);
    }
    // m == 0 with n ≥ 1 → no valid strings (single-symbol alphabet
    // cannot satisfy the no-consecutive-equal constraint).
    if m == 0 {
        return Graph::new(0, true);
    }

    let dims = KautzDims::compute(m, n)?;
    let (index1, index2) = build_kautz_indices(m, &dims)?;
    let edges = emit_kautz_arcs(m, &dims, &index1, &index2);

    let mut g = Graph::new(dims.vcount, true)?;
    g.add_edges(edges)?;
    Ok(g)
}

/// `n == 0` specialisation: directed complete graph on `m+1` vertices
/// with no self-loops. The C source delegates to `igraph_full`; we
/// inline so this constructor stays standalone.
fn kautz_n_zero(m: u32) -> IgraphResult<Graph> {
    let m_plus_1 = m.checked_add(1).ok_or_else(|| {
        IgraphError::InvalidArgument(format!("kautz: m + 1 overflows u32 (m = {m})"))
    })?;
    let mp1_us = usize::try_from(m_plus_1).map_err(|_| {
        IgraphError::InvalidArgument(format!("kautz: m + 1 = {m_plus_1} cannot fit usize"))
    })?;
    let ec = mp1_us
        .checked_mul(mp1_us.saturating_sub(1))
        .ok_or_else(|| {
            IgraphError::InvalidArgument(format!("kautz: (m+1)·m overflows usize (m = {m})"))
        })?;
    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(ec);
    for i in 0..m_plus_1 {
        for j in 0..m_plus_1 {
            if i != j {
                edges.push((i, j));
            }
        }
    }
    let mut g = Graph::new(m_plus_1, true)?;
    g.add_edges(edges)?;
    Ok(g)
}

/// Computed dimensions of `K(m, n)` after every overflow gate has
/// passed. All `*_us` fields are pre-cast for indexing.
struct KautzDims {
    m_plus_1: u32,
    vcount: u32,
    allstrings: u32,
    vcount_us: usize,
    allstrings_us: usize,
    ecount: usize,
    n_us: usize,
    n_plus_1_us: usize,
}

impl KautzDims {
    fn compute(m: u32, n: u32) -> IgraphResult<Self> {
        let m_plus_1 = m.checked_add(1).ok_or_else(|| {
            IgraphError::InvalidArgument(format!("kautz: m + 1 overflows u32 (m = {m})"))
        })?;
        let n_plus_1 = n.checked_add(1).ok_or_else(|| {
            IgraphError::InvalidArgument(format!("kautz: n + 1 overflows u32 (n = {n})"))
        })?;
        let m_to_n = m.checked_pow(n).ok_or_else(|| {
            IgraphError::InvalidArgument(format!("kautz: m^n overflows u32 (m = {m}, n = {n})"))
        })?;
        let vcount = m_plus_1.checked_mul(m_to_n).ok_or_else(|| {
            IgraphError::InvalidArgument(format!(
                "kautz: (m+1)·m^n overflows u32 (m = {m}, n = {n})"
            ))
        })?;
        let allstrings = m_plus_1.checked_pow(n_plus_1).ok_or_else(|| {
            IgraphError::InvalidArgument(format!(
                "kautz: (m+1)^(n+1) overflows u32 (m = {m}, n = {n})"
            ))
        })?;
        let vcount_us = usize::try_from(vcount).map_err(|_| {
            IgraphError::InvalidArgument(format!("kautz: vcount {vcount} cannot fit usize"))
        })?;
        let allstrings_us = usize::try_from(allstrings).map_err(|_| {
            IgraphError::InvalidArgument(format!("kautz: allstrings {allstrings} cannot fit usize"))
        })?;
        let m_us = usize::try_from(m)
            .map_err(|_| IgraphError::InvalidArgument(format!("kautz: m {m} cannot fit usize")))?;
        let n_us = usize::try_from(n)
            .map_err(|_| IgraphError::InvalidArgument(format!("kautz: n {n} cannot fit usize")))?;
        let n_plus_1_us = usize::try_from(n_plus_1).map_err(|_| {
            IgraphError::InvalidArgument(format!("kautz: n + 1 = {n_plus_1} cannot fit usize"))
        })?;
        let ecount = vcount_us.checked_mul(m_us).ok_or_else(|| {
            IgraphError::InvalidArgument(format!(
                "kautz: m·vcount overflows usize (m = {m}, n = {n})"
            ))
        })?;
        Ok(Self {
            m_plus_1,
            vcount,
            allstrings,
            vcount_us,
            allstrings_us,
            ecount,
            n_us,
            n_plus_1_us,
        })
    }
}

/// Walks all length-`n+1` strings over `{0..=m}` with no two
/// consecutive equal letters in lex order, building both the sparse
/// code→1-based-id map (`index1`) and the dense id→code map (`index2`).
fn build_kautz_indices(m: u32, dims: &KautzDims) -> IgraphResult<(Vec<u32>, Vec<u32>)> {
    // Positional weights: table[i] = (m+1)^(n-i). Each entry
    // ≤ allstrings/m_plus_1 < u32::MAX by the precheck.
    let mut table: Vec<u32> = vec![0; dims.n_plus_1_us];
    let mut weight: u64 = 1;
    for i in (0..dims.n_plus_1_us).rev() {
        table[i] = u32::try_from(weight).map_err(|_| {
            IgraphError::InvalidArgument(format!(
                "kautz: table[{i}] overflows u32 (internal invariant)"
            ))
        })?;
        weight = weight.saturating_mul(u64::from(dims.m_plus_1));
    }

    let mut index1: Vec<u32> = vec![0; dims.allstrings_us];
    let mut index2: Vec<u32> = vec![0; dims.vcount_us];
    let mut digits: Vec<u32> = vec![0; dims.n_plus_1_us];

    let mut actb: usize = 0;
    let mut actvalue: u32 = 0;
    let mut idx: usize = 0;

    loop {
        // Complete digits[0..=actb] into a valid string by filling
        // digits[actb+1..=n] with alternating 0/1 starting from
        // (digits[actb] == 0 ? 1 : 0).
        let mut z: u32 = u32::from(digits[actb] == 0);
        for k in (actb + 1)..=dims.n_us {
            digits[k] = z;
            actvalue += z * table[k];
            z = 1 - z;
        }
        actb = dims.n_us;

        index1[actvalue as usize] = u32::try_from(idx + 1).map_err(|_| {
            IgraphError::InvalidArgument(
                "kautz: idx + 1 overflows u32 (internal invariant)".to_string(),
            )
        })?;
        index2[idx] = actvalue;
        idx += 1;
        if idx >= dims.vcount_us {
            break;
        }

        // Advance to the next valid prefix.
        let advanced = advance_kautz_cursor(m, &table, &mut digits, &mut actb, &mut actvalue);
        if !advanced {
            // Defensive: unreachable when vcount is correct (there
            // are exactly vcount valid strings).
            break;
        }
    }

    Ok((index1, index2))
}

/// Tries to bump the cursor to the next valid prefix. Returns `false`
/// only when no further valid prefix exists (an internal invariant
/// violation; never happens for well-formed `(m, n)`).
fn advance_kautz_cursor(
    m: u32,
    table: &[u32],
    digits: &mut [u32],
    actb: &mut usize,
    actvalue: &mut u32,
) -> bool {
    loop {
        let mut next = digits[*actb] + 1;
        if *actb != 0 && digits[*actb - 1] == next {
            next += 1;
        }
        if next <= m {
            *actvalue += (next - digits[*actb]) * table[*actb];
            digits[*actb] = next;
            return true;
        }
        *actvalue -= digits[*actb] * table[*actb];
        if *actb == 0 {
            return false;
        }
        *actb -= 1;
    }
}

/// Source-major, digit-ascending arc emission. For every vertex `i`
/// with sparse code `fromvalue`, the `m` successors live at
/// `basis + digit` for `digit ∈ [0, m]` with `digit != lastdigit`.
fn emit_kautz_arcs(
    m: u32,
    dims: &KautzDims,
    index1: &[u32],
    index2: &[u32],
) -> Vec<(VertexId, VertexId)> {
    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(dims.ecount);
    let m_plus_1_u64 = u64::from(dims.m_plus_1);
    let allstrings_u64 = u64::from(dims.allstrings);
    for i in 0..dims.vcount {
        let fromvalue = index2[i as usize];
        let lastdigit = fromvalue % dims.m_plus_1;
        // fromvalue · (m+1) may exceed u32, but its reduction mod
        // allstrings lives in [0, allstrings) ≤ u32::MAX.
        let basis_u64 = (u64::from(fromvalue) * m_plus_1_u64) % allstrings_u64;
        #[allow(clippy::cast_possible_truncation)]
        let basis = basis_u64 as u32;
        for digit in 0..=m {
            if digit == lastdigit {
                continue;
            }
            let tovalue = basis + digit;
            let sentinel = index1[tovalue as usize];
            if sentinel == 0 {
                continue;
            }
            edges.push((i, sentinel - 1));
        }
    }
    edges
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn dump_arcs(g: &Graph) -> Vec<(VertexId, VertexId)> {
        let ec = u32::try_from(g.ecount()).expect("ecount fits u32 in tests");
        let mut out = Vec::with_capacity(g.ecount());
        for e in 0..ec {
            let u = g.edge_source(e).expect("edge in range");
            let v = g.edge_target(e).expect("edge in range");
            out.push((u, v));
        }
        out
    }

    #[test]
    fn n_zero_m_zero_singleton() {
        // K(0, 0) — single vertex, no arcs (one length-1 string).
        let g = kautz(0, 0).expect("K(0,0)");
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
        assert!(g.is_directed());
    }

    #[test]
    fn n_zero_m_five_directed_k6() {
        // K(5, 0) — directed K_6 with no loops, 30 arcs.
        let g = kautz(5, 0).expect("K(5,0)");
        assert_eq!(g.vcount(), 6);
        assert_eq!(g.ecount(), 30);
        let arcs: BTreeSet<_> = dump_arcs(&g).into_iter().collect();
        let mut expected = BTreeSet::new();
        for i in 0u32..6 {
            for j in 0u32..6 {
                if i != j {
                    expected.insert((i, j));
                }
            }
        }
        assert_eq!(arcs, expected);
    }

    #[test]
    fn m_zero_n_ten_empty() {
        let g = kautz(0, 10).expect("K(0,10)");
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
        assert!(g.is_directed());
    }

    #[test]
    fn k_2_1_counts() {
        let g = kautz(2, 1).expect("K(2,1)");
        assert_eq!(g.vcount(), 6);
        assert_eq!(g.ecount(), 12);
        assert!(g.is_directed());
    }

    #[test]
    fn k_2_1_in_out_degrees() {
        // K(2, 1): out-degree = in-degree = m = 2 for every vertex.
        let g = kautz(2, 1).expect("K(2,1)");
        let mut out_deg = [0u32; 6];
        let mut in_deg = [0u32; 6];
        for (u, v) in dump_arcs(&g) {
            out_deg[u as usize] += 1;
            in_deg[v as usize] += 1;
        }
        for (v, (&o, &i)) in out_deg.iter().zip(in_deg.iter()).enumerate() {
            assert_eq!(o, 2, "out-degree at {v}");
            assert_eq!(i, 2, "in-degree at {v}");
        }
    }

    #[test]
    fn k_3_2_counts() {
        // K(3, 2): (m+1)·m^n = 4·9 = 36 vertices, m·vcount = 108 arcs.
        let g = kautz(3, 2).expect("K(3,2)");
        assert_eq!(g.vcount(), 36);
        assert_eq!(g.ecount(), 108);
    }

    #[test]
    fn k_2_2_counts() {
        // K(2, 2): 3·4 = 12 vertices, 2·12 = 24 arcs.
        let g = kautz(2, 2).expect("K(2,2)");
        assert_eq!(g.vcount(), 12);
        assert_eq!(g.ecount(), 24);
    }

    #[test]
    fn vcount_overflow_rejected() {
        // (m+1)^(n+1) for m=2, n=31 is 3^32 which overflows u32.
        let err = kautz(2, 31).expect_err("overflow expected");
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptest_invariants {
    use super::*;
    use proptest::prelude::*;
    use std::collections::BTreeSet;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]

        /// `vcount == (m+1)·m^n` and `ecount == m·vcount` across the
        /// supported `(m, n)` grid that stays well clear of u32 overflow.
        #[test]
        fn vcount_and_ecount_are_exact(m in 1u32..=5, n in 1u32..=4) {
            let Some(m_to_n) = m.checked_pow(n) else { return Ok(()); };
            let Some(vcount) = (m + 1).checked_mul(m_to_n) else { return Ok(()); };
            let Some(ecount) = vcount.checked_mul(m) else { return Ok(()); };
            let g = kautz(m, n).expect("valid input");
            prop_assert_eq!(g.vcount(), vcount);
            prop_assert_eq!(g.ecount() as u64, u64::from(ecount));
            prop_assert!(g.is_directed());
        }

        /// Every vertex has out-degree `m` and in-degree `m`.
        #[test]
        fn vertex_in_and_out_degree_equals_m(m in 1u32..=4, n in 1u32..=3) {
            let g = kautz(m, n).expect("valid input");
            let vc = usize::try_from(g.vcount()).unwrap();
            let mut out_deg = vec![0u32; vc];
            let mut in_deg = vec![0u32; vc];
            let ec = u32::try_from(g.ecount()).unwrap();
            for e in 0..ec {
                let u = g.edge_source(e).expect("edge in range");
                let v = g.edge_target(e).expect("edge in range");
                out_deg[u as usize] += 1;
                in_deg[v as usize] += 1;
            }
            for v in 0..vc {
                prop_assert_eq!(out_deg[v], m);
                prop_assert_eq!(in_deg[v], m);
            }
        }

        /// Kautz graphs are loopless and simple: no `(v, v)` arcs and no
        /// repeated `(u, v)` pairs.
        #[test]
        fn loopless_and_simple(m in 1u32..=4, n in 1u32..=3) {
            let g = kautz(m, n).expect("valid input");
            let ec = u32::try_from(g.ecount()).unwrap();
            let mut seen: BTreeSet<(VertexId, VertexId)> = BTreeSet::new();
            for e in 0..ec {
                let u = g.edge_source(e).expect("edge in range");
                let v = g.edge_target(e).expect("edge in range");
                prop_assert_ne!(u, v, "Kautz graph must not contain self-loops");
                prop_assert!(seen.insert((u, v)), "duplicate arc ({u}, {v})");
            }
        }
    }
}

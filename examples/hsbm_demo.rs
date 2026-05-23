//! ALGO-GN-011 example: hierarchical stochastic block model.
//!
//! Generates a three-level hierarchy:
//!
//! ```text
//!   macro 0 ┐         ┐
//!   macro 1 ┤  inter  │
//!   macro 2 ┤   p     │  n = 120 vertices,
//!   macro 3 ┘         │  4 macros × m = 30,
//!                     │  per macro: 3 equal micro-clusters of 10,
//!   per macro: dense ─┘  intra-cluster p_in = 0.40,
//!   inside each cluster  inter-cluster p_off = 0.05.
//! ```
//!
//! We then visualise the planted structure from edge counts alone:
//!
//!   * the 4 × 4 macro-block edge matrix (diagonal = intra-macro),
//!   * within each macro: the 3 × 3 micro-cluster edge matrix,
//!   * mean degree per macro and per micro-cluster,
//!   * the same observable shape replayed through `hsbm_list_game` (per
//!     macro `m_list`, `rho_list`, `c_list`) — confirms the uniform and
//!     list APIs are interchangeable when the per-macro parameters are
//!     copy-pasted.
//!
//! Run: `cargo run --example hsbm_demo`.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    clippy::similar_names,
    clippy::needless_range_loop
)]

use rust_igraph::{Graph, hsbm_game, hsbm_list_game};

const MACROS: u32 = 4;
const M: u32 = 30;
const MICRO_PER_MACRO: usize = 3;
const MICRO_SIZE: u32 = M / MICRO_PER_MACRO as u32;
const N: u32 = MACROS * M;
const P_IN: f64 = 0.40;
const P_OFF: f64 = 0.05;
const P_INTER: f64 = 0.01;

fn macro_of(v: u32) -> u32 {
    v / M
}

fn micro_of(v: u32) -> usize {
    ((v % M) / MICRO_SIZE) as usize
}

fn degrees(g: &Graph) -> Vec<u32> {
    let n = g.vcount();
    let mut d = vec![0u32; n as usize];
    let m = u32::try_from(g.ecount()).expect("ecount fits in u32 for example");
    for eid in 0..m {
        let (u, v) = g.edge(eid).expect("edge id in bounds for example");
        d[u as usize] += 1;
        if u != v {
            d[v as usize] += 1;
        }
    }
    d
}

fn summarise(label: &str, g: &Graph) {
    println!("--- {label} ---");
    println!(
        "vcount = {}, ecount = {}, directed = {}",
        g.vcount(),
        g.ecount(),
        g.is_directed()
    );

    // Macro-level edge matrix.
    let k_macro = MACROS as usize;
    let mut macro_edges = vec![vec![0u64; k_macro]; k_macro];
    let m = u32::try_from(g.ecount()).expect("ecount fits in u32 for example");
    for eid in 0..m {
        let (u, v) = g.edge(eid).expect("edge id in bounds for example");
        let (bu, bv) = (macro_of(u) as usize, macro_of(v) as usize);
        let (lo, hi) = if bu <= bv { (bu, bv) } else { (bv, bu) };
        macro_edges[lo][hi] += 1;
    }
    let total_inter: u64 = (0..k_macro)
        .flat_map(|i| (i + 1..k_macro).map(move |j| (i, j)))
        .map(|(i, j)| macro_edges[i][j])
        .sum();
    let total_intra: u64 = (0..k_macro).map(|i| macro_edges[i][i]).sum();
    println!("  intra-macro edges = {total_intra}, inter-macro edges = {total_inter}");
    println!("  macro × macro edge matrix (diagonal = intra-macro):");
    print!("        ");
    for j in 0..k_macro {
        print!("   M{j:<2}");
    }
    println!();
    for (i, row) in macro_edges.iter().enumerate() {
        print!("    M{i:<2} ");
        for (j, &cell) in row.iter().enumerate() {
            let val = if i <= j { cell } else { macro_edges[j][i] };
            print!(" {val:>5}");
        }
        println!();
    }

    // Per-macro: micro-cluster edge matrix.
    println!("  per-macro micro-cluster edge matrices ({MICRO_PER_MACRO}×{MICRO_PER_MACRO}):");
    let mut micro_edges = vec![vec![vec![0u64; MICRO_PER_MACRO]; MICRO_PER_MACRO]; k_macro];
    for eid in 0..m {
        let (u, v) = g.edge(eid).expect("edge id in bounds for example");
        if macro_of(u) == macro_of(v) {
            let b = macro_of(u) as usize;
            let (cu, cv) = (micro_of(u), micro_of(v));
            let (lo, hi) = if cu <= cv { (cu, cv) } else { (cv, cu) };
            micro_edges[b][lo][hi] += 1;
        }
    }
    for (b, mat) in micro_edges.iter().enumerate() {
        print!("    macro {b}:");
        for i in 0..MICRO_PER_MACRO {
            for j in 0..MICRO_PER_MACRO {
                let val = if i <= j { mat[i][j] } else { mat[j][i] };
                print!(" {val:>3}");
                if j == MICRO_PER_MACRO - 1 {
                    print!("  ");
                }
            }
        }
        println!();
    }

    // Mean degree per macro.
    let deg = degrees(g);
    println!("  mean degree per macro:");
    for b in 0..k_macro {
        let lo = (b as u32 * M) as usize;
        let hi = ((b as u32 + 1) * M) as usize;
        let sum: u32 = deg[lo..hi].iter().sum();
        let mean = f64::from(sum) / f64::from(M);
        println!("    macro {b}: mean deg = {mean:>5.2}");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "hsbm_demo: n = {N}, macros = {MACROS}, m = {M}, micro = {MICRO_PER_MACRO} × {MICRO_SIZE}"
    );
    println!("  p_in = {P_IN}, p_off = {P_OFF}, p_inter = {P_INTER}");

    // ---- uniform variant ----
    let rho = vec![1.0_f64 / 3.0; MICRO_PER_MACRO];
    let c_mat = (0..MICRO_PER_MACRO)
        .map(|i| {
            (0..MICRO_PER_MACRO)
                .map(|j| if i == j { P_IN } else { P_OFF })
                .collect()
        })
        .collect::<Vec<Vec<f64>>>();
    let g_uniform = hsbm_game(N, M, &rho, &c_mat, P_INTER, 0x1130_2026)?;
    summarise("hsbm_game (uniform)", &g_uniform);

    // ---- list variant: replay the same shape per-macro ----
    let m_list = vec![M; MACROS as usize];
    let rho_list = vec![rho.clone(); MACROS as usize];
    let c_list = vec![c_mat.clone(); MACROS as usize];
    let g_list = hsbm_list_game(N, &m_list, &rho_list, &c_list, P_INTER, 0x1130_2027)?;
    summarise("hsbm_list_game (replicated)", &g_list);

    // Different seeds → different realised edges, but same parameters
    // ⇒ similar macro/micro statistics. The intra/inter split is the
    // signature of the planted hierarchy.
    println!("Both runs use the same parameters with different seeds — the");
    println!("intra-macro vs inter-macro edge counts (and the micro-cluster");
    println!("diagonals) show the planted three-level community structure.");

    Ok(())
}

//! ALGO-GN-013 example: static-fitness + static power-law generators.
//!
//! Two paired runs over the same `n = 200` vertex universe demonstrate
//! the two `static_*_game` constructors:
//!
//! ```text
//!   1. static_fitness_game     — explicit fitness vector w_i = 1 + 0.4·i
//!                                with E[d_i] ∝ w_i in the sparse limit.
//!   2. static_power_law_game   — synthesises w_i ∝ (i + i0)^(-1/(γ-1))
//!                                from a Pareto exponent γ; we sweep
//!                                γ ∈ {2.1, 2.5, 3.0} and contrast the
//!                                resulting hub-skew.
//! ```
//!
//! For each realisation we print:
//!
//!   * vcount / ecount / mean degree;
//!   * Pearson correlation between the *implied* fitness `f_i` (the
//!     explicit weights, or the internally-generated power-law fitness)
//!     and the *observed* degree `d_i` — the static-fitness contract is
//!     `E[d_i] ∝ f_i` so this should sit close to 1;
//!   * the top-5 hub vertices ranked by degree.
//!
//! We close with one directed run (`fitness_in ≠ fitness_out`) to show
//! that the two endpoints sample independently from their own fitness
//! distributions.
//!
//! Run: `cargo run --example static_fitness_demo`.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    clippy::unreadable_literal,
    clippy::unusual_byte_groupings
)]

use rust_igraph::{Graph, static_fitness_game, static_power_law_game};

const N: u32 = 200;
const M: u32 = 800;

fn linear_fitness() -> Vec<f64> {
    (0..N).map(|i| 1.0 + 0.4 * f64::from(i)).collect()
}

fn undirected_degrees(g: &Graph) -> Vec<u32> {
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

fn directed_degrees(g: &Graph) -> (Vec<u32>, Vec<u32>) {
    let n = g.vcount();
    let mut out = vec![0u32; n as usize];
    let mut din = vec![0u32; n as usize];
    let m = u32::try_from(g.ecount()).expect("ecount fits in u32 for example");
    for eid in 0..m {
        let (u, v) = g.edge(eid).expect("edge id in bounds for example");
        out[u as usize] += 1;
        din[v as usize] += 1;
    }
    (out, din)
}

fn pearson(x: &[f64], y: &[f64]) -> f64 {
    assert_eq!(x.len(), y.len());
    let n = x.len() as f64;
    let mx: f64 = x.iter().sum::<f64>() / n;
    let my: f64 = y.iter().sum::<f64>() / n;
    let mut num = 0.0;
    let mut sxx = 0.0;
    let mut syy = 0.0;
    for (&xi, &yi) in x.iter().zip(y.iter()) {
        let dx = xi - mx;
        let dy = yi - my;
        num += dx * dy;
        sxx += dx * dx;
        syy += dy * dy;
    }
    let denom = (sxx * syy).sqrt();
    if denom == 0.0 { 0.0 } else { num / denom }
}

fn summarise_undirected(label: &str, g: &Graph, fitness: &[f64]) {
    let deg = undirected_degrees(g);
    let deg_f: Vec<f64> = deg.iter().map(|&d| f64::from(d)).collect();
    let mean: f64 = deg_f.iter().sum::<f64>() / (deg.len() as f64);
    let corr = pearson(fitness, &deg_f);

    println!("--- {label} ---");
    println!(
        "  vcount = {}, ecount = {}, mean degree = {:.3}",
        g.vcount(),
        g.ecount(),
        mean
    );
    println!(
        "  Pearson(f_i, deg_i) = {corr:.4}   (closer to 1.0 → stronger fitness-degree alignment)"
    );

    let mut ranked: Vec<(u32, u32, f64)> = deg
        .iter()
        .enumerate()
        .map(|(i, &d)| (i as u32, d, fitness[i]))
        .collect();
    ranked.sort_by_key(|tup| std::cmp::Reverse(tup.1));
    println!("  top-5 hubs (vertex, observed_deg, fitness):");
    for &(v, d, f) in ranked.iter().take(5) {
        println!("     v={v:>3}  deg={d:>4}  f={f:>7.3}");
    }
}

fn summarise_directed(label: &str, g: &Graph, fout: &[f64], fin: &[f64]) {
    let (out, din) = directed_degrees(g);
    let out_f: Vec<f64> = out.iter().map(|&d| f64::from(d)).collect();
    let in_f: Vec<f64> = din.iter().map(|&d| f64::from(d)).collect();
    let mean_out: f64 = out_f.iter().sum::<f64>() / (out.len() as f64);
    let corr_out = pearson(fout, &out_f);
    let corr_in = pearson(fin, &in_f);

    println!("--- {label} ---");
    println!(
        "  vcount = {}, ecount = {}, mean out-degree = {:.3} (= mean in-degree)",
        g.vcount(),
        g.ecount(),
        mean_out
    );
    println!("  Pearson(f_out, out_deg_i) = {corr_out:.4}");
    println!("  Pearson(f_in , in_deg_i ) = {corr_in:.4}");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("static_fitness_demo: n = {N}, m = {M}");
    println!();

    // Part 1 — explicit fitness vector.
    let w = linear_fitness();
    println!(
        "## static_fitness_game — w_i = 1 + 0.4·i, range [{:.2}, {:.2}], Σw = {:.1}",
        w[0],
        w[(N - 1) as usize],
        w.iter().sum::<f64>()
    );
    let g = static_fitness_game(M, &w, None, false, false, 0x5F11_D000_01)?;
    summarise_undirected("undirected, simple (no loops, no multi)", &g, &w);
    println!();

    // Part 2 — power-law sweep. The internal fitness is f_i ∝
    // (i + i0)^(-1/(γ-1)), which we mirror here so the Pearson
    // correlation is meaningful.
    println!("## static_power_law_game — γ sweep (heavier tail at lower γ)");
    for (gamma, seed) in [
        (2.1_f64, 0x5F11_D000_10u64),
        (2.5_f64, 0x5F11_D000_11u64),
        (3.0_f64, 0x5F11_D000_12u64),
    ] {
        let g = static_power_law_game(N, M, gamma, None, false, false, true, seed)?;
        let alpha = -1.0 / (gamma - 1.0);
        let n_usize = N as usize;
        // Mirror upstream: fitness[i] = (n - i)^α, so the hub is at
        // index n-1 (since α < 0, the smallest base ⇒ largest weight).
        let f: Vec<f64> = (0..n_usize)
            .map(|i| ((n_usize - i) as f64).powf(alpha))
            .collect();
        summarise_undirected(&format!("γ = {gamma:.1}"), &g, &f);
        println!();
    }

    // Part 3 — directed run with asymmetric fitness vectors. Hubs
    // swap sides between in- and out-direction by reversing the
    // fitness vector.
    let w_out = w.clone();
    let mut w_in = w.clone();
    w_in.reverse();
    let g_dir = static_fitness_game(M, &w_out, Some(&w_in), false, false, 0x5F11_D000_20)?;
    // For the asymmetric directed case we expect the in-degrees to
    // correlate with the *reversed* fitness vector and out-degrees with
    // the original — print both Pearson scores against the matching
    // axis.
    summarise_directed(
        "directed, simple — fitness_out increases, fitness_in decreases",
        &g_dir,
        &w_out,
        &w_in,
    );
    println!();

    println!("Take-aways:");
    println!("  * Explicit-fitness Pearson sits close to 1 — degree tracks w_i.");
    println!("  * Power-law sweep: lower γ → wider hub spread → bigger top-5 deg.");
    println!("  * Directed asymmetric fitness: out-degrees align with f_out,");
    println!("    in-degrees with f_in, independently.");
    Ok(())
}

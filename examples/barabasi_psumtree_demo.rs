//! ALGO-GN-020 example: Barabási–Albert PSUMTREE diagnostics.
//!
//! Grows a directed BA graph with the Fenwick-BIT preferential-attachment
//! sampler (`barabasi_game_psumtree`) and prints summary statistics that
//! confirm the model's defining property — a heavy-tailed in-degree
//! distribution — for two contrasting kernel exponents:
//!
//!   * `power = 1.0` — classical linear preferential attachment, the
//!     original Barabási–Albert model (Science, 1999). Yields a
//!     power-law in-degree distribution with exponent ≈ 3.
//!   * `power = 0.5` — sub-linear preferential attachment. Theory
//!     (Krapivsky-Redner-Leyvraz, PRL 2000) predicts a stretched
//!     exponential tail rather than a true power law — fewer extreme
//!     hubs.
//!
//! For each run we print mean / max in-degree and a log-binned histogram
//! that makes the tail visible at a glance.
//!
//! Run: `cargo run --example barabasi_psumtree_demo`.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use rust_igraph::{Graph, barabasi_game_psumtree};

fn in_degrees(g: &Graph) -> Vec<u32> {
    let n = g.vcount();
    let mut ind = vec![0u32; n as usize];
    let m = u32::try_from(g.ecount()).expect("ecount fits in u32 for example");
    for eid in 0..m {
        let (_src, dst) = g.edge(eid).expect("edge id in bounds for example");
        ind[dst as usize] += 1;
    }
    ind
}

fn mean(xs: &[u32]) -> f64 {
    let sum: u64 = xs.iter().map(|&x| u64::from(x)).sum();
    sum as f64 / xs.len() as f64
}

fn report(label: &str, g: &Graph) {
    let ind = in_degrees(g);
    let max_in = *ind.iter().max().unwrap_or(&0);
    println!("{label}");
    println!("  vertices     = {}", g.vcount());
    println!("  edges        = {}", g.ecount());
    println!("  mean in-deg  = {:.3}", mean(&ind));
    println!("  max  in-deg  = {max_in}");

    let mut idx: Vec<u32> = (0..g.vcount()).collect();
    idx.sort_by_key(|&i| std::cmp::Reverse(ind[i as usize]));
    println!("  top-5 in-degree hubs:");
    for &i in idx.iter().take(5) {
        println!("    vertex {i:>5}  in-degree = {}", ind[i as usize]);
    }

    let n_bins = if max_in == 0 {
        1
    } else {
        f64::from(max_in).log2().ceil() as usize + 1
    };
    let mut bins = vec![0u32; n_bins.max(1)];
    for &d in &ind {
        let b = if d == 0 {
            0
        } else {
            f64::from(d).log2() as usize
        };
        bins[b] += 1;
    }
    println!("  in-degree histogram (powers of 2):");
    for (i, &count) in bins.iter().enumerate() {
        let lo = if i == 0 { 0 } else { 1u32 << i };
        let hi = (1u32 << (i + 1)) - 1;
        println!("    [{lo:>4}, {hi:>4}]  count = {count}");
    }
    println!();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let n: u32 = 2_000;
    let m: u32 = 2;
    let a = 1.0;
    let seed = 0xBA_5E_BA_5E_u64;

    // Classical linear BA. Expected: heavy power-law tail.
    let g_linear = barabasi_game_psumtree(n, 1.0, m, None, false, a, true, seed)?;
    report(
        "barabasi_game_psumtree(power=1.0)  — classical preferential attachment",
        &g_linear,
    );

    // Sub-linear BA. Expected: stretched-exponential tail, fewer
    // extreme hubs than the linear case.
    let g_sub = barabasi_game_psumtree(n, 0.5, m, None, false, a, true, seed)?;
    report(
        "barabasi_game_psumtree(power=0.5)  — sub-linear preferential attachment",
        &g_sub,
    );

    Ok(())
}

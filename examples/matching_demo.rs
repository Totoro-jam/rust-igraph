//! ALGO-CL-003 example: bipartite maximum matching demo.
//!
//! Run: `cargo run --example matching_demo`.

use rust_igraph::{
    create, is_matching, is_maximal_matching, maximum_bipartite_matching,
    maximum_bipartite_matching_weighted,
};

fn main() {
    println!("=== Unweighted K2,2 ===");
    let g = create(&[(0, 2), (0, 3), (1, 2), (1, 3)], 4, false).expect("ok");
    let types = vec![false, false, true, true];
    let r = maximum_bipartite_matching(&g, &types).expect("ok");
    println!("  Size: {}", r.matching_size);
    println!("  Matching: {:?}", r.matching);
    println!(
        "  Valid: {}",
        is_matching(&g, Some(&types), &r.matching).expect("ok")
    );
    println!(
        "  Maximal: {}",
        is_maximal_matching(&g, Some(&types), &r.matching).expect("ok")
    );

    println!("\n=== Unweighted K3,3 ===");
    let g = create(
        &[
            (0, 3),
            (0, 4),
            (0, 5),
            (1, 3),
            (1, 4),
            (1, 5),
            (2, 3),
            (2, 4),
            (2, 5),
        ],
        6,
        false,
    )
    .expect("ok");
    let types = vec![false, false, false, true, true, true];
    let r = maximum_bipartite_matching(&g, &types).expect("ok");
    println!("  Size: {}", r.matching_size);
    println!("  Matching: {:?}", r.matching);

    println!("\n=== Weighted matching (MIT lecture notes) ===");
    let g = create(
        &[
            (0, 6),
            (0, 7),
            (0, 8),
            (0, 9),
            (1, 5),
            (1, 6),
            (1, 7),
            (1, 8),
            (1, 9),
            (2, 5),
            (2, 6),
            (2, 7),
            (2, 8),
            (2, 9),
            (3, 5),
            (3, 7),
            (3, 9),
            (4, 7),
        ],
        10,
        false,
    )
    .expect("ok");
    let types: Vec<bool> = (0..10).map(|i| i >= 5).collect();
    let weights = vec![
        2.0, 7.0, 2.0, 3.0, 1.0, 3.0, 9.0, 3.0, 3.0, 1.0, 3.0, 3.0, 1.0, 2.0, 4.0, 1.0, 2.0, 3.0,
    ];
    let r = maximum_bipartite_matching_weighted(&g, &types, &weights, 0.0).expect("ok");
    println!("  Size: {}", r.matching_size);
    println!("  Weight: {}", r.matching_weight);
    println!("  Matching: {:?}", r.matching);

    println!("\n=== Star graph (one center) ===");
    let g = create(&[(0, 1), (0, 2), (0, 3), (0, 4)], 5, false).expect("ok");
    let types = vec![false, true, true, true, true];
    let r = maximum_bipartite_matching(&g, &types).expect("ok");
    println!("  Size: {} (expected 1)", r.matching_size);
}

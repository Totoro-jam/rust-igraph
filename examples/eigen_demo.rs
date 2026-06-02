use rust_igraph::{EigenWhich, Graph, eigen_adjacency, eigen_matrix_symmetric};

fn main() {
    // 1. Graph eigenvalues: the adjacency spectrum of a cycle C5.
    //    Exact eigenvalues: 2, (−1+√5)/2, (−1+√5)/2, (−1−√5)/2, (−1−√5)/2
    //    ≈ 2.0, 0.618, 0.618, −1.618, −1.618
    let mut g = Graph::with_vertices(5);
    for i in 0..5 {
        g.add_edge(i, (i + 1) % 5).unwrap();
    }

    let result = eigen_adjacency(&g, 2, EigenWhich::LargestAlgebraic).unwrap();
    println!("C5 top-2 eigenvalues:");
    for (i, val) in result.eigenvalues.iter().enumerate() {
        println!("  λ{} = {:.6}", i, val);
    }

    // 2. Custom symmetric matrix: 4×4 diagonal.
    let diag = [10.0, 7.0, 3.0, 1.0];
    let result = eigen_matrix_symmetric(
        4,
        |x, y| {
            for i in 0..4 {
                y[i] = diag[i] * x[i];
            }
        },
        3,
        EigenWhich::LargestAlgebraic,
    )
    .unwrap();

    println!("\ndiag(10, 7, 3, 1) top-3 eigenvalues:");
    for (i, val) in result.eigenvalues.iter().enumerate() {
        println!("  λ{} = {:.6}", i, val);
    }

    // 3. Smallest eigenvalue of the complete graph K4.
    //    Adjacency spectrum of K4: 3, −1, −1, −1.
    let mut k4 = Graph::with_vertices(4);
    for i in 0..4 {
        for j in (i + 1)..4 {
            k4.add_edge(i, j).unwrap();
        }
    }

    let result = eigen_adjacency(&k4, 1, EigenWhich::SmallestAlgebraic).unwrap();
    println!("\nK4 smallest eigenvalue: {:.6}", result.eigenvalues[0]);
}

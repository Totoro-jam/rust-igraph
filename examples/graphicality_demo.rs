use rust_igraph::{EdgeTypeFilter, is_bigraphical, is_graphical};

fn main() {
    println!("=== is_graphical demo ===\n");

    // Undirected simple
    let star = [4, 1, 1, 1, 1];
    println!(
        "Star K_{{1,4}} {:?} simple? {}",
        star,
        is_graphical(&star, None, EdgeTypeFilter::Simple)
    );

    let k4 = [3, 3, 3, 3];
    println!(
        "Complete K_4  {:?} simple? {}",
        k4,
        is_graphical(&k4, None, EdgeTypeFilter::Simple)
    );

    let odd = [3, 1, 1];
    println!(
        "Odd sum       {:?} simple? {}",
        odd,
        is_graphical(&odd, None, EdgeTypeFilter::Simple)
    );

    // Undirected with multi-edges
    let multi = [5, 1, 0];
    println!(
        "\nMulti-edge    {:?} multi?  {}",
        multi,
        is_graphical(&multi, None, EdgeTypeFilter::Multi)
    );
    println!(
        "Multi-edge    {:?} loops+multi? {}",
        multi,
        is_graphical(&multi, None, EdgeTypeFilter::LoopsMulti)
    );

    // Directed simple
    let out_deg = [3, 2, 1, 0];
    let in_deg = [0, 1, 2, 3];
    println!(
        "\nTournament out={:?} in={:?} simple? {}",
        out_deg,
        in_deg,
        is_graphical(&out_deg, Some(&in_deg), EdgeTypeFilter::Simple)
    );

    // Bigraphical
    let d1 = [3, 3];
    let d2 = [2, 2, 2];
    println!(
        "\nBipartite K_{{2,3}} {:?} {:?} simple? {}",
        d1,
        d2,
        is_bigraphical(&d1, &d2, EdgeTypeFilter::Simple)
    );

    let d1b = [3, 3];
    let d2b = [1, 1, 1];
    println!(
        "Bipartite       {:?} {:?} simple? {}",
        d1b,
        d2b,
        is_bigraphical(&d1b, &d2b, EdgeTypeFilter::Simple)
    );
}

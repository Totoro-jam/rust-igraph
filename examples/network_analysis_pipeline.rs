//! Complete network analysis pipeline.
//!
//! Demonstrates a realistic workflow: generate a network, compute structural
//! properties, find communities, identify key vertices, and export results.
//!
//! Run: `cargo run --example network_analysis_pipeline`

use rust_igraph::{
    AttributeValue, ConnectednessMode, Graph, GraphBuilder, betweenness, connected_components,
    density, is_connected, louvain, pagerank,
};

fn build_network() -> Result<Graph, Box<dyn std::error::Error>> {
    let g = GraphBuilder::undirected()
        .vertices(15)
        .clique(&[0, 1, 2, 3, 4])
        .clique(&[5, 6, 7, 8, 9])
        .clique(&[10, 11, 12, 13, 14])
        .edge(2, 5)
        .edge(4, 10)
        .edge(8, 12)
        .build()?;
    println!("Step 1: Network constructed");
    println!("  {g}\n");
    Ok(g)
}

fn analyze_structure(g: &Graph) -> Result<(), Box<dyn std::error::Error>> {
    let connected = is_connected(g, ConnectednessMode::Weak)?;
    let dens = density(g)?.unwrap_or(0.0);
    let components = connected_components(g)?;
    let diameter = g.diameter()?;
    let girth = g.girth()?;
    let triangles = g.count_triangles()?;
    let apl = g.average_path_length()?.unwrap_or(0.0);

    println!("Step 2: Structural properties");
    println!("  Connected: {connected}");
    println!("  Components: {}", components.count);
    println!("  Density: {dens:.4}");
    println!("  Diameter: {diameter:?}");
    println!("  Girth: {girth:?}");
    println!("  Triangles: {triangles}");
    println!("  Avg path length: {apl:.3}\n");
    Ok(())
}

fn analyze_centrality(g: &Graph) -> Result<(Vec<f64>, Vec<f64>), Box<dyn std::error::Error>> {
    let pr = pagerank(g)?;
    let bc = betweenness(g)?;

    let top_pr = pr
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(i, s)| (i, *s))
        .unwrap();
    let top_bc = bc
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(i, s)| (i, *s))
        .unwrap();

    println!("Step 3: Centrality analysis");
    println!("  Top PageRank:    vertex {} ({:.4})", top_pr.0, top_pr.1);
    println!("  Top betweenness: vertex {} ({:.2})", top_bc.0, top_bc.1);

    let n = g.vcount();
    let degrees: Vec<usize> = (0..n).map(|v| g.degree(v).unwrap_or(0)).collect();
    let max_deg = degrees.iter().max().copied().unwrap_or(0);
    let min_deg = degrees.iter().min().copied().unwrap_or(0);
    let avg_deg = if n > 0 {
        #[allow(clippy::cast_precision_loss)]
        let avg = degrees.iter().sum::<usize>() as f64 / f64::from(n);
        avg
    } else {
        0.0
    };
    println!("  Degree range: [{min_deg}, {max_deg}], avg: {avg_deg:.2}\n");

    Ok((pr, bc))
}

fn detect_communities(g: &Graph) -> Result<rust_igraph::LouvainResult, Box<dyn std::error::Error>> {
    let communities = louvain(g)?;
    println!("Step 4: Community detection (Louvain)");
    println!("  Membership: {:?}", communities.membership);
    println!("  Modularity: {:.4}", communities.modularity);

    let n_communities = communities.membership.iter().max().copied().unwrap_or(0) + 1;
    for c in 0..n_communities {
        let members: Vec<usize> = communities
            .membership
            .iter()
            .enumerate()
            .filter(|&(_, m)| *m == c)
            .map(|(i, _)| i)
            .collect();
        println!("  Community {c}: {members:?} ({} members)", members.len());
    }
    println!();
    Ok(communities)
}

fn export_with_attributes(
    g: &Graph,
    pr: &[f64],
    bc: &[f64],
    communities: &rust_igraph::LouvainResult,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut g_out = g.clone();
    let dens = density(g)?.unwrap_or(0.0);

    for v in 0..g_out.vcount() {
        let vi = v as usize;
        g_out.set_vertex_attribute("pagerank", v, AttributeValue::Numeric(pr[vi]))?;
        g_out.set_vertex_attribute("betweenness", v, AttributeValue::Numeric(bc[vi]))?;
        g_out.set_vertex_attribute(
            "community",
            v,
            AttributeValue::Numeric(f64::from(communities.membership[vi])),
        )?;
    }

    g_out.set_graph_attribute(
        "modularity",
        AttributeValue::Numeric(communities.modularity),
    );
    g_out.set_graph_attribute("density", AttributeValue::Numeric(dens));

    let mut buf = Vec::new();
    rust_igraph::write_gml(&g_out, &mut buf)?;
    let gml_size = buf.len();

    let mut xml_buf = Vec::new();
    rust_igraph::write_graphml(&g_out, None, &mut xml_buf)?;
    let xml_size = xml_buf.len();

    println!("Step 5: Export with attributes");
    println!("  GML output: {gml_size} bytes");
    println!("  GraphML output: {xml_size} bytes");
    println!("  Attributes per vertex: pagerank, betweenness, community");
    println!("  Graph attributes: modularity, density\n");

    let bridge_list = g.bridges()?;
    let art_points = g.articulation_points()?;
    println!("Step 6: Structural vulnerabilities");
    println!("  Bridges: {bridge_list:?}");
    println!("  Articulation points: {art_points:?}\n");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Network Analysis Pipeline ===\n");

    let g = build_network()?;
    analyze_structure(&g)?;
    let (pr, bc) = analyze_centrality(&g)?;
    let communities = detect_communities(&g)?;
    export_with_attributes(&g, &pr, &bc, &communities)?;

    let top_pr = pr
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(i, s)| (i, *s))
        .unwrap();

    println!("=== Analysis Complete ===");
    println!(
        "  15-vertex network with 3 communities (modularity {:.4})",
        communities.modularity
    );
    println!(
        "  Most influential vertex: {} (PageRank {:.4})",
        top_pr.0, top_pr.1
    );

    Ok(())
}

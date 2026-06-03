//! Comprehensive social network analysis demo.
//!
//! Demonstrates rust-igraph's breadth: graph construction, community
//! detection, centrality, shortest paths, structural properties,
//! vertex/edge/graph attributes, and attribute-aware I/O round-trips
//! — all in one coherent workflow on Zachary's karate club.
//!
//! Run: `cargo run --example social_network_demo`

use std::fs::File;
use std::path::PathBuf;

use rust_igraph::AttributeValue;

#[allow(clippy::too_many_lines)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- 1. Load graph ---
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    let file = File::open(path.canonicalize()?)?;
    let mut g = rust_igraph::read_edgelist(file)?;
    println!("=== Zachary's Karate Club ===");
    println!("{g}");
    println!();

    // --- 2. Basic properties ---
    #[allow(clippy::cast_precision_loss)]
    let density = g.ecount() as f64 / (f64::from(g.vcount()) * (f64::from(g.vcount()) - 1.0) / 2.0);
    let is_conn = rust_igraph::is_connected(&g, rust_igraph::ConnectednessMode::Weak)?;
    println!("Density: {density:.4}");
    println!("Connected: {is_conn}");
    let diam = rust_igraph::diameter(&g)?;
    println!(
        "Diameter: {}",
        diam.map_or("N/A".to_string(), |d| d.to_string())
    );
    println!();

    // --- 3. Centrality → store as vertex attributes ---
    let pr = rust_igraph::pagerank(&g)?;
    let bc = rust_igraph::betweenness(&g)?;
    let cl = rust_igraph::closeness(&g)?;

    g.set_vertex_attribute_all(
        "pagerank",
        pr.iter().copied().map(AttributeValue::Numeric).collect(),
    )?;
    g.set_vertex_attribute_all(
        "betweenness",
        bc.iter().copied().map(AttributeValue::Numeric).collect(),
    )?;
    g.set_vertex_attribute_all(
        "closeness",
        cl.iter()
            .map(|c| AttributeValue::Numeric(c.unwrap_or(0.0)))
            .collect(),
    )?;

    let mut top_pr: Vec<(usize, f64)> = pr.iter().copied().enumerate().collect();
    top_pr.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    println!("Top-5 by PageRank:");
    for (v, score) in top_pr.iter().take(5) {
        println!(
            "  vertex {v:>2}: PR={score:.4}, betweenness={:.1}, closeness={:.4}",
            bc[*v],
            cl[*v].unwrap_or(0.0)
        );
    }
    println!();

    // --- 4. Community detection → store as vertex attribute ---
    let louvain = rust_igraph::louvain(&g)?;
    let n_communities = louvain
        .membership
        .iter()
        .copied()
        .max()
        .map_or(0, |m| m + 1);
    println!(
        "Louvain: {n_communities} communities, Q = {:.4}",
        louvain.modularity
    );

    g.set_vertex_attribute_all(
        "community",
        louvain
            .membership
            .iter()
            .map(|&c| AttributeValue::Numeric(f64::from(c)))
            .collect(),
    )?;

    for c in 0..n_communities {
        let members: Vec<u32> = g
            .vertex_ids()
            .filter(|&v| louvain.membership[v as usize] == c)
            .collect();
        println!(
            "  Community {c}: {} members {:?}",
            members.len(),
            if members.len() > 8 {
                &members[..8]
            } else {
                &members
            }
        );
    }
    println!();

    // --- 5. Shortest paths ---
    let paths = rust_igraph::distances(&g, 0)?;
    let reachable: Vec<f64> = paths.iter().filter_map(|d| d.map(f64::from)).collect();
    #[allow(clippy::cast_precision_loss)]
    let avg_dist = reachable.iter().sum::<f64>() / (reachable.len().max(1) as f64);
    println!("Average distance from vertex 0: {avg_dist:.2}");

    // --- 6. Structural properties ---
    let transitivity = rust_igraph::transitivity_undirected(&g)?;
    println!(
        "Global clustering coefficient: {:.4}",
        transitivity.unwrap_or(0.0)
    );

    let components = rust_igraph::connected_components(&g)?;
    println!("Connected components: {}", components.count);
    println!();

    // --- 7. Graph-level attributes ---
    g.set_graph_attribute(
        "name",
        AttributeValue::String("Zachary's Karate Club".into()),
    );
    g.set_graph_attribute("vertices", AttributeValue::Numeric(f64::from(g.vcount())));

    println!("Graph attributes:");
    for name in g.graph_attribute_names() {
        if let Some(val) = g.graph_attribute(name) {
            println!("  {name} = {val}");
        }
    }
    println!();

    // --- 8. Attribute-aware I/O round-trip: GML ---
    let mut gml_buf = Vec::new();
    rust_igraph::write_gml(&g, &mut gml_buf)?;
    let gml_str = String::from_utf8_lossy(&gml_buf);
    println!("--- GML output (first 500 chars) ---");
    println!("{}", &gml_str[..gml_str.len().min(500)]);
    println!("...\n");

    let g_from_gml = rust_igraph::read_gml(gml_buf.as_slice())?;
    println!(
        "GML round-trip: {} vertices, {} edges, pagerank[0] = {:.4}",
        g_from_gml.vcount(),
        g_from_gml.ecount(),
        g_from_gml
            .vertex_attribute("pagerank", 0)
            .and_then(AttributeValue::as_f64)
            .unwrap_or(0.0),
    );
    println!(
        "  graph name = {:?}",
        g_from_gml
            .graph_attribute("name")
            .and_then(AttributeValue::as_str)
            .unwrap_or("?"),
    );
    println!();

    // --- 9. Attribute-aware I/O round-trip: GraphML ---
    let mut graphml_buf = Vec::new();
    rust_igraph::write_graphml(&g, None, &mut graphml_buf)?;
    let graphml_str = String::from_utf8_lossy(&graphml_buf);
    println!("--- GraphML output (first 500 chars) ---");
    println!("{}", &graphml_str[..graphml_str.len().min(500)]);
    println!("...\n");

    let g_from_graphml = rust_igraph::read_graphml(graphml_buf.as_slice())?;
    println!(
        "GraphML round-trip: {} vertices, {} edges, community[0] = {:.0}",
        g_from_graphml.graph.vcount(),
        g_from_graphml.graph.ecount(),
        g_from_graphml
            .graph
            .vertex_attribute("community", 0)
            .and_then(AttributeValue::as_f64)
            .unwrap_or(-1.0),
    );
    println!();

    // --- 10. Attribute-aware I/O: DOT output ---
    let labels: Vec<String> = (0..g.vcount()).map(|v| format!("v{v}")).collect();
    let mut dot_buf = Vec::new();
    rust_igraph::write_dot(&g, Some(&labels), &mut dot_buf)?;
    let dot_str = String::from_utf8_lossy(&dot_buf);
    println!("--- DOT output (first 500 chars) ---");
    println!("{}", &dot_str[..dot_str.len().min(500)]);
    println!("...\n");

    let dot_result = rust_igraph::read_dot(dot_buf.as_slice())?;
    println!(
        "DOT round-trip: {} vertices, {} edges",
        dot_result.graph.vcount(),
        dot_result.graph.ecount(),
    );
    println!(
        "  pagerank[0] = {:.4}, community[0] = {:.0}",
        dot_result
            .graph
            .vertex_attribute("pagerank", 0)
            .and_then(AttributeValue::as_f64)
            .unwrap_or(0.0),
        dot_result
            .graph
            .vertex_attribute("community", 0)
            .and_then(AttributeValue::as_f64)
            .unwrap_or(-1.0),
    );
    println!();

    // --- 11. Attribute-aware I/O: Pajek round-trip (via attributes) ---
    let mut pajek_buf = Vec::new();
    rust_igraph::write_pajek(&g, None, None, &mut pajek_buf)?;
    let pajek_result = rust_igraph::read_pajek(pajek_buf.as_slice())?;
    println!(
        "Pajek round-trip: {} vertices, {} edges",
        pajek_result.graph.vcount(),
        pajek_result.graph.ecount(),
    );
    println!();

    // --- 12. Attribute-aware I/O: NCOL round-trip (via attributes) ---
    let mut ncol_buf = Vec::new();
    rust_igraph::write_ncol(&g, None, None, &mut ncol_buf)?;
    let ncol_result = rust_igraph::read_ncol(ncol_buf.as_slice())?;
    println!(
        "NCOL round-trip: {} vertices, {} edges, name[0] = {:?}",
        ncol_result.graph.vcount(),
        ncol_result.graph.ecount(),
        ncol_result.names.first().unwrap_or(&String::new()),
    );
    println!();

    // --- 13. Attribute-aware I/O: LGL round-trip (via attributes) ---
    let mut lgl_buf = Vec::new();
    rust_igraph::write_lgl(&g, None, None, &mut lgl_buf)?;
    let lgl_result = rust_igraph::read_lgl(lgl_buf.as_slice())?;
    println!(
        "LGL round-trip: {} vertices, {} edges",
        lgl_result.graph.vcount(),
        lgl_result.graph.ecount(),
    );
    println!();

    // --- 14. Attribute-aware I/O: DL round-trip (via attributes) ---
    let mut dl_buf = Vec::new();
    rust_igraph::write_dl(&g, None, None, &mut dl_buf)?;
    let dl_result = rust_igraph::read_dl(dl_buf.as_slice(), false)?;
    println!(
        "DL round-trip: {} vertices, {} edges",
        dl_result.graph.vcount(),
        dl_result.graph.ecount(),
    );
    println!();

    // --- 15. Attribute-aware I/O: LEDA round-trip (via attributes) ---
    let mut leda_buf = Vec::new();
    rust_igraph::write_leda(&g, None, None, &mut leda_buf)?;
    let leda_result = rust_igraph::read_leda(leda_buf.as_slice())?;
    println!(
        "LEDA round-trip: {} vertices, {} edges",
        leda_result.graph.vcount(),
        leda_result.graph.ecount(),
    );
    println!();

    println!(
        "Done — {} capabilities demonstrated: construction, density, connectivity,\n\
         diameter, PageRank, betweenness, closeness, community detection, shortest\n\
         paths, clustering coefficient, attributes, GML I/O, GraphML I/O, DOT I/O,\n\
         Pajek I/O, NCOL I/O, LGL I/O, DL I/O, LEDA I/O.",
        19
    );
    Ok(())
}

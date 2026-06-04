//! File I/O demo — read/write graphs in multiple formats.
//!
//! Run: `cargo run --example file_io_demo`

use rust_igraph::{AttributeValue, Graph};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut g = Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 0), (0, 2)], false, None)?;

    g.set_vertex_attribute("name", 0, AttributeValue::String("Alice".into()))?;
    g.set_vertex_attribute("name", 1, AttributeValue::String("Bob".into()))?;
    g.set_vertex_attribute("name", 2, AttributeValue::String("Carol".into()))?;
    g.set_vertex_attribute("name", 3, AttributeValue::String("Dave".into()))?;

    g.set_edge_attribute("weight", 0, AttributeValue::Numeric(1.0))?;
    g.set_edge_attribute("weight", 1, AttributeValue::Numeric(2.5))?;
    g.set_edge_attribute("weight", 2, AttributeValue::Numeric(0.8))?;
    g.set_edge_attribute("weight", 3, AttributeValue::Numeric(1.2))?;
    g.set_edge_attribute("weight", 4, AttributeValue::Numeric(3.0))?;

    println!("Original: {g}");

    // --- Format-auto-detecting file I/O ---
    let tmp = std::env::temp_dir();

    // GML round-trip
    let gml_path = tmp.join("demo.gml");
    g.to_file(&gml_path)?;
    let g2 = Graph::from_file(&gml_path)?;
    println!("GML round-trip: {g2}");
    assert_eq!(g2.vcount(), g.vcount());

    // GraphML round-trip
    let graphml_path = tmp.join("demo.graphml");
    g.to_file(&graphml_path)?;
    let g3 = Graph::from_file(&graphml_path)?;
    println!("GraphML round-trip: {g3}");
    assert_eq!(g3.vcount(), g.vcount());

    // DOT round-trip
    let dot_path = tmp.join("demo.dot");
    g.to_file(&dot_path)?;
    let g4 = Graph::from_file(&dot_path)?;
    println!("DOT round-trip: {g4}");

    // Edge list round-trip
    let edge_path = tmp.join("demo.edges");
    g.to_file(&edge_path)?;
    let g5 = Graph::from_file(&edge_path)?;
    println!("Edge list round-trip: {g5}");

    // Pajek round-trip
    let pajek_path = tmp.join("demo.net");
    g.to_file(&pajek_path)?;
    let g6 = Graph::from_file(&pajek_path)?;
    println!("Pajek round-trip: {g6}");

    // --- Stream-based I/O (more control) ---
    let mut buf = Vec::new();
    rust_igraph::write_gml(&g, &mut buf)?;
    println!("\nGML output ({} bytes):", buf.len());
    println!("{}", String::from_utf8_lossy(&buf[..buf.len().min(200)]));

    // Clean up
    for p in [&gml_path, &graphml_path, &dot_path, &edge_path, &pajek_path] {
        let _ = std::fs::remove_file(p);
    }

    println!("\nAll format round-trips successful!");
    Ok(())
}

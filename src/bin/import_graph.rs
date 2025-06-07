//! CLI tool for importing graph files

use ia::domain::services::{GraphImportService, ImportFormat};
use std::env;
use std::path::Path;
use tracing::{info, error};

fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        return;
    }

    let file_path = &args[1];
    let format = if args.len() > 2 {
        match args[2].as_str() {
            "arrows" | "arrows_app" => ImportFormat::ArrowsApp,
            "cypher" => ImportFormat::Cypher,
            "mermaid" => ImportFormat::Mermaid,
            "dot" => ImportFormat::Dot,
            "progress" | "progress_json" => ImportFormat::ProgressJson,
            "vocabulary" | "vocabulary_json" => ImportFormat::VocabularyJson,
            "rss" | "atom" | "rss_atom" => ImportFormat::RssAtom,
            _ => {
                error!("Unknown format: {}", args[2]);
                print_usage(&args[0]);
                return;
            }
        }
    } else {
        // Try to detect format from file extension
        detect_format_from_path(file_path)
    };

    // Check if file exists
    if !Path::new(file_path).exists() {
        error!("File not found: {}", file_path);
        return;
    }

    info!("Importing file: {} with format: {:?}", file_path, format);

    // Create import service
    let import_service = GraphImportService::new();

    // Read file content
    match std::fs::read_to_string(file_path) {
        Ok(content) => {
            // Import the content
            match import_service.import_from_content(&content, format, None) {
                Ok(imported_graph) => {
                    info!("Successfully imported graph!");
                    info!("  Nodes: {}", imported_graph.nodes.len());
                    info!("  Edges: {}", imported_graph.edges.len());

                    // Print some details
                    for node in &imported_graph.nodes {
                        info!("  Node: {} - {} at ({}, {}, {})",
                            node.id,
                            node.label,
                            node.position.x,
                            node.position.y,
                            node.position.z
                        );
                    }

                    for edge in &imported_graph.edges {
                        info!("  Edge: {} from {} to {} ({})",
                            edge.id,
                            edge.source,
                            edge.target,
                            edge.edge_type
                        );
                    }
                }
                Err(e) => {
                    error!("Failed to import graph: {}", e);
                }
            }
        }
        Err(e) => {
            error!("Failed to read file: {}", e);
        }
    }
}

fn print_usage(program: &str) {
    println!("Usage: {program} <file_path> [format]");
    println!();
    println!("Formats:");
    println!("  arrows_app    - Arrows.app JSON format");
    println!("  cypher        - Cypher query language");
    println!("  mermaid       - Mermaid diagram");
    println!("  dot           - Graphviz DOT format");
    println!("  progress_json - Progress JSON format");
    println!("  vocabulary    - Vocabulary JSON format");
    println!("  rss_atom      - RSS/Atom feed format");
    println!();
    println!("If format is not specified, it will be detected from the file extension.");
}

fn detect_format_from_path(path: &str) -> ImportFormat {
    let path = Path::new(path);
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    match extension {
        "json" => {
            // Try to detect JSON format from filename
            let filename = path.file_stem().and_then(|f| f.to_str()).unwrap_or("");
            if filename.contains("progress") {
                ImportFormat::ProgressJson
            } else if filename.contains("vocabulary") || filename.contains("vocab") {
                ImportFormat::VocabularyJson
            } else {
                ImportFormat::ArrowsApp // Default JSON format
            }
        }
        "cypher" | "cql" => ImportFormat::Cypher,
        "mermaid" | "mmd" => ImportFormat::Mermaid,
        "dot" | "gv" => ImportFormat::Dot,
        "xml" | "rss" | "atom" => ImportFormat::RssAtom,
        _ => {
            info!("Unknown extension '{}', defaulting to ArrowsApp format", extension);
            ImportFormat::ArrowsApp
        }
    }
}

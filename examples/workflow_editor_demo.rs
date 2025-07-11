//! Workflow editor demo - Visual workflow creation

use alchemist::workflow_editor;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔄 Alchemist Workflow Editor Demo");
    println!("==================================");
    println!();
    println!("This demo shows the visual workflow editor with drag-and-drop.");
    println!("Features:");
    println!("  • Drag-and-drop node creation");
    println!("  • Connect nodes to create workflows");
    println!("  • Pan and zoom canvas");
    println!("  • Export to YAML format");
    println!("  • Node property editing");
    println!();
    println!("Controls:");
    println!("  • Click and drag nodes to move them");
    println!("  • Mouse wheel to zoom");
    println!("  • Middle mouse or space+drag to pan");
    println!();
    
    workflow_editor::run_workflow_editor().await
}
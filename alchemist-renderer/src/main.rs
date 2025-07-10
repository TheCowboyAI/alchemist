//! Alchemist Renderer - Spawns Bevy or Iced windows based on requests

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[cfg(feature = "bevy-renderer")]
mod bevy_renderer;
mod iced_simple;

#[derive(Parser)]
#[command(name = "alchemist-renderer")]
#[command(about = "Renderer for Alchemist - handles Bevy 3D and Iced 2D windows")]
struct Cli {
    /// Renderer type (bevy or iced)
    renderer: String,
    
    /// Data file containing render request
    #[arg(long)]
    data_file: PathBuf,
    
    /// Renderer ID
    #[arg(long)]
    id: String,
}

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("alchemist_renderer=debug".parse()?)
        )
        .init();
    
    let cli = Cli::parse();
    
    // Load render request from file
    let request_json = std::fs::read_to_string(&cli.data_file)?;
    let request: alchemist::renderer::RenderRequest = serde_json::from_str(&request_json)?;
    
    // Clean up temp file
    let _ = std::fs::remove_file(&cli.data_file);
    
    // Launch appropriate renderer
    match cli.renderer.as_str() {
        "bevy" => {
            #[cfg(feature = "bevy-renderer")]
            {
                bevy_renderer::run(request)?;
            }
            #[cfg(not(feature = "bevy-renderer"))]
            {
                anyhow::bail!("Bevy renderer not compiled in. Build with --features bevy-renderer");
            }
        }
        "iced" => {
            iced_simple::run(request)?;
        }
        _ => {
            anyhow::bail!("Unknown renderer type: {}", cli.renderer);
        }
    }
    
    Ok(())
}

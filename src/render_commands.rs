//! Render commands for the shell

use clap::Subcommand;

#[derive(Subcommand)]
pub enum RenderCommands {
    /// Launch a 3D graph visualization in Bevy
    Graph {
        /// Title for the window
        #[arg(short, long, default_value = "Graph Visualization")]
        title: String,
        
        /// Graph data file (JSON)
        #[arg(short, long)]
        file: Option<String>,
        
        /// Use 2D Iced renderer instead of 3D Bevy
        #[arg(long)]
        iced: bool,
    },
    
    /// Open a document viewer
    Document {
        /// Document file path
        file: String,
        
        /// Document format (auto-detected if not specified)
        #[arg(short, long)]
        format: Option<String>,
    },
    
    /// Open a text editor
    Edit {
        /// File to edit
        file: Option<String>,
        
        /// Language for syntax highlighting
        #[arg(short, long)]
        language: Option<String>,
    },
    
    /// Visualize a workflow
    Workflow {
        /// Workflow ID or file
        workflow: String,
        
        /// Use 3D visualization
        #[arg(long)]
        three_d: bool,
    },
    
    /// Launch a video player
    Video {
        /// Video file or URL
        url: String,
    },
    
    /// Launch an audio player
    Audio {
        /// Audio file or URL
        url: String,
        
        /// Additional files for playlist
        #[arg(short, long)]
        playlist: Vec<String>,
    },
    
    /// List active renderer windows
    List,
    
    /// Close a renderer window
    Close {
        /// Renderer ID
        id: String,
    },
    
    /// Demo: Show various renderer capabilities
    Demo {
        /// Which demo to run
        #[arg(value_enum)]
        demo_type: DemoType,
    },
}

#[derive(clap::ValueEnum, Clone)]
pub enum DemoType {
    /// 3D graph with animation
    Graph3d,
    /// Document viewer with markdown
    Document,
    /// Text editor
    Editor,
    /// Workflow visualization
    Workflow,
    /// Split view (multiple windows)
    Split,
}
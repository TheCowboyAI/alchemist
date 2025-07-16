//! Test Graph Loading with Real Files
//!
//! This example creates sample files and tests loading them through the shell

use alchemist::{
    shell::AlchemistShell,
    config::AlchemistConfig,
    shell_commands::Commands,
    render_commands::RenderCommands,
};
use anyhow::Result;
use std::fs;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("alchemist=info")
        .init();

    println!("=== Alchemist Graph Loading Test ===\n");

    // Create test files
    create_test_files()?;

    // Create shell
    let config = AlchemistConfig::default();
    let mut shell = AlchemistShell::new(config).await?;

    // Test 1: Load JSON graph
    println!("1. Testing JSON graph loading...");
    shell.handle_command(Commands::Render(RenderCommands::Graph {
        title: "JSON Test Graph".to_string(),
        file: Some("test_graph.json".to_string()),
        iced: false,
    })).await?;
    println!("✅ JSON graph loaded\n");

    // Test 2: Load Nix dependencies
    println!("2. Testing Nix dependency graph loading...");
    shell.handle_command(Commands::Render(RenderCommands::Graph {
        title: "Nix Dependencies".to_string(),
        file: Some("test_package.nix".to_string()),
        iced: false,
    })).await?;
    println!("✅ Nix dependency graph loaded\n");

    // Test 3: Load Markdown structure
    println!("3. Testing Markdown structure loading...");
    shell.handle_command(Commands::Render(RenderCommands::Graph {
        title: "Document Structure".to_string(),
        file: Some("test_document.md".to_string()),
        iced: false,
    })).await?;
    println!("✅ Markdown structure loaded\n");

    // Test 4: Load Cytoscape format
    println!("4. Testing Cytoscape format loading...");
    shell.handle_command(Commands::Render(RenderCommands::Graph {
        title: "Cytoscape Graph".to_string(),
        file: Some("test_cytoscape.json".to_string()),
        iced: false,
    })).await?;
    println!("✅ Cytoscape graph loaded\n");

    // Clean up test files
    cleanup_test_files();

    println!("All tests passed! 🎉");
    println!("\nNote: Renderer windows may have been spawned in background.");
    println!("Use 'render list' to see active renderers.");

    Ok(())
}

fn create_test_files() -> Result<()> {
    // Standard JSON graph
    let json_graph = r#"{
  "nodes": [
    {
      "id": "hub",
      "label": "Central Hub",
      "position": [0, 0, 0],
      "metadata": {
        "type": "hub",
        "importance": "high"
      }
    },
    {
      "id": "service1",
      "label": "API Service",
      "position": [5, 0, 0],
      "metadata": {
        "type": "service",
        "port": 8080
      }
    },
    {
      "id": "service2",
      "label": "Database",
      "position": [-5, 0, 0],
      "metadata": {
        "type": "database",
        "engine": "PostgreSQL"
      }
    },
    {
      "id": "service3",
      "label": "Cache",
      "position": [0, 5, 0],
      "metadata": {
        "type": "cache",
        "engine": "Redis"
      }
    },
    {
      "id": "client1",
      "label": "Web Client",
      "position": [5, 5, 0],
      "metadata": {
        "type": "client"
      }
    },
    {
      "id": "client2",
      "label": "Mobile Client",
      "position": [-5, 5, 0],
      "metadata": {
        "type": "client"
      }
    }
  ],
  "edges": [
    {
      "id": "e1",
      "source": "hub",
      "target": "service1",
      "label": "REST API"
    },
    {
      "id": "e2",
      "source": "hub",
      "target": "service2",
      "label": "queries"
    },
    {
      "id": "e3",
      "source": "hub",
      "target": "service3",
      "label": "caches"
    },
    {
      "id": "e4",
      "source": "client1",
      "target": "service1",
      "label": "HTTP"
    },
    {
      "id": "e5",
      "source": "client2",
      "target": "service1",
      "label": "HTTP"
    },
    {
      "id": "e6",
      "source": "service1",
      "target": "service3",
      "label": "read/write"
    }
  ]
}"#;
    fs::write("test_graph.json", json_graph)?;

    // Nix package definition
    let nix_content = r#"{ pkgs ? import <nixpkgs> {} }:

let
  myPython = pkgs.python3.withPackages (ps: with ps; [
    numpy
    pandas
    matplotlib
    scikit-learn
  ]);
in
pkgs.stdenv.mkDerivation rec {
  pname = "data-science-env";
  version = "1.0.0";
  
  buildInputs = [
    myPython
    pkgs.jupyter
    pkgs.git
    pkgs.curl
    pkgs.jq
  ];
  
  propagatedBuildInputs = [
    pkgs.nodejs
    pkgs.yarn
  ];
  
  nativeBuildInputs = [
    pkgs.gcc
    pkgs.cmake
  ];
  
  shellHook = ''
    echo "Data Science Environment Loaded"
  '';
}"#;
    fs::write("test_package.nix", nix_content)?;

    // Markdown document
    let markdown_content = r#"# Alchemist Documentation

## Overview
Alchemist is a revolutionary graph visualization system.

### Key Features
- Real-time graph rendering
- Multiple file format support
- Interactive visualization

### Architecture
The system uses a modular architecture.

#### Core Components
- Parser Module
- Renderer Engine
- Persistence Layer

#### Data Flow
1. Input parsing
2. Graph construction
3. Layout calculation
4. Rendering

## Getting Started

### Installation
Follow the installation guide at [docs/install.md](docs/install.md).

### Basic Usage
```bash
alchemist render graph --file mydata.json
```

### Advanced Features
See the [advanced guide](docs/advanced.md) for more features.

## API Reference

### Graph Parser API
- `parse_json_graph()`
- `parse_nix_graph()`
- `parse_markdown_graph()`

### Renderer API
Details at [api.example.com](https://api.example.com).

## Contributing
Please read [contributing.md](contributing.md) before submitting PRs.

### Development Setup
1. Clone the repository
2. Install dependencies
3. Run tests

## Resources
- [GitHub Repository](https://github.com/example/alchemist)
- [Issue Tracker](https://github.com/example/alchemist/issues)
- [Discord Community](https://discord.gg/example)
"#;
    fs::write("test_document.md", markdown_content)?;

    // Cytoscape format
    let cytoscape_content = r#"{
  "elements": [
    {
      "data": {
        "id": "a",
        "label": "Node A"
      },
      "position": {
        "x": 100,
        "y": 100
      }
    },
    {
      "data": {
        "id": "b",
        "label": "Node B"
      },
      "position": {
        "x": 200,
        "y": 100
      }
    },
    {
      "data": {
        "id": "c",
        "label": "Node C"
      },
      "position": {
        "x": 150,
        "y": 200
      }
    },
    {
      "data": {
        "id": "ab",
        "source": "a",
        "target": "b",
        "label": "Edge AB"
      }
    },
    {
      "data": {
        "id": "bc",
        "source": "b",
        "target": "c",
        "label": "Edge BC"
      }
    },
    {
      "data": {
        "id": "ca",
        "source": "c",
        "target": "a",
        "label": "Edge CA"
      }
    }
  ]
}"#;
    fs::write("test_cytoscape.json", cytoscape_content)?;

    println!("Created test files:");
    println!("  - test_graph.json (standard format)");
    println!("  - test_package.nix (Nix dependencies)");
    println!("  - test_document.md (Markdown structure)");
    println!("  - test_cytoscape.json (Cytoscape format)");
    println!();

    Ok(())
}

fn cleanup_test_files() {
    let files = [
        "test_graph.json",
        "test_package.nix",
        "test_document.md",
        "test_cytoscape.json",
    ];

    for file in &files {
        if let Err(e) = fs::remove_file(file) {
            eprintln!("Warning: Could not remove {}: {}", file, e);
        }
    }
}
# cim-domain-nix

A Domain-Driven Design (DDD) module for managing Nix ecosystem operations within the Composable Information Machine (CIM) architecture.

## Overview

This module provides a complete domain model for working with the Nix ecosystem, including:

- **Flakes**: Create, update, and manage Nix flakes
- **Modules**: Define and compose NixOS modules
- **Overlays**: Create package overlays
- **Configurations**: Manage NixOS system configurations
- **Packages**: Build and query Nix packages
- **Expressions**: Evaluate Nix expressions

## Architecture

The module follows DDD principles with clear separation of concerns:

### Domain Layer
- **Aggregates**: `FlakeAggregate`, `ModuleAggregate`, `OverlayAggregate`, `ConfigurationAggregate`
- **Value Objects**: `FlakeRef`, `AttributePath`, `Derivation`, `StorePath`, etc.
- **Events**: Domain events for all significant state changes
- **Commands**: Commands representing user intentions

### Application Layer
- **Command Handlers**: Process commands and interact with Nix CLI
- **Query Handlers**: Retrieve information about Nix entities
- **Projections**: Read models for efficient querying

## Usage

### Creating a Flake

```rust
use cim_domain_nix::{
    commands::{CreateFlake, NixCommand},
    handlers::NixCommandHandler,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let handler = NixCommandHandler::new();
    
    let command = CreateFlake {
        path: "/path/to/project".into(),
        description: "My Nix project".to_string(),
        template: Some("rust".to_string()),
    };
    
    let events = handler.handle_command(Box::new(command)).await?;
    
    for event in events {
        println!("Event: {:?}", event);
    }
    
    Ok(())
}
```

### Building a Package

```rust
use cim_domain_nix::{
    commands::{BuildPackage, NixCommand},
    handlers::NixCommandHandler,
    value_objects::AttributePath,
};

let command = BuildPackage {
    flake_ref: "github:NixOS/nixpkgs".to_string(),
    attribute: AttributePath::new(vec!["hello".to_string()]),
    output_path: None,
};

let events = handler.handle_command(Box::new(command)).await?;
```

### Managing NixOS Configurations

```rust
use cim_domain_nix::{
    commands::{CreateConfiguration, ActivateConfiguration, ActivationType},
    value_objects::NixOSConfiguration,
};

// Create configuration
let config = NixOSConfiguration {
    hostname: "my-machine".to_string(),
    system: "x86_64-linux".to_string(),
    modules: vec![],
    specialisations: HashMap::new(),
};

let create_cmd = CreateConfiguration {
    name: "my-config".to_string(),
    configuration: config,
};

// Activate configuration
let activate_cmd = ActivateConfiguration {
    name: "my-config".to_string(),
    activation_type: ActivationType::Switch,
};
```

## Integration with CIM

This module integrates seamlessly with the CIM architecture:

1. **Event-Driven**: All state changes emit domain events
2. **CQRS**: Separate command and query models
3. **Async**: Full async/await support for Nix operations
4. **Type-Safe**: Leverages Rust's type system for safety

## Development

### Running Tests

```bash
# Unit tests
cargo test --lib

# Integration tests (requires Nix)
cargo test --test integration_test

# All tests
cargo test
```

### Examples

See the `examples/` directory for more usage examples:

- `create_flake.rs`: Simple flake creation
- `flake_operations.rs`: Comprehensive flake operations

### Building with Nix

```bash
# Enter development shell
nix develop

# Build the package
nix build

# Run checks
nix flake check
```

## Features

- **Flake Management**: Create, update, and manage Nix flakes with inputs and outputs
- **Package Building**: Build packages from flakes or nixpkgs
- **Module System**: Create reusable NixOS modules
- **Overlay Support**: Define package overlays for customization
- **Configuration Management**: Create and activate NixOS configurations
- **Expression Evaluation**: Evaluate arbitrary Nix expressions
- **Garbage Collection**: Clean up unused store paths
- **Nixpkgs Search**: Search for packages in nixpkgs

## Error Handling

The module provides comprehensive error handling with specific error types:

- `FlakeError`: Flake-related errors
- `BuildError`: Package build failures
- `ParseError`: Parsing errors for Nix expressions
- `IoError`: File system errors
- `CommandError`: Nix command execution errors

## License

See the main CIM project license. 
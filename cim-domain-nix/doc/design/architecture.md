# CIM Domain Nix - Architecture Design

## Overview

The `cim-domain-nix` module provides a Domain-Driven Design (DDD) implementation for managing Nix ecosystem operations within the Composable Information Machine (CIM) architecture. This document outlines the architectural decisions and design patterns used.

## Domain Model

### Core Concepts

1. **Flake**: A self-contained Nix project with explicit dependencies
2. **Module**: A reusable NixOS configuration unit
3. **Overlay**: A function that modifies package sets
4. **Configuration**: A complete NixOS system configuration
5. **Package**: A buildable software unit
6. **Store Path**: An immutable path in the Nix store

### Aggregates

#### FlakeAggregate
- **Purpose**: Manages the lifecycle of Nix flakes
- **Invariants**: 
  - Must have a valid path
  - Inputs must be valid flake references
  - Cannot have circular dependencies
- **Commands**: CreateFlake, UpdateFlake, AddFlakeInput
- **Events**: FlakeCreated, FlakeUpdated, FlakeInputAdded

#### ModuleAggregate
- **Purpose**: Manages NixOS module definitions
- **Invariants**:
  - Options must have valid types
  - Imports must reference valid modules
- **Commands**: CreateModule
- **Events**: ModuleCreated

#### OverlayAggregate
- **Purpose**: Manages package overlays
- **Invariants**:
  - Must define valid package modifications
- **Commands**: CreateOverlay
- **Events**: OverlayCreated

#### ConfigurationAggregate
- **Purpose**: Manages NixOS system configurations
- **Invariants**:
  - Must have a valid hostname
  - System must be a supported platform
  - Generation numbers must be sequential
- **Commands**: CreateConfiguration, ActivateConfiguration
- **Events**: ConfigurationCreated, ConfigurationActivated

### Value Objects

#### FlakeRef
Represents a reference to a Nix flake:
- URI (e.g., "github:owner/repo")
- Optional revision
- Optional subflake path

#### AttributePath
Represents a dotted path to a Nix attribute:
- Segments (e.g., ["packages", "x86_64-linux", "hello"])

#### StorePath
Represents a path in the Nix store:
- Hash
- Name
- Full path validation

#### Derivation
Represents a Nix derivation:
- Name
- System
- Builder
- Arguments
- Environment variables
- Output paths

## Command/Query Responsibility Segregation (CQRS)

### Commands
Commands represent intentions to change state:
- CreateFlake
- UpdateFlake
- AddFlakeInput
- BuildPackage
- CreateModule
- CreateOverlay
- CreateConfiguration
- ActivateConfiguration
- EvaluateExpression
- RunGarbageCollection

### Queries
Queries retrieve information without side effects:
- FindFlakeQuery
- FindPackageQuery
- FindConfigurationQuery
- SearchNixPackagesQuery

### Command Handlers

#### FlakeCommandHandler
- Creates flake.nix files
- Manages flake inputs
- Runs `nix flake check`

#### PackageCommandHandler
- Executes `nix build`
- Handles build outputs
- Manages build cache

#### ExpressionCommandHandler
- Evaluates Nix expressions
- Returns JSON results

#### GarbageCollectionHandler
- Runs `nix-collect-garbage`
- Reports freed space

### Query Handlers

#### NixQueryHandler
- Basic queries for flakes, packages, configurations
- Uses projections for efficient lookups

#### AdvancedNixQueryHandler
- Extends basic queries
- Integrates with nixpkgs search

## Event-Driven Architecture

### Domain Events
Events represent facts that have occurred:
- FlakeCreated
- FlakeUpdated
- FlakeInputAdded
- PackageBuilt
- ModuleCreated
- OverlayCreated
- ConfigurationCreated
- ConfigurationActivated
- ExpressionEvaluated
- GarbageCollected

### Event Flow
1. Command received by handler
2. Handler validates command
3. Handler executes Nix operations
4. Domain events generated
5. Events published to event store
6. Projections updated
7. Queries can read from projections

## Integration Points

### Nix CLI Integration
- Commands execute actual Nix CLI tools
- Output parsing for structured data
- Error handling for command failures

### File System Integration
- Flake creation writes to disk
- Configuration management
- Store path validation

### Process Management
- Async command execution
- Timeout handling
- Resource cleanup

## Error Handling Strategy

### Error Types
1. **FlakeError**: Flake-specific failures
2. **BuildError**: Package build failures
3. **ParseError**: Nix expression parsing errors
4. **IoError**: File system errors
5. **CommandError**: Nix CLI execution errors

### Error Propagation
- Commands return Result types
- Errors include context and recovery hints
- Transient vs permanent error distinction

## Security Considerations

1. **Path Validation**: Prevent directory traversal
2. **Command Injection**: Sanitize inputs to Nix CLI
3. **Resource Limits**: Timeout long-running operations
4. **Sandboxing**: Leverage Nix's build sandboxing

## Performance Considerations

1. **Caching**: Leverage Nix's built-in caching
2. **Async Operations**: Non-blocking command execution
3. **Projection Optimization**: Efficient read models
4. **Batch Operations**: Group related commands

## Testing Strategy

### Unit Tests
- Test aggregates in isolation
- Mock file system and process execution
- Property-based testing for value objects

### Integration Tests
- Test actual Nix CLI integration
- Verify file system changes
- End-to-end command flows

### Example Tests
- Create and update flakes
- Build packages
- Activate configurations

## Future Enhancements

1. **Remote Builders**: Support distributed builds
2. **Flake Templates**: Pre-defined project templates
3. **Dependency Visualization**: Graph flake dependencies
4. **Performance Metrics**: Build time tracking
5. **Cross-Compilation**: Support multiple target systems
6. **Nix Profile Management**: User environment management 
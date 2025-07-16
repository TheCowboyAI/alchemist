# Archived Source Code - January 2025

This directory contains the archived source code from the main `/src` directory as of January 2025.

## Reason for Archive

We decided to restart the implementation with a cleaner approach:
- Start with just Bevy + NATS in main.rs
- Build up from cim-contextgraph and cim-domain modules
- Focus on a more modular, incremental approach

## Archive Contents

- **application/** - Command handlers, projections, query handlers, services
- **bin/** - Binary executables (demo_graph_composition, demo_nats_connection)
- **contexts/** - Bounded contexts (conceptual, ddd, graph, workflow)
- **domain/** - Domain layer (aggregates, commands, events, services, value objects)
- **infrastructure/** - Infrastructure layer (event bridge, event store, NATS, persistence)
- **presentation/** - Presentation layer (Bevy systems, components, plugins)
- **shared/** - Shared types and events
- **lib.rs** - Library exports
- **main.rs** - Main application entry point

## Test Results Before Archive

The main application (`cargo run --bin ia`) was running but had runtime errors:
- "Event not initialized" error in workflow visualization system
- Many compilation warnings about unused code
- Visual demos were partially broken due to refactoring

## Working Components

Before archiving, these components were tested and working:
- cim-domain: 192 tests passing
- cim-contextgraph: 31 tests passing
- cim-ipld: 14 tests passing
- graph-composition: 14 tests passing

Total: 251+ tests passing

## Date Archived

January 2025

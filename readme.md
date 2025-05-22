# Information Alchemist
"ia"

A program to visualize Information Architecture and Display it in a 3D World


We use the [WebGPU standard](https://www.w3.org/TR/WGSL/)


This World may connect to LIVE Events and Data Streams through the Event System.

Incoming Events will update Observables that the Entities watch.

![The Alchemist](./alchemist.webp)
>"a person who transforms or creates something through a seemingly magical process."

This is an experimental User Interface and Projection system for a CIM.

The idea is that everything in the Information System is Identified.
It is composed of Entities (identifiable objects), Values (components), Behaviors (systems) and Events. 

These equate to our three base Models:
  - Applied Categories
  - Entity Component System (ECS)
  - Domain Driven Design (DDD)

## Development with Nix

This project uses Nix for reproducible builds. See [cache-readme.md](./cache-readme.md) for detailed information on using the local Nix cache to speed up builds.

### Using the Local Cache Effectively

For best results with the local cache:

```bash
# NEW: Two-step build process for maximum caching efficiency
# Step 1: Build and cache only the dependencies (do once)
just build-deps

# Step 2: Build your application using cached dependencies (do every time)
just build-after-deps

# Check dependency cache status
just check-deps-cache

# HIGHLY RECOMMENDED: Build with pure inputs
# This completely eliminates Git dirty status issues
# and creates fully reproducible builds
just build-pure

# Run from a clean checkout (recommended with build-clean)
# This avoids dirty workspace issues and ensures cache hits
just run-clean

# Clean builds from latest commit (also works well)
# This avoids dirty workspace issues and ensures cache hits
just build-clean

# Development shell with cache
just develop

# Build with cache (works but may rebuild if workspace is dirty)
just build

# Run with cache (works but may rebuild if workspace is dirty)
just run

# Check if dependencies are in cache
just verify-deps

# Troubleshoot cache issues
just debug-cache
```

**IMPORTANT**: A dirty Git workspace (uncommitted changes) will create unique package hashes that won't match what's in the cache. For best performance, use one of these approaches:

1. **Two-step build process (RECOMMENDED)**: 
   - First run `just build-deps` to cache all libraries
   - Then run `just build-after-deps` for application builds
   - This ensures libraries are only built once regardless of your application changes

2. **Pure builds**:
   - Run `just build-pure` which filters the source
   - Creates consistent derivation hashes regardless of Git status

## Understanding Nix Caching

For those who are curious why dirty Git workspaces cause cache misses:

1. Nix creates derivation hashes based on all inputs, including the source code
2. When your Git workspace has uncommitted changes, the source input hash changes
3. This creates a unique derivation hash that won't match what's in the cache
4. The `build-pure` command works by filtering the source to only include relevant files and exclude Git metadata

## Mathematical Model
The Mathematical Model is our definition of Mathematics and how we apply it.

### Applied Categories
These are actual Categories we define using Applied Category Theory.
Categories are Mathematical Objects which Model our Worlds.
There are known specifications for Categories using Category Theory.

## Observable Model
The Observable Model is how we observe the system, these are User Interfaces (UIs) and Applicatiopn Programming Interfaces (APIs).

### Entity Component System
We use an Entity Component System where:

#### Components
Components are Values
They are collections of data structures
No functionality is provided other than providing data

#### Entities
Identifiable Object with a Unique Identifier
Entities are composed from Components
An Entity is an Identified Collection of Values

#### Systems
Systems are behaviors and functionality which can be applied to Entities

## Domain Model
Domains are the boundaries we set on collections of ECS Worlds.

Domains define everything about a given collection of Values, Entities containing these values, and the systems that operate on them.

These are the definitions of meaning we apply to the ECS world.




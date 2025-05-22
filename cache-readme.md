# Alchemist Nix Cache Guide

This document explains how to effectively use the local Nix cache (`http://localhost:5000`) to speed up builds for the Alchemist project.

## Overview

The Alchemist project uses a local Nix cache to speed up builds by avoiding redundant compilation of dependencies. The cache is particularly helpful for:

1. The heavy Rust dependencies that rarely change
2. The core system libraries needed for Bevy
3. The main application binary when building from clean Git states

## Cache Configuration

The cache is configured in several files:

- `cache-config.nix`: Central configuration for cache URLs and keys
- `cache-tools.nix`: Utilities for diagnosing cache issues
- `cache-management.nix`: Tools for pushing packages to the cache
- `analyze-cache-miss.nix`: Analyzer for debugging cache misses
- `justfile`: User-friendly commands for interacting with the cache

## Cache Limitations

There are some important limitations to be aware of:

1. **Git Dirty State Issue**: Nix adds unique timestamps to builds from dirty Git workspaces, making them uncacheable
2. **HTTP Endpoints**: The HTTP cache endpoints don't serve `.narinfo` files as expected, but Nix's internal protocols can still use the cache
3. **Large Dependencies**: Bevy and its dependencies are sizable and benefit most from caching

## Quick Start Guide

### 1. Update your Nix configuration

This adds the local cache to your global Nix configuration:

```bash
just update-nix-conf
```

### 2. Build and cache Rust dependencies (NEW RECOMMENDED APPROACH)

This builds and caches just the Rust dependencies as a separate package:

```bash
just build-deps
```

### 3. Build your application using the cached dependencies

```bash
just build-after-deps
```

### 4. Verify the dependency cache status

```bash
just check-deps-cache
```

### 5. Alternative: Prime the cache with Rust dependencies

This builds and pushes the Rust dependencies to the cache (older approach):

```bash
just prime-cache
```

### 6. Alternative: Build from a clean Git state

For maximum cache efficiency when using the older approach, build from a clean Git commit:

```bash
just build-from-commit HEAD .#
```

### 7. Run the application using the cache

```bash
just run .#
```

## Common Commands

| Command                            | Description                             |
| ---------------------------------- | --------------------------------------- |
| `just build-deps`                  | Build and cache only Rust dependencies  |
| `just build-after-deps`            | Build app using cached dependencies     |
| `just check-deps-cache`            | Check status of dependency cache        |
| `just build`                       | Build using the local cache             |
| `just run`                         | Run using the local cache               |
| `just build-from-commit`           | Build from a clean Git state            |
| `just cache-report`                | Check cache status                      |
| `just analyze-cache-miss`          | Analyze why a package isn't cached      |
| `just verify-deps`                 | Check if dependencies are in the cache  |
| `just add-to-cache /nix/store/...` | Add a specific path to the cache        |
| `just check-path /nix/store/...`   | Check if a path is in the cache         |
| `just optimized-build`             | Build with advanced cache optimizations |

## Troubleshooting

### Cache Miss Despite Clean Workspace

If you're experiencing cache misses with a clean Git workspace:

1. Use the analyzer to check for issues:
   ```bash
   just analyze-cache-miss
   ```

2. Check for unique/impure attributes in your derivation:
   ```bash
   nix show-derivation $(nix-instantiate .)
   ```

3. Explicitly add the package to the cache:
   ```bash
   just add-to-cache $(nix-build --no-out-link)
   ```

### Building Without Cache

If you need to build completely from scratch (bypassing cache):

```bash
nix build .# --option substitute false
```

### Cache Monitor

For continuous monitoring of cache status:

```bash
nix run .#cache-monitor
```

## Advanced Usage

### Two-Step Build Process (NEW RECOMMENDED APPROACH)

The most efficient way to use the cache is with the two-step build process:

1. **Step 1: Build and cache dependencies once**
   ```bash
   just build-deps
   ```
   This creates a separate Nix package containing just your Rust dependencies, which can be cached independently from your application code.

2. **Step 2: Build your application using cached dependencies**
   ```bash
   just build-after-deps
   ```
   This builds only your application code, using the pre-cached dependencies.

3. **Check cache status**
   ```bash
   just check-deps-cache
   ```
   This shows the status of the dependency cache and provides useful information.

The benefit of this approach is that even with a dirty Git workspace, the dependencies remain cached and don't need to be rebuilt.

### Building Just the Dependencies (Older Method)

To build and cache just the Rust dependencies using the older approach:

```bash
nix build .#rustDeps
just add-to-cache result
```

### Running the Cache Analysis Tools

For detailed cache analysis:

```bash
nix build .#cacheReport
./result/bin/cache-report
```

## Cache Architecture

- **Local Cache Server**: A nix-serve instance running on localhost:5000
- **Signing Key**: `dell-62S6063:F1R/DQVxh0R0YUBXEdVClqDsddJ5VLWVYzPrHC9mmqc=`
- **Cache Integration**: The cache is integrated at both the flake and devShell levels 
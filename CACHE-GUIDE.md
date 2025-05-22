# Content-Addressable Nix Caching Guide

This guide explains our improved approach to Nix caching that doesn't depend on Git state.

## The Problem

Previously, our build system had these issues:

1. **Git-based Inputs**: Nix would include the Git repository state in the derivation hash, which caused cache misses when your workspace was dirty.

2. **Monolithic Builds**: We were building everything together, so changing application code would cause all dependencies to be rebuilt.

## The Solution

We've implemented a content-addressable approach that:

1. **Ignores Git State**: Builds are based on file content, not Git status.

2. **Separates Dependencies**: Dependencies are built separately from application code.

3. **Content Filtering**: Only includes files that actually affect the build.

## How It Works

The system now uses these components:

1. **`pure-source.nix`**: Creates filtered source inputs that:
   - Include only build-relevant files
   - Exclude Git metadata and irrelevant directories
   - Create separate filtered sources for dependencies and application code

2. **Two-Phase Build Process**:
   - `rustDeps` package: Builds only the dependencies
   - `default` package: Builds the application using cached dependencies

## Using the Cache Effectively

For best results, use these commands:

```bash
# First-time setup: Build and cache dependencies
just build-deps-pure

# Regular development: Build application using cached dependencies
just build-after-deps-pure

# Run the application
just run-pure

# Check cache status
just check-pure-status
```

## Benefits

This approach provides:

1. **Consistent Builds**: Even with dirty Git workspace
2. **Faster Rebuilds**: Dependencies are properly cached
3. **Independent Caching**: Dependencies are cached separately from application code
4. **Local Development Friendly**: Changes to application code don't invalidate dependency cache

## Troubleshooting

If you encounter issues:

1. **Missing Cache**: Run `just build-deps-pure` to rebuild and cache dependencies
2. **Corrupted Cache**: Run `just check-pure-status` to verify cache status
3. **Still Having Issues**: Delete the `result` symlink and try again with `just build-pure-content` 
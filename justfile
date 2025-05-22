# Local cache settings
local-cache := "http://localhost:5000"
local-cache-key := "dell-62S6063:F1R/DQVxh0R0YUBXEdVClqDsddJ5VLWVYzPrHC9mmqc="
nixos-cache-key := "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY="
nix-community-key := "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
devenv-key := "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw="
substituters := "https://cache.nixos.org/ " + local-cache + " https://nix-community.cachix.org https://devenv.cachix.org"
trusted-keys := nixos-cache-key + " " + local-cache-key + " " + nix-community-key + " " + devenv-key

# ============================================================================
# LOCAL CACHE USAGE NOTES
# ============================================================================
# The local cache at http://localhost:5000 is a nix-serve instance that:
# 1. Is correctly recognized by Nix as a cache (nix path-info can query it)
# 2. Shows up as a valid build source when using --option substituters
# 3. Accepts packages via "nix copy --to"
# 
# However, there appears to be a configuration issue where:
# - The HTTP endpoints don't serve .narinfo files as expected
# - This makes "curl http://localhost:5000/[hash]-name.narinfo" return 404
# - But Nix's internal protocols can still read from it
#
# To get the best use of the cache:
# 1. Always build using the commands in this justfile (like "just build")
# 2. The cache usage is most effective for clean workspace builds
# 3. Dirty git workspaces create unique/uncacheable derivations
# ============================================================================

# Default recipe
default:
    @just --list

# Build with local cache
build *args:
    @echo "Building with local cache at {{local-cache}}"
    nix build --option substituters "{{substituters}}" --option trusted-public-keys "{{trusted-keys}}" -L -v --log-format bar-with-logs {{args}}

# Build with local cache from a specific git commit to avoid dirty repo issues
build-from-commit commit_ref="HEAD" target=".#":
    #!/usr/bin/env bash
    echo "Building from clean commit {{commit_ref}} with local cache at {{local-cache}}"
    # Verify commit exists
    if ! git rev-parse --verify {{commit_ref}} >/dev/null 2>&1; then
        echo "Error: Git commit {{commit_ref}} not found"
        exit 1
    fi
    
    # Get the full commit hash
    COMMIT=$(git rev-parse {{commit_ref}})
    echo "Using commit: $COMMIT"
    
    # Build using the git+file URL format with explicit commit
    nix build --option substituters "{{substituters}}" \
              --option trusted-public-keys "{{trusted-keys}}" \
              -L -v --log-format bar-with-logs \
              git+file:///git/thecowboyai/alchemist?ref=$COMMIT#{{target}}

# Run with local cache
run *args:
    @echo "Running with local cache at {{local-cache}}"
    nix run --option substituters "{{substituters}}" --option trusted-public-keys "{{trusted-keys}}" -v --log-format bar-with-logs {{args}}

# Develop with local cache
develop *args:
    @echo "Starting development shell with local cache at {{local-cache}}"
    nix develop --option substituters "{{substituters}}" --option trusted-public-keys "{{trusted-keys}}" {{args}}

# Check build with local cache
check *args:
    @echo "Checking with local cache at {{local-cache}}"
    nix flake check --option substituters "{{substituters}}" --option trusted-public-keys "{{trusted-keys}}" {{args}}

# Check cache connection
check-cache:
    @echo "Checking connection to local cache at {{local-cache}}"
    curl -I {{local-cache}}/nix-cache-info
    @echo "\nNote: HTTP caches don't support listing all entries. Use 'just check-path /nix/store/...' to check specific paths."

# Check if a specific path exists in the cache
check-path path:
    #!/usr/bin/env bash
    if [ -z "{{path}}" ]; then
        echo "Please provide a path to check"
        echo "Example: just check-path /nix/store/abc123-some-package"
        exit 1
    fi
    
    if [[ ! "{{path}}" =~ ^/nix/store/.* ]]; then
        echo "Error: Path must be a full /nix/store path"
        exit 1
    fi
    
    HASH=$(nix-store --query --hash "{{path}}" 2>/dev/null || echo "")
    if [ -z "$HASH" ]; then
        if [ ! -e "{{path}}" ]; then
            echo "Error: Path does not exist locally: {{path}}"
            exit 1
        fi
        echo "Failed to get hash for {{path}}"
        exit 1
    fi
    
    BASENAME=$(basename "{{path}}")
    echo "Checking if {{path}} exists in cache at {{local-cache}}..."
    echo "Hash: $HASH"
    echo "Path: $BASENAME"
    
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "{{local-cache}}/$BASENAME.narinfo")
    if [ "$HTTP_CODE" = "200" ]; then
        echo "✅ Path exists in cache!"
        curl -s "{{local-cache}}/$BASENAME.narinfo"
    else
        echo "❌ Path not found in cache (HTTP $HTTP_CODE)"
    fi

# Add a specific path to the cache
add-to-cache path:
    #!/usr/bin/env bash
    if [ -z "{{path}}" ]; then
        echo "Please provide a path to add to the cache"
        echo "Example: just add-to-cache /nix/store/abc123-some-package"
        exit 1
    fi
    
    if [[ ! "{{path}}" =~ ^/nix/store/.* ]]; then
        echo "Error: Path must be a full /nix/store path"
        exit 1
    fi
    
    if [ ! -e "{{path}}" ]; then
        echo "Error: Path does not exist locally: {{path}}"
        exit 1
    fi
    
    echo "Adding {{path}} to cache at {{local-cache}}..."
    nix copy --to {{local-cache}} "{{path}}" && echo "Successfully added to cache!" || echo "Failed to add to cache!"
    
    # Verify it was added
    just check-path {{path}}

# Show all available substituters and keys
show-nix-config:
    @echo "Current Nix configuration:"
    nix show-config | grep -E 'substituter|key'

# Advanced debugging command to show exactly what's happening with caching
debug-cache *args:
    #!/usr/bin/env bash
    echo "Debugging cache usage for {{args}}"
    echo "Current Git status:"
    git status --short
    echo ""
    echo "Running build with verbose substituter logs:"
    
    # Run with maximum verbosity to see all cache activity
    NIX_DEBUG=7 nix build --option substituters "{{substituters}}" \
            --option trusted-public-keys "{{trusted-keys}}" \
            -v --no-link --log-format bar-with-logs {{args}} 2>&1
    
# Check if a specific path is available from any of the substituters
check-output-path path:
    #!/usr/bin/env bash
    if [ -z "{{path}}" ]; then
        # First determine the derivation path for the default package
        DRVPATH=$(nix-instantiate --no-build-output . 2>/dev/null)
        # Then determine the output path
        OUTPUT_PATH=$(nix-store -q --outputs $DRVPATH 2>/dev/null)
        if [ -z "$OUTPUT_PATH" ]; then
            echo "Error: Could not determine output path for default package"
            exit 1
        fi
        PATH_TO_CHECK="$OUTPUT_PATH"
        echo "No path specified, using default package output path: $PATH_TO_CHECK"
    else
        PATH_TO_CHECK="{{path}}"
    fi
    
    echo "Checking if $PATH_TO_CHECK is available in substituters:"
    echo "- Checking local cache ({{local-cache}})..."
    nix path-info --store {{local-cache}} "$PATH_TO_CHECK" 2>/dev/null && echo "✅ Found in local cache" || echo "❌ Not found in local cache"
    
    echo "- Checking official cache (https://cache.nixos.org/)..."
    nix path-info --store https://cache.nixos.org/ "$PATH_TO_CHECK" 2>/dev/null && echo "✅ Found in official cache" || echo "❌ Not found in official cache"

# Standard Cargo commands
c-build:
    cargo build

c-run:
    cargo run

c-watch:
    RUSTFLAGS='-A warnings' cargo watch -x run 
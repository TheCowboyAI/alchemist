# Import cache settings from cache-config.nix
local-cache := `nix eval --raw --impure --expr 'with import ./cache-config.nix; localCache'`
local-cache-key := `nix eval --raw --impure --expr 'with import ./cache-config.nix; localCacheKey'`
substituters := `nix eval --raw --impure --expr 'with import ./cache-config.nix; builtins.concatStringsSep " " allSubstituters'`
trusted-keys := `nix eval --raw --impure --expr 'with import ./cache-config.nix; builtins.concatStringsSep " " allTrustedKeys'`

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

# ============================================================================
# CONTENT-ADDRESSABLE BUILD APPROACH
# ============================================================================
# Build using a content-addressable approach that doesn't depend on Git state
# This works by:
# 1. Filtering the source to only include files that affect the build
# 2. Creating a pure source derivation with only build-relevant files
# 3. Building from this filtered source instead of the Git workspace
# 
# Benefits:
# - Builds are cacheable even with dirty Git workspaces
# - Dependencies are built separately and cached independently
# - Multiple local changes don't invalidate the entire cache
# ============================================================================

# Build using content-addressable source filtering (Git-state independent)
build-pure-content target=".#":
    #!/usr/bin/env bash
    echo "Building with content-addressable approach (Git-state independent)"
    
    # Check if pure-source.nix exists, create it if missing
    if [ ! -f pure-source.nix ]; then
        echo "Error: pure-source.nix not found. Please create it first."
        exit 1
    fi
    
    # Build with --no-warn-dirty to ignore Git status
    echo "Building with content filtering..."
    nix build --no-warn-dirty \
             --option substituters "{{substituters}}" \
             --option trusted-public-keys "{{trusted-keys}}" \
             -L -v --log-format bar-with-logs \
             {{target}}

# Build only Rust dependencies using content-addressable approach
build-deps-pure:
    #!/usr/bin/env bash
    echo "Building Rust dependencies with content-addressable approach (Git-state independent)"
    
    # Check if pure-source.nix exists
    if [ ! -f pure-source.nix ]; then
        echo "Error: pure-source.nix not found. Please create it first."
        exit 1
    fi
    
    # Use nix build with --no-warn-dirty to ignore git status
    echo "Building rustDeps package with pure source..."
    nix build --no-warn-dirty --no-link -L ./\#rustDeps
    
    # Get the path to the built rustDeps package
    RUST_DEPS_PATH=$(nix path-info --no-warn-dirty ./\#rustDeps)
    
    # Add the result to the cache explicitly
    if [ -n "$RUST_DEPS_PATH" ]; then
        echo "Adding Rust dependencies to cache at path: $RUST_DEPS_PATH"
        nix copy --to {{local-cache}} "$RUST_DEPS_PATH"
        
        echo "✅ Rust dependencies successfully built and cached"
        echo "You can now build your application with 'just build-after-deps-pure' and it will use the cached dependencies"
    else
        echo "❌ Failed to build Rust dependencies"
        exit 1
    fi

# Build application after dependencies using content-addressable approach
build-after-deps-pure:
    #!/usr/bin/env bash
    echo "Building application with pure-source approach after pre-cached dependencies"
    
    # Get the path for the rustDeps package
    RUST_DEPS_PATH=$(nix path-info --no-warn-dirty ./\#rustDeps 2>/dev/null)
    
    if [ -z "$RUST_DEPS_PATH" ]; then
        echo "❌ Cannot determine path for Rust dependencies"
        echo "Run 'just build-deps-pure' first to build and cache the dependencies"
        exit 1
    fi
    
    # Check if the rustDeps package exists in the cache
    if nix path-info --store {{local-cache}} "$RUST_DEPS_PATH" 2>/dev/null; then
        echo "✅ Using cached Rust dependencies from: $RUST_DEPS_PATH"
    else
        echo "⚠️ Rust dependencies found but not in cache, adding to cache now..."
        nix copy --to {{local-cache}} "$RUST_DEPS_PATH"
    fi
    
    # Build the main package with dependencies from cache
    echo "Building application using content-addressable source and cached dependencies..."
    nix build --no-warn-dirty --option substituters "{{substituters}}" \
              --option trusted-public-keys "{{trusted-keys}}" \
              -L -v --log-format bar-with-logs

# Run application with content-addressable build
run-pure target=".#":
    #!/usr/bin/env bash
    echo "Running with content-addressable approach (Git-state independent)"
    
    # Build first with no warn dirty
    just build-pure-content {{target}}
    
    # Run with the result
    echo "Running the built application..."
    ./result/bin/alchemist

# Check cache status of content-addressable build
check-pure-status:
    #!/usr/bin/env bash
    echo "Checking status of content-addressable build packages..."
    
    # Check rustDeps
    echo "Checking rustDeps package..."
    RUST_DEPS_PATH=$(nix path-info --no-warn-dirty ./\#rustDeps 2>/dev/null || echo "")
    if [ -n "$RUST_DEPS_PATH" ]; then
        echo "✅ rustDeps found: $RUST_DEPS_PATH"
        
        if nix path-info --store {{local-cache}} "$RUST_DEPS_PATH" 2>/dev/null; then
            echo "✅ rustDeps is cached"
        else
            echo "❌ rustDeps not in cache"
        fi
    else
        echo "❌ rustDeps not found"
    fi
    
    # Check main application
    echo "Checking main application package..."
    MAIN_PATH=$(nix path-info --no-warn-dirty ./\#default 2>/dev/null || echo "")
    if [ -n "$MAIN_PATH" ]; then
        echo "✅ main application found: $MAIN_PATH"
        
        if nix path-info --store {{local-cache}} "$MAIN_PATH" 2>/dev/null; then
            echo "✅ main application is cached"
        else
            echo "❌ main application not in cache"
        fi
    else
        echo "❌ main application not found"
    fi

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
    
    # Get the absolute path to the repo
    REPO_PATH=$(git rev-parse --show-toplevel)
    
    # Build using the git+file URL format with explicit commit
    nix build --option substituters "{{substituters}}" \
              --option trusted-public-keys "{{trusted-keys}}" \
              -L -v --log-format bar-with-logs \
              "git+file://$REPO_PATH?ref=$COMMIT#{{target}}"

# Build with local cache from a specific git commit (alternative approach)
build-clean commit_ref="HEAD" target=".#":
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
    
    # Create a temporary clean directory
    TEMP_DIR=$(mktemp -d)
    trap "rm -rf $TEMP_DIR" EXIT
    
    echo "Creating clean checkout in $TEMP_DIR..."
    git archive $COMMIT | tar -x -C $TEMP_DIR
    
    # Copy any untracked cache files that might be needed
    for file in cache-config.nix cache-tools.nix cache-management.nix analyze-cache-miss.nix; do
        if [ -f "$file" ] && [ ! -f "$TEMP_DIR/$file" ]; then
            echo "Copying $file to clean directory..."
            cp "$file" "$TEMP_DIR/$file"
        fi
    done
    
    # Build from the clean directory
    echo "Building from clean directory..."
    pushd $TEMP_DIR > /dev/null
    
    # Use --out-link to create a result symlink in the temp directory
    nix build --option substituters "{{substituters}}" \
              --option trusted-public-keys "{{trusted-keys}}" \
              -L -v --log-format bar-with-logs \
              {{target}}
    
    BUILD_SUCCESS=$?
    
    popd > /dev/null
    
    # If successful, create a symlink to the result instead of copying
    if [ $BUILD_SUCCESS -eq 0 ] && [ -e "$TEMP_DIR/result" ]; then
        echo "Build successful, creating result symlink..."
        
        # Get the actual store path
        RESULT_PATH=$(readlink -f "$TEMP_DIR/result")
        
        # Remove existing result if it's a symlink or empty directory 
        if [ -L "result" ]; then
            rm -f result
        elif [ -d "result" ]; then
            echo "Warning: Removing existing result directory"
            chmod -R u+w result 2>/dev/null || true
            rm -rf result
        fi
        
        # Create the symlink to the store path
        ln -sf "$RESULT_PATH" result
        echo "Done! Created symlink: result -> $RESULT_PATH"
    else
        echo "Build failed or produced no output"
        exit 1
    fi

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

# Run the advanced cache analysis tool on a derivation
analyze-cache-miss drv="":
    #!/usr/bin/env bash
    if [ -z "{{drv}}" ]; then
        # Try to get the derivation path for the default package
        DRV_PATH=$(nix-instantiate --no-build-output . 2>/dev/null)
        if [ -z "$DRV_PATH" ]; then
            echo "Error: No derivation specified and couldn't determine default"
            echo "Usage: just analyze-cache-miss /nix/store/hash-name.drv"
            exit 1
        fi
    else
        DRV_PATH="{{drv}}"
    fi
    
    # Run the analyzer
    nix-build analyze-cache-miss.nix --argstr drvPath "$DRV_PATH" --no-out-link
    ANALYZER=$(readlink -f $(nix-build analyze-cache-miss.nix --argstr drvPath "$DRV_PATH" --no-out-link 2>/dev/null)/bin/analyze-cache-miss)
    $ANALYZER

# Generate a cache report
cache-report:
    @nix-build cache-tools.nix -A cacheReport --no-out-link
    @$(nix-build cache-tools.nix -A cacheReport --no-out-link)/bin/cache-report

# Prime the cache with the most important dependencies
prime-cache:
    #!/usr/bin/env bash
    echo "Priming the cache with key dependencies..."
    
    # Check if we have a clean git state
    if ! git diff-index --quiet HEAD --; then
        echo "⚠️ Warning: Git workspace is dirty"
        echo "This will create unique/uncacheable package versions"
        echo "Consider running with a clean git state for better caching"
        
        read -p "Continue anyway? [y/N] " answer
        if [[ "$answer" != "y" && "$answer" != "Y" ]]; then
            echo "Aborting"
            exit 1
        fi
    fi
    
    # Build only the key dependencies
    echo "Building key Rust dependencies..."
    nix build --option substituters "{{substituters}}" \
            --option trusted-public-keys "{{trusted-keys}}" \
            -L --no-link .#rustDeps
    
    # Add them to the cache
    echo "Adding dependencies to cache..."
    nix copy --to {{local-cache}} $(nix-build --no-out-link .#rustDeps)
    
    echo "Done priming cache!"

# Update local Nix configuration to use the cache
update-nix-conf:
    #!/usr/bin/env bash
    echo "Updating Nix configuration to use local cache..."
    
    NIX_CONF="$HOME/.config/nix/nix.conf"
    CONTENT=$(nix eval --raw --impure --expr 'with import ./cache-config.nix; nixConfig')
    SYSTEM_NIX_CONF="/etc/nix/nix.conf"
    
    # First try user-level configuration
    echo "Attempting to update user-level configuration at $NIX_CONF"
    
    # Ensure the directory exists
    mkdir -p "$(dirname "$NIX_CONF")"
    
    # Check if the file exists
    if [ -f "$NIX_CONF" ]; then
        echo "Found existing nix.conf, making backup..."
        cp "$NIX_CONF" "$NIX_CONF.bak.$(date +%Y%m%d%H%M%S)"
        
        # Check for existing substituters and trusted-public-keys
        if grep -q "^substituters\s*=" "$NIX_CONF"; then
            echo "Existing substituters found, updating..."
            sed -i '/^substituters\s*=/d' "$NIX_CONF"
        fi
        
        if grep -q "^trusted-public-keys\s*=" "$NIX_CONF"; then
            echo "Existing trusted-public-keys found, updating..."
            sed -i '/^trusted-public-keys\s*=/d' "$NIX_CONF"
        fi
    else
        echo "Creating new nix.conf file..."
        touch "$NIX_CONF"
    fi
    
    # Add our configuration
    echo "$CONTENT" >> "$NIX_CONF"
    
    # Check if we need to also update system configuration
    if [ -f "$SYSTEM_NIX_CONF" ]; then
        echo ""
        echo "System-level Nix configuration exists at $SYSTEM_NIX_CONF"
        echo "To update system-level configuration, run this command with sudo:"
        echo ""
        echo "  sudo bash -c 'mkdir -p /etc/nix && echo \"$(nix eval --raw --impure --expr 'with import ./cache-config.nix; nixConfig')\" > $SYSTEM_NIX_CONF'"
        echo ""
        echo "Then restart the Nix daemon:"
        echo ""
        echo "  sudo systemctl restart nix-daemon.service"
        echo ""
    fi
    
    echo "User-level Nix configuration updated successfully!"
    echo "You may need to restart your shell for changes to take effect."

# Check for all dependencies for current build in cache
verify-deps:
    #!/usr/bin/env bash
    echo "Verifying all dependencies in cache..."
    
    # Create a temporary nix file
    cat > verify-temp.nix <<EOF
    { pkgs ? import <nixpkgs> {} }:
    let
      tools = import ./cache-tools.nix { inherit pkgs; };
    in
    tools.verifyDepsScript tools.dummyPackage
    EOF
    
    # Build and run it
    nix-build verify-temp.nix --no-out-link
    $(nix-build verify-temp.nix --no-out-link)/bin/verify-deps
    
    # Clean up
    rm verify-temp.nix

# Build the project with optimized caching and reduced size
optimized-build package=".#default":
    #!/usr/bin/env bash
    echo "Building with optimized caching for package: {{package}}"
    
    # Check if git workspace is clean
    if ! git diff-index --quiet HEAD --; then
        echo "⚠️ Warning: Git workspace is dirty!"
        echo "This will create uncacheable derivations with unique timestamps"
        echo "For optimal caching, commit your changes and use just build-from-commit"
        
        read -p "Continue anyway? [y/N] " answer
        if [[ "$answer" != "y" && "$answer" != "Y" ]]; then
            echo "Aborting. Commit your changes or use just build-from-commit"
            exit 1
        fi
    fi
    
    # Set the NIX_CONFIG variable to include our cache settings
    export NIX_CONFIG="$(nix eval --raw --impure --expr 'with import ./cache-config.nix; nixConfig')"
    
    # Build with all optimizations
    nix build {{package}} \
        --option substituters "{{substituters}}" \
        --option trusted-public-keys "{{trusted-keys}}" \
        --log-format bar-with-logs \
        --quiet
    
    # Add the result to the cache
    echo "Adding build result to cache..."
    nix copy --to {{local-cache}} ./result

# Standard Cargo commands
c-build:
    cargo build

c-run:
    cargo run

c-watch:
    RUSTFLAGS='-A warnings' cargo watch -x run

# Run from clean checkout, using same approach as build-clean
run-clean commit_ref="HEAD" target=".#":
    #!/usr/bin/env bash
    echo "Running from clean commit {{commit_ref}} with local cache at {{local-cache}}"
    # Verify commit exists
    if ! git rev-parse --verify {{commit_ref}} >/dev/null 2>&1; then
        echo "Error: Git commit {{commit_ref}} not found"
        exit 1
    fi
    
    # Get the full commit hash
    COMMIT=$(git rev-parse {{commit_ref}})
    echo "Using commit: $COMMIT"
    
    # Create a temporary clean directory
    TEMP_DIR=$(mktemp -d)
    trap "rm -rf $TEMP_DIR" EXIT
    
    echo "Creating clean checkout in $TEMP_DIR..."
    git archive $COMMIT | tar -x -C $TEMP_DIR
    
    # Copy any untracked cache files that might be needed
    for file in cache-config.nix cache-tools.nix cache-management.nix analyze-cache-miss.nix; do
        if [ -f "$file" ] && [ ! -f "$TEMP_DIR/$file" ]; then
            echo "Copying $file to clean directory..."
            cp "$file" "$TEMP_DIR/$file"
        fi
    done
    
    # Run from the clean directory
    echo "Running from clean directory..."
    pushd $TEMP_DIR > /dev/null
    
    nix run --option substituters "{{substituters}}" \
           --option trusted-public-keys "{{trusted-keys}}" \
           -L -v --log-format bar-with-logs \
           {{target}}
    
    RUN_EXIT_CODE=$?
    popd > /dev/null
    
    # Return the same exit code
    exit $RUN_EXIT_CODE

# Build with a filtered path approach for better reproducibility
build-reproducible commit_ref="HEAD" target=".#":
    #!/usr/bin/env bash
    echo "Building with path filtering for reproducibility with local cache at {{local-cache}}"
    
    # Create a temporary flake.nix that uses a specific filter for the source
    TEMP_DIR=$(mktemp -d)
    trap "rm -rf $TEMP_DIR" EXIT
    
    # Get commit hash
    COMMIT=$(git rev-parse {{commit_ref}})
    
    # Create a modified flake.nix that uses a filtered source
    cat > $TEMP_DIR/flake-repro.nix << 'EOF'
    { 
      # This is a wrapper around the main flake to produce reproducible builds
      description = "Reproducible Information Alchemist build";
      
      inputs = {
        nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
        flake-utils.url = "github:numtide/flake-utils";
        mainFlake.url = "path:/git/thecowboyai/alchemist";
      };
      
      outputs = { self, nixpkgs, flake-utils, mainFlake }: 
        flake-utils.lib.eachDefaultSystem (system:
          let
            pkgs = nixpkgs.legacyPackages.${system};
            
            # Import the cache configuration from the main flake
            cacheConfig = import /git/thecowboyai/alchemist/cache-config.nix;
          in {
            # Re-export the main flake's packages
            packages = mainFlake.packages.${system};
            
            # Default package is simply the main flake's default
            defaultPackage = mainFlake.packages.${system}.default;
            
            # Explicitly set this to enable substituters
            nixConfig = {
              extra-substituters = cacheConfig.allSubstituters;
              extra-trusted-public-keys = cacheConfig.allTrustedKeys;
            };
          }
        );
    }
    EOF
    
    # Build using our reproducible flake
    echo "Building from reproducible flake..."
    nix build --option substituters "{{substituters}}" \
            --option trusted-public-keys "{{trusted-keys}}" \
            --no-warn-dirty \
            -L -v --log-format bar-with-logs \
            $TEMP_DIR#{{target}} 

# Build using pure inputs to eliminate Git dirty status influence
build-pure target=".#":
    #!/usr/bin/env bash
    echo "Building with pure inputs for consistent cache usage at {{local-cache}}"
    
    # Create temporary directory for our files
    TEMP_DIR=$(mktemp -d)
    trap "rm -rf $TEMP_DIR" EXIT
    
    # Copy necessary files to the temp directory
    echo "Copying essential files to temporary directory..."
    cp flake.nix $TEMP_DIR/
    cp flake.lock $TEMP_DIR/
    cp -r src $TEMP_DIR/
    cp Cargo.toml $TEMP_DIR/
    cp Cargo.lock $TEMP_DIR/
    cp cache-config.nix $TEMP_DIR/
    cp cache-tools.nix $TEMP_DIR/
    cp cache-management.nix $TEMP_DIR/
    cp analyze-cache-miss.nix $TEMP_DIR/
    
    # Create the filter script that removes Git metadata
    cat > $TEMP_DIR/filter-git.nix << 'EOF'
    { pkgs ? import <nixpkgs> {} }:
    
    pkgs.stdenv.mkDerivation {
      name = "alchemist-filtered-source";
      # Use the current directory as source
      src = ./.;
      
      # We don't need to build anything
      dontBuild = true;
      
      # Just copy the files to the output
      installPhase = ''
        mkdir -p $out
        cp -r * $out/
        # Remove any Git-related files if they exist
        rm -rf $out/.git $out/.gitignore
      '';
    }
    EOF
    
    # Create a simple wrapper script for building with a cleaned source
    cat > $TEMP_DIR/build.sh << 'EOF'
    #!/usr/bin/env bash
    set -e
    
    # Create a filtered source derivation
    SOURCE_DRV=$(nix-build filter-git.nix --no-out-link)
    echo "Created filtered source at $SOURCE_DRV"
    
    # Build using the filtered source
    cd $SOURCE_DRV
    echo "Building from filtered source..."
    nix build --option substituters "$1" \
              --option trusted-public-keys "$2" \
              --no-warn-dirty \
              -L -v --log-format bar-with-logs \
              "$3"
              
    # Get the result path
    if [ -e "result" ]; then
        readlink -f "result"
    else
        echo "Build failed"
        exit 1
    fi
    EOF
    chmod +x $TEMP_DIR/build.sh
    
    # Run the build script
    echo "Running filtered build..."
    RESULT_PATH=$(cd $TEMP_DIR && ./build.sh "{{substituters}}" "{{trusted-keys}}" "{{target}}")
    
    # Copy the result back if successful
    if [ -n "$RESULT_PATH" ] && [ -d "$RESULT_PATH" ]; then
        echo "Build successful, creating result symlink..."
        rm -f result
        ln -sf "$RESULT_PATH" result
        echo "Created symlink: result -> $RESULT_PATH"
    else
        echo "Build failed or produced no output"
        exit 1
    fi 

# Determine exactly why a rebuild is needed by directly checking the cache for the derivation
cache-check target=".#":
    #!/usr/bin/env bash
    echo "Checking why a rebuild might be needed for {{target}}..."
    
    # First, determine the derivation path
    echo "Getting derivation path..."
    DRV_PATH=$(nix-instantiate --no-build-output . -A default 2>/dev/null)
    
    if [ -z "$DRV_PATH" ]; then
        echo "Error: Failed to get derivation path. Trying with '{{target}}'..."
        # Try using the target directly
        DRV_PATH=$(nix-instantiate --no-build-output . -A {{target}} 2>/dev/null)
        if [ -z "$DRV_PATH" ]; then
            echo "Error: Still failed to get derivation path. Using flake derivation show instead..."
            nix derivation show {{target}} >/dev/null 2>&1
            if [ $? -ne 0 ]; then
                echo "Error: Target {{target}} doesn't seem to be valid."
                exit 1
            else
                echo "Your flake derivation exists, but next steps need the deriver path."
                echo "Let's try to get the output path directly..."
                
                # Try to get the output path directly from a build
                OUTPUT_PATH=$(nix eval --raw {{target}}.outPath 2>/dev/null || echo "")
                if [ -n "$OUTPUT_PATH" ]; then
                    echo "Output path: $OUTPUT_PATH"
                    
                    echo "Checking if output exists in cache..."
                    if nix path-info --store {{local-cache}} "$OUTPUT_PATH" 2>/dev/null; then
                        echo "✅ Output path exists in the cache! Your build should not rebuild."
                        echo "   If it's still rebuilding, there may be a cache configuration issue."
                    else
                        echo "❌ Output path not found in cache."
                        echo "   This explains why Nix is rebuilding - the output isn't cached."
                    fi
                    exit 0
                else
                    echo "Error: Failed to determine output path."
                    echo "The best solution is to commit your changes and build with 'just build-clean'."
                    exit 1
                fi
            fi
        fi
    fi
    
    echo "Derivation path: $DRV_PATH"
    
    # Check if this derivation is in the cache
    echo "Checking if derivation exists in cache..."
    if nix path-info --store {{local-cache}} "$DRV_PATH" 2>/dev/null; then
        echo "✅ Good news! The exact derivation exists in the cache."
        echo "    If builds are still happening, it may be due to:"
        echo "    1. A dependency that's not in the cache"
        echo "    2. Nix not finding the derivation in the cache (network issue)"
        echo "    3. The actual output path doesn't match what Nix expects"
        
        # Now check the output path
        OUTPUT_PATH=$(nix-store -q --outputs "$DRV_PATH" 2>/dev/null)
        echo "Output path: $OUTPUT_PATH"
        
        echo "Checking if output exists in cache..."
        if nix path-info --store {{local-cache}} "$OUTPUT_PATH" 2>/dev/null; then
            echo "✅ Output path also exists in the cache! Your build should not rebuild."
            echo "   If it's still rebuilding, there may be a cache configuration issue."
        else
            echo "❌ Output path not found in cache."
            echo "   This explains why Nix is rebuilding - the output isn't cached."
        fi
    else
        echo "❌ Derivation not found in cache."
        echo "   This explains why Nix is rebuilding - the exact derivation doesn't exist in the cache."
        echo ""
        echo "Why this happens:"
        echo "1. Git dirty state: Uncommitted changes create a unique derivation hash"
        echo "2. Input changes: Dependencies or environment variables have changed"
        echo "3. Configuration differences: Nix settings that affect the build"
        
        # Recommend a solution
        echo ""
        echo "Recommended solution:"
        echo "1. Commit your changes to create a clean state"
        echo "2. Use 'just build-clean' to build from a clean checkout"
        echo "3. Once built, add the result to the cache: just add-to-cache \$(readlink -f result)" 

# Build only Rust dependencies to cache them separately, then add to the cache
build-deps:
    #!/usr/bin/env bash
    echo "Building only Rust dependencies to cache them separately"
    echo "This allows libraries to be cached independently of your application code"
    
    # Use nix build with --no-warn-dirty to ignore git status
    echo "Building rustDeps package..."
    nix build --no-warn-dirty --no-link -L ./\#rustDeps
    
    # Get the path to the built rustDeps package
    RUST_DEPS_PATH=$(nix path-info --no-warn-dirty ./\#rustDeps)
    
    # Add the result to the cache explicitly
    if [ -n "$RUST_DEPS_PATH" ]; then
        echo "Adding Rust dependencies to cache at path: $RUST_DEPS_PATH"
        nix copy --to {{local-cache}} "$RUST_DEPS_PATH"
        
        echo "✅ Rust dependencies successfully built and cached"
        echo "You can now build your application with 'just build-after-deps' and it will use the cached dependencies"
    else
        echo "❌ Failed to build Rust dependencies"
        exit 1
    fi

# Build application after dependencies (more efficient workflow)
build-after-deps:
    #!/usr/bin/env bash
    echo "Building application with pre-cached dependencies"
    
    # Get the path for the rustDeps package
    RUST_DEPS_PATH=$(nix path-info --no-warn-dirty ./\#rustDeps 2>/dev/null)
    
    if [ -z "$RUST_DEPS_PATH" ]; then
        echo "❌ Cannot determine path for Rust dependencies"
        echo "Run 'just build-deps' first to build and cache the dependencies"
        exit 1
    fi
    
    # Check if the rustDeps package exists in the cache
    if nix path-info --store {{local-cache}} "$RUST_DEPS_PATH" 2>/dev/null; then
        echo "✅ Using cached Rust dependencies from: $RUST_DEPS_PATH"
    else
        echo "⚠️ Rust dependencies found but not in cache, adding to cache now..."
        nix copy --to {{local-cache}} "$RUST_DEPS_PATH"
    fi
    
    # Build the main package with dependencies from cache
    echo "Building application using cached dependencies..."
    # Use --no-warn-dirty to ignore Git status
    nix build --no-warn-dirty --option substituters "{{substituters}}" \
              --option trusted-public-keys "{{trusted-keys}}" \
              -L -v --log-format bar-with-logs

# Build application after dependencies with detailed diagnostics
build-after-deps-debug:
    #!/usr/bin/env bash
    echo "Building application with pre-cached dependencies (DEBUG MODE)"
    
    # Get the path for the rustDeps package
    RUST_DEPS_PATH=$(nix path-info --no-warn-dirty ./\#rustDeps 2>/dev/null)
    
    if [ -z "$RUST_DEPS_PATH" ]; then
        echo "❌ Cannot determine path for Rust dependencies"
        echo "Run 'just build-deps' first to build and cache the dependencies"
        exit 1
    fi
    
    # Verify what exists in the rustDeps package
    echo "=== Examining rustDeps package ==="
    echo "rustDeps path: $RUST_DEPS_PATH"
    if [ -d "$RUST_DEPS_PATH/lib" ]; then
        echo "- lib directory exists"
        ls -la "$RUST_DEPS_PATH/lib" | head -n 20
        
        echo "- Counting library files:"
        find "$RUST_DEPS_PATH/lib" -name "*.rlib" | wc -l
        
        if [ -d "$RUST_DEPS_PATH/lib/deps" ]; then
            echo "- deps directory exists"
            ls -la "$RUST_DEPS_PATH/lib/deps" | head -n 20
        else
            echo "❌ deps directory missing"
        fi
        
        if [ -d "$RUST_DEPS_PATH/lib/.fingerprint" ]; then
            echo "- .fingerprint directory exists"
            ls -la "$RUST_DEPS_PATH/lib/.fingerprint" | head -n 20
        else
            echo "❌ .fingerprint directory missing"
        fi
    else
        echo "❌ lib directory missing in rustDeps package"
    fi
    
    # Check if the rustDeps package exists in the cache
    if nix path-info --store {{local-cache}} "$RUST_DEPS_PATH" 2>/dev/null; then
        echo "✅ rustDeps exists in cache: $RUST_DEPS_PATH"
    else
        echo "❌ rustDeps not in cache, adding now..."
        nix copy --to {{local-cache}} "$RUST_DEPS_PATH"
    fi
    
    # Build the main package with dependencies from cache and extra verbosity
    echo "=== Building application with maximum verbosity ==="
    NIX_DEBUG=10 nix build --no-warn-dirty --option substituters "{{substituters}}" \
              --option trusted-public-keys "{{trusted-keys}}" \
              -L --verbose --show-trace

# Display the cache status of the Rust dependencies
check-deps-cache:
    #!/usr/bin/env bash
    echo "Checking if Rust dependencies are cached..."
    
    # Check if we can build rustDeps locally
    if nix build --no-link --dry-run .#rustDeps 2>/dev/null; then
        # Get the path for the rustDeps package
        RUST_DEPS_PATH=$(nix eval --raw .#rustDeps.outPath 2>/dev/null)
        if [ -z "$RUST_DEPS_PATH" ]; then
            RUST_DEPS_PATH=$(nix build .#rustDeps --no-link --print-out-paths 2>/dev/null)
        fi
        
        if [ -n "$RUST_DEPS_PATH" ]; then
            echo "✅ Rust dependencies found at: $RUST_DEPS_PATH"
            
            # Check if it exists in the cache
            if nix path-info --store {{local-cache}} "$RUST_DEPS_PATH" 2>/dev/null; then
                echo "✅ Rust dependencies are cached"
                
                # Get information about the package
                echo ""
                echo "Package info:"
                nix path-info --json "$RUST_DEPS_PATH" | jq 2>/dev/null || echo "Could not get package info"
                
                # Check narinfo in cache
                echo ""
                echo "Cache entry details:"
                BASENAME=$(basename "$RUST_DEPS_PATH")
                curl -s "{{local-cache}}/$BASENAME.narinfo" || echo "Failed to fetch narinfo"
            else
                echo "❌ Rust dependencies are not cached"
                echo "To cache dependencies, run 'just build-deps'"
            fi
        else
            echo "❌ Could not determine path for Rust dependencies"
            echo "Run 'just build-deps' to build and cache dependencies"
            exit 1
        fi
    else
        echo "❌ Rust dependencies package cannot be built"
        echo "Run 'just build-deps' to build and cache dependencies"
        exit 1
    fi 

# Create a stable hash-based build that doesn't depend on Git state
build-hash:
    @echo "Building with content-addressable hash-based approach (Git-state independent)"
    @echo "This build will create a cache entry based on dependency inputs, not Git hash"
    nix build -L --no-warn-dirty .#rustDeps
    @echo "✅ Dependencies built and cached with stable hash"
    nix build -L --no-warn-dirty
    @echo "✅ App built using cached dependencies"

# Build only rust dependencies with the hash-based caching approach
build-deps-hash:
    @echo "Building Rust dependencies with content-addressable hash approach (Git-state independent)"
    nix build -L --no-warn-dirty --show-trace .#rustDeps
    @if [ $? -eq 0 ]; then echo "✅ Rust dependencies built and cached successfully"; else echo "❌ Failed to build Rust dependencies"; exit 1; fi 
# Local cache settings
local-cache := "http://localhost:5000"
local-cache-key := "dell-62S6063:F1R/DQVxh0R0YUBXEdVClqDsddJ5VLWVYzPrHC9mmqc="
nixos-cache-key := "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY="
nix-community-key := "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
devenv-key := "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw="
substituters := "https://cache.nixos.org/ " + local-cache + " https://nix-community.cachix.org https://devenv.cachix.org"
trusted-keys := nixos-cache-key + " " + local-cache-key + " " + nix-community-key + " " + devenv-key

# Default recipe
default:
    @just --list

# Build with local cache
build *args:
    @echo "Building with local cache at {{local-cache}}"
    nix build --option substituters "{{substituters}}" --option trusted-public-keys "{{trusted-keys}}" -L -v --log-format bar-with-logs {{args}}

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
    @echo "\nChecking cache contents:"
    nix path-info --store {{local-cache}} --all | head -n 20

# Show all available substituters and keys
show-nix-config:
    @echo "Current Nix configuration:"
    nix show-config | grep -E 'substituter|key'

# Standard Cargo commands
c-build:
    cargo build

c-run:
    cargo run

c-watch:
    RUSTFLAGS='-A warnings' cargo watch -x run 
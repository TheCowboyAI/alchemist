{ pkgs ? import <nixpkgs> {} }:

let
  # Repository configurations
  repos = [
    {
      name = "cim-component";
      description = "Core component definitions for the Composable Information Machine";
    }
    {
      name = "cim-compose";
      description = "Composition utilities and helpers for CIM";
    }
    {
      name = "cim-conceptual-core";
      description = "Conceptual space core functionality and category theory implementations";
    }
    {
      name = "cim-core-domain";
      description = "Core domain models and abstractions";
    }
    {
      name = "cim-domain";
      description = "Domain implementation with aggregates, events, and commands";
    }
    {
      name = "cim-identity-context";
      description = "Identity bounded context for person and organization management";
    }
    {
      name = "cim-infrastructure";
      description = "Infrastructure layer with NATS integration and persistence";
    }
    {
      name = "cim-subject";
      description = "Subject management and routing utilities";
    }
    {
      name = "cim-domain-bevy";
      description = "Bevy-based visualization components for CIM";
    }
  ];

  # Script to create repositories
  createRepoScript = pkgs.writeShellScriptBin "create-cim-repos" ''
    #!/usr/bin/env bash
    set -euo pipefail

    # Color codes
    GREEN='\033[0;32m'
    RED='\033[0;31m'
    YELLOW='\033[1;33m'
    NC='\033[0m'

    print_status() {
        echo -e "''${GREEN}[INFO]''${NC} $1"
    }

    print_error() {
        echo -e "''${RED}[ERROR]''${NC} $1"
    }

    print_warning() {
        echo -e "''${YELLOW}[WARNING]''${NC} $1"
    }

    # Check if gh CLI is available
    if ! command -v gh &> /dev/null; then
        print_error "GitHub CLI (gh) is not installed. Please install it first."
        exit 1
    fi

    # Check if authenticated
    if ! gh auth status &> /dev/null; then
        print_error "Not authenticated with GitHub. Please run: gh auth login"
        exit 1
    fi

    print_status "Creating GitHub repositories for CIM submodules..."

    ${pkgs.lib.concatMapStringsSep "\n" (repo: ''
      print_status "Creating repository: ${repo.name}"

      # Check if repo already exists
      if gh repo view "TheCowboyAI/${repo.name}" &> /dev/null; then
          print_warning "Repository TheCowboyAI/${repo.name} already exists, skipping..."
      else
          gh repo create "TheCowboyAI/${repo.name}" \
              --public \
              --description "${repo.description}" \
              --confirm

          print_status "Successfully created TheCowboyAI/${repo.name}"
      fi

      echo ""
    '') repos}

    print_status "All repositories have been processed!"
  '';

in
pkgs.mkShell {
  buildInputs = with pkgs; [
    gh  # GitHub CLI
    git
    createRepoScript
  ];

  shellHook = ''
    echo "GitHub repository creation environment loaded."
    echo "Run 'create-cim-repos' to create all CIM repositories on GitHub."
    echo ""
    echo "Make sure you're authenticated with GitHub CLI:"
    echo "  gh auth status"
    echo ""
    echo "If not authenticated, run:"
    echo "  gh auth login"
  '';
}

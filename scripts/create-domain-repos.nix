{ pkgs ? import <nixpkgs> {} }:

let
  # Domain repository configurations
  domainRepos = [
    {
      name = "cim-domain-person";
      description = "Person/People domain for the Composable Information Machine";
    }
    {
      name = "cim-domain-organization";
      description = "Organization domain for the Composable Information Machine";
    }
    {
      name = "cim-domain-agent";
      description = "Agent domain for the Composable Information Machine";
    }
    {
      name = "cim-domain-policy";
      description = "Policy domain for the Composable Information Machine";
    }
    {
      name = "cim-domain-document";
      description = "Document domain for the Composable Information Machine";
    }
    {
      name = "cim-domain-workflow";
      description = "Workflow domain for the Composable Information Machine";
    }
  ];

  # Script to create repositories
  createReposScript = pkgs.writeShellScriptBin "create-domain-repos" ''
    set -euo pipefail

    echo "Creating CIM Domain Repositories"
    echo "================================"
    echo ""

    # Check if gh is authenticated
    if ! ${pkgs.gh}/bin/gh auth status &>/dev/null; then
      echo "Error: GitHub CLI not authenticated"
      echo "Run: gh auth login"
      exit 1
    fi

    # Create each repository
    ${pkgs.lib.concatMapStringsSep "\n" (repo: ''
      echo "Creating ${repo.name}..."
      if ${pkgs.gh}/bin/gh repo view TheCowboyAI/${repo.name} &>/dev/null; then
        echo "  Repository ${repo.name} already exists, skipping..."
      else
        ${pkgs.gh}/bin/gh repo create TheCowboyAI/${repo.name} \
          --public \
          --description "${repo.description}" \
          --license MIT \
          || echo "  Failed to create ${repo.name}"
      fi
      echo ""
    '') domainRepos}

    echo "Repository creation complete!"
    echo ""
    echo "Next steps:"
    echo "1. Run the extract-domain-submodules.sh script for each domain"
    echo "2. Extract and organize the code from cim-domain"
    echo "3. Add each as a submodule to the main project"
  '';

in pkgs.mkShell {
  buildInputs = with pkgs; [
    gh
    git
    createReposScript
  ];

  shellHook = ''
    echo "Domain Repository Creation Environment"
    echo "====================================="
    echo ""
    echo "Available commands:"
    echo "  create-domain-repos - Create all domain repositories on GitHub"
    echo ""
    echo "Make sure you're authenticated with GitHub CLI:"
    echo "  gh auth status"
    echo ""
  '';
}

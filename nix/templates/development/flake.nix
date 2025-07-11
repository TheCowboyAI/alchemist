{
  description = "Alchemist Development Deployment Template";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in {
        nixosConfigurations.development = nixpkgs.lib.nixosSystem {
          inherit system;
          modules = [
            ./configuration.nix
            ../../modules/alchemist-service.nix
            ../../modules/alchemist-agent.nix
            ../../modules/nats-mesh.nix
          ];
        };

        apps.deploy = {
          type = "app";
          program = "${pkgs.writeShellScript "deploy-dev" ''
            #!${pkgs.bash}/bin/bash
            set -e
            
            echo "🚀 Deploying Alchemist Development Environment"
            echo "============================================="
            
            # Check if running on NixOS
            if [ -f /etc/nixos/configuration.nix ]; then
              echo "Detected NixOS system, using nixos-rebuild..."
              sudo nixos-rebuild switch --flake .#development
            else
              echo "Non-NixOS system detected, using home-manager or nix-darwin..."
              # Add support for other platforms as needed
              echo "Platform-specific deployment not yet implemented"
              exit 1
            fi
            
            echo ""
            echo "✅ Development deployment complete!"
            echo "   NATS: nats://localhost:4222"
            echo "   API: http://localhost:8080"
            echo "   Monitor: http://localhost:8222"
          ''}";
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            nats-server
            natscli
            curl
            jq
          ];
          
          shellHook = ''
            echo "Alchemist Development Environment"
            echo "================================="
            echo "Available commands:"
            echo "  nix run .#deploy - Deploy the development environment"
            echo "  nats-server - Start NATS server"
            echo "  nats sub '>' - Subscribe to all NATS messages"
            echo ""
          '';
        };
      });
}
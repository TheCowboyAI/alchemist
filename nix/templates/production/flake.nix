{
  description = "Alchemist Production Deployment Template";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    sops-nix.url = "github:Mic92/sops-nix";
  };

  outputs = { self, nixpkgs, flake-utils, sops-nix }:
    let
      system = "x86_64-linux";
      
      # Production node configurations
      mkNode = nodeName: nodeConfig: nixpkgs.lib.nixosSystem {
        inherit system;
        modules = [
          sops-nix.nixosModules.sops
          ./nodes/${nodeName}.nix
          ../../modules/alchemist-service.nix
          ../../modules/alchemist-agent.nix
          ../../modules/nats-mesh.nix
        ] ++ (nodeConfig.extraModules or []);
        
        specialArgs = {
          inherit nodeName;
          clusterNodes = [ "control1" "control2" "control3" ];
          workerNodes = [ "worker1" "worker2" "worker3" ];
        };
      };
      
    in {
      nixosConfigurations = {
        # Control plane nodes (3 for HA)
        control1 = mkNode "control1" {
          extraModules = [ ./control-plane.nix ];
        };
        control2 = mkNode "control2" {
          extraModules = [ ./control-plane.nix ];
        };
        control3 = mkNode "control3" {
          extraModules = [ ./control-plane.nix ];
        };
        
        # Worker nodes
        worker1 = mkNode "worker1" {
          extraModules = [ ./worker-node.nix ];
        };
        worker2 = mkNode "worker2" {
          extraModules = [ ./worker-node.nix ];
        };
        worker3 = mkNode "worker3" {
          extraModules = [ ./worker-node.nix ];
        };
      };

      apps.${system} = {
        # Deploy all nodes
        deploy-all = {
          type = "app";
          program = "${nixpkgs.legacyPackages.${system}.writeShellScript "deploy-all" ''
            #!${nixpkgs.legacyPackages.${system}.bash}/bin/bash
            set -e
            
            echo "🚀 Deploying Alchemist Production Cluster"
            echo "========================================"
            
            # Deploy control plane nodes first
            for node in control1 control2 control3; do
              echo "Deploying $node..."
              nixos-rebuild switch --flake .#$node --target-host $node.prod.example.com
            done
            
            # Wait for control plane to stabilize
            echo "Waiting for control plane to stabilize..."
            sleep 30
            
            # Deploy worker nodes
            for node in worker1 worker2 worker3; do
              echo "Deploying $node..."
              nixos-rebuild switch --flake .#$node --target-host $node.prod.example.com
            done
            
            echo ""
            echo "✅ Production deployment complete!"
            echo "   Control plane: control[1-3].prod.example.com"
            echo "   Workers: worker[1-3].prod.example.com"
            echo "   Load balancer: https://alchemist.prod.example.com"
          ''}";
        };
        
        # Rolling update
        rolling-update = {
          type = "app";
          program = "${nixpkgs.legacyPackages.${system}.writeShellScript "rolling-update" ''
            #!${nixpkgs.legacyPackages.${system}.bash}/bin/bash
            set -e
            
            echo "🔄 Starting rolling update..."
            
            # Update one node at a time with health checks
            for node in control1 control2 control3 worker1 worker2 worker3; do
              echo "Updating $node..."
              
              # Drain node if it's a worker
              if [[ $node == worker* ]]; then
                echo "Draining $node..."
                ssh $node.prod.example.com "systemctl stop alchemist-agent-*"
                sleep 10
              fi
              
              # Apply update
              nixos-rebuild switch --flake .#$node --target-host $node.prod.example.com
              
              # Wait for health check
              echo "Waiting for $node to become healthy..."
              for i in {1..30}; do
                if curl -sf http://$node.prod.example.com:8080/health > /dev/null; then
                  echo "$node is healthy"
                  break
                fi
                sleep 10
              done
              
              # Pause between nodes
              sleep 30
            done
            
            echo "✅ Rolling update complete!"
          ''}";
        };
        
        # Health check
        health-check = {
          type = "app";
          program = "${nixpkgs.legacyPackages.${system}.writeShellScript "health-check" ''
            #!${nixpkgs.legacyPackages.${system}.bash}/bin/bash
            
            echo "🏥 Checking cluster health..."
            echo "============================"
            
            # Check control plane
            echo "Control Plane:"
            for node in control1 control2 control3; do
              printf "  $node: "
              if curl -sf http://$node.prod.example.com:8080/health > /dev/null; then
                echo "✅ Healthy"
              else
                echo "❌ Unhealthy"
              fi
            done
            
            # Check workers
            echo ""
            echo "Worker Nodes:"
            for node in worker1 worker2 worker3; do
              printf "  $node: "
              if ssh $node.prod.example.com "systemctl is-active alchemist-agent-worker" > /dev/null; then
                echo "✅ Active"
              else
                echo "❌ Inactive"
              fi
            done
            
            # Check NATS cluster
            echo ""
            echo "NATS Cluster:"
            ${nixpkgs.legacyPackages.${system}.natscli}/bin/nats --server=nats://control1.prod.example.com:4222 server list
          ''}";
        };
      };
    };
}
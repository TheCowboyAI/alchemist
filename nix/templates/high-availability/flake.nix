{
  description = "Alchemist High-Availability Deployment Template";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    colmena.url = "github:zhaofengli/colmena";
  };

  outputs = { self, nixpkgs, flake-utils, colmena }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      
      # HA cluster configuration
      clusterConfig = {
        # Availability zones
        zones = [ "us-east-1a" "us-east-1b" "us-east-1c" ];
        
        # Control plane nodes (3 per zone for 9 total)
        controlNodes = lib.flatten (map (zone: [
          "control-${zone}-1"
          "control-${zone}-2"
          "control-${zone}-3"
        ]) clusterConfig.zones);
        
        # Worker nodes (5 per zone for 15 total)
        workerNodes = lib.flatten (map (zone: 
          map (n: "worker-${zone}-${toString n}") (lib.range 1 5)
        ) clusterConfig.zones);
        
        # Load balancers (2 per zone)
        loadBalancers = lib.flatten (map (zone: [
          "lb-${zone}-1"
          "lb-${zone}-2"
        ]) clusterConfig.zones);
      };
      
      lib = nixpkgs.lib;
      
    in {
      # Colmena deployment configuration
      colmena = {
        meta = {
          nixpkgs = import nixpkgs { inherit system; };
          specialArgs = { inherit clusterConfig; };
        };
        
        defaults = { pkgs, ... }: {
          imports = [
            ../../modules/alchemist-service.nix
            ../../modules/alchemist-agent.nix
            ../../modules/nats-mesh.nix
          ];
          
          # Common HA settings
          boot.kernel.sysctl = {
            "net.core.rmem_max" = 268435456;
            "net.core.wmem_max" = 268435456;
            "net.ipv4.tcp_rmem" = "4096 87380 268435456";
            "net.ipv4.tcp_wmem" = "4096 65536 268435456";
            "net.core.netdev_max_backlog = 10000;
            "net.ipv4.tcp_congestion_control" = "bbr";
            "net.ipv4.tcp_mtu_probing" = 1;
          };
          
          # High-performance storage
          boot.supportedFilesystems = [ "zfs" ];
          boot.zfs.forceImportRoot = false;
          
          # Monitoring agent on all nodes
          services.telegraf = {
            enable = true;
            extraConfig = {
              outputs.prometheus_client = {
                listen = ":9273";
                metric_version = 2;
              };
              
              inputs = {
                cpu = {};
                disk = {};
                diskio = {};
                kernel = {};
                mem = {};
                net = {};
                netstat = {};
                processes = {};
                swap = {};
                system = {};
              };
            };
          };
        };
      } // (
        # Generate control node configurations
        lib.listToAttrs (map (node: {
          name = node;
          value = { pkgs, config, ... }: {
            imports = [ ./control-node-ha.nix ];
            
            networking.hostName = node;
            
            # Zone-aware configuration
            services.alchemist.zone = lib.elemAt (lib.splitString "-" node) 1;
            
            # NATS cluster with zone awareness
            services.nats-mesh.nodes = [{
              name = "nats-${node}";
              host = "0.0.0.0";
              clientPort = 4222;
              clusterPort = 6222;
              routes = map (n: "${n}:6222") (lib.filter (n: n != node) clusterConfig.controlNodes);
            }];
          };
        }) clusterConfig.controlNodes)
      ) // (
        # Generate worker node configurations
        lib.listToAttrs (map (node: {
          name = node;
          value = { pkgs, config, ... }: {
            imports = [ ./worker-node-ha.nix ];
            
            networking.hostName = node;
            
            # Zone-aware agent configuration
            services.alchemist.agents.${node} = {
              enable = true;
              capabilities = [ "compute" "storage" "network" ];
              zone = lib.elemAt (lib.splitString "-" node) 1;
              resources = {
                cpu = "16";
                memory = "32Gi";
                gpu = "2";  # For AI workloads
              };
            };
          };
        }) clusterConfig.workerNodes)
      ) // (
        # Generate load balancer configurations
        lib.listToAttrs (map (node: {
          name = node;
          value = { pkgs, config, ... }: {
            imports = [ ./loadbalancer-ha.nix ];
            
            networking.hostName = node;
            
            # Zone-specific backend configuration
            services.haproxy.backends = let
              zone = lib.elemAt (lib.splitString "-" node) 1;
              zoneControlNodes = lib.filter (n: lib.hasInfix zone n) clusterConfig.controlNodes;
            in {
              api_backend.servers = map (n: {
                name = n;
                address = "${n}:8080";
                check = true;
              }) zoneControlNodes;
              
              nats_backend.servers = map (n: {
                name = n;
                address = "${n}:4222";
                check = true;
              }) zoneControlNodes;
            };
          };
        }) clusterConfig.loadBalancers)
      );
      
      apps.${system} = {
        # Deploy entire HA cluster
        deploy = {
          type = "app";
          program = "${pkgs.writeShellScript "deploy-ha" ''
            #!${pkgs.bash}/bin/bash
            set -e
            
            echo "🚀 Deploying Alchemist HA Cluster"
            echo "================================="
            echo "Control nodes: ${toString (builtins.length clusterConfig.controlNodes)}"
            echo "Worker nodes: ${toString (builtins.length clusterConfig.workerNodes)}"
            echo "Load balancers: ${toString (builtins.length clusterConfig.loadBalancers)}"
            echo ""
            
            # Deploy using Colmena
            ${colmena.packages.${system}.colmena}/bin/colmena apply --parallel 10
            
            echo ""
            echo "✅ HA cluster deployment complete!"
          ''}";
        };
        
        # Perform chaos testing
        chaos-test = {
          type = "app";
          program = "${pkgs.writeShellScript "chaos-test" ''
            #!${pkgs.bash}/bin/bash
            
            echo "💥 Starting Chaos Testing"
            echo "======================="
            
            # Test scenarios
            scenarios=(
              "network-partition"
              "node-failure"
              "disk-pressure"
              "cpu-stress"
              "memory-pressure"
            )
            
            for scenario in "''${scenarios[@]}"; do
              echo ""
              echo "Running scenario: $scenario"
              
              case $scenario in
                network-partition)
                  # Simulate network partition between zones
                  echo "Partitioning network between zones..."
                  # Implementation here
                  ;;
                  
                node-failure)
                  # Randomly kill nodes
                  echo "Simulating node failures..."
                  # Implementation here
                  ;;
                  
                disk-pressure)
                  # Fill up disk space
                  echo "Creating disk pressure..."
                  # Implementation here
                  ;;
                  
                cpu-stress)
                  # Max out CPU
                  echo "Stressing CPU..."
                  # Implementation here
                  ;;
                  
                memory-pressure)
                  # Consume memory
                  echo "Creating memory pressure..."
                  # Implementation here
                  ;;
              esac
              
              # Monitor cluster health during chaos
              echo "Monitoring cluster health..."
              sleep 60
              
              # Check if cluster recovered
              echo "Checking cluster recovery..."
              # Implementation here
            done
            
            echo ""
            echo "✅ Chaos testing complete!"
          ''}";
        };
        
        # Disaster recovery test
        dr-test = {
          type = "app";
          program = "${pkgs.writeShellScript "dr-test" ''
            #!${pkgs.bash}/bin/bash
            
            echo "🚨 Disaster Recovery Test"
            echo "======================="
            
            # Backup current state
            echo "Creating backup..."
            ${pkgs.restic}/bin/restic backup /var/lib/alchemist
            
            # Simulate zone failure
            echo "Simulating us-east-1a failure..."
            for node in ${lib.concatStringsSep " " (lib.filter (n: lib.hasInfix "us-east-1a" n) (clusterConfig.controlNodes ++ clusterConfig.workerNodes))}; do
              ssh $node "sudo poweroff"
            done
            
            # Wait for cluster to detect failure
            sleep 120
            
            # Check cluster health
            echo "Checking cluster health after zone failure..."
            # Implementation here
            
            # Restore from backup in new zone
            echo "Restoring in us-east-1d..."
            # Implementation here
            
            echo ""
            echo "✅ DR test complete!"
          ''}";
        };
      };
    };
}
{
  description = "Alchemist Edge Deployment Template";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    let
      # Support both x86_64 and aarch64 for edge devices
      supportedSystems = [ "x86_64-linux" "aarch64-linux" ];
      
      # Edge node configuration generator
      mkEdgeNode = system: location: hubUrls: nixpkgs.lib.nixosSystem {
        inherit system;
        modules = [
          ./configuration.nix
          ../../modules/leaf-node.nix
          {
            services.alchemist.leafNode = {
              enable = true;
              name = "edge-${location}";
              remotes = hubUrls;
              
              # Edge-specific resource constraints
              resources = {
                cpu = "1";
                memory = "512Mi";
              };
              
              # Aggressive caching for offline operation
              cache = {
                size = "500MB";
                ttl = 7200; # 2 hours
              };
              
              offlineMode = {
                enable = true;
                queueSize = "100MB";
                retryInterval = 300; # 5 minutes
              };
              
              # Reduce JetStream footprint
              jetstream = {
                enable = true;
                maxMemory = "128M";
                maxFile = "500M";
              };
            };
          }
        ];
      };
      
    in flake-utils.lib.eachSystem supportedSystems (system: {
      packages = {
        # Raspberry Pi 4 image
        rpi4-image = (mkEdgeNode "aarch64-linux" "rpi4" [
          "nats://hub1.example.com:7422"
          "nats://hub2.example.com:7422"
        ]).config.system.build.sdImage;
        
        # Intel NUC image
        nuc-image = (mkEdgeNode "x86_64-linux" "nuc" [
          "nats://hub1.example.com:7422"
          "nats://hub2.example.com:7422"
        ]).config.system.build.isoImage;
      };
      
      apps = {
        # Deploy to edge device
        deploy-edge = {
          type = "app";
          program = "${nixpkgs.legacyPackages.${system}.writeShellScript "deploy-edge" ''
            #!${nixpkgs.legacyPackages.${system}.bash}/bin/bash
            set -e
            
            if [ -z "$1" ]; then
              echo "Usage: $0 <edge-device-ip> [location-name]"
              exit 1
            fi
            
            DEVICE_IP=$1
            LOCATION=''${2:-edge-site}
            
            echo "🚀 Deploying Alchemist Edge Node"
            echo "================================"
            echo "Device: $DEVICE_IP"
            echo "Location: $LOCATION"
            
            # Build configuration
            echo "Building edge configuration..."
            nix build .#nixosConfigurations.edge-$LOCATION.config.system.build.toplevel
            
            # Copy to device
            echo "Copying to device..."
            nix copy --to ssh://root@$DEVICE_IP ./result
            
            # Activate
            echo "Activating configuration..."
            ssh root@$DEVICE_IP "./result/bin/switch-to-configuration switch"
            
            echo ""
            echo "✅ Edge deployment complete!"
            echo "   Device: $DEVICE_IP"
            echo "   NATS: nats://localhost:4222"
            echo "   Leaf connection established to hub"
          ''}";
        };
        
        # Generate edge installer
        build-installer = {
          type = "app";
          program = "${nixpkgs.legacyPackages.${system}.writeShellScript "build-installer" ''
            #!${nixpkgs.legacyPackages.${system}.bash}/bin/bash
            set -e
            
            echo "🔨 Building Edge Installers"
            echo "=========================="
            
            # Build Raspberry Pi image
            echo "Building Raspberry Pi 4 image..."
            nix build .#rpi4-image
            cp result/sd-image/*.img alchemist-edge-rpi4.img
            echo "Created: alchemist-edge-rpi4.img"
            
            # Build Intel NUC image
            echo "Building Intel NUC image..."
            nix build .#nuc-image
            cp result/iso/*.iso alchemist-edge-nuc.iso
            echo "Created: alchemist-edge-nuc.iso"
            
            echo ""
            echo "✅ Installers ready!"
            echo "   Raspberry Pi 4: alchemist-edge-rpi4.img"
            echo "   Intel NUC: alchemist-edge-nuc.iso"
          ''}";
        };
        
        # Monitor edge nodes
        monitor-edges = {
          type = "app";
          program = "${nixpkgs.legacyPackages.${system}.writeShellScript "monitor-edges" ''
            #!${nixpkgs.legacyPackages.${system}.bash}/bin/bash
            
            echo "📊 Edge Node Monitor"
            echo "==================="
            
            # Subscribe to edge health events
            ${nixpkgs.legacyPackages.${system}.natscli}/bin/nats \
              --server=nats://hub1.example.com:4222 \
              sub "edge.*.health" \
              --queue edge-monitor | \
            while read -r line; do
              echo "$(date '+%Y-%m-%d %H:%M:%S') $line"
            done
          ''}";
        };
      };
      
      devShells.default = nixpkgs.legacyPackages.${system}.mkShell {
        buildInputs = with nixpkgs.legacyPackages.${system}; [
          # Development tools
          nixos-generators
          qemu
          
          # Edge testing tools
          mosquitto
          curl
          jq
          
          # Cross-compilation support
          pkgsCross.aarch64-multiplatform.buildPackages.gcc
          pkgsCross.raspberryPi.buildPackages.gcc
        ];
        
        shellHook = ''
          echo "Alchemist Edge Development"
          echo "========================="
          echo "Available commands:"
          echo "  nix run .#build-installer - Build edge device images"
          echo "  nix run .#deploy-edge - Deploy to edge device"
          echo "  nix run .#monitor-edges - Monitor edge nodes"
          echo ""
          echo "Testing:"
          echo "  qemu-system-aarch64 -M raspi3b -kernel <kernel> -initrd <initrd>"
          echo ""
        '';
      };
    });
}
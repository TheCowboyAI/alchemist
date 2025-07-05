# CIM Cluster Setup Guide

## Overview

This guide walks through setting up a production CIM cluster with both Linux (NVIDIA GPU) and macOS (Apple Silicon) nodes connected via NATS leaf nodes.

## Prerequisites

### Hardware Requirements
- **Linux Nodes**: x86_64 servers with NVIDIA RTX 3080 Ti or better
- **Mac Studio Nodes**: Apple M3 Ultra with 192GB+ unified memory
- **Network**: 10Gbps Ethernet backbone with low latency (<1ms between nodes)

### Software Requirements
- NixOS 23.11 or later (Linux nodes)
- macOS Sonoma 14.0+ (Mac Studio nodes)
- Nix package manager 2.18+
- Git with submodule support

## Step 1: NATS Cluster Setup

### 1.1 Deploy NATS Cloud Cluster

First, set up the central NATS cluster that will coordinate all leaf nodes:

```nix
# nats-cluster.nix
{ config, pkgs, ... }:
{
  services.nats = {
    enable = true;
    serverName = "nats-cluster-1";
    
    cluster = {
      name = "cim-cluster";
      listen = "0.0.0.0:6222";
      
      routes = [
        "nats://nats-cluster-2:6222"
        "nats://nats-cluster-3:6222"
      ];
    };
    
    leafnodes = {
      listen = "0.0.0.0:7422";
      
      authorization = {
        users = [
          {
            user = "leaf-node";
            password = "$2a$11$..."; # bcrypt hash
            account = "CIM_LEAF_NODES";
          }
        ];
      };
    };
    
    jetstream = {
      enable = true;
      storeDir = "/var/lib/nats/jetstream";
      maxMemory = "16G";
      maxStore = "1T";
    };
  };
}
```

Deploy the cluster:
```bash
nixos-anywhere --flake .#nats-cluster-1 root@cluster1.cim.internal
nixos-anywhere --flake .#nats-cluster-2 root@cluster2.cim.internal
nixos-anywhere --flake .#nats-cluster-3 root@cluster3.cim.internal
```

### 1.2 Verify Cluster Health

```bash
# Check cluster status
nats server list

# Verify JetStream
nats stream list

# Test pub/sub
nats pub test.subject "Hello CIM" --count=10
nats sub test.subject
```

## Step 2: Linux Node Setup (NVIDIA GPU)

### 2.1 Prepare Hardware

1. Install NixOS using the CIM ISO:
```bash
# Build custom ISO with GPU drivers
nix build .#nixosConfigurations.cim-gpu-node.config.system.build.isoImage

# Write to USB
dd if=result/iso/*.iso of=/dev/sdX bs=4M status=progress
```

2. Boot from USB and install:
```bash
# During installation, ensure:
# - UEFI boot enabled
# - Secure Boot disabled (for NVIDIA drivers)
# - IOMMU enabled in BIOS (for GPU passthrough)
```

### 2.2 Deploy CIM Configuration

```nix
# cim-gpu-node.nix
{ config, pkgs, lib, ... }:
{
  imports = [
    ./hardware-configuration.nix
    ./cim-services.nix
  ];

  # GPU Configuration
  hardware.nvidia = {
    modesetting.enable = true;
    powerManagement.enable = true;
    powerManagement.finegrained = false;
    
    # Use production drivers
    package = config.boot.kernelPackages.nvidiaPackages.production;
  };

  # CUDA Setup
  environment.systemPackages = with pkgs; [
    cudaPackages.cudatoolkit
    cudaPackages.cudnn
    nvidia-docker
  ];

  # CIM Agent Service
  systemd.services.cim-agent = {
    description = "CIM Agent with GPU Support";
    after = [ "network.target" "nats.service" ];
    wantedBy = [ "multi-user.target" ];
    
    environment = {
      RUST_LOG = "info";
      CUDA_VISIBLE_DEVICES = "0";
      CIM_GPU_MEMORY_FRACTION = "0.9";
    };
    
    serviceConfig = {
      Type = "notify";
      ExecStart = "${pkgs.cim}/bin/cim-agent";
      Restart = "always";
      RestartSec = "10s";
      
      # GPU access
      PrivateDevices = false;
      DeviceAllow = [
        "/dev/nvidia0 rw"
        "/dev/nvidiactl rw"
        "/dev/nvidia-modeset rw"
      ];
    };
  };
}
```

Deploy:
```bash
nixos-anywhere --flake .#cim-gpu-node-1 root@gpu1.cim.internal
```

## Step 3: Mac Studio Setup (Apple Silicon)

### 3.1 Install nix-darwin

```bash
# On the Mac Studio
sh <(curl -L https://github.com/LnL7/nix-darwin/releases/latest/download/installer)

# Add to shell profile
echo 'source /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh' >> ~/.zshrc
```

### 3.2 Configure CIM Services

```nix
# flake.nix (darwin configuration)
{
  darwinConfigurations."cim-mac-studio-1" = darwin.lib.darwinSystem {
    system = "aarch64-darwin";
    modules = [
      ./darwin/cim-mac-studio.nix
      {
        # CIM-specific configuration
        services.cim = {
          enable = true;
          nodeType = "compute";
          
          nats = {
            leafNode = {
              remoteUrl = "nats://cluster.cim.internal:7422";
              credentials = ./secrets/leaf-node.creds;
            };
          };
          
          agent = {
            enable = true;
            gpuBackend = "metal";
            
            models = [
              "llama-3.1-70b"
              "stable-diffusion-xl"
            ];
            
            resources = {
              maxMemoryGB = 200;  # Leave 56GB for system
              maxGPUCores = 70;   # Leave 6 cores for system
            };
          };
        };
      }
    ];
  };
}
```

Deploy:
```bash
darwin-rebuild switch --flake .#cim-mac-studio-1
```

### 3.3 Verify Metal Performance Shaders

```bash
# Check GPU availability
system_profiler SPDisplaysDataType

# Test Metal performance
xcrun metal -std=metal3.0 test.metal

# Monitor GPU usage
sudo powermetrics --samplers gpu_power -i1000
```

## Step 4: Network Configuration

### 4.1 PXE Boot Setup

For automated deployment of new nodes:

```nix
# pxe-server.nix
services.pixiecore = {
  enable = true;
  
  kernel = "${nixosInstaller}/kernel";
  initrd = "${nixosInstaller}/initrd";
  
  cmdline = [
    "init=${nixosInstaller}/init"
    "nixos.install.url=http://pxe.cim.internal/install.nix"
    "nixos.install.flake=github:cim/infrastructure#cim-gpu-node"
  ];
  
  dhcpRange = "10.0.100.100,10.0.100.200";
};
```

### 4.2 DHCP Configuration

```nix
services.dhcpd4 = {
  enable = true;
  interfaces = [ "eth0" ];
  
  extraConfig = ''
    subnet 10.0.0.0 netmask 255.255.0.0 {
      range 10.0.100.1 10.0.100.254;
      
      # GPU nodes
      host gpu-node-1 {
        hardware ethernet 00:11:22:33:44:55;
        fixed-address 10.0.1.10;
        option host-name "gpu-node-1";
      }
      
      # Mac Studios
      host mac-studio-1 {
        hardware ethernet aa:bb:cc:dd:ee:ff;
        fixed-address 10.0.2.10;
        option host-name "mac-studio-1";
      }
    }
  '';
};
```

## Step 5: Cluster Orchestration

### 5.1 Deploy All Nodes

```bash
#!/usr/bin/env bash
# deploy-cluster.sh

# Deploy NATS cluster
for i in {1..3}; do
  nixos-anywhere --flake .#nats-cluster-$i root@cluster$i.cim.internal &
done
wait

# Deploy GPU nodes
for i in {1..4}; do
  nixos-anywhere --flake .#cim-gpu-node-$i root@gpu$i.cim.internal &
done

# Deploy Mac Studios (run on each Mac)
for i in {1..2}; do
  ssh admin@mac-studio-$i.cim.internal \
    "darwin-rebuild switch --flake github:cim/infrastructure#cim-mac-studio-$i"
done

wait
echo "Cluster deployment complete!"
```

### 5.2 Health Monitoring

```nix
# monitoring.nix
services.prometheus = {
  enable = true;
  
  scrapeConfigs = [
    {
      job_name = "nats";
      static_configs = [{
        targets = [
          "cluster1:8222"
          "gpu1:8222"
          "mac-studio-1:8222"
        ];
      }];
    }
    {
      job_name = "nvidia_gpu";
      static_configs = [{
        targets = map (n: "gpu${toString n}:9835") (range 1 4);
      }];
    }
    {
      job_name = "metal_gpu";
      static_configs = [{
        targets = [ "mac-studio-1:9836" "mac-studio-2:9836" ];
      }];
    }
  ];
};
```

## Step 6: Testing and Validation

### 6.1 NATS Connectivity Test

```bash
# Test leaf node connections
nats server info --server nats://gpu1.cim.internal:4222

# Verify event flow
nats sub "cim.>" --server nats://cluster.cim.internal:4222

# Publish test event
nats pub cim.test "Hello from GPU node" --server nats://gpu1.cim.internal:4222
```

### 6.2 GPU Workload Test

```rust
// test-gpu-allocation.rs
use cim_domain::agent::{GpuResourceManager, ModelRequirements};

#[tokio::test]
async fn test_heterogeneous_gpu_allocation() {
    let mut gpu_manager = GpuResourceManager::from_cluster().await?;
    
    // Request large model (needs 24GB+ VRAM)
    let large_model = ModelRequirements {
        model_name: "llama-3.1-70b",
        vram_usage_gb: 40.0,
        compute_requirements: ComputeReqs::High,
    };
    
    // Should allocate on Mac Studio (256GB unified memory)
    let allocation = gpu_manager.allocate_gpu_for_agent(
        AgentId::new(),
        large_model,
    ).await?;
    
    match allocation.device {
        AllocatedDevice::Metal { .. } => {
            println!("✓ Large model allocated on Apple Silicon");
        }
        _ => panic!("Expected Metal allocation for large model"),
    }
    
    // Request batch processing (prefers CUDA)
    let batch_job = ModelRequirements {
        model_name: "stable-diffusion-xl",
        vram_usage_gb: 8.0,
        compute_requirements: ComputeReqs::BatchProcessing,
    };
    
    let batch_allocation = gpu_manager.allocate_gpu_for_agent(
        AgentId::new(),
        batch_job,
    ).await?;
    
    match batch_allocation.device {
        AllocatedDevice::Cuda { .. } => {
            println!("✓ Batch job allocated on NVIDIA GPU");
        }
        _ => panic!("Expected CUDA allocation for batch processing"),
    }
}
```

### 6.3 Performance Benchmarks

```bash
# Run CIM benchmarks across cluster
nix run .#cim-benchmarks -- \
  --nodes gpu1,gpu2,mac-studio-1 \
  --workload mixed \
  --duration 300 \
  --output benchmark-results.json
```

## Step 7: Production Operations

### 7.1 Rolling Updates

```bash
# Update a single node
./scripts/rolling-update.sh gpu-node-1

# Update all GPU nodes
./scripts/rolling-update.sh --group gpu-nodes

# Update with canary deployment
./scripts/rolling-update.sh --canary 10%
```

### 7.2 Backup and Recovery

```nix
# backup.nix
services.restic = {
  enable = true;
  
  backups.cim-state = {
    paths = [
      "/var/lib/nats/jetstream"
      "/var/lib/cim"
    ];
    
    repository = "s3:s3.amazonaws.com/cim-backups";
    timerConfig = {
      OnCalendar = "hourly";
    };
    
    pruneOpts = [
      "--keep-hourly 24"
      "--keep-daily 7"
      "--keep-weekly 4"
      "--keep-monthly 12"
    ];
  };
};
```

## Troubleshooting

### Common Issues

1. **NATS Leaf Node Connection Failed**
   ```bash
   # Check credentials
   nats context info
   
   # Verify network connectivity
   nc -zv cluster.cim.internal 7422
   
   # Check firewall rules
   sudo iptables -L -n | grep 7422
   ```

2. **GPU Not Available**
   ```bash
   # Linux/NVIDIA
   nvidia-smi
   lsmod | grep nvidia
   
   # macOS/Metal
   system_profiler SPDisplaysDataType
   ioreg -l | grep GPU
   ```

3. **Memory Pressure on Mac Studio**
   ```bash
   # Check memory stats
   vm_stat
   
   # Adjust CIM memory limits
   launchctl setenv CIM_MAX_MEMORY_GB 180
   ```

## Next Steps

- Set up monitoring dashboards in Grafana
- Configure alerting rules for cluster health
- Implement automated scaling policies
- Create disaster recovery procedures

For advanced configurations and optimizations, see the [Performance Tuning Guide](./performance-tuning.md). 
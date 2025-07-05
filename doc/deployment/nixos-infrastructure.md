# CIM NixOS Infrastructure Architecture

## Overview

The Composable Information Machine (CIM) uses a pure NixOS-based infrastructure that provides declarative, reproducible deployments without the complexity and overhead of container orchestration platforms. This approach aligns with CIM's philosophy of composable, event-driven systems.

## Hardware Requirements

### Leaf Node Specifications

CIM Leaf Nodes support two hardware configurations for AI agent workloads:

#### Option 1: x86_64 with NVIDIA GPU
- **CPU**: Minimum 16 cores (32 threads) - AMD EPYC or Intel Xeon recommended
- **RAM**: Minimum 64GB DDR4/DDR5 ECC
- **GPU**: NVIDIA RTX 3080 Ti or better (required)
  - Minimum 12GB VRAM
  - CUDA Compute Capability 8.6+
  - Supported models:
    - RTX 3080 Ti (12GB)
    - RTX 3090 (24GB)
    - RTX 4080 (16GB)
    - RTX 4090 (24GB)
    - A100 (40GB/80GB) for datacenter deployments
- **Storage**: 
  - 1TB NVMe SSD for OS and CIM services
  - 2TB+ NVMe SSD for NATS JetStream storage
  - Optional: Additional HDDs for long-term event storage
- **Network**: 10Gbps Ethernet minimum

#### Option 2: Apple Silicon (Mac Studio)
- **CPU**: Apple M3 Ultra or newer
  - M3 Ultra: 24-core CPU (16 performance + 8 efficiency)
  - M3 Max: 16-core CPU (12 performance + 4 efficiency)
- **RAM**: 192GB or 256GB unified memory
- **GPU**: Integrated Apple Silicon GPU
  - M3 Ultra: 76-core GPU
  - M3 Max: 40-core GPU
  - Hardware-accelerated ML with Neural Engine
- **Storage**:
  - 2TB+ internal SSD
  - External Thunderbolt 4 storage arrays supported
- **Network**: 10Gbps Ethernet built-in

### GPU Passthrough Configuration

For virtualized deployments, GPU passthrough is supported:

```nix
# GPU passthrough for VMs
virtualisation.kvmgt = {
  enable = true;
  vgpus = {
    "nvidia-rtx3080ti" = {
      uuid = "a297db4a-f4c2-11e6-90f6-d3b88d6c9525";
      # Reserve GPU for specific VM
    };
  };
};
```

## Core Infrastructure Components

### 1. NATS Topology

CIM operates on a distributed NATS architecture:

```
┌─────────────────────────────────────────────────┐
│            NATS Cloud Cluster                    │
│         (Multi-region backbone)                  │
└─────────────┬───────────────┬───────────────────┘
              │               │
    ┌─────────▼─────┐   ┌─────▼─────────┐
    │  Leaf Node 1  │   │  Leaf Node 2   │
    │  (Region A)   │   │  (Region B)    │
    └───────────────┘   └────────────────┘
           │                     │
    ┌──────▼────────┐    ┌──────▼────────┐
    │ CIM Instance  │    │ CIM Instance  │
    │   (NixOS)     │    │   (NixOS)     │
    └───────────────┘    └───────────────┘
```

### 2. NixOS Deployment Stack

- **disko**: Declarative disk partitioning
- **nixos-generators**: Create bootable images for various platforms
- **nixos-anywhere**: Remote deployment to bare metal or VMs
- **nix-topology**: Define and visualize network topology
- **deploy-rs**: Zero-downtime deployments with rollback

### 3. Network Boot Infrastructure

```nix
# PXE boot server configuration
services.pixiecore = {
  enable = true;
  kernel = "${nixos-installer}/kernel";
  initrd = "${nixos-installer}/initrd";
  cmdline = "init=${nixos-installer}/init";
  
  # DHCP integration
  dhcpMode = "proxy";
  listenAddr = "0.0.0.0";
};

# DHCP server with NixOS deployment support
services.dhcpd4 = {
  enable = true;
  extraConfig = ''
    subnet 10.0.0.0 netmask 255.255.255.0 {
      range 10.0.0.100 10.0.0.200;
      option routers 10.0.0.1;
      
      # PXE boot options
      next-server 10.0.0.1;
      filename "pxelinux.0";
      
      # Host-specific configurations
      host cim-node-1 {
        hardware ethernet 00:11:22:33:44:55;
        fixed-address 10.0.0.10;
        option host-name "cim-node-1";
      }
    }
  '';
};
```

## CIM Deployment Configuration

### 1. Base CIM Node Configuration

#### Linux/x86_64 Configuration

```nix
# cim-node.nix
{ config, pkgs, lib, ... }:
{
  imports = [
    ./hardware-configuration.nix
    ./cim-services.nix
  ];

  # GPU Configuration for AI Agents
  hardware.nvidia = {
    # Enable NVIDIA drivers
    package = config.boot.kernelPackages.nvidiaPackages.stable;
    modesetting.enable = true;
    
    # Enable CUDA support
    datacenter.enable = false;  # Set to true for datacenter GPUs
    powerManagement.enable = true;
    
    # Open kernel modules for better compatibility
    open = false;
    
    # Enable nvidia-settings
    nvidiaSettings = true;
  };

  # CUDA and GPU passthrough configuration
  hardware.opengl = {
    enable = true;
    driSupport = true;
    driSupport32Bit = true;
    
    extraPackages = with pkgs; [
      nvidia-vaapi-driver
      vaapiVdpau
      libvdpau-va-gl
    ];
  };

  # CUDA toolkit for AI workloads
  environment.systemPackages = with pkgs; [
    cudaPackages.cudatoolkit
    cudaPackages.cudnn
    cudaPackages.tensorrt
    nvidia-docker
  ];

  # GPU device permissions for containers/agents
  services.udev.extraRules = ''
    # NVIDIA GPU devices
    KERNEL=="nvidia", RUN+="${pkgs.coreutils}/bin/chmod 0666 /dev/nvidia*"
    KERNEL=="nvidia_uvm", RUN+="${pkgs.coreutils}/bin/chmod 0666 /dev/nvidia-uvm*"
    KERNEL=="nvidia_modeset", RUN+="${pkgs.coreutils}/bin/chmod 0666 /dev/nvidia-modeset*"
  '';

  # NATS Leaf Node Configuration
  services.nats = {
    enable = true;
    serverName = config.networking.hostName;
    
    leafnode = {
      remotes = [{
        url = "nats://cluster.cim.internal:7422";
        credentials = "/var/lib/nats/leaf.creds";
      }];
    };
    
    jetstream = {
      enable = true;
      storeDir = "/var/lib/nats/jetstream";
      maxMemory = "4G";
      maxStore = "100G";
    };
  };

  # CIM Services with GPU support
  services.cim = {
    enable = true;
    package = pkgs.ia;
    
    settings = {
      nats_url = "nats://localhost:4222";
      event_store = "jetstream";
      
      # GPU configuration for agents
      gpu = {
        enable = true;
        device = "/dev/nvidia0";
        cuda_version = "12.2";
        memory_fraction = 0.8;  # Reserve 80% of GPU memory for AI workloads
      };
      
      domains = {
        graph.enable = true;
        workflow.enable = true;
        agent = {
          enable = true;
          gpu_acceleration = true;
          max_gpu_agents = 4;  # Limit concurrent GPU-using agents
        };
        conceptualspaces = {
          enable = true;
          use_gpu_embeddings = true;
        };
      };
    };
  };

  # Monitoring with GPU metrics
  services.prometheus = {
    enable = true;
    exporters = {
      node.enable = true;
      nats.enable = true;
      nvidia = {
        enable = true;
        port = 9835;
      };
    };
  };

  # Ensure kernel modules are loaded
  boot.kernelModules = [ "nvidia" "nvidia_modeset" "nvidia_uvm" "nvidia_drm" ];
  boot.blacklistedKernelModules = [ "nouveau" ];
  
  # Set GPU performance mode
  boot.kernelParams = [ "nvidia-drm.modeset=1" ];
}
```

#### macOS/Apple Silicon Configuration (nix-darwin)

```nix
# cim-node-darwin.nix
{ config, pkgs, lib, ... }:
{
  imports = [
    ./cim-services-darwin.nix
  ];

  # Enable nix-darwin settings
  programs.zsh.enable = true;
  services.nix-daemon.enable = true;

  # System configuration
  system = {
    defaults = {
      NSGlobalDomain = {
        AppleShowAllExtensions = true;
        AppleEnableMouseSwipeNavigateWithScrolls = false;
        AppleEnableSwipeNavigateWithScrolls = false;
      };
      
      # Disable sleep for server operation
      loginwindow = {
        DisableConsoleAccess = true;
      };
    };
  };

  # Hardware utilization for ML
  environment.systemPackages = with pkgs; [
    # ML frameworks with Metal Performance Shaders support
    tensorflow-metal
    pytorch-mps
    jax-metal
    
    # Development tools
    rustup
    cmake
    pkg-config
    
    # Monitoring
    stats  # System monitor
    istatmenus
  ];

  # NATS Leaf Node Configuration
  launchd.daemons.nats = {
    script = ''
      ${pkgs.nats-server}/bin/nats-server \
        --config /etc/nats/leaf-node.conf
    '';
    
    serviceConfig = {
      KeepAlive = true;
      RunAtLoad = true;
      StandardOutPath = "/var/log/nats.log";
      StandardErrorPath = "/var/log/nats.error.log";
    };
  };

  # CIM Services for Darwin
  launchd.daemons.cim = {
    script = ''
      export RUST_LOG=info
      export CIM_CONFIG=/etc/cim/config.toml
      ${pkgs.ia}/bin/ia --daemon
    '';
    
    serviceConfig = {
      KeepAlive = true;
      RunAtLoad = true;
      EnvironmentVariables = {
        NATS_URL = "nats://localhost:4222";
        CIM_PLATFORM = "darwin-aarch64";
        
        # Metal Performance Shaders configuration
        METAL_DEVICE_WRAPPER_TYPE = "1";
        METAL_DEBUG_ERROR_MODE = "0";
        MLCompute_FORCE_USE_GPU = "1";
      };
    };
  };

  # Network configuration
  networking = {
    hostName = "cim-mac-studio";
    localHostName = "cim-mac-studio";
    
    # Firewall configuration
    firewall = {
      enable = true;
      allowedTCPPorts = [ 
        4222  # NATS
        6222  # NATS cluster
        8222  # NATS monitoring
        9090  # Prometheus
        3000  # Grafana
      ];
    };
  };

  # Resource limits for server operation
  launchd.daemons.sysctl = {
    script = ''
      # Increase system limits
      sysctl -w kern.maxfiles=65536
      sysctl -w kern.maxfilesperproc=65536
      sysctl -w net.inet.ip.portrange.first=1024
      sysctl -w net.inet.tcp.msl=1000
      
      # Optimize for server workloads
      sysctl -w kern.timer.coalescing_enabled=0
    '';
    
    serviceConfig = {
      RunAtLoad = true;
    };
  };

  # Monitoring with Prometheus node exporter
  launchd.daemons.node-exporter = {
    script = ''
      ${pkgs.prometheus-node-exporter}/bin/node_exporter \
        --collector.cpu \
        --collector.meminfo \
        --collector.diskstats \
        --collector.netdev \
        --collector.thermal
    '';
    
    serviceConfig = {
      KeepAlive = true;
      RunAtLoad = true;
    };
  };

  # Power management for 24/7 operation
  system.activationScripts.postActivation.text = ''
    # Prevent sleep
    pmset -a sleep 0
    pmset -a disksleep 0
    pmset -a displaysleep 0
    pmset -a powernap 0
    
    # Enable wake on network
    pmset -a womp 1
    
    # Set performance mode
    pmset -a highpowermode 1
    
    # Disable App Nap for server processes
    defaults write NSGlobalDomain NSAppSleepDisabled -bool YES
  '';
}
```

### 2. Network Topology Definition

```nix
# topology.nix
{
  network = {
    description = "CIM Production Network";
    
    # Define network segments
    networks = {
      management = {
        subnet = "10.0.0.0/24";
        vlan = 10;
      };
      
      cim-internal = {
        subnet = "10.1.0.0/24";
        vlan = 20;
      };
      
      nats-cluster = {
        subnet = "10.2.0.0/24";
        vlan = 30;
      };
    };
  };

  # Node definitions
  nodes = {
    cim-master-1 = {
      imports = [ ./cim-node.nix ];
      deployment.targetHost = "10.0.0.10";
      
      networking = {
        hostName = "cim-master-1";
        interfaces.eth0.ipv4.addresses = [{
          address = "10.1.0.10";
          prefixLength = 24;
        }];
      };
    };
    
    cim-worker-1 = {
      imports = [ ./cim-node.nix ];
      deployment.targetHost = "10.0.0.11";
      
      networking = {
        hostName = "cim-worker-1";
        interfaces.eth0.ipv4.addresses = [{
          address = "10.1.0.11";
          prefixLength = 24;
        }];
      };
    };
  };
}
```

### 3. Deployment Commands

#### Linux Deployment
```bash
# Generate ISO for new node
nix build .#nixosConfigurations.cim-node-1.config.system.build.isoImage

# Deploy to existing machine
nixos-anywhere --flake .#cim-node-1 root@10.0.0.10

# Update running system
deploy-rs .#cim-node-1 --skip-checks

# Visualize topology
nix run .#topology-visualization
```

#### macOS Deployment (nix-darwin)
```bash
# Install nix-darwin on Mac Studio
sh <(curl -L https://github.com/LnL7/nix-darwin/releases/latest/download/installer)

# Deploy CIM configuration
darwin-rebuild switch --flake .#cim-mac-studio

# Update running system
darwin-rebuild switch --flake .#cim-mac-studio --show-trace

# Verify Metal GPU availability
system_profiler SPDisplaysDataType | grep "Metal"

# Check unified memory usage
vm_stat | grep "Pages free"
```

## CIM Domain Integration

### 1. NATS Command Interface

CIM domains expose commands through NATS subjects:

```rust
// Deployment commands
pub enum DeploymentCommand {
    DeployNode {
        hostname: String,
        configuration: NixConfiguration,
        target_ip: IpAddr,
        gpu_requirements: Option<GpuRequirements>,
    },
    UpdateNode {
        hostname: String,
        configuration_changes: ConfigDelta,
    },
    ScaleService {
        service: ServiceName,
        replicas: u32,
    },
    DeployGpuAgent {
        agent_id: AgentId,
        model_requirements: ModelRequirements,
        target_node: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuRequirements {
    pub min_vram_gb: u32,
    pub cuda_compute_capability: f32,
    pub tensor_cores_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequirements {
    pub model_name: String,
    pub vram_usage_gb: f32,
    pub batch_size: u32,
    pub precision: Precision,  // FP16, INT8, etc.
}

// NATS subjects
const DEPLOY_SUBJECT: &str = "cim.deployment.deploy";
const UPDATE_SUBJECT: &str = "cim.deployment.update";
const SCALE_SUBJECT: &str = "cim.deployment.scale";
const GPU_AGENT_SUBJECT: &str = "cim.deployment.gpu-agent";
```

### 2. NixOS Integration Module

```nix
# cim-deployment-module.nix
{ config, lib, pkgs, ... }:
with lib;
let
  cfg = config.services.cim-deployment;
in {
  options.services.cim-deployment = {
    enable = mkEnableOption "CIM deployment service";
    
    natsUrl = mkOption {
      type = types.str;
      default = "nats://localhost:4222";
      description = "NATS server URL";
    };
    
    deploymentKey = mkOption {
      type = types.path;
      description = "SSH key for nixos-anywhere deployments";
    };
  };

  config = mkIf cfg.enable {
    systemd.services.cim-deployment = {
      description = "CIM Deployment Service";
      wantedBy = [ "multi-user.target" ];
      
      serviceConfig = {
        ExecStart = "${pkgs.cim-deployment}/bin/cim-deployment";
        Restart = "always";
        
        Environment = [
          "NATS_URL=${cfg.natsUrl}"
          "DEPLOY_KEY=${cfg.deploymentKey}"
        ];
      };
    };
  };
}
```

## Scaling Strategy

### 1. Horizontal Scaling

```nix
# Scale by adding nodes to topology
nodes = lib.genAttrs (map (i: "cim-worker-${toString i}") (lib.range 1 10)) (name: {
  imports = [ ./cim-node.nix ];
  deployment.targetHost = "10.0.0.${toString (10 + lib.toInt (lib.last (lib.splitString "-" name)))}";
});
```

### 2. Service Distribution

```rust
// CIM automatically distributes work across nodes
impl WorkDistribution {
    pub fn assign_work(&self, task: Task) -> NodeId {
        // Use NATS queue groups for automatic load balancing
        self.publish_to_queue_group("cim.work", &task)
    }
}
```

### 3. Auto-scaling Triggers

```nix
# Monitoring-based scaling decisions
services.cim-autoscaler = {
  enable = true;
  
  rules = [
    {
      metric = "cpu_usage";
      threshold = 80;
      action = "deploy_node";
      cooldown = "5m";
    }
    {
      metric = "event_queue_depth";
      threshold = 10000;
      action = "scale_workers";
      increment = 2;
    }
  ];
};
```

## Advantages Over Container Orchestration

1. **Simplicity**: No container runtime overhead, just native systemd/launchd services
2. **Reproducibility**: Entire system state defined in Nix expressions
3. **Performance**: Direct hardware access without virtualization layers
4. **Security**: Immutable infrastructure with cryptographic verification
5. **Integration**: Native service management (systemd on Linux, launchd on macOS)
6. **Efficiency**: Shared Nix store reduces disk usage across nodes

## Multi-Platform GPU Advantages

### Hardware Diversity
- **NVIDIA GPUs**: Industry-standard CUDA for existing ML models
- **Apple Silicon**: Superior performance-per-watt and unified memory architecture
- **Cost Optimization**: Choose hardware based on workload requirements

### Performance Characteristics
| Platform        | Strengths                        | Best For                                    |
| --------------- | -------------------------------- | ------------------------------------------- |
| NVIDIA RTX 4090 | Raw compute power, 24GB VRAM     | Large language models, batch processing     |
| Apple M3 Ultra  | 256GB unified memory, efficiency | Long-running agents, memory-intensive tasks |

### Unified Abstraction
CIM's GPU abstraction layer allows agents to run on either platform without code changes:
```rust
// Agent code remains the same
let embedding = agent.compute_embedding(&text).await?;

// CIM handles platform-specific implementation
match gpu_allocation.device {
    AllocatedDevice::Cuda { .. } => cuda_compute_embedding(&text),
    AllocatedDevice::Metal { .. } => metal_compute_embedding(&text),
}
```

## Monitoring and Observability

```nix
# Unified monitoring stack with GPU metrics
services.grafana = {
  enable = true;
  provision = {
    datasources = [{
      name = "Prometheus";
      type = "prometheus";
      url = "http://localhost:9090";
    }];
    
    dashboards = [
      {
        name = "CIM Overview";
        folder = "CIM";
        path = ./dashboards/cim-overview.json;
      }
      {
        name = "GPU Metrics";
        folder = "CIM";
        path = ./dashboards/gpu-metrics.json;
      }
    ];
  };
};

services.loki = {
  enable = true;
  configuration = {
    ingester = {
      lifecycler = {
        address = "127.0.0.1";
        ring.kvstore.store = "inmemory";
      };
    };
  };
};

# GPU-specific monitoring
services.dcgm-exporter = {
  enable = true;
  port = 9400;
  settings = {
    collectors = [
      "DCGM_FI_DEV_GPU_UTIL"
      "DCGM_FI_DEV_MEM_COPY_UTIL"
      "DCGM_FI_DEV_FB_FREE"
      "DCGM_FI_DEV_FB_USED"
      "DCGM_FI_DEV_GPU_TEMP"
      "DCGM_FI_DEV_POWER_USAGE"
    ];
  };
};
```

### GPU Resource Management

```rust
// GPU resource allocation for agents (supports both CUDA and Metal)
pub struct GpuResourceManager {
    available_gpus: Vec<GpuDevice>,
    allocations: HashMap<AgentId, GpuAllocation>,
}

#[derive(Debug, Clone)]
pub enum GpuDevice {
    Nvidia(NvidiaGpu),
    AppleSilicon(MetalGpu),
}

#[derive(Debug, Clone)]
pub struct NvidiaGpu {
    pub device_id: u32,
    pub model: String,
    pub vram_total_mb: u32,
    pub vram_available_mb: u32,
    pub compute_capability: f32,
    pub current_temperature: f32,
    pub power_usage_watts: f32,
}

#[derive(Debug, Clone)]
pub struct MetalGpu {
    pub device_name: String,
    pub gpu_cores: u32,
    pub neural_engine_cores: u32,
    pub unified_memory_total_mb: u32,
    pub unified_memory_available_mb: u32,
    pub metal_family: MetalFamily,
    pub max_threads_per_threadgroup: u32,
}

#[derive(Debug, Clone)]
pub enum MetalFamily {
    Apple8,  // M3 family
    Apple9,  // Future M4
}

#[derive(Debug, Clone)]
pub struct GpuAllocation {
    pub agent_id: AgentId,
    pub device: AllocatedDevice,
    pub memory_allocated_mb: u32,
    pub allocated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum AllocatedDevice {
    Cuda {
        device_id: u32,
        context: CudaContext,
    },
    Metal {
        device_name: String,
        command_queue: MetalCommandQueue,
    },
}

impl GpuResourceManager {
    pub async fn allocate_gpu_for_agent(
        &mut self,
        agent_id: AgentId,
        requirements: ModelRequirements,
    ) -> Result<GpuAllocation, AllocationError> {
        let required_memory_mb = (requirements.vram_usage_gb * 1024.0) as u32;
        
        // Find suitable GPU (NVIDIA or Apple Silicon)
        let (gpu_index, allocated_device) = self.available_gpus.iter()
            .enumerate()
            .find_map(|(idx, gpu)| {
                match gpu {
                    GpuDevice::Nvidia(nvidia) => {
                        if nvidia.vram_available_mb >= required_memory_mb
                            && nvidia.compute_capability >= 8.6 {
                            Some((idx, AllocatedDevice::Cuda {
                                device_id: nvidia.device_id,
                                context: self.create_cuda_context(nvidia.device_id).ok()?,
                            }))
                        } else {
                            None
                        }
                    }
                    GpuDevice::AppleSilicon(metal) => {
                        if metal.unified_memory_available_mb >= required_memory_mb {
                            Some((idx, AllocatedDevice::Metal {
                                device_name: metal.device_name.clone(),
                                command_queue: self.create_metal_queue(&metal.device_name).ok()?,
                            }))
                        } else {
                            None
                        }
                    }
                }
            })
            .ok_or(AllocationError::NoSuitableGpuAvailable)?;

        // Create allocation
        let allocation = GpuAllocation {
            agent_id,
            device: allocated_device,
            memory_allocated_mb: required_memory_mb,
            allocated_at: Utc::now(),
        };

        // Update available memory
        self.update_gpu_availability(gpu_index, required_memory_mb);

        // Publish allocation event
        self.publish_allocation_event(&allocation).await?;

        Ok(allocation)
    }

    fn create_metal_queue(&self, device_name: &str) -> Result<MetalCommandQueue, Error> {
        // Metal Performance Shaders initialization
        // This would interface with the Metal API
        Ok(MetalCommandQueue::new(device_name)?)
    }
}
```

## Disaster Recovery

```bash
# Backup system state
nix run .#backup-cim-state -- --output /backup/cim-state.tar.gz

# Restore to new hardware
nixos-anywhere --flake .#cim-node-restored root@new-host \
  --extra-files /backup/cim-state.tar.gz

# Verify cluster health
nats --server nats://cluster.cim.internal:4222 server check
```

## Conclusion

CIM's NixOS-based infrastructure provides a robust, scalable foundation that aligns with its event-driven, composable architecture. By leveraging NixOS's declarative configuration and NATS's distributed messaging, CIM achieves enterprise-grade reliability without the complexity of container orchestration platforms. 
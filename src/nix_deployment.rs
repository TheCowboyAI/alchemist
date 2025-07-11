//! Nix deployment functionality for Alchemist
//! 
//! Provides functionality to generate and deploy Nix configurations for
//! Alchemist services, agents, and CIM instances.

use crate::config::{AgentConfig, ServiceConfig, DeploymentConfig};
use crate::deployment::{DeploymentTarget, DeploymentStrategy};
use crate::error::AlchemistError;
use crate::nats_client::NatsClient;
use async_nats::jetstream::consumer::PullConsumer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use futures::StreamExt;

/// Nix deployment manager
#[derive(Clone)]
pub struct NixDeployer {
    /// NATS client for publishing deployment events
    nats_client: NatsClient,
    /// Base directory for Nix configurations
    nix_dir: PathBuf,
    /// Deployment templates directory
    templates_dir: PathBuf,
}

/// Nix deployment specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NixDeploymentSpec {
    /// Deployment ID
    pub id: String,
    /// Target environment
    pub target: DeploymentTarget,
    /// Services to deploy
    pub services: Vec<NixServiceSpec>,
    /// Agents to deploy
    pub agents: Vec<NixAgentSpec>,
    /// NATS mesh configuration
    pub nats_mesh: NatsMeshConfig,
    /// Deployment strategy
    pub strategy: DeploymentStrategy,
    /// Environment variables
    pub environment: HashMap<String, String>,
    /// Secrets configuration
    pub secrets: SecretsConfig,
}

/// Nix service specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NixServiceSpec {
    /// Service name
    pub name: String,
    /// Service configuration
    pub config: ServiceConfig,
    /// Resource limits
    pub resources: ResourceLimits,
    /// Health check configuration
    pub health_check: HealthCheckConfig,
    /// Replicas
    pub replicas: u32,
}

/// Nix agent specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NixAgentSpec {
    /// Agent name
    pub name: String,
    /// Agent configuration
    pub config: AgentConfig,
    /// Resource limits
    pub resources: ResourceLimits,
    /// Capabilities
    pub capabilities: Vec<String>,
}

/// NATS mesh configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsMeshConfig {
    /// Cluster nodes
    pub nodes: Vec<NatsNode>,
    /// Leaf nodes
    pub leaf_nodes: Vec<LeafNode>,
    /// JetStream configuration
    pub jetstream: JetStreamConfig,
}

/// NATS node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsNode {
    /// Node name
    pub name: String,
    /// Host address
    pub host: String,
    /// Client port
    pub client_port: u16,
    /// Cluster port
    pub cluster_port: u16,
    /// Routes to other nodes
    pub routes: Vec<String>,
}

/// Leaf node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeafNode {
    /// Node name
    pub name: String,
    /// Remote URLs
    pub remotes: Vec<String>,
    /// Credentials
    pub credentials: Option<String>,
}

/// JetStream configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JetStreamConfig {
    /// Storage directory
    pub store_dir: String,
    /// Max memory
    pub max_memory: String,
    /// Max file storage
    pub max_file: String,
}

/// Resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// CPU limit
    pub cpu: Option<String>,
    /// Memory limit
    pub memory: Option<String>,
    /// Disk limit
    pub disk: Option<String>,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// HTTP endpoint
    pub http_endpoint: Option<String>,
    /// TCP port
    pub tcp_port: Option<u16>,
    /// Interval in seconds
    pub interval: u32,
    /// Timeout in seconds
    pub timeout: u32,
    /// Retries
    pub retries: u32,
}

/// Secrets configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsConfig {
    /// Secrets provider
    pub provider: SecretsProvider,
    /// Secrets paths
    pub paths: HashMap<String, String>,
}

/// Secrets provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecretsProvider {
    /// Environment variables
    Environment,
    /// File-based secrets
    File { base_path: String },
    /// HashiCorp Vault
    Vault { url: String, token: String },
    /// SOPS
    Sops { key_file: String },
}

/// Deployment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentStatus {
    /// Deployment ID
    pub id: String,
    /// Current state
    pub state: DeploymentState,
    /// Progress percentage
    pub progress: u8,
    /// Messages
    pub messages: Vec<String>,
    /// Deployed services
    pub services: HashMap<String, ServiceStatus>,
    /// Deployed agents
    pub agents: HashMap<String, AgentStatus>,
}

/// Deployment state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentState {
    /// Deployment pending
    Pending,
    /// Generating configurations
    Generating,
    /// Building
    Building,
    /// Deploying
    Deploying,
    /// Verifying
    Verifying,
    /// Completed
    Completed,
    /// Failed
    Failed { error: String },
    /// Rolling back
    RollingBack,
    /// Rolled back
    RolledBack,
}

/// Service status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    /// Service name
    pub name: String,
    /// Running instances
    pub running: u32,
    /// Desired instances
    pub desired: u32,
    /// Health status
    pub health: HealthStatus,
}

/// Agent status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    /// Agent name
    pub name: String,
    /// Running state
    pub running: bool,
    /// Health status
    pub health: HealthStatus,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Healthy
    Healthy,
    /// Degraded
    Degraded,
    /// Unhealthy
    Unhealthy,
    /// Unknown
    Unknown,
}

impl NixDeployer {
    /// Create a new Nix deployer
    pub async fn new(
        nats_client: NatsClient,
        nix_dir: PathBuf,
        templates_dir: PathBuf,
    ) -> Result<Self, AlchemistError> {
        // Ensure directories exist
        fs::create_dir_all(&nix_dir).await?;
        fs::create_dir_all(&templates_dir).await?;

        Ok(Self {
            nats_client,
            nix_dir,
            templates_dir,
        })
    }

    /// Generate Nix configurations for deployment
    pub async fn generate_configs(
        &self,
        spec: &NixDeploymentSpec,
    ) -> Result<PathBuf, AlchemistError> {
        let deployment_dir = self.nix_dir.join(&spec.id);
        fs::create_dir_all(&deployment_dir).await?;

        // Generate flake.nix
        self.generate_flake(&deployment_dir, spec).await?;

        // Generate service modules
        for service in &spec.services {
            self.generate_service_module(&deployment_dir, service).await?;
        }

        // Generate agent modules
        for agent in &spec.agents {
            self.generate_agent_module(&deployment_dir, agent).await?;
        }

        // Generate NATS mesh configuration
        self.generate_nats_mesh(&deployment_dir, &spec.nats_mesh).await?;

        // Generate deployment configuration
        self.generate_deployment_config(&deployment_dir, spec).await?;

        // Publish generation complete event
        self.publish_deployment_event(
            &spec.id,
            DeploymentEvent::ConfigsGenerated {
                path: deployment_dir.to_string_lossy().to_string(),
            },
        )
        .await?;

        Ok(deployment_dir)
    }

    /// Generate flake.nix
    async fn generate_flake(
        &self,
        deployment_dir: &Path,
        spec: &NixDeploymentSpec,
    ) -> Result<(), AlchemistError> {
        let flake_content = format!(
            r#"{{
  description = "Alchemist deployment: {}";

  inputs = {{
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  }};

  outputs = {{ self, nixpkgs, flake-utils }}:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${{system}};
        
        alchemistService = import ./modules/alchemist-service.nix {{ inherit pkgs; }};
        alchemistAgent = import ./modules/alchemist-agent.nix {{ inherit pkgs; }};
        natsMesh = import ./modules/nats-mesh.nix {{ inherit pkgs; }};
        
      in {{
        # Development shell
        devShells.default = pkgs.mkShell {{
          buildInputs = with pkgs; [
            rustc
            cargo
            nats-server
            nats-top
            natscli
          ];
        }};
        
        # NixOS configurations
        nixosConfigurations = {{
          {} = nixpkgs.lib.nixosSystem {{
            inherit system;
            modules = [
              ./configuration.nix
              alchemistService
              alchemistAgent
              natsMesh
            ];
          }};
        }};
        
        # Deployment apps
        apps = {{
          deploy = {{
            type = "app";
            program = "${{pkgs.writeShellScript "deploy" ''
              #!${{pkgs.bash}}/bin/bash
              set -e
              
              echo "Deploying Alchemist configuration..."
              nixos-rebuild switch --flake .#{}
            ''}}/bin/deploy";
          }};
          
          validate = {{
            type = "app";
            program = "${{pkgs.writeShellScript "validate" ''
              #!${{pkgs.bash}}/bin/bash
              set -e
              
              echo "Validating configuration..."
              nix flake check
            ''}}/bin/validate";
          }};
        }};
      }});
}}"#,
            spec.id,
            spec.target.name(),
            spec.target.name()
        );

        let flake_path = deployment_dir.join("flake.nix");
        let mut file = fs::File::create(flake_path).await?;
        file.write_all(flake_content.as_bytes()).await?;

        Ok(())
    }

    /// Generate service module
    async fn generate_service_module(
        &self,
        deployment_dir: &Path,
        service: &NixServiceSpec,
    ) -> Result<(), AlchemistError> {
        let modules_dir = deployment_dir.join("modules");
        fs::create_dir_all(&modules_dir).await?;

        let module_content = format!(
            r#"{{ config, lib, pkgs, ... }}:

let
  cfg = config.services.alchemist.{};
in {{
  options.services.alchemist.{} = {{
    enable = lib.mkEnableOption "Alchemist service: {}";
    
    port = lib.mkOption {{
      type = lib.types.int;
      default = {};
      description = "Service port";
    }};
    
    replicas = lib.mkOption {{
      type = lib.types.int;
      default = {};
      description = "Number of replicas";
    }};
    
    resources = {{
      cpu = lib.mkOption {{
        type = lib.types.nullOr lib.types.str;
        default = {};
        description = "CPU limit";
      }};
      
      memory = lib.mkOption {{
        type = lib.types.nullOr lib.types.str;
        default = {};
        description = "Memory limit";
      }};
    }};
  }};
  
  config = lib.mkIf cfg.enable {{
    systemd.services."alchemist-{}" = {{
      description = "Alchemist service: {}";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" "nats.service" ];
      
      serviceConfig = {{
        Type = "notify";
        ExecStart = "${{pkgs.alchemist}}/bin/alchemist service --name {} --port ${{toString cfg.port}}";
        Restart = "always";
        RestartSec = 5;
        
        # Resource limits
        {} 
        {}
        
        # Security hardening
        DynamicUser = true;
        PrivateTmp = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        NoNewPrivileges = true;
      }};
      
      environment = {{
        NATS_URL = "nats://localhost:4222";
        SERVICE_NAME = "{}";
        {}
      }};
    }};
    
    # Health check
    systemd.timers."alchemist-{}-health" = {{
      wantedBy = [ "timers.target" ];
      timerConfig = {{
        OnBootSec = "1m";
        OnUnitActiveSec = "{}s";
      }};
    }};
    
    systemd.services."alchemist-{}-health" = {{
      serviceConfig = {{
        Type = "oneshot";
        ExecStart = pkgs.writeShellScript "health-check" ''
          #!${{pkgs.bash}}/bin/bash
          {}
        '';
      }};
    }};
  }};
}}"#,
            service.name,
            service.name,
            service.name,
            service.config.port.map(|p| p.to_string()).unwrap_or_else(|| "null".to_string()),
            service.replicas,
            service.resources.cpu.as_deref().map(|c| format!("\"{}\"", c)).unwrap_or("null".to_string()),
            service.resources.memory.as_deref().map(|m| format!("\"{}\"", m)).unwrap_or("null".to_string()),
            service.name,
            service.name,
            service.name,
            service.resources.cpu.as_deref().map(|c| format!("CPUQuota = \"{}\";", c)).unwrap_or_default(),
            service.resources.memory.as_deref().map(|m| format!("MemoryLimit = \"{}\";", m)).unwrap_or_default(),
            service.name,
            self.format_environment(&service.config.environment),
            service.name,
            service.health_check.interval,
            service.name,
            self.generate_health_check_script(&service.health_check)
        );

        let module_path = modules_dir.join(format!("{}.nix", service.name));
        let mut file = fs::File::create(module_path).await?;
        file.write_all(module_content.as_bytes()).await?;

        Ok(())
    }

    /// Generate agent module
    async fn generate_agent_module(
        &self,
        deployment_dir: &Path,
        agent: &NixAgentSpec,
    ) -> Result<(), AlchemistError> {
        let modules_dir = deployment_dir.join("modules");
        fs::create_dir_all(&modules_dir).await?;

        let module_content = format!(
            r#"{{ config, lib, pkgs, ... }}:

let
  cfg = config.services.alchemist.agents.{};
in {{
  options.services.alchemist.agents.{} = {{
    enable = lib.mkEnableOption "Alchemist agent: {}";
    
    capabilities = lib.mkOption {{
      type = lib.types.listOf lib.types.str;
      default = [ {} ];
      description = "Agent capabilities";
    }};
    
    resources = {{
      cpu = lib.mkOption {{
        type = lib.types.nullOr lib.types.str;
        default = {};
        description = "CPU limit";
      }};
      
      memory = lib.mkOption {{
        type = lib.types.nullOr lib.types.str;
        default = {};
        description = "Memory limit";
      }};
    }};
  }};
  
  config = lib.mkIf cfg.enable {{
    systemd.services."alchemist-agent-{}" = {{
      description = "Alchemist agent: {}";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" "nats.service" ];
      
      serviceConfig = {{
        Type = "notify";
        ExecStart = "${{pkgs.alchemist}}/bin/alchemist agent --name {} --capabilities ${{lib.concatStringsSep "," cfg.capabilities}}";
        Restart = "always";
        RestartSec = 5;
        
        # Resource limits
        {}
        {}
        
        # Security hardening
        DynamicUser = true;
        PrivateTmp = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        NoNewPrivileges = true;
        
        # Capabilities for agent
        AmbientCapabilities = [ {} ];
      }};
      
      environment = {{
        NATS_URL = "nats://localhost:4222";
        AGENT_NAME = "{}";
        {}
      }};
    }};
  }};
}}"#,
            agent.name,
            agent.name,
            agent.name,
            agent.capabilities.iter().map(|c| format!("\"{}\"", c)).collect::<Vec<_>>().join(" "),
            agent.resources.cpu.as_deref().map(|c| format!("\"{}\"", c)).unwrap_or("null".to_string()),
            agent.resources.memory.as_deref().map(|m| format!("\"{}\"", m)).unwrap_or("null".to_string()),
            agent.name,
            agent.name,
            agent.name,
            agent.resources.cpu.as_deref().map(|c| format!("CPUQuota = \"{}\";", c)).unwrap_or_default(),
            agent.resources.memory.as_deref().map(|m| format!("MemoryLimit = \"{}\";", m)).unwrap_or_default(),
            self.map_capabilities_to_linux(&agent.capabilities),
            agent.name,
            self.format_environment(&agent.config.environment)
        );

        let module_path = modules_dir.join(format!("agent-{}.nix", agent.name));
        let mut file = fs::File::create(module_path).await?;
        file.write_all(module_content.as_bytes()).await?;

        Ok(())
    }

    /// Generate NATS mesh configuration
    async fn generate_nats_mesh(
        &self,
        deployment_dir: &Path,
        mesh_config: &NatsMeshConfig,
    ) -> Result<(), AlchemistError> {
        let modules_dir = deployment_dir.join("modules");
        fs::create_dir_all(&modules_dir).await?;

        let module_content = format!(
            r#"{{ config, lib, pkgs, ... }}:

let
  cfg = config.services.nats-mesh;
in {{
  options.services.nats-mesh = {{
    enable = lib.mkEnableOption "NATS mesh network";
    
    nodes = lib.mkOption {{
      type = lib.types.listOf (lib.types.submodule {{
        options = {{
          name = lib.mkOption {{
            type = lib.types.str;
            description = "Node name";
          }};
          host = lib.mkOption {{
            type = lib.types.str;
            description = "Host address";
          }};
          clientPort = lib.mkOption {{
            type = lib.types.int;
            default = 4222;
            description = "Client port";
          }};
          clusterPort = lib.mkOption {{
            type = lib.types.int;
            default = 6222;
            description = "Cluster port";
          }};
          routes = lib.mkOption {{
            type = lib.types.listOf lib.types.str;
            default = [];
            description = "Routes to other nodes";
          }};
        }};
      }});
      default = [];
      description = "NATS cluster nodes";
    }};
    
    leafNodes = lib.mkOption {{
      type = lib.types.listOf (lib.types.submodule {{
        options = {{
          name = lib.mkOption {{
            type = lib.types.str;
            description = "Leaf node name";
          }};
          remotes = lib.mkOption {{
            type = lib.types.listOf lib.types.str;
            description = "Remote URLs";
          }};
          credentials = lib.mkOption {{
            type = lib.types.nullOr lib.types.str;
            default = null;
            description = "Credentials file";
          }};
        }};
      }});
      default = [];
      description = "NATS leaf nodes";
    }};
    
    jetstream = {{
      storeDir = lib.mkOption {{
        type = lib.types.str;
        default = "/var/lib/nats/jetstream";
        description = "JetStream storage directory";
      }};
      maxMemory = lib.mkOption {{
        type = lib.types.str;
        default = "1G";
        description = "Maximum memory for JetStream";
      }};
      maxFile = lib.mkOption {{
        type = lib.types.str;
        default = "10G";
        description = "Maximum file storage for JetStream";
      }};
    }};
  }};
  
  config = lib.mkIf cfg.enable {{
    # Generate NATS configuration for each node
    {}
    
    # Generate leaf node configurations
    {}
  }};
}}"#,
            self.generate_nats_node_configs(&mesh_config.nodes),
            self.generate_leaf_node_configs(&mesh_config.leaf_nodes)
        );

        let module_path = modules_dir.join("nats-mesh.nix");
        let mut file = fs::File::create(module_path).await?;
        file.write_all(module_content.as_bytes()).await?;

        Ok(())
    }

    /// Generate deployment configuration
    async fn generate_deployment_config(
        &self,
        deployment_dir: &Path,
        spec: &NixDeploymentSpec,
    ) -> Result<(), AlchemistError> {
        let config_content = format!(
            r#"{{ config, pkgs, ... }}:

{{
  imports = [
    ./modules/nats-mesh.nix
    {}
    {}
  ];
  
  # Enable NATS mesh
  services.nats-mesh = {{
    enable = true;
    nodes = [
      {}
    ];
    leafNodes = [
      {}
    ];
    jetstream = {{
      storeDir = "{}";
      maxMemory = "{}";
      maxFile = "{}";
    }};
  }};
  
  # Enable services
  {}
  
  # Enable agents
  {}
  
  # Environment configuration
  environment.systemPackages = with pkgs; [
    nats-server
    natscli
    alchemist
  ];
  
  # Networking
  networking.firewall.allowedTCPPorts = [ 4222 6222 8222 ];
  
  # System configuration
  system.stateVersion = "24.05";
}}"#,
            spec.services.iter()
                .map(|s| format!("./modules/{}.nix", s.name))
                .collect::<Vec<_>>()
                .join("\n    "),
            spec.agents.iter()
                .map(|a| format!("./modules/agent-{}.nix", a.name))
                .collect::<Vec<_>>()
                .join("\n    "),
            self.format_nats_nodes(&spec.nats_mesh.nodes),
            self.format_leaf_nodes(&spec.nats_mesh.leaf_nodes),
            spec.nats_mesh.jetstream.store_dir,
            spec.nats_mesh.jetstream.max_memory,
            spec.nats_mesh.jetstream.max_file,
            self.format_service_enables(&spec.services),
            self.format_agent_enables(&spec.agents)
        );

        let config_path = deployment_dir.join("configuration.nix");
        let mut file = fs::File::create(config_path).await?;
        file.write_all(config_content.as_bytes()).await?;

        Ok(())
    }

    /// Apply deployment
    pub async fn apply_deployment(
        &self,
        spec: &NixDeploymentSpec,
        deployment_dir: &Path,
    ) -> Result<String, AlchemistError> {
        // Publish deployment starting event
        self.publish_deployment_event(
            &spec.id,
            DeploymentEvent::DeploymentStarted {
                target: spec.target.name().to_string(),
                strategy: format!("{:?}", spec.strategy),
            },
        )
        .await?;

        match spec.strategy {
            DeploymentStrategy::RollingUpdate { max_unavailable } => {
                self.apply_rolling_update(spec, deployment_dir, max_unavailable).await
            }
            DeploymentStrategy::BlueGreen => {
                self.apply_blue_green(spec, deployment_dir).await
            }
            DeploymentStrategy::Recreate => {
                self.apply_recreate(spec, deployment_dir).await
            }
        }
    }

    /// Apply rolling update deployment
    async fn apply_rolling_update(
        &self,
        spec: &NixDeploymentSpec,
        deployment_dir: &Path,
        max_unavailable: u32,
    ) -> Result<String, AlchemistError> {
        // For rolling updates, we deploy services in batches
        let batch_size = spec.services.len().saturating_sub(max_unavailable as usize);
        
        for (i, service_batch) in spec.services.chunks(batch_size).enumerate() {
            self.publish_deployment_event(
                &spec.id,
                DeploymentEvent::Progress {
                    message: format!("Deploying batch {} of services", i + 1),
                    percentage: ((i + 1) * 100 / spec.services.len()) as u8,
                },
            )
            .await?;

            // Deploy this batch
            self.run_nix_deploy(deployment_dir, &spec.target).await?;
            
            // Wait for health checks
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            
            // Verify deployment
            self.verify_service_health(service_batch).await?;
        }

        Ok(spec.id.clone())
    }

    /// Apply blue-green deployment
    async fn apply_blue_green(
        &self,
        spec: &NixDeploymentSpec,
        deployment_dir: &Path,
    ) -> Result<String, AlchemistError> {
        // Deploy to blue environment
        self.publish_deployment_event(
            &spec.id,
            DeploymentEvent::Progress {
                message: "Deploying to blue environment".to_string(),
                percentage: 25,
            },
        )
        .await?;

        self.run_nix_deploy(deployment_dir, &spec.target).await?;

        // Verify blue environment
        self.publish_deployment_event(
            &spec.id,
            DeploymentEvent::Progress {
                message: "Verifying blue environment".to_string(),
                percentage: 50,
            },
        )
        .await?;

        self.verify_deployment_health(spec).await?;

        // Switch traffic to blue
        self.publish_deployment_event(
            &spec.id,
            DeploymentEvent::Progress {
                message: "Switching traffic to blue environment".to_string(),
                percentage: 75,
            },
        )
        .await?;

        // In a real implementation, this would update load balancer or DNS
        self.switch_traffic_to_blue(spec).await?;

        Ok(spec.id.clone())
    }

    /// Apply recreate deployment
    async fn apply_recreate(
        &self,
        spec: &NixDeploymentSpec,
        deployment_dir: &Path,
    ) -> Result<String, AlchemistError> {
        // Stop existing services
        self.publish_deployment_event(
            &spec.id,
            DeploymentEvent::Progress {
                message: "Stopping existing services".to_string(),
                percentage: 25,
            },
        )
        .await?;

        // Deploy new configuration
        self.publish_deployment_event(
            &spec.id,
            DeploymentEvent::Progress {
                message: "Deploying new configuration".to_string(),
                percentage: 50,
            },
        )
        .await?;

        self.run_nix_deploy(deployment_dir, &spec.target).await?;

        // Verify deployment
        self.publish_deployment_event(
            &spec.id,
            DeploymentEvent::Progress {
                message: "Verifying deployment".to_string(),
                percentage: 75,
            },
        )
        .await?;

        self.verify_deployment_health(spec).await?;

        Ok(spec.id.clone())
    }

    /// Run Nix deployment command
    async fn run_nix_deploy(
        &self,
        deployment_dir: &Path,
        target: &DeploymentTarget,
    ) -> Result<(), AlchemistError> {
        let output = Command::new("nix")
            .arg("run")
            .arg(format!("{}#apps.deploy", deployment_dir.display()))
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(AlchemistError::DeploymentFailed(error.to_string()));
        }

        Ok(())
    }

    /// Validate deployment configuration
    pub async fn validate_deployment(
        &self,
        deployment_dir: &Path,
    ) -> Result<(), AlchemistError> {
        let output = Command::new("nix")
            .arg("flake")
            .arg("check")
            .arg(deployment_dir)
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(AlchemistError::ValidationFailed(error.to_string()));
        }

        Ok(())
    }

    /// Get deployment status
    pub async fn get_deployment_status(
        &self,
        deployment_id: &str,
    ) -> Result<DeploymentStatus, AlchemistError> {
        // Query deployment status from NATS
        let status_subject = format!("deployment.{}.status", deployment_id);
        let response = self.nats_client.request(&status_subject, vec![]).await?;
        
        let status: DeploymentStatus = serde_json::from_slice(&response.payload)?;
        Ok(status)
    }

    /// Rollback deployment
    pub async fn rollback_deployment(
        &self,
        deployment_id: &str,
    ) -> Result<(), AlchemistError> {
        // Publish rollback event
        self.publish_deployment_event(
            deployment_id,
            DeploymentEvent::RollbackStarted,
        )
        .await?;

        // Get previous deployment configuration
        let previous_config = self.get_previous_deployment_config(deployment_id).await?;
        
        // Apply previous configuration
        self.apply_deployment(&previous_config, &self.nix_dir.join(&previous_config.id)).await?;

        // Publish rollback complete event
        self.publish_deployment_event(
            deployment_id,
            DeploymentEvent::RollbackCompleted,
        )
        .await?;

        Ok(())
    }

    /// Monitor deployment health
    pub async fn monitor_deployment(
        &self,
        deployment_id: &str,
    ) -> Result<(), AlchemistError> {
        // Subscribe to health events
        let health_subject = format!("deployment.{}.health.*", deployment_id);
        let mut subscription = self.nats_client.subscribe(&health_subject).await?;

        // Start background monitoring tasks
        self.start_metrics_collection(deployment_id).await?;
        self.start_log_aggregation(deployment_id).await?;
        self.start_alert_monitoring(deployment_id).await?;

        while let Some(message) = subscription.next().await {
            let health_event: HealthEvent = serde_json::from_slice(&message.payload)?;
            
            // Process health event
            match health_event {
                HealthEvent::ServiceHealthy { ref service, .. } => {
                    tracing::info!("Service {} is healthy", service);
                    self.update_service_metrics(service, true).await?
                }
                HealthEvent::ServiceUnhealthy { ref service, ref reason, .. } => {
                    tracing::warn!("Service {} is unhealthy: {}", service, reason);
                    self.update_service_metrics(service, false).await?;
                    
                    // Trigger automatic rollback if configured
                    if self.should_auto_rollback(&health_event) {
                        self.rollback_deployment(deployment_id).await?;
                        break;
                    }
                }
                HealthEvent::AgentHealthy { ref agent, .. } => {
                    tracing::info!("Agent {} is healthy", agent);
                    self.update_agent_metrics(agent, true).await?
                }
                HealthEvent::AgentUnhealthy { ref agent, ref reason, .. } => {
                    tracing::warn!("Agent {} is unhealthy: {}", agent, reason);
                    self.update_agent_metrics(agent, false).await?;
                }
            }
        }

        Ok(())
    }

    /// Start metrics collection for deployment
    async fn start_metrics_collection(&self, deployment_id: &str) -> Result<(), AlchemistError> {
        let subject = format!("deployment.{}.metrics.collect", deployment_id);
        let nats_client = self.nats_client.clone();
        let deployment_id = deployment_id.to_string();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                // Collect metrics from all services and agents
                let metrics = DeploymentMetrics {
                    timestamp: chrono::Utc::now().timestamp(),
                    deployment_id: deployment_id.clone(),
                    services: HashMap::new(), // Would be populated with actual metrics
                    agents: HashMap::new(),   // Would be populated with actual metrics
                    system: SystemMetrics {
                        cpu_usage: 0.0,
                        memory_usage: 0.0,
                        disk_usage: 0.0,
                        network_in: 0,
                        network_out: 0,
                    },
                };
                
                if let Ok(payload) = serde_json::to_vec(&metrics) {
                    let _ = nats_client.publish(&subject, payload).await;
                }
            }
        });

        Ok(())
    }

    /// Start log aggregation for deployment
    async fn start_log_aggregation(&self, deployment_id: &str) -> Result<(), AlchemistError> {
        let subject = format!("deployment.{}.logs.*", deployment_id);
        let mut subscription = self.nats_client.subscribe(&subject).await?;
        let deployment_id = deployment_id.to_string();

        tokio::spawn(async move {
            while let Some(message) = subscription.next().await {
                // Parse log entry
                if let Ok(log_entry) = serde_json::from_slice::<LogEntry>(&message.payload) {
                    // Store in log aggregation system
                    tracing::info!(
                        deployment_id = %deployment_id,
                        service = %log_entry.service,
                        level = %log_entry.level,
                        message = %log_entry.message,
                        "Deployment log"
                    );
                }
            }
        });

        Ok(())
    }

    /// Start alert monitoring for deployment
    async fn start_alert_monitoring(&self, deployment_id: &str) -> Result<(), AlchemistError> {
        let subject = format!("deployment.{}.alerts", deployment_id);
        let mut subscription = self.nats_client.subscribe(&subject).await?;
        let nats_client = self.nats_client.clone();

        tokio::spawn(async move {
            while let Some(message) = subscription.next().await {
                if let Ok(alert) = serde_json::from_slice::<DeploymentAlert>(&message.payload) {
                    tracing::warn!(
                        severity = %alert.severity,
                        service = %alert.service,
                        message = %alert.message,
                        "Deployment alert"
                    );
                    
                    // Send notification
                    let notification_subject = format!("notifications.deployment.{}", alert.severity);
                    let _ = nats_client.publish(&notification_subject, message.payload.to_vec()).await;
                }
            }
        });

        Ok(())
    }

    /// Update service metrics
    async fn update_service_metrics(&self, service: &str, healthy: bool) -> Result<(), AlchemistError> {
        let subject = format!("metrics.service.{}.health", service);
        let metric = HealthMetric {
            service: service.to_string(),
            healthy,
            timestamp: chrono::Utc::now().timestamp(),
        };
        
        let payload = serde_json::to_vec(&metric)?;
        self.nats_client.publish(&subject, payload).await?;
        
        Ok(())
    }

    /// Update agent metrics
    async fn update_agent_metrics(&self, agent: &str, healthy: bool) -> Result<(), AlchemistError> {
        let subject = format!("metrics.agent.{}.health", agent);
        let metric = HealthMetric {
            service: agent.to_string(),
            healthy,
            timestamp: chrono::Utc::now().timestamp(),
        };
        
        let payload = serde_json::to_vec(&metric)?;
        self.nats_client.publish(&subject, payload).await?;
        
        Ok(())
    }

    /// Publish deployment event
    async fn publish_deployment_event(
        &self,
        deployment_id: &str,
        event: DeploymentEvent,
    ) -> Result<(), AlchemistError> {
        let subject = format!("deployment.{}.events", deployment_id);
        let payload = serde_json::to_vec(&event)?;
        self.nats_client.publish(&subject, payload).await?;
        Ok(())
    }

    /// Verify service health
    async fn verify_service_health(
        &self,
        services: &[NixServiceSpec],
    ) -> Result<(), AlchemistError> {
        for service in services {
            if let Some(endpoint) = &service.health_check.http_endpoint {
                // Perform HTTP health check
                let port = service.config.port.unwrap_or(8080);
                let url = format!("http://localhost:{}{}", port, endpoint);
                let response = reqwest::get(&url).await?;
                
                if !response.status().is_success() {
                    return Err(AlchemistError::HealthCheckFailed(service.name.clone()));
                }
            } else if let Some(port) = service.health_check.tcp_port {
                // Perform TCP health check
                use tokio::net::TcpStream;
                let addr = format!("127.0.0.1:{}", port);
                TcpStream::connect(&addr).await?;
            }
        }
        
        Ok(())
    }

    /// Verify deployment health
    async fn verify_deployment_health(
        &self,
        spec: &NixDeploymentSpec,
    ) -> Result<(), AlchemistError> {
        // Verify all services
        self.verify_service_health(&spec.services).await?;
        
        // Verify agents
        for agent in &spec.agents {
            // Query agent status via NATS
            let status_subject = format!("agent.{}.status", agent.name);
            let response = self.nats_client.request(&status_subject, vec![]).await?;
            
            if response.payload.is_empty() {
                return Err(AlchemistError::AgentNotResponding(agent.name.clone()));
            }
        }
        
        Ok(())
    }

    /// Switch traffic to blue environment (for blue-green deployments)
    async fn switch_traffic_to_blue(
        &self,
        spec: &NixDeploymentSpec,
    ) -> Result<(), AlchemistError> {
        // In a real implementation, this would:
        // 1. Update load balancer configuration
        // 2. Update DNS records
        // 3. Drain connections from green environment
        
        // For now, just publish an event
        self.publish_deployment_event(
            &spec.id,
            DeploymentEvent::TrafficSwitched {
                from: "green".to_string(),
                to: "blue".to_string(),
            },
        )
        .await?;
        
        Ok(())
    }

    /// Get previous deployment configuration
    async fn get_previous_deployment_config(
        &self,
        deployment_id: &str,
    ) -> Result<NixDeploymentSpec, AlchemistError> {
        // In a real implementation, this would retrieve from a deployment history store
        // For now, return an error
        Err(AlchemistError::DeploymentNotFound(deployment_id.to_string()))
    }

    /// Check if automatic rollback should be triggered
    fn should_auto_rollback(&self, health_event: &HealthEvent) -> bool {
        // In a real implementation, this would check rollback policies
        matches!(health_event, HealthEvent::ServiceUnhealthy { critical: true, .. })
    }

    /// Format environment variables for Nix
    fn format_environment(&self, env: &HashMap<String, String>) -> String {
        env.iter()
            .map(|(k, v)| format!("{} = \"{}\";", k, v))
            .collect::<Vec<_>>()
            .join("\n        ")
    }

    /// Generate health check script
    fn generate_health_check_script(&self, config: &HealthCheckConfig) -> String {
        if let Some(endpoint) = &config.http_endpoint {
            format!(
                "curl -f -s -m {} http://localhost:${{cfg.port}}{} || exit 1",
                config.timeout, endpoint
            )
        } else if let Some(port) = config.tcp_port {
            format!(
                "timeout {} bash -c 'cat < /dev/null > /dev/tcp/localhost/{}'",
                config.timeout, port
            )
        } else {
            "true".to_string()
        }
    }

    /// Map capabilities to Linux capabilities
    fn map_capabilities_to_linux(&self, capabilities: &[String]) -> String {
        capabilities.iter()
            .filter_map(|cap| {
                match cap.as_str() {
                    "network" => Some("CAP_NET_ADMIN CAP_NET_RAW"),
                    "filesystem" => Some("CAP_DAC_OVERRIDE"),
                    "process" => Some("CAP_SYS_PTRACE"),
                    _ => None,
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Generate NATS node configurations
    fn generate_nats_node_configs(&self, nodes: &[NatsNode]) -> String {
        nodes.iter()
            .map(|node| {
                format!(
                    r#"systemd.services."nats-{}" = {{
      description = "NATS server node: {}";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];
      
      serviceConfig = {{
        Type = "simple";
        ExecStart = "${{pkgs.nats-server}}/bin/nats-server -c ${{pkgs.writeText "nats-{}.conf" ''
          server_name: {}
          listen: {}:{}
          
          cluster {{
            name: alchemist
            listen: {}:{}
            routes: [ {} ]
          }}
          
          jetstream {{
            store_dir: ${{cfg.jetstream.storeDir}}/{}
            max_mem: ${{cfg.jetstream.maxMemory}}
            max_file: ${{cfg.jetstream.maxFile}}
          }}
        ''}}/nats-{}.conf";
        Restart = "always";
        RestartSec = 5;
      }};
    }};"#,
                    node.name,
                    node.name,
                    node.name,
                    node.name,
                    node.host,
                    node.client_port,
                    node.host,
                    node.cluster_port,
                    node.routes.iter().map(|r| format!("\"{}\"", r)).collect::<Vec<_>>().join(" "),
                    node.name,
                    node.name
                )
            })
            .collect::<Vec<_>>()
            .join("\n    ")
    }

    /// Generate leaf node configurations
    fn generate_leaf_node_configs(&self, leaf_nodes: &[LeafNode]) -> String {
        leaf_nodes.iter()
            .map(|node| {
                format!(
                    r#"systemd.services."nats-leaf-{}" = {{
      description = "NATS leaf node: {}";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];
      
      serviceConfig = {{
        Type = "simple";
        ExecStart = "${{pkgs.nats-server}}/bin/nats-server -c ${{pkgs.writeText "nats-leaf-{}.conf" ''
          server_name: leaf_{}
          
          leafnodes {{
            remotes: [
              {}
            ]
          }}
          
          jetstream {{
            store_dir: ${{cfg.jetstream.storeDir}}/leaf_{}
            max_mem: 512M
            max_file: 1G
          }}
        ''}}/nats-leaf-{}.conf";
        Restart = "always";
        RestartSec = 5;
      }};
    }};"#,
                    node.name,
                    node.name,
                    node.name,
                    node.name,
                    node.remotes.iter()
                        .map(|r| format!("{{ url: \"{}\" }}", r))
                        .collect::<Vec<_>>()
                        .join("\n              "),
                    node.name,
                    node.name
                )
            })
            .collect::<Vec<_>>()
            .join("\n    ")
    }

    /// Format NATS nodes for configuration
    fn format_nats_nodes(&self, nodes: &[NatsNode]) -> String {
        nodes.iter()
            .map(|node| {
                format!(
                    r#"{{
        name = "{}";
        host = "{}";
        clientPort = {};
        clusterPort = {};
        routes = [ {} ];
      }}"#,
                    node.name,
                    node.host,
                    node.client_port,
                    node.cluster_port,
                    node.routes.iter().map(|r| format!("\"{}\"", r)).collect::<Vec<_>>().join(" ")
                )
            })
            .collect::<Vec<_>>()
            .join("\n      ")
    }

    /// Format leaf nodes for configuration
    fn format_leaf_nodes(&self, nodes: &[LeafNode]) -> String {
        nodes.iter()
            .map(|node| {
                format!(
                    r#"{{
        name = "{}";
        remotes = [ {} ];
        credentials = {};
      }}"#,
                    node.name,
                    node.remotes.iter().map(|r| format!("\"{}\"", r)).collect::<Vec<_>>().join(" "),
                    node.credentials.as_deref().map(|c| format!("\"{}\"", c)).unwrap_or("null".to_string())
                )
            })
            .collect::<Vec<_>>()
            .join("\n      ")
    }

    /// Format service enables for configuration
    fn format_service_enables(&self, services: &[NixServiceSpec]) -> String {
        services.iter()
            .map(|service| {
                format!(
                    r#"services.alchemist.{} = {{
    enable = true;
    port = {};
    replicas = {};
    resources = {{
      cpu = {};
      memory = {};
    }};
  }};"#,
                    service.name,
                    service.config.port.map(|p| p.to_string()).unwrap_or_else(|| "null".to_string()),
                    service.replicas,
                    service.resources.cpu.as_deref().map(|c| format!("\"{}\"", c)).unwrap_or("null".to_string()),
                    service.resources.memory.as_deref().map(|m| format!("\"{}\"", m)).unwrap_or("null".to_string())
                )
            })
            .collect::<Vec<_>>()
            .join("\n  ")
    }

    /// Format agent enables for configuration
    fn format_agent_enables(&self, agents: &[NixAgentSpec]) -> String {
        agents.iter()
            .map(|agent| {
                format!(
                    r#"services.alchemist.agents.{} = {{
    enable = true;
    capabilities = [ {} ];
    resources = {{
      cpu = {};
      memory = {};
    }};
  }};"#,
                    agent.name,
                    agent.capabilities.iter().map(|c| format!("\"{}\"", c)).collect::<Vec<_>>().join(" "),
                    agent.resources.cpu.as_deref().map(|c| format!("\"{}\"", c)).unwrap_or("null".to_string()),
                    agent.resources.memory.as_deref().map(|m| format!("\"{}\"", m)).unwrap_or("null".to_string())
                )
            })
            .collect::<Vec<_>>()
            .join("\n  ")
    }
}

/// Deployment event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentEvent {
    /// Configs generated
    ConfigsGenerated { path: String },
    /// Deployment started
    DeploymentStarted { target: String, strategy: String },
    /// Progress update
    Progress { message: String, percentage: u8 },
    /// Deployment completed
    DeploymentCompleted,
    /// Deployment failed
    DeploymentFailed { error: String },
    /// Rollback started
    RollbackStarted,
    /// Rollback completed
    RollbackCompleted,
    /// Traffic switched
    TrafficSwitched { from: String, to: String },
}

/// Health event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthEvent {
    /// Service healthy
    ServiceHealthy {
        service: String,
        timestamp: i64,
    },
    /// Service unhealthy
    ServiceUnhealthy {
        service: String,
        reason: String,
        critical: bool,
        timestamp: i64,
    },
    /// Agent healthy
    AgentHealthy {
        agent: String,
        timestamp: i64,
    },
    /// Agent unhealthy
    AgentUnhealthy {
        agent: String,
        reason: String,
        timestamp: i64,
    },
}

/// Deployment metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentMetrics {
    pub timestamp: i64,
    pub deployment_id: String,
    pub services: HashMap<String, ServiceMetrics>,
    pub agents: HashMap<String, AgentMetrics>,
    pub system: SystemMetrics,
}

/// Service metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetrics {
    pub requests_per_second: f64,
    pub error_rate: f64,
    pub response_time_ms: f64,
    pub cpu_usage: f64,
    pub memory_usage_mb: f64,
}

/// Agent metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub cpu_usage: f64,
    pub memory_usage_mb: f64,
    pub queue_size: u32,
}

/// System metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_in: u64,
    pub network_out: u64,
}

/// Log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: i64,
    pub service: String,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, serde_json::Value>,
}

/// Deployment alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentAlert {
    pub timestamp: i64,
    pub severity: String,
    pub service: String,
    pub message: String,
    pub details: HashMap<String, serde_json::Value>,
}

/// Health metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetric {
    pub service: String,
    pub healthy: bool,
    pub timestamp: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_generate_configs() {
        let temp_dir = TempDir::new().unwrap();
        let nats_client = NatsClient::new("nats://localhost:4222").await.unwrap();
        
        let deployer = NixDeployer::new(
            nats_client,
            temp_dir.path().to_path_buf(),
            temp_dir.path().join("templates"),
        )
        .await
        .unwrap();

        let spec = NixDeploymentSpec {
            id: "test-deployment".to_string(),
            target: DeploymentTarget::Local,
            services: vec![
                NixServiceSpec {
                    name: "api".to_string(),
                    config: ServiceConfig {
                        name: "api".to_string(),
                        executable: "/usr/bin/alchemist-api".to_string(),
                        args: vec![],
                        port: Some(8080),
                        environment: HashMap::new(),
                        resources: crate::config::ResourceLimits {
                            cpu: Some(0.5),
                            memory: Some(512),
                            disk: None,
                        },
                        health_check: Some(crate::config::HealthCheckConfig {
                            endpoint: "/health".to_string(),
                            interval: 30,
                            timeout: 5,
                            failure_threshold: 3,
                        }),
                    },
                    resources: ResourceLimits {
                        cpu: Some("500m".to_string()),
                        memory: Some("512Mi".to_string()),
                        disk: None,
                    },
                    health_check: HealthCheckConfig {
                        http_endpoint: Some("/health".to_string()),
                        tcp_port: None,
                        interval: 30,
                        timeout: 5,
                        retries: 3,
                    },
                    replicas: 2,
                },
            ],
            agents: vec![
                NixAgentSpec {
                    name: "worker".to_string(),
                    config: AgentConfig {
                        name: "worker".to_string(),
                        agent_type: "worker".to_string(),
                        capabilities: vec!["process".to_string()],
                        config: HashMap::new(),
                        environment: HashMap::new(),
                        resources: crate::config::ResourceLimits {
                            cpu: Some(1.0),
                            memory: Some(1024),
                            disk: None,
                        },
                    },
                    resources: ResourceLimits {
                        cpu: Some("1000m".to_string()),
                        memory: Some("1Gi".to_string()),
                        disk: None,
                    },
                    capabilities: vec!["process".to_string()],
                },
            ],
            nats_mesh: NatsMeshConfig {
                nodes: vec![
                    NatsNode {
                        name: "nats-1".to_string(),
                        host: "localhost".to_string(),
                        client_port: 4222,
                        cluster_port: 6222,
                        routes: vec![],
                    },
                ],
                leaf_nodes: vec![],
                jetstream: JetStreamConfig {
                    store_dir: "/var/lib/nats/jetstream".to_string(),
                    max_memory: "1G".to_string(),
                    max_file: "10G".to_string(),
                },
            },
            strategy: DeploymentStrategy::RollingUpdate { max_unavailable: 1 },
            environment: HashMap::new(),
            secrets: SecretsConfig {
                provider: SecretsProvider::Environment,
                paths: HashMap::new(),
            },
        };

        let deployment_dir = deployer.generate_configs(&spec).await.unwrap();
        
        // Verify files were created
        assert!(deployment_dir.join("flake.nix").exists());
        assert!(deployment_dir.join("configuration.nix").exists());
        assert!(deployment_dir.join("modules/api.nix").exists());
        assert!(deployment_dir.join("modules/agent-worker.nix").exists());
        assert!(deployment_dir.join("modules/nats-mesh.nix").exists());
    }
}
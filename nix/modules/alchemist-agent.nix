{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.alchemist.agents;
  
  # Alchemist package
  alchemist = config.services.alchemist.package or (pkgs.rustPlatform.buildRustPackage rec {
    pname = "alchemist";
    version = "0.1.0";
    
    src = ../..;
    
    cargoSha256 = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
    
    buildInputs = with pkgs; [
      openssl
      pkg-config
    ];
  });
  
  # Agent configuration generator
  mkAgentConfig = name: agentCfg: {
    description = "Alchemist agent: ${name}";
    wantedBy = [ "multi-user.target" ];
    after = [ "network.target" ] ++ optional config.services.alchemist.nats.enable "nats.service";
    
    serviceConfig = {
      Type = "notify";
      ExecStart = "${alchemist}/bin/alchemist agent"
        + " --name ${name}"
        + " --capabilities ${concatStringsSep "," agentCfg.capabilities}"
        + optionalString (agentCfg.config != null) " --config ${agentCfg.config}";
      
      Restart = "always";
      RestartSec = 5;
      
      # Resource limits
      ${optionalString (agentCfg.resources.cpu != null) "CPUQuota = \"${agentCfg.resources.cpu}\";"}
      ${optionalString (agentCfg.resources.memory != null) "MemoryLimit = \"${agentCfg.resources.memory}\";"}
      
      # Security configuration based on capabilities
      DynamicUser = !elem "system" agentCfg.capabilities;
      PrivateTmp = true;
      ProtectSystem = if elem "filesystem" agentCfg.capabilities then "full" else "strict";
      ProtectHome = !elem "filesystem" agentCfg.capabilities;
      NoNewPrivileges = !elem "system" agentCfg.capabilities;
      
      # Grant Linux capabilities based on agent capabilities
      AmbientCapabilities = mkMerge [
        (optionals (elem "network" agentCfg.capabilities) [ "CAP_NET_ADMIN" "CAP_NET_RAW" ])
        (optionals (elem "filesystem" agentCfg.capabilities) [ "CAP_DAC_OVERRIDE" ])
        (optionals (elem "process" agentCfg.capabilities) [ "CAP_SYS_PTRACE" "CAP_KILL" ])
        (optionals (elem "system" agentCfg.capabilities) [ "CAP_SYS_ADMIN" ])
      ];
      
      # Additional security hardening
      RestrictNamespaces = !elem "system" agentCfg.capabilities;
      RestrictRealtime = true;
      RestrictSUIDSGID = true;
      RemoveIPC = !elem "process" agentCfg.capabilities;
      LockPersonality = true;
      ProtectClock = !elem "system" agentCfg.capabilities;
      ProtectHostname = !elem "system" agentCfg.capabilities;
      ProtectKernelLogs = !elem "system" agentCfg.capabilities;
      ProtectKernelModules = !elem "system" agentCfg.capabilities;
      ProtectKernelTunables = !elem "system" agentCfg.capabilities;
      ProtectControlGroups = !elem "system" agentCfg.capabilities;
      
      # Network restrictions
      RestrictAddressFamilies = 
        if elem "network" agentCfg.capabilities
        then [ "AF_INET" "AF_INET6" "AF_UNIX" "AF_NETLINK" "AF_PACKET" ]
        else [ "AF_INET" "AF_INET6" "AF_UNIX" ];
      
      # System call filtering
      SystemCallArchitectures = "native";
      SystemCallFilter = mkMerge [
        [ "@system-service" ]
        (optionals (elem "network" agentCfg.capabilities) [ "@network-io" ])
        (optionals (elem "filesystem" agentCfg.capabilities) [ "@file-system" ])
        (optionals (elem "process" agentCfg.capabilities) [ "@process" ])
        (optionals (!elem "system" agentCfg.capabilities) [ "~@privileged" ])
      ];
      
      # Runtime directories
      RuntimeDirectory = "alchemist/agents/${name}";
      StateDirectory = "alchemist/agents/${name}";
      LogsDirectory = "alchemist/agents/${name}";
      
      # Working directory
      WorkingDirectory = "/var/lib/alchemist/agents/${name}";
    };
    
    environment = {
      RUST_LOG = agentCfg.logLevel;
      NATS_URL = config.services.alchemist.nats.url or "nats://localhost:4222";
      AGENT_NAME = name;
      AGENT_CAPABILITIES = concatStringsSep "," agentCfg.capabilities;
    } // agentCfg.environment;
  };
  
  # Agent health monitor
  mkAgentMonitor = name: agentCfg: {
    "alchemist-agent-${name}-monitor" = {
      description = "Monitor for Alchemist agent: ${name}";
      wantedBy = [ "multi-user.target" ];
      after = [ "alchemist-agent-${name}.service" ];
      
      serviceConfig = {
        Type = "simple";
        ExecStart = pkgs.writeShellScript "agent-monitor-${name}" ''
          #!${pkgs.bash}/bin/bash
          
          # Monitor agent health via NATS
          ${pkgs.natscli}/bin/nats sub "agent.${name}.health" \
            --server="${config.services.alchemist.nats.url or "nats://localhost:4222"}" | \
          while read -r line; do
            echo "$(date): $line"
            
            # Parse health status and take action if needed
            if echo "$line" | grep -q '"status":"unhealthy"'; then
              echo "Agent ${name} reported unhealthy status"
              # Could trigger alerts or recovery actions here
            fi
          done
        '';
        
        Restart = "always";
        RestartSec = 10;
      };
    };
  };

in {
  options.services.alchemist.agents = mkOption {
    type = types.attrsOf (types.submodule {
      options = {
        enable = mkOption {
          type = types.bool;
          default = true;
          description = "Whether to enable this agent";
        };
        
        capabilities = mkOption {
          type = types.listOf (types.enum [ "compute" "storage" "network" "filesystem" "process" "system" ]);
          default = [ "compute" ];
          description = "Agent capabilities";
        };
        
        config = mkOption {
          type = types.nullOr types.path;
          default = null;
          description = "Path to agent configuration file";
        };
        
        resources = {
          cpu = mkOption {
            type = types.nullOr types.str;
            default = null;
            description = "CPU limit (e.g., '1000m', '4')";
          };
          
          memory = mkOption {
            type = types.nullOr types.str;
            default = null;
            description = "Memory limit (e.g., '1Gi', '4Gi')";
          };
          
          gpu = mkOption {
            type = types.nullOr types.str;
            default = null;
            description = "GPU resources (e.g., '1', 'nvidia.com/gpu=1')";
          };
        };
        
        monitor = mkOption {
          type = types.bool;
          default = true;
          description = "Whether to enable agent monitoring";
        };
        
        environment = mkOption {
          type = types.attrsOf types.str;
          default = {};
          description = "Environment variables for the agent";
        };
        
        logLevel = mkOption {
          type = types.str;
          default = "info";
          description = "Log level for the agent";
        };
        
        workDir = mkOption {
          type = types.nullOr types.path;
          default = null;
          description = "Working directory for the agent";
        };
        
        maxTasks = mkOption {
          type = types.int;
          default = 10;
          description = "Maximum concurrent tasks the agent can handle";
        };
        
        priority = mkOption {
          type = types.int;
          default = 50;
          description = "Agent priority for task assignment (0-100, higher is more priority)";
        };
      };
    });
    default = {};
    description = "Alchemist agents to run";
  };
  
  config = mkIf (config.services.alchemist.enable or false) {
    # Create systemd services for each configured agent
    systemd.services = mkMerge (
      mapAttrsToList (name: agentCfg:
        mkIf agentCfg.enable (
          { "alchemist-agent-${name}" = mkAgentConfig name agentCfg; }
          // optionalAttrs agentCfg.monitor (mkAgentMonitor name agentCfg)
        )
      ) cfg
    );
    
    # Create necessary directories
    systemd.tmpfiles.rules = mapAttrsToList (name: agentCfg:
      mkIf agentCfg.enable
        "d /var/lib/alchemist/agents/${name} 0750 alchemist alchemist -"
    ) cfg;
    
    # Create alchemist user/group if using DynamicUser=false
    users = mkIf (any (agentCfg: elem "system" agentCfg.capabilities) (attrValues cfg)) {
      users.alchemist = {
        isSystemUser = true;
        group = "alchemist";
        description = "Alchemist agent user";
      };
      groups.alchemist = {};
    };
  };
}
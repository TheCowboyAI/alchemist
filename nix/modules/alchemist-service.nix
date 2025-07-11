{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.alchemist;
  
  # Alchemist package (should be built from the Rust project)
  alchemist = pkgs.rustPlatform.buildRustPackage rec {
    pname = "alchemist";
    version = "0.1.0";
    
    src = ../..;
    
    cargoSha256 = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
    
    buildInputs = with pkgs; [
      openssl
      pkg-config
    ];
  };
  
  # Service configuration generator
  mkServiceConfig = name: serviceCfg: {
    description = "Alchemist service: ${name}";
    wantedBy = [ "multi-user.target" ];
    after = [ "network.target" ] ++ optional cfg.nats.enable "nats.service";
    
    serviceConfig = {
      Type = "notify";
      ExecStart = "${alchemist}/bin/alchemist service"
        + " --name ${name}"
        + " --port ${toString serviceCfg.port}"
        + optionalString (serviceCfg.config != null) " --config ${serviceCfg.config}";
      
      Restart = "always";
      RestartSec = 5;
      
      # Resource limits
      ${optionalString (serviceCfg.resources.cpu != null) "CPUQuota = \"${serviceCfg.resources.cpu}\";"}
      ${optionalString (serviceCfg.resources.memory != null) "MemoryLimit = \"${serviceCfg.resources.memory}\";"}
      
      # Security hardening
      DynamicUser = true;
      PrivateTmp = true;
      ProtectSystem = "strict";
      ProtectHome = true;
      NoNewPrivileges = true;
      RestrictNamespaces = true;
      RestrictRealtime = true;
      RestrictSUIDSGID = true;
      RemoveIPC = true;
      LockPersonality = true;
      ProtectClock = true;
      ProtectHostname = true;
      ProtectKernelLogs = true;
      ProtectKernelModules = true;
      ProtectKernelTunables = true;
      ProtectControlGroups = true;
      RestrictAddressFamilies = [ "AF_INET" "AF_INET6" "AF_UNIX" ];
      SystemCallArchitectures = "native";
      SystemCallFilter = [ "@system-service" "~@privileged" ];
      
      # Runtime directory
      RuntimeDirectory = "alchemist/${name}";
      StateDirectory = "alchemist/${name}";
      LogsDirectory = "alchemist/${name}";
    };
    
    environment = {
      RUST_LOG = serviceCfg.logLevel;
      NATS_URL = cfg.nats.url;
      SERVICE_NAME = name;
    } // serviceCfg.environment;
  };
  
  # Health check timer/service generator
  mkHealthCheck = name: serviceCfg: optionalAttrs (serviceCfg.healthCheck.enable) {
    "alchemist-${name}-health" = {
      description = "Health check for Alchemist service: ${name}";
      serviceConfig = {
        Type = "oneshot";
        ExecStart = pkgs.writeShellScript "health-check-${name}" ''
          #!${pkgs.bash}/bin/bash
          set -e
          
          ${if serviceCfg.healthCheck.http != null then ''
            response=$(${pkgs.curl}/bin/curl -s -o /dev/null -w "%{http_code}" \
              -m ${toString serviceCfg.healthCheck.timeout} \
              "${serviceCfg.healthCheck.http}")
            
            if [ "$response" != "200" ]; then
              echo "Health check failed: HTTP $response"
              exit 1
            fi
          '' else if serviceCfg.healthCheck.tcp != null then ''
            ${pkgs.netcat}/bin/nc -z -w ${toString serviceCfg.healthCheck.timeout} \
              localhost ${toString serviceCfg.healthCheck.tcp} || exit 1
          '' else ''
            # No health check configured
            true
          ''}
        '';
      };
    };
  };
  
  # Health check timer generator
  mkHealthCheckTimer = name: serviceCfg: optionalAttrs (serviceCfg.healthCheck.enable) {
    "alchemist-${name}-health" = {
      wantedBy = [ "timers.target" ];
      timerConfig = {
        OnBootSec = "1m";
        OnUnitActiveSec = "${toString serviceCfg.healthCheck.interval}s";
        Unit = "alchemist-${name}-health.service";
      };
    };
  };

in {
  options.services.alchemist = {
    enable = mkEnableOption "Alchemist services";
    
    package = mkOption {
      type = types.package;
      default = alchemist;
      description = "Alchemist package to use";
    };
    
    nats = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Whether to enable NATS integration";
      };
      
      url = mkOption {
        type = types.str;
        default = "nats://localhost:4222";
        description = "NATS server URL";
      };
    };
    
    services = mkOption {
      type = types.attrsOf (types.submodule {
        options = {
          enable = mkOption {
            type = types.bool;
            default = true;
            description = "Whether to enable this service";
          };
          
          port = mkOption {
            type = types.int;
            description = "Service port";
          };
          
          config = mkOption {
            type = types.nullOr types.path;
            default = null;
            description = "Path to service configuration file";
          };
          
          replicas = mkOption {
            type = types.int;
            default = 1;
            description = "Number of service replicas";
          };
          
          resources = {
            cpu = mkOption {
              type = types.nullOr types.str;
              default = null;
              description = "CPU limit (e.g., '500m', '2')";
            };
            
            memory = mkOption {
              type = types.nullOr types.str;
              default = null;
              description = "Memory limit (e.g., '512Mi', '2Gi')";
            };
          };
          
          healthCheck = {
            enable = mkOption {
              type = types.bool;
              default = true;
              description = "Whether to enable health checks";
            };
            
            http = mkOption {
              type = types.nullOr types.str;
              default = null;
              description = "HTTP health check endpoint (e.g., 'http://localhost:8080/health')";
            };
            
            tcp = mkOption {
              type = types.nullOr types.int;
              default = null;
              description = "TCP health check port";
            };
            
            interval = mkOption {
              type = types.int;
              default = 30;
              description = "Health check interval in seconds";
            };
            
            timeout = mkOption {
              type = types.int;
              default = 5;
              description = "Health check timeout in seconds";
            };
          };
          
          environment = mkOption {
            type = types.attrsOf types.str;
            default = {};
            description = "Environment variables for the service";
          };
          
          logLevel = mkOption {
            type = types.str;
            default = "info";
            description = "Log level for the service";
          };
        };
      });
      default = {};
      description = "Alchemist services to run";
    };
  };
  
  config = mkIf cfg.enable {
    # Create systemd services for each configured service
    systemd.services = mkMerge (
      mapAttrsToList (name: serviceCfg: 
        mkIf serviceCfg.enable (
          # Create service instances based on replicas
          listToAttrs (map (i: {
            name = if serviceCfg.replicas == 1 
              then "alchemist-${name}"
              else "alchemist-${name}-${toString i}";
            value = mkServiceConfig name serviceCfg;
          }) (range 1 serviceCfg.replicas))
          // mkHealthCheck name serviceCfg
        )
      ) cfg.services
    );
    
    # Create health check timers
    systemd.timers = mkMerge (
      mapAttrsToList (name: serviceCfg:
        mkIf (serviceCfg.enable && serviceCfg.healthCheck.enable)
          (mkHealthCheckTimer name serviceCfg)
      ) cfg.services
    );
    
    # Ensure required packages are installed
    environment.systemPackages = [ cfg.package ];
  };
}
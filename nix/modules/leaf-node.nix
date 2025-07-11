{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.alchemist.leafNode;
  
  # Leaf node specific configuration for edge deployments
  leafNodeConfig = pkgs.writeText "alchemist-leaf.conf" ''
    # Alchemist Leaf Node Configuration
    server_name: ${cfg.name}
    
    # Client connections
    port: ${toString cfg.port}
    
    # Leaf node connection to hub
    leafnodes {
      remotes: [
        ${concatMapStringsSep "\n    " (remote: ''
        {
          url: "${remote}"
          ${optionalString (cfg.credentials != null) ''credentials: "${cfg.credentials}"''}
          ${optionalString (cfg.account != null) ''account: "${cfg.account}"''}
          
          # TLS configuration
          ${optionalString cfg.tls.enable ''
          tls {
            cert_file: "${cfg.tls.certFile}"
            key_file: "${cfg.tls.keyFile}"
            ca_file: "${cfg.tls.caFile}"
            verify: true
          }
          ''}
        }
        '') cfg.remotes}
      ]
      
      # Reconnect settings
      reconnect_interval: ${toString cfg.reconnectInterval}s
    }
    
    # Local JetStream for edge caching
    ${optionalString cfg.jetstream.enable ''
    jetstream {
      store_dir: "${cfg.jetstream.storeDir}"
      max_mem: ${cfg.jetstream.maxMemory}
      max_file: ${cfg.jetstream.maxFile}
      
      # Domain for edge isolation
      domain: ${cfg.jetstream.domain}
      
      # Limits for edge deployment
      limits {
        max_consumers: ${toString cfg.jetstream.limits.maxConsumers}
        max_streams: ${toString cfg.jetstream.limits.maxStreams}
      }
    }
    ''}
    
    # Edge-specific optimizations
    write_deadline: "${cfg.writeDeadline}"
    max_pending: ${cfg.maxPending}
    
    # Compression for bandwidth optimization
    compression: ${if cfg.compression then "enabled" else "disabled"}
    
    # Logging
    debug: ${if cfg.debug then "true" else "false"}
    trace: ${if cfg.trace then "true" else "false"}
    logtime: true
    
    # Resource limits for edge devices
    max_connections: ${toString cfg.limits.maxConnections}
    max_subscriptions: ${toString cfg.limits.maxSubscriptions}
    max_payload: ${cfg.limits.maxPayload}
  '';
  
  # Alchemist edge agent configuration
  edgeAgentConfig = {
    # Limited capabilities for edge deployment
    capabilities = [ "compute" "storage" ];
    
    # Edge-specific environment
    environment = {
      EDGE_MODE = "true";
      LEAF_NODE_NAME = cfg.name;
      CACHE_SIZE = cfg.cache.size;
      OFFLINE_MODE_ENABLED = if cfg.offlineMode.enable then "true" else "false";
    };
    
    # Resource constraints for edge devices
    resources = {
      cpu = cfg.resources.cpu;
      memory = cfg.resources.memory;
    };
  };

in {
  options.services.alchemist.leafNode = {
    enable = mkEnableOption "Alchemist leaf node for edge deployment";
    
    name = mkOption {
      type = types.str;
      default = config.networking.hostName;
      description = "Leaf node name";
    };
    
    port = mkOption {
      type = types.int;
      default = 4222;
      description = "Local NATS port";
    };
    
    remotes = mkOption {
      type = types.listOf types.str;
      description = "Remote NATS URLs to connect to";
      example = [ "nats://hub1.example.com:7422" "nats://hub2.example.com:7422" ];
    };
    
    credentials = mkOption {
      type = types.nullOr types.path;
      default = null;
      description = "Path to credentials file for authentication";
    };
    
    account = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Account to use for leaf connection";
    };
    
    reconnectInterval = mkOption {
      type = types.int;
      default = 10;
      description = "Reconnect interval in seconds";
    };
    
    tls = {
      enable = mkOption {
        type = types.bool;
        default = false;
        description = "Enable TLS for leaf connections";
      };
      
      certFile = mkOption {
        type = types.path;
        description = "TLS certificate file";
      };
      
      keyFile = mkOption {
        type = types.path;
        description = "TLS key file";
      };
      
      caFile = mkOption {
        type = types.path;
        description = "TLS CA file";
      };
    };
    
    jetstream = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable local JetStream for edge caching";
      };
      
      storeDir = mkOption {
        type = types.path;
        default = "/var/lib/alchemist/leaf/jetstream";
        description = "JetStream storage directory";
      };
      
      maxMemory = mkOption {
        type = types.str;
        default = "256M";
        description = "Maximum memory for JetStream";
      };
      
      maxFile = mkOption {
        type = types.str;
        default = "1G";
        description = "Maximum file storage for JetStream";
      };
      
      domain = mkOption {
        type = types.str;
        default = "edge";
        description = "JetStream domain for edge isolation";
      };
      
      limits = {
        maxConsumers = mkOption {
          type = types.int;
          default = 100;
          description = "Maximum number of consumers";
        };
        
        maxStreams = mkOption {
          type = types.int;
          default = 10;
          description = "Maximum number of streams";
        };
      };
    };
    
    writeDeadline = mkOption {
      type = types.str;
      default = "10s";
      description = "Write deadline for slow consumers";
    };
    
    maxPending = mkOption {
      type = types.str;
      default = "32MB";
      description = "Maximum pending size";
    };
    
    compression = mkOption {
      type = types.bool;
      default = true;
      description = "Enable compression for bandwidth optimization";
    };
    
    limits = {
      maxConnections = mkOption {
        type = types.int;
        default = 1024;
        description = "Maximum number of connections";
      };
      
      maxSubscriptions = mkOption {
        type = types.int;
        default = 10000;
        description = "Maximum number of subscriptions";
      };
      
      maxPayload = mkOption {
        type = types.str;
        default = "256KB";
        description = "Maximum message payload size";
      };
    };
    
    cache = {
      size = mkOption {
        type = types.str;
        default = "100MB";
        description = "Local cache size for offline operation";
      };
      
      ttl = mkOption {
        type = types.int;
        default = 3600;
        description = "Cache TTL in seconds";
      };
    };
    
    offlineMode = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable offline mode support";
      };
      
      queueSize = mkOption {
        type = types.str;
        default = "50MB";
        description = "Queue size for offline messages";
      };
      
      retryInterval = mkOption {
        type = types.int;
        default = 60;
        description = "Retry interval for queued messages in seconds";
      };
    };
    
    resources = {
      cpu = mkOption {
        type = types.str;
        default = "500m";
        description = "CPU limit for edge deployment";
      };
      
      memory = mkOption {
        type = types.str;
        default = "512Mi";
        description = "Memory limit for edge deployment";
      };
    };
    
    monitoring = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable monitoring";
      };
      
      metricsPort = mkOption {
        type = types.int;
        default = 9090;
        description = "Prometheus metrics port";
      };
      
      healthCheckInterval = mkOption {
        type = types.int;
        default = 30;
        description = "Health check interval in seconds";
      };
    };
    
    debug = mkOption {
      type = types.bool;
      default = false;
      description = "Enable debug logging";
    };
    
    trace = mkOption {
      type = types.bool;
      default = false;
      description = "Enable trace logging";
    };
  };
  
  config = mkIf cfg.enable {
    # NATS leaf node service
    systemd.services.alchemist-leaf-nats = {
      description = "Alchemist NATS leaf node";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];
      
      serviceConfig = {
        Type = "simple";
        ExecStart = "${pkgs.nats-server}/bin/nats-server -c ${leafNodeConfig}";
        ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
        Restart = "always";
        RestartSec = 5;
        
        # Run with limited privileges
        DynamicUser = true;
        StateDirectory = "alchemist/leaf";
        RuntimeDirectory = "alchemist/leaf";
        LogsDirectory = "alchemist/leaf";
        
        # Resource limits
        CPUQuota = cfg.resources.cpu;
        MemoryLimit = cfg.resources.memory;
        
        # Security hardening
        PrivateTmp = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        NoNewPrivileges = true;
        RestrictAddressFamilies = [ "AF_INET" "AF_INET6" "AF_UNIX" ];
        SystemCallFilter = [ "@system-service" "~@privileged" ];
      };
      
      environment = {
        GOMEMLIMIT = cfg.resources.memory;
      };
    };
    
    # Edge agent service
    systemd.services.alchemist-edge-agent = {
      description = "Alchemist edge agent";
      wantedBy = [ "multi-user.target" ];
      after = [ "alchemist-leaf-nats.service" ];
      requires = [ "alchemist-leaf-nats.service" ];
      
      serviceConfig = {
        Type = "notify";
        ExecStart = "${config.services.alchemist.package}/bin/alchemist agent"
          + " --name edge-${cfg.name}"
          + " --capabilities ${concatStringsSep "," edgeAgentConfig.capabilities}"
          + " --edge-mode";
        
        Restart = "always";
        RestartSec = 10;
        
        # Resource limits
        CPUQuota = cfg.resources.cpu;
        MemoryLimit = cfg.resources.memory;
        
        # Security
        DynamicUser = true;
        StateDirectory = "alchemist/edge-agent";
        RuntimeDirectory = "alchemist/edge-agent";
        LogsDirectory = "alchemist/edge-agent";
        
        # Hardening
        PrivateTmp = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        NoNewPrivileges = true;
      };
      
      environment = edgeAgentConfig.environment // {
        NATS_URL = "nats://localhost:${toString cfg.port}";
        RUST_LOG = if cfg.debug then "debug" else "info";
      };
    };
    
    # Offline queue manager
    systemd.services.alchemist-offline-queue = mkIf cfg.offlineMode.enable {
      description = "Alchemist offline queue manager";
      wantedBy = [ "multi-user.target" ];
      after = [ "alchemist-leaf-nats.service" ];
      
      serviceConfig = {
        Type = "simple";
        ExecStart = pkgs.writeShellScript "offline-queue-manager" ''
          #!${pkgs.bash}/bin/bash
          
          QUEUE_DIR="/var/lib/alchemist/leaf/offline-queue"
          mkdir -p "$QUEUE_DIR"
          
          while true; do
            # Check connectivity to remotes
            CONNECTED=false
            for remote in ${concatStringsSep " " cfg.remotes}; do
              if ${pkgs.netcat}/bin/nc -z -w 5 $(echo $remote | sed 's|nats://||' | cut -d: -f1) $(echo $remote | sed 's|.*:||' | sed 's|/.*||') 2>/dev/null; then
                CONNECTED=true
                break
              fi
            done
            
            if [ "$CONNECTED" = "true" ]; then
              # Process queued messages
              for msg in "$QUEUE_DIR"/*.msg 2>/dev/null; do
                if [ -f "$msg" ]; then
                  # Attempt to send message
                  if ${pkgs.natscli}/bin/nats --server=localhost:${toString cfg.port} pub --input="$msg"; then
                    rm "$msg"
                  fi
                fi
              done
            fi
            
            sleep ${toString cfg.offlineMode.retryInterval}
          done
        '';
        
        Restart = "always";
        RestartSec = 10;
        
        # Security
        DynamicUser = true;
        StateDirectory = "alchemist/leaf/offline-queue";
      };
    };
    
    # Monitoring endpoint
    systemd.services.alchemist-leaf-monitor = mkIf cfg.monitoring.enable {
      description = "Alchemist leaf node monitor";
      wantedBy = [ "multi-user.target" ];
      after = [ "alchemist-leaf-nats.service" ];
      
      serviceConfig = {
        Type = "simple";
        ExecStart = pkgs.writeShellScript "leaf-monitor" ''
          #!${pkgs.bash}/bin/bash
          
          ${pkgs.prometheus-node-exporter}/bin/node_exporter \
            --web.listen-address=":${toString cfg.monitoring.metricsPort}" \
            --collector.textfile.directory=/var/lib/alchemist/leaf/metrics \
            --no-collector.arp \
            --no-collector.bcache \
            --no-collector.bonding \
            --no-collector.conntrack \
            --no-collector.cpufreq \
            --no-collector.diskstats \
            --no-collector.edac \
            --no-collector.entropy \
            --no-collector.filefd \
            --no-collector.filesystem \
            --no-collector.hwmon \
            --no-collector.infiniband \
            --no-collector.ipvs \
            --no-collector.mdadm \
            --no-collector.meminfo \
            --no-collector.netclass \
            --no-collector.netdev \
            --no-collector.netstat \
            --no-collector.nfs \
            --no-collector.nfsd \
            --no-collector.pressure \
            --no-collector.rapl \
            --no-collector.schedstat \
            --no-collector.sockstat \
            --no-collector.softnet \
            --no-collector.stat \
            --no-collector.thermal_zone \
            --no-collector.time \
            --no-collector.timex \
            --no-collector.udp_queues \
            --no-collector.uname \
            --no-collector.vmstat \
            --no-collector.xfs \
            --no-collector.zfs &
          
          # Health check loop
          while true; do
            # Check NATS connectivity
            if ${pkgs.natscli}/bin/nats --server=localhost:${toString cfg.port} server ping 2>/dev/null; then
              echo "leaf_node_health{status=\"healthy\"} 1" > /var/lib/alchemist/leaf/metrics/health.prom
            else
              echo "leaf_node_health{status=\"unhealthy\"} 0" > /var/lib/alchemist/leaf/metrics/health.prom
            fi
            
            # Check remote connectivity
            CONNECTED=0
            for remote in ${concatStringsSep " " cfg.remotes}; do
              if ${pkgs.netcat}/bin/nc -z -w 5 $(echo $remote | sed 's|nats://||' | cut -d: -f1) $(echo $remote | sed 's|.*:||' | sed 's|/.*||') 2>/dev/null; then
                CONNECTED=1
                break
              fi
            done
            echo "leaf_node_connected{} $CONNECTED" >> /var/lib/alchemist/leaf/metrics/health.prom
            
            sleep ${toString cfg.monitoring.healthCheckInterval}
          done
        '';
        
        Restart = "always";
        RestartSec = 10;
        
        # Security
        DynamicUser = true;
        StateDirectory = "alchemist/leaf/metrics";
      };
    };
    
    # Create necessary directories
    systemd.tmpfiles.rules = [
      "d ${cfg.jetstream.storeDir} 0750 alchemist alchemist -"
      "d /var/lib/alchemist/leaf/offline-queue 0750 alchemist alchemist -"
      "d /var/lib/alchemist/leaf/cache 0750 alchemist alchemist -"
      "d /var/lib/alchemist/leaf/metrics 0750 alchemist alchemist -"
    ];
    
    # Open firewall ports
    networking.firewall.allowedTCPPorts = [ cfg.port ] 
      ++ optional cfg.monitoring.enable cfg.monitoring.metricsPort;
    
    # Install required packages
    environment.systemPackages = with pkgs; [
      nats-server
      natscli
      netcat
    ] ++ optional cfg.monitoring.enable prometheus-node-exporter;
  };
}
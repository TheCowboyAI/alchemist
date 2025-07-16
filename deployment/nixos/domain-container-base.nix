{ config, pkgs, lib, ... }:

with lib;

{
  options.services.alchemist-domain = {
    enable = mkEnableOption "Alchemist domain service";
    
    domain = mkOption {
      type = types.str;
      description = "Domain name (e.g., graph, workflow, agent)";
    };
    
    port = mkOption {
      type = types.port;
      default = 8080;
      description = "HTTP port for the domain service";
    };
    
    natsUrl = mkOption {
      type = types.str;
      default = "nats://localhost:4222";
      description = "NATS server URL";
    };
    
    dependencies = mkOption {
      type = types.listOf types.str;
      default = [];
      description = "List of domain dependencies";
    };
    
    settings = mkOption {
      type = types.attrs;
      default = {};
      description = "Additional domain-specific settings";
    };
  };
  
  config = mkIf config.services.alchemist-domain.enable {
    # Domain service
    systemd.services."alchemist-domain-${config.services.alchemist-domain.domain}" = {
      description = "Alchemist ${config.services.alchemist-domain.domain} domain";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];
      
      environment = {
        RUST_LOG = "info";
        DOMAIN_NAME = config.services.alchemist-domain.domain;
        NATS_URL = config.services.alchemist-domain.natsUrl;
        HTTP_PORT = toString config.services.alchemist-domain.port;
        DOMAIN_CONFIG = builtins.toJSON config.services.alchemist-domain.settings;
      };
      
      serviceConfig = {
        Type = "notify";
        ExecStart = "${pkgs.alchemist}/bin/alchemist-domain-${config.services.alchemist-domain.domain}";
        Restart = "always";
        RestartSec = 10;
        User = "alchemist";
        Group = "alchemist";
        
        # Security
        NoNewPrivileges = true;
        PrivateTmp = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        ReadWritePaths = [ "/var/lib/alchemist/${config.services.alchemist-domain.domain}" ];
        
        # Resource limits based on domain settings
        MemoryMax = mkDefault (
          if config.services.alchemist-domain.settings ? max_memory_mb
          then "${toString config.services.alchemist-domain.settings.max_memory_mb}M"
          else "1G"
        );
        CPUQuota = mkDefault (
          if config.services.alchemist-domain.settings ? worker_threads
          then "${toString (config.services.alchemist-domain.settings.worker_threads * 100)}%"
          else "200%"
        );
      };
      
      # Wait for dependencies
      preStart = concatStringsSep "\n" (map (dep: ''
        until ${pkgs.curl}/bin/curl -s http://10.233.${toString (2 + (elemAt ["graph" "workflow" "agent"] (elemAt ["graph" "workflow" "agent"] dep)))}.2:808${toString (1 + (elemAt ["graph" "workflow" "agent"] (elemAt ["graph" "workflow" "agent"] dep)))}/health; do
          echo "Waiting for ${dep} domain..."
          sleep 5
        done
      '') config.services.alchemist-domain.dependencies);
    };
    
    # Create user if not exists
    users.users.alchemist = mkDefault {
      isSystemUser = true;
      group = "alchemist";
      home = "/var/lib/alchemist";
      createHome = true;
    };
    
    users.groups.alchemist = mkDefault {};
    
    # Create domain-specific data directory
    systemd.tmpfiles.rules = [
      "d /var/lib/alchemist/${config.services.alchemist-domain.domain} 0750 alchemist alchemist -"
    ];
    
    # Firewall
    networking.firewall.allowedTCPPorts = [ config.services.alchemist-domain.port 9090 ];
    
    # Monitoring
    services.prometheus.exporters.node = {
      enable = true;
      port = 9100;
      enabledCollectors = [
        "systemd"
        "processes"
        "filesystem"
        "meminfo"
        "netstat"
      ];
    };
    
    # Domain metrics exporter
    systemd.services."alchemist-metrics-${config.services.alchemist-domain.domain}" = {
      description = "Alchemist ${config.services.alchemist-domain.domain} metrics exporter";
      after = [ "alchemist-domain-${config.services.alchemist-domain.domain}.service" ];
      wantedBy = [ "multi-user.target" ];
      
      serviceConfig = {
        Type = "simple";
        ExecStart = "${pkgs.alchemist}/bin/alchemist-metrics --domain ${config.services.alchemist-domain.domain} --port 9090";
        Restart = "always";
        User = "alchemist";
        Group = "alchemist";
      };
    };
  };
}
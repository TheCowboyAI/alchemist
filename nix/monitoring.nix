# CIM Monitoring Stack Configuration
{ config, pkgs, lib, ... }:

{
  # Prometheus for metrics collection
  services.prometheus = {
    enable = true;
    port = 9090;
    
    globalConfig = {
      scrape_interval = "15s";
      evaluation_interval = "15s";
    };
    
    scrapeConfigs = [
      {
        job_name = "nats";
        static_configs = [{
          targets = [
            "localhost:8222"  # NATS monitoring endpoint
          ];
        }];
      }
      {
        job_name = "node";
        static_configs = [{
          targets = [ "localhost:9100" ];
        }];
      }
      {
        job_name = "nvidia_gpu";
        static_configs = [{
          targets = [ "localhost:9835" ];
        }];
      }
      {
        job_name = "cim_application";
        static_configs = [{
          targets = [ "localhost:9091" ];  # CIM metrics endpoint
        }];
      }
    ];
    
    ruleFiles = [
      ./monitoring/alerts.yaml
    ];
  };
  
  # Grafana for visualization
  services.grafana = {
    enable = true;
    settings = {
      server = {
        http_port = 3000;
        domain = "localhost";
      };
      
      security = {
        admin_user = "admin";
        admin_password = "$__file{/var/lib/grafana/admin-password}";
      };
    };
    
    provision = {
      enable = true;
      
      datasources.settings.datasources = [
        {
          name = "Prometheus";
          type = "prometheus";
          url = "http://localhost:9090";
          isDefault = true;
        }
      ];
      
      dashboards.settings.providers = [
        {
          name = "CIM Dashboards";
          folder = "CIM";
          type = "file";
          options.path = ./dashboards;
        }
      ];
    };
  };
  
  # CIM Event Stream Monitoring Service
  # This service subscribes to NATS event streams and converts them to Prometheus metrics
  systemd.services.cim-event-monitor = {
    description = "CIM Event Stream Monitor";
    wantedBy = [ "multi-user.target" ];
    after = [ "network.target" "nats.service" ];
    
    serviceConfig = {
      Type = "simple";
      ExecStart = "${pkgs.cim-event-monitor}/bin/cim-event-monitor";
      Restart = "always";
      RestartSec = "10s";
      
      Environment = [
        "NATS_URL=nats://localhost:4222"
        "METRICS_PORT=9091"
        "EVENT_SUBJECTS=cim.>"
      ];
    };
  };
  

  
  # Node exporter for system metrics
  services.prometheus.exporters.node = {
    enable = true;
    port = 9100;
    enabledCollectors = [
      "cpu"
      "diskstats"
      "filesystem"
      "loadavg"
      "meminfo"
      "netdev"
      "stat"
      "time"
      "uname"
      "systemd"
    ];
  };
  
  # NVIDIA GPU exporter (if NVIDIA GPU present)
  systemd.services.nvidia-gpu-prometheus-exporter = lib.mkIf config.hardware.nvidia.modesetting.enable {
    description = "NVIDIA GPU Prometheus Exporter";
    wantedBy = [ "multi-user.target" ];
    after = [ "network.target" ];
    
    serviceConfig = {
      ExecStart = "${pkgs.nvidia-gpu-prometheus-exporter}/bin/nvidia-gpu-prometheus-exporter";
      Restart = "always";
      RestartSec = "30s";
    };
  };
  
  # Alertmanager for alert routing
  services.prometheus.alertmanager = {
    enable = true;
    port = 9093;
    
    configuration = {
      global = {
        smtp_smarthost = "localhost:25";
        smtp_from = "alertmanager@cim.local";
      };
      
      route = {
        group_by = [ "alertname" "cluster" "service" ];
        group_wait = "10s";
        group_interval = "10s";
        repeat_interval = "12h";
        receiver = "default";
        
        routes = [
          {
            match = {
              severity = "critical";
            };
            receiver = "critical";
          }
        ];
      };
      
      receivers = [
        {
          name = "default";
          webhook_configs = [
            {
              url = "http://localhost:8080/alerts";
              send_resolved = true;
            }
          ];
        }
        {
          name = "critical";
          webhook_configs = [
            {
              url = "http://localhost:8080/critical-alerts";
              send_resolved = true;
            }
          ];
        }
      ];
    };
  };
  
  # Open firewall ports
  networking.firewall.allowedTCPPorts = [
    9090  # Prometheus
    9091  # CIM event monitor metrics
    9093  # Alertmanager
    3000  # Grafana
    9100  # Node exporter
    9835  # NVIDIA GPU exporter
  ];
} 
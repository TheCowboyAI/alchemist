{ config, pkgs, lib, ... }:

{
  # Extra containers configuration for distributed Alchemist deployment
  
  # Graph domain container
  containers.alchemist-graph = {
    autoStart = true;
    privateNetwork = true;
    hostAddress = "10.233.2.1";
    localAddress = "10.233.2.2";
    
    config = { config, pkgs, ... }: {
      imports = [ ./domain-container-base.nix ];
      
      services.alchemist-domain = {
        enable = true;
        domain = "graph";
        port = 8081;
        natsUrl = "nats://10.233.1.2:4222";
        
        settings = {
          max_memory_mb = 2048;
          timeout_seconds = 60;
          worker_threads = 4;
        };
      };
    };
  };
  
  # Workflow domain container
  containers.alchemist-workflow = {
    autoStart = true;
    privateNetwork = true;
    hostAddress = "10.233.3.1";
    localAddress = "10.233.3.2";
    
    config = { config, pkgs, ... }: {
      imports = [ ./domain-container-base.nix ];
      
      services.alchemist-domain = {
        enable = true;
        domain = "workflow";
        port = 8082;
        natsUrl = "nats://10.233.1.2:4222";
        dependencies = [ "graph" ];
        
        settings = {
          max_memory_mb = 1024;
          timeout_seconds = 120;
          worker_threads = 2;
        };
      };
    };
  };
  
  # Agent domain container
  containers.alchemist-agent = {
    autoStart = true;
    privateNetwork = true;
    hostAddress = "10.233.4.1";
    localAddress = "10.233.4.2";
    
    config = { config, pkgs, ... }: {
      imports = [ ./domain-container-base.nix ];
      
      services.alchemist-domain = {
        enable = true;
        domain = "agent";
        port = 8083;
        natsUrl = "nats://10.233.1.2:4222";
        dependencies = [ "graph" ];
        
        settings = {
          max_memory_mb = 4096;
          timeout_seconds = 180;
          worker_threads = 8;
          
          # AI provider settings
          ai_providers = {
            openai = {
              enabled = true;
              api_key = "@OPENAI_API_KEY@";
              max_concurrent = 5;
            };
            anthropic = {
              enabled = true;
              api_key = "@ANTHROPIC_API_KEY@";
              max_concurrent = 5;
            };
            ollama = {
              enabled = true;
              endpoint = "http://10.233.5.2:11434";
              max_concurrent = 2;
            };
          };
        };
      };
    };
  };
  
  # Ollama container for local AI models
  containers.ollama = {
    autoStart = true;
    privateNetwork = true;
    hostAddress = "10.233.5.1";
    localAddress = "10.233.5.2";
    
    config = { config, pkgs, ... }: {
      environment.systemPackages = with pkgs; [ ollama ];
      
      systemd.services.ollama = {
        description = "Ollama Local AI Models";
        after = [ "network.target" ];
        wantedBy = [ "multi-user.target" ];
        
        serviceConfig = {
          Type = "simple";
          ExecStart = "${pkgs.ollama}/bin/ollama serve";
          Restart = "always";
          User = "ollama";
          Group = "ollama";
          
          Environment = [
            "OLLAMA_HOST=0.0.0.0"
            "OLLAMA_MODELS=/var/lib/ollama/models"
            "OLLAMA_NUM_PARALLEL=2"
            "OLLAMA_MAX_LOADED_MODELS=2"
            "OLLAMA_KEEP_ALIVE=5m"
          ];
          
          StateDirectory = "ollama";
          MemoryHigh = "12G";
          MemoryMax = "16G";
        };
        
        # Pre-load models
        postStart = ''
          ${pkgs.ollama}/bin/ollama pull llama3:70b
          ${pkgs.ollama}/bin/ollama pull codellama:34b
        '';
      };
      
      users.users.ollama = {
        isSystemUser = true;
        group = "ollama";
        home = "/var/lib/ollama";
        createHome = true;
      };
      
      users.groups.ollama = {};
      
      networking.firewall.allowedTCPPorts = [ 11434 ];
    };
  };
  
  # Monitoring container with Prometheus and Grafana
  containers.monitoring = {
    autoStart = true;
    privateNetwork = true;
    hostAddress = "10.233.6.1";
    localAddress = "10.233.6.2";
    
    config = { config, pkgs, ... }: {
      services.prometheus = {
        enable = true;
        port = 9091;
        
        globalConfig = {
          scrape_interval = "15s";
          evaluation_interval = "15s";
        };
        
        scrapeConfigs = [
          {
            job_name = "alchemist";
            static_configs = [
              {
                targets = [
                  "10.233.1.2:9090"  # Main alchemist
                  "10.233.2.2:9090"  # Graph domain
                  "10.233.3.2:9090"  # Workflow domain
                  "10.233.4.2:9090"  # Agent domain
                ];
              }
            ];
          }
          {
            job_name = "nats";
            static_configs = [
              {
                targets = [ "10.233.1.2:8222" ];
              }
            ];
          }
          {
            job_name = "node";
            static_configs = [
              {
                targets = [
                  "10.233.1.2:9100"
                  "10.233.2.2:9100"
                  "10.233.3.2:9100"
                  "10.233.4.2:9100"
                  "10.233.5.2:9100"
                ];
              }
            ];
          }
        ];
        
        rules = [
          (pkgs.writeText "alchemist-rules.yml" ''
            groups:
            - name: alchemist
              rules:
              - alert: HighErrorRate
                expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.05
                for: 5m
                labels:
                  severity: critical
                annotations:
                  summary: "High error rate on {{ $labels.instance }}"
                  
              - alert: DomainDown
                expr: up{job="alchemist"} == 0
                for: 1m
                labels:
                  severity: critical
                annotations:
                  summary: "Domain {{ $labels.instance }} is down"
                  
              - alert: HighMemoryUsage
                expr: (node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes) / node_memory_MemTotal_bytes > 0.9
                for: 5m
                labels:
                  severity: warning
                annotations:
                  summary: "High memory usage on {{ $labels.instance }}"
          '')
        ];
      };
      
      services.grafana = {
        enable = true;
        settings = {
          server = {
            http_addr = "0.0.0.0";
            http_port = 3000;
            root_url = "https://monitoring.alchemist.local";
          };
          
          security = {
            admin_password = "@GRAFANA_PASSWORD@";
            secret_key = "@GRAFANA_SECRET@";
          };
          
          analytics.reporting_enabled = false;
        };
        
        provision = {
          enable = true;
          
          datasources.settings.datasources = [
            {
              name = "Prometheus";
              type = "prometheus";
              url = "http://localhost:9091";
              isDefault = true;
            }
          ];
          
          dashboards.settings.providers = [
            {
              name = "Alchemist Dashboards";
              folder = "Alchemist";
              type = "file";
              options.path = ./dashboards;
            }
          ];
        };
      };
      
      networking.firewall.allowedTCPPorts = [ 9091 3000 ];
    };
  };
}
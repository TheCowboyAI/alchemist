{ config, pkgs, lib, nodeName, clusterNodes, ... }:

let
  nodeIndex = lib.strings.toInt (lib.strings.removePrefix "control" nodeName);
  otherNodes = lib.filter (n: n != nodeName) clusterNodes;
in {
  # Control plane configuration
  services.alchemist = {
    enable = true;
    
    services = {
      # API service with HA
      api = {
        enable = true;
        port = 8080;
        replicas = 2;  # 2 replicas per node
        resources = {
          cpu = "2";
          memory = "4Gi";
        };
        environment = {
          NODE_NAME = nodeName;
          CLUSTER_NODES = lib.concatStringsSep "," clusterNodes;
          ENVIRONMENT = "production";
        };
        healthCheck = {
          enable = true;
          http = "http://localhost:8080/health";
          interval = 10;
          timeout = 5;
          retries = 3;
        };
      };
      
      # Scheduler service
      scheduler = {
        enable = nodeIndex == 1;  # Only on first control node
        port = 8081;
        replicas = 1;
        resources = {
          cpu = "1";
          memory = "2Gi";
        };
        environment = {
          NODE_NAME = nodeName;
          ENVIRONMENT = "production";
        };
      };
      
      # Metrics aggregator
      metrics = {
        enable = true;
        port = 9090;
        replicas = 1;
        resources = {
          cpu = "500m";
          memory = "1Gi";
        };
      };
    };
  };
  
  # NATS cluster configuration
  services.nats-mesh = {
    enable = true;
    clusterName = "alchemist-prod";
    
    nodes = [{
      name = "nats-${nodeName}";
      host = "0.0.0.0";
      clientPort = 4222;
      clusterPort = 6222;
      monitorPort = 8222;
      leafPort = 7422;
      routes = map (n: "${n}.prod.example.com:6222") otherNodes;
    }];
    
    jetstream = {
      storeDir = "/data/nats/jetstream";
      maxMemory = "8G";
      maxFile = "100G";
      domain = "production";
    };
    
    # Production accounts
    accounts = {
      SYS = {
        jetstream = false;
        limits = {
          maxConnections = 100;
        };
      };
      
      ALCHEMIST = {
        jetstream = true;
        limits = {
          maxConnections = 10000;
          maxPayload = "10MB";
          maxPending = "1GB";
        };
      };
      
      MONITORING = {
        jetstream = false;
        limits = {
          maxConnections = 50;
        };
      };
    };
    
    systemAccount = "SYS";
    
    # TLS configuration
    tls = {
      enable = true;
      certFile = "/etc/alchemist/tls/server.crt";
      keyFile = "/etc/alchemist/tls/server.key";
      caFile = "/etc/alchemist/tls/ca.crt";
    };
    
    leafnodes.tls = {
      enable = true;
      certFile = "/etc/alchemist/tls/leaf.crt";
      keyFile = "/etc/alchemist/tls/leaf.key";
      caFile = "/etc/alchemist/tls/ca.crt";
    };
  };
  
  # Load balancer configuration (using HAProxy)
  services.haproxy = {
    enable = true;
    config = ''
      global
        log /dev/log local0
        maxconn 4096
        
      defaults
        mode http
        timeout connect 5000ms
        timeout client 50000ms
        timeout server 50000ms
        option httplog
        
      frontend api_frontend
        bind *:443 ssl crt /etc/alchemist/tls/alchemist.pem
        default_backend api_backend
        
      backend api_backend
        balance roundrobin
        option httpchk GET /health
        ${lib.concatMapStringsSep "\n" (node: 
          "server ${node} ${node}.prod.example.com:8080 check ssl verify none"
        ) clusterNodes}
        
      frontend nats_frontend
        mode tcp
        bind *:4222
        default_backend nats_backend
        
      backend nats_backend
        mode tcp
        balance roundrobin
        ${lib.concatMapStringsSep "\n" (node: 
          "server ${node} ${node}.prod.example.com:4222 check"
        ) clusterNodes}
    '';
  };
  
  # PostgreSQL for persistent storage
  services.postgresql = {
    enable = true;
    package = pkgs.postgresql_15;
    
    ensureDatabases = [ "alchemist" ];
    ensureUsers = [{
      name = "alchemist";
      ensurePermissions = {
        "DATABASE alchemist" = "ALL PRIVILEGES";
      };
    }];
    
    settings = {
      shared_buffers = "2GB";
      effective_cache_size = "6GB";
      maintenance_work_mem = "512MB";
      checkpoint_completion_target = 0.9;
      wal_buffers = "16MB";
      default_statistics_target = 100;
      random_page_cost = 1.1;
      effective_io_concurrency = 200;
      work_mem = "10MB";
      min_wal_size = "1GB";
      max_wal_size = "4GB";
    };
    
    # Enable replication
    enableTCPIP = true;
    authentication = ''
      host replication all ${nodeName}.prod.example.com trust
      ${lib.concatMapStringsSep "\n" (node: 
        "host replication all ${node}.prod.example.com trust"
      ) otherNodes}
    '';
  };
  
  # Monitoring and alerting
  services.prometheus = {
    enable = true;
    port = 9090;
    
    scrapeConfigs = [
      {
        job_name = "alchemist";
        static_configs = [{
          targets = lib.flatten (map (node: [
            "${node}.prod.example.com:8080"
            "${node}.prod.example.com:8081"
            "${node}.prod.example.com:8222"
          ]) clusterNodes);
        }];
      }
    ];
    
    rules = [
      ''
        groups:
          - name: alchemist_alerts
            rules:
              - alert: ServiceDown
                expr: up == 0
                for: 5m
                labels:
                  severity: critical
                annotations:
                  summary: "Service {{ $labels.instance }} is down"
                  
              - alert: HighMemoryUsage
                expr: (1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) > 0.9
                for: 5m
                labels:
                  severity: warning
                annotations:
                  summary: "High memory usage on {{ $labels.instance }}"
      ''
    ];
  };
  
  # Alertmanager
  services.prometheus.alertmanager = {
    enable = true;
    configuration = {
      route = {
        group_by = [ "alertname" ];
        group_wait = "10s";
        group_interval = "10s";
        repeat_interval = "1h";
        receiver = "team-notifications";
      };
      
      receivers = [{
        name = "team-notifications";
        webhook_configs = [{
          url = "https://alerts.example.com/webhook";
        }];
      }];
    };
  };
  
  # System configuration
  boot.kernel.sysctl = {
    "net.core.rmem_max" = 134217728;
    "net.core.wmem_max" = 134217728;
    "net.ipv4.tcp_rmem" = "4096 87380 134217728";
    "net.ipv4.tcp_wmem" = "4096 65536 134217728";
    "net.core.netdev_max_backlog" = 5000;
    "net.ipv4.tcp_congestion_control" = "bbr";
  };
  
  # Storage configuration
  fileSystems."/data" = {
    device = "/dev/disk/by-label/data";
    fsType = "ext4";
    options = [ "noatime" "nodiratime" ];
  };
  
  # Backup configuration
  services.restic.backups.alchemist = {
    enable = true;
    paths = [
      "/var/lib/alchemist"
      "/data/nats/jetstream"
      "/var/lib/postgresql"
    ];
    repository = "s3:s3.amazonaws.com/alchemist-backups/${nodeName}";
    passwordFile = "/etc/alchemist/backup-password";
    timerConfig = {
      OnCalendar = "daily";
    };
    pruneOpts = [
      "--keep-daily 7"
      "--keep-weekly 4"
      "--keep-monthly 12"
    ];
  };
}
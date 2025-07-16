{ config, pkgs, lib, ... }:

{
  # Alchemist production container configuration for NixOS
  containers.alchemist = {
    autoStart = true;
    privateNetwork = true;
    hostAddress = "10.233.1.1";
    localAddress = "10.233.1.2";
    
    config = { config, pkgs, ... }: {
      # System packages
      environment.systemPackages = with pkgs; [
        alchemist
        nats-server
        postgresql_15
        redis
        qdrant
      ];
      
      # Alchemist service
      systemd.services.alchemist = {
        description = "Alchemist - Composable Information Machine";
        after = [ "network.target" "nats.service" "postgresql.service" "redis.service" ];
        wantedBy = [ "multi-user.target" ];
        
        environment = {
          RUST_LOG = "warn";
          ALCHEMIST_CONFIG = "/etc/alchemist/config.toml";
          OPENAI_API_KEY = "@OPENAI_API_KEY@";  # Replaced by secrets management
          ANTHROPIC_API_KEY = "@ANTHROPIC_API_KEY@";  # Replaced by secrets management
          ALCHEMIST_JWT_SECRET = "@JWT_SECRET@";  # Replaced by secrets management
          ALCHEMIST_API_KEY = "@API_KEY@";  # Replaced by secrets management
        };
        
        serviceConfig = {
          Type = "notify";
          ExecStart = "${pkgs.alchemist}/bin/alchemist";
          Restart = "always";
          RestartSec = 10;
          User = "alchemist";
          Group = "alchemist";
          
          # Security hardening
          NoNewPrivileges = true;
          PrivateTmp = true;
          ProtectSystem = "strict";
          ProtectHome = true;
          ReadWritePaths = [ "/var/lib/alchemist" ];
          
          # Resource limits
          LimitNOFILE = 65536;
          LimitNPROC = 4096;
          MemoryHigh = "6G";
          MemoryMax = "8G";
          CPUQuota = "400%";
        };
      };
      
      # NATS JetStream service
      services.nats = {
        enable = true;
        jetstream = true;
        dataDir = "/var/lib/nats";
        
        settings = {
          server_name = "alchemist-nats";
          port = 4222;
          http_port = 8222;
          
          jetstream = {
            store_dir = "/var/lib/nats/jetstream";
            max_memory_store = "2G";
            max_file_store = "100G";
          };
          
          cluster = {
            name = "alchemist-cluster";
            port = 6222;
          };
          
          leafnodes = {
            port = 7422;
          };
          
          tls = {
            cert_file = "/etc/alchemist/certs/nats-cert.pem";
            key_file = "/etc/alchemist/certs/nats-key.pem";
            ca_file = "/etc/alchemist/certs/ca.pem";
            verify = true;
          };
          
          authorization = {
            users = [
              {
                user = "alchemist";
                password = "$2a$11$W8xLmO1N.hZOYjNlLYJlT.VYx7jB5fkPtWKQN1wTJKhm8VmFkFGC";  # bcrypt hash
                permissions = {
                  publish = ">";
                  subscribe = ">";
                };
              }
            ];
          };
        };
      };
      
      # PostgreSQL service
      services.postgresql = {
        enable = true;
        package = pkgs.postgresql_15;
        dataDir = "/var/lib/postgresql/15";
        
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
          max_worker_processes = 8;
          max_parallel_workers_per_gather = 4;
          max_parallel_workers = 8;
          max_parallel_maintenance_workers = 4;
        };
        
        authentication = ''
          local all all trust
          host all all 127.0.0.1/32 trust
          host all all ::1/128 trust
          host all all 10.233.1.0/24 scram-sha-256
        '';
        
        initialScript = pkgs.writeText "init.sql" ''
          CREATE DATABASE alchemist;
          CREATE USER alchemist WITH ENCRYPTED PASSWORD '@POSTGRES_PASSWORD@';
          GRANT ALL PRIVILEGES ON DATABASE alchemist TO alchemist;
        '';
        
        ensureDatabases = [ "alchemist" ];
        ensureUsers = [
          {
            name = "alchemist";
            ensurePermissions = {
              "DATABASE alchemist" = "ALL PRIVILEGES";
            };
          }
        ];
      };
      
      # Redis service
      services.redis.servers."alchemist" = {
        enable = true;
        port = 6379;
        bind = "127.0.0.1 ::1 10.233.1.2";
        
        settings = {
          protected-mode = "yes";
          requirepass = "@REDIS_PASSWORD@";  # Replaced by secrets management
          maxmemory = "2gb";
          maxmemory-policy = "allkeys-lru";
          save = [ "900 1" "300 10" "60 10000" ];
          appendonly = "yes";
          appendfsync = "everysec";
        };
      };
      
      # Qdrant vector database
      systemd.services.qdrant = {
        description = "Qdrant Vector Database";
        after = [ "network.target" ];
        wantedBy = [ "multi-user.target" ];
        
        serviceConfig = {
          Type = "simple";
          ExecStart = "${pkgs.qdrant}/bin/qdrant";
          Restart = "always";
          User = "qdrant";
          Group = "qdrant";
          
          Environment = [
            "QDRANT__SERVICE__HTTP_PORT=6333"
            "QDRANT__SERVICE__GRPC_PORT=6334"
            "QDRANT__STORAGE__STORAGE_PATH=/var/lib/qdrant/storage"
            "QDRANT__STORAGE__SNAPSHOTS_PATH=/var/lib/qdrant/snapshots"
            "QDRANT__TLS__ENABLE=true"
            "QDRANT__TLS__CERT=/etc/alchemist/certs/qdrant-cert.pem"
            "QDRANT__TLS__KEY=/etc/alchemist/certs/qdrant-key.pem"
          ];
          
          StateDirectory = "qdrant";
          NoNewPrivileges = true;
          PrivateTmp = true;
        };
      };
      
      # Users and groups
      users.users.alchemist = {
        isSystemUser = true;
        group = "alchemist";
        home = "/var/lib/alchemist";
        createHome = true;
      };
      
      users.groups.alchemist = {};
      
      users.users.qdrant = {
        isSystemUser = true;
        group = "qdrant";
        home = "/var/lib/qdrant";
        createHome = true;
      };
      
      users.groups.qdrant = {};
      
      # Firewall configuration
      networking.firewall = {
        allowedTCPPorts = [ 8080 9090 8081 ];  # API, metrics, health
        allowedTCPPortRanges = [
          { from = 4222; to = 4222; }  # NATS client
          { from = 6333; to = 6334; }  # Qdrant HTTP/gRPC
        ];
      };
      
      # Monitoring with Prometheus node exporter
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
    };
  };
}
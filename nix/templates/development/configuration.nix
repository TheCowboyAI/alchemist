{ config, pkgs, ... }:

{
  # Import Alchemist modules
  imports = [
    ../../modules/alchemist-service.nix
    ../../modules/alchemist-agent.nix
    ../../modules/nats-mesh.nix
  ];

  # Basic system configuration
  system.stateVersion = "24.05";
  
  # Development environment settings
  environment.systemPackages = with pkgs; [
    vim
    git
    htop
    curl
    jq
    nats-server
    natscli
  ];

  # Enable Alchemist services
  services.alchemist = {
    enable = true;
    
    # Development API service
    services = {
      api = {
        enable = true;
        port = 8080;
        replicas = 1;
        resources = {
          cpu = "500m";
          memory = "512Mi";
        };
        environment = {
          RUST_LOG = "debug";
          ENVIRONMENT = "development";
        };
        healthCheck = {
          enable = true;
          http = "http://localhost:8080/health";
          interval = 30;
        };
      };
      
      scheduler = {
        enable = true;
        port = 8081;
        replicas = 1;
        resources = {
          cpu = "250m";
          memory = "256Mi";
        };
        environment = {
          RUST_LOG = "debug";
          ENVIRONMENT = "development";
        };
      };
    };
    
    # Development agent
    agents = {
      dev-worker = {
        enable = true;
        capabilities = [ "compute" "storage" ];
        resources = {
          cpu = "1";
          memory = "1Gi";
        };
        environment = {
          RUST_LOG = "debug";
        };
        maxTasks = 5;
      };
    };
  };

  # NATS mesh configuration for development
  services.nats-mesh = {
    enable = true;
    clusterName = "alchemist-dev";
    
    nodes = [{
      name = "nats-dev";
      host = "0.0.0.0";
      clientPort = 4222;
      clusterPort = 6222;
      monitorPort = 8222;
      leafPort = 7422;
      routes = [];
    }];
    
    jetstream = {
      storeDir = "/var/lib/nats/jetstream";
      maxMemory = "512M";
      maxFile = "1G";
    };
    
    # Development-friendly settings
    debug = true;
    trace = false;
    
    # Simple authorization for development
    authorization = {
      token = "development-token";
    };
  };

  # Networking
  networking = {
    hostName = "alchemist-dev";
    firewall = {
      enable = true;
      allowedTCPPorts = [ 
        4222  # NATS client
        6222  # NATS cluster
        7422  # NATS leaf
        8080  # API
        8081  # Scheduler
        8222  # NATS monitor
      ];
    };
  };

  # Enable SSH for development
  services.openssh = {
    enable = true;
    settings = {
      PasswordAuthentication = false;
      PermitRootLogin = "no";
    };
  };

  # Development user
  users.users.alchemist = {
    isNormalUser = true;
    description = "Alchemist Developer";
    extraGroups = [ "wheel" "docker" ];
    openssh.authorizedKeys.keys = [
      # Add your SSH public key here
    ];
  };

  # Enable Docker for development
  virtualisation.docker = {
    enable = true;
    enableOnBoot = true;
  };

  # System monitoring
  services.prometheus = {
    enable = true;
    port = 9090;
    
    scrapeConfigs = [
      {
        job_name = "alchemist";
        static_configs = [{
          targets = [
            "localhost:8080"  # API metrics
            "localhost:8081"  # Scheduler metrics
            "localhost:8222"  # NATS metrics
          ];
        }];
      }
    ];
  };

  # Log aggregation
  services.journald = {
    extraConfig = ''
      MaxRetentionSec=7d
      SystemMaxUse=1G
    '';
  };
}
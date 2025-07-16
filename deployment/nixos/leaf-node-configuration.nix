{ config, pkgs, lib, ... }:

{
  # Complete Leaf Node configuration for Alchemist deployment
  
  imports = [
    ./alchemist-container.nix
    ./extra-containers.nix
  ];
  
  # Leaf Node system configuration
  networking.hostName = "alchemist-leaf-node";
  networking.domain = "local";
  
  # Enable container support
  boot.enableContainers = true;
  
  # Network configuration for containers
  networking.bridges = {
    br-alchemist = {
      interfaces = [];
    };
  };
  
  networking.interfaces.br-alchemist = {
    ipv4.addresses = [
      { address = "10.233.0.1"; prefixLength = 16; }
    ];
  };
  
  # NAT for containers
  networking.nat = {
    enable = true;
    internalInterfaces = [ "br-alchemist" ];
    externalInterface = "eth0";  # Adjust to your external interface
  };
  
  # Firewall configuration
  networking.firewall = {
    enable = true;
    
    # Allow container traffic
    trustedInterfaces = [ "br-alchemist" ];
    
    # External access to services
    allowedTCPPorts = [
      80    # HTTP (nginx)
      443   # HTTPS (nginx)
      8080  # Alchemist API
      3000  # Grafana
    ];
    
    # Port forwarding to containers
    extraCommands = ''
      # Forward external ports to container services
      iptables -t nat -A PREROUTING -p tcp --dport 8080 -j DNAT --to-destination 10.233.1.2:8080
      iptables -t nat -A PREROUTING -p tcp --dport 3000 -j DNAT --to-destination 10.233.6.2:3000
    '';
  };
  
  # Nginx reverse proxy
  services.nginx = {
    enable = true;
    recommendedProxySettings = true;
    recommendedTlsSettings = true;
    recommendedOptimisation = true;
    recommendedGzipSettings = true;
    
    # Rate limiting
    appendHttpConfig = ''
      limit_req_zone $binary_remote_addr zone=api_limit:10m rate=10r/s;
      limit_req_zone $binary_remote_addr zone=auth_limit:10m rate=5r/m;
    '';
    
    virtualHosts = {
      "api.alchemist.local" = {
        enableACME = false;  # Use self-signed for local deployment
        forceSSL = true;
        sslCertificate = "/etc/alchemist/certs/nginx.crt";
        sslCertificateKey = "/etc/alchemist/certs/nginx.key";
        
        locations."/" = {
          proxyPass = "http://10.233.1.2:8080";
          extraConfig = ''
            limit_req zone=api_limit burst=20 nodelay;
            
            # CORS headers
            add_header 'Access-Control-Allow-Origin' '$http_origin' always;
            add_header 'Access-Control-Allow-Methods' 'GET, POST, PUT, DELETE, OPTIONS' always;
            add_header 'Access-Control-Allow-Headers' 'Authorization, Content-Type, X-API-Key' always;
            
            # Security headers
            add_header X-Frame-Options "SAMEORIGIN" always;
            add_header X-Content-Type-Options "nosniff" always;
            add_header X-XSS-Protection "1; mode=block" always;
            add_header Referrer-Policy "strict-origin-when-cross-origin" always;
          '';
        };
        
        locations."/auth" = {
          proxyPass = "http://10.233.1.2:8080/auth";
          extraConfig = ''
            limit_req zone=auth_limit burst=5 nodelay;
          '';
        };
      };
      
      "monitoring.alchemist.local" = {
        enableACME = false;
        forceSSL = true;
        sslCertificate = "/etc/alchemist/certs/nginx.crt";
        sslCertificateKey = "/etc/alchemist/certs/nginx.key";
        
        # Basic auth for monitoring
        basicAuth = {
          admin = "@MONITORING_PASSWORD@";  # Replaced by secrets
        };
        
        locations."/" = {
          proxyPass = "http://10.233.6.2:3000";
        };
        
        locations."/prometheus" = {
          proxyPass = "http://10.233.6.2:9091";
        };
      };
    };
  };
  
  # Certificate generation script
  system.activationScripts.alchemist-certs = ''
    mkdir -p /etc/alchemist/certs
    if [ ! -f /etc/alchemist/certs/ca.pem ]; then
      ${pkgs.openssl}/bin/openssl req -new -x509 -days 3650 -nodes \
        -out /etc/alchemist/certs/ca.pem \
        -keyout /etc/alchemist/certs/ca-key.pem \
        -subj "/C=US/ST=State/L=City/O=Alchemist/CN=Alchemist CA"
    fi
    
    # Generate certificates for each service
    for service in nginx nats qdrant alchemist; do
      if [ ! -f /etc/alchemist/certs/$service.crt ]; then
        ${pkgs.openssl}/bin/openssl req -new -nodes \
          -out /etc/alchemist/certs/$service.csr \
          -keyout /etc/alchemist/certs/$service.key \
          -subj "/C=US/ST=State/L=City/O=Alchemist/CN=$service.alchemist.local"
        
        ${pkgs.openssl}/bin/openssl x509 -req -days 365 \
          -in /etc/alchemist/certs/$service.csr \
          -CA /etc/alchemist/certs/ca.pem \
          -CAkey /etc/alchemist/certs/ca-key.pem \
          -CAcreateserial \
          -out /etc/alchemist/certs/$service.crt
          
        # Convert for services that need .pem
        cp /etc/alchemist/certs/$service.crt /etc/alchemist/certs/$service-cert.pem
        cp /etc/alchemist/certs/$service.key /etc/alchemist/certs/$service-key.pem
      fi
    done
    
    chmod -R 640 /etc/alchemist/certs/*
    chown -R root:alchemist /etc/alchemist/certs
  '';
  
  # Secrets management using agenix or sops-nix
  # This is a placeholder - in production, use proper secrets management
  environment.etc."alchemist/secrets.env" = {
    mode = "0640";
    user = "root";
    group = "alchemist";
    text = ''
      # These should be managed by agenix or sops-nix in production
      OPENAI_API_KEY=your_openai_key_here
      ANTHROPIC_API_KEY=your_anthropic_key_here
      ALCHEMIST_JWT_SECRET=your_jwt_secret_here
      ALCHEMIST_API_KEY=your_api_key_here
      POSTGRES_PASSWORD=your_postgres_password_here
      REDIS_PASSWORD=your_redis_password_here
      GRAFANA_PASSWORD=your_grafana_password_here
      GRAFANA_SECRET=your_grafana_secret_here
      MONITORING_PASSWORD=your_monitoring_password_here
    '';
  };
  
  # Backup configuration
  services.borgbackup.jobs.alchemist = {
    paths = [
      "/var/lib/alchemist"
      "/var/lib/postgresql"
      "/var/lib/redis"
      "/var/lib/qdrant"
      "/var/lib/nats"
      "/etc/alchemist"
    ];
    repo = "/backup/alchemist";  # Or remote repository
    encryption.mode = "repokey";
    encryption.passphrase = "@BACKUP_PASSPHRASE@";
    startAt = "daily";
    prune.keep = {
      daily = 7;
      weekly = 4;
      monthly = 6;
    };
  };
  
  # System monitoring and alerting
  services.netdata = {
    enable = true;
    config = {
      global = {
        "memory mode" = "dbengine";
        "page cache size" = 64;
        "dbengine disk space" = 1024;
      };
    };
  };
  
  # Log aggregation
  services.journalbeat = {
    enable = true;
    extraConfig = ''
      journalbeat.inputs:
      - paths: []
        include_matches:
          - _SYSTEMD_UNIT=alchemist.service
          - _SYSTEMD_UNIT=alchemist-domain-.+.service
          - CONTAINER_NAME=alchemist.*
      
      output.elasticsearch:
        hosts: ["localhost:9200"]
        index: "alchemist-%{+yyyy.MM.dd}"
    '';
  };
  
  # Resource limits for the host
  systemd.slices.alchemist = {
    description = "Slice for Alchemist containers";
    sliceConfig = {
      MemoryMax = "32G";
      CPUQuota = "800%";
    };
  };
  
  # Maintenance and updates
  system.autoUpgrade = {
    enable = true;
    allowReboot = false;  # Manual reboot for production
    dates = "Sun 03:00";
  };
  
  # Security hardening
  security.apparmor.enable = true;
  security.audit.enable = true;
  
  # Required system packages
  environment.systemPackages = with pkgs; [
    htop
    iotop
    nethogs
    tcpdump
    dig
    curl
    jq
    ripgrep
    git
  ];
}
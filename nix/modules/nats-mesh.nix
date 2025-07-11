{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.nats-mesh;
  
  # Generate NATS server configuration
  mkNatsConfig = node: pkgs.writeText "nats-${node.name}.conf" ''
    # NATS Server Configuration for ${node.name}
    server_name: ${node.name}
    
    # Client connections
    port: ${toString node.clientPort}
    host: ${node.host}
    
    # HTTP monitoring
    http_port: ${toString node.monitorPort}
    
    # Cluster configuration
    cluster {
      name: ${cfg.clusterName}
      port: ${toString node.clusterPort}
      
      # Routes to other cluster members
      routes: [
        ${concatMapStringsSep "\n    " (route: "nats-route://${route}") node.routes}
      ]
      
      # Cluster authorization
      ${optionalString (cfg.cluster.user != null) ''
      authorization {
        user: ${cfg.cluster.user}
        password: ${cfg.cluster.password}
      }
      ''}
    }
    
    # JetStream configuration
    jetstream {
      store_dir: "${cfg.jetstream.storeDir}/${node.name}"
      max_mem: ${cfg.jetstream.maxMemory}
      max_file: ${cfg.jetstream.maxFile}
      
      # Domain isolation
      ${optionalString (cfg.jetstream.domain != null) "domain: ${cfg.jetstream.domain}"}
    }
    
    # Leaf node configuration
    leafnodes {
      port: ${toString node.leafPort}
      
      # TLS for leaf connections
      ${optionalString cfg.leafnodes.tls.enable ''
      tls {
        cert_file: ${cfg.leafnodes.tls.certFile}
        key_file: ${cfg.leafnodes.tls.keyFile}
        ca_file: ${cfg.leafnodes.tls.caFile}
        verify: true
      }
      ''}
      
      # Authorization for leaf nodes
      ${optionalString (cfg.leafnodes.authorization != null) ''
      authorization {
        users: [
          ${concatMapStringsSep "\n      " (user: ''
          {
            user: "${user.name}"
            password: "${user.password}"
            account: "${user.account}"
          }
          '') cfg.leafnodes.authorization.users}
        ]
      }
      ''}
    }
    
    # Accounts configuration
    ${optionalString (cfg.accounts != {}) ''
    accounts {
      ${concatStringsSep "\n  " (mapAttrsToList (name: acc: ''
      ${name}: {
        jetstream: ${if acc.jetstream then "enabled" else "disabled"}
        ${optionalString (acc.limits != null) ''
        limits {
          ${optionalString (acc.limits.maxConnections != null) "max_connections: ${toString acc.limits.maxConnections}"}
          ${optionalString (acc.limits.maxSubscriptions != null) "max_subscriptions: ${toString acc.limits.maxSubscriptions}"}
          ${optionalString (acc.limits.maxPayload != null) "max_payload: ${acc.limits.maxPayload}"}
          ${optionalString (acc.limits.maxPending != null) "max_pending: ${acc.limits.maxPending}"}
        }
        ''}
      }
      '') cfg.accounts)}
    }
    ''}
    
    # System account for monitoring
    ${optionalString (cfg.systemAccount != null) ''
    system_account: ${cfg.systemAccount}
    ''}
    
    # Authorization
    ${optionalString (cfg.authorization != null) ''
    authorization {
      ${optionalString (cfg.authorization.users != []) ''
      users: [
        ${concatMapStringsSep "\n    " (user: ''
        {
          user: "${user.name}"
          password: "${user.password}"
          ${optionalString (user.permissions != null) ''
          permissions {
            ${optionalString (user.permissions.publish != null) ''
            publish: [${concatMapStringsSep ", " (p: "\"${p}\"") user.permissions.publish}]
            ''}
            ${optionalString (user.permissions.subscribe != null) ''
            subscribe: [${concatMapStringsSep ", " (s: "\"${s}\"") user.permissions.subscribe}]
            ''}
          }
          ''}
        }
        '') cfg.authorization.users}
      ]
      ''}
      
      ${optionalString (cfg.authorization.token != null) ''
      token: "${cfg.authorization.token}"
      ''}
    }
    ''}
    
    # Logging
    debug: ${if cfg.debug then "true" else "false"}
    trace: ${if cfg.trace then "true" else "false"}
    logtime: true
    
    # Limits
    max_connections: ${toString cfg.limits.maxConnections}
    max_payload: ${cfg.limits.maxPayload}
    max_pending: ${cfg.limits.maxPending}
    
    # TLS Configuration
    ${optionalString cfg.tls.enable ''
    tls {
      cert_file: ${cfg.tls.certFile}
      key_file: ${cfg.tls.keyFile}
      ca_file: ${cfg.tls.caFile}
      verify: true
    }
    ''}
  '';
  
  # Generate leaf node configuration
  mkLeafConfig = leaf: pkgs.writeText "nats-leaf-${leaf.name}.conf" ''
    # NATS Leaf Node Configuration for ${leaf.name}
    server_name: leaf_${leaf.name}
    
    # Client connections
    port: ${toString leaf.port}
    
    # Leaf node configuration
    leafnodes {
      remotes: [
        ${concatMapStringsSep "\n    " (remote: ''
        {
          url: "${remote.url}"
          ${optionalString (remote.credentials != null) ''credentials: "${remote.credentials}"''}
          ${optionalString (remote.account != null) ''account: "${remote.account}"''}
        }
        '') leaf.remotes}
      ]
    }
    
    # JetStream configuration for leaf
    jetstream {
      store_dir: "${cfg.jetstream.storeDir}/leaf_${leaf.name}"
      max_mem: ${leaf.jetstream.maxMemory}
      max_file: ${leaf.jetstream.maxFile}
      ${optionalString (leaf.jetstream.domain != null) "domain: ${leaf.jetstream.domain}"}
    }
    
    # Logging
    debug: ${if cfg.debug then "true" else "false"}
    trace: ${if cfg.trace then "true" else "false"}
    logtime: true
  '';

in {
  options.services.nats-mesh = {
    enable = mkEnableOption "NATS mesh network";
    
    clusterName = mkOption {
      type = types.str;
      default = "alchemist";
      description = "Name of the NATS cluster";
    };
    
    nodes = mkOption {
      type = types.listOf (types.submodule {
        options = {
          name = mkOption {
            type = types.str;
            description = "Node name";
          };
          
          host = mkOption {
            type = types.str;
            default = "0.0.0.0";
            description = "Host to bind to";
          };
          
          clientPort = mkOption {
            type = types.int;
            default = 4222;
            description = "Port for client connections";
          };
          
          clusterPort = mkOption {
            type = types.int;
            default = 6222;
            description = "Port for cluster connections";
          };
          
          monitorPort = mkOption {
            type = types.int;
            default = 8222;
            description = "Port for HTTP monitoring";
          };
          
          leafPort = mkOption {
            type = types.int;
            default = 7422;
            description = "Port for leaf node connections";
          };
          
          routes = mkOption {
            type = types.listOf types.str;
            default = [];
            description = "Routes to other cluster nodes (host:port)";
          };
        };
      });
      default = [];
      description = "NATS cluster nodes";
    };
    
    leafNodes = mkOption {
      type = types.listOf (types.submodule {
        options = {
          name = mkOption {
            type = types.str;
            description = "Leaf node name";
          };
          
          port = mkOption {
            type = types.int;
            default = 4222;
            description = "Port for client connections";
          };
          
          remotes = mkOption {
            type = types.listOf (types.submodule {
              options = {
                url = mkOption {
                  type = types.str;
                  description = "Remote server URL";
                };
                
                credentials = mkOption {
                  type = types.nullOr types.path;
                  default = null;
                  description = "Path to credentials file";
                };
                
                account = mkOption {
                  type = types.nullOr types.str;
                  default = null;
                  description = "Account to use for this connection";
                };
              };
            });
            description = "Remote servers to connect to";
          };
          
          jetstream = {
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
              type = types.nullOr types.str;
              default = null;
              description = "JetStream domain for isolation";
            };
          };
        };
      });
      default = [];
      description = "NATS leaf nodes";
    };
    
    jetstream = {
      storeDir = mkOption {
        type = types.path;
        default = "/var/lib/nats/jetstream";
        description = "JetStream storage directory";
      };
      
      maxMemory = mkOption {
        type = types.str;
        default = "1G";
        description = "Maximum memory for JetStream";
      };
      
      maxFile = mkOption {
        type = types.str;
        default = "10G";
        description = "Maximum file storage for JetStream";
      };
      
      domain = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "JetStream domain for multi-tenancy";
      };
    };
    
    cluster = {
      user = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Cluster authorization user";
      };
      
      password = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Cluster authorization password";
      };
    };
    
    leafnodes = {
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
      
      authorization = mkOption {
        type = types.nullOr (types.submodule {
          options = {
            users = mkOption {
              type = types.listOf (types.submodule {
                options = {
                  name = mkOption {
                    type = types.str;
                    description = "Username";
                  };
                  password = mkOption {
                    type = types.str;
                    description = "Password";
                  };
                  account = mkOption {
                    type = types.str;
                    description = "Account name";
                  };
                };
              });
              default = [];
              description = "Authorized leaf node users";
            };
          };
        });
        default = null;
        description = "Leaf node authorization";
      };
    };
    
    accounts = mkOption {
      type = types.attrsOf (types.submodule {
        options = {
          jetstream = mkOption {
            type = types.bool;
            default = true;
            description = "Enable JetStream for this account";
          };
          
          limits = mkOption {
            type = types.nullOr (types.submodule {
              options = {
                maxConnections = mkOption {
                  type = types.nullOr types.int;
                  default = null;
                  description = "Maximum connections";
                };
                
                maxSubscriptions = mkOption {
                  type = types.nullOr types.int;
                  default = null;
                  description = "Maximum subscriptions";
                };
                
                maxPayload = mkOption {
                  type = types.nullOr types.str;
                  default = null;
                  description = "Maximum payload size";
                };
                
                maxPending = mkOption {
                  type = types.nullOr types.str;
                  default = null;
                  description = "Maximum pending size";
                };
              };
            });
            default = null;
            description = "Account limits";
          };
        };
      });
      default = {};
      description = "NATS accounts configuration";
    };
    
    systemAccount = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "System account for monitoring and management";
    };
    
    authorization = mkOption {
      type = types.nullOr (types.submodule {
        options = {
          users = mkOption {
            type = types.listOf (types.submodule {
              options = {
                name = mkOption {
                  type = types.str;
                  description = "Username";
                };
                
                password = mkOption {
                  type = types.str;
                  description = "Password";
                };
                
                permissions = mkOption {
                  type = types.nullOr (types.submodule {
                    options = {
                      publish = mkOption {
                        type = types.nullOr (types.listOf types.str);
                        default = null;
                        description = "Publish permissions";
                      };
                      
                      subscribe = mkOption {
                        type = types.nullOr (types.listOf types.str);
                        default = null;
                        description = "Subscribe permissions";
                      };
                    };
                  });
                  default = null;
                  description = "User permissions";
                };
              };
            });
            default = [];
            description = "Authorized users";
          };
          
          token = mkOption {
            type = types.nullOr types.str;
            default = null;
            description = "Authorization token";
          };
        };
      });
      default = null;
      description = "NATS authorization configuration";
    };
    
    limits = {
      maxConnections = mkOption {
        type = types.int;
        default = 65536;
        description = "Maximum number of connections";
      };
      
      maxPayload = mkOption {
        type = types.str;
        default = "1MB";
        description = "Maximum message payload size";
      };
      
      maxPending = mkOption {
        type = types.str;
        default = "64MB";
        description = "Maximum pending size";
      };
    };
    
    tls = {
      enable = mkOption {
        type = types.bool;
        default = false;
        description = "Enable TLS";
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
    # Create systemd services for each NATS node
    systemd.services = mkMerge [
      # Cluster nodes
      (listToAttrs (map (node: {
        name = "nats-${node.name}";
        value = {
          description = "NATS server node: ${node.name}";
          wantedBy = [ "multi-user.target" ];
          after = [ "network.target" ];
          
          serviceConfig = {
            Type = "simple";
            ExecStart = "${pkgs.nats-server}/bin/nats-server -c ${mkNatsConfig node}";
            ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
            Restart = "always";
            RestartSec = 5;
            
            # Security
            DynamicUser = true;
            StateDirectory = "nats/${node.name}";
            RuntimeDirectory = "nats/${node.name}";
            LogsDirectory = "nats/${node.name}";
            
            # Hardening
            PrivateTmp = true;
            ProtectSystem = "strict";
            ProtectHome = true;
            NoNewPrivileges = true;
            ProtectKernelTunables = true;
            ProtectKernelModules = true;
            ProtectControlGroups = true;
            RestrictAddressFamilies = [ "AF_INET" "AF_INET6" "AF_UNIX" ];
            LockPersonality = true;
            RestrictRealtime = true;
            SystemCallFilter = [ "@system-service" "~@privileged" ];
          };
        };
      }) cfg.nodes))
      
      # Leaf nodes
      (listToAttrs (map (leaf: {
        name = "nats-leaf-${leaf.name}";
        value = {
          description = "NATS leaf node: ${leaf.name}";
          wantedBy = [ "multi-user.target" ];
          after = [ "network.target" ];
          
          serviceConfig = {
            Type = "simple";
            ExecStart = "${pkgs.nats-server}/bin/nats-server -c ${mkLeafConfig leaf}";
            ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
            Restart = "always";
            RestartSec = 5;
            
            # Security
            DynamicUser = true;
            StateDirectory = "nats/leaf_${leaf.name}";
            RuntimeDirectory = "nats/leaf_${leaf.name}";
            LogsDirectory = "nats/leaf_${leaf.name}";
            
            # Hardening
            PrivateTmp = true;
            ProtectSystem = "strict";
            ProtectHome = true;
            NoNewPrivileges = true;
            ProtectKernelTunables = true;
            ProtectKernelModules = true;
            ProtectControlGroups = true;
            RestrictAddressFamilies = [ "AF_INET" "AF_INET6" "AF_UNIX" ];
            LockPersonality = true;
            RestrictRealtime = true;
            SystemCallFilter = [ "@system-service" "~@privileged" ];
          };
        };
      }) cfg.leafNodes))
    ];
    
    # Create JetStream directories
    systemd.tmpfiles.rules = 
      (map (node: 
        "d ${cfg.jetstream.storeDir}/${node.name} 0750 nats nats -"
      ) cfg.nodes) ++
      (map (leaf: 
        "d ${cfg.jetstream.storeDir}/leaf_${leaf.name} 0750 nats nats -"
      ) cfg.leafNodes);
    
    # Open firewall ports
    networking.firewall.allowedTCPPorts = mkMerge [
      (map (node: [ node.clientPort node.clusterPort node.monitorPort node.leafPort ]) cfg.nodes)
      (map (leaf: leaf.port) cfg.leafNodes)
    ];
    
    # Install NATS tools
    environment.systemPackages = with pkgs; [
      nats-server
      natscli
      nats-top
    ];
  };
}
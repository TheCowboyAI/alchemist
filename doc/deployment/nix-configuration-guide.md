# CIM Nix Configuration Guide

## Overview

This guide covers Nix configuration patterns specific to CIM deployments, including how Domain Objects are translated to Nix configurations and how existing Nix files are imported and composed.

## Domain Object to Nix Translation

### Graph Domain Configuration

```typescript
// Domain Object (TypeScript)
const graphDomain = new GraphDomain({
  maxNodes: 100000,
  persistence: PersistenceType.RocksDB,
  indexing: IndexingStrategy.RTree,
  cache: {
    size: "16GB",
    ttl: Duration.hours(24)
  }
});
```

Translates to:

```nix
# Nix Configuration
cim.domains.graph = {
  enable = true;
  
  config = {
    maxNodes = 100000;
    persistence = "rocksdb";
    indexing = "rtree";
    
    cache = {
      size = "16GB";
      ttl = 86400; # seconds
    };
  };
  
  # System service configuration
  systemd.services.cim-graph = {
    description = "CIM Graph Domain Service";
    wantedBy = [ "multi-user.target" ];
    after = [ "nats.service" ];
    
    serviceConfig = {
      Type = "notify";
      ExecStart = "${pkgs.cim-graph}/bin/cim-graph-server";
      Restart = "on-failure";
      RestartSec = 5;
      
      # Resource limits
      LimitNOFILE = 65536;
      MemoryLimit = "32G";
    };
    
    environment = {
      RUST_LOG = "info";
      CIM_GRAPH_CONFIG = "/etc/cim/graph.toml";
    };
  };
};
```

### Agent Domain Configuration

```typescript
// Domain Object
const agentDomain = new AgentDomain({
  providers: [
    new OpenAIProvider({ 
      apiKey: "sk-...",
      model: "gpt-4",
      maxTokens: 4000
    }),
    new OllamaProvider({
      endpoint: "http://localhost:11434",
      models: ["llama2", "codellama"]
    })
  ],
  maxAgents: 100,
  capabilities: ["reasoning", "code-generation", "analysis"]
});
```

Translates to:

```nix
# Nix Configuration
cim.domains.agent = {
  enable = true;
  
  config = {
    maxAgents = 100;
    capabilities = [ "reasoning" "code-generation" "analysis" ];
    
    providers = {
      openai = {
        enable = true;
        model = "gpt-4";
        maxTokens = 4000;
        # API key from secrets
        apiKeyFile = config.sops.secrets."openai-api-key".path;
      };
      
      ollama = {
        enable = true;
        endpoint = "http://localhost:11434";
        models = [ "llama2" "codellama" ];
      };
    };
  };
  
  # GPU configuration for local models
  hardware.nvidia = {
    enable = true;
    powerManagement.enable = true;
  };
};
```

## Composing Configurations

### Base System Module

```nix
# modules/cim-base.nix
{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.cim;
in
{
  options.cim = {
    enable = mkEnableOption "Composable Information Machine";
    
    instance = {
      id = mkOption {
        type = types.str;
        description = "Unique identifier for this CIM instance";
      };
      
      name = mkOption {
        type = types.str;
        description = "Human-readable name for this instance";
      };
      
      location = mkOption {
        type = types.str;
        default = "default";
        description = "Physical or logical location";
      };
      
      capabilities = mkOption {
        type = types.listOf types.str;
        default = [];
        description = "List of capabilities this instance provides";
      };
    };
  };
  
  config = mkIf cfg.enable {
    # Base system configuration
    environment.systemPackages = with pkgs; [
      cim-cli
      nats-cli
      jq
      ripgrep
    ];
    
    # Base monitoring
    services.prometheus.exporters.node = {
      enable = true;
      enabledCollectors = [ "systemd" "processes" ];
    };
    
    # System identification
    environment.etc."cim/instance.json".text = builtins.toJSON {
      inherit (cfg.instance) id name location capabilities;
      nixos_version = config.system.nixos.release;
    };
  };
}
```

### Domain Module Template

```nix
# modules/domains/template.nix
{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.cim.domains.myDomain;
  
  configFile = pkgs.writeText "mydomain-config.toml" ''
    [domain]
    id = "${cfg.id}"
    ${optionalString (cfg.config ? maxItems) ''
      max_items = ${toString cfg.config.maxItems}
    ''}
    
    [nats]
    url = "${config.services.nats.serverUrl}"
    subject_prefix = "domain.mydomain"
  '';
in
{
  options.cim.domains.myDomain = {
    enable = mkEnableOption "My Domain";
    
    id = mkOption {
      type = types.str;
      default = "mydomain-${config.cim.instance.id}";
      description = "Domain instance ID";
    };
    
    config = mkOption {
      type = types.attrs;
      default = {};
      description = "Domain-specific configuration";
    };
  };
  
  config = mkIf cfg.enable {
    systemd.services.cim-mydomain = {
      description = "CIM My Domain Service";
      wantedBy = [ "multi-user.target" ];
      after = [ "nats.service" ];
      
      serviceConfig = {
        ExecStart = "${pkgs.cim-mydomain}/bin/server -c ${configFile}";
        Restart = "on-failure";
        User = "cim-mydomain";
        Group = "cim";
      };
    };
    
    users.users.cim-mydomain = {
      isSystemUser = true;
      group = "cim";
    };
  };
}
```

## Importing Existing Configurations

### Strategy 1: Direct Import

```nix
# flake.nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    
    # Import existing configurations
    hardware-config.url = "path:./hardware";
    network-config.url = "path:./network";
    security-config.url = "path:./security";
  };
  
  outputs = { self, nixpkgs, hardware-config, network-config, security-config }: {
    nixosConfigurations.cim-prod-001 = nixpkgs.lib.nixosSystem {
      modules = [
        # Import existing configs
        hardware-config.nixosModules.default
        network-config.nixosModules.default
        security-config.nixosModules.default
        
        # Add CIM configuration
        ./cim-configuration.nix
      ];
    };
  };
}
```

### Strategy 2: Overlay Pattern

```nix
# overlays/cim-domains.nix
self: super: {
  # Override or extend existing packages
  cim-graph = super.cim-graph.override {
    enableGPU = true;
    rustcFlags = "-C target-cpu=native";
  };
  
  # Add new domain packages
  cim-custom-domain = self.callPackage ../pkgs/custom-domain { };
}
```

### Strategy 3: Configuration Merging

```nix
# merge-configs.nix
{ config, lib, ... }:

let
  # Load existing configurations
  existingNetwork = import ./existing/network.nix { inherit config lib; };
  existingSecurity = import ./existing/security.nix { inherit config lib; };
  
  # Domain configurations from Alchemist
  domainConfig = lib.importJSON ./generated/domains.json;
  
  # Merge strategies
  mergeNetwork = lib.recursiveUpdate existingNetwork {
    networking.firewall.allowedTCPPorts = 
      existingNetwork.networking.firewall.allowedTCPPorts ++ [ 4222 8222 ];
  };
in
{
  imports = [
    # Apply merged configurations
    mergeNetwork
    existingSecurity
  ];
  
  # Apply domain configurations
  cim.domains = lib.mapAttrs (name: cfg: {
    enable = true;
    config = cfg;
  }) domainConfig;
}
```

## Advanced Patterns

### Conditional Domain Loading

```nix
# conditional-domains.nix
{ config, lib, pkgs, ... }:

with lib;

{
  # Load domains based on instance capabilities
  cim.domains = mkMerge [
    (mkIf (elem "graph" config.cim.instance.capabilities) {
      graph = {
        enable = true;
        config = {
          persistence = if config.cim.instance.production
            then "rocksdb"
            else "memory";
        };
      };
    })
    
    (mkIf (elem "agent" config.cim.instance.capabilities) {
      agent = {
        enable = true;
        config = {
          providers = if config.hardware.nvidia.enable
            then [ "ollama" "openai" ]
            else [ "openai" ];
        };
      };
    })
  ];
}
```

### Resource-Based Configuration

```nix
# resource-aware.nix
{ config, lib, pkgs, ... }:

let
  # Detect available resources
  cpuCount = config.nix.settings.cores;
  totalMemory = config.hardware.memorySize or 16384; # MB
  hasGPU = config.hardware.nvidia.enable or false;
  
  # Calculate resource allocations
  graphMemory = if totalMemory > 65536 then "32G" else "8G";
  agentWorkers = if cpuCount > 16 then 8 else 4;
in
{
  cim.domains = {
    graph.config = {
      cache.size = graphMemory;
      workers = cpuCount / 2;
    };
    
    agent.config = {
      maxConcurrent = agentWorkers;
      enableGPU = hasGPU;
    };
  };
}
```

### Environment-Specific Overrides

```nix
# environments/production.nix
{ config, lib, ... }:

{
  cim.instance.production = true;
  
  # Production-specific settings
  cim.domains = lib.mapAttrs (name: domain: 
    domain // {
      config = domain.config // {
        logging = "warn";
        metrics = true;
        tracing = true;
      };
    }
  ) config.cim.domains;
  
  # Production security
  security.sudo.wheelNeedsPassword = true;
  services.fail2ban.enable = true;
  
  # Production monitoring
  services.prometheus.enable = true;
  services.grafana.enable = true;
}
```

## Secrets Management

### Using sops-nix

```nix
# secrets.nix
{ config, pkgs, ... }:

{
  imports = [ <sops-nix/modules/sops> ];
  
  sops = {
    defaultSopsFile = ./secrets/cim.yaml;
    age.sshKeyPaths = [ "/etc/ssh/ssh_host_ed25519_key" ];
    
    secrets = {
      "openai-api-key" = {
        owner = "cim-agent";
      };
      "nats-credentials" = {
        owner = "cim";
        path = "/etc/nats/leaf.creds";
      };
      "tls-cert" = {
        owner = "nginx";
        path = "/etc/ssl/cim.crt";
      };
      "tls-key" = {
        owner = "nginx";
        path = "/etc/ssl/cim.key";
      };
    };
  };
  
  # Use secrets in domain configuration
  cim.domains.agent.config.providers.openai.apiKeyFile = 
    config.sops.secrets."openai-api-key".path;
}
```

### Using Vault

```nix
# vault-secrets.nix
{ config, pkgs, ... }:

let
  vaultAddr = "https://vault.example.com:8200";
  
  getSecret = path: default: 
    let
      result = builtins.exec [
        "${pkgs.vault}/bin/vault"
        "kv" "get" "-field=value"
        path
      ];
    in
      if result.success then result.stdout else default;
in
{
  cim.domains.agent.config = {
    providers.openai.apiKey = getSecret "secret/cim/openai-key" "";
    providers.anthropic.apiKey = getSecret "secret/cim/anthropic-key" "";
  };
}
```

## Validation and Testing

### Configuration Validation

```nix
# validation.nix
{ config, lib, pkgs, ... }:

with lib;

{
  assertions = [
    {
      assertion = config.cim.enable -> config.services.nats.enable;
      message = "CIM requires NATS to be enabled";
    }
    {
      assertion = config.cim.domains.agent.enable -> 
        (config.cim.domains.agent.config.providers != {});
      message = "Agent domain requires at least one provider";
    }
    {
      assertion = config.cim.instance.id != "";
      message = "CIM instance ID must be set";
    }
  ];
  
  # Runtime validation script
  environment.systemPackages = [
    (pkgs.writeScriptBin "cim-validate" ''
      #!${pkgs.bash}/bin/bash
      
      echo "Validating CIM configuration..."
      
      # Check NATS connectivity
      ${pkgs.natscli}/bin/nats --server=$NATS_URL rtt || exit 1
      
      # Check domain services
      for domain in ${toString (attrNames config.cim.domains)}; do
        systemctl is-active cim-$domain || exit 1
      done
      
      echo "CIM configuration valid!"
    '')
  ];
}
```

### Integration Tests

```nix
# tests/cim-integration.nix
{ pkgs, ... }:

{
  name = "cim-integration-test";
  
  nodes = {
    hub = { ... }: {
      services.nats = {
        enable = true;
        jetstream = true;
      };
    };
    
    leaf = { ... }: {
      imports = [ ../cim-configuration.nix ];
      
      cim = {
        enable = true;
        instance.id = "test-001";
        domains.graph.enable = true;
      };
    };
  };
  
  testScript = ''
    start_all()
    
    hub.wait_for_unit("nats.service")
    leaf.wait_for_unit("cim-graph.service")
    
    # Test NATS connectivity
    leaf.succeed("nats --server=hub:4222 pub test 'hello'")
    
    # Test domain functionality
    leaf.succeed("cim-cli graph create test-graph")
    leaf.succeed("cim-cli graph list | grep test-graph")
  '';
}
```

## Best Practices

1. **Modularity**: Keep domain configurations in separate modules
2. **Type Safety**: Use NixOS options system for validation
3. **Secrets**: Never commit secrets, use sops-nix or Vault
4. **Testing**: Always test configurations in VM before deployment
5. **Documentation**: Document all custom options and patterns
6. **Version Control**: Track all configuration changes in Git

---

This guide provides patterns for translating CIM Domain Objects to Nix configurations and composing them with existing infrastructure code. 
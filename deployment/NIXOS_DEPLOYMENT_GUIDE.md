# NixOS Leaf Node Deployment Guide for Alchemist

This guide provides comprehensive instructions for deploying Alchemist on NixOS-based Leaf Nodes using nixos-containers.

## Table of Contents
1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Leaf Node Setup](#leaf-node-setup)
4. [Container Configuration](#container-configuration)
5. [Security Configuration](#security-configuration)
6. [Monitoring Setup](#monitoring-setup)
7. [Backup and Recovery](#backup-and-recovery)
8. [Performance Tuning](#performance-tuning)
9. [Troubleshooting](#troubleshooting)

## Overview

Alchemist is deployed on NixOS Leaf Nodes using declarative nixos-containers. This provides:
- Reproducible deployments
- Atomic updates and rollbacks
- Resource isolation
- Declarative configuration
- No Docker or Kubernetes dependencies

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Leaf Node (NixOS)                    │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────┐  │
│  │  Alchemist  │  │    Graph     │  │   Workflow    │  │
│  │  Container  │  │  Container   │  │   Container   │  │
│  │             │  │              │  │               │  │
│  │ - NATS      │  │ - Domain Svc │  │ - Domain Svc  │  │
│  │ - PostgreSQL│  │ - Metrics    │  │ - Metrics     │  │
│  │ - Redis     │  └──────────────┘  └───────────────┘  │
│  │ - Qdrant    │                                        │
│  └─────────────┘  ┌──────────────┐  ┌───────────────┐  │
│                   │    Agent     │  │   Monitoring  │  │
│                   │  Container   │  │   Container   │  │
│                   │              │  │               │  │
│                   │ - Domain Svc │  │ - Prometheus  │  │
│                   │ - AI Models  │  │ - Grafana     │  │
│                   └──────────────┘  └───────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## Prerequisites

### System Requirements
- **OS**: NixOS 23.11 or later
- **CPU**: Minimum 8 cores, recommended 16+ cores
- **Memory**: Minimum 16GB RAM, recommended 32GB+ RAM
- **Storage**: 200GB+ SSD storage
- **Network**: Stable internet connection

### Required Nix Channels
```bash
# Add required channels
sudo nix-channel --add https://nixos.org/channels/nixos-unstable nixos
sudo nix-channel --update
```

### API Keys Required
- OpenAI API key (for GPT-4 integration)
- Anthropic API key (for Claude integration)
- JWT secret for authentication
- API key for service authentication

## Leaf Node Setup

### 1. Prepare the Leaf Node

```bash
# Clone the Alchemist repository
git clone https://github.com/thecowboyai/alchemist.git /etc/nixos/alchemist

# Create directory structure
sudo mkdir -p /etc/alchemist/{certs,secrets}
sudo mkdir -p /var/lib/alchemist
sudo mkdir -p /backup/alchemist
```

### 2. Configure Secrets

Using agenix (recommended):
```nix
# /etc/nixos/secrets.nix
{
  age.secrets = {
    openai-api-key = {
      file = ./secrets/openai-api-key.age;
      mode = "640";
      group = "alchemist";
    };
    anthropic-api-key = {
      file = ./secrets/anthropic-api-key.age;
      mode = "640";
      group = "alchemist";
    };
    jwt-secret = {
      file = ./secrets/jwt-secret.age;
      mode = "640";
      group = "alchemist";
    };
    postgres-password = {
      file = ./secrets/postgres-password.age;
      mode = "640";
      group = "alchemist";
    };
  };
}
```

Or using environment file (development only):
```bash
# Create secrets file
sudo cat > /etc/alchemist/secrets.env << 'EOF'
OPENAI_API_KEY=your_openai_key_here
ANTHROPIC_API_KEY=your_anthropic_key_here
ALCHEMIST_JWT_SECRET=your_jwt_secret_here
ALCHEMIST_API_KEY=your_api_key_here
POSTGRES_PASSWORD=your_postgres_password_here
REDIS_PASSWORD=your_redis_password_here
EOF

sudo chmod 640 /etc/alchemist/secrets.env
sudo chown root:alchemist /etc/alchemist/secrets.env
```

### 3. Update NixOS Configuration

Add to `/etc/nixos/configuration.nix`:
```nix
{ config, pkgs, ... }:

{
  imports = [
    ./hardware-configuration.nix
    ./alchemist/deployment/nixos/leaf-node-configuration.nix
  ];
  
  # Your existing configuration...
}
```

### 4. Build and Deploy

```bash
# Test configuration
sudo nixos-rebuild test

# Deploy
sudo nixos-rebuild switch

# Verify containers are running
sudo nixos-container list
sudo systemctl status container@alchemist
```

## Container Configuration

### Main Alchemist Container

The main container includes:
- NATS JetStream for event streaming
- PostgreSQL for persistent storage
- Redis for caching
- Qdrant for vector search

Configuration: `/etc/nixos/alchemist/deployment/nixos/alchemist-container.nix`

### Domain Containers

Each domain runs in its own container:
- **Graph**: Core graph operations
- **Workflow**: Business process execution
- **Agent**: AI provider integration

Configuration: `/etc/nixos/alchemist/deployment/nixos/extra-containers.nix`

### Container Management

```bash
# Start/stop containers
sudo nixos-container start alchemist
sudo nixos-container stop alchemist

# Access container shell
sudo nixos-container root-login alchemist

# View container status
sudo nixos-container show-info alchemist

# Update container configuration
sudo nixos-container update alchemist
```

## Security Configuration

### 1. Network Security

```nix
# Firewall configuration (already in leaf-node-configuration.nix)
networking.firewall = {
  enable = true;
  allowedTCPPorts = [ 80 443 8080 3000 ];
  trustedInterfaces = [ "br-alchemist" ];
};
```

### 2. TLS Certificates

Production certificates with Let's Encrypt:
```nix
security.acme = {
  acceptTerms = true;
  defaults.email = "admin@alchemist.local";
};

services.nginx.virtualHosts."api.alchemist.com" = {
  enableACME = true;
  forceSSL = true;
};
```

### 3. Container Isolation

Each container has:
- Private network namespace
- Resource limits
- Security hardening
- Minimal privileges

### 4. Audit Logging

```bash
# View audit logs
sudo journalctl -u audit

# Container-specific logs
sudo journalctl -M alchemist -u alchemist
```

## Monitoring Setup

### 1. Access Monitoring

```bash
# Grafana dashboard
https://monitoring.alchemist.local
# Default: admin / <configured password>

# Prometheus metrics
https://monitoring.alchemist.local/prometheus
```

### 2. Key Metrics

- Request rate and latency
- Domain event flow
- Resource usage
- Error rates
- AI provider usage

### 3. Alerting

Alerts are configured in Prometheus for:
- High error rates
- Domain failures
- Resource exhaustion
- Certificate expiration

### 4. Log Aggregation

```bash
# View aggregated logs
sudo journalctl -f -u alchemist-domain-*

# Export logs
sudo journalctl -u alchemist --since "1 hour ago" > alchemist-logs.txt
```

## Backup and Recovery

### 1. Automated Backups

Configured via BorgBackup:
```bash
# Manual backup
sudo systemctl start borgbackup-alchemist

# View backup status
sudo borg list /backup/alchemist

# List archives
sudo borg list /backup/alchemist
```

### 2. Restore Procedure

```bash
# Stop services
sudo nixos-container stop alchemist

# Restore from backup
sudo borg extract /backup/alchemist::archive-name

# Restart services
sudo nixos-container start alchemist
```

### 3. Disaster Recovery

1. Boot from NixOS installation media
2. Mount system partitions
3. Restore NixOS configuration
4. Run `nixos-install`
5. Restore data from backups

## Performance Tuning

### 1. Container Resources

Adjust in container configuration:
```nix
systemd.services.alchemist = {
  serviceConfig = {
    MemoryHigh = "6G";
    MemoryMax = "8G";
    CPUQuota = "400%";
  };
};
```

### 2. PostgreSQL Tuning

Edit in `alchemist-container.nix`:
```nix
services.postgresql.settings = {
  shared_buffers = "4GB";
  effective_cache_size = "12GB";
  work_mem = "20MB";
};
```

### 3. System Tuning

```bash
# Increase file descriptors
echo "* soft nofile 65536" >> /etc/security/limits.conf
echo "* hard nofile 65536" >> /etc/security/limits.conf

# Kernel parameters
boot.kernel.sysctl = {
  "net.core.somaxconn" = 65535;
  "net.ipv4.tcp_max_syn_backlog" = 65535;
  "vm.swappiness" = 10;
};
```

## Troubleshooting

### Common Issues

1. **Container Won't Start**
   ```bash
   # Check container logs
   sudo journalctl -M alchemist -xe
   
   # Verify configuration
   sudo nixos-container show-config alchemist
   ```

2. **Network Issues**
   ```bash
   # Check bridge
   ip addr show br-alchemist
   
   # Test connectivity
   sudo nixos-container run alchemist -- ping 10.233.1.1
   ```

3. **Resource Issues**
   ```bash
   # Check resource usage
   systemd-cgtop
   
   # Container-specific resources
   sudo systemctl status container@alchemist
   ```

### Debug Commands

```bash
# Container shell access
sudo nixos-container root-login alchemist

# Run commands in container
sudo nixos-container run alchemist -- <command>

# View container journal
sudo journalctl -M alchemist -f

# Check systemd services
sudo nixos-container run alchemist -- systemctl status

# Network debugging
sudo nixos-container run alchemist -- ss -tlnp
```

### Recovery Procedures

1. **Rollback Configuration**
   ```bash
   # List generations
   sudo nix-env --list-generations -p /nix/var/nix/profiles/system
   
   # Rollback
   sudo nixos-rebuild switch --rollback
   ```

2. **Rebuild Container**
   ```bash
   # Destroy and recreate
   sudo nixos-container destroy alchemist
   sudo nixos-rebuild switch
   ```

## Maintenance

### Updates

```bash
# Update channels
sudo nix-channel --update

# Update system
sudo nixos-rebuild switch --upgrade

# Garbage collection
sudo nix-collect-garbage -d
```

### Health Checks

```bash
# API health
curl -k https://api.alchemist.local/health

# Container health
for container in alchemist alchemist-graph alchemist-workflow alchemist-agent; do
  echo "Checking $container..."
  sudo nixos-container run $container -- systemctl is-system-running
done
```

## Production Checklist

- [ ] All secrets properly configured with agenix/sops-nix
- [ ] TLS certificates valid and auto-renewing
- [ ] Monitoring and alerting active
- [ ] Backup procedures tested
- [ ] Resource limits configured
- [ ] Security hardening applied
- [ ] Network policies configured
- [ ] Audit logging enabled
- [ ] Documentation up to date
- [ ] Runbooks prepared
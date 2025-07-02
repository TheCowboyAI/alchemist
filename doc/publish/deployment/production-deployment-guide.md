# CIM Production Deployment Guide

This guide provides comprehensive instructions for deploying the Composable Information Machine (CIM) in production environments.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Infrastructure Requirements](#infrastructure-requirements)
3. [NATS Configuration](#nats-configuration)
4. [Application Deployment](#application-deployment)
5. [Security Configuration](#security-configuration)
6. [Monitoring Setup](#monitoring-setup)
7. [Performance Tuning](#performance-tuning)
8. [Backup and Recovery](#backup-and-recovery)
9. [Troubleshooting](#troubleshooting)

## Prerequisites

### System Requirements

- **Operating System**: NixOS 23.11+ (recommended) or any Linux distribution with Nix
- **CPU**: Minimum 4 cores, recommended 8+ cores
- **RAM**: Minimum 8GB, recommended 16GB+
- **Storage**: 50GB+ SSD for event store
- **Network**: Low-latency network for NATS clustering

### Software Dependencies

```bash
# Install Nix (if not on NixOS)
curl -L https://nixos.org/nix/install | sh

# Clone the repository
git clone https://github.com/thecowboyai/alchemist.git
cd alchemist

# Enter development shell
nix develop
```

## Infrastructure Requirements

### NATS JetStream

NATS is the backbone of CIM's event-driven architecture.

#### Single Node Setup

```yaml
# /etc/nats/nats-server.conf
server_name: cim-nats-1
listen: 0.0.0.0:4222
monitor_port: 8222

jetstream {
  store_dir: "/var/lib/nats/jetstream"
  max_memory_store: 4GB
  max_file_store: 100GB
}

authorization {
  users = [
    {
      user: cim_app
      password: "$2a$11$..."  # Use nats-server passwd to generate
      permissions: {
        publish: ["cmd.>", "query.>"]
        subscribe: ["event.>", "_INBOX.>"]
        allow_responses: true
      }
    }
  ]
}

# TLS Configuration
tls {
  cert_file: "/etc/nats/certs/server.crt"
  key_file: "/etc/nats/certs/server.key"
  ca_file: "/etc/nats/certs/ca.crt"
  verify: true
}
```

#### Cluster Setup (3 nodes)

```yaml
# Node 1: /etc/nats/nats-server-1.conf
server_name: cim-nats-1
listen: 0.0.0.0:4222
cluster {
  name: cim-cluster
  listen: 0.0.0.0:6222
  routes: [
    nats://nats-2.example.com:6222
    nats://nats-3.example.com:6222
  ]
}

jetstream {
  store_dir: "/var/lib/nats/jetstream"
  max_memory_store: 4GB
  max_file_store: 100GB
}
```

### Database (Optional)

While CIM uses event sourcing, you may want a PostgreSQL database for projections:

```sql
-- Create database and user
CREATE DATABASE cim_projections;
CREATE USER cim_app WITH ENCRYPTED PASSWORD 'secure_password';
GRANT ALL PRIVILEGES ON DATABASE cim_projections TO cim_app;

-- Enable required extensions
\c cim_projections
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "btree_gist";
```

## Application Deployment

### Building for Production

```bash
# Build all components
nix build .#cim-alchemist

# Or build specific components
nix build .#cim-domain-graph
nix build .#cim-agent-alchemist

# Build Docker image
nix build .#dockerImage
docker load < result
```

### Configuration

Create production configuration:

```toml
# config/production.toml
[nats]
url = "nats://cim-nats-1.example.com:4222"
user = "cim_app"
password = "${NATS_PASSWORD}"  # Use environment variable
tls_required = true
tls_cert = "/etc/cim/certs/client.crt"
tls_key = "/etc/cim/certs/client.key"
tls_ca = "/etc/cim/certs/ca.crt"

[jetstream]
stream_prefix = "CIM"
consumer_durable = true
ack_wait = "30s"
max_deliver = 3

[application]
log_level = "info"
metrics_port = 9090
health_check_port = 8080

[event_store]
snapshot_interval = 1000
retention_days = 365
compression = "zstd"

[ai_providers]
openai_api_key = "${OPENAI_API_KEY}"
anthropic_api_key = "${ANTHROPIC_API_KEY}"
ollama_host = "http://ollama.internal:11434"
```

### Systemd Service

```ini
# /etc/systemd/system/cim-alchemist.service
[Unit]
Description=CIM Alchemist Service
After=network.target nats.service
Requires=nats.service

[Service]
Type=simple
User=cim
Group=cim
WorkingDirectory=/opt/cim
Environment="RUST_LOG=info,cim=debug"
Environment="CONFIG_PATH=/etc/cim/production.toml"
EnvironmentFile=/etc/cim/env
ExecStart=/opt/cim/bin/cim-alchemist
Restart=always
RestartSec=10

# Security
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/cim

[Install]
WantedBy=multi-user.target
```

## Security Configuration

### TLS Certificates

Generate certificates for NATS and application:

```bash
# Create CA
openssl req -new -x509 -days 3650 -key ca-key.pem -out ca.pem

# Create server certificate
openssl req -new -key server-key.pem -out server.csr
openssl x509 -req -days 365 -in server.csr -CA ca.pem -CAkey ca-key.pem -out server.pem

# Create client certificate
openssl req -new -key client-key.pem -out client.csr
openssl x509 -req -days 365 -in client.csr -CA ca.pem -CAkey ca-key.pem -out client.pem
```

### Network Security

```bash
# Firewall rules (ufw)
ufw allow 4222/tcp  # NATS client
ufw allow 6222/tcp  # NATS cluster
ufw allow 8080/tcp  # Health check
ufw allow 9090/tcp  # Metrics
ufw allow 443/tcp   # HTTPS (if web interface)
```

### Secrets Management

Use environment variables or a secrets manager:

```bash
# /etc/cim/env (chmod 600)
NATS_PASSWORD=secure_password_here
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
DATABASE_URL=postgresql://cim_app:password@localhost/cim_projections
```

## Monitoring Setup

### Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'cim-alchemist'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: '/metrics'

  - job_name: 'nats'
    static_configs:
      - targets: ['localhost:8222']
    metrics_path: '/metrics'
```

### Grafana Dashboards

Import the provided dashboards:

```bash
# Copy dashboard files
cp monitoring/dashboards/*.json /var/lib/grafana/dashboards/

# Key metrics to monitor:
# - Event processing rate
# - Command success/failure rate
# - Query latency (p50, p95, p99)
# - NATS connection status
# - Memory usage
# - CPU usage
```

### Health Checks

```bash
# Basic health check
curl http://localhost:8080/health

# Detailed health check
curl http://localhost:8080/health/detailed
```

### Logging

Configure structured logging:

```toml
# In production.toml
[logging]
format = "json"
level = "info"
output = "/var/log/cim/alchemist.log"

# Separate error log
error_output = "/var/log/cim/error.log"

# Log rotation
max_size = "100MB"
max_backups = 10
max_age = 30
```

## Performance Tuning

### NATS Optimization

```yaml
# Optimized NATS configuration
jetstream {
  store_dir: "/var/lib/nats/jetstream"
  max_memory_store: 8GB
  max_file_store: 500GB
  
  # Performance tuning
  sync_interval: "30s"
  max_outstanding_catchup: 64MB
}

# Connection limits
max_connections: 10000
max_control_line: 4KB
max_payload: 8MB
max_pending: 128MB

# Write performance
write_deadline: "10s"
```

### Application Tuning

```toml
# production.toml performance section
[performance]
# Event processing
event_batch_size = 100
event_batch_timeout = "10ms"
max_concurrent_handlers = 50

# Query optimization
query_cache_size = 10000
query_cache_ttl = "5m"

# Connection pooling
max_nats_connections = 10
connection_timeout = "5s"

# Memory management
max_memory_percent = 80
gc_interval = "1m"
```

### System Tuning

```bash
# /etc/sysctl.d/99-cim.conf
# Network performance
net.core.rmem_max = 134217728
net.core.wmem_max = 134217728
net.ipv4.tcp_rmem = 4096 87380 134217728
net.ipv4.tcp_wmem = 4096 65536 134217728
net.core.netdev_max_backlog = 5000

# File descriptors
fs.file-max = 1000000

# Apply settings
sysctl -p /etc/sysctl.d/99-cim.conf
```

## Backup and Recovery

### Event Store Backup

```bash
#!/bin/bash
# /opt/cim/scripts/backup.sh

BACKUP_DIR="/backup/cim/$(date +%Y%m%d_%H%M%S)"
mkdir -p "$BACKUP_DIR"

# Stop writes (optional)
curl -X POST http://localhost:8080/admin/pause_writes

# Backup NATS JetStream
cp -r /var/lib/nats/jetstream "$BACKUP_DIR/"

# Backup configuration
cp -r /etc/cim "$BACKUP_DIR/config"

# Backup projections database
pg_dump -h localhost -U cim_app cim_projections | gzip > "$BACKUP_DIR/projections.sql.gz"

# Resume writes
curl -X POST http://localhost:8080/admin/resume_writes

# Upload to S3 (optional)
aws s3 sync "$BACKUP_DIR" "s3://cim-backups/$(basename $BACKUP_DIR)"
```

### Recovery Procedure

```bash
#!/bin/bash
# /opt/cim/scripts/restore.sh

RESTORE_FROM="$1"

# Stop services
systemctl stop cim-alchemist
systemctl stop nats

# Restore NATS data
rm -rf /var/lib/nats/jetstream
cp -r "$RESTORE_FROM/jetstream" /var/lib/nats/

# Restore configuration
cp -r "$RESTORE_FROM/config/*" /etc/cim/

# Restore database
gunzip < "$RESTORE_FROM/projections.sql.gz" | psql -h localhost -U cim_app cim_projections

# Start services
systemctl start nats
systemctl start cim-alchemist
```

## Troubleshooting

### Common Issues

#### NATS Connection Issues

```bash
# Check NATS status
systemctl status nats
nats-top

# Test connection
nats pub test.ping "hello" --server nats://localhost:4222

# Check TLS
openssl s_client -connect localhost:4222 -cert client.pem -key client-key.pem
```

#### Event Processing Stopped

```bash
# Check consumer status
nats consumer info CIM-EVENTS CIM-CONSUMER

# Check for poison messages
nats stream view CIM-EVENTS --subject "event.error.>"

# Reset consumer
nats consumer rm CIM-EVENTS CIM-CONSUMER
nats consumer add CIM-EVENTS CIM-CONSUMER --config consumer.json
```

#### High Memory Usage

```bash
# Check memory usage by component
curl http://localhost:9090/metrics | grep memory

# Force garbage collection
curl -X POST http://localhost:8080/admin/gc

# Check for memory leaks
GODEBUG=gctrace=1 /opt/cim/bin/cim-alchemist
```

### Debug Mode

Enable debug logging temporarily:

```bash
# Via systemd
systemctl edit cim-alchemist
# Add: Environment="RUST_LOG=debug,cim=trace"
systemctl restart cim-alchemist

# Via kill signal
kill -USR1 $(pidof cim-alchemist)  # Toggle debug mode
```

### Performance Profiling

```bash
# Enable profiling endpoint
curl -X POST http://localhost:8080/admin/profiling/enable

# Capture CPU profile
go tool pprof http://localhost:8080/debug/pprof/profile?seconds=30

# Capture heap profile
go tool pprof http://localhost:8080/debug/pprof/heap
```

## Production Checklist

Before going live:

- [ ] TLS enabled for all connections
- [ ] Authentication configured for NATS
- [ ] Firewall rules configured
- [ ] Monitoring dashboards set up
- [ ] Alerts configured
- [ ] Backup scripts tested
- [ ] Recovery procedure documented
- [ ] Load testing completed
- [ ] Security scan passed
- [ ] Documentation updated

## Support

For production support:

- **GitHub Issues**: https://github.com/thecowboyai/alchemist/issues
- **Documentation**: https://cim.thecowboy.ai/docs
- **Community**: Discord/Slack channels

---

*Last updated: January 2025*
*Version: 0.4.2* 
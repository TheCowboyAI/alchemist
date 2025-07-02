# CIM Deployment Documentation

Comprehensive guides for deploying and operating CIM in production environments.

## 📚 Deployment Guides

### [Production Deployment Guide](./production-deployment-guide.md)
Complete guide for deploying CIM in production:
- Infrastructure requirements
- NATS configuration
- Application deployment
- Security setup
- Backup and recovery

### [Monitoring Setup](./monitoring-setup.md)
Comprehensive monitoring and observability:
- Prometheus metrics
- Grafana dashboards
- Distributed tracing
- Log aggregation
- Health checks

### [Performance Optimization](./performance-optimization.md)
Optimization strategies for production:
- Event processing optimization
- Query performance tuning
- Memory optimization
- Concurrency tuning
- Profiling and benchmarking

## 🚀 Quick Start

For a production deployment, follow these steps in order:

1. **Review Requirements**: Check system and infrastructure requirements in the [Production Deployment Guide](./production-deployment-guide.md#prerequisites)

2. **Set Up Infrastructure**: 
   - Install NATS with JetStream
   - Configure networking and security
   - Set up monitoring infrastructure

3. **Deploy Application**:
   ```bash
   # Build for production
   nix build .#cim-alchemist
   
   # Deploy with systemd
   sudo cp result/bin/cim-alchemist /opt/cim/bin/
   sudo systemctl enable cim-alchemist
   sudo systemctl start cim-alchemist
   ```

4. **Configure Monitoring**: Set up Prometheus, Grafana, and alerts

5. **Optimize Performance**: Apply tuning based on workload

## 🔧 Configuration Examples

### Minimal Production Config

```toml
# /etc/cim/production.toml
[nats]
url = "nats://localhost:4222"
user = "cim_app"
password = "${NATS_PASSWORD}"

[application]
log_level = "info"
metrics_port = 9090

[event_store]
snapshot_interval = 1000
retention_days = 30
```

### High-Performance Config

```toml
# /etc/cim/high-performance.toml
[nats]
url = "nats://nats-cluster:4222"
min_connections = 10
max_connections = 50

[event_processing]
batch_size = 500
batch_timeout = "5ms"
max_concurrent_batches = 20

[query_cache]
enabled = true
max_entries = 50000
ttl_seconds = 600

[performance]
worker_threads = 16
memory_pool_size = "2GB"
```

## 🏗️ Deployment Architectures

### Single Node
- Suitable for development and small deployments
- All components on one server
- Simple to manage

### Multi-Node Cluster
- NATS cluster for high availability
- Application instances behind load balancer
- Shared storage for projections

### Kubernetes
- Containerized deployment
- Horizontal pod autoscaling
- Service mesh integration

## 📊 Capacity Planning

| Workload                  | CPU       | RAM   | Storage   | Network  |
| ------------------------- | --------- | ----- | --------- | -------- |
| Small (< 1K events/sec)   | 4 cores   | 8GB   | 50GB SSD  | 1 Gbps   |
| Medium (< 10K events/sec) | 8 cores   | 16GB  | 200GB SSD | 10 Gbps  |
| Large (< 100K events/sec) | 16+ cores | 32GB+ | 1TB+ NVMe | 10+ Gbps |

## 🔒 Security Checklist

- [ ] TLS enabled for all connections
- [ ] Authentication configured
- [ ] Network segmentation in place
- [ ] Secrets managed securely
- [ ] Regular security updates
- [ ] Audit logging enabled
- [ ] Backup encryption configured

## 🆘 Troubleshooting

Common issues and solutions:

### NATS Connection Failed
```bash
# Check NATS status
systemctl status nats
nats-top

# Test connectivity
nats pub test "hello" --server nats://localhost:4222
```

### High Memory Usage
```bash
# Check memory breakdown
curl http://localhost:9090/metrics | grep memory

# Force garbage collection
curl -X POST http://localhost:8080/admin/gc
```

### Slow Queries
```bash
# Check query metrics
curl http://localhost:9090/metrics | grep query_duration

# Analyze slow query log
tail -f /var/log/cim/slow-queries.log
```

## 📚 Additional Resources

- [CIM Architecture Overview](../architecture/README.md)
- [Domain Documentation](../domains/README.md)
- [API Reference](../api/README.md)
- [Development Guide](../guides/README.md)

## 🤝 Getting Help

- **GitHub Issues**: [Report deployment issues](https://github.com/thecowboyai/alchemist/issues)
- **Community Chat**: Join our Discord/Slack
- **Commercial Support**: Contact support@thecowboy.ai

---

*Last updated: January 2025*
*Version: 0.4.2* 
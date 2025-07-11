# Alchemist Deployment Guide

This directory contains production-ready configuration files and deployment scripts for the Alchemist AI Agent platform.

## Directory Structure

```
deployment/
├── configs/                 # Environment-specific configurations
│   ├── development/        # Local development settings
│   ├── staging/           # Staging environment settings
│   └── production/        # Production environment settings
├── docker/                 # Docker deployment files
│   ├── Dockerfile.production
│   └── docker-compose.production.yml
├── kubernetes/            # Kubernetes manifests
│   ├── namespace.yaml
│   ├── configmap.yaml
│   ├── deployment.yaml
│   └── service.yaml
├── systemd/               # SystemD service files
│   ├── alchemist.service
│   ├── alchemist-nats.service
│   └── alchemist-ollama.service
├── scripts/               # Deployment automation scripts
│   ├── deploy.sh
│   ├── rollback.sh
│   └── health-check.sh
└── monitoring/            # Monitoring configurations
    ├── prometheus.yml
    ├── alerts/
    └── grafana/
```

## Quick Start

### Docker Deployment

1. **Development Environment**
   ```bash
   cd deployment/docker
   docker-compose -f docker-compose.development.yml up -d
   ```

2. **Production Environment**
   ```bash
   # Configure environment
   cp configs/production/.env.example configs/production/.env
   # Edit .env file with your values
   
   # Deploy
   ./scripts/deploy.sh
   ```

### Kubernetes Deployment

1. **Create namespace and secrets**
   ```bash
   kubectl apply -f kubernetes/namespace.yaml
   kubectl create secret generic alchemist-secrets \
     --from-literal=anthropic-api-key=your-key \
     --from-literal=database-url=your-db-url \
     --from-literal=redis-url=your-redis-url \
     --from-literal=jwt-secret=your-secret \
     -n alchemist
   ```

2. **Deploy the application**
   ```bash
   kubectl apply -f kubernetes/
   ```

### SystemD Deployment

1. **Install service files**
   ```bash
   sudo cp systemd/*.service /etc/systemd/system/
   sudo systemctl daemon-reload
   ```

2. **Start services**
   ```bash
   sudo systemctl enable --now alchemist-nats
   sudo systemctl enable --now alchemist-ollama
   sudo systemctl enable --now alchemist
   ```

## Configuration

### Environment Variables

Key environment variables that must be configured:

- `ANTHROPIC_API_KEY`: Anthropic API key for Claude
- `OPENAI_API_KEY`: OpenAI API key (optional fallback)
- `DATABASE_URL`: PostgreSQL connection string
- `REDIS_URL`: Redis connection string
- `JWT_SECRET`: Secret for JWT token signing
- `NATS_URL`: NATS server URL
- `LOG_LEVEL`: Logging level (debug, info, warn, error)

### Configuration Files

Each environment has two main configuration files:

1. **alchemist.toml**: Rust-specific configuration for the Alchemist binary
2. **config.yaml**: General service configuration

### Security Considerations

1. **TLS/SSL**: Production deployments should use TLS for all connections
2. **Secrets Management**: Use proper secret management (Kubernetes Secrets, HashiCorp Vault, etc.)
3. **Network Policies**: Implement network segmentation and policies
4. **Authentication**: Enable JWT authentication in production
5. **API Keys**: Rotate API keys regularly

## Deployment Process

### Production Deployment Steps

1. **Pre-deployment Checks**
   ```bash
   ./scripts/health-check.sh
   ```

2. **Backup Current State**
   ```bash
   # Automated backup is part of deploy.sh
   # Manual backup:
   docker-compose exec postgres pg_dump -U alchemist alchemist > backup.sql
   ```

3. **Deploy New Version**
   ```bash
   VERSION=v1.2.0 ./scripts/deploy.sh
   ```

4. **Verify Deployment**
   ```bash
   ./scripts/health-check.sh
   curl http://localhost:8081/health/ready
   ```

### Rollback Process

If issues occur, rollback to previous version:

```bash
./scripts/rollback.sh
```

## Monitoring

### Metrics

Prometheus metrics are exposed on port 9090 at `/metrics`

Key metrics to monitor:
- `alchemist_http_requests_total`: Total HTTP requests
- `alchemist_http_request_duration_seconds`: Request latency
- `alchemist_ai_provider_requests_total`: AI provider usage
- `alchemist_workflow_executions_total`: Workflow execution count

### Dashboards

Grafana dashboards are available at http://localhost:3000

Default credentials:
- Username: admin
- Password: Set via `GRAFANA_PASSWORD` environment variable

### Alerts

Critical alerts configured:
- Service downtime
- High error rates (>5%)
- Slow response times (>2s p95)
- High memory usage (>3.5GB)
- AI provider failures
- Database connection issues

## Scaling

### Horizontal Scaling

1. **Docker Compose**
   ```bash
   docker-compose up -d --scale alchemist=4
   ```

2. **Kubernetes**
   ```bash
   kubectl scale deployment alchemist --replicas=5 -n alchemist
   ```

### Vertical Scaling

Adjust resource limits in:
- Docker: `deploy.resources` section in docker-compose.yml
- Kubernetes: `resources` section in deployment.yaml
- SystemD: `MemoryLimit` and `CPUQuota` in service files

## Troubleshooting

### Common Issues

1. **Service won't start**
   - Check logs: `docker logs alchemist-agent`
   - Verify dependencies are running
   - Check configuration syntax

2. **Database connection errors**
   - Verify DATABASE_URL is correct
   - Check network connectivity
   - Ensure migrations have run

3. **High memory usage**
   - Check for memory leaks with metrics
   - Adjust cache sizes
   - Scale horizontally

### Debug Mode

Enable debug logging:
```bash
LOG_LEVEL=debug ./scripts/deploy.sh
```

### Health Checks

Run comprehensive health check:
```bash
./scripts/health-check.sh
```

## Maintenance

### Regular Tasks

1. **Weekly**
   - Review metrics and alerts
   - Check disk usage
   - Verify backups

2. **Monthly**
   - Update dependencies
   - Rotate credentials
   - Performance analysis

3. **Quarterly**
   - Security audit
   - Capacity planning
   - Disaster recovery test

### Backup Strategy

Automated backups include:
- PostgreSQL database dumps
- Docker volume snapshots
- Configuration files

Backups are stored in `/var/backups/alchemist` with 90-day retention.

## Support

For deployment issues:
1. Check the [troubleshooting guide](#troubleshooting)
2. Review application logs
3. Contact the development team

## License

See the main project LICENSE file.
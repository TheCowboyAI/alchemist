# Production Deployment Guide for Alchemist

This guide provides comprehensive instructions for deploying Alchemist in production environments.

## Table of Contents
1. [Prerequisites](#prerequisites)
2. [Docker Deployment](#docker-deployment)
3. [Kubernetes Deployment](#kubernetes-deployment)
4. [Security Configuration](#security-configuration)
5. [Monitoring Setup](#monitoring-setup)
6. [Backup and Recovery](#backup-and-recovery)
7. [Performance Tuning](#performance-tuning)
8. [Troubleshooting](#troubleshooting)

## Prerequisites

### System Requirements
- **CPU**: Minimum 4 cores, recommended 8+ cores
- **Memory**: Minimum 8GB RAM, recommended 16GB+ RAM
- **Storage**: 100GB+ SSD storage for data persistence
- **Network**: Stable internet connection for AI provider APIs

### Software Requirements
- Docker 20.10+ and Docker Compose 2.0+ (for Docker deployment)
- Kubernetes 1.25+ (for Kubernetes deployment)
- Helm 3.0+ (optional, for Kubernetes)
- NATS 2.10+ with JetStream enabled
- PostgreSQL 15+ or compatible database
- Redis 7.0+ for caching
- Qdrant 1.7+ for vector search

### API Keys Required
- OpenAI API key (for GPT-4 integration)
- Anthropic API key (for Claude integration)
- JWT secret for authentication
- API key for service authentication

## Docker Deployment

### 1. Environment Setup

Create a `.env` file with required secrets:
```bash
# API Keys
OPENAI_API_KEY=your_openai_api_key
ANTHROPIC_API_KEY=your_anthropic_api_key
ALCHEMIST_JWT_SECRET=your_jwt_secret_here
ALCHEMIST_API_KEY=your_api_key_here

# Database Passwords
POSTGRES_PASSWORD=secure_postgres_password
REDIS_PASSWORD=secure_redis_password

# Grafana
GRAFANA_ADMIN_PASSWORD=secure_grafana_password
```

### 2. TLS Certificate Setup

Generate or obtain TLS certificates:
```bash
# Create certificate directories
mkdir -p deployment/certs/{nginx,nats,qdrant,alchemist}

# For production, use Let's Encrypt or your CA
# For testing, generate self-signed certificates:
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout deployment/certs/nginx/server.key \
  -out deployment/certs/nginx/server.crt \
  -subj "/C=US/ST=State/L=City/O=Organization/CN=api.alchemist.example.com"
```

### 3. Deploy with Docker Compose

```bash
# Pull latest images
docker-compose -f deployment/docker-compose.production.yml pull

# Start services
docker-compose -f deployment/docker-compose.production.yml up -d

# Check service health
docker-compose -f deployment/docker-compose.production.yml ps

# View logs
docker-compose -f deployment/docker-compose.production.yml logs -f alchemist
```

### 4. Verify Deployment

```bash
# Check API health
curl -k https://localhost/health

# Check metrics endpoint
curl http://localhost:9090/metrics

# Access Grafana dashboard
# Navigate to http://localhost:3000 (admin/your_password)
```

## Kubernetes Deployment

### 1. Namespace Setup

```bash
# Create namespace
kubectl create namespace alchemist-prod

# Set default namespace
kubectl config set-context --current --namespace=alchemist-prod
```

### 2. Create Secrets

```bash
# Create secret for API keys
kubectl create secret generic alchemist-secrets \
  --from-literal=openai-api-key=$OPENAI_API_KEY \
  --from-literal=anthropic-api-key=$ANTHROPIC_API_KEY \
  --from-literal=jwt-secret=$ALCHEMIST_JWT_SECRET \
  --from-literal=api-key=$ALCHEMIST_API_KEY

# Create TLS secret
kubectl create secret tls alchemist-tls \
  --cert=path/to/tls.crt \
  --key=path/to/tls.key
```

### 3. Deploy Supporting Services

```bash
# Deploy NATS
helm repo add nats https://nats-io.github.io/k8s/helm/charts/
helm install nats nats/nats \
  --set nats.jetstream.enabled=true \
  --set nats.jetstream.memStorage.enabled=true \
  --set nats.jetstream.memStorage.size=2Gi

# Deploy Redis
helm repo add bitnami https://charts.bitnami.com/bitnami
helm install redis bitnami/redis \
  --set auth.password=$REDIS_PASSWORD \
  --set replica.replicaCount=3

# Deploy PostgreSQL
helm install postgres bitnami/postgresql \
  --set auth.postgresPassword=$POSTGRES_PASSWORD \
  --set auth.database=alchemist \
  --set primary.persistence.size=50Gi
```

### 4. Deploy Alchemist

```bash
# Apply ConfigMaps
kubectl apply -f deployment/kubernetes/alchemist-configmap.yaml

# Create PersistentVolumeClaim
kubectl apply -f - <<EOF
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: alchemist-data
  namespace: alchemist-prod
spec:
  accessModes:
    - ReadWriteMany
  resources:
    requests:
      storage: 100Gi
  storageClassName: fast-ssd
EOF

# Deploy application
kubectl apply -f deployment/kubernetes/alchemist-deployment.yaml

# Deploy Ingress
kubectl apply -f deployment/kubernetes/alchemist-ingress.yaml

# Check deployment status
kubectl rollout status deployment/alchemist
```

### 5. Verify Kubernetes Deployment

```bash
# Check pods
kubectl get pods -l app=alchemist

# Check services
kubectl get svc

# Check ingress
kubectl get ingress

# View logs
kubectl logs -f deployment/alchemist

# Test connectivity
kubectl run -it --rm debug --image=curlimages/curl --restart=Never -- \
  curl http://alchemist:8080/health
```

## Security Configuration

### 1. Network Security

- Use NetworkPolicies to restrict pod-to-pod communication
- Enable TLS for all external connections
- Implement IP whitelisting for admin endpoints
- Use WAF (Web Application Firewall) for public endpoints

### 2. Authentication & Authorization

- Enable JWT authentication for API access
- Use API keys for service-to-service communication
- Implement RBAC for Kubernetes resources
- Rotate secrets regularly

### 3. Data Security

- Enable encryption at rest for databases
- Use encrypted connections for all data transfers
- Implement data retention policies
- Regular security audits

### 4. Container Security

```bash
# Scan images for vulnerabilities
docker scan alchemist:latest

# Run containers as non-root user
# Set security contexts in Kubernetes
# Enable read-only root filesystem where possible
```

## Monitoring Setup

### 1. Prometheus Configuration

```bash
# Deploy Prometheus
helm install prometheus prometheus-community/kube-prometheus-stack \
  --set prometheus.prometheusSpec.serviceMonitorSelectorNilUsesHelmValues=false

# Apply ServiceMonitor for Alchemist
kubectl apply -f - <<EOF
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: alchemist
  namespace: alchemist-prod
spec:
  selector:
    matchLabels:
      app: alchemist
  endpoints:
  - port: metrics
    interval: 30s
    path: /metrics
EOF
```

### 2. Grafana Dashboards

Import provided dashboards:
- Alchemist Overview Dashboard
- Domain Event Flow Dashboard
- AI Provider Usage Dashboard
- System Performance Dashboard

### 3. Alerting Rules

```yaml
# Example alert rules
groups:
- name: alchemist
  rules:
  - alert: HighErrorRate
    expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.05
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "High error rate detected"
      
  - alert: HighMemoryUsage
    expr: container_memory_usage_bytes{pod=~"alchemist-.*"} / container_spec_memory_limit_bytes > 0.9
    for: 5m
    labels:
      severity: warning
```

## Backup and Recovery

### 1. Database Backups

```bash
# PostgreSQL backup
kubectl exec -it postgres-0 -- pg_dump -U alchemist alchemist > backup-$(date +%Y%m%d).sql

# Redis backup
kubectl exec -it redis-master-0 -- redis-cli BGSAVE

# Qdrant backup
curl -X POST http://qdrant:6333/collections/backup
```

### 2. Persistent Volume Backups

Use volume snapshots or backup tools like Velero:
```bash
# Install Velero
helm install velero vmware-tanzu/velero \
  --set provider=aws \
  --set bucket=alchemist-backups

# Create backup
velero backup create alchemist-backup --include-namespaces alchemist-prod
```

### 3. Disaster Recovery Plan

1. Regular automated backups (daily minimum)
2. Off-site backup storage
3. Documented recovery procedures
4. Regular recovery drills
5. RTO/RPO targets defined

## Performance Tuning

### 1. Resource Optimization

```yaml
# Optimize pod resources based on usage
resources:
  requests:
    cpu: "2"
    memory: "4Gi"
  limits:
    cpu: "4"
    memory: "8Gi"
```

### 2. Database Tuning

```sql
-- PostgreSQL optimization
ALTER SYSTEM SET shared_buffers = '4GB';
ALTER SYSTEM SET effective_cache_size = '12GB';
ALTER SYSTEM SET maintenance_work_mem = '1GB';
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
```

### 3. Caching Strategy

- Use Redis for frequently accessed data
- Implement CDN for static assets
- Enable HTTP caching headers
- Use in-memory caches where appropriate

### 4. Scaling Configuration

```yaml
# HPA settings for auto-scaling
metrics:
- type: Resource
  resource:
    name: cpu
    target:
      type: Utilization
      averageUtilization: 70
- type: Pods
  pods:
    metric:
      name: http_requests_per_second
    target:
      type: AverageValue
      averageValue: "1000"
```

## Troubleshooting

### Common Issues

1. **Service Won't Start**
   - Check logs: `kubectl logs -f deployment/alchemist`
   - Verify secrets are created correctly
   - Check resource limits

2. **Database Connection Issues**
   - Verify network policies
   - Check connection strings
   - Ensure database is running

3. **High Memory Usage**
   - Review memory leaks
   - Adjust garbage collection settings
   - Scale horizontally

4. **Slow Response Times**
   - Check database query performance
   - Review caching effectiveness
   - Monitor network latency

### Debug Commands

```bash
# Get detailed pod information
kubectl describe pod <pod-name>

# Execute commands in container
kubectl exec -it <pod-name> -- /bin/sh

# Check events
kubectl get events --sort-by='.lastTimestamp'

# Resource usage
kubectl top pods
kubectl top nodes

# Network debugging
kubectl run -it --rm debug --image=nicolaka/netshoot --restart=Never -- /bin/bash
```

### Support

For production support:
- Documentation: https://alchemist.example.com/docs
- Issues: https://github.com/thecowboyai/alchemist/issues
- Emergency: security@alchemist.example.com

## Production Checklist

- [ ] All secrets configured and secured
- [ ] TLS certificates installed and valid
- [ ] Monitoring and alerting configured
- [ ] Backup procedures tested
- [ ] Security scanning completed
- [ ] Performance benchmarks met
- [ ] Disaster recovery plan documented
- [ ] Team trained on procedures
- [ ] Documentation up to date
- [ ] Support channels established
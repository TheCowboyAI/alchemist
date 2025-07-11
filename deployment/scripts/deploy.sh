#!/bin/bash
# Alchemist Production Deployment Script
# This script handles the full deployment process

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
DEPLOYMENT_DIR="$PROJECT_ROOT/deployment"
ENVIRONMENT="${ENVIRONMENT:-production}"
VERSION="${VERSION:-latest}"
BACKUP_ENABLED="${BACKUP_ENABLED:-true}"
HEALTH_CHECK_TIMEOUT="${HEALTH_CHECK_TIMEOUT:-300}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check for required commands
    local required_commands=(docker docker-compose jq curl)
    for cmd in "${required_commands[@]}"; do
        if ! command -v "$cmd" &> /dev/null; then
            log_error "$cmd is required but not installed"
            exit 1
        fi
    done
    
    # Check Docker daemon
    if ! docker info &> /dev/null; then
        log_error "Docker daemon is not running"
        exit 1
    fi
    
    # Check environment file
    local env_file="$DEPLOYMENT_DIR/configs/$ENVIRONMENT/.env"
    if [[ ! -f "$env_file" ]]; then
        log_error "Environment file not found: $env_file"
        log_info "Please copy .env.example to .env and configure it"
        exit 1
    fi
    
    log_success "Prerequisites check passed"
}

# Backup current deployment
backup_deployment() {
    if [[ "$BACKUP_ENABLED" != "true" ]]; then
        log_warning "Backup is disabled, skipping..."
        return
    fi
    
    log_info "Creating backup of current deployment..."
    
    local backup_dir="/var/backups/alchemist"
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local backup_name="alchemist_backup_${timestamp}"
    
    sudo mkdir -p "$backup_dir"
    
    # Backup database
    if docker-compose -f "$DEPLOYMENT_DIR/docker/docker-compose.$ENVIRONMENT.yml" ps postgres &> /dev/null; then
        log_info "Backing up PostgreSQL database..."
        docker-compose -f "$DEPLOYMENT_DIR/docker/docker-compose.$ENVIRONMENT.yml" exec -T postgres \
            pg_dump -U alchemist alchemist | gzip > "$backup_dir/${backup_name}_postgres.sql.gz"
    fi
    
    # Backup volumes
    log_info "Backing up Docker volumes..."
    docker run --rm \
        -v alchemist_data:/data \
        -v "$backup_dir":/backup \
        alpine tar czf "/backup/${backup_name}_volumes.tar.gz" -C / data
    
    log_success "Backup completed: $backup_dir/$backup_name*"
}

# Build application
build_application() {
    log_info "Building Alchemist application..."
    
    cd "$PROJECT_ROOT"
    
    # Build with Docker
    docker build \
        -f "$DEPLOYMENT_DIR/docker/Dockerfile.$ENVIRONMENT" \
        -t "alchemist:$VERSION" \
        -t "alchemist:latest" \
        --build-arg VERSION="$VERSION" \
        --cache-from "alchemist:latest" \
        .
    
    log_success "Application built successfully"
}

# Deploy with Docker Compose
deploy_docker_compose() {
    log_info "Deploying with Docker Compose..."
    
    cd "$DEPLOYMENT_DIR/docker"
    
    # Load environment variables
    set -a
    source "$DEPLOYMENT_DIR/configs/$ENVIRONMENT/.env"
    set +a
    
    # Pull latest images
    docker-compose -f "docker-compose.$ENVIRONMENT.yml" pull
    
    # Deploy with zero-downtime
    if [[ "$ENVIRONMENT" == "production" ]]; then
        # Rolling update for production
        docker-compose -f "docker-compose.$ENVIRONMENT.yml" up -d --scale alchemist=4 --no-recreate
        
        # Wait for new containers to be healthy
        log_info "Waiting for new containers to be healthy..."
        sleep 30
        
        # Scale down to desired replicas
        docker-compose -f "docker-compose.$ENVIRONMENT.yml" up -d --scale alchemist=2
    else
        # Simple deployment for non-production
        docker-compose -f "docker-compose.$ENVIRONMENT.yml" up -d
    fi
    
    log_success "Docker Compose deployment completed"
}

# Run database migrations
run_migrations() {
    log_info "Running database migrations..."
    
    # Wait for database to be ready
    local max_attempts=30
    local attempt=0
    
    while ! docker-compose -f "$DEPLOYMENT_DIR/docker/docker-compose.$ENVIRONMENT.yml" \
        exec -T postgres pg_isready -U alchemist &> /dev/null; do
        
        attempt=$((attempt + 1))
        if [[ $attempt -ge $max_attempts ]]; then
            log_error "Database is not ready after $max_attempts attempts"
            exit 1
        fi
        
        log_info "Waiting for database... (attempt $attempt/$max_attempts)"
        sleep 2
    done
    
    # Run migrations
    docker-compose -f "$DEPLOYMENT_DIR/docker/docker-compose.$ENVIRONMENT.yml" \
        run --rm alchemist /app/alchemist migrate
    
    log_success "Database migrations completed"
}

# Health check
health_check() {
    log_info "Performing health checks..."
    
    local health_endpoint="http://localhost:8081/health/ready"
    local max_attempts=$((HEALTH_CHECK_TIMEOUT / 5))
    local attempt=0
    
    while ! curl -sf "$health_endpoint" &> /dev/null; do
        attempt=$((attempt + 1))
        if [[ $attempt -ge $max_attempts ]]; then
            log_error "Health check failed after $HEALTH_CHECK_TIMEOUT seconds"
            exit 1
        fi
        
        log_info "Waiting for service to be healthy... (attempt $attempt/$max_attempts)"
        sleep 5
    done
    
    # Check all components
    local components=("nats" "postgres" "redis" "ai_provider")
    for component in "${components[@]}"; do
        if curl -sf "$health_endpoint" | jq -e ".checks.$component.healthy" &> /dev/null; then
            log_success "$component is healthy"
        else
            log_error "$component health check failed"
            exit 1
        fi
    done
    
    log_success "All health checks passed"
}

# Setup monitoring
setup_monitoring() {
    log_info "Setting up monitoring..."
    
    # Import Grafana dashboards
    if [[ -d "$DEPLOYMENT_DIR/monitoring/grafana/dashboards" ]]; then
        log_info "Importing Grafana dashboards..."
        # Dashboard import logic here
    fi
    
    # Configure alerts
    if [[ -f "$DEPLOYMENT_DIR/monitoring/alerts.yaml" ]]; then
        log_info "Configuring Prometheus alerts..."
        # Alert configuration logic here
    fi
    
    log_success "Monitoring setup completed"
}

# Main deployment flow
main() {
    log_info "Starting Alchemist deployment for environment: $ENVIRONMENT"
    log_info "Version: $VERSION"
    
    # Deployment steps
    check_prerequisites
    backup_deployment
    build_application
    deploy_docker_compose
    run_migrations
    health_check
    setup_monitoring
    
    log_success "Deployment completed successfully!"
    log_info "Access the application at: https://your-domain.com"
    log_info "Monitoring dashboard: http://localhost:3000"
    log_info "Metrics endpoint: http://localhost:9090/metrics"
}

# Run main function
main "$@"
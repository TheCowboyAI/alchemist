#!/bin/bash
# Alchemist Health Check Script
# This script performs comprehensive health checks on all components

set -euo pipefail

# Configuration
HEALTH_ENDPOINT="${HEALTH_ENDPOINT:-http://localhost:8081/health}"
METRICS_ENDPOINT="${METRICS_ENDPOINT:-http://localhost:9090/metrics}"
NATS_URL="${NATS_URL:-nats://localhost:4222}"
POSTGRES_HOST="${POSTGRES_HOST:-localhost}"
REDIS_HOST="${REDIS_HOST:-localhost}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Exit codes
EXIT_SUCCESS=0
EXIT_WARNING=1
EXIT_CRITICAL=2

# Component status
declare -A component_status

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

log_error() {
    echo -e "${RED}[✗]${NC} $1"
}

# Check Alchemist service
check_alchemist() {
    log_info "Checking Alchemist service..."
    
    # Liveness check
    if curl -sf "$HEALTH_ENDPOINT/live" &> /dev/null; then
        log_success "Alchemist liveness check passed"
        component_status["alchemist_live"]="ok"
    else
        log_error "Alchemist liveness check failed"
        component_status["alchemist_live"]="critical"
        return 1
    fi
    
    # Readiness check
    local ready_response=$(curl -sf "$HEALTH_ENDPOINT/ready" 2>/dev/null || echo "{}")
    if [[ -n "$ready_response" ]] && echo "$ready_response" | jq -e '.status == "ready"' &> /dev/null; then
        log_success "Alchemist readiness check passed"
        component_status["alchemist_ready"]="ok"
        
        # Check individual components from readiness response
        local components=("nats" "database" "cache" "ai_provider")
        for comp in "${components[@]}"; do
            if echo "$ready_response" | jq -e ".checks.$comp.healthy" &> /dev/null; then
                log_success "  └─ $comp: healthy"
            else
                log_warning "  └─ $comp: unhealthy"
                component_status["alchemist_$comp"]="warning"
            fi
        done
    else
        log_error "Alchemist readiness check failed"
        component_status["alchemist_ready"]="critical"
        return 1
    fi
    
    return 0
}

# Check NATS
check_nats() {
    log_info "Checking NATS server..."
    
    if command -v nats &> /dev/null; then
        if nats server ping --server="$NATS_URL" &> /dev/null; then
            log_success "NATS server is responding"
            component_status["nats"]="ok"
            
            # Check JetStream
            if nats server report jetstream --server="$NATS_URL" &> /dev/null; then
                log_success "  └─ JetStream: enabled"
            else
                log_warning "  └─ JetStream: not available"
                component_status["nats_jetstream"]="warning"
            fi
        else
            log_error "NATS server not responding"
            component_status["nats"]="critical"
            return 1
        fi
    else
        # Fallback to HTTP monitoring endpoint
        if curl -sf "http://localhost:8222/healthz" &> /dev/null; then
            log_success "NATS server is healthy (HTTP check)"
            component_status["nats"]="ok"
        else
            log_error "NATS server health check failed"
            component_status["nats"]="critical"
            return 1
        fi
    fi
    
    return 0
}

# Check PostgreSQL
check_postgres() {
    log_info "Checking PostgreSQL database..."
    
    if command -v pg_isready &> /dev/null; then
        if pg_isready -h "$POSTGRES_HOST" -U alchemist &> /dev/null; then
            log_success "PostgreSQL is ready"
            component_status["postgres"]="ok"
        else
            log_error "PostgreSQL is not ready"
            component_status["postgres"]="critical"
            return 1
        fi
    else
        # Docker-based check
        if docker exec alchemist-postgres pg_isready -U alchemist &> /dev/null; then
            log_success "PostgreSQL is ready (Docker check)"
            component_status["postgres"]="ok"
        else
            log_error "PostgreSQL health check failed"
            component_status["postgres"]="critical"
            return 1
        fi
    fi
    
    return 0
}

# Check Redis
check_redis() {
    log_info "Checking Redis cache..."
    
    if command -v redis-cli &> /dev/null; then
        if redis-cli -h "$REDIS_HOST" ping &> /dev/null; then
            log_success "Redis is responding"
            component_status["redis"]="ok"
            
            # Check memory usage
            local used_memory=$(redis-cli -h "$REDIS_HOST" info memory | grep used_memory_human | cut -d: -f2 | tr -d '\r')
            log_info "  └─ Memory usage: $used_memory"
        else
            log_error "Redis not responding"
            component_status["redis"]="critical"
            return 1
        fi
    else
        # Docker-based check
        if docker exec alchemist-redis redis-cli ping &> /dev/null; then
            log_success "Redis is responding (Docker check)"
            component_status["redis"]="ok"
        else
            log_error "Redis health check failed"
            component_status["redis"]="critical"
            return 1
        fi
    fi
    
    return 0
}

# Check Ollama
check_ollama() {
    log_info "Checking Ollama AI service..."
    
    if curl -sf "http://localhost:11434/api/tags" &> /dev/null; then
        log_success "Ollama service is responding"
        component_status["ollama"]="ok"
        
        # Check loaded models
        local models=$(curl -sf "http://localhost:11434/api/tags" | jq -r '.models[].name' 2>/dev/null || echo "none")
        if [[ "$models" != "none" ]] && [[ -n "$models" ]]; then
            log_info "  └─ Loaded models: $(echo $models | tr '\n' ', ' | sed 's/, $//')"
        else
            log_warning "  └─ No models loaded"
            component_status["ollama_models"]="warning"
        fi
    else
        log_error "Ollama service not responding"
        component_status["ollama"]="critical"
        return 1
    fi
    
    return 0
}

# Check metrics
check_metrics() {
    log_info "Checking metrics endpoint..."
    
    if curl -sf "$METRICS_ENDPOINT" | grep -q "alchemist_"; then
        log_success "Metrics endpoint is healthy"
        component_status["metrics"]="ok"
        
        # Sample some key metrics
        local up_metric=$(curl -sf "$METRICS_ENDPOINT" | grep "^up " | awk '{print $2}')
        if [[ "$up_metric" == "1" ]]; then
            log_success "  └─ Service status: UP"
        else
            log_warning "  └─ Service status: DOWN"
            component_status["metrics_up"]="warning"
        fi
    else
        log_error "Metrics endpoint not responding"
        component_status["metrics"]="critical"
        return 1
    fi
    
    return 0
}

# Check disk space
check_disk_space() {
    log_info "Checking disk space..."
    
    local threshold=80
    local warnings=0
    
    while IFS= read -r line; do
        local usage=$(echo "$line" | awk '{print $5}' | sed 's/%//')
        local mount=$(echo "$line" | awk '{print $6}')
        
        if [[ $usage -gt $threshold ]]; then
            log_warning "Disk usage on $mount: ${usage}%"
            component_status["disk_$mount"]="warning"
            warnings=$((warnings + 1))
        fi
    done < <(df -h | grep -E "^/dev/")
    
    if [[ $warnings -eq 0 ]]; then
        log_success "Disk space usage is healthy"
        component_status["disk"]="ok"
    fi
    
    return 0
}

# Generate summary
generate_summary() {
    echo ""
    log_info "Health Check Summary:"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    local critical_count=0
    local warning_count=0
    local ok_count=0
    
    for component in "${!component_status[@]}"; do
        case "${component_status[$component]}" in
            "ok")
                echo -e "${GREEN}[✓]${NC} $component"
                ok_count=$((ok_count + 1))
                ;;
            "warning")
                echo -e "${YELLOW}[!]${NC} $component"
                warning_count=$((warning_count + 1))
                ;;
            "critical")
                echo -e "${RED}[✗]${NC} $component"
                critical_count=$((critical_count + 1))
                ;;
        esac
    done
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Total: OK=$ok_count, Warnings=$warning_count, Critical=$critical_count"
    
    # Determine exit code
    if [[ $critical_count -gt 0 ]]; then
        return $EXIT_CRITICAL
    elif [[ $warning_count -gt 0 ]]; then
        return $EXIT_WARNING
    else
        return $EXIT_SUCCESS
    fi
}

# Main health check flow
main() {
    log_info "Starting comprehensive health check..."
    echo ""
    
    # Run all checks (continue even if some fail)
    check_alchemist || true
    check_nats || true
    check_postgres || true
    check_redis || true
    check_ollama || true
    check_metrics || true
    check_disk_space || true
    
    # Generate and display summary
    generate_summary
    local exit_code=$?
    
    echo ""
    case $exit_code in
        $EXIT_SUCCESS)
            log_success "All systems operational"
            ;;
        $EXIT_WARNING)
            log_warning "System operational with warnings"
            ;;
        $EXIT_CRITICAL)
            log_error "Critical issues detected"
            ;;
    esac
    
    exit $exit_code
}

# Run main function
main "$@"
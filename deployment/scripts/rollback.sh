#!/bin/bash
# Alchemist Rollback Script
# This script handles rolling back to a previous deployment

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
DEPLOYMENT_DIR="$PROJECT_ROOT/deployment"
ENVIRONMENT="${ENVIRONMENT:-production}"
BACKUP_DIR="/var/backups/alchemist"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

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

# List available backups
list_backups() {
    log_info "Available backups:"
    
    if [[ ! -d "$BACKUP_DIR" ]]; then
        log_error "Backup directory not found: $BACKUP_DIR"
        exit 1
    fi
    
    local backups=($(ls -1 "$BACKUP_DIR" | grep -E "alchemist_backup_[0-9]{8}_[0-9]{6}" | sort -r | uniq))
    
    if [[ ${#backups[@]} -eq 0 ]]; then
        log_error "No backups found"
        exit 1
    fi
    
    for i in "${!backups[@]}"; do
        echo "$((i+1)). ${backups[$i]}"
    done
    
    echo ""
    read -p "Select backup number (or 'q' to quit): " selection
    
    if [[ "$selection" == "q" ]]; then
        exit 0
    fi
    
    if [[ ! "$selection" =~ ^[0-9]+$ ]] || [[ $selection -lt 1 ]] || [[ $selection -gt ${#backups[@]} ]]; then
        log_error "Invalid selection"
        exit 1
    fi
    
    SELECTED_BACKUP="${backups[$((selection-1))]}"
}

# Restore database
restore_database() {
    local backup_file="$1"
    
    log_info "Restoring database from: $backup_file"
    
    # Stop the application to prevent new writes
    docker-compose -f "$DEPLOYMENT_DIR/docker/docker-compose.$ENVIRONMENT.yml" stop alchemist
    
    # Restore database
    gunzip -c "$backup_file" | docker-compose -f "$DEPLOYMENT_DIR/docker/docker-compose.$ENVIRONMENT.yml" \
        exec -T postgres psql -U alchemist -d alchemist
    
    log_success "Database restored successfully"
}

# Restore volumes
restore_volumes() {
    local backup_file="$1"
    
    log_info "Restoring volumes from: $backup_file"
    
    # Stop all services
    docker-compose -f "$DEPLOYMENT_DIR/docker/docker-compose.$ENVIRONMENT.yml" down
    
    # Restore volumes
    docker run --rm \
        -v alchemist_data:/data \
        -v "$BACKUP_DIR":/backup \
        alpine sh -c "rm -rf /data/* && tar xzf /backup/$(basename "$backup_file") -C /"
    
    log_success "Volumes restored successfully"
}

# Rollback Docker image
rollback_image() {
    log_info "Rolling back to previous Docker image..."
    
    # Get previous image tag
    local previous_tag=$(docker images alchemist --format "{{.Tag}}" | grep -v latest | head -2 | tail -1)
    
    if [[ -z "$previous_tag" ]]; then
        log_error "No previous image found"
        exit 1
    fi
    
    log_info "Rolling back to image: alchemist:$previous_tag"
    
    # Update docker-compose to use previous image
    export VERSION="$previous_tag"
    
    # Restart services with previous image
    docker-compose -f "$DEPLOYMENT_DIR/docker/docker-compose.$ENVIRONMENT.yml" up -d
    
    log_success "Rolled back to image: alchemist:$previous_tag"
}

# Main rollback flow
main() {
    log_warning "Starting Alchemist rollback for environment: $ENVIRONMENT"
    log_warning "This will restore a previous backup and may cause data loss!"
    
    read -p "Are you sure you want to continue? (yes/no): " confirm
    if [[ "$confirm" != "yes" ]]; then
        log_info "Rollback cancelled"
        exit 0
    fi
    
    # Select backup
    list_backups
    
    log_info "Selected backup: $SELECTED_BACKUP"
    
    # Find backup files
    local db_backup="$BACKUP_DIR/${SELECTED_BACKUP}_postgres.sql.gz"
    local volume_backup="$BACKUP_DIR/${SELECTED_BACKUP}_volumes.tar.gz"
    
    # Perform rollback
    if [[ -f "$db_backup" ]]; then
        restore_database "$db_backup"
    else
        log_warning "Database backup not found, skipping..."
    fi
    
    if [[ -f "$volume_backup" ]]; then
        restore_volumes "$volume_backup"
    else
        log_warning "Volume backup not found, skipping..."
    fi
    
    # Rollback Docker image
    rollback_image
    
    # Health check
    log_info "Waiting for services to be healthy..."
    sleep 30
    
    if curl -sf "http://localhost:8081/health/ready" &> /dev/null; then
        log_success "Rollback completed successfully!"
    else
        log_error "Service health check failed after rollback"
        exit 1
    fi
}

# Run main function
main "$@"
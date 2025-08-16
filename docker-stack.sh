#!/bin/bash

# Rust Commerce Docker Compose Management Script
# Usage: ./docker-stack.sh <action> [service]
# action: up|down|restart|logs|status|build
# service: (optional) specific service name

set -e

COMPOSE_FILE="docker-compose.yml"
PROJECT_NAME="rust-commerce"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to display help
show_help() {
    echo "Rust Commerce Docker Stack Manager"
    echo ""
    echo "Usage: $0 <action> [service]"
    echo ""
    echo "Actions:"
    echo "  up        Start all services (or specific service)"
    echo "  down      Stop all services (or specific service)"
    echo "  restart   Restart all services (or specific service)"
    echo "  logs      Show logs for all services (or specific service)"
    echo "  status    Show status of all services"
    echo "  build     Build all images (or specific service)"
    echo "  clean     Stop and remove all containers, networks, and volumes"
    echo ""
    echo "Services:"
    echo "  mongodb, nats, catalog-service, inventory-service, orders-service, price-service"
    echo ""
    echo "Examples:"
    echo "  $0 up                    # Start all services"
    echo "  $0 up catalog-service    # Start only catalog service (and dependencies)"
    echo "  $0 logs                  # Show logs for all services"
    echo "  $0 logs nats            # Show logs for NATS service only"
    echo "  $0 status               # Show status of all services"
    echo "  $0 down                 # Stop all services"
    echo ""
}

# Function to check if Docker is running
check_docker() {
    if ! docker info > /dev/null 2>&1; then
        print_error "Docker is not running. Please start Docker first."
        exit 1
    fi
}

# Function to start services
start_services() {
    local service=$1
    
    if [ -n "$service" ]; then
        print_status "Starting service: $service"
        docker-compose -f "$COMPOSE_FILE" up -d "$service"
    else
        print_status "Starting all Rust Commerce services..."
        docker-compose -f "$COMPOSE_FILE" up -d
    fi
    
    if [ $? -eq 0 ]; then
        print_success "Services started successfully!"
        show_status
    else
        print_error "Failed to start services"
        exit 1
    fi
}

# Function to stop services
stop_services() {
    local service=$1
    
    if [ -n "$service" ]; then
        print_status "Stopping service: $service"
        docker-compose -f "$COMPOSE_FILE" stop "$service"
    else
        print_status "Stopping all Rust Commerce services..."
        docker-compose -f "$COMPOSE_FILE" down
    fi
    
    print_success "Services stopped successfully!"
}

# Function to restart services
restart_services() {
    local service=$1
    
    if [ -n "$service" ]; then
        print_status "Restarting service: $service"
        docker-compose -f "$COMPOSE_FILE" restart "$service"
    else
        print_status "Restarting all Rust Commerce services..."
        docker-compose -f "$COMPOSE_FILE" restart
    fi
    
    print_success "Services restarted successfully!"
}

# Function to show logs
show_logs() {
    local service=$1
    
    if [ -n "$service" ]; then
        print_status "Showing logs for service: $service"
        docker-compose -f "$COMPOSE_FILE" logs -f "$service"
    else
        print_status "Showing logs for all services (press Ctrl+C to exit)..."
        docker-compose -f "$COMPOSE_FILE" logs -f
    fi
}

# Function to show status
show_status() {
    print_status "Current status of Rust Commerce services:"
    echo ""
    docker-compose -f "$COMPOSE_FILE" ps
    echo ""
    
    # Show network information
    print_status "Network information:"
    docker network ls | grep rust-commerce || echo "rust-commerce network not found"
}

# Function to build images
build_images() {
    local service=$1
    
    if [ -n "$service" ]; then
        print_status "Building image for service: $service"
        docker-compose -f "$COMPOSE_FILE" build "$service"
    else
        print_status "Building all Rust Commerce service images..."
        docker-compose -f "$COMPOSE_FILE" build
    fi
    
    print_success "Build completed successfully!"
}

# Function to clean everything
clean_all() {
    print_warning "This will stop and remove all containers, networks, and volumes."
    read -p "Are you sure? (y/N): " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_status "Cleaning up all Rust Commerce resources..."
        docker-compose -f "$COMPOSE_FILE" down -v --remove-orphans
        docker system prune -f
        print_success "Cleanup completed successfully!"
    else
        print_status "Cleanup cancelled."
    fi
}

# Main script logic
check_docker

if [ $# -eq 0 ]; then
    show_help
    exit 0
fi

ACTION=$1
SERVICE=$2

case $ACTION in
    up|start)
        start_services "$SERVICE"
        ;;
    down|stop)
        stop_services "$SERVICE"
        ;;
    restart)
        restart_services "$SERVICE"
        ;;
    logs)
        show_logs "$SERVICE"
        ;;
    status|ps)
        show_status
        ;;
    build)
        build_images "$SERVICE"
        ;;
    clean)
        clean_all
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        print_error "Unknown action: $ACTION"
        echo ""
        show_help
        exit 1
        ;;
esac

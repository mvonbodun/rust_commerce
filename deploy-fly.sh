#!/bin/bash

# Rust Commerce Fly.io Deployment Script
# This script helps deploy all microservices to Fly.io

set -e

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

# Services to deploy
SERVICES=("catalog" "inventory" "orders" "price")
ROOT_DIR="/Users/vb/Software/rust/rust_commerce"

# Function to check if flyctl is installed
check_flyctl() {
    if ! command -v flyctl &> /dev/null; then
        print_error "flyctl is not installed. Please install it first:"
        echo "curl -L https://fly.io/install.sh | sh"
        exit 1
    fi
}

# Function to check if user is logged in
check_auth() {
    if ! flyctl auth whoami &> /dev/null; then
        print_error "You are not logged in to Fly.io. Please run:"
        echo "flyctl auth login"
        exit 1
    fi
}

# Function to deploy a single service
deploy_service() {
    local service=$1
    local service_dir="${ROOT_DIR}/${service}"
    
    print_status "Deploying ${service} service..."
    
    # Check if directory exists
    if [ ! -d "$service_dir" ]; then
        print_error "Service directory not found: $service_dir"
        return 1
    fi
    
    # Check if fly.toml exists
    if [ ! -f "$service_dir/fly.toml" ]; then
        print_error "fly.toml not found in $service_dir"
        return 1
    fi
    
    # Change to service directory
    cd "$service_dir"
    
    # Check if app exists, if not create it
    app_name="rust-commerce-${service}"
    if ! flyctl apps list | grep -q "$app_name"; then
        print_status "Creating new app: $app_name"
        flyctl apps create "$app_name" --org personal
    fi
    
    # Set secrets for the service
    print_status "Setting secrets for $service..."
    
    # Updated URLs for Fly.io deployment
    flyctl secrets set \
        MONGODB_URL="mongodb+srv://your_user:your_password@your_cluster.mongodb.net/${service}?retryWrites=true&w=majority" \
        NATS_URL="nats://rust-commerce-nats.flycast:4222" \
        --app "$app_name"
    
    # Deploy the service
    print_status "Deploying $service to Fly.io..."
    flyctl deploy --app "$app_name"
    
    if [ $? -eq 0 ]; then
        print_success "$service deployed successfully!"
    else
        print_error "Failed to deploy $service"
        return 1
    fi
    
    # Return to root directory
    cd "$ROOT_DIR"
}

# Function to show service status
show_status() {
    print_status "Checking status of all services..."
    
    for service in "${SERVICES[@]}"; do
        app_name="rust-commerce-${service}"
        echo ""
        print_status "Status for $service:"
        flyctl status --app "$app_name" || print_warning "App $app_name not found"
    done
}

# Function to show logs
show_logs() {
    local service=$1
    
    if [ -z "$service" ]; then
        print_error "Please specify a service: catalog, inventory, orders, or price"
        return 1
    fi
    
    app_name="rust-commerce-${service}"
    print_status "Showing logs for $service..."
    flyctl logs --app "$app_name"
}

# Function to scale services
scale_service() {
    local service=$1
    local count=$2
    
    if [ -z "$service" ] || [ -z "$count" ]; then
        print_error "Usage: scale <service> <count>"
        return 1
    fi
    
    app_name="rust-commerce-${service}"
    print_status "Scaling $service to $count instances..."
    flyctl scale count "$count" --app "$app_name"
}

# Function to display help
show_help() {
    echo "Rust Commerce Fly.io Deployment Manager"
    echo ""
    echo "Usage: $0 <command> [options]"
    echo ""
    echo "Commands:"
    echo "  deploy [service]     Deploy all services or specific service"
    echo "  status               Show status of all services"
    echo "  logs <service>       Show logs for specific service"
    echo "  scale <service> <n>  Scale service to n instances"
    echo "  destroy [service]    Destroy all apps or specific service"
    echo ""
    echo "Services: catalog, inventory, orders, price"
    echo ""
    echo "Examples:"
    echo "  $0 deploy                    # Deploy all services"
    echo "  $0 deploy catalog           # Deploy only catalog service"
    echo "  $0 status                   # Show status of all services"
    echo "  $0 logs catalog             # Show logs for catalog service"
    echo "  $0 scale catalog 2          # Scale catalog to 2 instances"
    echo ""
    echo "Prerequisites:"
    echo "  1. Install flyctl: curl -L https://fly.io/install.sh | sh"
    echo "  2. Login: flyctl auth login"
    echo "  3. Update MongoDB and NATS URLs in this script"
}

# Function to destroy services
destroy_service() {
    local service=$1
    
    if [ -n "$service" ]; then
        app_name="rust-commerce-${service}"
        print_warning "This will destroy the $service app permanently!"
        read -p "Are you sure? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            flyctl apps destroy "$app_name" --yes
            print_success "$service app destroyed"
        fi
    else
        print_warning "This will destroy ALL Rust Commerce apps permanently!"
        read -p "Are you sure? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            for service in "${SERVICES[@]}"; do
                app_name="rust-commerce-${service}"
                flyctl apps destroy "$app_name" --yes || true
            done
            print_success "All apps destroyed"
        fi
    fi
}

# Main script logic
check_flyctl
check_auth

if [ $# -eq 0 ]; then
    show_help
    exit 0
fi

COMMAND=$1
SERVICE=$2

case $COMMAND in
    deploy)
        if [ -n "$SERVICE" ]; then
            # Deploy specific service
            if [[ " ${SERVICES[@]} " =~ " ${SERVICE} " ]]; then
                deploy_service "$SERVICE"
            else
                print_error "Invalid service: $SERVICE"
                print_error "Valid services: ${SERVICES[*]}"
                exit 1
            fi
        else
            # Deploy all services
            print_status "Deploying all Rust Commerce services..."
            for service in "${SERVICES[@]}"; do
                deploy_service "$service"
                echo ""
            done
            print_success "All services deployed!"
        fi
        ;;
    status)
        show_status
        ;;
    logs)
        show_logs "$SERVICE"
        ;;
    scale)
        scale_service "$SERVICE" "$3"
        ;;
    destroy)
        destroy_service "$SERVICE"
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        print_error "Unknown command: $COMMAND"
        echo ""
        show_help
        exit 1
        ;;
esac

#!/bin/bash

# NATS Server Fly.io Deployment Script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if flyctl is installed
check_flyctl() {
    if ! command -v flyctl &> /dev/null; then
        print_error "flyctl is not installed. Please install it first:"
        echo "curl -L https://fly.io/install.sh | sh"
        exit 1
    fi
}

# Check if user is logged in
check_auth() {
    if ! flyctl auth whoami &> /dev/null; then
        print_error "You are not logged in to Fly.io. Please run:"
        echo "flyctl auth login"
        exit 1
    fi
}

deploy_nats() {
    print_status "Deploying NATS server to Fly.io..."
    
    # Create app if it doesn't exist
    if ! flyctl apps list | grep -q "rust-commerce-nats"; then
        print_status "Creating NATS app..."
        flyctl apps create rust-commerce-nats --org personal
    fi
    
    # Create volume for JetStream persistence
    print_status "Creating volume for NATS data..."
    flyctl volumes create nats_data --size 3 --app rust-commerce-nats --region sjc || true
    
    # Deploy the NATS server
    print_status "Deploying NATS server..."
    flyctl deploy --app rust-commerce-nats
    
    if [ $? -eq 0 ]; then
        print_success "NATS server deployed successfully!"
        print_status "NATS connection URL: nats://rust-commerce-nats.flycast:4222"
        print_status "HTTP monitoring: https://rust-commerce-nats.fly.dev:8222"
        
        # Show status
        flyctl status --app rust-commerce-nats
    else
        print_error "Failed to deploy NATS server"
        exit 1
    fi
}

show_status() {
    print_status "NATS server status:"
    flyctl status --app rust-commerce-nats
}

show_logs() {
    print_status "NATS server logs:"
    flyctl logs --app rust-commerce-nats
}

destroy_nats() {
    print_error "This will destroy the NATS server permanently!"
    read -p "Are you sure? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        flyctl apps destroy rust-commerce-nats --yes
        print_success "NATS server destroyed"
    fi
}

show_help() {
    echo "NATS Server Fly.io Deployment Manager"
    echo ""
    echo "Usage: $0 <command>"
    echo ""
    echo "Commands:"
    echo "  deploy    Deploy NATS server to Fly.io"
    echo "  status    Show NATS server status"
    echo "  logs      Show NATS server logs"
    echo "  destroy   Destroy NATS server"
    echo ""
    echo "After deployment, use this connection URL in your microservices:"
    echo "  NATS_URL=nats://rust-commerce-nats.flycast:4222"
}

# Main script
check_flyctl
check_auth

COMMAND=$1

case $COMMAND in
    deploy)
        deploy_nats
        ;;
    status)
        show_status
        ;;
    logs)
        show_logs
        ;;
    destroy)
        destroy_nats
        ;;
    help|--help|-h|"")
        show_help
        ;;
    *)
        print_error "Unknown command: $COMMAND"
        show_help
        exit 1
        ;;
esac

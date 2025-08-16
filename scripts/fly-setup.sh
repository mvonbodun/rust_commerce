#!/bin/bash

# Fly.io App Setup Script for Rust Commerce Microservices
# Usage: ./fly-setup.sh <service> <env> [options]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

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

print_header() {
    echo -e "${CYAN}================================${NC}"
    echo -e "${CYAN}$1${NC}"
    echo -e "${CYAN}================================${NC}"
}

# Check if flyctl is installed
check_flyctl() {
    if ! command -v flyctl &> /dev/null; then
        print_error "flyctl is not installed. Please install it first:"
        echo "curl -L https://fly.io/install.sh | sh"
        exit 1
    fi
    
    # Check if logged in
    if ! flyctl auth whoami &> /dev/null; then
        print_error "You are not logged in to Fly.io. Please run:"
        echo "flyctl auth login"
        exit 1
    fi
    
    print_success "flyctl is installed and you are logged in"
}

# Load environment variables from .env file
load_env_file() {
    local service=$1
    local env_type=$2
    local env_file=""
    
    # Determine the env file path
    case $env_type in
        local)
            env_file="../$service/.env.local"
            ;;
        staging)
            env_file="../$service/.env.staging"
            ;;
        production)
            env_file="../$service/.env.production"
            ;;
        *)
            env_file="../$service/.env"
            ;;
    esac
    
    if [ ! -f "$env_file" ]; then
        print_error "Environment file not found: $env_file"
        exit 1
    fi
    
    print_status "Loading environment from: $env_file"
    
    # Source the env file and extract MONGODB_URL and NATS_URL
    while IFS= read -r line; do
        # Skip comments and empty lines
        [[ $line =~ ^[[:space:]]*# ]] && continue
        [[ -z "${line// }" ]] && continue
        
        # Extract MONGODB_URL and NATS_URL
        if [[ $line =~ ^MONGODB_URL= ]]; then
            MONGODB_URL=$(echo "$line" | cut -d'=' -f2- | sed 's/^"\|"$//g')
        elif [[ $line =~ ^NATS_URL= ]]; then
            NATS_URL=$(echo "$line" | cut -d'=' -f2- | sed 's/^"\|"$//g')
        fi
    done < "$env_file"
    
    if [ -z "$MONGODB_URL" ] || [ -z "$NATS_URL" ]; then
        print_error "Required environment variables not found in $env_file"
        print_error "Make sure MONGODB_URL and NATS_URL are defined"
        exit 1
    fi
    
    print_success "Environment variables loaded successfully"
}

# Check if app already exists
check_app_exists() {
    local app_name=$1
    
    if flyctl apps list | grep -q "^$app_name"; then
        return 0  # App exists
    else
        return 1  # App doesn't exist
    fi
}

# Create Fly.io app
create_app() {
    local app_name=$1
    local service=$2
    
    print_header "Creating Fly.io App: $app_name"
    
    if check_app_exists "$app_name"; then
        print_warning "App '$app_name' already exists"
        read -p "Do you want to continue with existing app? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_error "Aborted by user"
            exit 1
        fi
    else
        print_status "Creating new app: $app_name"
        
        # Change to service directory
        cd "../$service"
        
        # Create the app without deploying
        flyctl apps create "$app_name"
        
        print_success "App '$app_name' created successfully"
        
        # Return to scripts directory
        cd "../scripts"
    fi
}

# Set secrets
set_secrets() {
    local app_name=$1
    
    print_header "Setting Secrets for: $app_name"
    
    print_status "Setting MONGODB_URL..."
    if flyctl secrets set --app "$app_name" MONGODB_URL="$MONGODB_URL"; then
        print_success "MONGODB_URL set successfully"
    else
        print_error "Failed to set MONGODB_URL"
        exit 1
    fi
    
    print_status "Setting NATS_URL..."
    if flyctl secrets set --app "$app_name" NATS_URL="$NATS_URL"; then
        print_success "NATS_URL set successfully"
    else
        print_error "Failed to set NATS_URL"
        exit 1
    fi
    
    print_success "All secrets set successfully"
}

# Deploy the app
deploy_app() {
    local app_name=$1
    local service=$2
    
    print_header "Deploying App: $app_name"
    
    # Change to service directory
    cd "../$service"
    
    print_status "Starting deployment..."
    if flyctl deploy --app "$app_name"; then
        print_success "Deployment completed successfully"
    else
        print_error "Deployment failed"
        exit 1
    fi
    
    # Return to scripts directory
    cd "../scripts"
}

# Show app status
show_status() {
    local app_name=$1
    
    print_header "App Status: $app_name"
    
    print_status "App information:"
    flyctl apps show "$app_name"
    
    echo ""
    print_status "Machine status:"
    flyctl machines list --app "$app_name"
    
    echo ""
    print_status "Recent logs:"
    flyctl logs --app "$app_name" --lines 20
}

# Show help
show_help() {
    echo "Fly.io App Setup Script for Rust Commerce Microservices"
    echo ""
    echo "Usage: $0 <service> <env> [options]"
    echo ""
    echo "Parameters:"
    echo "  service     Service name (catalog, inventory, orders, price, nats)"
    echo "  env         Environment type (local, staging, production)"
    echo ""
    echo "Options:"
    echo "  --skip-create    Skip app creation (use existing app)"
    echo "  --skip-secrets   Skip setting secrets"
    echo "  --skip-deploy    Skip deployment"
    echo "  --show-status    Show app status after deployment"
    echo "  --dry-run        Show what would be done without executing"
    echo ""
    echo "Examples:"
    echo "  $0 catalog production"
    echo "  $0 inventory staging --skip-create"
    echo "  $0 price local --show-status"
    echo "  $0 orders production --dry-run"
    echo ""
    echo "Available services:"
    echo "  catalog     - Product catalog service"
    echo "  inventory   - Inventory management service"
    echo "  orders      - Order processing service"
    echo "  price       - Pricing service"
    echo "  nats        - NATS messaging server"
    echo ""
    echo "Environment files used:"
    echo "  local       - .env.local"
    echo "  staging     - .env.staging"
    echo "  production  - .env.production"
}

# Validate service name
validate_service() {
    local service=$1
    local valid_services=("catalog" "inventory" "orders" "price" "nats")
    
    for valid_service in "${valid_services[@]}"; do
        if [ "$service" == "$valid_service" ]; then
            return 0
        fi
    done
    
    print_error "Invalid service: $service"
    echo "Valid services: ${valid_services[*]}"
    exit 1
}

# Main execution
main() {
    local service=$1
    local env_type=$2
    local app_name="rust-commerce-$service"
    
    # Parse options
    local skip_create=false
    local skip_secrets=false
    local skip_deploy=false
    local show_status_flag=false
    local dry_run=false
    
    # Process remaining arguments
    shift 2 2>/dev/null || true
    while [[ $# -gt 0 ]]; do
        case $1 in
            --skip-create)
                skip_create=true
                ;;
            --skip-secrets)
                skip_secrets=true
                ;;
            --skip-deploy)
                skip_deploy=true
                ;;
            --show-status)
                show_status_flag=true
                ;;
            --dry-run)
                dry_run=true
                ;;
            *)
                print_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
        shift
    done
    
    # Validate inputs
    if [ -z "$service" ] || [ -z "$env_type" ]; then
        print_error "Service and environment type are required"
        show_help
        exit 1
    fi
    
    validate_service "$service"
    
    print_header "Fly.io Setup for $service ($env_type)"
    
    if [ "$dry_run" == true ]; then
        print_warning "DRY RUN MODE - No changes will be made"
        echo ""
        echo "Would execute:"
        echo "  1. Load environment from: ../$service/.env.$env_type"
        echo "  2. Create app: $app_name (skip: $skip_create)"
        echo "  3. Set secrets for MONGODB_URL and NATS_URL (skip: $skip_secrets)"
        echo "  4. Deploy app from ../$service directory (skip: $skip_deploy)"
        if [ "$show_status_flag" == true ]; then
            echo "  5. Show app status"
        fi
        exit 0
    fi
    
    # Execute setup steps
    check_flyctl
    load_env_file "$service" "$env_type"
    
    if [ "$skip_create" != true ]; then
        create_app "$app_name" "$service"
    else
        print_warning "Skipping app creation"
    fi
    
    if [ "$skip_secrets" != true ]; then
        set_secrets "$app_name"
    else
        print_warning "Skipping secrets setup"
    fi
    
    if [ "$skip_deploy" != true ]; then
        deploy_app "$app_name" "$service"
    else
        print_warning "Skipping deployment"
    fi
    
    if [ "$show_status_flag" == true ]; then
        show_status "$app_name"
    fi
    
    print_success "Setup completed for $service!"
    echo ""
    echo "Next steps:"
    echo "  - Check app status: flyctl status --app $app_name"
    echo "  - View logs: flyctl logs --app $app_name"
    echo "  - Scale if needed: flyctl scale count 2 --app $app_name"
    echo "  - Monitor: flyctl dashboard --app $app_name"
}

# Handle help and no arguments
if [ $# -eq 0 ] || [ "$1" == "help" ] || [ "$1" == "--help" ] || [ "$1" == "-h" ]; then
    show_help
    exit 0
fi

# Run main function with all arguments
main "$@"

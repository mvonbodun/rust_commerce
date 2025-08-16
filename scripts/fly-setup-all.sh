#!/bin/bash

# Batch Fly.io Setup Script for All Rust Commerce Services
# Usage: ./fly-setup-all.sh <env> [options]

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

show_help() {
    echo "Batch Fly.io Setup Script for All Rust Commerce Services"
    echo ""
    echo "Usage: $0 <env> [options]"
    echo ""
    echo "Parameters:"
    echo "  env         Environment type (local, staging, production)"
    echo ""
    echo "Options:"
    echo "  --services \"list\"  Specify services to deploy (default: catalog inventory orders price)"
    echo "  --skip-create      Skip app creation for all services"
    echo "  --skip-secrets     Skip setting secrets for all services"
    echo "  --skip-deploy      Skip deployment for all services"
    echo "  --parallel         Deploy services in parallel (experimental)"
    echo "  --dry-run          Show what would be done without executing"
    echo ""
    echo "Examples:"
    echo "  $0 production"
    echo "  $0 staging --services \"catalog orders\""
    echo "  $0 production --skip-create"
    echo "  $0 local --dry-run"
    echo ""
    echo "Service deployment order:"
    echo "  1. nats      - Message broker (if included)"
    echo "  2. catalog   - Product catalog service"
    echo "  3. inventory - Inventory management service"
    echo "  4. price     - Pricing service"
    echo "  5. orders    - Order processing service"
}

# Deploy services in sequence
deploy_sequential() {
    local env_type=$1
    local services=($2)
    local options="$3"
    
    print_header "Sequential Deployment"
    
    local failed_services=()
    local successful_services=()
    
    for service in "${services[@]}"; do
        print_status "Deploying $service..."
        
        if ./fly-setup.sh "$service" "$env_type" $options; then
            successful_services+=("$service")
            print_success "$service deployed successfully"
        else
            failed_services+=("$service")
            print_error "$service deployment failed"
        fi
        
        echo ""
        sleep 2  # Brief pause between deployments
    done
    
    # Summary
    print_header "Deployment Summary"
    
    if [ ${#successful_services[@]} -gt 0 ]; then
        print_success "Successfully deployed: ${successful_services[*]}"
    fi
    
    if [ ${#failed_services[@]} -gt 0 ]; then
        print_error "Failed deployments: ${failed_services[*]}"
        return 1
    fi
    
    return 0
}

# Deploy services in parallel (experimental)
deploy_parallel() {
    local env_type=$1
    local services=($2)
    local options="$3"
    
    print_header "Parallel Deployment (Experimental)"
    print_warning "This is experimental and may cause issues with resource limits"
    
    local pids=()
    local results_dir="/tmp/fly-deploy-results-$$"
    mkdir -p "$results_dir"
    
    # Start all deployments
    for service in "${services[@]}"; do
        print_status "Starting deployment for $service..."
        (
            if ./fly-setup.sh "$service" "$env_type" $options &> "$results_dir/$service.log"; then
                echo "SUCCESS" > "$results_dir/$service.result"
            else
                echo "FAILED" > "$results_dir/$service.result"
            fi
        ) &
        pids+=($!)
    done
    
    # Wait for all deployments
    print_status "Waiting for all deployments to complete..."
    for pid in "${pids[@]}"; do
        wait $pid
    done
    
    # Check results
    local failed_services=()
    local successful_services=()
    
    for service in "${services[@]}"; do
        if [ -f "$results_dir/$service.result" ]; then
            result=$(cat "$results_dir/$service.result")
            if [ "$result" == "SUCCESS" ]; then
                successful_services+=("$service")
            else
                failed_services+=("$service")
            fi
        else
            failed_services+=("$service")
        fi
    done
    
    # Show logs for failed services
    for service in "${failed_services[@]}"; do
        print_error "Logs for failed service: $service"
        cat "$results_dir/$service.log" || echo "No logs available"
        echo ""
    done
    
    # Cleanup
    rm -rf "$results_dir"
    
    # Summary
    print_header "Parallel Deployment Summary"
    
    if [ ${#successful_services[@]} -gt 0 ]; then
        print_success "Successfully deployed: ${successful_services[*]}"
    fi
    
    if [ ${#failed_services[@]} -gt 0 ]; then
        print_error "Failed deployments: ${failed_services[*]}"
        return 1
    fi
    
    return 0
}

# Main execution
main() {
    local env_type=$1
    
    # Default services in deployment order
    local default_services="catalog inventory price orders"
    local services="$default_services"
    local parallel=false
    local dry_run=false
    local additional_options=""
    
    # Parse options
    shift 1 2>/dev/null || true
    while [[ $# -gt 0 ]]; do
        case $1 in
            --services)
                services="$2"
                shift 2
                ;;
            --skip-create)
                additional_options="$additional_options --skip-create"
                shift
                ;;
            --skip-secrets)
                additional_options="$additional_options --skip-secrets"
                shift
                ;;
            --skip-deploy)
                additional_options="$additional_options --skip-deploy"
                shift
                ;;
            --parallel)
                parallel=true
                shift
                ;;
            --dry-run)
                dry_run=true
                additional_options="$additional_options --dry-run"
                shift
                ;;
            *)
                print_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    # Validate environment type
    if [ -z "$env_type" ]; then
        print_error "Environment type is required"
        show_help
        exit 1
    fi
    
    # Convert services string to array
    local services_array=($services)
    
    print_header "Batch Fly.io Setup for Rust Commerce"
    echo "Environment: $env_type"
    echo "Services: ${services_array[*]}"
    echo "Parallel: $parallel"
    echo "Additional options: $additional_options"
    echo ""
    
    if [ "$dry_run" == true ]; then
        print_warning "DRY RUN MODE - No changes will be made"
        echo ""
        echo "Would deploy the following services:"
        for service in "${services_array[@]}"; do
            echo "  - $service (rust-commerce-$service)"
        done
        exit 0
    fi
    
    # Check if fly-setup.sh exists
    if [ ! -f "./fly-setup.sh" ]; then
        print_error "fly-setup.sh not found in current directory"
        print_error "Please run this script from the scripts directory"
        exit 1
    fi
    
    # Execute deployment
    if [ "$parallel" == true ]; then
        deploy_parallel "$env_type" "${services_array[*]}" "$additional_options"
    else
        deploy_sequential "$env_type" "${services_array[*]}" "$additional_options"
    fi
    
    if [ $? -eq 0 ]; then
        print_success "All services deployed successfully!"
        echo ""
        echo "Next steps:"
        echo "  - Check all apps: flyctl apps list | grep rust-commerce"
        echo "  - Monitor deployments: ./fly-monitor.sh"
        echo "  - View logs: flyctl logs --app rust-commerce-<service>"
    else
        print_error "Some deployments failed. Check the logs above."
        exit 1
    fi
}

# Handle help and no arguments
if [ $# -eq 0 ] || [ "$1" == "help" ] || [ "$1" == "--help" ] || [ "$1" == "-h" ]; then
    show_help
    exit 0
fi

# Run main function with all arguments
main "$@"

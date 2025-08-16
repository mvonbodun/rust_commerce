#!/bin/bash

# Fly.io Apps Monitoring Script for Rust Commerce Services
# Usage: ./fly-monitor.sh [options]

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
    
    if ! flyctl auth whoami &> /dev/null; then
        print_error "You are not logged in to Fly.io. Please run:"
        echo "flyctl auth login"
        exit 1
    fi
}

# Get app status
get_app_status() {
    local app_name=$1
    
    if flyctl apps list | grep -q "^$app_name"; then
        local status=$(flyctl status --app "$app_name" --json 2>/dev/null | jq -r '.status' 2>/dev/null || echo "unknown")
        local machines=$(flyctl machines list --app "$app_name" --json 2>/dev/null | jq -r 'length' 2>/dev/null || echo "0")
        local running=$(flyctl machines list --app "$app_name" --json 2>/dev/null | jq -r '[.[] | select(.state == "started")] | length' 2>/dev/null || echo "0")
        
        echo "$status|$machines|$running"
    else
        echo "not_found|0|0"
    fi
}

# Monitor all services
monitor_all() {
    local services=("catalog" "inventory" "orders" "price" "nats")
    local show_logs=$1
    local watch_mode=$2
    
    while true; do
        clear
        print_header "Rust Commerce Services Status"
        echo "$(date)"
        echo ""
        
        printf "%-20s %-15s %-10s %-10s %-15s\n" "SERVICE" "APP STATUS" "MACHINES" "RUNNING" "APP NAME"
        printf "%-20s %-15s %-10s %-10s %-15s\n" "-------" "----------" "--------" "-------" "--------"
        
        local all_healthy=true
        
        for service in "${services[@]}"; do
            local app_name="rust-commerce-$service"
            local status_info=$(get_app_status "$app_name")
            
            IFS='|' read -r status machines running <<< "$status_info"
            
            local status_color=""
            if [ "$status" == "running" ] && [ "$running" -gt 0 ]; then
                status_color="${GREEN}"
            elif [ "$status" == "not_found" ]; then
                status_color="${YELLOW}"
                all_healthy=false
            else
                status_color="${RED}"
                all_healthy=false
            fi
            
            printf "${status_color}%-20s %-15s %-10s %-10s %-15s${NC}\n" \
                "$service" "$status" "$machines" "$running" "$app_name"
        done
        
        echo ""
        if [ "$all_healthy" == true ]; then
            print_success "All services are healthy"
        else
            print_warning "Some services need attention"
        fi
        
        if [ "$show_logs" == true ]; then
            echo ""
            print_header "Recent Logs"
            for service in "${services[@]}"; do
                local app_name="rust-commerce-$service"
                if flyctl apps list | grep -q "^$app_name"; then
                    echo ""
                    print_status "Recent logs for $service:"
                    flyctl logs --app "$app_name" --lines 3 2>/dev/null || echo "No logs available"
                fi
            done
        fi
        
        if [ "$watch_mode" != true ]; then
            break
        fi
        
        echo ""
        echo "Press Ctrl+C to exit watch mode. Refreshing in 30 seconds..."
        sleep 30
    done
}

# Show detailed status for a specific service
show_service_detail() {
    local service=$1
    local app_name="rust-commerce-$service"
    
    print_header "Detailed Status for $service"
    
    if ! flyctl apps list | grep -q "^$app_name"; then
        print_error "App $app_name not found"
        return 1
    fi
    
    print_status "App information:"
    flyctl apps show "$app_name"
    
    echo ""
    print_status "Machine status:"
    flyctl machines list --app "$app_name"
    
    echo ""
    print_status "Recent logs (last 50 lines):"
    flyctl logs --app "$app_name" --lines 50
    
    echo ""
    print_status "App status:"
    flyctl status --app "$app_name"
}

# Show help
show_help() {
    echo "Fly.io Apps Monitoring Script for Rust Commerce Services"
    echo ""
    echo "Usage: $0 [options] [service]"
    echo ""
    echo "Options:"
    echo "  --watch        Monitor in watch mode (refresh every 30s)"
    echo "  --logs         Show recent logs for all services"
    echo "  --detail       Show detailed status (requires service name)"
    echo "  --list         List all rust-commerce apps"
    echo ""
    echo "Examples:"
    echo "  $0                    # Show status once"
    echo "  $0 --watch           # Continuous monitoring"
    echo "  $0 --logs            # Show status with recent logs"
    echo "  $0 --detail catalog  # Detailed status for catalog service"
    echo "  $0 --list            # List all apps"
    echo ""
    echo "Available services:"
    echo "  catalog, inventory, orders, price, nats"
}

# List all rust-commerce apps
list_apps() {
    print_header "All Rust Commerce Apps"
    
    flyctl apps list | grep "rust-commerce" || print_warning "No rust-commerce apps found"
}

# Main execution
main() {
    local watch_mode=false
    local show_logs=false
    local detail_mode=false
    local list_mode=false
    local service=""
    
    # Parse options
    while [[ $# -gt 0 ]]; do
        case $1 in
            --watch)
                watch_mode=true
                shift
                ;;
            --logs)
                show_logs=true
                shift
                ;;
            --detail)
                detail_mode=true
                shift
                ;;
            --list)
                list_mode=true
                shift
                ;;
            catalog|inventory|orders|price|nats)
                service="$1"
                shift
                ;;
            help|--help|-h)
                show_help
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    check_flyctl
    
    if [ "$list_mode" == true ]; then
        list_apps
    elif [ "$detail_mode" == true ]; then
        if [ -z "$service" ]; then
            print_error "Service name required for detailed view"
            show_help
            exit 1
        fi
        show_service_detail "$service"
    else
        monitor_all "$show_logs" "$watch_mode"
    fi
}

# Handle no arguments
if [ $# -eq 0 ]; then
    monitor_all false false
    exit 0
fi

# Run main function with all arguments
main "$@"

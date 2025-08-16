#!/bin/bash

# Script to get Fly.io VM IP addresses for MongoDB whitelisting

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

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
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

# Get IP addresses for a specific app
get_app_ips() {
    local app_name=$1
    
    if [ -z "$app_name" ]; then
        print_error "Please provide an app name"
        return 1
    fi
    
    print_status "Getting IP addresses for app: $app_name"
    echo ""
    
    # Get all IP addresses for the app
    print_status "Public IP addresses:"
    flyctl ips list --app "$app_name" 2>/dev/null || print_warning "No public IPs found or app doesn't exist"
    
    echo ""
    print_status "Machine details with IPs:"
    flyctl machines list --app "$app_name" 2>/dev/null || print_warning "No machines found or app doesn't exist"
    
    echo ""
    print_status "Getting internal IP from machine:"
    # Get machine ID and then its IP
    MACHINE_ID=$(flyctl machines list --app "$app_name" --json 2>/dev/null | jq -r '.[0].id' 2>/dev/null || echo "")
    
    if [ -n "$MACHINE_ID" ] && [ "$MACHINE_ID" != "null" ]; then
        print_status "Machine ID: $MACHINE_ID"
        flyctl machines status "$MACHINE_ID" --app "$app_name" 2>/dev/null || true
    fi
}

# Get all IP addresses for all rust-commerce apps
get_all_ips() {
    local apps=("rust-commerce-catalog" "rust-commerce-inventory" "rust-commerce-orders" "rust-commerce-price" "rust-commerce-nats")
    
    print_status "Getting IP addresses for all Rust Commerce services..."
    echo ""
    
    for app in "${apps[@]}"; do
        echo "=================================================="
        print_status "App: $app"
        echo "=================================================="
        get_app_ips "$app"
        echo ""
    done
}

# Get Fly.io's IP ranges
get_fly_ip_ranges() {
    print_status "Fly.io IP ranges for MongoDB whitelisting:"
    echo ""
    
    print_warning "Fly.io uses dynamic IP allocation. Here are the current IP ranges:"
    echo ""
    
    # Fly.io's documented IP ranges (these may change)
    echo "🌍 Fly.io Global IP Ranges:"
    echo "  IPv4: 66.241.124.0/24, 66.241.125.0/24"
    echo "  IPv6: 2a09:8280:1::/48"
    echo ""
    
    print_warning "⚠️  These ranges may change. For production, consider:"
    echo "  1. Use MongoDB Atlas network peering"
    echo "  2. Use a NAT gateway with static IP"
    echo "  3. Use Fly.io's dedicated IPv4 addresses"
    echo ""
}

# Check what IP the app sees itself as
check_external_ip() {
    local app_name=$1
    
    if [ -z "$app_name" ]; then
        print_error "Please provide an app name"
        return 1
    fi
    
    print_status "Checking external IP as seen by the app: $app_name"
    
    # Run a command inside the app to check its external IP
    flyctl ssh console --app "$app_name" --command "curl -s ifconfig.me" 2>/dev/null || \
    flyctl ssh console --app "$app_name" --command "wget -qO- ifconfig.me" 2>/dev/null || \
    print_warning "Could not determine external IP. App may not be running or doesn't have curl/wget."
}

# Show help
show_help() {
    echo "Fly.io IP Address Discovery Tool for MongoDB Whitelisting"
    echo ""
    echo "Usage: $0 <command> [app_name]"
    echo ""
    echo "Commands:"
    echo "  app <name>     Get IP addresses for specific app"
    echo "  all            Get IP addresses for all rust-commerce apps"
    echo "  ranges         Show Fly.io IP ranges"
    echo "  external <app> Check external IP as seen by the app"
    echo ""
    echo "Examples:"
    echo "  $0 app rust-commerce-catalog"
    echo "  $0 all"
    echo "  $0 ranges"
    echo "  $0 external rust-commerce-catalog"
    echo ""
    echo "For MongoDB Atlas whitelisting:"
    echo "  1. Use the external IP addresses shown"
    echo "  2. Consider whitelisting Fly.io's IP ranges"
    echo "  3. For production, use network peering or NAT gateway"
}

# Main script
check_flyctl

COMMAND=$1
APP_NAME=$2

case $COMMAND in
    app)
        if [ -z "$APP_NAME" ]; then
            print_error "Please provide an app name"
            show_help
            exit 1
        fi
        get_app_ips "$APP_NAME"
        ;;
    all)
        get_all_ips
        ;;
    ranges)
        get_fly_ip_ranges
        ;;
    external)
        if [ -z "$APP_NAME" ]; then
            print_error "Please provide an app name"
            show_help
            exit 1
        fi
        check_external_ip "$APP_NAME"
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

#!/bin/bash

# Orders Docker Container Management Script
# Usage: ./orders-docker.sh <env> <action>
# env: local|production (matches .env.local or .env.production)
# action: start|stop

CONTAINER_NAME="rust-commerce-order-service"
IMAGE_NAME="order-service:latest"

# Function to check if container is running
check_container_status() {
    if docker ps --filter "name=$CONTAINER_NAME" --format "table {{.Names}}\t{{.Status}}" | grep -q "$CONTAINER_NAME"; then
        echo "✅ Container '$CONTAINER_NAME' is currently running:"
        docker ps --filter "name=$CONTAINER_NAME" --format "table {{.Names}}\t{{.Status}}\t{{.Networks}}"
        return 0
    else
        echo "❌ Container '$CONTAINER_NAME' is not running"
        return 1
    fi
}

# Function to display help
show_help() {
    echo "Orders Service Docker Container Manager"
    echo ""
    echo "Usage: $0 <env> <action>"
    echo ""
    echo "Parameters:"
    echo "  env     Environment configuration (local|production)"
    echo "          - 'local' loads .env.local"
    echo "          - 'production' loads .env.production"
    echo ""
    echo "  action  Container action (start|stop)"
    echo "          - 'start' starts the container"
    echo "          - 'stop' stops and removes the container"
    echo ""
    echo "Examples:"
    echo "  $0 local start      # Start container with local environment"
    echo "  $0 production start # Start container with production environment"
    echo "  $0 local stop       # Stop the container"
    echo ""
    echo "Current container status:"
    check_container_status
}

# Function to start container
start_container() {
    local env=$1
    
    # Check if container is already running
    if check_container_status > /dev/null 2>&1; then
        echo "⚠️  Container '$CONTAINER_NAME' is already running. Stop it first."
        return 1
    fi
    
    # Validate environment file exists
    if [ ! -f ".env.$env" ]; then
        echo "❌ Environment file '.env.$env' not found!"
        echo "Available environment files:"
        ls -1 .env.* 2>/dev/null || echo "No .env files found"
        return 1
    fi
    
    echo "🚀 Starting container '$CONTAINER_NAME' with environment: $env"
    
    # Start container with environment file and connect to rust-commerce network
    docker run -d \
        --name "$CONTAINER_NAME" \
        --env-file ".env.$env" \
        -e RUST_ENV="$env" \
        --network "rust-commerce" \
        "$IMAGE_NAME"
    
    if [ $? -eq 0 ]; then
        echo "✅ Container started successfully!"
        sleep 2
        check_container_status
    else
        echo "❌ Failed to start container"
        return 1
    fi
}

# Function to stop container
stop_container() {
    echo "🛑 Stopping container '$CONTAINER_NAME'..."
    
    # Stop and remove container
    docker stop "$CONTAINER_NAME" 2>/dev/null
    docker rm "$CONTAINER_NAME" 2>/dev/null
    
    if [ $? -eq 0 ]; then
        echo "✅ Container stopped and removed successfully!"
    else
        echo "❌ Container was not running or failed to stop"
    fi
}

# Main script logic
if [ $# -eq 0 ]; then
    # No parameters provided - show help and status
    show_help
    exit 0
fi

if [ $# -ne 2 ]; then
    echo "❌ Error: Invalid number of parameters"
    echo ""
    show_help
    exit 1
fi

ENV=$1
ACTION=$2

# Validate environment parameter
if [ "$ENV" != "local" ] && [ "$ENV" != "production" ]; then
    echo "❌ Error: Invalid environment '$ENV'. Must be 'local' or 'production'"
    exit 1
fi

# Validate action parameter
if [ "$ACTION" != "start" ] && [ "$ACTION" != "stop" ]; then
    echo "❌ Error: Invalid action '$ACTION'. Must be 'start' or 'stop'"
    exit 1
fi

# Execute the requested action
case $ACTION in
    start)
        start_container "$ENV"
        ;;
    stop)
        stop_container
        ;;
    *)
        echo "❌ Error: Unknown action '$ACTION'"
        show_help
        exit 1
        ;;
esac

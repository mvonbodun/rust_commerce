#!/bin/bash

# Script to run integration tests with Docker Compose
set -e

echo "🐳 Starting Docker Compose test environment..."

# Change to project root
cd "$(dirname "$0")/../.."

# Stop any existing test containers
docker-compose -f docker-compose.test.yml down 2>/dev/null || true

# Start test environment
echo "📦 Starting test services..."
docker-compose -f docker-compose.test.yml up -d --wait

# Wait for services to be healthy
echo "⏳ Waiting for services to be ready..."
max_attempts=30
attempt=0

while [ $attempt -lt $max_attempts ]; do
    # Check if MongoDB is ready
    if docker-compose -f docker-compose.test.yml exec -T mongodb-test mongosh --eval "db.adminCommand('ping')" --quiet 2>/dev/null | grep -q "1"; then
        echo "✅ MongoDB is ready"
        break
    fi
    
    echo "   Waiting for MongoDB... (attempt $((attempt + 1))/$max_attempts)"
    sleep 2
    attempt=$((attempt + 1))
done

if [ $attempt -eq $max_attempts ]; then
    echo "❌ MongoDB failed to start"
    docker-compose -f docker-compose.test.yml logs mongodb-test
    exit 1
fi

# Check NATS
attempt=0
while [ $attempt -lt $max_attempts ]; do
    if curl -s http://localhost:8223/varz | grep -q '"port":4222' 2>/dev/null; then
        echo "✅ NATS is ready"
        break
    fi
    
    echo "   Waiting for NATS... (attempt $((attempt + 1))/$max_attempts)"
    sleep 2
    attempt=$((attempt + 1))
done

if [ $attempt -eq $max_attempts ]; then
    echo "❌ NATS failed to start"
    docker-compose -f docker-compose.test.yml logs nats-test
    exit 1
fi

# Wait for catalog service
attempt=0
while [ $attempt -lt $max_attempts ]; do
    if docker-compose -f docker-compose.test.yml logs catalog-service-test 2>/dev/null | grep -q "Catalog service is ready"; then
        echo "✅ Catalog service is ready"
        break
    fi
    
    echo "   Waiting for Catalog service... (attempt $((attempt + 1))/$max_attempts)"
    sleep 2
    attempt=$((attempt + 1))
done

if [ $attempt -eq $max_attempts ]; then
    echo "⚠️  Catalog service may not be fully ready"
    docker-compose -f docker-compose.test.yml logs catalog-service-test | tail -20
fi

# Run tests
echo ""
echo "🧪 Running integration tests..."
cd typescript-client

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    npm install
fi

# Generate types if needed
if [ ! -d "generated" ]; then
    echo "🔧 Generating TypeScript types..."
    npm run generate
fi

# Set test environment variables
export NATS_TEST_URL=nats://localhost:4223

# Run tests
npm run test:integration

TEST_EXIT_CODE=$?

# Cleanup
echo ""
echo "🧹 Cleaning up Docker containers..."
cd ..
docker-compose -f docker-compose.test.yml down

# Exit with test exit code
exit $TEST_EXIT_CODE
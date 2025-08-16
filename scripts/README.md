# Fly.io Deployment Scripts

This directory contains scripts to automate the deployment and management of Rust Commerce microservices on Fly.io.

## Scripts Overview

### 1. `fly-setup.sh` - Individual Service Setup
Sets up a single microservice on Fly.io with app creation, secrets configuration, and deployment.

**Usage:**
```bash
./fly-setup.sh <service> <env> [options]
```

**Examples:**
```bash
# Deploy catalog service with production config
./fly-setup.sh catalog production

# Deploy inventory service with staging config, skip app creation
./fly-setup.sh inventory staging --skip-create

# Dry run for price service
./fly-setup.sh price local --dry-run

# Deploy with status check
./fly-setup.sh orders production --show-status
```

**Available Services:**
- `catalog` - Product catalog service
- `inventory` - Inventory management service
- `orders` - Order processing service
- `price` - Pricing service
- `nats` - NATS messaging server

**Environment Types:**
- `local` - Uses `.env.local`
- `staging` - Uses `.env.staging`
- `production` - Uses `.env.production`

**Options:**
- `--skip-create` - Skip app creation (use existing app)
- `--skip-secrets` - Skip setting secrets
- `--skip-deploy` - Skip deployment
- `--show-status` - Show app status after deployment
- `--dry-run` - Show what would be done without executing

### 2. `fly-setup-all.sh` - Batch Service Setup
Deploys multiple services in sequence or parallel.

**Usage:**
```bash
./fly-setup-all.sh <env> [options]
```

**Examples:**
```bash
# Deploy all services to production
./fly-setup-all.sh production

# Deploy specific services to staging
./fly-setup-all.sh staging --services "catalog orders"

# Parallel deployment (experimental)
./fly-setup-all.sh production --parallel

# Skip secrets for all services
./fly-setup-all.sh staging --skip-secrets
```

**Options:**
- `--services "list"` - Specify services to deploy (default: catalog inventory price orders)
- `--skip-create` - Skip app creation for all services
- `--skip-secrets` - Skip setting secrets for all services
- `--skip-deploy` - Skip deployment for all services
- `--parallel` - Deploy services in parallel (experimental)
- `--dry-run` - Show what would be done without executing

### 3. `fly-monitor.sh` - Service Monitoring
Monitors the status of deployed services.

**Usage:**
```bash
./fly-monitor.sh [options] [service]
```

**Examples:**
```bash
# Show status once
./fly-monitor.sh

# Continuous monitoring (refreshes every 30s)
./fly-monitor.sh --watch

# Show status with recent logs
./fly-monitor.sh --logs

# Detailed status for specific service
./fly-monitor.sh --detail catalog

# List all rust-commerce apps
./fly-monitor.sh --list
```

**Options:**
- `--watch` - Monitor in watch mode (refresh every 30s)
- `--logs` - Show recent logs for all services
- `--detail` - Show detailed status (requires service name)
- `--list` - List all rust-commerce apps

## Prerequisites

1. **Install Fly.io CLI:**
   ```bash
   curl -L https://fly.io/install.sh | sh
   ```

2. **Login to Fly.io:**
   ```bash
   flyctl auth login
   ```

3. **Environment Files:**
   Ensure each service has the appropriate `.env` files:
   - `.env.local` - Local development
   - `.env.staging` - Staging environment
   - `.env.production` - Production environment

   Required variables:
   - `MONGODB_URL` - MongoDB connection string
   - `NATS_URL` - NATS server URL

## Deployment Workflow

### Single Service Deployment
```bash
# 1. Navigate to scripts directory
cd scripts

# 2. Deploy a single service
./fly-setup.sh catalog production

# 3. Monitor the deployment
./fly-monitor.sh --detail catalog
```

### Full Stack Deployment
```bash
# 1. Navigate to scripts directory
cd scripts

# 2. Deploy all services
./fly-setup-all.sh production

# 3. Monitor all services
./fly-monitor.sh --watch
```

### Staged Deployment
```bash
# 1. Deploy NATS first (if needed)
./fly-setup.sh nats production

# 2. Deploy core services
./fly-setup-all.sh production --services "catalog inventory price"

# 3. Deploy orders service
./fly-setup.sh orders production

# 4. Monitor everything
./fly-monitor.sh --logs
```

## Secrets Management

The scripts automatically extract `MONGODB_URL` and `NATS_URL` from the specified environment file and set them as Fly.io secrets. Other environment variables (like `RUST_LOG`) are configured in the `fly.toml` files.

### Manual Secrets Management
```bash
# Set secrets manually
flyctl secrets set --app rust-commerce-catalog MONGODB_URL="..."

# List secrets
flyctl secrets list --app rust-commerce-catalog

# Remove secrets
flyctl secrets unset --app rust-commerce-catalog SECRET_NAME
```

## Troubleshooting

### Common Issues

1. **App Already Exists:**
   ```bash
   # Use --skip-create to use existing app
   ./fly-setup.sh catalog production --skip-create
   ```

2. **Deployment Fails:**
   ```bash
   # Check logs
   flyctl logs --app rust-commerce-catalog
   
   # Check machine status
   flyctl machines list --app rust-commerce-catalog
   ```

3. **Secrets Not Set:**
   ```bash
   # Set secrets manually
   flyctl secrets set --app rust-commerce-catalog MONGODB_URL="..."
   ```

4. **Service Not Responding:**
   ```bash
   # Restart the service
   flyctl machines restart --app rust-commerce-catalog
   
   # Scale down and up
   flyctl scale count 0 --app rust-commerce-catalog
   flyctl scale count 1 --app rust-commerce-catalog
   ```

### Debug Mode
```bash
# Use dry-run to see what would happen
./fly-setup.sh catalog production --dry-run

# Check detailed status
./fly-monitor.sh --detail catalog
```

## File Structure

```
scripts/
├── fly-setup.sh       # Individual service setup
├── fly-setup-all.sh   # Batch service setup
├── fly-monitor.sh     # Service monitoring
└── README.md          # This file
```

## Notes

- Scripts must be run from the `scripts/` directory
- Environment files are expected in each service directory
- Logging configuration is handled by `fly.toml` files
- Services communicate via NATS messaging (no HTTP endpoints)
- Each service gets a unique app name: `rust-commerce-<service>`

## Production Checklist

Before deploying to production:

- [ ] Verify all `.env.production` files have correct values
- [ ] Test deployment in staging first
- [ ] Ensure MongoDB Atlas allows Fly.io IP ranges
- [ ] Verify NATS server is accessible from Fly.io
- [ ] Review resource allocation in `fly.toml` files
- [ ] Set up monitoring and alerting
- [ ] Configure backup strategies

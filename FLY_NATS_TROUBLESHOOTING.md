# Fly.io NATS Connection Troubleshooting Guide

## Problem
Catalog service fails to connect to NATS with DNS error:
```
ERROR: DNS error: failed to lookup address information: No address associated with hostname
```

## Root Cause Analysis
The issue is with Fly.io internal networking and service discovery. The `.flycast` hostname may not be resolving correctly.

## Solutions

### 1. Verify NATS App Configuration

Check your NATS app configuration:
```bash
# Check if NATS app is running
fly status -a rust-commerce-nats

# Check NATS app networking
fly ips list -a rust-commerce-nats
```

### 2. Update fly.toml for Internal Networking

Ensure your catalog service `fly.toml` has proper internal network configuration:

```toml
# fly.toml for catalog-service
app = "rust-commerce-catalog"

[build]
  dockerfile = "catalog/Dockerfile"

[[services]]
  internal_port = 8080
  processes = ["app"]
  protocol = "tcp"

# Internal networking configuration
[experimental]
  private_network = true

# Environment variables
[env]
  NATS_URL = "nats://rust-commerce-nats.flycast:4222"
  # or try: NATS_URL = "nats://rust-commerce-nats:4222"
```

### 3. Alternative NATS URL Formats to Try

The enhanced catalog service will now try multiple connection strategies:

1. **Original URL**: `nats://rust-commerce-nats.flycast:4222`
2. **Direct app name**: `nats://rust-commerce-nats:4222`
3. **Internal suffix**: `nats://rust-commerce-nats.internal:4222`

### 4. NATS Server Configuration

Ensure your NATS server `fly.toml` is configured for internal access:

```toml
# fly.toml for NATS server
app = "rust-commerce-nats"

[[services]]
  internal_port = 4222
  processes = ["app"]
  protocol = "tcp"
  
  # Internal service (no external ports needed for flycast)
  [[services.ports]]
    port = 4222
    handlers = ["tcp"]

# For external access (if still needed)
[[services]]
  internal_port = 4222
  processes = ["app"]
  protocol = "tcp"
  
  [[services.ports]]
    port = 4222
    handlers = ["tcp"]

# Environment variables
[env]
  NATS_HOST = "0.0.0.0"
  NATS_PORT = "4222"
```

### 5. Debugging Commands

```bash
# SSH into catalog service for debugging
fly ssh console -a rust-commerce-catalog

# Inside the container, test DNS resolution:
nslookup rust-commerce-nats.flycast
ping rust-commerce-nats.flycast
telnet rust-commerce-nats.flycast 4222

# Test different hostname variants:
nslookup rust-commerce-nats
nslookup rust-commerce-nats.internal
```

### 6. Environment Variable Options

Try these different NATS_URL configurations:

```bash
# Option 1: Standard flycast
NATS_URL=nats://rust-commerce-nats.flycast:4222

# Option 2: Direct app name (sometimes works better)
NATS_URL=nats://rust-commerce-nats:4222

# Option 3: With region (if apps are in different regions)
NATS_URL=nats://rust-commerce-nats.flycast:4222

# Option 4: External IP (if internal networking fails)
NATS_URL=nats://YOUR_DEDICATED_IP:4222
```

### 7. Enhanced Logging Output

The improved catalog service will now provide detailed debugging:
- DNS resolution attempts
- Multiple connection strategies
- Fly.io environment detection
- Network debugging information

### 8. Temporary Workaround

If internal networking continues to fail, use the external dedicated IP:

```bash
# Set in fly.toml or as secret
fly secrets set NATS_URL="nats://YOUR_DEDICATED_IPV4:4222" -a rust-commerce-catalog
```

## Testing the Fix

1. **Build and deploy** the updated catalog service:
```bash
cargo build --bin catalog-service
fly deploy -a rust-commerce-catalog
```

2. **Monitor logs** for the enhanced debugging:
```bash
fly logs -a rust-commerce-catalog
```

3. **Look for**:
   - DNS resolution debugging
   - Multiple connection strategy attempts
   - Successful connection message

## Next Steps

If the issue persists:
1. Check if both apps are in the same Fly.io region
2. Verify private networking is enabled for both apps
3. Consider using external IP as temporary workaround
4. Contact Fly.io support with the enhanced logs

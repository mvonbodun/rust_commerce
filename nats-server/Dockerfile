# NATS Server Dockerfile for Fly.io
FROM nats:2.10-alpine

# Copy NATS configuration
COPY nats-server.conf /etc/nats/nats-server.conf

# Expose NATS ports
EXPOSE 4222 6222 8222

# Start NATS server with configuration
CMD ["nats-server", "--config", "/etc/nats/nats-server.conf"]

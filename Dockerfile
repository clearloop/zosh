# =============================================================================
# Runtime Image
# =============================================================================
# Pre-requisite: Build the binary locally with `cargo build --release -p zosh-node`

FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libsqlite3-0 \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the pre-built binary
COPY target/release/zoshd /usr/local/bin/zoshd
RUN chmod +x /usr/local/bin/zoshd

# Expose the default development server port
EXPOSE 1439

# Default command
# Use 0.0.0.0:1439 to bind to all interfaces so it's accessible from outside the container
ENTRYPOINT ["zoshd"]
CMD ["dev", "--address", "0.0.0.0:1439"]

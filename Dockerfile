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

# Create non-root user for security
RUN useradd --create-home --shell /bin/bash zosh

# Create config and cache directories
RUN mkdir -p /home/zosh/.config/zosh /home/zosh/.cache/zosh && \
    chown -R zosh:zosh /home/zosh

# Copy the pre-built binary
COPY target/release/zoshd /usr/local/bin/zoshd
RUN chmod +x /usr/local/bin/zoshd

# Switch to non-root user
USER zosh
WORKDIR /home/zosh

# Expose the default development server port
EXPOSE 1439

# Set environment variables
ENV RUST_LOG=info
ENV ZOSH_CONFIG_DIR=/home/zosh/.config/zosh
ENV ZOSH_CACHE_DIR=/home/zosh/.cache/zosh

# Volume mounts for persistent data
VOLUME ["/home/zosh/.config/zosh", "/home/zosh/.cache/zosh"]

# Default command
ENTRYPOINT ["zoshd"]
CMD ["dev"]

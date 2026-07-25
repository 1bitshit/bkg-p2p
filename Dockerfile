# ── Stage 1: Build frontend ─────────────────────────────────────────
FROM node:22-alpine AS frontend
WORKDIR /app/web
COPY web/package.json web/package-lock.json* ./
RUN npm ci --ignore-scripts
COPY web/ .
RUN npm run build

# ── Stage 2: Build Rust binary ─────────────────────────────────────
FROM rust:1.88-bookworm AS builder
RUN apt-get update && apt-get install -y \
    cmake pkg-config libssl-dev protobuf-compiler \
    libclang-dev clang \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Cache dependencies: copy manifests, build.rs, and compile-time assets,
# create dummy src so cargo resolves + builds all deps without real source.
COPY Cargo.toml Cargo.lock build.rs ./
COPY prompts/ prompts/
COPY templates/ templates/
RUN mkdir -p src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release 2>/dev/null || true

# Now copy real source and rebuild (deps are cached)
COPY src/ src/
RUN touch src/main.rs && cargo build --release

# ── Stage 3: Runtime ───────────────────────────────────────────────
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates python3 python3-pip \
    curl jq poppler-utils \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /bkg-peer

# Copy binary
COPY --from=builder /app/target/release/bkg-p2p /usr/local/bin/bkg-peer

# Copy frontend dist
COPY --from=frontend /app/web/dist /bkg-peer/web/dist

# Copy templates and prompts
COPY templates/ /bkg-peer/templates/
COPY prompts/ /bkg-peer/prompts/

# Data directory
RUN mkdir -p /data/.bkg-peer
ENV BKG_PEER_HOME=/data/.bkg-peer
ENV BKG_PEER_WEB_DIST=/bkg-peer/web/dist

# Default port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s \
    CMD curl -sf http://localhost:8080/api/status || exit 1

# Default: serve with web dashboard, connect to Ollama on host
ENTRYPOINT ["bkg-peer"]
CMD ["serve", "--web", "0.0.0.0:8080", "--ollama"]

# ==========================================
# Stage 1: Build React Frontend with Bun
# ==========================================
FROM oven/bun:alpine AS frontend-builder
WORKDIR /app/frontend

COPY frontend/package.json frontend/bun.lock ./
RUN bun install --frozen-lockfile

COPY frontend/ ./
RUN bun run build

# ==========================================
# Stage 2: Build Rust Monolith Binary
# ==========================================
FROM rust:1-slim-bookworm AS backend-builder
WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo build --release -p quickbox-app

# ==========================================
# Stage 3: Minimal Production Runner
# ==========================================
FROM debian:bookworm-slim AS runner
WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates sqlite3 && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=backend-builder /app/target/release/quickbox-app /usr/local/bin/quickbox-app

# Copy compiled frontend assets
COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist

# Setup default directories & volumes
RUN mkdir -p /data/attachments
ENV DATABASE_URL="sqlite:///data/quickbox.db"
ENV ATTACHMENTS_DIR="/data/attachments"
ENV FRONTEND_DIST="/app/frontend/dist"
ENV DOMAIN="quickbox.dev"
ENV SMTP_PORT="25"
ENV HTTP_PORT="8080"

EXPOSE 25 8080

VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/quickbox-app"]


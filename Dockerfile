# Stage 1: Build the Frontend (Yew WebAssembly)
FROM rust:1.96-slim as frontend-builder
ARG TARGETARCH
WORKDIR /usr/src/app

# Install compilation dependencies and Trunk binary
RUN apt-get update && apt-get install -y --no-install-recommends \
    wget pkg-config libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown
RUN if [ "$TARGETARCH" = "arm64" ]; then TRUNK_ARCH="aarch64"; else TRUNK_ARCH="x86_64"; fi && \
    wget -qO- "https://github.com/trunk-rs/trunk/releases/download/v0.21.14/trunk-${TRUNK_ARCH}-unknown-linux-gnu.tar.gz" | tar -xzf- -C /usr/local/bin

COPY Cargo.toml Cargo.lock ./
COPY backend/ ./backend/
COPY frontend/ ./frontend/
WORKDIR /usr/src/app/frontend
RUN trunk build --release

# Stage 2: Build the Backend
FROM rust:1.96-slim as backend-builder
WORKDIR /usr/src/app

# Install dependencies required to build native packages like openssl-sys
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev gcc g++ make && \
    rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY backend/ ./backend/
COPY frontend/ ./frontend/
# We only compile the backend binary here
RUN cargo build --release --bin backend

# Stage 3: Final package
FROM debian:bookworm-slim
WORKDIR /usr/src/app

# Install runtime dependencies (SSL certificates for HTTPS requests and wget for health checks)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates wget && \
    rm -rf /var/lib/apt/lists/*

ENV PORT=4404

COPY --from=backend-builder /usr/src/app/target/release/backend ./rustwho
COPY --from=frontend-builder /usr/src/app/frontend/dist ./frontend/dist

RUN chown -R 99:100 /usr/src/app

# Run as Unraid nobody:users
USER 99:100

EXPOSE 4404

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s CMD wget -qO- http://localhost:4404/health || exit 1

CMD ["./rustwho"]
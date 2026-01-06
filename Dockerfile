# ============================
# Build
# ============================
FROM rust:1.92-bookworm AS builder

RUN apt-get update && \
    apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY assets ./assets

RUN cargo build --release

# ============================
# Runtime
# ============================
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y \
    ffmpeg \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/onvif-ip-camera-mock /app/onvif-ip-camera-mock
COPY assets ./assets

ENV RUST_LOG=info

CMD ["./onvif-ip-camera-mock"]
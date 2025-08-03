# ============ BUILDER IMAGE ============
FROM rust:slim-bookworm AS builder
WORKDIR /app

# Install system dependencies
RUN apt update -y && \
    apt install -y curl build-essential libssl-dev poppler-utils libopencv-dev clang libclang-dev && \
    apt clean && \
    rm -rf /var/lib/apt/lists/*

COPY . .
RUN cargo build --release

# ============ RUNNER IMAGE ============
FROM rust:slim-bookworm
WORKDIR /app

# Install system dependencies
RUN apt update -y && \
    apt install -y build-essential pkg-config poppler-utils libopencv-dev clang libclang-dev libssl-dev curl && \
    apt clean && \
    rm -rf /var/lib/apt/lists/*

# Install dependencies
RUN cargo install cargo-make && \
    cargo install cargo-nextest

COPY --from=builder /app/target/release/app ./target/release/app

# Stage 1: Build Environment
FROM rust:latest AS base
WORKDIR /usr/src/tinty
COPY . .
RUN cargo build --release

# Stage 2: Run cargo clippy
FROM base AS clippy
RUN rustup component add clippy && \
    cargo clippy -- -D warnings

# Stage 3: Run cargo fmt
FROM base as fmt
RUN rustup component add rustfmt && \
    cargo fmt --all -- --check

# Stage 4: Run tests
FROM base as test
ENV RUST_TEST_THREADS=1
RUN cargo test --release

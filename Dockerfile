# Stage 1: Build Environment
FROM rust:1.79.0 AS base
WORKDIR /usr/src/tinty
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY tests ./tests
COPY fixtures ./fixtures

RUN cargo build --release

# Stage 2: Run lint and tests
FROM base AS tests
RUN rustup component add clippy rustfmt
RUN cargo clippy -- -D warnings
RUN cargo fmt --all -- --check
ENV RUST_TEST_THREADS=1

CMD ["cargo", "test", "--release", "--", "--nocapture"]

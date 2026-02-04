# Stage 1: Chef setup
FROM rust:1.92.0 AS chef
RUN cargo install cargo-chef
WORKDIR /usr/src/tinty

# Stage 2: Prepare recipe
FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Build dependencies (cached layer)
FROM chef AS builder
COPY --from=planner /usr/src/tinty/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source and build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY tests ./tests
COPY fixtures ./fixtures
RUN cargo build --release

# Stage 4: Run lint and tests
FROM builder AS tests
RUN rustup component add clippy rustfmt
RUN cargo clippy -- -D warnings
RUN cargo fmt --all -- --check
ENV RUST_TEST_THREADS=1

CMD ["cargo", "test", "--release", "--", "--nocapture"]

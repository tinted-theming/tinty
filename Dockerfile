# Use an official Rust runtime as a parent image
FROM rust:latest

# Set the working directory in the container
WORKDIR /usr/src/myapp

# Copy the current directory contents into the container at /usr/src/myapp
COPY . .

# Compile the application
RUN cargo build --release

ENV RUST_TEST_THREADS=1

# Run tests
CMD ["cargo", "test"]

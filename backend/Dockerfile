# --- Stage 1: Build Stage ---
FROM rust:1.86-slim-bookworm as builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y libssl-dev pkg-config

# Copy the source code and build
COPY ./Cargo.toml ./Cargo.lock* ./
COPY ./src ./src

# Build the application
RUN cargo build --release

# --- Stage 2: Final Image ---
# Use a minimal, secure base image
FROM debian:bookworm-slim

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/backend /usr/local/bin/

# Expose the port the app runs on
EXPOSE 3000

# Set the command to run the application
CMD ["/usr/local/bin/backend"]


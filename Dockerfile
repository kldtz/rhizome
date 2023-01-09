FROM rust:1.66-slim AS builder

WORKDIR /app
# Install system dependencies for linking configuration
RUN apt update && apt install lld clang pkg-config libssl-dev -y

# Force offline compile-time verification
ENV SQLX_OFFLINE true

# Create dummy project to compile dependencies
RUN cargo init --lib
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release --lib

# Copy all files from working environment
COPY . .
# Build binary with release profile
RUN touch src/lib.rs && cargo build --release


FROM debian:bullseye-slim AS runtime

WORKDIR /app
COPY --from=builder /app/target/release/knowledge-base knowledge-base
COPY configuration configuration
COPY public public
ENV APP_ENVIRONMENT production
# Execute binary as entry point
ENTRYPOINT ["./knowledge-base"]
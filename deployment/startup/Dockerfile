FROM rust:1.66-slim AS builder

WORKDIR /app
# Install system dependencies
RUN apt update && apt install lld clang pkg-config libssl-dev -y

# Install sqlx CLI
RUN cargo install sqlx-cli

# Copy migrations
COPY migrations migrations

# Run database migrations
CMD ["sqlx", "migrate", "run"]

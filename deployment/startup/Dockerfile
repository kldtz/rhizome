FROM rust:1.66-slim AS builder

WORKDIR /app
# Install system dependencies
RUN apt update && apt install lld clang pkg-config libssl-dev -y

# Install sqlx CLI
RUN cargo install sqlx-cli


FROM debian:bullseye-slim AS runtime

RUN apt update && apt install postgresql-client -y

WORKDIR /app

# Copy sqlx client binary
COPY --from=builder /usr/local/cargo/bin/sqlx sqlx

# Copy migrations
COPY migrations migrations

# Run database migrations
CMD ["./sqlx", "migrate", "run"]

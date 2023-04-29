FROM rust:1.66-slim

WORKDIR /app
# Install system dependencies for linking configuration
RUN apt update && apt install lld clang pkg-config libssl-dev -y
RUN cargo install cargo-watch

# Force offline compile-time verification
ENV SQLX_OFFLINE true

# Create dummy project to compile dependencies
RUN cargo init --lib
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release --lib

# Copy all files from working environment
COPY . .
ENV APP_ENVIRONMENT production
# Watch files and run on change
CMD ["cargo", "watch", "-x", "run"]
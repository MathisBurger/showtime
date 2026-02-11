# Stage 1: Cargo Chef segments
FROM rust:1.82-slim-bookworm AS chef
RUN cargo install cargo-chef
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Stage 2: Planner
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Builder
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Now copy the actual source
COPY . .
RUN cargo build --release -p server-worker

# Stage 4: Final minimal runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -ms /bin/bash showtime
USER showtime
WORKDIR /home/showtime

# Copy the binary
COPY --from=builder /app/target/release/server-worker /usr/local/bin/server-worker

# Copy default config if needed, or assume it's mounted
# COPY --from=builder /app/server-worker/device_config.toml .

# Standard sACN port
EXPOSE 5568/udp

CMD ["server-worker"]

FROM rust:1.84.0 as builder

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./

COPY . .
RUN cargo build --release

FROM debian:buster-slim

# Install dependencies required for running the binary
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy over the compiled binary
COPY --from=builder /usr/src/app/target/release/api_gateway /usr/local/bin/api_gateway

EXPOSE 8080

CMD ["api_gateway"]

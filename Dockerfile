FROM rustlang/rust:nightly AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

# to build deps first
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --locked
RUN rm src/main.rs

COPY src/ ./src/

RUN cargo build --release --locked

FROM debian:12-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/omajinai /usr/local/bin/omajinai

ENTRYPOINT ["/usr/local/bin/omajinai"]

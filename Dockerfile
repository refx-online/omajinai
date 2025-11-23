FROM rustlang/rust:nightly AS builder

ENV RUSTFLAGS="-C target-cpu=native -C link-arg=-s"

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

# to build deps first
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# build da dependencies with registry cache
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release

RUN rm -rf src/

COPY src/ ./src/

# "build" da actual application with cache mounts
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked && \
    cp /app/target/release/omajinai /omajinai

FROM gcr.io/distroless/cc-debian12

COPY --from=builder /omajinai /usr/local/bin/omajinai

ENTRYPOINT ["/usr/local/bin/omajinai"]

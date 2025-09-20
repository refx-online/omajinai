FROM rustlang/rust:nightly AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

# to build deps first
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src/

COPY src/ ./src/

RUN RUSTFLAGS="-C target-cpu=native -C link-arg=-s" cargo build --release --locked

FROM gcr.io/distroless/cc-debian12

COPY --from=builder /app/target/release/omajinai /usr/local/bin/omajinai

ENTRYPOINT ["/usr/local/bin/omajinai"]

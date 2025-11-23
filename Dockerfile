FROM rustlang/rust:nightly AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

ENV RUSTFLAGS="-C target-cpu=native -C link-arg=-s"

COPY --from=planner /app/recipe.json recipe.json

# build da dependencies with registry cache
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --recipe-path recipe.json

COPY . .

# "build" da actual application with cache mounts
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked && \
    cp target/release/omajinai /omajinai

FROM gcr.io/distroless/cc-debian12

COPY --from=builder /omajinai /usr/local/bin/omajinai

ENTRYPOINT ["/usr/local/bin/omajinai"]

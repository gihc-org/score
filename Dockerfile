FROM rust:1-slim AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && echo '' > src/lib.rs && mkdir -p src/routes && touch src/routes/mod.rs
RUN cargo build --release && rm -rf src

COPY src ./src
COPY migrations ./migrations
RUN touch src/main.rs src/lib.rs && cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/* \
    && groupadd -r appuser && useradd -r -g appuser appuser

WORKDIR /app
COPY --from=builder /app/target/release/score .
COPY migrations ./migrations

USER appuser

CMD ["./score"]

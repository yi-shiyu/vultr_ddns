FROM rust:bookworm AS builder

WORKDIR /usr/src/myapp

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /usr/src/myapp

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/myapp/target/release/vultr_ddns .

ENTRYPOINT ["./vultr_ddns"]
FROM rust:bookworm AS builder

WORKDIR /usr/src/myapp

COPY . .

RUN apt update && apt install -y musl-tools && rustup target add x86_64-unknown-linux-musl && cargo build --target x86_64-unknown-linux-musl --release

FROM alpine:latest

WORKDIR /usr/src/myapp

#RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/myapp/target/x86_64-unknown-linux-musl/release/vultr_ddns .

ENTRYPOINT ["./vultr_ddns"]

FROM rust:bookworm AS builder

WORKDIR /usr/src/myapp

COPY . .

ENV RUSTFLAGS='-C linker=x86_64-linux-gnu-gcc'

RUN apt -y update && apt install -y musl-tools musl-dev && rustup target add x86_64-unknown-linux-musl && cargo build --target x86_64-unknown-linux-musl --release

FROM alpine:latest

WORKDIR /usr/src/myapp

COPY --from=builder /usr/src/myapp/target/x86_64-unknown-linux-musl/release/vultr_ddns .

ENTRYPOINT ["./vultr_ddns"]

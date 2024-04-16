FROM rust:alpine AS builder

WORKDIR /usr/src/myapp

COPY . .

RUN apk update && apk add musl musl-dev && cargo build --release

FROM alpine:latest

WORKDIR /usr/src/myapp

COPY --from=builder /usr/src/myapp/target/release/vultr_ddns .

ENTRYPOINT ["./vultr_ddns"]

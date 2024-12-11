FROM rust:alpine AS builder

RUN apk update
RUN apk add --no-cache \
  pkgconfig protoc gcc g++ libc-dev musl-dev \
  openssl openssl-dev cmake opus opus-dev
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /bami

# Build dependencies
RUN USER=root cargo init .
COPY Cargo.toml /bami/
COPY Cargo.lock /bami/
RUN RUSTFLAGS="-Ctarget-feature=-crt-static" cargo build --target=x86_64-unknown-linux-musl --release

# Build
COPY src src
RUN echo "//" >> /bami/src/main.rs;
RUN RUSTFLAGS="-Ctarget-feature=-crt-static" cargo build --target=x86_64-unknown-linux-musl --release

FROM alpine

RUN apk update
RUN apk add --no-cache \
  libgcc openssl opus yt-dlp

COPY --from=builder /bami/target/x86_64-unknown-linux-musl/release/bami /bin/

CMD ["/bin/bami"]

FROM rust:alpine as builder

WORKDIR /usr/src

RUN apk add openssl-dev musl-dev

RUN USER=root cargo new rustbase

COPY Cargo.toml Cargo.lock /usr/src/rustbase/
COPY src/build.rs /usr/src/rustbase/src/
COPY .cargo/ /usr/src/rustbase/.cargo/

WORKDIR /usr/src/rustbase

RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl

COPY src /usr/src/rustbase/src/

RUN touch /usr/src/rustbase/src/main.rs

RUN TARGET_CC=x86_64-linux-musl-gcc cargo build --target x86_64-unknown-linux-musl --release

# -------

FROM alpine:latest as runtime

COPY --from=builder /usr/src/rustbase/target/x86_64-unknown-linux-musl/release/rustbase /usr/local/bin/rustbase/rustbase_server

VOLUME /usr/local/bin/rustbase/data

EXPOSE 23561

CMD ["/usr/local/bin/rustbase/rustbase_server"]
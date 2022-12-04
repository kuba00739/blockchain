FROM rust:latest

WORKDIR /app

#COPY registry/ /usr/local/cargo/registry/
COPY ./ /app

RUN cargo build --release

ENTRYPOINT ["/app/target/release/node"]

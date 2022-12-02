FROM rust:latest

WORKDIR /app

COPY ./ /app

RUN cargo build --release

ENTRYPOINT ["/app/target/release/blockchain"]

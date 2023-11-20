FROM rust:1.72.1-slim-bullseye as builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev
COPY Cargo.toml ./
RUN mkdir src/
RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs
RUN cargo build --release
RUN rm -f target/release/candle-ocr-server*

COPY src ./src
COPY .cargo ./.cargo

# update the last modified time of main.rs to force a rebuild
RUN touch src/main.rs
RUN cargo build --release

COPY ./create-swap.sh /create-swap.sh
COPY ./app/dist ./app/dist

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y libssl-dev procps htop
COPY --from=builder /target/release/candle-ocr-server /usr/local/bin/candle-ocr-server
COPY --from=builder /create-swap.sh /usr/local/bin/create-swap.sh
COPY --from=builder /app/dist /app/dist

RUN chmod a+x /usr/local/bin/create-swap.sh

ENTRYPOINT [ "bin/bash", "-c", "/usr/local/bin/create-swap.sh; candle-ocr-server" ]

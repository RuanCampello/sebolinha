FROM rust:1.98-slim-bookworm AS builder

RUN apt-get update \
    && apt-get install --yes --no-install-recommends build-essential ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock schema.sql ./
COPY src ./src
RUN cargo build --locked --release

FROM debian:bookworm-slim

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

RUN groupadd --system sebolinha \
    && useradd --system --gid sebolinha sebolinha \
    && mkdir /data \
    && chown sebolinha:sebolinha /data

COPY --from=builder /app/target/release/sebolinha /usr/local/bin/sebolinha

USER sebolinha
ENV SEBOLINHA_DB=/data/sebolinha.sqlite3
VOLUME ["/data"]
ENTRYPOINT ["/usr/local/bin/sebolinha"]

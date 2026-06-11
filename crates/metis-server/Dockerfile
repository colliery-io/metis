# Metis 3.0 server image (multi-stage). Build from the repo root:
#
#   docker build -f crates/metis-server/Dockerfile -t metis-server .
#
# (diesel-dualdb resolves from crates.io, so no out-of-tree dependencies.)

FROM rust:1.93-slim AS builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev libpq-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /build
COPY . .
RUN cargo build --release -p metis-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libpq5 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/metis /usr/local/bin/metis
EXPOSE 7878
ENV METIS_BIND=0.0.0.0:7878
ENTRYPOINT ["metis"]
CMD ["serve"]

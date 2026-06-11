# Metis 3.0 server image (multi-stage).
#
# BUILD CONTEXT NOTE: metis-core depends on diesel-dualdb, currently an
# out-of-tree path dependency (`../../../diesel-dualdb`). Until it is published
# to crates.io or pinned as a git dependency, this image must be built with a
# context that contains BOTH the metis repo and a sibling `diesel-dualdb`
# checkout, e.g.:
#
#   (cd .. && docker build -f metis/crates/metis-server/Dockerfile \
#       --build-arg METIS=metis --build-arg DUALDB=diesel-dualdb .)
#
# This is the documented release blocker tracked in METIS-T-0126.

FROM rust:1.93-slim AS builder
ARG METIS=metis
ARG DUALDB=diesel-dualdb
RUN apt-get update && apt-get install -y pkg-config libssl-dev libpq-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /build
COPY ${DUALDB} ./diesel-dualdb
COPY ${METIS} ./metis
WORKDIR /build/metis
RUN cargo build --release -p metis-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libpq5 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/metis/target/release/metis /usr/local/bin/metis
EXPOSE 7878
ENV METIS_BIND=0.0.0.0:7878
ENTRYPOINT ["metis"]
CMD ["serve"]

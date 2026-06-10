# Mercury relay — multi-stage build.
#
# The relay is a dumb opaque-ciphertext router (see docs/RELAY-DEPLOYMENT.md). It serves plain
# HTTP and does no outbound TLS, so the runtime image needs only glibc — no OpenSSL, no certs.
# TLS is terminated by the Caddy reverse proxy in docker-compose.yml.
#
# Build context is the REPOSITORY ROOT (docker-compose sets `context: ..`). We copy only the root
# manifests + core/rust (every Cargo workspace member lives there); the excluded `fuzz` and
# `ui/app/src-tauri` crates are not needed and are not copied.

# --- build stage -------------------------------------------------------------
FROM rust:1.92-bookworm AS build
WORKDIR /src

# Manifests + all workspace member sources.
COPY Cargo.toml Cargo.lock ./
COPY core/rust ./core/rust

# Build only the relay server binary in release. (First build is slow — it compiles the relay's
# dependency tree from scratch; subsequent rebuilds reuse the layer cache when sources are unchanged.)
RUN cargo build -p mercury-relay --bin mercury-relay-server --release \
    && strip target/release/mercury-relay-server

# --- runtime stage -----------------------------------------------------------
FROM debian:bookworm-slim AS runtime

# Run as an unprivileged user; /data holds the durable redb queue (mounted volume).
RUN useradd --system --uid 10001 --no-create-home relay \
    && mkdir -p /data && chown relay:relay /data

COPY --from=build /src/target/release/mercury-relay-server /usr/local/bin/mercury-relay-server

USER relay
WORKDIR /data

# Listen on all interfaces INSIDE the container; the port is only reachable on the private compose
# network (it is `expose`d, not published). The host exposes nothing but Caddy's 80/443.
ENV MERCURY_RELAY_ADDR=0.0.0.0:8787 \
    MERCURY_RELAY_DB=/data/relay.redb
EXPOSE 8787

ENTRYPOINT ["mercury-relay-server"]

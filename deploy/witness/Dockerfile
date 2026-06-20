# Mercury KT witness / auditor — multi-stage build.
#
# A witness co-signs the relay's published tree heads, REFUSING any head that is not an append-only
# extension of what it last vouched for (see docs/WITNESS-OPERATORS.md + docs/WITNESSING.md). It is a
# CLIENT of the relay: it makes OUTBOUND HTTPS only (rustls + webpki-roots — the public CA roots are
# compiled IN), so the runtime image needs no `ca-certificates` package and opens no inbound ports.
#
# SINGLE-SHOT: one fetch -> verify -> co-sign round per run, then exit. Schedule it on a timer
# (systemd timer / cron / k8s CronJob) — see the runbook. The whole protection is that this runs on a
# host INDEPENDENT of the relay operator; the image cannot manufacture that independence.
#
# Build context is the REPOSITORY ROOT (the Cargo workspace lives at core/rust), e.g.
#   docker build -f deploy/witness/Dockerfile -t mercury-witness .

# --- build stage -------------------------------------------------------------
FROM rust:1.92-bookworm AS build
WORKDIR /src

# Manifests + all workspace member sources.
COPY Cargo.toml Cargo.lock ./
COPY core/rust ./core/rust

# Build only the witness binary in release. (First build is slow — it compiles the dependency tree
# from scratch; subsequent rebuilds reuse the layer cache when sources are unchanged.)
RUN cargo build -p mercury-witness --bin mercury-kt-witness --release \
    && strip target/release/mercury-kt-witness

# --- runtime stage -----------------------------------------------------------
FROM debian:bookworm-slim AS runtime

# Run as an unprivileged user; /data holds the persisted last-seen head (and, if you choose to mount
# it here, the signing key — keep that file `0600` and owned by this user).
RUN useradd --system --uid 10001 --no-create-home witness \
    && mkdir -p /data && chown witness:witness /data

COPY --from=build /src/target/release/mercury-kt-witness /usr/local/bin/mercury-kt-witness

USER witness
WORKDIR /data

# No inbound ports: a witness only makes OUTBOUND requests to the relay. Pass the witness flags as
# `docker run` arguments (they are appended to this entrypoint), e.g.:
#
#   docker run --rm -v /srv/witness:/data:rw mercury-witness \
#     --relay https://relay.example.com --key-file /data/witness.key --index 0 \
#     --state /data/witness-state.json --log-key <64-hex-log-pubkey>
#
# Generate a keypair first with:  docker run --rm mercury-witness keygen > witness.key
ENTRYPOINT ["mercury-kt-witness"]

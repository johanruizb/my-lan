# Multi-stage build for mylan-agent (ADR-4: single binary serves agent + API).
# buildx multiarch: docker buildx build --platform linux/amd64,linux/arm64 ...

FROM rust:1.96-slim AS builder
WORKDIR /build
# Cache deps: copy manifests + source, then build.
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY apps/ apps/
COPY tests/ tests/
COPY agent/mylan-agent.toml agent/mylan-agent.toml
RUN cargo build --release -p mylan-agent

# Runtime en trixie (Debian 13) para alinear GLIBC con el builder
# `rust:1.96-slim` (trixie, GLIBC ≥ 2.39). Bookworm (GLIBC 2.36) hace fallar
# al binario release al arrancar: "GLIBC_2.39 not found".
FROM debian:trixie-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /build/target/release/mylan-agent /usr/local/bin/mylan-agent
COPY agent/mylan-agent.toml /etc/mylan/mylan-agent.toml
# DB + token persist in /var/lib/mylan (mount a volume).
VOLUME ["/var/lib/mylan"]
EXPOSE 43117
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD curl -f -H "Authorization: Bearer $(cat /var/lib/mylan/api-token)" http://127.0.0.1:43117/api/v1/status || exit 1
CMD ["mylan-agent", "--serve-api", "--config", "/etc/mylan/mylan-agent.toml"]
# SPDX-License-Identifier: PMPL-1.0-or-later
# Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <jonathan.jewell@open.ac.uk>
#
# Build: cerro-torre build -f Containerfile -t indieweb2-bastion:latest .
# Sign:  cerro-torre sign indieweb2-bastion:latest
# Seal:  selur seal indieweb2-bastion:latest
# Run:   vordr run indieweb2-bastion:latest

# --- Build stage ---
FROM cgr.dev/chainguard/wolfi-base:latest AS build

RUN apk add --no-cache deno rust cargo

WORKDIR /build
COPY graphql-dns-api/ ./graphql-dns-api/
COPY services/ ./services/
COPY policy/ ./policy/

# Build graphql-dns-api
RUN cd graphql-dns-api && cargo build --release

# Build webmention-rate-limiter
RUN cd services/webmention-rate-limiter && cargo build --release

# --- Runtime stage ---
FROM cgr.dev/chainguard/static:latest

LABEL maintainer="Jonathan D.A. Jewell <jonathan.jewell@open.ac.uk>"
LABEL version="0.2.0"
LABEL org.opencontainers.image.source="https://github.com/hyperpolymath/indieweb2-bastion"
LABEL org.opencontainers.image.licenses="PMPL-1.0-or-later"

COPY --from=build /build/graphql-dns-api/target/release/graphql-dns-api /usr/local/bin/graphql-dns-api
COPY --from=build /build/services/webmention-rate-limiter/target/release/webmention-rate-limiter /usr/local/bin/webmention-rate-limiter
COPY policy/ /etc/indieweb2/policy/

# Security: run as non-root
USER nonroot:nonroot

EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/graphql-dns-api"]

# SPDX-License-Identifier: PMPL-1.0-or-later
# Justfile - indieweb2-bastion task runner

default:
    @just --list

# --- Build ---

# Build all Rust services
build:
    cd graphql-dns-api && cargo build --release
    cd services/webmention-rate-limiter && cargo build --release

# Build container image (cerro-torre, NOT docker/podman build)
container-build:
    cerro-torre build -f Containerfile -t indieweb2-bastion:latest .

# Sign container image (Ed25519 signatures via cerro-torre)
container-sign:
    cerro-torre sign indieweb2-bastion:latest

# Seal container for zero-copy IPC (selur)
container-seal:
    selur seal indieweb2-bastion:latest

# Full container pipeline: build → sign → seal
container: container-build container-sign container-seal

# --- Run ---

# Start all services (selur-compose, NOT docker-compose)
up:
    selur-compose up

# Stop all services
down:
    selur-compose down

# Run via vordr (formally verified container runtime)
run:
    vordr run indieweb2-bastion:latest

# --- Test ---

# Run all tests
test:
    cd graphql-dns-api && cargo test
    cd services/webmention-rate-limiter && cargo test

# Run security scan (panic-attack)
scan:
    panic-attack assail . --output /tmp/indieweb2-bastion-scan.json

# --- Lint ---

# Run lints and checks
lint:
    cd graphql-dns-api && cargo clippy -- -D warnings
    cd services/webmention-rate-limiter && cargo clippy -- -D warnings

# Format code
fmt:
    cd graphql-dns-api && cargo fmt
    cd services/webmention-rate-limiter && cargo fmt

# Run all checks
check: lint test

# --- Clean ---

# Clean build artifacts
clean:
    cd graphql-dns-api && cargo clean
    cd services/webmention-rate-limiter && cargo clean

# --- Release ---

# Prepare a release
release VERSION:
    @echo "Releasing {{VERSION}}..."
    just build
    just container

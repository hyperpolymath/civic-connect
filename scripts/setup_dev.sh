#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell
#
# CivicConnect Development Environment Setup
# This script sets up the complete development environment

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check for required tools
check_requirements() {
    log_info "Checking requirements..."

    local missing=()

    # Check Podman (NOT Docker)
    if ! command -v podman &> /dev/null; then
        missing+=("podman")
    fi

    if ! command -v podman-compose &> /dev/null; then
        missing+=("podman-compose")
    fi

    # Check Ada toolchain (optional for first setup)
    if ! command -v gnat &> /dev/null; then
        log_warn "GNAT (Ada compiler) not found. Install with: sudo apt install gnat gprbuild"
    fi

    # Check Rust
    if ! command -v cargo &> /dev/null; then
        missing+=("rust/cargo")
    fi

    # Check Elixir
    if ! command -v mix &> /dev/null; then
        log_warn "Elixir not found. Install with: asdf install elixir 1.16.0"
    fi

    if [ ${#missing[@]} -gt 0 ]; then
        log_error "Missing required tools: ${missing[*]}"
        log_info "Please install missing tools and try again."
        exit 1
    fi

    log_info "All required tools found."
}

# Start infrastructure services
start_services() {
    log_info "Starting infrastructure services..."

    cd "$(dirname "$0")/../infrastructure"

    # Create .env file if it doesn't exist
    if [ ! -f .env ]; then
        log_info "Creating .env file with default values..."
        cat > .env << EOF
POSTGRES_PASSWORD=civicconnect_dev
MINIO_ROOT_USER=civicconnect
MINIO_ROOT_PASSWORD=civicconnect_dev
EOF
    fi

    podman-compose up -d

    log_info "Waiting for services to be ready..."
    sleep 5

    # Check PostgreSQL
    if podman exec civicconnect-postgres pg_isready -U civicconnect &> /dev/null; then
        log_info "PostgreSQL is ready."
    else
        log_warn "PostgreSQL may not be ready yet. Check with: podman logs civicconnect-postgres"
    fi

    # Check Redis
    if podman exec civicconnect-redis redis-cli ping &> /dev/null; then
        log_info "Redis is ready."
    else
        log_warn "Redis may not be ready yet. Check with: podman logs civicconnect-redis"
    fi

    cd - > /dev/null
}

# Set up Rust API
setup_rust() {
    log_info "Setting up Rust API..."

    cd "$(dirname "$0")/../backend/rust-api"

    # Build in development mode
    cargo build

    log_info "Rust API built successfully."
    cd - > /dev/null
}

# Set up Elixir Phoenix
setup_elixir() {
    if ! command -v mix &> /dev/null; then
        log_warn "Skipping Elixir setup (mix not found)"
        return
    fi

    log_info "Setting up Elixir Phoenix..."

    cd "$(dirname "$0")/../backend/elixir-phoenix"

    # Install dependencies
    mix deps.get

    log_info "Elixir dependencies installed."
    cd - > /dev/null
}

# Set up Ada core
setup_ada() {
    if ! command -v gprbuild &> /dev/null; then
        log_warn "Skipping Ada setup (gprbuild not found)"
        return
    fi

    log_info "Setting up Ada core..."

    cd "$(dirname "$0")/../backend/ada-core"

    # Create obj and bin directories
    mkdir -p obj bin

    # Build
    gprbuild -P civicconnect.gpr

    log_info "Ada core built successfully."
    cd - > /dev/null
}

# Main
main() {
    log_info "CivicConnect Development Setup"
    log_info "=============================="

    check_requirements
    start_services
    setup_rust
    setup_elixir
    setup_ada

    log_info ""
    log_info "Setup complete!"
    log_info ""
    log_info "Next steps:"
    log_info "  1. Run Rust API:     cd backend/rust-api && cargo run"
    log_info "  2. Run Elixir:       cd backend/elixir-phoenix && mix phx.server"
    log_info "  3. View services:    podman ps"
    log_info "  4. Stop services:    cd infrastructure && podman-compose down"
    log_info ""
    log_info "Service URLs:"
    log_info "  - PostgreSQL:  localhost:5432"
    log_info "  - Redis:       localhost:6379"
    log_info "  - MinIO:       localhost:9000 (API) / localhost:9001 (Console)"
}

main "$@"

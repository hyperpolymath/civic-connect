#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell
#
# CivicConnect Test Runner
# Runs tests for all components (Ada, Rust, Elixir)

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Track test results
declare -A RESULTS
FAILED=0

# Run Ada tests
run_ada_tests() {
    log_info "Running Ada tests..."

    if ! command -v gprbuild &> /dev/null; then
        log_warn "Skipping Ada tests (gprbuild not found)"
        RESULTS["Ada"]="SKIPPED"
        return
    fi

    cd "$PROJECT_ROOT/backend/ada-core"

    # Build and run tests
    if gprbuild -P civicconnect.gpr; then
        RESULTS["Ada"]="PASSED"
    else
        RESULTS["Ada"]="FAILED"
        FAILED=1
    fi

    cd - > /dev/null
}

# Run Rust tests
run_rust_tests() {
    log_info "Running Rust tests..."

    if ! command -v cargo &> /dev/null; then
        log_warn "Skipping Rust tests (cargo not found)"
        RESULTS["Rust"]="SKIPPED"
        return
    fi

    cd "$PROJECT_ROOT/backend/rust-api"

    if cargo test; then
        RESULTS["Rust"]="PASSED"
    else
        RESULTS["Rust"]="FAILED"
        FAILED=1
    fi

    cd - > /dev/null
}

# Run Elixir tests
run_elixir_tests() {
    log_info "Running Elixir tests..."

    if ! command -v mix &> /dev/null; then
        log_warn "Skipping Elixir tests (mix not found)"
        RESULTS["Elixir"]="SKIPPED"
        return
    fi

    cd "$PROJECT_ROOT/backend/elixir-phoenix"

    if mix test; then
        RESULTS["Elixir"]="PASSED"
    else
        RESULTS["Elixir"]="FAILED"
        FAILED=1
    fi

    cd - > /dev/null
}

# Run linters
run_linters() {
    log_info "Running linters..."

    # Rust linting
    if command -v cargo &> /dev/null; then
        cd "$PROJECT_ROOT/backend/rust-api"
        if cargo clippy -- -D warnings; then
            RESULTS["Rust Lint"]="PASSED"
        else
            RESULTS["Rust Lint"]="FAILED"
            FAILED=1
        fi
        cd - > /dev/null
    else
        RESULTS["Rust Lint"]="SKIPPED"
    fi

    # Elixir linting
    if command -v mix &> /dev/null; then
        cd "$PROJECT_ROOT/backend/elixir-phoenix"
        if mix credo --strict; then
            RESULTS["Elixir Lint"]="PASSED"
        else
            RESULTS["Elixir Lint"]="FAILED"
            FAILED=1
        fi
        cd - > /dev/null
    else
        RESULTS["Elixir Lint"]="SKIPPED"
    fi
}

# Print summary
print_summary() {
    log_info ""
    log_info "=============================="
    log_info "Test Summary"
    log_info "=============================="

    for component in "${!RESULTS[@]}"; do
        result="${RESULTS[$component]}"
        case "$result" in
            "PASSED")
                echo -e "${GREEN}[PASS]${NC} $component"
                ;;
            "FAILED")
                echo -e "${RED}[FAIL]${NC} $component"
                ;;
            "SKIPPED")
                echo -e "${YELLOW}[SKIP]${NC} $component"
                ;;
        esac
    done

    log_info "=============================="

    if [ $FAILED -eq 0 ]; then
        log_info "All tests passed!"
        exit 0
    else
        log_error "Some tests failed!"
        exit 1
    fi
}

# Main
main() {
    log_info "CivicConnect Test Runner"
    log_info "=============================="

    # Parse arguments
    RUN_LINT=false
    while [[ $# -gt 0 ]]; do
        case $1 in
            --lint)
                RUN_LINT=true
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done

    run_ada_tests
    run_rust_tests
    run_elixir_tests

    if [ "$RUN_LINT" = true ]; then
        run_linters
    fi

    print_summary
}

main "$@"

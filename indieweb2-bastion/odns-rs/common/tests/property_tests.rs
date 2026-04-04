// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// Property-based test entry-point for odns-common.
//
// Cargo discovers integration tests by scanning the `tests/` directory for
// `*.rs` files. Each top-level file becomes its own test binary.
// This file re-exports the property sub-module so all proptest cases are
// included in a single binary named `property_tests`.

mod property {
    mod crypto_properties_test;
}

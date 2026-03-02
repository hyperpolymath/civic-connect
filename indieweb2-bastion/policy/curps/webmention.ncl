# SPDX-FileCopyrightText: 2025 Hyperpolymath
# SPDX-License-Identifier: PMPL-1.0-or-later
#
# Webmention ingress hardening policy — v0.1.0
#
# This policy enforces rate limiting and validation for Webmention endpoints
# at the ingress layer, protecting against spam floods, DoS attacks, and
# amplification abuse.

let schema = import "./schema.ncl"

let webmention_policy = schema.WebmentionPolicy {
  routes = [
    "/webmention",
    "/webmention.php",
    "/.well-known/webmention",
  ],
  constraints = {
    # Stricter than default 120 rpm due to processing overhead
    max_rate_rpm = 60,

    # Per source URL limit to prevent single-source floods
    max_rate_per_source = 10,

    # Webmention spec requires form-urlencoded
    require_content_type = ["application/x-www-form-urlencoded"],

    # Reject requests missing source or target parameters
    require_source_target = true,

    # Block self-referential pings (same domain source/target)
    block_self_ping = true,

    # Cooldown period after burst detection (30 seconds)
    cooldown_on_burst_ms = 30000,
  },
  enabled = true,
}

export webmention_policy

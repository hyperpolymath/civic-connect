# SPDX-License-Identifier: PMPL-1.0-or-later
# CURPS Policy for SDP
capabilities:
  - mutate_routes: maintainers
  - rotate_keys: maintainers
  - join_overlay: trusted_contributors
constraints:
  - require_mtls: true
  - log_all_mutations: true

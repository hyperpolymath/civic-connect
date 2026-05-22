# SPDX-License-Identifier: MPL-2.0
# CURPS Policy for SDP
capabilities:
  - mutate_routes: maintainers
  - rotate_keys: maintainers
  - join_overlay: trusted_contributors
constraints:
  - require_mtls: true
  - log_all_mutations: true

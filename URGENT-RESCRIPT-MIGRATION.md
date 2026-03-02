# URGENT: ReScript Migration Required

**Generated:** 2026-03-02
**Current stable ReScript:** 12.2.0
**Pre-release:** 13.0.0-alpha.2 (2025-02-27)

This repo has ReScript code that needs migration. Address in priority order.

## CRITICAL: Pure BuckleScript (bsconfig.json only)

These locations use the legacy BuckleScript config format with NO rescript.json.
**ReScript 13 removes bsconfig.json support entirely.**

- `indieweb2-bastion`

**Action required:**
1. Rename `bsconfig.json` → `rescript.json`
2. Replace `bs-dependencies` → `dependencies`, `bs-dev-dependencies` → `dev-dependencies`
3. Remove `bsc-flags` (deprecated)
4. Check for `.re` (Reason) files → convert to `.res` syntax
5. Replace any `bs-platform` dependency with `rescript` ^12.2.0
6. Migrate all `@bs.` attributes to `@` equivalents
7. Migrate `Js.*` APIs to `@rescript/core` equivalents

---

## ReScript 13 Preparation (v13.0.0-alpha.2 available)

v13 is in alpha. These breaking changes are CONFIRMED — prepare now:

1. **`bsconfig.json` support removed** — must use `rescript.json` only
2. **`rescript-legacy` command removed** — only modern build system
3. **`bs-dependencies`/`bs-dev-dependencies`/`bsc-flags` config keys removed**
4. **Uncurried `(. args) => ...` syntax removed** — use standard `(args) => ...`
5. **`es6`/`es6-global` module format names removed** — use `esmodule`
6. **`external-stdlib` config option removed**
7. **`--dev`, `--create-sourcedirs`, `build -w` CLI flags removed**
8. **`Int.fromString`/`Float.fromString` API changes** — no explicit radix arg
9. **`js-post-build` behaviour changed** — now passes correct output paths

**Migration path:** Complete all v12 migration FIRST, then test against v13-alpha.

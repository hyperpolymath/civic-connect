// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <jonathan.jewell@open.ac.uk>
//
// Minimal Deno runtime bindings for ReScript.

@val @scope("Deno") external args: array<string> = "args"
@val @scope("Deno") external readTextFileSync: string => string = "readTextFileSync"
@val @scope("Deno") external exit: int => unit = "exit"

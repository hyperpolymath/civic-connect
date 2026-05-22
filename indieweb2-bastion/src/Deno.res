// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// Minimal Deno runtime bindings for ReScript.

@val @scope("Deno") external args: array<string> = "args"
@val @scope("Deno") external readTextFileSync: string => string = "readTextFileSync"
@val @scope("Deno") external exit: int => unit = "exit"

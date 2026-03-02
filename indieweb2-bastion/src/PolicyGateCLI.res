// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <jonathan.jewell@open.ac.uk>
//
// CLI entry point for policy-gate validation (Deno runtime).
// Usage: deno run --allow-read src/PolicyGateCLI.res.mjs <policy.json>

let readFile = (path: string): option<string> =>
  try Some(Deno.readTextFileSync(path))
  catch {
  | _ => None
  }

let exitWith = (code: int, msg: string) => {
  Console.error(msg)
  Deno.exit(code)
}

let () = {
  let args = Deno.args
  switch args->Array.get(0) {
  | None => exitWith(2, "Usage: deno run --allow-read PolicyGateCLI.res.mjs <policy.json>")
  | Some(path) =>
    switch readFile(path) {
    | None => exitWith(2, "Unable to read file: " ++ path)
    | Some(s) =>
      switch JSON.parseExn(s) {
      | exception _ => exitWith(2, "Invalid JSON")
      | parsed =>
        switch PolicyDecode.decodePolicy(parsed) {
        | None => exitWith(2, "Policy shape invalid — check required fields")
        | Some(policy) =>
          let errs = PolicyGate.validateAll(policy)
          if errs->Array.length == 0 {
            Console.log("Policy validation OK")
          } else {
            Console.log("Policy validation FAILED:")
            errs->Array.forEachWithIndex((e, i) =>
              Console.log("  " ++ Int.toString(i + 1) ++ ". " ++ e)
            )
            exitWith(1, Int.toString(Array.length(errs)) ++ " error(s)")
          }
        }
      }
    }
  }
}

# SPDX-License-Identifier: PMPL-1.0-or-later
# Nickel: command/spec matrix for multi-shell generation (consumed by Deno JS)
let Command = { name | string, description | string, steps | list string }
let ShellTarget = { shell | string, ext | string, shebang | string, template | string }
let ScriptSet = {
  version | string,
  commands | list Command,
  targets | list ShellTarget,
  templates | {
    bash | string, zsh | string, nushell | string, powershell | string, cmd | string,
    ksh | string, tcsh | string, csh | string, fish | string, ash | string, dash | string,
    oil | string, murex | string, ion | string, ngs | string
  }
}

let scripts : ScriptSet = {
  version = "0.1.0",
  commands = [
    { name = "export-snapshot", description = "SurrealDB → JSON-LD → IPFS", steps = [
      "surreal export snapshot > surrealdb/snapshot.json",
      "ipfs add surrealdb/snapshot.json"
    ]},
    { name = "rotate-keys", description = "Rotate bastion/resolver/SDP keys", steps = ["scripts/rotate-keys.sh"]},
    { name = "validate", description = "Policy + schema validations", steps = [
      "nickel check policy/curps/policy.ncl",
      "capnp compile -oc++ surrealdb/provenance.capnp",
      "surrealdb validate surrealdb/schema.surql"
    ]}
  ],
  targets = [
    { shell = "bash", ext = ".sh", shebang = "#!/usr/bin/env bash", template = "bash" },
    { shell = "zsh", ext = ".zsh", shebang = "#!/usr/bin/env zsh", template = "zsh" },
    { shell = "nushell", ext = ".nu", shebang = "#!/usr/bin/env nu", template = "nushell" },
    { shell = "powershell", ext = ".ps1", shebang = "#!/usr/bin/env pwsh", template = "powershell" },
    { shell = "cmd", ext = ".bat", shebang = "", template = "cmd" },
    { shell = "ksh", ext = ".ksh", shebang = "#!/usr/bin/env ksh", template = "ksh" },
    { shell = "tcsh", ext = ".tcsh", shebang = "#!/usr/bin/env tcsh", template = "tcsh" },
    { shell = "csh", ext = ".csh", shebang = "#!/usr/bin/env csh", template = "csh" },
    { shell = "fish", ext = ".fish", shebang = "#!/usr/bin/env fish", template = "fish" },
    { shell = "ash", ext = ".ash", shebang = "#!/usr/bin/env ash", template = "bash" },
    { shell = "dash", ext = ".dash", shebang = "#!/usr/bin/env dash", template = "bash" },
    { shell = "oil", ext = ".oil", shebang = "#!/usr/bin/env oil", template = "bash" },
    { shell = "murex", ext = ".mx", shebang = "#!/usr/bin/env murex", template = "bash" },
    { shell = "ion", ext = ".ion", shebang = "#!/usr/bin/env ion", template = "bash" },
    { shell = "ngs", ext = ".ngs", shebang = "#!/usr/bin/env ngs", template = "bash" }
  ],
  templates = {
    bash = "$shebang\nset -e\n# $name - $description\n$steps\n",
    zsh = "$shebang\nset -e\n# $name - $description\n$steps\n",
    nushell = "$shebang\n# $name - $description\n$steps\n",
    powershell = "$shebang\n# $name - $description\n$steps\n",
    cmd = "REM $name - $description\n$steps\n",
    ksh = "$shebang\nset -e\n# $name - $description\n$steps\n",
    tcsh = "$shebang\n# $name - $description\n$steps\n",
    csh = "$shebang\n# $name - $description\n$steps\n",
    fish = "$shebang\n# $name - $description\n$steps\n"
  }
}
export scripts

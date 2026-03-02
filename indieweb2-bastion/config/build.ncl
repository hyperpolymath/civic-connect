# SPDX-License-Identifier: PMPL-1.0-or-later
# Build Configuration for Indieweb2 Bastion
# Multi-chain smart contract platform: Solidity + Vyper + Motoko + ReScript

{
  # Project metadata
  project = {
    name = "indieweb2-bastion",
    version = "0.1.0",
    description = "Multi-chain smart contract platform with decentralized identity",
    license = "MIT OR Palimpsest-0.8.0",
  },

  # Solidity configuration
  solidity = {
    version = "0.8.24",
    compiler = "solc",

    settings = {
      optimizer = {
        enabled = true,
        runs = 200,
      },
      evm_version = "paris",
      metadata = {
        use_literal_content = true,
        bytecode_hash = "ipfs",
      },
      output_selection = {
        "*" = {
          "*" = ["abi", "evm.bytecode", "evm.deployedBytecode", "metadata"],
        },
      },
    },

    paths = {
      sources = "contracts/solidity",
      artifacts = "artifacts/solidity",
      cache = "cache/solidity",
    },
  },

  # Vyper configuration
  vyper = {
    version = "0.3.10",
    compiler = "vyper",

    settings = {
      optimize = "gas",
      evm_version = "paris",
    },

    paths = {
      sources = "contracts/vyper",
      artifacts = "artifacts/vyper",
    },
  },

  # Motoko configuration (Internet Computer)
  motoko = {
    version = "0.10.4",
    compiler = "moc",

    settings = {
      package_base = ".vessel",
      actor_idl_path = "idl",
    },

    paths = {
      sources = "src/motoko",
      artifacts = ".dfx/local/canisters",
    },
  },

  # ReScript configuration (frontend)
  rescript = {
    version = "11.0.1",
    compiler = "rescript",

    settings = {
      sources = ["src/frontend"],
      suffix = ".res",
      jsx = { version = 4 },
    },

    paths = {
      sources = "src/frontend",
      lib = "lib",
    },
  },

  # Node.js/Deno configuration
  javascript = {
    runtime | [| 'node, 'deno, 'bun |] = 'deno,
    node_version = "20.11.1",
    deno_version = "1.43.0",

    build = {
      bundle = true,
      minify = false,
      target = "es2022",
    },
  },

  # Build outputs
  outputs = {
    solidity_artifacts = "artifacts/solidity",
    vyper_artifacts = "artifacts/vyper",
    motoko_canisters = ".dfx/local/canisters",
    rescript_lib = "lib",
    frontend_dist = "dist",

    combined_abis = "abis",
    deployment_info = "deployments",
  },

  # Build tools integration
  tools = {
    hardhat = {
      enabled = true,
      config = "hardhat.config.js",
    },

    foundry = {
      enabled = true,
      config = "foundry.toml",
    },

    dfx = {
      enabled = true,
      config = "dfx.json",
      network = "local",
    },
  },

  # Compiler warnings
  warnings = {
    as_errors = false,
    solidity_pedantic = true,
  },

  # Build environment
  env_vars = {
    SOLC_VERSION = "0.8.24",
    VYPER_VERSION = "0.3.10",
    NODE_ENV = "production",
  },

  # Build hooks
  hooks = {
    pre_compile = ["just validate"],
    post_compile = ["just generate-types"],
  },

  # Validation
  validation = {
    enabled = true,
    run_tests = true,
    security_scan = true,
  },
}

# SPDX-License-Identifier: PMPL-1.0-or-later
# CI/CD Configuration for Indieweb2 Bastion
# Multi-chain smart contract CI pipeline

{
  # Pipeline metadata
  pipeline = {
    name = "indieweb2-bastion-ci",
    version = "1.0.0",
    description = "Multi-chain smart contract CI/CD",
  },

  # GitLab CI configuration
  gitlab = {
    stages = [
      "preflight",
      "lint",
      "build",
      "test",
      "security",
      "integration",
      "deploy",
      "release",
    ],

    variables = {
      NODE_VERSION = "20",
      DENO_VERSION = "1.43.0",
      SOLIDITY_VERSION = "0.8.24",
      PYTHON_VERSION = "3.11",
    },

    cache = {
      node = {
        key = "node-$CI_COMMIT_REF_SLUG",
        paths = ["node_modules/", ".npm/"],
      },
      deno = {
        key = "deno-$CI_COMMIT_REF_SLUG",
        paths = [".deno/"],
      },
    },
  },

  # Build configuration
  build = {
    parallel = true,
    timeout_minutes = 30,

    solidity = {
      enabled = true,
      command = "just solidity-build",
      compiler = "hardhat",
    },

    vyper = {
      enabled = true,
      command = "just vyper-build",
    },

    motoko = {
      enabled = true,
      command = "dfx build",
    },

    rescript = {
      enabled = true,
      command = "npx rescript build",
    },
  },

  # Testing
  test = {
    unit = {
      enabled = true,
      solidity_command = "npx hardhat test",
      motoko_command = "dfx test",
    },

    integration = {
      enabled = true,
      command = "just test-integration",
      network = "testnet",
    },

    gas_analysis = {
      enabled = true,
      report = true,
    },
  },

  # Security scanning
  security = {
    slither = {
      enabled = true,
      command = "slither .",
      fail_on_high = true,
    },

    mythril = {
      enabled = true,
      command = "myth analyze contracts/solidity/*.sol",
    },

    deps_audit = {
      enabled = true,
      npm_audit = true,
    },
  },

  # Deployment
  deploy = {
    testnet = {
      enabled = true,
      auto_deploy = true,
      networks = ["sepolia", "mumbai"],
    },

    mainnet = {
      enabled = true,
      auto_deploy = false,
      manual_approval = true,
      networks = ["ethereum", "polygon", "ic"],
    },

    local = {
      enabled = true,
      command = "just deploy-local",
    },
  },

  # Documentation
  docs = {
    enabled = true,
    contract_docs = true,
    natspec = true,
  },
}

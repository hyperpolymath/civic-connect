# SPDX-License-Identifier: PMPL-1.0-or-later
# Development Environment Configuration for Indieweb2 Bastion
# Multi-chain smart contract development settings

{
  environment = {
    name = "indieweb2-bastion-dev",
    version = "1.0.0",
    description = "Local development for multi-chain smart contracts",
  },

  # Tool versions
  tools = {
    nodejs = { version = "20.11.1", install_via = "asdf", required = true },
    deno = { version = "1.43.0", install_via = "asdf", required = true },
    solidity = { version = "0.8.24", install_via = "asdf", required = true },
    python = { version = "3.11.8", install_via = "asdf", required = true },
    just = { version = "1.25.2", install_via = "asdf", required = true },
    nickel = { version = "1.4.0", install_via = "asdf", required = true },
  },

  # Development dependencies
  dev_dependencies = {
    solidity = ["hardhat", "foundry", "slither", "mythril"],
    python = ["vyper", "eth-brownie"],
    motoko = ["dfx", "vessel"],
    frontend = ["rescript", "vite"],
  },

  # Local blockchain networks
  networks = {
    hardhat = {
      enabled = true,
      port = 8545,
      chain_id = 31337,
      auto_mine = true,
    },

    ganache = {
      enabled = false,
      port = 8546,
    },

    dfx_local = {
      enabled = true,
      port = 8000,
      replica_port = 4943,
    },
  },

  # Testing
  testing = {
    watch_mode = true,
    coverage = true,
    gas_reporting = true,
  },

  # Environment variables
  env_vars = {
    NODE_ENV = "development",
    HARDHAT_NETWORK = "localhost",
    DFX_NETWORK = "local",
  },
}

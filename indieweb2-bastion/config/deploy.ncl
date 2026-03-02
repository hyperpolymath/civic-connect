# SPDX-License-Identifier: PMPL-1.0-or-later
# Deployment Configuration for Indieweb2 Bastion
# Multi-chain smart contract deployment

{
  deployment = {
    name = "indieweb2-bastion-deploy",
    version = "1.0.0",
    description = "Multi-chain deployment configuration",
  },

  # EVM-compatible chains (Ethereum, Polygon, etc.)
  evm_chains = {
    ethereum = {
      mainnet = {
        enabled = false,
        rpc_url = "https://mainnet.infura.io/v3/YOUR_KEY",
        chain_id = 1,
        gas_price_strategy = "medium",
        approval_required = true,
      },
      sepolia = {
        enabled = true,
        rpc_url = "https://sepolia.infura.io/v3/YOUR_KEY",
        chain_id = 11155111,
        gas_price_strategy = "fast",
      },
    },

    polygon = {
      mainnet = {
        enabled = false,
        rpc_url = "https://polygon-rpc.com",
        chain_id = 137,
        approval_required = true,
      },
      mumbai = {
        enabled = true,
        rpc_url = "https://rpc-mumbai.maticvigil.com",
        chain_id = 80001,
      },
    },
  },

  # Internet Computer
  internet_computer = {
    mainnet = {
      enabled = false,
      network = "ic",
      approval_required = true,
    },
    local = {
      enabled = true,
      network = "local",
      replica_port = 4943,
    },
  },

  # Deployment strategy
  strategy = {
    verify_contracts = true,
    save_deployment_info = true,
    generate_abis = true,
  },

  # Contract verification
  verification = {
    etherscan = {
      enabled = false,
      api_key = null,
    },
    sourcify = {
      enabled = true,
    },
  },

  # Post-deployment
  post_deploy = {
    run_smoke_tests = true,
    update_frontend = true,
    notify_team = false,
  },
}

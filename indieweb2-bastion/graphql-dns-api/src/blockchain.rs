// SPDX-License-Identifier: PMPL-1.0-or-later
//! Blockchain integration for DNS record provenance anchoring
//!
//! Supports:
//! - Ethereum mainnet and testnets (Sepolia)
//! - Polygon mainnet and testnets (Amoy)

use crate::error::{AppError, Result};
use ethers::{
    core::types::{Transaction, TransactionReceipt, TransactionRequest, TxHash, U256},
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
};
use std::sync::Arc;

/// Blockchain client for anchoring DNS record hashes
pub struct BlockchainClient {
    network: String,
    provider: Arc<Provider<Http>>,
    wallet: Option<LocalWallet>,
}

impl BlockchainClient {
    /// Create a new blockchain client
    pub fn new(network: &str) -> Result<Self> {
        // Get RPC URL from environment
        let rpc_url = match network {
            "ethereum" => std::env::var("ETHEREUM_RPC_URL")
                .unwrap_or_else(|_| "https://eth.llamarpc.com".to_string()),
            "sepolia" => std::env::var("SEPOLIA_RPC_URL")
                .unwrap_or_else(|_| "https://rpc.sepolia.org".to_string()),
            "polygon" => std::env::var("POLYGON_RPC_URL")
                .unwrap_or_else(|_| "https://polygon-rpc.com".to_string()),
            "polygon-amoy" => std::env::var("POLYGON_AMOY_RPC_URL")
                .unwrap_or_else(|_| "https://rpc-amoy.polygon.technology".to_string()),
            _ => {
                return Err(AppError::Blockchain(format!(
                    "Unsupported network: {}",
                    network
                )))
            }
        };

        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| AppError::Blockchain(e.to_string()))?;

        // Load wallet from environment (optional for read-only operations)
        let wallet = std::env::var("PRIVATE_KEY")
            .ok()
            .and_then(|key| key.parse::<LocalWallet>().ok());

        Ok(Self {
            network: network.to_string(),
            provider: Arc::new(provider),
            wallet,
        })
    }

    /// Anchor a content hash to the blockchain
    ///
    /// This creates a transaction with the hash embedded in the data field,
    /// providing immutable provenance without requiring a smart contract.
    pub async fn anchor_hash(&self, hash: &str) -> Result<(String, i64)> {
        let wallet = self
            .wallet
            .as_ref()
            .ok_or_else(|| AppError::Blockchain("No wallet configured".to_string()))?
            .clone();

        let chain_id = self
            .provider
            .get_chainid()
            .await
            .map_err(|e| AppError::Blockchain(e.to_string()))?;

        let wallet = wallet.with_chain_id(chain_id.as_u64());
        let client = SignerMiddleware::new(self.provider.clone(), wallet);

        // Get current gas price
        let gas_price = client
            .get_gas_price()
            .await
            .map_err(|e| AppError::Blockchain(e.to_string()))?;

        // Create transaction with hash in data field
        // Format: 0x + "DNS:" + hex(hash)
        let data_prefix = "DNS:";
        let data = format!("0x{}{}", hex::encode(data_prefix), hash);

        let tx = TransactionRequest::new()
            .to(client.address()) // Send to self
            .value(U256::zero()) // No ETH transfer
            .data(hex::decode(&data[2..]).map_err(|e| AppError::Blockchain(format!("Invalid hex data: {}", e)))?)
            .gas_price(gas_price);

        // Send transaction
        let pending_tx = client
            .send_transaction(tx, None)
            .await
            .map_err(|e| AppError::Blockchain(format!("Failed to send transaction: {}", e)))?;

        let tx_hash = format!("{:?}", pending_tx.tx_hash());

        // Wait for confirmation
        let receipt = pending_tx
            .await
            .map_err(|e| AppError::Blockchain(format!("Transaction failed: {}", e)))?
            .ok_or_else(|| AppError::Blockchain("No transaction receipt".to_string()))?;

        let block_number = receipt
            .block_number
            .ok_or_else(|| AppError::Blockchain("No block number in receipt".to_string()))?
            .as_u64() as i64;

        Ok((tx_hash, block_number))
    }

    /// Verify a content hash on the blockchain
    pub async fn verify_hash(&self, tx_hash: &str, expected_hash: &str) -> Result<bool> {
        let tx: Transaction = self
            .provider
            .get_transaction(tx_hash.parse::<TxHash>().unwrap())
            .await
            .map_err(|e| AppError::Blockchain(e.to_string()))?
            .ok_or_else(|| AppError::Blockchain("Transaction not found".to_string()))?;

        // Extract hash from transaction data
        let data = tx.input;
        let data_str = hex::encode(data.to_vec());

        // Remove "DNS:" prefix
        let prefix_len = "DNS:".len() * 2; // hex encoded
        if data_str.len() <= prefix_len {
            return Ok(false);
        }

        let stored_hash = &data_str[prefix_len..];
        Ok(stored_hash == expected_hash)
    }

    /// Get transaction receipt
    pub async fn get_receipt(&self, tx_hash: &str) -> Result<TransactionReceipt> {
        let receipt = self
            .provider
            .get_transaction_receipt(tx_hash.parse::<TxHash>().unwrap())
            .await
            .map_err(|e| AppError::Blockchain(e.to_string()))?
            .ok_or_else(|| AppError::Blockchain("Transaction receipt not found".to_string()))?;

        Ok(receipt)
    }

    /// Get current block number
    pub async fn get_block_number(&self) -> Result<i64> {
        let block = self
            .provider
            .get_block_number()
            .await
            .map_err(|e| AppError::Blockchain(e.to_string()))?;

        Ok(block.as_u64() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_blockchain_client_creation() {
        // This test just verifies the client can be created with default RPC URLs
        let client = BlockchainClient::new("sepolia");
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_unsupported_network() {
        let client = BlockchainClient::new("invalid-network");
        assert!(client.is_err());
    }
}

use std::sync::Arc;
use std::collections::HashMap;
use std::time::Duration;
use eyre::Result;
use tokio::time::timeout;
use url::Url;
use serde::{Serialize, Deserialize};
use thiserror::Error;

// alloy imports
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::client::RpcClient;
use alloy::transports::http::Http;
use alloy::primitives::{Address, B256, U256, BlockNumber};


pub struct BlockParser {
    provider: Box<dyn alloy::providers::Provider + 'static>,
}

pub struct ParserConfig {
    timeout_seconds: u64,
    max_batch_size: usize,
    retry_attempts: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockAnalysis {
    pub block_number: u64,
    pub block_hash: B256,
    pub timestamp: u64,
    pub miner: Address,
    pub transaction_count: usize,
    pub gas_used: u64,
    pub gas_limit: u64,
    pub total_value: U256,
    pub transactions: Vec<TransactionAnalysis>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionAnalysis {
    pub hash: B256,
    pub from: Address,
    pub to: Option<Address>,
    pub value: U256,
    pub transaction_type: TransactionType,
    pub gas_used: u64,
    pub gas_price: U256,
    pub status: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum TransactionType {
    Transfer,
    ContractCreation,
    ContractCall,
    Unknown,
}

#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error("RPC error: {0}")]
    RpcError(String),
    
    #[error("Block not found: {0}")]
    BlockNotFound(String),
    
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Timeout error")]
    TimeoutError,
}

impl BlockParser {
    pub async fn new(rpc_url: &str) -> Result<Self, ParserError> {
        // parse the URL
        let url = rpc_url.parse()
            .map_err(|e| ParserError::ConnectionError(format!("Invalid URL: {}", e)))?;
        
        // HTTP transport
        let http = Http::new(url);
        
        // new RPC client
        let rpc_client = RpcClient::new(http, false);
        
        // provider with the HTTP transport
        let provider = ProviderBuilder::new().on_client(rpc_client);
        
        // box the provider to store it
        let provider_boxed = Box::new(provider);
        
        // parser instance
        let parser = Self { provider: provider_boxed };
        
        // test connection
        match parser.get_block_number().await {
            Ok(_) => Ok(parser),
            Err(e) => Err(e),
        }
    }
    
    pub async fn get_block_number(&self) -> Result<u64, ParserError> {
        self.provider
            .get_block_number()
            .await
            .map_err(|e| ParserError::RpcError(e.to_string()))
    }
}
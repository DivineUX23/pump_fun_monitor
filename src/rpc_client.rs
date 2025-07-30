//! # Solana RPC Client
//!
//! This module handles the connection to Solana's RPC WebSocket endpoint and monitors the pump.fun program for token creation events. It processes transactions in real-time and extracts relevant token metadata for broadcasting to connected clients.

use crate::data_models::{BondingCurveAccountData, CreateInstructionData, PumpFunData, TokenCreatedEvent, TokenDetails};
use crate::error::{MonitorError, Result};
use borsh::BorshDeserialize;
use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info, warn};
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::{RpcTransactionConfig, RpcSendTransactionConfig}};
use solana_program::instruction::Instruction;
use solana_program::program_pack::Pack;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature};
use solana_transaction_status::{EncodedTransactionWithStatusMeta, UiTransactionEncoding};
use spl_token::state::Mint;
use std::{str::FromStr, sync::Arc, time::Duration};
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

/// 8-byte prefix identifies token creation transactions.
const PUMP_FUN_CREATE_DISCRIMINATOR: [u8; 8] = [0x61, 0x21, 0xdf, 0x27, 0x22, 0x30, 0x04, 0x2f];

/// identify and parse bonding curve account data.
const BONDING_CURVE_DISCRIMINATOR: [u8; 8] = [0x68, 0x93, 0x5a, 0x56, 0x57, 0x5a, 0x0d, 0x73];


/// Main monitor struct that handles Solana RPC connections and pump.fun event processing.
///
pub struct SolanaRpcMonitor {
    rpc_client: Arc<RpcClient>,
    wss_url: String,
    pump_fun_program_id: Pubkey,
    event_sender: broadcast::Sender<TokenCreatedEvent>,
}

impl SolanaRpcMonitor {
    /// Creates a new Solana RPC monitor instance.
    ///
    pub fn new(
        http_url: String,
        wss_url: String,
        pump_fun_program_id: String,
        event_sender: broadcast::Sender<TokenCreatedEvent>,
    ) -> Result<Self> {
        let rpc_client = Arc::new(RpcClient::new_with_commitment(
            http_url,
            CommitmentConfig::confirmed(),
        ));
        let pump_fun_program_id =
            Pubkey::from_str(&pump_fun_program_id).map_err(|_| MonitorError::PubkeyParse)?;

        Ok(Self {
            rpc_client,
            wss_url,
            pump_fun_program_id,
            event_sender,
        })
    }

    pub async fn start(&self) {
        info!("Starting Solana monitor...");
        loop {
            if let Err(e) = self.connect_and_monitor().await {
                error!("Monitor task failed: {}. Reconnecting in 5 seconds...", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }

    async fn connect_and_monitor(&self) -> Result<()> {
        let (ws_stream, _) = connect_async(&self.wss_url).await?;
        info!("Connected to Solana WebSocket at {}", self.wss_url);

        let (mut write, mut read) = ws_stream.split();
        let subscription_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "logsSubscribe",
            "params": [
                { "mentions": [self.pump_fun_program_id.to_string()] },
                { "encoding": "jsonParsed", "commitment": "confirmed" }
            ]
        });

        write.send(Message::Text(subscription_request.to_string())).await?;
        info!("Subscribed to logs mentioning program: {}", self.pump_fun_program_id);

        let (tx_processor, mut rx_processor) = mpsc::channel::<Signature>(100);

        // a separate task for processing transactions to not block the WebSocket reader
        let rpc_client_clone = self.rpc_client.clone();
        let event_sender_clone = self.event_sender.clone();
        let pump_fun_id_clone = self.pump_fun_program_id;
        tokio::spawn(async move {
            while let Some(signature) = rx_processor.recv().await {
                match process_transaction(rpc_client_clone.clone(), signature, pump_fun_id_clone).await {
                    Ok(Some(event)) => {
                        info!("Successfully processed token creation: '{}' ({})", event.token.name, event.token.symbol);
                        if event_sender_clone.send(event).is_err() {
                            warn!("No active listeners for token creation events.");
                        }
                    }
                    Ok(None) => { /* Not a token creation tx */ }
                    Err(e) => warn!("Failed to process transaction {}: {}", signature, e),
                }
            }
        });

        // Main WebSocket reader loop
        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    if let Ok(log_notification) = serde_json::from_str::<serde_json::Value>(&text) {
                        if log_notification["params"]["result"]["value"]["err"].is_null() {
                            if let Some(signature_str) = log_notification["params"]["result"]["value"]["signature"].as_str() {
                                if let Ok(signature) = Signature::from_str(signature_str) {
                                    if tx_processor.send(signature).await.is_err() {
                                        error!("Transaction processing channel is closed.");
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    warn!("WebSocket connection closed by server.");
                    break;
                }
                Err(e) => {
                    error!("WebSocket read error: {}", e);
                    break;
                }
                _ => {}
            }
        }
        Err(MonitorError::WebSocket(tokio_tungstenite::tungstenite::Error::ConnectionClosed))
    }
}

async fn process_transaction(
    rpc_client: Arc<RpcClient>,
    signature: Signature,
    pump_fun_program_id: Pubkey,
) -> Result<Option<TokenCreatedEvent>> {
    let config = RpcTransactionConfig {
        encoding: Some(UiTransactionEncoding::Base64),
        commitment: Some(CommitmentConfig::confirmed()),
        max_supported_transaction_version: Some(0),
    };
    
    // retry logic for fetching the transaction
    let mut attempts = 0;
    let tx_meta = loop {
        match rpc_client.get_transaction_with_config(&signature, config).await {
            Ok(tx) => break tx,
            Err(e) if attempts < 3 => {
                attempts += 1;
                warn!(
                    "Attempt {} to fetch transaction {} failed: {}. Retrying...",
                    attempts, signature, e
                );
                tokio::time::sleep(Duration::from_millis(500 * attempts)).await;
            }
            Err(e) => return Err(MonitorError::RpcClient(e)),
        }
    };

    let Some(transaction) = tx_meta.transaction.transaction.decode() else {
        return Err(MonitorError::TransactionParse("Failed to decode transaction".to_string()));
    };

    let Some(meta) = tx_meta.transaction.meta else {
        return Err(MonitorError::TransactionParse("Transaction metadata missing".to_string()));
    };

    let account_keys = transaction.message.static_account_keys();

    for instruction in transaction.message.instructions() {
        if account_keys[instruction.program_id_index as usize] != pump_fun_program_id {
            continue;
        }

        if instruction.data.starts_with(&PUMP_FUN_CREATE_DISCRIMINATOR) {
            let parsed_instruction = CreateInstructionData::deserialize(&mut &instruction.data[8..])?;

            let creator = account_keys[0].to_string(); // fee payer is the creator
            //let mint_address = account_keys[instruction.accounts[0] as usize].to_string();
            
            let mint_address = account_keys[instruction.accounts[0] as usize];
            let bonding_curve_address = account_keys[instruction.accounts[4] as usize];

            let (mint_info_result, bonding_curve_info_result) = tokio::join!(
                get_mint_info(rpc_client.clone(), &mint_address),
                get_bonding_curve_info(rpc_client.clone(), &bonding_curve_address)
            );

            let (supply, decimals) = mint_info_result?;
            let bonding_curve_data = bonding_curve_info_result?;

            //let (supply, decimals) = get_mint_info(rpc_client.clone(), &mint_address).await?;

            let event = TokenCreatedEvent {
                event_type: "tokenCreated".to_string(),
                timestamp: chrono::Utc::now(),
                transaction_signature: signature.to_string(),
                token: TokenDetails {
                    mint_address: mint_address.to_string(),
                    name: parsed_instruction.name,
                    symbol: parsed_instruction.symbol,
                    uri: parsed_instruction.uri,
                    creator,
                    supply,
                    decimals,
                },
                pump_data: PumpFunData {                
                    bonding_curve: bonding_curve_address.to_string(),
                    virtual_sol_reserves: bonding_curve_data.virtual_sol_reserves,
                    virtual_token_reserves: bonding_curve_data.virtual_token_reserves,                
                },
            };
            return Ok(Some(event));
        }
    }

    Ok(None)
}


async fn get_mint_info(rpc_client: Arc<RpcClient>, mint_address: &Pubkey) -> Result<(u64, u8)> {    

    // let account = rpc_client.get_account(mint_address).await?;    
    // retry logic for fetching account data
    let mut attempts = 0;
    let account = loop {
        match rpc_client.get_account(mint_address).await {
            Ok(account) => break account,
            Err(e) if attempts < 3 => {
                attempts += 1;
                warn!(
                    "Attempt {} to fetch mint info for {} failed: {}. Retrying...",
                    attempts, mint_address, e
                );
                tokio::time::sleep(Duration::from_millis(500 * attempts)).await;
            }
            Err(e) => return Err(MonitorError::RpcClient(e)),
        }
    };
    let mint_data =
        Mint::unpack(&account.data).map_err(|e| MonitorError::TransactionParse(e.to_string()))?;

    Ok((mint_data.supply, mint_data.decimals))
}


async fn get_bonding_curve_info(
    rpc_client: Arc<RpcClient>,
    bonding_curve_address: &Pubkey,
) -> Result<BondingCurveAccountData> {
    let account = rpc_client.get_account(bonding_curve_address).await?;
    let mut account_data = &account.data[..];

    if account_data.len() < 8 || !account_data.starts_with(&BONDING_CURVE_DISCRIMINATOR) {
        return Err(MonitorError::TransactionParse(
            "Account is not a valid bonding curve account".to_string(),
        ));
    }
    
    // deserialize the rest of the data
    let curve_data = BondingCurveAccountData::deserialize(&mut &account_data[8..])?;
    Ok(curve_data)
}
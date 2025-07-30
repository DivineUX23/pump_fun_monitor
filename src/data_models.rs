//! # Data Models
//! This module defines the data structures used throughout the pump.fun monitor service.


use borsh::BorshDeserialize;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The main event structure broadcast to WebSocket clients when a new token is created.
///
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenCreatedEvent {
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub transaction_signature: String,
    pub token: TokenDetails,
    pub pump_data: PumpFunData,
}

/// detailed information about a newly created token.
///
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenDetails {
    pub mint_address: String,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub creator: String,
    pub supply: u64,
    pub decimals: u8,
}

/// pump.fun specific data extracted from the bonding curve and transaction.
///
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PumpFunData {
    pub bonding_curve: String,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
}


/// raw bonding curve account data structure for Borsh deserialization.
///
#[derive(BorshDeserialize, Debug)]
pub struct BondingCurveAccountData {
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
}

/// instruction data for pump.fun's Create instruction.
///
#[derive(BorshDeserialize, Debug)]
pub struct CreateInstructionData {
    pub name: String,
    pub symbol: String,
    pub uri: String,
}


/// client-side filtering criteria for token creation events.
///
#[derive(Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FilterCriteria {
    pub creator: Option<String>,
    pub symbol: Option<String>,
    pub name_contains: Option<String>,
}

/// messages that clients can send to the WebSocket server.
///
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase", tag = "action")]
pub enum ClientMessage {
    SetFilter {
        filter: FilterCriteria
    },
}
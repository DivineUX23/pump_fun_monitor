//! # Pump.fun Token Monitor
//!
//! A real-time monitoring service for pump.fun token creation events on Solana.
//!
mod data_models;
mod error;
mod rpc_client;
mod websocket_server;

use dotenv::dotenv;
use log::info;
use rpc_client::SolanaRpcMonitor;
use std::env;
use tokio::sync::broadcast;

/// Main entry point for the pump.fun token monitor service.
///
/// This function:
/// 1. Loads configuration from environment variables
/// 2. Sets up logging
/// 3. Creates a broadcast channel for token events
/// 4. Spawns the RPC monitor and WebSocket server tasks
/// 5. Runs both tasks concurrently until one exits
#[tokio::main]
async fn main() {
    dotenv().ok();

    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    info!("Starting pump.fun monitor service...");

    // load configuration from environment variables
    let http_url = env::var("SOLANA_RPC_HTTP_URL").expect("SOLANA_RPC_HTTP_URL must be set");
    let wss_url = env::var("SOLANA_RPC_WSS_URL").expect("SOLANA_RPC_WSS_URL must be set");
    let pump_fun_id = env::var("PUMP_FUN_PROGRAM_ID").expect("PUMP_FUN_PROGRAM_ID must be set");
    let ws_port = env::var("WEBSOCKET_SERVER_PORT")
        .expect("WEBSOCKET_SERVER_PORT must be set")
        .parse::<u16>()
        .expect("Invalid WebSocket port number");

    let (tx, rx) = broadcast::channel(100);

    let monitor = SolanaRpcMonitor::new(http_url, wss_url, pump_fun_id, tx)
        .expect("Failed to create Solana Monitor");

    let monitor_handle = tokio::spawn(async move {
        monitor.start().await;
    });

    let ws_addr = format!("127.0.0.1:{}", ws_port);
    let server_handle = tokio::spawn(async move {
        if let Err(e) = websocket_server::start_websocket_server(&ws_addr, rx).await {
            log::error!("WebSocket server error: {}", e);
        }
    });

    // run both tasks concurrently until one exits
    tokio::select! {
        _ = monitor_handle => info!("Solana RPC monitor task exited."),
        _ = server_handle => info!("WebSocket server task exited."),
    }
}
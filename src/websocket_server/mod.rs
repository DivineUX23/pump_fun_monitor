//! WebSocket server module for broadcasting pump.fun token creation events.
//!
//! # architecture
//! the server maintains a list of connected clients, each with their own filter criteria.
//! when a token creation event is received, it's checked against each client's filter and only sent to clients where the event matches their criteria.

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, Mutex};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use log::{info, warn, error};
use serde_json;

use crate::data_models::{TokenCreatedEvent, FilterCriteria, ClientMessage};

type ClientTx = tokio::sync::mpsc::UnboundedSender<Message>;


/// each client maintains its own connection state and filter criteria,
struct Client {
    addr: SocketAddr,
    tx: ClientTx,
    filter: Arc<Mutex<FilterCriteria>>,
}

/// starts the WebSocket server and handles client connections.
///
/// # arguments
/// * `addr` - the address to bind the server to (e.g., "127.0.0.1:8080")
/// * `mut event_receiver` - broadcast receiver for token creation events
///
/// # returns
/// * `Result<(), Box<dyn std::error::Error>>` - Ok if server starts successfully
pub async fn start_websocket_server(
    addr: &str,
    mut event_receiver: broadcast::Receiver<TokenCreatedEvent>,
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(addr).await?;
    info!("🚀 WebSocket server listening on {}", addr);

    let clients: Arc<Mutex<Vec<Arc<Client>>>> = Arc::new(Mutex::new(Vec::new()));
    let broadcast_clients = Arc::clone(&clients);

    tokio::spawn(async move {
        loop {
            match event_receiver.recv().await {
                Ok(event) => {
                    let mut dead_clients = Vec::new();
                    let locked_clients = broadcast_clients.lock().await;

                    for client in locked_clients.iter() {
                        let filter = client.filter.lock().await;
                        if matches_filter(&event, &filter) {
                            let event_json = serde_json::to_string(&event).unwrap();
                            let message = Message::Text(event_json);
                            
                            if let Err(_) = client.tx.send(message) {
                                dead_clients.push(client.addr);
                            }
                        }
                    }

                    // remove dead clients outside
                    drop(locked_clients);
                    if !dead_clients.is_empty() {
                        let mut locked_clients = broadcast_clients.lock().await;
                        locked_clients.retain(|client| !dead_clients.contains(&client.addr));
                        for addr in dead_clients {
                            info!("Removed dead client: {}", addr);
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    warn!("WebSocket broadcast lagged, skipped {} events", skipped);
                }
                Err(broadcast::error::RecvError::Closed) => {
                    error!("Event broadcast channel closed");
                    break;
                }
            }
        }
    });

    // accept incoming connections
    while let Ok((stream, addr)) = listener.accept().await {
        let clients_clone = Arc::clone(&clients);
        tokio::spawn(handle_connection(stream, addr, clients_clone));
    }

    Ok(())
}

/// handles a single WebSocket client connection.
///
/// # Arguments
/// * `stream` - the TCP stream for the client connection
/// * `addr` - the client's socket address
/// * `clients` - shared list of connected clients
async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    clients: Arc<Mutex<Vec<Arc<Client>>>>,
) {
    info!("New client connected: {}", addr);

    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!("Failed to accept WebSocket connection from {}: {}", addr, e);
            return;
        }
    };

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    let client = Arc::new(Client {
        addr,
        tx,
        filter: Arc::new(Mutex::new(FilterCriteria::default())),
    });

    clients.lock().await.push(Arc::clone(&client));

    let client_for_sender = Arc::clone(&client);
    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if let Err(e) = ws_sender.send(message).await {
                error!("Failed to send message to {}: {}", client_for_sender.addr, e);
                break;
            }
        }
    });

    // handle incoming messages
    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Try to parse as a client message
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(ClientMessage::SetFilter { filter }) => {
                        let mut client_filter = client.filter.lock().await;
                        *client_filter = filter.clone();
                        info!("Updated filter for client {}: {:?}", addr, filter);
                    }
                    Err(e) => {
                        warn!("Invalid message from client {}: {} (error: {})", addr, text, e);
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("Client {} sent close message", addr);
                break;
            }
            Err(e) => {
                error!("WebSocket error for client {}: {}", addr, e);
                break;
            }
            _ => {
            }
        }
    }

    info!("Client {} disconnected", addr);
    // Remove the client from the broadcast list
    clients.lock().await.retain(|client| client.addr != addr);
}


/// Checks if a token creation event matches the specified filter criteria.
fn matches_filter(event: &TokenCreatedEvent, filter: &FilterCriteria) -> bool {
    // check creator filter
    if let Some(creator_filter) = &filter.creator {
        if &event.token.creator != creator_filter {
            return false;
        }
    }
    
    // check symbol filter
    if let Some(symbol_filter) = &filter.symbol {
        if event.token.symbol.to_uppercase() != symbol_filter.to_uppercase() {
            return false;
        }
    }
    
    // check name contains filter
    if let Some(name_filter) = &filter.name_contains {
        if !event.token.name.to_uppercase().contains(&name_filter.to_uppercase()) {
            return false;
        }
    }
    
    true
}

#[cfg(test)]
mod tests;

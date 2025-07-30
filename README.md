# Pump.fun Token Monitor

A real-time Rust-based WebSocket service that monitors the Solana blockchain for pump.fun token creation events and streams them to connected clients.

## 🚀 Live Demo

**Test the service live on Render:**
- **WebSocket URL:** `wss://pump-fun-monitor-latest.onrender.com`
- **Test with:** Any WebSocket client or the examples below

Try connecting to the live service to see real-time pump.fun token creation events!

## Features

- 🔗 **Real-time Solana RPC monitoring** via WebSocket connection
- 🎯 **Pump.fun contract tracking** for token creation events
- 📡 **WebSocket server** for broadcasting events to multiple clients
- 🔍 **Client-side filtering** - Each client can set custom filters for creator, symbol, or name patterns
- 🔄 **Dynamic filter updates** - Clients can update their filters in real-time without reconnecting
- 🔄 **Automatic reconnection** with robust error handling
- 📊 **Structured JSON output** with comprehensive token metadata
- ⚡ **Async/concurrent processing** for high performance

## Quick Start

### Prerequisites

- Rust 1.70+ installed
- Access to Solana RPC endpoints (HTTP + WebSocket)

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd solana
```

2. Install dependencies:
```bash
cargo build
```

3. Configure environment variables:
```bash
cp .env.example .env
# Edit .env with your RPC endpoints
```

4. Run the service:
```bash
cargo run
```

The service will start monitoring pump.fun and serve WebSocket clients on the configured port.

## Configuration

Create a `.env` file in the project root:

```env
# Solana RPC endpoints
SOLANA_RPC_HTTP_URL="https://api.mainnet-beta.solana.com"
SOLANA_RPC_WSS_URL="wss://api.mainnet-beta.solana.com"

# WebSocket server configuration
WEBSOCKET_SERVER_PORT=8080

# Pump.fun program ID (should not change)
PUMP_FUN_PROGRAM_ID="6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P"
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SOLANA_RPC_HTTP_URL` | Solana HTTP RPC endpoint | Required |
| `SOLANA_RPC_WSS_URL` | Solana WebSocket RPC endpoint | Required |
| `WEBSOCKET_SERVER_PORT` | Port for WebSocket server | Required |
| `PUMP_FUN_PROGRAM_ID` | Pump.fun program address | `6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P` |

## Usage

### Starting the Service

```bash
# Development mode with logs
RUST_LOG=info cargo run

# Production mode
cargo run --release
```

### Connecting Clients

Connect to the WebSocket server to receive real-time token creation events:

#### Local Development
```javascript
const ws = new WebSocket('ws://localhost:8080');

ws.onopen = () => {
    // Optional: Set up filtering to only receive specific tokens
    const filterMessage = {
        action: "setFilter",
        filter: {
            symbol: "DOGE",           // Only DOGE tokens
            nameContains: "moon"      // Tokens with "moon" in the name
        }
    };
    ws.send(JSON.stringify(filterMessage));
};

ws.onmessage = (event) => {
    const tokenEvent = JSON.parse(event.data);
    console.log('New token created:', tokenEvent);
};
```

#### Live Production Service
```javascript
const ws = new WebSocket('wss://pump-fun-monitor-latest.onrender.com');

ws.onopen = () => {
    console.log('Connected to live pump.fun monitor!');
    // Optional: Set up filtering to only receive specific tokens
    const filterMessage = {
        action: "setFilter",
        filter: {
            symbol: "DOGE",           // Only DOGE tokens
            nameContains: "moon"      // Tokens with "moon" in the name
        }
    };
    ws.send(JSON.stringify(filterMessage));
};

ws.onmessage = (event) => {
    const tokenEvent = JSON.parse(event.data);
    console.log('New token created:', tokenEvent);
};

ws.onerror = (error) => {
    console.error('WebSocket error:', error);
};

ws.onclose = (event) => {
    console.log('Connection closed:', event.code, event.reason);
};
```

#### Testing with curl (Health Check)
```bash
# Test if the service is running
curl -I https://pump-fun-monitor-latest.onrender.com

# Note: The service is primarily a WebSocket server, so HTTP requests
# will not return data, but a successful connection indicates the service is running
```

### Client-Side Filtering

Each client can set custom filters to receive only the events they're interested in:

#### Filter Options
- **`creator`**: Exact match for token creator address (case-sensitive)
- **`symbol`**: Exact match for token symbol (case-insensitive)
- **`nameContains`**: Partial match for token name (case-insensitive)

#### Filter Examples

```javascript
// Filter for tokens created by a specific address
ws.send(JSON.stringify({
    action: "setFilter",
    filter: {
        creator: "DEF456ghi789JKL012mno345PQR678stu901VWX234yza567BCD890efg123"
    }
}));

// Filter for PEPE tokens with "moon" in the name
ws.send(JSON.stringify({
    action: "setFilter",
    filter: {
        symbol: "PEPE",
        nameContains: "moon"
    }
}));

// Clear all filters (receive all events)
ws.send(JSON.stringify({
    action: "setFilter",
    filter: {}
}));
```

### Sample Output

When a new token is created on pump.fun, clients receive events in this format:

```json
{
  "eventType": "tokenCreated",
  "timestamp": "2024-01-15T10:30:45Z",
  "transactionSignature": "5x7K8mN9pQ2rS3tU4vW6xY7zA8bC9dE0fG1hI2jK3lM4nO5pQ6rS7tU8vW9xY0zA",
  "token": {
    "mintAddress": "ABC123def456GHI789jkl012MNO345pqr678STU901vwx234YZA567bcd890",
    "name": "MyAwesomeToken",
    "symbol": "MAT",
    "uri": "https://example.com/metadata.json",
    "creator": "DEF456ghi789JKL012mno345PQR678stu901VWX234yza567BCD890efg123",
    "supply": 1000000000,
    "decimals": 6
  },
  "pumpData": {
    "bondingCurve": "GHI789jkl012MNO345pqr678STU901vwx234YZA567bcd890EFG123hij456",
    "virtualSolReserves": 30000000000,
    "virtualTokenReserves": 1073000000000000
  }
}
```

## Architecture

The service consists of several key components:

### Core Modules

- **`main.rs`** - Application entry point and service orchestration
- **`rpc_client.rs`** - Solana RPC connection and transaction monitoring
- **`websocket_server.rs`** - WebSocket server for client connections
- **`data_models.rs`** - Data structures and serialization models
- **`error.rs`** - Error handling and custom error types

### Data Flow

1. **RPC Monitor** connects to Solana WebSocket and subscribes to pump.fun program logs
2. **Transaction Processor** fetches and parses transactions containing token creation events
3. **Event Broadcaster** sends structured events to all connected WebSocket clients
4. **Client Handler** manages individual client connections and message routing

## API Reference

### WebSocket Events

#### Token Creation Event

Sent when a new token is created on pump.fun.

**Event Type:** `tokenCreated`

**Payload:**
- `eventType` (string) - Always "tokenCreated"
- `timestamp` (ISO 8601) - Event timestamp
- `transactionSignature` (string) - Solana transaction signature
- `token` (object) - Token details
  - `mintAddress` (string) - Token mint address
  - `name` (string) - Token name
  - `symbol` (string) - Token symbol
  - `uri` (string) - Metadata URI
  - `creator` (string) - Creator wallet address
  - `supply` (number) - Total token supply
  - `decimals` (number) - Token decimal places
- `pumpData` (object) - Pump.fun specific data
  - `bondingCurve` (string) - Bonding curve account address
  - `virtualSolReserves` (number) - Virtual SOL reserves
  - `virtualTokenReserves` (number) - Virtual token reserves

## Development

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Check for errors without building
cargo check
```

### Testing

The project includes comprehensive unit tests for filtering functionality and core components.

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test websocket_server::tests

# Run tests in release mode (faster)
cargo test --release

# Run with backtrace for debugging
RUST_BACKTRACE=1 cargo test
```

#### **Test Coverage**

The test suite covers:

✅ **Filtering Logic Tests**
- Empty filter matches all events
- Creator address filtering (exact match, case-sensitive)
- Symbol filtering (exact match, case-insensitive)
- Name contains filtering (partial match, case-insensitive)
- Multiple criteria filtering (AND logic)
- Edge cases (empty strings, special characters)
- Real-world scenarios (DOGE, PEPE tokens)

✅ **Test Structure**
```
src/
├── websocket_server/
│   ├── mod.rs           # Main WebSocket server implementation
│   └── tests.rs         # Comprehensive filtering tests
├── data_models.rs       # Data structures and serialization
├── rpc_client.rs        # Solana RPC client logic
└── error.rs            # Error handling
```

#### **Writing New Tests**

When adding new filtering features:

```rust
#[test]
fn test_new_filter_feature() {
    let event = create_test_event("creator", "Token Name", "SYM");
    let filter = FilterCriteria {
        new_field: Some("test_value".to_string()),
        ..Default::default()
    };
    assert!(matches_filter(&event, &filter));
}
```

### Logging

The service uses `env_logger`. Set the `RUST_LOG` environment variable to control log levels:

```bash
# Info level (recommended for production)
RUST_LOG=info cargo run

# Debug level (for development)
RUST_LOG=debug cargo run

# Module-specific logging
RUST_LOG=pump_fun_monitor_corrected::rpc_client=debug cargo run
```

## Troubleshooting

### Common Issues

**Connection Errors:**
- Verify RPC endpoints are accessible
- Check firewall settings for WebSocket connections
- Ensure sufficient RPC rate limits

**Rate Limiting:**
```
WARN Failed to process transaction: RPC client error: HTTP status client error (429 Too Many Requests)
```
- Use a paid RPC provider (Helius, QuickNode, Alchemy)
- Implement request throttling
- Add retry logic with exponential backoff

**WebSocket Disconnections:**
```
ERROR WebSocket read error: IO error: An existing connection was forcibly closed
```
- Normal behavior - the service automatically reconnects
- Monitor logs for successful reconnection messages

### Performance Tuning

- Use dedicated RPC endpoints for production
- Adjust channel buffer sizes in `main.rs`
- Monitor memory usage with high client counts
- Consider horizontal scaling for high throughput



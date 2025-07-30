# Usage Examples

This document provides practical examples of how to use the pump.fun token monitor service.

## Basic Setup and Running

### 1. Environment Configuration

Create a `.env` file:

```bash
# Required: Solana RPC endpoints
SOLANA_RPC_HTTP_URL="https://api.mainnet-beta.solana.com"
SOLANA_RPC_WSS_URL="wss://api.mainnet-beta.solana.com"

# Required: WebSocket server port
WEBSOCKET_SERVER_PORT=8080

# Optional: Pump.fun program ID (default provided)
PUMP_FUN_PROGRAM_ID="6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P"
```

### 2. Starting the Service

```bash
# Development mode with detailed logging
RUST_LOG=debug cargo run

# Production mode
RUST_LOG=info cargo run --release

# Background service
nohup cargo run --release > pump_monitor.log 2>&1 &
```

## Client Examples

### Simple Event Logger with Filtering (JavaScript)

```html
<!DOCTYPE html>
<html>
<head>
    <title>Pump.fun Token Monitor</title>
    <style>
        .filter-controls { margin: 20px 0; padding: 10px; background: #f5f5f5; }
        .filter-controls input { margin: 5px; padding: 5px; }
        .event { border: 1px solid #ddd; margin: 10px 0; padding: 10px; }
    </style>
</head>
<body>
    <h1>Live Token Creations</h1>

    <div class="filter-controls">
        <h3>Filter Settings</h3>
        <input type="text" id="creatorFilter" placeholder="Creator address">
        <input type="text" id="symbolFilter" placeholder="Symbol (e.g., DOGE)">
        <input type="text" id="nameFilter" placeholder="Name contains (e.g., moon)">
        <button onclick="updateFilter()">Apply Filter</button>
        <button onclick="clearFilter()">Clear Filter</button>
    </div>

    <div id="events"></div>

    <script>
        const ws = new WebSocket('ws://localhost:8080');
        const eventsDiv = document.getElementById('events');

        ws.onopen = () => {
            console.log('Connected to pump.fun monitor');
            // Start with no filter
            clearFilter();
        };

        ws.onmessage = (event) => {
            const tokenEvent = JSON.parse(event.data);

            const eventDiv = document.createElement('div');
            eventDiv.className = 'event';
            eventDiv.innerHTML = `
                <h3>${tokenEvent.token.name} (${tokenEvent.token.symbol})</h3>
                <p><strong>Creator:</strong> ${tokenEvent.token.creator}</p>
                <p><strong>Supply:</strong> ${tokenEvent.token.supply.toLocaleString()}</p>
                <p><strong>Transaction:</strong> <a href="https://solscan.io/tx/${tokenEvent.transactionSignature}" target="_blank">${tokenEvent.transactionSignature.substring(0, 20)}...</a></p>
                <p><strong>Time:</strong> ${new Date(tokenEvent.timestamp).toLocaleString()}</p>
            `;

            eventsDiv.insertBefore(eventDiv, eventsDiv.firstChild);

            // Keep only last 50 events
            while (eventsDiv.children.length > 50) {
                eventsDiv.removeChild(eventsDiv.lastChild);
            }
        };

        function updateFilter() {
            const filter = {};

            const creator = document.getElementById('creatorFilter').value.trim();
            const symbol = document.getElementById('symbolFilter').value.trim();
            const nameContains = document.getElementById('nameFilter').value.trim();

            if (creator) filter.creator = creator;
            if (symbol) filter.symbol = symbol;
            if (nameContains) filter.nameContains = nameContains;

            const filterMessage = {
                action: "setFilter",
                filter: filter
            };

            ws.send(JSON.stringify(filterMessage));
            console.log('Filter applied:', filter);
        }

        function clearFilter() {
            document.getElementById('creatorFilter').value = '';
            document.getElementById('symbolFilter').value = '';
            document.getElementById('nameFilter').value = '';

            const filterMessage = {
                action: "setFilter",
                filter: {}
            };

            ws.send(JSON.stringify(filterMessage));
            console.log('Filter cleared');
        }
    </script>
</body>
</html>
```

### Token Alert Bot with Dynamic Filtering (Node.js)

```javascript
const WebSocket = require('ws');
const fs = require('fs');

class TokenAlertBot {
    constructor() {
        this.ws = null;
        this.reconnectInterval = 5000;
        this.alertCriteria = {
            minSupply: 1000000,
            maxSupply: 1000000000,
            symbolPattern: /^[A-Z]{3,5}$/
        };
        this.currentFilter = {};
    }

    connect() {
        this.ws = new WebSocket('ws://localhost:8080');

        this.ws.on('open', () => {
            console.log('🟢 Connected to pump.fun monitor');
            this.setInitialFilter();
        });

        this.ws.on('message', (data) => {
            try {
                const event = JSON.parse(data.toString());
                this.processEvent(event);
            } catch (error) {
                console.error('❌ Error parsing event:', error);
            }
        });

        this.ws.on('close', () => {
            console.log('🔴 Disconnected. Reconnecting in 5 seconds...');
            setTimeout(() => this.connect(), this.reconnectInterval);
        });

        this.ws.on('error', (error) => {
            console.error('❌ WebSocket error:', error);
        });
    }

    processEvent(event) {
        if (event.eventType !== 'tokenCreated') return;

        const token = event.token;
        
        // Apply filtering criteria
        if (this.shouldAlert(token)) {
            this.sendAlert(event);
            this.logEvent(event);
        }
    }

    shouldAlert(token) {
        return (
            token.supply >= this.alertCriteria.minSupply &&
            token.supply <= this.alertCriteria.maxSupply &&
            this.alertCriteria.symbolPattern.test(token.symbol)
        );
    }

    sendAlert(event) {
        const token = event.token;
        const alert = `
🚨 NEW TOKEN ALERT 🚨
Name: ${token.name}
Symbol: ${token.symbol}
Creator: ${token.creator}
Supply: ${token.supply.toLocaleString()}
Transaction: https://solscan.io/tx/${event.transactionSignature}
        `;
        
        console.log(alert);
        
        // Send to Discord, Telegram, email, etc.
        // this.sendToDiscord(alert);
        // this.sendToTelegram(alert);
    }

    logEvent(event) {
        const logEntry = {
            timestamp: event.timestamp,
            token: event.token,
            transaction: event.transactionSignature
        };

        fs.appendFileSync('token_alerts.jsonl', JSON.stringify(logEntry) + '\n');
    }

    setInitialFilter() {
        // Set up server-side filtering to reduce network traffic
        this.currentFilter = {
            // Only tokens with reasonable supply ranges
            // Note: Additional client-side filtering still applies
        };

        const filterMessage = {
            action: "setFilter",
            filter: this.currentFilter
        };

        this.ws.send(JSON.stringify(filterMessage));
        console.log('🔍 Initial filter applied:', this.currentFilter);
    }

    updateFilter(newFilter) {
        this.currentFilter = { ...this.currentFilter, ...newFilter };

        const filterMessage = {
            action: "setFilter",
            filter: this.currentFilter
        };

        this.ws.send(JSON.stringify(filterMessage));
        console.log('🔄 Filter updated:', this.currentFilter);
    }

    // Method to focus on specific creators
    watchCreator(creatorAddress) {
        this.updateFilter({ creator: creatorAddress });
        console.log(`👀 Now watching creator: ${creatorAddress}`);
    }

    // Method to focus on specific token patterns
    watchTokenPattern(symbolPattern, namePattern) {
        const filter = {};
        if (symbolPattern) filter.symbol = symbolPattern;
        if (namePattern) filter.nameContains = namePattern;

        this.updateFilter(filter);
        console.log(`🎯 Now watching pattern - Symbol: ${symbolPattern}, Name: ${namePattern}`);
    }

    // Method to clear all filters
    clearFilters() {
        this.currentFilter = {};
        this.updateFilter({});
        console.log('🧹 All filters cleared');
    }
}

// Start the bot
const bot = new TokenAlertBot();
bot.connect();

// Example usage of dynamic filtering
setTimeout(() => {
    // After 30 seconds, focus on DOGE tokens
    bot.watchTokenPattern('DOGE', null);
}, 30000);

setTimeout(() => {
    // After 60 seconds, watch for tokens with "moon" in the name
    bot.watchTokenPattern(null, 'moon');
}, 60000);

setTimeout(() => {
    // After 90 seconds, watch a specific creator
    bot.watchCreator('DEF456ghi789JKL012mno345PQR678stu901VWX234yza567BCD890efg123');
}, 90000);
```

### Data Collector with Filtering (Python)

```python
import asyncio
import websockets
import json
import sqlite3
from datetime import datetime

class TokenDataCollector:
    def __init__(self, db_path='tokens.db', filter_config=None):
        self.db_path = db_path
        self.filter_config = filter_config or {}
        self.setup_database()
    
    def setup_database(self):
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS tokens (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mint_address TEXT UNIQUE,
                name TEXT,
                symbol TEXT,
                creator TEXT,
                supply INTEGER,
                decimals INTEGER,
                transaction_signature TEXT,
                bonding_curve TEXT,
                virtual_sol_reserves INTEGER,
                virtual_token_reserves INTEGER,
                created_at TIMESTAMP,
                processed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        ''')
        
        conn.commit()
        conn.close()
    
    def save_token(self, event):
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        token = event['token']
        pump_data = event['pumpData']
        
        try:
            cursor.execute('''
                INSERT OR REPLACE INTO tokens (
                    mint_address, name, symbol, creator, supply, decimals,
                    transaction_signature, bonding_curve, virtual_sol_reserves,
                    virtual_token_reserves, created_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ''', (
                token['mintAddress'],
                token['name'],
                token['symbol'],
                token['creator'],
                token['supply'],
                token['decimals'],
                event['transactionSignature'],
                pump_data['bondingCurve'],
                pump_data['virtualSolReserves'],
                pump_data['virtualTokenReserves'],
                event['timestamp']
            ))
            
            conn.commit()
            print(f"✅ Saved token: {token['name']} ({token['symbol']})")
            
        except sqlite3.Error as e:
            print(f"❌ Database error: {e}")
        finally:
            conn.close()
    
    async def collect_data(self):
        uri = "ws://localhost:8080"

        while True:
            try:
                async with websockets.connect(uri) as websocket:
                    print("🟢 Connected to pump.fun monitor")

                    # Apply filter if configured
                    if self.filter_config:
                        filter_message = {
                            "action": "setFilter",
                            "filter": self.filter_config
                        }
                        await websocket.send(json.dumps(filter_message))
                        print(f"🔍 Filter applied: {self.filter_config}")

                    async for message in websocket:
                        event = json.loads(message)

                        if event['eventType'] == 'tokenCreated':
                            self.save_token(event)
                            
            except websockets.exceptions.ConnectionClosed:
                print("🔴 Connection closed. Reconnecting in 5 seconds...")
                await asyncio.sleep(5)
            except Exception as e:
                print(f"❌ Error: {e}")
                await asyncio.sleep(5)

# Run the collector with different filter configurations

# Example 1: Collect all tokens
collector_all = TokenDataCollector()
# asyncio.run(collector_all.collect_data())

# Example 2: Collect only DOGE tokens
collector_doge = TokenDataCollector(
    db_path='doge_tokens.db',
    filter_config={'symbol': 'DOGE'}
)
# asyncio.run(collector_doge.collect_data())

# Example 3: Collect tokens from specific creator
collector_creator = TokenDataCollector(
    db_path='creator_tokens.db',
    filter_config={
        'creator': 'DEF456ghi789JKL012mno345PQR678stu901VWX234yza567BCD890efg123'
    }
)
# asyncio.run(collector_creator.collect_data())

# Example 4: Collect tokens with "moon" in the name
collector_moon = TokenDataCollector(
    db_path='moon_tokens.db',
    filter_config={'nameContains': 'moon'}
)
asyncio.run(collector_moon.collect_data())
```

## Sample Output

### Console Output (Service)

```
[2024-01-15T10:30:45Z INFO  pump_fun_monitor_corrected] Starting pump.fun monitor service...
[2024-01-15T10:30:45Z INFO  pump_fun_monitor_corrected::rpc_client] Connected to Solana WebSocket at wss://api.mainnet-beta.solana.com
[2024-01-15T10:30:45Z INFO  pump_fun_monitor_corrected::rpc_client] Subscribed to logs mentioning program: 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P
[2024-01-15T10:30:45Z INFO  pump_fun_monitor_corrected::websocket_server] WebSocket server listening on: ws://127.0.0.1:8080
[2024-01-15T10:30:50Z INFO  pump_fun_monitor_corrected::websocket_server] New WebSocket connection from: 127.0.0.1:54321
[2024-01-15T10:31:15Z INFO  pump_fun_monitor_corrected::rpc_client] Successfully processed token creation: 'MyAwesomeToken' (MAT)
```

### WebSocket Event (JSON)

```json
{
  "eventType": "tokenCreated",
  "timestamp": "2024-01-15T10:31:15.123Z",
  "transactionSignature": "5x7K8mN9pQ2rS3tU4vW6xY7zA8bC9dE0fG1hI2jK3lM4nO5pQ6rS7tU8vW9xY0zA",
  "token": {
    "mintAddress": "ABC123def456GHI789jkl012MNO345pqr678STU901vwx234YZA567bcd890",
    "name": "MyAwesomeToken",
    "symbol": "MAT",
    "uri": "https://arweave.net/abc123def456ghi789jkl012mno345pqr678stu901vwx234yza567bcd890",
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

## Testing Examples

### Running the Test Suite

The project includes comprehensive unit tests for the filtering functionality:

```bash
# Run all tests
cargo test

# Run only WebSocket server tests
cargo test websocket_server

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_filter_by_creator_exact_match
```

### Test Output Example

```
running 11 tests
test websocket_server::tests::test_filter_by_creator_exact_match ... ok
test websocket_server::tests::test_filter_by_creator_no_match ... ok
test websocket_server::tests::test_filter_by_multiple_criteria_all_match ... ok
test websocket_server::tests::test_filter_by_multiple_criteria_partial_match_fails ... ok
test websocket_server::tests::test_filter_by_name_contains_case_insensitive_match ... ok
test websocket_server::tests::test_filter_by_name_contains_no_match ... ok
test websocket_server::tests::test_filter_by_symbol_case_insensitive_match ... ok
test websocket_server::tests::test_filter_by_symbol_no_match ... ok
test websocket_server::tests::test_filter_edge_cases ... ok
test websocket_server::tests::test_filter_real_world_scenarios ... ok
test websocket_server::tests::test_no_filter_matches_all ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Manual Testing with WebSocket Clients

You can manually test the filtering functionality using any WebSocket client:

#### Using `wscat` (Node.js)

```bash
# Install wscat
npm install -g wscat

# Connect to the server
wscat -c ws://localhost:8080

# Send filter messages
{"action": "setFilter", "filter": {"symbol": "DOGE"}}
{"action": "setFilter", "filter": {"nameContains": "moon"}}
{"action": "setFilter", "filter": {}}
```

#### Using Browser Developer Console

```javascript
// Open browser console and connect
const ws = new WebSocket('ws://localhost:8080');

// Set up event handlers
ws.onmessage = (event) => {
    console.log('Received:', JSON.parse(event.data));
};

// Test different filters
ws.send(JSON.stringify({
    action: "setFilter",
    filter: { symbol: "PEPE" }
}));

// Clear filter
ws.send(JSON.stringify({
    action: "setFilter",
    filter: {}
}));
```

## Production Deployment

### Docker Deployment

```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/pump_fun_monitor_corrected /usr/local/bin/
COPY .env /app/.env

WORKDIR /app
EXPOSE 8080

CMD ["pump_fun_monitor_corrected"]
```

### Systemd Service

```ini
[Unit]
Description=Pump.fun Token Monitor
After=network.target

[Service]
Type=simple
User=pump-monitor
WorkingDirectory=/opt/pump-monitor
ExecStart=/opt/pump-monitor/pump_fun_monitor_corrected
Restart=always
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

### Monitoring Script

```bash
#!/bin/bash
# monitor.sh - Health check script

SERVICE_URL="ws://localhost:8080"
LOG_FILE="/var/log/pump-monitor.log"

# Check if service is responding
if ! nc -z localhost 8080; then
    echo "$(date): Service not responding, restarting..." >> $LOG_FILE
    systemctl restart pump-monitor
fi

# Check log for errors
if tail -n 100 $LOG_FILE | grep -q "ERROR"; then
    echo "$(date): Errors detected in logs" >> $LOG_FILE
    # Send alert notification
fi
```

## Troubleshooting

### Common Issues

1. **Rate Limiting**: Use paid RPC providers for production
2. **Memory Usage**: Monitor with high client counts
3. **Connection Drops**: Implement client-side reconnection
4. **Missing Events**: Check RPC endpoint reliability

### Performance Tuning

- Adjust broadcast channel buffer size
- Use dedicated RPC endpoints
- Implement client connection limits
- Monitor system resources

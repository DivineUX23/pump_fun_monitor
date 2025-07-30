# API Documentation

## WebSocket API

The pump.fun monitor service provides a WebSocket API for real-time token creation events. Clients connect to the WebSocket server and receive JSON-formatted events as they occur.

### Connection

**Endpoint:** `ws://localhost:8080` (or configured port)

**Protocol:** WebSocket (RFC 6455)

### Authentication

Currently, no authentication is required. All connected clients can receive events based on their filter settings.

### Client Messages

Clients can send messages to the server to configure their event filtering preferences.

#### Set Filter Message

Allows clients to configure which events they want to receive.

**Message Type:** `setFilter`

**Message Format:**
```json
{
  "action": "setFilter",
  "filter": {
    "creator": "DEF456ghi789JKL012mno345PQR678stu901VWX234yza567BCD890efg123",
    "symbol": "MAT",
    "nameContains": "Awesome"
  }
}
```

**Filter Fields (all optional):**
- `creator` - Exact match for token creator address
- `symbol` - Exact match for token symbol (case-insensitive)
- `nameContains` - Partial match for token name (case-insensitive)

**Notes:**
- All filter fields are optional - omit fields you don't want to filter by
- Filters are applied with AND logic (all specified criteria must match)
- Send an empty filter object `{}` to receive all events
- Filters are applied immediately and persist for the connection duration

### Events

#### Token Creation Event

Sent when a new token is created on pump.fun that matches the client's filter criteria.

**Event Type:** `tokenCreated`

**Message Format:**
```json
{
  "eventType": "tokenCreated",
  "timestamp": "2024-01-15T10:30:45.123Z",
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

### Field Descriptions

#### Root Level Fields

| Field | Type | Description |
|-------|------|-------------|
| `eventType` | string | Always "tokenCreated" for token creation events |
| `timestamp` | string | ISO 8601 timestamp when the event was processed |
| `transactionSignature` | string | Solana transaction signature (base58 encoded) |
| `token` | object | Token details object |
| `pumpData` | object | Pump.fun specific data object |

#### Token Object Fields

| Field | Type | Description |
|-------|------|-------------|
| `mintAddress` | string | SPL token mint address (base58 encoded public key) |
| `name` | string | Human-readable token name |
| `symbol` | string | Token symbol/ticker (usually 3-5 characters) |
| `uri` | string | URI pointing to token metadata JSON |
| `creator` | string | Wallet address of the token creator |
| `supply` | number | Total token supply in smallest unit (considering decimals) |
| `decimals` | number | Number of decimal places for the token |

#### PumpData Object Fields

| Field | Type | Description |
|-------|------|-------------|
| `bondingCurve` | string | Address of the bonding curve account |
| `virtualSolReserves` | number | Virtual SOL reserves in lamports |
| `virtualTokenReserves` | number | Virtual token reserves in token's smallest unit |

### Client Implementation Examples

#### JavaScript/Browser

```javascript
const ws = new WebSocket('ws://localhost:8080');

ws.onopen = () => {
    console.log('Connected to pump.fun monitor');

    // Set up a filter to only receive tokens with "DOGE" in the name
    const filterMessage = {
        action: "setFilter",
        filter: {
            nameContains: "DOGE"
        }
    };
    ws.send(JSON.stringify(filterMessage));
};

ws.onmessage = (event) => {
    const tokenEvent = JSON.parse(event.data);

    if (tokenEvent.eventType === 'tokenCreated') {
        console.log(`New token: ${tokenEvent.token.name} (${tokenEvent.token.symbol})`);
        console.log(`Creator: ${tokenEvent.token.creator}`);
        console.log(`Transaction: ${tokenEvent.transactionSignature}`);
    }
};

ws.onclose = () => {
    console.log('Disconnected from pump.fun monitor');
};

ws.onerror = (error) => {
    console.error('WebSocket error:', error);
};

// Function to update filter dynamically
function updateFilter(newFilter) {
    const filterMessage = {
        action: "setFilter",
        filter: newFilter
    };
    ws.send(JSON.stringify(filterMessage));
}

// Example: Filter for tokens created by a specific address
updateFilter({
    creator: "DEF456ghi789JKL012mno345PQR678stu901VWX234yza567BCD890efg123"
});
```

#### Node.js

```javascript
const WebSocket = require('ws');

const ws = new WebSocket('ws://localhost:8080');

ws.on('open', () => {
    console.log('Connected to pump.fun monitor');

    // Set up a filter for tokens with specific criteria
    const filterMessage = {
        action: "setFilter",
        filter: {
            symbol: "PEPE",  // Only PEPE tokens
            creator: "DEF456ghi789JKL012mno345PQR678stu901VWX234yza567BCD890efg123"
        }
    };
    ws.send(JSON.stringify(filterMessage));
});

ws.on('message', (data) => {
    const tokenEvent = JSON.parse(data.toString());

    if (tokenEvent.eventType === 'tokenCreated') {
        console.log(`New token: ${tokenEvent.token.name} (${tokenEvent.token.symbol})`);

        // Process the event (save to database, send notifications, etc.)
        processTokenCreation(tokenEvent);
    }
});

ws.on('close', () => {
    console.log('Disconnected from pump.fun monitor');
    // Implement reconnection logic here
});

function processTokenCreation(event) {
    // Your business logic here
    console.log('Processing token creation event:', event);
}

// Function to change filter based on conditions
function setDynamicFilter() {
    const newFilter = {
        nameContains: "moon"  // Look for tokens with "moon" in the name
    };

    const filterMessage = {
        action: "setFilter",
        filter: newFilter
    };
    ws.send(JSON.stringify(filterMessage));
}
```

#### Python

```python
import asyncio
import websockets
import json

async def listen_to_events():
    uri = "ws://localhost:8080"

    async with websockets.connect(uri) as websocket:
        print("Connected to pump.fun monitor")

        # Set up initial filter
        filter_message = {
            "action": "setFilter",
            "filter": {
                "symbol": "DOGE",
                "nameContains": "coin"
            }
        }
        await websocket.send(json.dumps(filter_message))

        async for message in websocket:
            event = json.loads(message)

            if event['eventType'] == 'tokenCreated':
                token = event['token']
                print(f"New token: {token['name']} ({token['symbol']})")
                print(f"Creator: {token['creator']}")
                print(f"Transaction: {event['transactionSignature']}")

async def dynamic_filtering_example():
    uri = "ws://localhost:8080"

    async with websockets.connect(uri) as websocket:
        print("Connected to pump.fun monitor")

        # Start with no filter (receive all events)
        await websocket.send(json.dumps({
            "action": "setFilter",
            "filter": {}
        }))

        # After 10 seconds, switch to filtering for specific creator
        await asyncio.sleep(10)
        await websocket.send(json.dumps({
            "action": "setFilter",
            "filter": {
                "creator": "DEF456ghi789JKL012mno345PQR678stu901VWX234yza567BCD890efg123"
            }
        }))

        async for message in websocket:
            event = json.loads(message)
            if event['eventType'] == 'tokenCreated':
                print(f"Filtered token: {event['token']['name']}")

# Run the client
asyncio.run(listen_to_events())
```

### Error Handling

#### Connection Errors

- **Connection Refused**: Server is not running or port is blocked
- **Connection Timeout**: Network issues or server overload
- **Connection Closed**: Normal disconnection or server restart

#### Message Errors

- **Invalid JSON**: Malformed message (should not occur with this service)
- **Missing Fields**: Event structure changed (check for service updates)


### Monitoring and Health

The service logs connection events and errors. Monitor the service logs for:
- Client connection/disconnection events
- Event broadcasting statistics
- Error conditions and recovery

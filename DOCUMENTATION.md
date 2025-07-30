# Documentation Overview

This document provides a comprehensive overview of the pump.fun token monitor service documentation.

## Documentation Structure

### Core Documentation Files

1. **[README.md](README.md)** - Main project documentation
   - Quick start guide
   - Installation instructions
   - Configuration details
   - Usage examples
   - Troubleshooting guide

2. **[API.md](API.md)** - WebSocket API reference
   - Connection details
   - Event formats and schemas
   - Field descriptions
   - Client implementation examples
   - Error handling

3. **[EXAMPLES.md](EXAMPLES.md)** - Practical usage examples
   - Basic setup and configuration
   - Client implementations in multiple languages
   - Production deployment examples
   - Monitoring and health checks

4. **[DOCUMENTATION.md](DOCUMENTATION.md)** - This overview file

### Inline Code Documentation

All source files include comprehensive inline documentation following Rust documentation standards:

#### Module-Level Documentation (`//!`)
- **`src/main.rs`** - Application entry point and service orchestration
- **`src/rpc_client.rs`** - Solana RPC connection and transaction monitoring
- **`src/websocket_server.rs`** - WebSocket server for client connections
- **`src/data_models.rs`** - Data structures and serialization models
- **`src/error.rs`** - Error handling and custom error types

#### Function and Struct Documentation (`///`)
- All public functions include parameter descriptions
- Return value documentation with error conditions
- Usage examples where appropriate
- Performance considerations and limitations

## Architecture Documentation

### System Overview

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Solana RPC    │    │  Monitor Service │    │  WebSocket      │
│   WebSocket     │◄──►│                  │◄──►│  Clients        │
│                 │    │  ┌─────────────┐ │    │                 │
└─────────────────┘    │  │ RPC Monitor │ │    └─────────────────┘
                       │  └─────────────┘ │
┌─────────────────┐    │  ┌─────────────┐ │    ┌─────────────────┐
│   Solana RPC    │    │  │ Event       │ │    │  Log Files      │
│   HTTP          │◄──►│  │ Processor   │ │◄──►│                 │
│                 │    │  └─────────────┘ │    └─────────────────┘
└─────────────────┘    │  ┌─────────────┐ │
                       │  │ WebSocket   │ │
                       │  │ Server      │ │
                       │  └─────────────┘ │
                       └──────────────────┘
```

### Data Flow

1. **RPC Monitor** subscribes to Solana WebSocket for pump.fun program logs
2. **Event Processor** fetches transaction details and parses token creation data
3. **WebSocket Server** broadcasts structured events to all connected clients
4. **Error Handler** manages connection issues and implements retry logic

### Key Components

#### SolanaRpcMonitor
- Manages WebSocket connection to Solana RPC
- Subscribes to logs mentioning pump.fun program
- Processes transactions asynchronously
- Implements automatic reconnection logic

#### WebSocket Server
- Accepts multiple concurrent client connections
- Supports per-client filtering with real-time filter updates
- Broadcasts filtered events to connected clients based on their criteria
- Handles client disconnections gracefully
- Maintains client connection registry with individual filter states

#### Data Models
- `TokenCreatedEvent` - Main event structure for clients
- `TokenDetails` - Comprehensive token metadata
- `PumpFunData` - Pump.fun specific bonding curve data
- `CreateInstructionData` - Raw instruction data parsing
- `FilterCriteria` - Client-side filtering configuration
- `ClientMessage` - Client-to-server message structure for filter updates

#### Error Handling
- Comprehensive error types for all failure scenarios
- Automatic retry logic for transient failures
- Graceful degradation for network issues
- Detailed logging for debugging

## API Documentation

### Client Messages

#### Set Filter Message
Clients can send filter configuration messages to customize which events they receive:

```json
{
  "action": "setFilter",
  "filter": {
    "creator": "DEF456ghi789JKL012mno345PQR678stu901VWX234yza567BCD890efg123",
    "symbol": "DOGE",
    "nameContains": "moon"
  }
}
```

**Filter Options:**
- `creator` - Exact match for token creator address
- `symbol` - Exact match for token symbol (case-insensitive)
- `nameContains` - Partial match for token name (case-insensitive)

All filter fields are optional. Filters use AND logic (all specified criteria must match).

### WebSocket Events

#### Token Creation Event
Sent when a new token is created that matches the client's filter criteria:

```json
{
  "eventType": "tokenCreated",
  "timestamp": "2024-01-15T10:30:45.123Z",
  "transactionSignature": "...",
  "token": {
    "mintAddress": "...",
    "name": "...",
    "symbol": "...",
    "uri": "...",
    "creator": "...",
    "supply": 1000000000,
    "decimals": 6
  },
  "pumpData": {
    "bondingCurve": "...",
    "virtualSolReserves": 30000000000,
    "virtualTokenReserves": 1073000000000000
  }
}
```

### Configuration

#### Environment Variables
- `SOLANA_RPC_HTTP_URL` - HTTP RPC endpoint
- `SOLANA_RPC_WSS_URL` - WebSocket RPC endpoint
- `WEBSOCKET_SERVER_PORT` - Server port
- `PUMP_FUN_PROGRAM_ID` - Program address to monitor

#### Logging
- Uses `env_logger` with configurable levels
- Structured logging with context information
- Performance metrics and error tracking

## Development Documentation

### Building and Testing

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Check code quality
cargo clippy
cargo fmt
```

### Code Quality Standards

- **Documentation**: All public APIs documented
- **Error Handling**: Comprehensive error types and handling
- **Testing**: Unit tests for core functionality
- **Logging**: Structured logging throughout
- **Performance**: Async/await patterns for concurrency

### Contributing Guidelines

1. Follow Rust naming conventions
2. Add documentation for all public APIs
3. Include unit tests for new functionality
4. Update relevant documentation files
5. Ensure code passes `cargo clippy` and `cargo fmt`

## Deployment Documentation

### Production Deployment

#### Docker
- Multi-stage build for optimized images
- Health checks and monitoring
- Environment variable configuration
- Log aggregation setup

#### Systemd Service
- Service file configuration
- Automatic restart on failure
- Log rotation and management
- Resource limits and monitoring

#### Monitoring
- Health check endpoints
- Performance metrics collection
- Error rate monitoring
- Connection count tracking

### Security Considerations

- No authentication currently implemented
- Rate limiting considerations
- Network security (firewall rules)
- Log sanitization for sensitive data

## Performance Documentation

### Benchmarks

- Connection handling capacity
- Event processing throughput
- Memory usage patterns
- CPU utilization under load

### Optimization Guidelines

- Use dedicated RPC endpoints for production
- Implement client connection limits
- Monitor memory usage with high client counts
- Consider horizontal scaling for high throughput

### Resource Requirements

- **Memory**: 50-100MB base + ~1MB per 100 clients
- **CPU**: Low usage, spikes during transaction processing
- **Network**: Dependent on RPC endpoint and client count
- **Storage**: Minimal, mainly for logs

## Troubleshooting Documentation

### Common Issues

1. **Rate Limiting**: RPC endpoint limitations
2. **Connection Drops**: Network instability
3. **Memory Leaks**: High client connection counts
4. **Missing Events**: RPC endpoint reliability

### Debugging

- Enable debug logging: `RUST_LOG=debug`
- Monitor connection status in logs
- Check RPC endpoint health
- Verify network connectivity

### Support

- Check logs for error messages
- Verify configuration settings
- Test RPC endpoint connectivity
- Monitor system resources
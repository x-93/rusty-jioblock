# JIO Blockchain Explorer Backend

This is the backend service for the JIO Blockchain Explorer, providing REST API endpoints, WebSocket real-time updates, and blockchain indexing capabilities.

## Features

- **REST API**: Comprehensive REST endpoints for blocks, transactions, addresses, and statistics
- **WebSocket Server**: Real-time updates for new blocks and transactions
- **Indexer Service**: Continuous indexing of blockchain data into SQLite
- **Database**: SQLite with optimized schema for blockchain data
- **Caching**: Redis integration for frequently accessed data

## Architecture

```
explorer/
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Library root
│   ├── api/                 # REST API server
│   │   ├── server.rs        # Axum server setup
│   │   ├── routes/          # API route handlers
│   │   └── handlers/        # Request handlers
│   ├── indexer/             # Blockchain indexer
│   │   ├── service.rs       # Main indexer service
│   │   ├── block_indexer.rs # Block indexing logic
│   │   ├── transaction_indexer.rs # Transaction indexing
│   │   └── address_indexer.rs # Address indexing
│   ├── database/            # Database layer
│   │   ├── connection.rs    # Database connection
│   │   ├── queries.rs       # SQL queries
│   │   └── schema.rs        # Schema definitions
│   ├── websocket/           # WebSocket server
│   │   ├── server.rs        # WS server implementation
│   │   └── subscriptions.rs # Subscription management
│   ├── models.rs            # Data models
│   ├── cache.rs             # Redis cache layer
│   └── error.rs             # Error types
├── migrations/              # Database migrations
│   └── 001_initial_schema.sql
└── Cargo.toml
```

## Setup

### Prerequisites

- Rust 1.70+
- SQLite (built-in with most systems)
- Redis 7+ (optional, for caching)

### Database Setup

The database is automatically created and migrated on first run. The default database file is `jio_explorer.db` in the current directory.

### Configuration

Set environment variables (optional):
- `DATABASE_URL`: SQLite connection string (default: `jio_explorer.db`)
- `REDIS_URL`: Redis connection string (optional)
- `EXPLORER_PORT`: API server port (default: 3001)
- `RUST_LOG`: Logging level (default: info)

### Running the Backend API

```bash
cd explorer
cargo run --bin jio-explorer
```

The API server will start on port 3001 by default.

### Running the Frontend

In a separate terminal:

```bash
cd explorer-frontend
npm install
npm run dev
```

The frontend will start on port 3000 by default and connect to the backend API on port 3001.

## API Endpoints

### Blocks
- `GET /api/v1/blocks` - List blocks (paginated)
- `GET /api/v1/blocks/:hash` - Get block by hash
- `GET /api/v1/blocks/height/:height` - Get block by height
- `GET /api/v1/blocks/recent` - Get recent blocks

### Transactions
- `GET /api/v1/transactions` - List transactions (paginated)
- `GET /api/v1/transactions/:hash` - Get transaction by hash
- `GET /api/v1/transactions/pending` - Get pending transactions

### Addresses
- `GET /api/v1/addresses/:address` - Get address summary
- `GET /api/v1/addresses/:address/transactions` - Get address transactions

### Statistics
- `GET /api/v1/stats/network` - Network statistics

### Search
- `GET /api/v1/search?q=:query` - Global search

## WebSocket

Connect to `ws://localhost:3000/ws` for real-time updates.

### Subscribe to channels:
```json
{
  "type": "subscribe",
  "channel": "blocks:new"
}
```

### Available channels:
- `blocks:new` - New blocks
- `transactions:new` - New transactions
- `mempool:updates` - Mempool updates
- `network:stats` - Network statistics

## Development

### Adding new endpoints

1. Add route in `src/api/routes/`
2. Add handler function
3. Register route in `src/api/server.rs`

### Adding new indexer logic

1. Create new indexer module in `src/indexer/`
2. Implement indexing logic
3. Register in `IndexerService`

## TODO

- [ ] Complete RPC coordinator integration
- [ ] Implement full transaction lookup from blocks
- [ ] Add block height indexing
- [ ] Implement search functionality
- [ ] Add rate limiting
- [ ] Add authentication for WebSocket
- [ ] Performance optimization
- [ ] Add comprehensive tests


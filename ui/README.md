# Zosh UI Web Service

A web service that subscribes to Zosh blocks via RPC, stores them in SQLite, and provides a REST API to query bridge transactions.

## Features

- **Block Subscription**: Automatically subscribes to new blocks from the Zosh RPC server
- **SQLite Storage**: Persists blocks, bridge transactions, and receipts in a local database
- **REST API**: Query bridge transactions by transaction ID

## Configuration

Configure the service using environment variables:

- `ZOSH_RPC_URL`: Zosh RPC WebSocket URL (default: `ws://localhost:9944`)
- `ZOSH_DB_PATH`: SQLite database file path (default: `./blocks.db`)
- `ZOSH_LISTEN_ADDR`: Web service bind address (default: `0.0.0.0:3000`)

## Building

```bash
cargo build --release
```

## Running

```bash
# With default configuration
cargo run --bin zosh-ui

# With custom configuration
ZOSH_RPC_URL=ws://localhost:9944 \
ZOSH_DB_PATH=/data/blocks.db \
ZOSH_LISTEN_ADDR=0.0.0.0:8080 \
cargo run --bin zosh-ui
```

## API Endpoints

### GET `/txid/{txid}`

Query a bridge transaction by its transaction ID.

**Parameters:**

- `txid`: Transaction ID in hex (32 bytes for Zcash) or base58 (64 bytes for Solana) encoding

**Response:**

```json
{
  "coin": "Zec",
  "amount": 12345678,
  "recipient": "base58-encoded-recipient-address",
  "source": "Zcash",
  "target": "Solana",
  "bundle_slot": 1234,
  "receipt": {
    "txid": "receipt-transaction-id",
    "slot": 1235
  }
}
```

**Error Responses:**

- `400 Bad Request`: Invalid txid format or length
- `404 Not Found`: Transaction not found
- `500 Internal Server Error`: Database error

**Example:**

```bash
# Query a Zcash transaction (32 bytes, hex encoded)
curl http://localhost:3000/txid/abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890

# Query a Solana transaction (64 bytes, base58 encoded)
curl http://localhost:3000/txid/5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4tjF3ZpRzrFmBV6UjKdiSZkQUW
```

## Architecture

1. **Subscriber Task**: Connects to Zosh RPC and receives blocks via WebSocket subscription
2. **Database Layer**: Stores blocks and transactions with proper indexing for efficient queries
3. **Web Server**: Axum-based HTTP server that queries the database

## Database Schema

### Tables

- `blocks`: Block headers with slot, parent, state, accumulator, extrinsic, and votes
- `bridges`: Bridge transactions with txid, coin, recipient, amount, source, target, and block slot
- `receipts`: Receipt transactions with txid, anchor (source txid), coin, source, target, and block slot

### Indexes

- `idx_bridges_block_slot`: Index on bridge transactions by block slot
- `idx_receipts_anchor`: Index on receipts by anchor (for matching with source transactions)
- `idx_receipts_block_slot`: Index on receipts by block slot

## Logging

Set the `RUST_LOG` environment variable to control logging verbosity:

```bash
RUST_LOG=ui=debug,axum=info cargo run --bin zosh-ui
```

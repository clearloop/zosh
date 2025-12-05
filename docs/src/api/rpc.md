# RPC API

JSON-RPC WebSocket API for querying zosh node state and subscribing to events.

## Connection

**Default endpoint:** `ws://localhost:9944`

```javascript
const WebSocket = require('ws');
const ws = new WebSocket('ws://localhost:9944');
```

## Methods

All methods are under the `zosh` namespace.

### `zosh_chainInfo`

Get current chain state including validators and BFT configuration.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "zosh_chainInfo",
  "params": [],
  "id": 1
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "validators": ["<pubkey1>", "<pubkey2>", ...],
    "threshold": 2,
    "height": 12345,
    "...": "..."
  },
  "id": 1
}
```

**Returns:**
- `validators`: List of validator Ed25519 public keys
- `threshold`: Number of validators required for BFT consensus
- `height`: Current block height
- Additional state data

### `zosh_subscribeBlock`

Subscribe to new block events.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "zosh_subscribeBlock",
  "params": [],
  "id": 1
}
```

**Response (initial):**
```json
{
  "jsonrpc": "2.0",
  "result": "<subscription_id>",
  "id": 1
}
```

**Notifications:**
```json
{
  "jsonrpc": "2.0",
  "method": "zosh_subscribeBlock",
  "params": {
    "subscription": "<subscription_id>",
    "result": {
      "block": "<hex_encoded_block_data>"
    }
  }
}
```

**Block data:**
- Encoded in postcard format (binary)
- Contains: header, extrinsics, BFT votes

### `zosh_subscribeTransaction`

Subscribe to a specific transaction status.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "zosh_subscribeTransaction",
  "params": ["<txid_hex>"],
  "id": 1
}
```

**Response (initial):**
```json
{
  "jsonrpc": "2.0",
  "result": "<subscription_id>",
  "id": 1
}
```

**Notifications:**
```json
{
  "jsonrpc": "2.0",
  "method": "zosh_subscribeTransaction",
  "params": {
    "subscription": "<subscription_id>",
    "result": "<tx_status_bytes>"
  }
}
```

## Error Handling

Errors follow JSON-RPC 2.0 specification:

```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32600,
    "message": "Invalid request"
  },
  "id": 1
}
```

## Usage

The RPC API is primarily used by:
- UI service for block indexing
- Collectors for submitting bridge requests
- External monitoring tools

See the UI service implementation for a complete example.

# zosh-rpc

JSON-RPC client and server for Zosh nodes.

## Overview

Provides JSON-RPC 2.0 interface for interacting with Zosh nodes:

- **Client**: Query node state, subscribe to blocks, query transactions
- **Server**: Expose node functionality via HTTP and WebSocket endpoints
- **Subscriptions**: Real-time block updates via WebSocket

## Features

- JSON-RPC 2.0 compliant API
- WebSocket subscriptions for block events
- HTTP transport for queries
- Type-safe request/response handling

## Usage

The RPC crate is used by:
- `zosh-node` to expose node functionality
- `zosh-ui` to subscribe to blocks and query state
- External clients to interact with Zosh network

See [RPC API](../../docs/src/api/rpc.md) for endpoint documentation.


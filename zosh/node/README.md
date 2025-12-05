# zosh-node

Zosh validator node implementation.

## Overview

The main node binary that runs Zosh validators. Combines consensus, runtime, and chain synchronization:

- **ZoshBFT consensus**: Participates in BFT voting and block production
- **Chain synchronization**: Monitors Solana and Zcash for bridge events
- **Block validation**: Verifies and signs blocks from other validators
- **State management**: Maintains blockchain state using parity-db

## Features

- VRF-based leader selection for block production
- Threshold signature aggregation for bridge bundles
- Embedded collectors for Solana and Zcash monitoring
- Optional UI and RPC server features

## Components

- **Runtime**: Block import, validation, and state management
- **Sync**: Chain collectors for bridge event detection
- **Storage**: Parity-db backend for persistent state
- **Network**: P2P communication with other validators

## Usage

```bash
# Run a validator node
cargo run -p zosh-node -- dev

# With RPC server
cargo run -p zosh-node --features rpc -- dev
```

See [Node CLI](../../docs/src/api/node.md) and [Validators](../../docs/src/validators.md) for detailed documentation.


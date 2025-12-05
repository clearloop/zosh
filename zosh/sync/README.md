# zosh-sync

Chain synchronization and bridge event collection.

## Overview

Monitors Solana and Zcash for bridge events and submits them to the mempool:

- **Solana collector**: Watches for zoZEC burn events via WebSocket subscriptions
- **Zcash collector**: Scans orchard shielded pool for ZEC deposits to bridge address
- **Bridge request creation**: Converts chain events into bridge transactions
- **FROST signing**: Threshold signature coordination for bridge bundles

## Components

- **SolanaClient**: Subscribes to Solana program logs, detects BurnEvent emissions
- **ZcashClient**: Light wallet scanning of orchard pool, decodes transaction memos
- **Bundle coordination**: Creates bridge bundles and aggregates threshold signatures

## Architecture

Collectors run as independent services that:

1. Monitor external chains (Solana/Zcash)
2. Detect bridge events (burns/deposits)
3. Create bridge requests with recipient addresses
4. Submit to local mempool for validator consensus

## Requirements

- Solana RPC endpoint (WebSocket subscription)
- Zcash lightwalletd server connection
- Bridge viewing keys for Zcash orchard scanning
- FROST threshold signing setup

See [Collectors](../../docs/src/collectors.md) and [Bridge](../../docs/src/bridge/) for detailed workflows.


# zosh-network

P2P networking layer for Zosh validators.

## Overview

Provides QUIC-based peer-to-peer communication for ZoshBFT consensus:

- **QUIC transport**: Low-latency UDP-based protocol with built-in TLS 1.3 encryption
- **Block gossip**: Propagation of new blocks across the validator network
- **Signature aggregation**: Collection of validator votes for consensus finality
- **Peer discovery**: Bootstrap and peer exchange for network connectivity

## Architecture

The network layer enables:

- **Block propagation**: Leaders broadcast blocks, validators verify and sign
- **Vote collection**: Validators exchange signatures until 2/3 threshold reached
- **State sync**: New validators download chain state from peers
- **Bundle coordination**: Bridge bundle creation and threshold signing

## Security

- TLS 1.3 encryption on all connections
- Ed25519 peer authentication
- Message validation and DoS protection

See [Network](../docs/src/protocol/network.md) for detailed protocol documentation.


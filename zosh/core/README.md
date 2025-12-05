# zosh-core

Core types and data structures for the Zosh network.

## Overview

Provides the fundamental types used across the Zosh protocol:

- **Block**: Block structure with header (slot, parent, state root, accumulator, extrinsic root) and extrinsic data
- **State**: Chain state including BFT consensus state, present block head, and transaction accumulator
- **Extrinsic**: Transaction data (bridge bundles, receipts)
- **BFT**: Consensus state (validator set, threshold, randomness series)
- **Registry**: Supported chains (Solana, Zcash) and coins (ZEC)

## Key Types

- `Block`: Complete block with header and extrinsic
- `Header`: Block header with slot, parent hash, state root, accumulator, and validator votes
- `State`: Network state including BFT consensus and accumulator
- `Extrinsic`: Transaction container for bridge bundles and receipts
- `Chain`: Enum for supported chains with bundle size limits
- `Coin`: Supported token types

## Constants

- `EPOCH_LENGTH`: Epoch length for validator rotation (currently 12 slots)

## Hash & Signature Types

- `Hash`: 32-byte Blake3 hash
- `Ed25519Signature`: 64-byte Ed25519 signature
- `TrieKey`: 31-byte key for trie storage

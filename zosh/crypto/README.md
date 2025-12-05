# zosh-crypto

Cryptographic primitives for the Zosh bridge.

## Overview

Provides essential cryptographic functions used throughout the Zosh protocol:

- **Blake3 hashing**: Fast, secure hashing for block headers and state roots
- **Ed25519 signatures**: Validator signing and threshold signature aggregation
- **Merkle tree operations**: State root computation and transaction verification

## Features

- Blake3 hash computation for block headers and accumulators
- Ed25519 keypair generation and signing
- Merkle root calculation for state commitments

## Usage

The crate provides low-level cryptographic primitives used by:
- `zosh-core` for block hashing and state roots
- `zosh-node` for validator signing
- `zosh-runtime` for transaction verification


# zosh-program

Solana on-chain program for the Zosh bridge.

## Overview

Anchor-based Solana program that handles bridge operations on Solana:

- **Mint zoZEC**: Mints wrapped ZEC tokens when bridging from Zcash
- **Burn zoZEC**: Burns wrapped tokens when bridging to Zcash
- **MPC verification**: Validates threshold signatures from validator set
- **Batch processing**: Supports up to 10 recipients per transaction

## Features

- Threshold signature verification using validator MPC pubkey
- Automatic ATA (Associated Token Account) creation
- Event emission for collector monitoring
- Token metadata integration

## Bridge Operations

### Zcash → Solana

1. Validators create bridge bundle with threshold signatures
2. Anyone can submit mint transaction to Solana
3. Program verifies MPC signature and mints zoZEC to recipients

### Solana → Zcash

1. Users burn zoZEC tokens via program instruction
2. Program emits BurnEvent for collectors to detect
3. Validators create bridge bundle for Zcash withdrawal

## Security

- MPC threshold signatures prevent single-point-of-failure
- Program validates all signatures before minting/burning
- No nonces required - validation based on MPC authority

See [Bridge Documentation](../../docs/src/bridge/) for complete bridge workflows.


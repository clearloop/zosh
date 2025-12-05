# Validators

Validators are the consensus nodes in the Zosh network. They run the zoshBFT consensus algorithm and sign bridge transactions using FROST threshold cryptography.

## What Validators Do

- Participate in zoshBFT consensus (BFT voting, block production)
- Maintain the zosh blockchain state
- Sign blocks with Ed25519 signatures
- Create bridge bundles (batching multiple requests)
- Perform FROST threshold signing for bridge transactions
- Commit signed bundles to zosh blocks

> Validators must also run [collectors](./collectors.md) to monitor external chains for bridge events.

## Requirements

### Infrastructure
- **Storage**: parity-db for blockchain state
- **Network**: Stable connection for P2P consensus
- **Collectors**: See [Collectors](./collectors.md) for chain monitoring requirements

### Resources
- Moderate compute for BFT consensus
- Network I/O for P2P communication
- Storage for blockchain state

## Running a Validator Node

See [Node CLI](./api/node.md) for detailed setup and commands.

## Validator Selection

Validators are selected as block leaders using VRF (Verifiable Random Function):
- Randomized leader selection prevents predictability
- Fair rotation based on cryptographic randomness
- Timeout conditions for leader rotation if unresponsive

See [ZoshBFT](./protocol/zoshbft.md) for consensus algorithm details.

## Economics

**Current (POC):**
- Permissionless - anyone can run a validator
- No staking requirements
- No rewards (testnet only)

**Future:**
- Stake SOL to become a validator
- Transaction fees distributed to validators
- Slashing for malicious behavior:
  - Providing false bridge event data
  - Double-signing
  - Extended downtime

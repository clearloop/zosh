# Collectors

Collectors are off-chain services that monitor Solana and Zcash for bridge events. Each validator must run their own collectors to independently verify bridge requests.

## Why Collectors Are Required

Validators cannot verify bridge events without off-chain data:
- Solana burn transactions require RPC access to verify
- Zcash deposits are in shielded pool (require light wallet scanning)

> **Critical:** Validators MUST run their own collectors. Outsourcing to third parties creates a trust assumption that breaks the security model.

## What Collectors Do

### Watching Solana
- Subscribe to Solana program logs via WebSocket
- Detect `BurnEvent` emissions (zoZEC → ZEC bridges)
- Parse event data: sender, amount, Zcash recipient
- Submit bridge requests to local mempool

### Watching Zcash
- Scan orchard shielded pool using light wallet
- Detect incoming ZEC deposits to bridge address
- Decode transaction memos to extract Solana recipient (32-byte pubkey, base58)
- Blacklist invalid memos
- Submit bridge requests to local mempool

## Requirements

### Infrastructure
- **Solana RPC**: Self-hosted or third-party (Helius, Quicknode, etc.)
- **Zcash lightwalletd**: Connection to Zcash network
- **Storage**: Cache for blacklist and processed transactions
- **Bridge keys**: Viewing keys for scanning Zcash orchard pool

### Resources
- Network I/O for chain monitoring (primary bottleneck)
- Storage for cache and blacklist
- Minimal compute

## Running Collectors

Collectors are embedded in the zosh node:

```bash
# Collectors start automatically with the node
cargo run -p zosh-node -- dev
```

**Configuration** (`~/.config/zosh/`):
- Solana RPC endpoint
- Zcash lightwalletd server
- Bridge viewing keys

See [Node CLI](./api/node.md) for detailed configuration.

## Verification Flow

1. Collector detects bridge event (Solana burn or Zcash deposit)
2. Submits to validator's local mempool
3. Validator independently verifies using their collector's data
4. Validator votes on consensus only if their collector confirms the event
5. Bridge bundle created only when threshold validators confirm

This ensures no single point of failure - each validator independently verifies external chain state.

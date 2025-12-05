# Node CLI

Command-line interface for running the zosh node (`zoshd`).

## Installation

```bash
# Build from source
cargo build --release -p zosh-node

# Binary location
./target/release/zoshd
```

## Commands

### `dev` - Development Mode

Run a development node with default configuration.

```bash
# Start dev node (default port 1439)
zoshd dev

# Custom address
zoshd dev --address 0.0.0.0:8080

# Short flag
zoshd dev -a 0.0.0.0:8080
```

### Verbosity

Control log output level:

```bash
# Info level (default)
zoshd dev

# Debug level
zoshd dev -v

# Trace level (maximum verbosity)
zoshd dev -vv
```

**Verbosity levels:**
- `0` (default): Info
- `1` (`-v`): Debug
- `2` (`-vv`): Trace

Alternatively, use `RUST_LOG` environment variable:

```bash
RUST_LOG=debug zoshd dev
```

## Configuration

### Config Directory

`~/.config/zosh/`

Configuration files for:
- Solana RPC endpoint
- Zcash lightwalletd server
- Bridge viewing keys
- Network parameters

### Cache Directory

`~/.cache/zosh/`

Cached data for:
- Collector blacklist
- Processed transactions
- Light wallet state

### Solana Keypair

`~/.config/solana/id.json`

Required for:
- Signing Solana transactions
- MPC threshold operations

## Subcommands

### `solana`

Solana-specific operations (for advanced users):

```bash
zoshd solana <subcommand>
```

### `zcash`

Zcash-specific operations (for advanced users):

```bash
zoshd zcash <subcommand>
```

> **Note:** Subcommands are for development and testing. Normal operation uses `dev` mode.

## Port Configuration

**Default ports:**
- Node P2P: 1439
- RPC WebSocket: 9944 (default, configurable)

## Examples

```bash
# Basic development node
zoshd dev

# Custom port with debug logging
zoshd dev -a 0.0.0.0:8080 -v

# Maximum verbosity for troubleshooting
zoshd dev -vv
```

## Version Information

```bash
# Show version, branch, commit hash, build time
zoshd --version
```

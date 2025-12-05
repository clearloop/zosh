# zosh-runtime

Runtime for block validation, import, and state execution.

## Overview

Core runtime logic for processing blocks and managing chain state:

- **Block validation**: Verifies block structure, state roots, and accumulator continuity
- **Block import**: Atomically commits validated blocks to storage
- **Mempool management**: Handles bridge bundle signature aggregation and receipt queuing
- **State execution**: Processes transactions and updates chain state

## Architecture

The runtime provides a configurable interface with:

- **Config trait**: Defines storage and hook implementations
- **Pool**: Mempool for bridge bundles and receipts
- **Storage**: Abstract storage interface for state persistence
- **Hook**: Callbacks for block events and state changes

## Key Operations

- `validate()`: Verify block before voting
- `import()`: Commit validated block to storage
- `author()`: Create new blocks from mempool
- `pool`: Manage bridge bundle signatures and receipt queue

## Storage

Runtime uses abstract storage interface supporting:
- State root management
- Block and transaction storage
- Accumulator tracking
- BFT consensus state

See [MemPool](../../docs/src/protocol/mempool.md) and [Chain State](../../docs/src/protocol/state.md) for protocol details.


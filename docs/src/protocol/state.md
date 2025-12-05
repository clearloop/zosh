# Chain State

The Zosh chain state maintains a cryptographic commitment to all on-chain data and consensus state. Each block header contains commitments to both the current state and the historical transaction accumulator.

## State Root

The state root is a Merkle root commitment to the current chain state, including:

- **BFT consensus state**: Validator set, threshold, and randomness series
- **Present block head**: Current slot height and block hash
- **Accumulator**: Historical transaction accumulation root

The state root is validated during block import to ensure continuity. Each new block must reference the correct parent state root, preventing forks from diverging state.

## Accumulator

The accumulator provides a cumulative cryptographic commitment to all processed transactions since genesis. It enables efficient verification that a transaction was included in the chain history without replaying the entire chain.

**Accumulation process:**

1. Start with the previous accumulator hash
2. Concatenate all transaction IDs from the current block
3. Hash the combined data to produce the new accumulator

The accumulator hash is included in the block header and validated during import. Invalid accumulator values cause block rejection, ensuring transaction integrity across the chain.

## State Validation

When importing a block, validators verify:

1. **State root match**: Block's parent state root equals current storage root
2. **Accumulator continuity**: New accumulator correctly extends previous accumulator
3. **Vote threshold**: Sufficient validator signatures (2/3 consensus)
4. **Extrinsic root**: Merkle root matches included transactions

Failed validation results in block rejection, maintaining chain integrity and preventing invalid state transitions.

## Storage

The state is persisted using a key-value storage backend with atomic commits:

- **BFT state**: Serialized validator set and consensus parameters
- **Present head**: Current block slot and hash
- **Accumulator**: Latest transaction accumulation root
- **Blocks**: Full block data indexed by hash
- **Transactions**: Individual transaction IDs for lookups

State updates are atomic to prevent corruption from crashes or network failures during block import.

# Block

Blocks are the fundamental units of the Zosh blockchain, containing both consensus metadata and transaction data. Each block extends the chain through cryptographic commitments to previous state and new transactions.

## Structure

A block consists of two components:

- **Header**: Consensus metadata and cryptographic commitments
- **Extrinsic**: Transaction data (bridge bundles and receipts)

## Header

The block header contains all consensus-critical metadata:

- **slot**: Block height (incrementing sequence number starting from genesis)
- **parent**: Hash of the previous block header (32 bytes)
- **state**: Merkle root of the parent state (32 bytes)
- **accumulator**: Cumulative hash of all transaction IDs up to this block (32 bytes)
- **extrinsic**: Merkle root of transactions included in this block (32 bytes)
- **votes**: Map of validator public keys to Ed25519 signatures

**Block Hash:**

The block hash is computed as:

```
BLAKE3(slot || parent || state || accumulator || extrinsic)
```

The votes field is excluded from the hash computation to allow validators to sign and aggregate their signatures after the block is proposed.

## Extrinsic

The extrinsic contains the actual transaction data:

- **bridge**: Map of bundle hashes to [bridge bundles](./transaction.md#bridge) (batch bridge requests)
- **receipts**: Vector of [receipt transactions](./transaction.md#receipt) (bridge confirmations)

Transactions are organized into bundles for efficient processing. The extrinsic root is a Merkle tree commitment to all transaction IDs, allowing efficient verification without processing all transactions.

See [Transaction](./transaction.md) for detailed information about bridge requests and receipts.

## Production

1. Leader collects transactions from the mempool
2. Leader computes the new accumulator from parent accumulator + new transaction IDs
3. Leader builds the header with all commitments
4. Leader proposes the block to validators
5. Validators sign the block hash if valid
6. Once 2/3 threshold is reached, the block is finalized

## Validation

When receiving a block, validators verify:

1. **Parent hash**: Block extends the correct parent
2. **State root**: Parent state root matches current storage
3. **Accumulator**: New accumulator correctly extends previous
4. **Extrinsic root**: Merkle root matches included transactions
5. **Signatures**: At least 2/3 validators signed the block hash

Failed validation results in block rejection, preventing invalid blocks from entering the chain.

## Finality

Blocks achieve finality once they receive 2/3 validator signatures. Unlike probabilistic finality in Nakamoto consensus, zoshBFT provides deterministic finality - finalized blocks cannot be reverted.

The votes field accumulates signatures until the threshold is met. Once finalized, the block is committed to storage and propagated to the network.

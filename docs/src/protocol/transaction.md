# Transaction

Zosh processes three types of transactions: Bridge requests, Receipts, and Disputes.

## Bridge

Bridge transactions represent requests to move assets between Solana and Zcash. Collectors detect these requests when users lock ZEC on Zcash or burn zoZEC on Solana.

Validators aggregate multiple bridge requests into bundles for batch processing. Each bundle requires threshold signatures from 2/3 of validators before execution on the target chain.

**Limits:**
- Solana bundles: Maximum 10 requests per bundle
- Zcash bundles: Limited by available notes in the orchard pool

## Receipt

Receipts are confirmation transactions that link source and target chain operations. When validators successfully execute a bridge bundle, they generate receipts proving the cross-chain transfer completed.

Receipts allow users to verify their bridge operations by matching the original transaction ID with the confirmation transaction on the target chain.

## Dispute

> **Status**: Not yet implemented

Disputes allow validators to challenge bridge operations that failed or executed incorrectly. When a bridge request doesn't complete successfully, validators can submit a dispute to trigger re-verification by other validators.

Invalid disputes may result in slashing of the challenger's staked SOL, preventing spam attacks on the dispute mechanism.

## Transaction Flow

1. Collector detects lock/burn on source chain
2. Validators aggregate requests in mempool
3. 2/3 validators sign bundle with threshold signatures
4. Bundle executed on target chain
5. Receipt generated and propagated to network
6. Dispute submitted if execution fails (planned feature)

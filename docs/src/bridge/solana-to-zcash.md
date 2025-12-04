# Solana to Zcash

Bridging zoZEC from Solana to Zcash is pretty straightforward here.

## 1. User burns zoZEC with ZEC recipient specified

```rust
struct BridgeToZcash {
    /// The amount of the transaction
    amount: u64,

    /// The zcash recipient address (orchard only)
    recipient: String,
}
```

User send the burn instruction to our solana program.

## 2. Validators identify and pack the transaction

Same as what we do in the zcash to solana bridging, anyone can submit the
transaction to zosh, the difference is that for packing this block, we
need two-round signature aggregation with frost.

```rust
struct BridgeBundleToZcash {
    /// The unbridged requests.
    txs: Vec<(BridgeToZcash, [u8; 32])>,

    /// The orchard bundle of the requests.
    bundle: OrchardBundle,

    /// The signature of the randomizer.
    signature: Signature,
}
```

1. Verify signatures of solana transactions
2. Select all unbridged solan-to-zcash notes on chain.
3. Pack the transactions into an orchard bundle.
4. Do the 2-round signature aggregation with frost via p2p.
5. Pack the finalized zcash transaction into a new block.

> The selected validator who packs the bundle need also handle the UTXO stuff
> on the zcash side, unspent funds will transfer back to the bridge's orchard
> address.

## 3. The recipient get ZEC on Zcash

The raw zcash transaction intrudoced at `2.` will be stored on chain as well,
except the selected validator, anyone else can submit it to the zcash network
as well!

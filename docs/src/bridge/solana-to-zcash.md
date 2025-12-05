# Solana to Zcash

Bridging zoZEC from Solana to Zcash is pretty straightforward here.

## 1. User burns zoZEC with ZEC recipient specified

```rust
struct BridgeToZcash {
    /// The amount of the transaction
    amount: u64,

    /// The zcash recipient address (orchard unified address, 110 chars)
    recipient: String,
}
```

User calls the burn instruction on the Solana program. This is a public action - anyone can burn their own zoZEC.

The program:
1. Burns the zoZEC from the user's token account
2. Emits a `BurnEvent` with sender, amount, and Zcash recipient
3. Collectors watch these events to trigger the bridge process

## 2. Collectors detect and submit bridge requests

Collectors subscribe to Solana program logs and detect `BurnEvent` emissions:

1. Listen to Solana program logs for burn events
2. Parse the event to extract sender, amount, and Zcash recipient
3. Submit bridge request to zosh mempool

```rust
struct Bridge {
    coin: Coin::Zec,
    recipient: Vec<u8>,  // Zcash orchard address
    amount: u64,
    txid: Vec<u8>,       // Solana burn transaction signature
    source: Chain::Solana,
    target: Chain::Zcash,
}
```

Anyone can run a collector - it's permissionless.

## 3. Validators create and sign Zcash transaction

Validators collect bridge requests in the mempool and create Zcash transactions:

1. Bridge requests enter the mempool
2. Validators create a Zcash orchard transaction bundle
3. FROST threshold signing: Validators perform 2-round signature aggregation to sign the Zcash transaction
4. The finalized Zcash transaction is committed to a zosh block

```rust
struct BridgeBundle {
    target: Chain::Zcash,
    bridge: Vec<Bridge>,           // Burn requests
    data: Vec<u8>,                 // Zcash transaction data
    signatures: Vec<Vec<u8>>,      // FROST threshold signatures
}
```

> The Zcash transaction is created using the bridge's orchard wallet. Unspent funds (UTXO change)
> are sent back to the bridge's orchard address.

## 4. The recipient receives ZEC on Zcash

After validators sign the Zcash transaction with FROST, **anyone** can submit it to the Zcash network:
- The user themselves
- A collector/relayer service
- Any third party (permissionless)

The signed Zcash transaction is stored in the zosh block and can be broadcast by anyone with access to a Zcash node.

> No on-chain verification on Zcash is needed - the FROST signature already proves validator consensus.
> Recipients receive ZEC directly to their orchard shielded address.

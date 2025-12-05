# Zcash to Solana

Since zcash doesn't have a VM, we have to maintain a threshold wallet, to
make the funds on zcash secure and trustless.

## 1. User deposits ZEC to the shielded address

User send ZEC to the shielded address of bridge with `memo` which includes
a serialized `bridge` instruction of Zosh network.

```rust
enum MemoInstruction {
    BridgeToSolana {
        // the recipient address on solana
        recipient: Pubkey,
    }
}
```

> The sender will get refunded if the bridge can not decode the memo.

## 2. Collectors detect and submit bridge requests

Collectors (off-chain services) watch the Zcash orchard pool for incoming deposits:

1. Scan spendable notes using the bridge's viewing key
2. Decode memo to extract Solana recipient address (32-byte pubkey, base58 encoded)
3. Create bridge request and submit to the zosh mempool

```rust
struct Bridge {
    /// The token of the transaction
    coin: Coin::Zec,

    /// The recipient address (Solana pubkey from memo)
    recipient: Vec<u8>,

    /// The amount of the transaction
    amount: u64,

    /// The zcash transaction id
    txid: Vec<u8>,

    /// The source chain
    source: Chain::Zcash,

    /// The target chain
    target: Chain::Solana,
}
```

Anyone can run a collector - it's permissionless. Invalid bridge requests (wrong memo format, invalid addresses) are automatically blacklisted.

## 3. Validators aggregate threshold signatures

Validators collect bridge requests in the mempool and create bundles:

1. **Mempool queuing**: Bridge requests enter the mempool
2. **Bundle creation**: Validators batch multiple bridge requests together
3. **Threshold signing**: Each validator signs the bundle. When threshold (e.g., 2/3) is reached, bundle is ready
4. **Consensus**: Bundle is committed to a zosh block

```rust
struct BridgeBundle {
    /// The target chain of the bundle
    target: Chain::Solana,

    /// The bridge transactions
    bridge: Vec<Bridge>,

    /// The data we need for reconstructing the outer transaction
    data: Vec<u8>,

    /// The signatures for the upcoming outer transactions
    signatures: Vec<Vec<u8>>,
}
```

Once enough validators sign, the bundle moves to completed status in the mempool and can be executed on Solana.

## 4. The recipient receives zoZEC on Solana

After validators sign the bundle, **anyone** can submit the mint transaction to Solana:
- The user themselves
- A collector/relayer service
- Any third party (permissionless)

The Solana program verifies:

1. **MPC signature**: Transaction must be signed by the validator MPC pubkey
2. **Batch processing**: Up to 10 recipients can be minted in one transaction
3. **ATA creation**: Automatically creates Associated Token Accounts if needed

```rust
// Solana program validates MPC signature
require!(payer.key() == bridge_state.mpc, InvalidMpcSigner);
```

The program mints **zoZEC** (8 decimals) directly to recipients' token accounts.

> Validation is based on MPC threshold signatures, not nonces. The MPC pubkey represents
> the collective signing authority of the validator set.

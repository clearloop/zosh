# Zcash to Solana

Since zcash doesn't have a VM, we have to maintain a threshold wallet, to
make the funds on zcash secure and trustless.

## 1. User deposits ZEC to the shielded address

User send ZEC to the shielded address of bridge with `memo` which includes
a serialized `bridge` instruction of Zorch network.

```rust
enum MemoInstruction {
    BridgeToSolana {
        // the recipient address on solana
        recipient: Pubkey,
    }
}
```

> The sender will get refunded if the bridge can not decode the memo.

## 2. Validators identify and pack the transaction

Anyone can submit the transaction to the zorch chain with a `bridge` transaction.

```rust
struct BridgeToSolana {
    /// the recipient address on solana
    recipient: Pubkey,

    /// the amount of the transaction
    amount: u64,

    /// the zcash transaction id
    id: [u8; 32],
}
```

The transactions will be packed into a block and get validated by all of the validators:

- Fetch spendable notes after the latest zcash height stored on the zorch chain.
- Check if the amount and the recipient are matched with the transaction id.
- Check if the notes have already been bridged.

> All bridged notes will be marked as bridged on the zorch chain, sort like the UTXO
> design.

If everything are valid, the block will be committed to the chain.

## 3. Validators aggregate signatures for solana

At the commitment of a block, the network will aggregate signatures for a bundle of
bridging notes.

```rust
struct BridgeBundleToSolana {
    /// The unbridged requests.
    txs: Vec<BridgeToSolana>,

    /// The merkle root of the requests.
    root: [u8; 32],

    /// The nonce of the mint state
    nonce: u64,

    /// The signatures map of the merkle root.
    signatures: BTreeMap<ValidatorId, Signature>,
}

```

1. Select all unbridged notes on chain.
2. Append signatures to the bridge bundle.
3. Pack the bridge bundle into a new block.

Once this block get committed on chain, all notes inside of it will be transited to
the bridged status.

## 4. The recipient get zrcZEC on Solana

After the commitment of the bundle block, anyone can use the bundled data to mint
zrcZEC on solana, the user itself, a solver, a relayer, or anybody wants to do it,
we likely need to do sort of incentive mechanism to encourage them to do it.

On the solana program side, we verify the following stuffs:

- all of the signatures are valid.
- validated signatures are over the threshold of the validator set.
- the nounce mathces the one in the program

> The solana program maintains a `nounce` to deduplicate the mint requests, each time
> a bundle processed, the `nounce` will be incremented.

Mint the zrcZEC, bumps the nounce.

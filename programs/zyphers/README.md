# Zorch: Zcash-Solana Bridge Consensus Program

A Solana program implementing threshold signature-based consensus for bridging ZEC tokens between Zcash and Solana networks.

## Overview

This program mints sZEC (Solana-wrapped ZEC) based on validator signatures and allows users to burn sZEC to bridge back to Zcash. The program uses a threshold signature scheme where actions require approval from a specified number of validators.

## Architecture

### Core Components

1. **BridgeState** - Main program state storing:

   - Validator set (list of validator public keys)
   - Threshold requirement (e.g., 2 of 3)
   - Nonce for replay protection
   - sZEC mint authority

2. **Threshold Actions** - Require validator signatures:

   - Minting sZEC to recipients
   - (Future) Updating validator sets

3. **Public Actions** - Anyone can submit:
   - Burning sZEC to bridge to Zcash

## Instructions

### 1. Initialize

Initializes the bridge with an initial set of validators and creates the sZEC SPL token mint.

**Parameters:**

- `initial_validators: Vec<Pubkey>` - Initial set of validator public keys
- `threshold: u16` - Minimum number of signatures required (e.g., 2 for 2/3)

**Accounts:**

- Creates `bridge_state` PDA (seeds: `["bridge-state"]`)
- Creates `szec_mint` with 8 decimals (seeds: `["szec-mint"]`)

### 2. Mint sZEC

Mints sZEC tokens to a recipient's account. Requires threshold validator signatures.

**Parameters:**

- `recipient: Pubkey` - Recipient's public key
- `amount: u64` - Amount to mint (in lamports, 8 decimals)
- `signatures: Vec<[u8; 64]>` - Ed25519 signatures from validators

**Process:**

1. Constructs action message from action type, nonce, and action data
2. Verifies signatures against current validator set
3. Checks threshold requirement is met
4. Mints tokens using bridge_state as mint authority
5. Emits `MintEvent`
6. Increments nonce

### 3. Burn sZEC

Burns sZEC tokens to initiate bridging back to Zcash. Public action.

**Parameters:**

- `amount: u64` - Amount to burn
- `zec_recipient: String` - Zcash recipient address (26-95 characters)

**Process:**

1. Burns tokens from signer's account
2. Emits `BurnEvent` with ZEC recipient address
3. Off-chain validators monitor events and process ZEC transfers

## Events

- **MintEvent** - Emitted when sZEC is minted

  - `recipient`, `amount`, `nonce`, `timestamp`

- **BurnEvent** - Emitted when sZEC is burned

  - `sender`, `amount`, `zec_recipient`, `timestamp`

- **ValidatorSetUpdated** - Emitted when validator set changes (reserved for future use)

## Security Features

1. **Nonce-based Replay Protection** - Each action increments a nonce to prevent replay attacks

2. **Threshold Signatures** - Actions require approval from multiple validators

3. **PDA Authority** - Bridge state is a PDA with deterministic address

4. **SPL Token Standard** - sZEC uses standard SPL Token-2022 interface

## Known Limitations & TODOs

### Critical for Production

1. **Signature Verification** - Current implementation uses placeholder verification

   - **Required**: Implement proper Ed25519 signature verification
   - **Options**:
     - Use Solana's Ed25519Program precompile for on-chain verification
     - Move verification off-chain and use on-chain authorization checks
   - See `utils.rs::ed25519_verify()` for implementation notes

2. **Action Record PDAs** - Currently simplified for compilation

   - Reimplement action record PDAs with proper hash-based seeds
   - Add replay protection using ActionRecord accounts

3. **Validator Set Management** - Not yet implemented
   - Add `update_validators_full` instruction
   - Add `add_validator` and `remove_validator` instructions
   - Implement proper account reallocation for dynamic validator sets

### Recommended Enhancements

4. **Hash Function** - Currently uses simple XOR-based hash

   - Replace with proper cryptographic hash (SHA-256 or Keccak)
   - Use for action record seeds and signature messages

5. **IDL Generation** - Build currently skips IDL

   - Fix IDL generation for anchor-spl types
   - Add proper type definitions for client libraries

6. **Testing** - Add comprehensive test suite

   - Unit tests for threshold verification
   - Integration tests for full bridge flow
   - Fuzz testing for edge cases

7. **Governance** - Consider adding:
   - Timelock for sensitive operations
   - Multi-sig authority for emergency actions
   - Pausable functionality

## Building

```bash
# Build Solana program (BPF)
cargo build-sbf --manifest-path programs/zorch/Cargo.toml

# Build with Anchor (requires IDL fixes)
anchor build
```

## Program ID

```
2KwobV7wjmUzGRQfpd3G5HVRfCRUXfry9MoM3Hbks9dz
```

## Dependencies

- `anchor-lang` - 0.32.1
- `anchor-spl` - 0.32.1

## File Structure

```
programs/zorch/src/
├── lib.rs           # Main program and instruction handlers
├── errors.rs        # Custom error definitions
├── events.rs        # Event definitions
└── utils.rs         # Signature verification utilities
```

## Usage Example

```typescript
// Initialize bridge
await program.methods
  .initialize(validatorPubkeys, threshold)
  .accounts({
    payer: payer.publicKey,
    bridgeState,
    szecMint,
    systemProgram,
    tokenProgram,
    rent,
  })
  .rpc();

// Mint sZEC (with validator signatures)
await program.methods
  .mintSzec(recipient, amount, signatures)
  .accounts({
    payer: payer.publicKey,
    bridgeState,
    szecMint,
    recipientTokenAccount,
    tokenProgram,
    systemProgram,
  })
  .rpc();

// Burn sZEC
await program.methods
  .burnSzec(amount, zecAddress)
  .accounts({
    signer: signer.publicKey,
    signerTokenAccount,
    szecMint,
    bridgeState,
    tokenProgram,
  })
  .rpc();
```

## Contributing

Before deploying to production:

1. Implement proper signature verification
2. Add comprehensive tests
3. Complete security audit
4. Implement validator set management
5. Add monitoring and alerting

## License

[Add your license here]

# Program API

Solana program interface for the Zosh bridge (Anchor framework).

**Program ID:** `zosh4npemTuTj18MHgbn7NRihzMkTfgswTyP34LPaVx`

## Instructions

### `initialize`

Initialize the bridge state and create zoZEC SPL token mint.

**Authority required:** Program deployer (one-time setup)

**Parameters:**
- `mpc: Pubkey` - MPC public key for validator threshold signing

**Accounts:**
- Creates `bridge_state` PDA
- Creates `zec_mint` PDA (8 decimals)

**Example:**
```rust
initialize(
    mpc: Pubkey
)
```

### `metadata`

Update zoZEC token metadata (name, symbol, URI).

**Authority required:** Bridge authority only

**Parameters:**
- `name: String` - Token name
- `symbol: String` - Token symbol (e.g., "zoZEC")
- `uri: String` - Metadata URI

### `mint`

Mint zoZEC to recipients (threshold action, validator-only).

**Authority required:** MPC signature (validators)

**Parameters:**
- `mints: Vec<MintEntry>` - Batch of mint operations (max 10)

**MintEntry structure:**
```rust
struct MintEntry {
    recipient: Pubkey,  // Solana address to receive zoZEC
    amount: u64,        // Amount in lamports (8 decimals)
}
```

**Features:**
- Batch minting (up to 10 recipients per transaction)
- Automatic ATA creation if needed
- Emits `MintEvent` for each recipient

**Requirements:**
- Transaction must be signed by MPC pubkey
- Validates all recipient addresses

### `burn`

Burn zoZEC to bridge back to Zcash (public action).

**Authority required:** None (anyone can burn their own zoZEC)

**Parameters:**
- `amount: u64` - Amount to burn (8 decimals)
- `zec_recipient: String` - Zcash orchard address (110 characters)

**Requirements:**
- User must have sufficient zoZEC balance
- Zcash address must be valid orchard unified address (110 chars)

**Emits:**
- `BurnEvent` with sender, amount, and Zcash recipient
- Collectors watch for this event to trigger bridge process

**Example:**
```rust
burn(
    amount: 100_000_000,  // 1 ZEC (8 decimals)
    zec_recipient: "utest1..."  // 110-char address
)
```

### `update_mpc`

Update the MPC public key (threshold action, validator-only).

**Authority required:** Current MPC signature (validators)

**Parameters:**
- `mpc: Pubkey` - New MPC public key

**Use case:** Validator set rotation, key refresh

## State Accounts

### `bridge_state`

PDA storing bridge configuration.

**Seeds:** `["bridge-state"]`

**Fields:**
- `authority: Pubkey` - Bridge authority (for metadata updates)
- `mpc: Pubkey` - MPC public key (validator threshold signing)
- `zec_mint: Pubkey` - zoZEC SPL token mint address
- `bump: u8` - PDA bump seed

### `zec_mint`

SPL token mint for zoZEC.

**Seeds:** `["zec-mint"]`

**Properties:**
- Decimals: 8 (same as Zcash)
- Mint authority: `bridge_state` PDA
- Freeze authority: None

## Events

### `MintEvent`

Emitted when zoZEC is minted.

```rust
struct MintEvent {
    mints: Vec<(Pubkey, u64)>,  // (recipient, amount) pairs
    timestamp: i64,
}
```

### `BurnEvent`

Emitted when zoZEC is burned.

```rust
struct BurnEvent {
    sender: Pubkey,
    amount: u64,
    zec_recipient: String,
    timestamp: i64,
}
```

## Error Codes

- `InvalidMpcSigner` - Transaction not signed by MPC pubkey
- `InvalidRecipient` - Invalid recipient address
- `InvalidAmount` - Amount is zero or invalid
- `InvalidMint` - Incorrect mint account
- `InvalidBatchSize` - Batch size exceeds maximum (10)
- `InvalidZcashAddress` - Zcash address invalid (must be 110 chars)

## Deployment

**Testnet (Devnet):**
- Program: `zosh4npemTuTj18MHgbn7NRihzMkTfgswTyP34LPaVx`
- See [Addresses](./addresses.md) for bridge state and mint addresses

**Mainnet:** Not yet deployed

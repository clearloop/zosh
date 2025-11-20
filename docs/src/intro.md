# ZypherBridge

The privacy bridge for Solana and Zcash.

```mermaid
flowchart TD
subgraph ZC [Zcash Chain]
ZA[Shielded Multi-sig<br>Escrow Address]
ZB[User's Shielded Address]
end

    subgraph SC [Solana Chain Programs]
        LC[Zcash Light Client<br>Stores & Validates Headers]
        VP[State Proof Verifier<br>ZK Proof Verification]
        BC[Bridge Logic<br>Mint/Burn wZEC]
        CG[Contract Governance<br>Multi-sig Upgradeable]
    end

    subgraph GN [Guardian Network TSS/MPC]
        G1[Guardian 1]
        G2[Guardian 2]
        G3[Guardian ...]
        G4[Guardian N]

        DKG[Distributed Key Generation]
        PSS[Proactive Secret Sharing]
        TS[Threshold Signing Protocol]
    end

    %% Zcash to Solana Flow
    U1[User: Deposit ZEC to Bridge] --> ZT[Send ZEC to Shielded<br>Escrow Address]
    ZT --> ZA

    RL[Relayer Network] --> MH[Monitors Zcash &<br>Generates State Proof]
    MH --> SP[Submits Block Header<br>& State Proof to Solana]

    SP --> LC
    SP --> VP
    LC -->|Provides trusted header| VP
    VP -->|Proof Valid| BC
    BC --> UM[Mints wZEC to<br>User's Solana Address]

    %% Solana to Zcash Flow
    U2[User: Redeem ZEC from Bridge] --> BT[Burn wZEC on Solana]
    BT --> BE[Emit Burn Event]

    BE --> GN
    GN -->|Observes Event| TS
    TS -->|M-of-N Signers Collaborate| ST[Create Shielded Tx<br>Spend from Escrow]
    ST --> ZS[Send ZEC to User's<br>Zcash Address]
    ZS --> ZB

    %% Guardian Management
    DKG -.->|Initial Setup| GN
    PSS -.->|Regular Key Refresh| GN
    CG -.->|Governs Parameters| GN
```

### Development Links

- [zecfaucet][faucet]
- [zcash testnet][zectestnet]
- [zcash RPC][rpc]
- [zcash explorer][explorer]

[explorer]: https://mainnet.zcashexplorer.app/
[rpc]: https://zcash.github.io/rpc/
[faucet]: https://testnet.zecfaucet.com/
[zectestnet]: https://blockexplorer.one/zcash/testnet

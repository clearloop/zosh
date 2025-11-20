# Bridge

## Solana to Zcash

The verification is based on our [MPC Protocol](./mpc.md).

## Zcash to Solana

The verification is based on [ZIP-221][zip221].

```mermaid
flowchart TD
subgraph ZC [Zcash Chain]
ZT[User Deposits ZEC<br>to Shielded Address]
ZB[Zcash Blocks with MMR]
end

    subgraph RL [Relayer Network]
        RM[Monitors Zcash Chain]
        MG[Generates State Proofs]
        SM[Submits to Solana]
    end

    subgraph SC [Solana Programs]
        LC[Zcash Light Client<br>Stores MMR & Headers]
        TS[TSS Signature Verifier<br>MPC Committee Approval]
        MM[MMR State Manager<br>Updates Chain State]
        BR[Bridge Logic<br>Mints sZEC]
    end

    subgraph MPC [MPC Committee]
        MP1[MPC Node 1]
        MP2[MPC Node 2]
        MP3[MPC Node N]
        TSG[TSS Signature Generation]
    end

    ZT --> RM
    ZB --> RM
    RM --> MG
    MG --> SM
    SM --> LC
    SM --> TS
    LC --> MM
    TS --> MM
    MM --> BR
    BR --> UM[Mints sZEC to User]

    MPC --> TSG
    TSG --> TS

    style SC fill:#e8f5e8
    style MPC fill:#fff3e0
```

[zip221]: https://zips.z.cash/zip-0221

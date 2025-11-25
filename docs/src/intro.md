# Zorch

The trustless privacy bridge for Solana and Zcash.

- **Security**: The funds in the [bridge](/bridge) are secured by [the Zorch Protocol](/consensus.md).

  - No custody wallet, everything on-chain.
  - [frost][frost] to manage the zcash orchard pool.
  - `multi-sig` to manage the zrcZEC on the solana side.

- **Privacy**: The privacy is ensured by the [orchard][orchard] pool from zcash.
  - On bridging ZEC to SOL, Zorch doesn't know the depositor of ZEC.
  - On bridging SOL to ZEC, Zorch sends the funds back to the orchard pool.

Check out [bridge](/bridge) and [demo](/demo.md) for more details.

## Technical Overview

Zorch is a compact relay protocol in blockchain structure.

Zorch uses a custom consensus algorithm called [zorchBFT](/zorchbft.md) inspired by Hotstuff and
its successors. Both the algorithm and networking stack are optimized from the ground up to
support the unique demands of the Bridge.

Zorch state execution is heavily based on external transactions, all confirmed output transactions
will be committed on chain and finally can be executed by anyone.

```mermaid
flowchart LR
    A[Zcash]
    B[Solana]
    C[ZorshBFT <-> State Machine]

    A -.-> |FlyClient| C
    B -.-> |LightClient| C
    C --> |Frost| A
    C --> |Multisig| B

    subgraph The Privacy Bridge
        direction LR
        A
        B
    end
```

[frost]: https://frost.zfnd.org/
[orchard]: https://zcash.github.io/orchard/

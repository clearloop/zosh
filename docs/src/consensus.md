# Consensus

The consensus is using [ZorchBFT](#the-zorchbft) which connects offchain data sources
efficiently to a secure, reliable, and unstoppable network.

## Data Sources

Each node wraps the following data sources:

- Zcash light wallet for indexing/validating the incoming zcash transactions.
- Solana light client for indexing solana side operations.

It's currently okay using third-party API to fetch data sources since we'll submit
our results of the data on chain and get validated by other nodes finally.

> For the long-term plan, each of the nodes will be required to run
>
> - a light client of solana, developed by us.
> - a flyclient of zcash, developed by us.
>
> Hope we can get a startup funding at the zyperpunk hackathon, then we can make
> things as what they should be!

### External Transactions Confirmations

The block time of zcash is too long, for providing a good UE, we need to speed
up things, and the biggest enemy is the reorg problem, small transfers will get
fast confirmed (1-3), users will get refunded once validators have committed
mistakes, which will be triggered by the dispute transactions, for large transfer,
we'd make sure that they get enough confirmations (7-10).

## The ZorchBFT

ZorchBFT is based on PoA with an optimized [hotstuff][hotstuff] implementation,
shortly:

- No fixed block time
- VRF for randomizing the leader selection
- Threshold signatures for the block commitment
- Timeout conditions for force rotating leaders
- Key refreshing for the orchard package

> We will upgrade zorch to a PoS network after getting the product market fit.

### Dual blocks

Some of the multi-round signatures are processed across blocks, for fast confirmation
cases, we'll have dispute transactions to update them, users can cancel their bridge
requests as well if they want, and such kind of transactions will have the priority
to be processed.

[hotstuff]: https://arxiv.org/abs/2107.04947

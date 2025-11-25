# Data Sources

Each zorch node wraps the following data sources:

- Zcash light wallet for indexing/validating the incoming zcash transactions.
- Solana light client for indexing solana side operations.

It's currently okay using third-party API to fetch data sources since we'll submit
our results of the data on chain and get validated by other nodes finally.

> For the long-term plan, each of the nodes will be required to run
>
> - a light client of solana, developed by us.
> - a flyclient of zcash, developed by us.

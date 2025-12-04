# ZoshBFT

ZoshBFT is based on PoS with an optimized [hotstuff][hotstuff] implementation, shortly:

- No fixed block time
- VRF for randomizing the leader selection
- Threshold signatures for the block commitment
- Timeout conditions for force rotating leaders
- Key refreshing for the orchard package

> We will upgrade zosh to a PoS network after getting the product market fit.

## Dual blocks

Some of the multi-round signatures are processed across blocks, for fast confirmation
cases, we'll have dispute transactions to update them, users can cancel their bridge
requests as well if they want, and such kind of transactions will have the priority
to be processed.

[hotstuff]: https://arxiv.org/abs/2107.04947

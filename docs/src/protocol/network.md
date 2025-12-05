# Network

The Zosh network layer enables P2P communication between validators using QUIC protocol for efficient, secure message exchange. The network facilitates consensus operations, block propagation, and bridge transaction coordination.

## Transport

Zosh uses QUIC (Quick UDP Internet Connections) for P2P networking:

- **Low latency**: UDP-based with multiplexing and 0-RTT connection establishment
- **Built-in encryption**: TLS 1.3 by default, securing all validator communication
- **Connection migration**: Maintains connections across network changes
- **Stream multiplexing**: Multiple concurrent message streams without head-of-line blocking

QUIC is well-suited for BFT consensus where rapid message exchange is critical for achieving 2/3 validator agreement.

## Message Types

The network layer handles several types of messages:

### Block Gossip

Validators propagate new blocks to the network:

1. Leader authors a new block
2. Leader broadcasts block to all validators
3. Validators verify and sign the block
4. Signatures gossiped back to leader and peers
5. Once 2/3 threshold reached, block is finalized

Block gossip ensures all validators maintain synchronized chain state.

### Round Signing

During consensus rounds, validators exchange signatures:

- **Vote messages**: Validators sign block hashes and broadcast votes
- **Signature aggregation**: Network collects votes until 2/3 threshold
- **QC formation**: Quorum certificate formed when threshold met

Round signing enables the zoshBFT consensus to achieve finality.

### Bridge Bundle Coordination

Validators coordinate bridge bundle creation and signing across the network. See [Bundle](./bundle.md) for the complete bundling workflow.

### State Synchronization

New validators or validators recovering from downtime sync state:

- Request missing blocks from peers
- Download chain state snapshots
- Verify state transitions and accumulator

## Peer Discovery

Validators discover and maintain connections to peers:

- **Bootstrap nodes**: Initial peer set from configuration
- **Peer exchange**: Validators share known peer addresses
- **Connection management**: Maintain stable connections to subset of network

## Security

Network security mechanisms:

- **TLS 1.3 encryption**: All QUIC connections encrypted by default
- **Peer authentication**: Validators verify peer identities using Ed25519 keys
- **Message validation**: Invalid messages rejected before processing
- **DoS protection**: Rate limiting and connection quotas

The network layer ensures validators can securely coordinate consensus and bridge operations without centralized infrastructure.

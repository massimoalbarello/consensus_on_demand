# Fast IC Consensus

This is an implementation of a consensus algorithm based on the Internet Computer Consensus which improves the latency of block finalization.

Using different terminal windows, start six instances and pass as a parameter a different number in `[1,6]`. This will identify the node number of each replica.
The other parameters can be set to `6 1 1 false` to use the original IC Consensus, or to `6 1 1 true` to use the Fast IC Consensus.

Example: to start replica 2 with FICC, run: `cargo run 2 6 1 1 true`. Then repeat for all the other replicas by changing only the first parameter. Start node number 1 last. If your local network allows mDNS, they will automatically connect.
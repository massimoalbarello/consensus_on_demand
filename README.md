# Fast IC Consensus

This is an implementation of a new consensus algorithm based on the Internet Computer Consensus which improves the latency of block finalization.

To create a local test subnet, using different terminal windows, start `6` instances by running `cargo run -- --r <replica_number> --cod`, where `<replica_number` must be an integer within `[1, 6]`.

By default, replicas run for `300` seconds and them automatically exit. You can modify this by adding the flag `--t <time_in_seconds>`.
 
The following flags allow you to modify other parameters of the subnet:
- `--n`: total number of nodes (`n > 3f + 2p`)
- `--f`: number of byzantine nodes
- `--p`: number of disagreeing nodes (only when running FIC Consensus)

You can remove the `--cod` flag to run the original IC Consensus.
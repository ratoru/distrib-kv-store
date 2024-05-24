# TODOs

## Sharding

- [x] Implemented CARP protocol and added it to server.
- [ ] Modify CARP to also store addresses of followers (not just leader).
- [ ] Make client request and use CARP protocol.
- [ ] Make a script and/or function to set up sharded clusters. The script must also send out the CARP config (which is just a local `Carp` object).

## Benchmarking

- [ ] Test different workloads on different architecture.
- [ ] Use [Criterion](https://github.com/tikv/raft-rs?tab=readme-ov-file#benchmarks) for benchmarking.
- [ ] Could also use [hyperfine](https://github.com/sharkdp/hyperfine) instead. This would benchmark shell commands instead of Rust code.


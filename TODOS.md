# TODOs

## 1. Building a basic distributed key-value store

- [x] Fix and run `openraft` basic example.
- [ ] Replace `tide` with more performant `axum`.
- [ ] Replace `toy-rpc` with more performant `tonic`.
- [ ] Improve testing scripts:
    - [ ] Improve integrations tests `test_cluster`.
    - [ ] Test `store.rs` with [openraft::testing](https://docs.rs/openraft/latest/openraft/testing/struct.Suite.html#method.test_all).

## 2. Consistent hashing

...

## 3. Benchmarking

- [ ] Use [Criterion](https://github.com/tikv/raft-rs?tab=readme-ov-file#benchmarks) for benchmarking.


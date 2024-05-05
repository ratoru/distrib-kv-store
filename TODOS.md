# TODOs

## 1. Building a basic distributed key-value store

- [x] Fix and run `openraft` basic example.
- [x] Replace `tide` with more performant `axum`.
- [ ] Replace `toy-rpc` with more performant `tonic`.
- [x] Improve testing scripts:
    - [x] Improve integrations tests with `test_cluster`. Currently broken because Axum doesn't always respond with ("OK": ...) or ("Err": ...). Either read response code, or change API.
    - [x] Test `store.rs` with [openraft::testing](https://docs.rs/openraft/latest/openraft/testing/struct.Suite.html#method.test_all).

## 2. Consistent hashing

...

## 3. Benchmarking

- [ ] Use [Criterion](https://github.com/tikv/raft-rs?tab=readme-ov-file#benchmarks) for benchmarking.
- [ ] Could also use [hyperfine](https://github.com/sharkdp/hyperfine) instead. This would benchmark shell commands instead of Rust code.


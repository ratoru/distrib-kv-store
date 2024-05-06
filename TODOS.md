# TODOs

## 1. Building a basic distributed key-value store

- [x] Fix and run `openraft` basic example.
- [x] Replace `tide` with more performant `axum`.
- [ ] Replace `toy-rpc` with more performant `tonic` or `tarpc`.
- [x] Improve testing scripts:
    - [x] Improve integrations tests with `test_cluster`. Currently broken because Axum doesn't always respond with ("OK": ...) or ("Err": ...). Either read response code, or change API.
    - [x] Test `store.rs` with [openraft::testing](https://docs.rs/openraft/latest/openraft/testing/struct.Suite.html#method.test_all).

## 2. Consistent hashing

We have to decide how exactly to do this:

- Hash vs range based? Virtual nodes?
- How do we want to split the nodes? For more info see [TikV explanation](https://www.pingcap.com/blog/building-a-large-scale-distributed-storage-system-based-on-raft/).
    - Option 1: A Node is actually a Raft cluster of $k$ machines. If we have $n$ nodes, we have $n$ Raft clusters and $n*k$ machines in total. Somehow need to communicate between Raft clusters when nodes are added or removed.
    - Option 2: Do what TikV does. Assume we have $n$ nodes and replication factor $k$. Then node's $i$ data is also replicated on the nodes ${i, \dots, i+k}$. Example for $n=4, k=3$:

    | Node 1 | Node 2 | Node 3 | Node 4 |
    | ---    | --- | --- | --- |
    | Data A | Data A | Data A |  - | 
    | -      | Data B | Data B | Data B |
    | Data C | -      | Data C | Data C |
    | Data D | Data D | -      | Data D |

    Seems complicated because now, a node is part of $k$ Raft groups. Additionally, we still need to communicate somehow when a node gets added or removed.
- How do we manage nodes in the ring?
    - Option 1: A master sharder.
    - Option 2: Use a distributed hash table (DHT) like Chord to map keys to nodes.

## 3. Benchmarking

- [ ] Use [Criterion](https://github.com/tikv/raft-rs?tab=readme-ov-file#benchmarks) for benchmarking.
- [ ] Could also use [hyperfine](https://github.com/sharkdp/hyperfine) instead. This would benchmark shell commands instead of Rust code.


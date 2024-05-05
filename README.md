# CS 244B Project

The goal of the project is to build a high-performance distributed key-value store with consistent hashing, sharding, and fault tolerance. We plan to implement the KVS in Rust. For consensus between nodes, we will build on an out of the box implementation of Raft in Rust and adapt it to use gRPCs as the communication mechanism. We will then build our own implementation of consistent hashing and demonstrate (with benchmarks) that our system efficiently and evenly partitions the data across all nodes, even when new nodes are added. We hope to demonstrate that our system is capable of sustaining both sudden addition and removal of nodes while moving a minimal amount of cached data between nodes.

## Overview

To see the distributed kv store at work, run the following:

```bash
sh test-cluster.sh
```

### Folder Structure

- `bin/main.rs` can be used to start a Raft node. This is used by `test-cluster.sh` for testing purposes.
- `lib.rs` contains the starting point and core implementation of creating a Raft node.
- `network` contains all the files needed for a client to interact with the system and for the Raft nodes to talk to each other.
    - `api.rs` contains the applications API that can be called by a client (see `client.rs` for more info.)
    - `management.rs` contains the API used to set up the Raft network. This API is exposed via an Axum HTTP server.
    - `error.rs` contains a custom error type used to make Axum handlers easier to work with.
    - `raft.rs` and `raft_network_impl.rs` implement the communication of Raft nodes. This is done via RPCs.
- `client.rs` implements a basic client that can be used to interact with the distributed kv store. This implementation is used by `tests/test_cluster.rs` to test whether the implementation works as expected.
- `store.rs` implements the Log Store and State Machine used by Raft.

### Tech Stack

This project is using Protobufs (gRPCs) for the communication. You will have to run `brew install protobuf` before being able to compile this repo.

- [Openraft](https://github.com/datafuselabs/openraft) as the underlying Raft protocol.
- [Tonic](https://github.com/hyperium/tonic), a gRPC over HTTP/2 implementation for communication. Need to have `protobuf` installed.
- [Rocksdb](https://crates.io/crates/rocksdb), a library that provides an embeddable, persistent key-value store for fast storage.
- [Axum](https://github.com/tokio-rs/axum) as the web framework.
- [Tracing](https://docs.rs/tracing/latest/tracing/) for asynchronous logging.
- [Criterion.rs](https://github.com/bheisler/criterion.rs) for benchmark tests. Need to have `gnuplot` installed.


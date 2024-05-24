# CS 244B Project

The goal of the project is to build a high-performance distributed key-value store with consistent hashing, sharding, and fault tolerance. We plan to implement the KVS in Rust. For consensus between nodes, we will build on an out of the box implementation of Raft in Rust and adapt it to use RPCs as the communication mechanism. We will then build our own implementation of consistent hashing using Cache Array Routing Protocol (CARP) and demonstrate (with benchmarks) that our system efficiently and evenly partitions the data across all nodes.

We use CARP to create a consistent hash ring for data sharding. Each node on the ring is a Raft cluster, which provides data replication. Routing to the correct cluster is done client-side. The client needs to request the CARP config before using it to send requests to the right place.

## Overview

To see the a single Raft cluster at work, run the following:

```bash
sh tests/test-raft-cluster.sh
```

You can also run `cargo test` to make sure everything works properly. This will test the CARP implementation and the Raft implementation.

### Folder Structure

- `bin/main.rs` can be used to start a Raft node. This is used by `test-cluster.sh` for testing purposes.
- `lib.rs` contains the starting point and core implementation of creating a Raft node.
- `network` contains all the files needed for a client to interact with the system and for the Raft nodes to talk to each other.
    - `api.rs` contains the applications API that can be called by a client (see `client.rs` for more info.)
    - `management.rs` contains the API used to set up the Raft network. This API is exposed via an Axum HTTP server.
    - `error.rs` contains a custom error type used to make Axum handlers easier to work with.
    - `raft.rs` and `raft_network_impl.rs` implement the communication of Raft nodes. This is done via RPCs. `raft.rs` implements the gRPC server. `raft_network_impl.rs` implements the actual communication between nodes.
- `client.rs` implements a basic client that can be used to interact with the distributed kv store. This implementation is used by `tests/test_raft_cluster.rs` to test whether the implementation works as expected.
- `store.rs` implements the Log Store and State Machine used by Raft.
- `carp.rs` implements the Cache Array Routing Protocol.

### Tech Stack

- [Openraft](https://github.com/datafuselabs/openraft) as the underlying Raft consensus protocol.
- Custom CARP implementation. See `carp.rs` for more info.
- [Toy-RPC](https://github.com/minghuaw/toy-rpc), an RPC implementation that mimics golang's `net/rpc`.
- [Rocksdb](https://crates.io/crates/rocksdb), a library that provides an embeddable, persistent key-value store for fast storage.
- [Axum](https://github.com/tokio-rs/axum) as the web framework.
- [Tracing](https://docs.rs/tracing/latest/tracing/) for asynchronous logging.


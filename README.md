# CS 244B Project

The goal of the project is a high-performance distributed key-value store with consistent hashing, sharding, and fault tolerance. We implement the KVS in Rust. For consensus between nodes, we use an out of the box implementation of Raft in Rust and adapt it to use RPCs as the communication mechanism. We build our own implementation of consistent hashing using Cache Array Routing Protocol (CARP) and demonstrate (with benchmarks) that our system efficiently and evenly partitions the data across all nodes.

We use CARP to create a consistent hash ring for data sharding. Each node on the ring is a Raft cluster, which provides data replication. Routing to the correct cluster is done client-side. The client needs to request the CARP config before using it to send requests to the right place.

## Overview

To see the a single Raft cluster at work, run the following:

```bash
sh scripts/test-single-cluster.sh
```

To see multiple Raft clusters at work, run the following:
```bash
sh scripts/test-multiple-clusters.sh
```

You can also run `cargo test` to make sure everything works properly. This will test the CARP implementation and the Raft implementation.

Run `cargo bench` (after running `cargo run --bin admin --release` in a separate terminal to launch the clusters) to run several benchmarks.

### Folder Structure

- `bin/main.rs` can be used to start a Raft node. This is used by `test-single-cluster.sh` for testing purposes.
- `bin/admin.rs` is a sample admin to launch clusters of Raft nodes based on the configuration in `Config.toml`.
- `bin/client.rs` is a sample client application that uses the client in `kvclient.rs` to read/write.
- `lib.rs` contains the starting point and core implementation of creating a Raft node.
- `network` contains all the files needed for a client to interact with the system and for the Raft nodes to talk to each other.
    - `api.rs` contains the applications API that can be called by a client node (see `raft_node.rs` for more info.)
    - `management.rs` contains the API used to set up the Raft network. This API is exposed via an Axum HTTP server.
    - `error.rs` contains a custom error type used to make Axum handlers easier to work with.
    - `raft.rs` and `raft_network_impl.rs` implement the communication of Raft nodes. This is done via RPCs. `raft.rs` implements the gRPC server. `raft_network_impl.rs` implements the actual communication between nodes.
- `raft_node.rs` implements a raft node that can be used externally. This implementation is used by `tests/test_raft_cluster.rs` to test whether the implementation works as expected.
- `store.rs` implements the Log Store and State Machine used by Raft.
- `carp.rs` implements the Cache Array Routing Protocol.
- `kvclient.rs` implements a client that can be used to interact with the distributed key-value store.
- `cluster_manager.rs` implements a cluster manager that starts and shuts down a local cluster (this could be modified to launch across servers on the cloud).

### Tech Stack

- [Openraft](https://github.com/datafuselabs/openraft) as the underlying Raft consensus protocol.
- Custom CARP implementation. See `carp.rs` for more info.
- [Toy-RPC](https://github.com/minghuaw/toy-rpc), an RPC implementation that mimics golang's `net/rpc`.
- [Rocksdb](https://crates.io/crates/rocksdb), a library that provides an embeddable, persistent key-value store for fast storage.
- [Axum](https://github.com/tokio-rs/axum) as the web framework.
- [Tracing](https://docs.rs/tracing/latest/tracing/) for asynchronous logging.


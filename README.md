# CS 244B Project

The goal of the project is to build a high-performance distributed key-value store with consistent hashing, sharding, and fault tolerance. We plan to implement the KVS in Rust. For consensus between nodes, we will build on an out of the box implementation of Raft in Rust and adapt it to use gRPCs as the communication mechanism. We will then build our own implementation of consistent hashing and demonstrate (with benchmarks) that our system efficiently and evenly partitions the data across all nodes, even when new nodes are added. We hope to demonstrate that our system is capable of sustaining both sudden addition and removal of nodes while moving a minimal amount of cached data between nodes.

## Overview

### Folder Structure

### Tech Stack

This project is using Protobufs (gRPCs) for the communication. You will have to run `brew install protobuf` before being able to compile this repo.

- [Openraft](https://github.com/datafuselabs/openraft) as the underlying Raft protocol.
- [Tonic](https://github.com/hyperium/tonic), a gRPC over HTTP/2 implementation for communication.
- [Rocksdb](https://crates.io/crates/rocksdb), a library that provides an embeddable, persistent key-value store for fast storage.
- [Axum](https://github.com/tokio-rs/axum) as the web framework.
- [Tracing](https://docs.rs/tracing/latest/tracing/) for asynchronous logging.
- [Criterion.rs](https://github.com/bheisler/criterion.rs) for benchmark tests.


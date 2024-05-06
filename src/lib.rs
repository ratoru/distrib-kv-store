#![deny(unused_qualifications)]

use std::fmt::Display;
use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;

use openraft::Config;
use tokio::net::TcpListener;
use tonic::transport::Server;

use crate::app::App;
use crate::network::api;
use crate::network::management;
use crate::network::Network;
use crate::store::new_storage;
use crate::store::Request;
use crate::store::Response;
// use raft_grpc::raft_grpc_server::RaftGrpc;
use raft_grpc::raft_grpc_server::RaftGrpcServer;

pub mod raft_grpc {
    tonic::include_proto!("raft_grpc"); // The string specified here must match the proto package name
}

pub mod app;
pub mod client;
pub mod network;
pub mod store;

pub type NodeId = u64;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
pub struct Node {
    pub rpc_addr: String,
    pub api_addr: String,
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Node {{ rpc_addr: {}, api_addr: {} }}",
            self.rpc_addr, self.api_addr
        )
    }
}

pub type SnapshotData = Cursor<Vec<u8>>;

openraft::declare_raft_types!(
    pub TypeConfig:
        D = Request,
        R = Response,
        Node = Node,
);

pub mod typ {
    use openraft::error::Infallible;

    use crate::Node;
    use crate::NodeId;
    use crate::TypeConfig;

    pub type Entry = openraft::Entry<TypeConfig>;

    pub type RaftError<E = Infallible> = openraft::error::RaftError<NodeId, E>;
    pub type RPCError<E = Infallible> = openraft::error::RPCError<NodeId, Node, RaftError<E>>;

    pub type ClientWriteError = openraft::error::ClientWriteError<NodeId, Node>;
    pub type CheckIsLeaderError = openraft::error::CheckIsLeaderError<NodeId, Node>;
    pub type ForwardToLeader = openraft::error::ForwardToLeader<NodeId, Node>;
    pub type InitializeError = openraft::error::InitializeError<NodeId, Node>;

    pub type ClientWriteResponse = openraft::raft::ClientWriteResponse<TypeConfig>;
}

pub type ExampleRaft = openraft::Raft<TypeConfig>;

type AppState = Arc<App>;

pub async fn start_example_raft_node<P>(
    node_id: NodeId,
    dir: P,
    http_addr: String,
    rpc_addr: String,
) -> std::io::Result<()>
where
    P: AsRef<Path>,
{
    // Create a configuration for the raft instance.
    let config = Config {
        heartbeat_interval: 250,
        election_timeout_min: 299,
        ..Default::default()
    };

    let config = Arc::new(config.validate().unwrap());

    let (log_store, state_machine_store) = new_storage(&dir).await;

    let kvs = state_machine_store.data.kvs.clone();

    // Create the network layer that will connect and communicate the raft instances and
    // will be used in conjunction with the store created above.
    let network = Network {};

    // Create a local raft instance.
    let raft = openraft::Raft::new(
        node_id,
        config.clone(),
        network,
        log_store,
        state_machine_store,
    )
    .await
    .unwrap();

    let app_state = Arc::new(App {
        id: node_id,
        api_addr: http_addr.clone(),
        rpc_addr: rpc_addr.clone(),
        raft,
        key_values: kvs,
        config,
    });

    // Create the Raft service.
    let raft_service = network::raft::Raft::new(app_state.clone());

    // Create an application that will store all the instances created above, this will
    // be later used on the axum handlers.
    let app_listener = TcpListener::bind(http_addr).await.unwrap();
    let app = axum::Router::new()
        .nest("/api", api::rest())
        .nest("/cluster", management::rest())
        .with_state(app_state);

    // Serve the application.
    axum::serve(app_listener, app).await.unwrap();
    // Serve the Rafy gRPC server.
    Server::builder()
        .add_service(RaftGrpcServer::new(raft_service))
        .serve(rpc_addr.parse().unwrap())
        .await
        .unwrap();
    Ok(())
}

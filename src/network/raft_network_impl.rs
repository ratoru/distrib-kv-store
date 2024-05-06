use std::any::Any;
use std::fmt::Display;
use std::sync::Arc;

use openraft::error::InstallSnapshotError;
use openraft::error::NetworkError;
use openraft::error::RPCError;
use openraft::error::RaftError;
use openraft::error::RemoteError;
use openraft::network::RPCOption;
use openraft::network::RaftNetwork;
use openraft::network::RaftNetworkFactory;
use openraft::raft::AppendEntriesRequest;
use openraft::raft::AppendEntriesResponse;
use openraft::raft::InstallSnapshotRequest;
use openraft::raft::InstallSnapshotResponse;
use openraft::raft::VoteRequest;
use openraft::raft::VoteResponse;
use openraft::AnyError;
use serde::de::DeserializeOwned;
use tonic::transport::Channel;
use tonic::transport::Endpoint;
use tonic::Code;
use tonic::Request;
use tonic::Status;

use crate::raft_grpc::raft_grpc_client::RaftGrpcClient;
use crate::raft_grpc::RaftRequest;
use crate::Node;
use crate::NodeId;
use crate::TypeConfig;

pub struct Network {}

// RaftNetworkFactory is a singleton responsible for creating RaftNetwork instances for each replication target node. This function should not establish a connection; instead, it should create a client that connects when necessary.
// NOTE: This could be implemented also on `Arc<Network>`, but since it's empty, implemented
// directly.
impl RaftNetworkFactory<TypeConfig> for Network {
    type Network = NetworkConnection;

    #[tracing::instrument(level = "debug", skip_all)]
    async fn new_client(&mut self, target: NodeId, node: &Node) -> Self::Network {
        let addr = format!("http://{}", node.rpc_addr);
        let endpoint = Endpoint::from_shared(addr.clone()).expect("Failed to parse address!");
        let channel = endpoint.connect_lazy();
        let client = RaftGrpcClient::new(channel);

        // let client = Client::dial_websocket(&addr).await.ok();
        tracing::debug!("new_client lazily created for {}", addr.clone());

        NetworkConnection {
            addr,
            client: client.into(),
            target,
        }
    }
}

pub struct NetworkConnection {
    addr: String,
    client: Arc<RaftGrpcClient<Channel>>,
    target: NodeId,
}
// impl NetworkConnection {
//     async fn c<E: std::error::Error + DeserializeOwned>(
//         &self,
//     ) -> Result<RaftGrpcClient<Channel>, RPCError<NodeId, Node, E>> {
//         self.client.clone()
//     }
// }

#[derive(Debug)]
struct ErrWrap(Box<dyn std::error::Error>);

impl Display for ErrWrap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for ErrWrap {}

fn to_error<E: std::error::Error + 'static + Clone>(
    e: Status,
    target: NodeId,
) -> RPCError<NodeId, Node, E> {
    RPCError::Network(NetworkError::new(&e))
    // match e.code() {
    //     Code::IoError => RPCError::Network(NetworkError::new(&e)),
    //     Code::ParseError => RPCError::Network(NetworkError::new(&ErrWrap(e))),
    //     Code::Internal => {
    //         let any: &dyn Any = &e;
    //         let error: &E = any.downcast_ref().unwrap();
    //         RPCError::RemoteError(RemoteError::new(target, error.clone()))
    //     }
    //     e @ (Code::InvalidArgument
    //     | Code::ServiceNotFound
    //     | Code::MethodNotFound
    //     | Code::ExecutionError
    //     | Code::Canceled
    //     | Code::Timeout
    //     | Code::MaxRetriesReached) => RPCError::Network(NetworkError::new(&e)),
    // }
}

// An implementation of RaftNetwork can be considered as a wrapper that invokes the corresponding methods of a remote Raft. It is responsible for sending and receiving messages between Raft nodes.
impl RaftNetwork<TypeConfig> for NetworkConnection {
    #[tracing::instrument(level = "debug", skip_all, err(Debug))]
    async fn append_entries(
        &mut self,
        req: AppendEntriesRequest<TypeConfig>,
        _option: RPCOption,
    ) -> Result<AppendEntriesResponse<NodeId>, RPCError<NodeId, Node, RaftError<NodeId>>> {
        tracing::debug!(req = debug(&req), "append_entries");

        let c = self.client.clone();

        let mes = RaftRequest {
            data: bincode::serialize(&req).expect("fail to serialize"),
        };
        let tonic_req = tonic::Request::new(mes);
        let target = self.target;
        let msg = c
            .append(tonic_req)
            .await
            .map_err(|e| to_error(e, target))?
            .into_inner();
        let resp: AppendEntriesResponse<NodeId> =
            bincode::deserialize(&msg.data).expect("fail to deserialize");

        tracing::debug!("append_entries resp from: id={}: {:?}", self.target, resp);
        Ok(resp)
    }

    #[tracing::instrument(level = "debug", skip_all, err(Debug))]
    async fn install_snapshot(
        &mut self,
        req: InstallSnapshotRequest<TypeConfig>,
        _option: RPCOption,
    ) -> Result<
        InstallSnapshotResponse<NodeId>,
        RPCError<NodeId, Node, RaftError<NodeId, InstallSnapshotError>>,
    > {
        tracing::debug!(req = debug(&req), "install_snapshot");
        self.c()
            .await?
            .raft()
            .snapshot(req)
            .await
            .map_err(|e| to_error(e, self.target))
    }

    #[tracing::instrument(level = "debug", skip_all, err(Debug))]
    async fn vote(
        &mut self,
        req: VoteRequest<NodeId>,
        _option: RPCOption,
    ) -> Result<VoteResponse<NodeId>, RPCError<NodeId, Node, RaftError<NodeId>>> {
        tracing::debug!(req = debug(&req), "vote");
        self.c()
            .await?
            .raft()
            .vote(req)
            .await
            .map_err(|e| to_error(e, self.target))
    }
}

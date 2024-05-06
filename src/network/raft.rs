use std::sync::Arc;

use openraft::raft::AppendEntriesRequest;
use openraft::raft::InstallSnapshotRequest;
use openraft::raft::VoteRequest;
use serde::Serialize;
use tonic::Request;
use tonic::Response;
use tonic::Status;

use crate::raft_grpc::raft_grpc_server::RaftGrpc;
use crate::raft_grpc::RaftReply;
use crate::raft_grpc::RaftRequest;

use crate::app::App;
use crate::typ::RaftError;
use crate::NodeId;
use crate::TypeConfig;

/// Handles the conversion from the raft error type to the RPC RaftReply.
impl<T> From<Result<T, RaftError>> for RaftReply
where
    T: Serialize,
{
    fn from(r: Result<T, RaftError>) -> Self {
        match r {
            Ok(x) => {
                let data = bincode::serialize(&x).expect("fail to serialize");
                RaftReply {
                    data,
                    error: Default::default(),
                }
            }
            Err(e) => {
                let error = bincode::serialize(&e).expect("fail to serialize");
                RaftReply {
                    data: Default::default(),
                    error,
                }
            }
        }
    }
}

/// Raft protocol service.
pub struct Raft {
    app: Arc<App>,
}

impl Raft {
    pub fn new(app: Arc<App>) -> Self {
        Self { app }
    }
}

#[tonic::async_trait]
impl RaftGrpc for Raft {
    async fn vote(&self, request: Request<RaftRequest>) -> Result<Response<RaftReply>, Status> {
        let message = request.into_inner();
        let vote: VoteRequest<NodeId> = bincode::deserialize(&message.data)
            .map_err(|_| Status::data_loss("couldn't deserialize msg"))?;
        // RaftError is converted to RaftReply by the From trait.
        Ok(Response::new(self.app.raft.vote(vote).await.into()))
    }

    async fn append(&self, request: Request<RaftRequest>) -> Result<Response<RaftReply>, Status> {
        tracing::debug!("handle append");
        let message = request.into_inner();
        let req: AppendEntriesRequest<TypeConfig> = bincode::deserialize(&message.data)
            .map_err(|_| Status::data_loss("couldn't deserialize msg"))?;
        self.app.raft.append_entries(req).await
    }

    async fn snapshot(&self, request: Request<RaftRequest>) -> Result<Response<RaftReply>, Status> {
        let message = request.into_inner();
        let req: InstallSnapshotRequest<TypeConfig> = bincode::deserialize(&message.data)
            .map_err(|_| Status::data_loss("couldn't deserialize msg"))?;
        self.app.raft.install_snapshot(req).await
    }
}

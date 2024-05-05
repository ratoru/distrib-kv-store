use std::collections::BTreeMap;
use std::collections::BTreeSet;

use axum::extract::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::routing::post;
use axum::Router;
use openraft::raft::ClientWriteResponse;
use openraft::RaftMetrics;

use crate::network::error::AppError;
use crate::AppState;
use crate::Node;
use crate::NodeId;
use crate::TypeConfig;

// --- Cluster management

/// Creates a new `axum::Router` instance with the configured routes for Cluster Management API.
pub fn rest() -> Router<AppState> {
    Router::new()
        .route("/add-learner", post(add_learner))
        .route("/change-membership", post(change_membership))
        .route("/init", post(init))
        .route("/metrics", get(metrics))
}

/// Add a node as **Learner**.
///
/// A Learner receives log replication from the leader but does not vote.
/// This should be done before adding a node as a member into the cluster
/// (by calling `change-membership`)
async fn add_learner(
    State(state): State<AppState>,
    Json(payload): Json<(NodeId, String, String)>,
) -> Result<(StatusCode, Json<ClientWriteResponse<TypeConfig>>), AppError> {
    let (node_id, api_addr, rpc_addr) = payload;
    let node = Node { rpc_addr, api_addr };
    let res = state.raft.add_learner(node_id, node, true).await?;
    Ok((StatusCode::OK, Json(res)))
}

/// Changes specified learners to members, or remove members.
async fn change_membership(
    State(state): State<AppState>,
    Json(payload): Json<BTreeSet<NodeId>>,
) -> Result<(StatusCode, Json<ClientWriteResponse<TypeConfig>>), AppError> {
    let res = state.raft.change_membership(payload, false).await?;
    Ok((StatusCode::OK, Json(res)))
}

/// Initialize a single-node cluster.
async fn init(State(state): State<AppState>) -> Result<(StatusCode, Json<()>), AppError> {
    let mut nodes = BTreeMap::new();
    let node = Node {
        api_addr: state.api_addr.clone(),
        rpc_addr: state.rpc_addr.clone(),
    };

    nodes.insert(state.id, node);
    let res = state.raft.initialize(nodes).await?;
    Ok((StatusCode::OK, Json(res)))
}

/// Get the latest metrics of the cluster
async fn metrics(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<RaftMetrics<NodeId, Node>>), AppError> {
    let metrics = state.raft.metrics().borrow().clone();
    Ok((StatusCode::OK, Json(metrics)))
}

use axum::extract::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::Router;
use openraft::error::CheckIsLeaderError;
use openraft::error::Infallible;
use openraft::raft::ClientWriteResponse;

use crate::carp::Carp;
use crate::network::error::AppError;
use crate::store;
use crate::AppState;
use crate::Node;
use crate::NodeId;
use crate::TypeConfig;

/// Creates a new `axum::Router` instance with the configured routes for Application API.
///
/// The returned `Router` instance will have the following routes set up:
///
/// - `/write` (HTTP POST)
/// - `/read` (HTTP POST)
/// - `/consistent_read` (HTTP POST)
pub fn rest() -> Router<AppState> {
    Router::new()
        .route("/write", post(write))
        .route("/read", post(read))
        .route("/consistent_read", post(consistent_read))
        .route("/get_hash_ring", post(get_hash_ring))
}

/**
 * Application API
 *
 *  - `POST - /write` saves a value in a key and sync the nodes.
 *  - `POST - /read` attempt to find a value from a given key.
 *  - `POST - /consistent_read` attempt to find a value from a given key ensuring that the value is linearizable.
 *  - `POST - /get_hash_ring` to get the routing table for all nodes.
 */
async fn write(
    State(state): State<AppState>,
    Json(payload): Json<store::Request>,
) -> Result<(StatusCode, Json<ClientWriteResponse<TypeConfig>>), AppError> {
    let res = state.raft.client_write(payload).await?;
    Ok((StatusCode::CREATED, Json(res)))
}

async fn read(
    State(state): State<AppState>,
    Json(key): Json<String>,
) -> Result<(StatusCode, Json<String>), AppError> {
    let kvs = state.key_values.read().await;
    let value = kvs.get(&key);

    let res: Result<String, Infallible> = Ok(value.cloned().unwrap_or_default());

    Ok((StatusCode::OK, Json(res?)))
}

async fn consistent_read(
    State(state): State<AppState>,
    Json(key): Json<String>,
) -> Result<(StatusCode, Json<String>), AppError> {
    let _ = state.raft.ensure_linearizable().await?;

    let kvs = state.key_values.read().await;

    let value = kvs.get(&key);

    let res: Result<String, CheckIsLeaderError<NodeId, Node>> =
        Ok(value.cloned().unwrap_or_default());

    Ok((StatusCode::OK, Json(res?)))
}

async fn get_hash_ring(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Carp>), AppError> {
    let ring_lock = state.hash_ring.read().await;
    let hash_ring: Carp = ring_lock.clone();
    Ok((StatusCode::OK, Json(hash_ring)))
}

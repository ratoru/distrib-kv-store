// use anyhow::Context;
use axum::extract::Json;
use axum::extract::State;
use axum::routing::post;
use axum::Router;
use openraft::error::CheckIsLeaderError;
use openraft::error::Infallible;
use openraft::raft::ClientWriteResponse;

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
}

/**
 * Application API
 *
 * This is where you place your application, you can use the example below to create your
 * API. The current implementation:
 *
 *  - `POST - /write` saves a value in a key and sync the nodes.
 *  - `POST - /read` attempt to find a value from a given key.
 */
async fn write(
    State(state): State<AppState>,
    Json(payload): Json<store::Request>,
) -> Result<Json<ClientWriteResponse<TypeConfig>>, AppError> {
    let res = state.raft.client_write(payload).await?;
    //.context("raft client write failed")?; // Adds context to the error using anyhow.
    Ok(Json(res))
}

async fn read(
    State(state): State<AppState>,
    Json(key): Json<String>,
) -> Result<Json<String>, AppError> {
    let kvs = state.key_values.read().await;
    let value = kvs.get(&key);

    let res: Result<String, Infallible> = Ok(value.cloned().unwrap_or_default());

    // TODO: Example returned Okay regardless of error?
    Ok(Json(res?))
}

async fn consistent_read(
    State(state): State<AppState>,
    Json(key): Json<String>,
) -> Result<String, AppError> {
    // TODO: String might be wrong return type here.
    let ret = state.raft.ensure_linearizable().await;

    match ret {
        Ok(_) => {
            let kvs = state.key_values.read().await;

            let value = kvs.get(&key);

            let res: Result<String, CheckIsLeaderError<NodeId, Node>> =
                Ok(value.cloned().unwrap_or_default());

            // TODO: Example returned Okay regardless of error?
            Ok(res?)
        }
        // TODO: Example returned Okay regardless of error?
        e => Ok(e.err().unwrap().to_string()),
    }
}

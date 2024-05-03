use crate::Node;
use crate::NodeId;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
use openraft::error::CheckIsLeaderError;
use openraft::error::ClientWriteError;
use openraft::error::InitializeError;
use openraft::error::RaftError;
use serde::Serialize;
use thiserror::Error;

/// Error type for the application.
/// Used to convert `openraft::error::RaftError` into `axum::response::Response`.
#[derive(Error, Debug, Serialize)]
pub enum AppError {
    #[error("{0}")]
    ClientWriteError(#[from] RaftError<NodeId, ClientWriteError<NodeId, Node>>),
    #[error("{0}")]
    CheckIsLeaderError(#[from] CheckIsLeaderError<NodeId, Node>),
    #[error("{0}")]
    InitializeError(#[from] RaftError<NodeId, InitializeError<NodeId, Node>>),
    #[error("{0}")]
    Infalible(#[from] openraft::error::Infallible),
}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(self);
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

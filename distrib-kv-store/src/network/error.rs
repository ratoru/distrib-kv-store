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
use serde::Serializer;
use thiserror::Error;

/// Error type for the application.
/// Used to convert `openraft::error::RaftError` into `axum::response::Response`.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    RaftClientWriteError(#[from] RaftError<NodeId, ClientWriteError<NodeId, Node>>),
    #[error("{0}")]
    RaftCheckIsLeaderError(#[from] RaftError<NodeId, CheckIsLeaderError<NodeId, Node>>),
    #[error("{0}")]
    CheckIsLeaderError(#[from] CheckIsLeaderError<NodeId, Node>),
    #[error("{0}")]
    RaftInitializeError(#[from] RaftError<NodeId, InitializeError<NodeId, Node>>),
    #[error("{0}")]
    Infallible(#[from] openraft::error::Infallible),
}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(self);
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

// Custom serialization for `AppError` to avoid leaking internal details to client.
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            AppError::RaftClientWriteError(err) => err.serialize(serializer),
            AppError::RaftCheckIsLeaderError(err) => err.serialize(serializer),
            AppError::CheckIsLeaderError(err) => err.serialize(serializer),
            AppError::RaftInitializeError(err) => err.serialize(serializer),
            AppError::Infallible(err) => err.serialize(serializer),
        }
    }
}

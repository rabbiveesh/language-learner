use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// What an API call can fail with. The body is `{"error": "..."}`.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("not found")]
    NotFound,
    #[error("{0}")]
    BadRequest(String),
    #[error("sign in first")]
    Unauthorized,
    #[error("{0}")]
    Forbidden(String),
    #[error("the passkey check failed: {0}")]
    Passkey(#[from] webauthn_rs::prelude::WebauthnError),
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Schedule(#[from] domain::ScheduleError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self {
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::BadRequest(_) | ApiError::Passkey(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        if status.is_server_error() {
            tracing::error!(error = %self, "request failed");
        }
        (
            status,
            Json(serde_json::json!({ "error": self.to_string() })),
        )
            .into_response()
    }
}

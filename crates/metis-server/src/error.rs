//! HTTP error type and the JSON error envelope.
//!
//! Every error response is `{ "error": { "code", "message", "details"? } }`.
//! This envelope is the contract the MCP layer and clients reuse, so it is
//! defined once here and mapped from [`metis_core::service::ServiceError`].

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use metis_core::service::ServiceError;
use serde_json::json;

/// An API error: an HTTP status, a stable machine code, and a message.
#[derive(Debug)]
pub struct ApiError {
    /// HTTP status code.
    pub status: StatusCode,
    /// Stable, machine-readable code (e.g. `not_found`, `phase_gate`).
    pub code: &'static str,
    /// Human-readable message.
    pub message: String,
    /// Optional structured detail.
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    /// 401.
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "unauthorized",
            message: msg.into(),
            details: None,
        }
    }
    /// 500.
    pub fn internal(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "internal",
            message: msg.into(),
            details: None,
        }
    }
    /// 400.
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "bad_request",
            message: msg.into(),
            details: None,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut error = json!({ "code": self.code, "message": self.message });
        if let Some(details) = self.details {
            error["details"] = details;
        }
        (self.status, Json(json!({ "error": error }))).into_response()
    }
}

impl From<ServiceError> for ApiError {
    fn from(e: ServiceError) -> Self {
        use metis_core::workflow::transition::TransitionError;
        match e {
            ServiceError::NotFound(m) => Self {
                status: StatusCode::NOT_FOUND,
                code: "not_found",
                message: m,
                details: None,
            },
            ServiceError::Validation(m) => Self {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                code: "validation",
                message: m,
                details: None,
            },
            ServiceError::Conflict(m) => Self {
                status: StatusCode::CONFLICT,
                code: "conflict",
                message: m,
                details: None,
            },
            // Phase-gate violations are 409 with the gate spelled out.
            ServiceError::Transition(t) => {
                let details = match &t {
                    TransitionError::NonAdjacent { from, to } => {
                        Some(json!({ "from": from, "to": to, "reason": "non_adjacent" }))
                    }
                    TransitionError::ExitCriteriaNotMet { from } => {
                        Some(json!({ "from": from, "reason": "exit_criteria_not_met" }))
                    }
                    TransitionError::NoNextPhase { phase } => {
                        Some(json!({ "phase": phase, "reason": "no_next_phase" }))
                    }
                    TransitionError::UnknownPhase { phase } => {
                        Some(json!({ "phase": phase, "reason": "unknown_phase" }))
                    }
                };
                Self {
                    status: StatusCode::CONFLICT,
                    code: "phase_gate",
                    message: t.to_string(),
                    details,
                }
            }
            ServiceError::Config(c) => Self {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                code: "invalid_config",
                message: c.to_string(),
                details: None,
            },
            ServiceError::Object(o) => Self::internal(format!("object store: {o}")),
            ServiceError::Db(d) => Self::internal(format!("database: {d}")),
        }
    }
}

//! Bearer-token authentication.
//!
//! Tokens look like `mtk_<43 url-safe chars>` (~256 bits). Only a SHA-256 hash
//! is stored; the plaintext is shown once at issuance. The middleware resolves
//! a presented token to an [`Actor`] and inserts it into request extensions for
//! handlers to read. In local mode auth is disabled and the implicit single
//! user is injected.

use axum::extract::State;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use metis_core::actor::Actor;
use metis_core::service::user;
use rand::distributions::{Alphanumeric, DistString};

use crate::error::ApiError;
use crate::state::AppState;

/// Token prefix marking a Metis token.
pub const TOKEN_PREFIX: &str = "mtk_";

/// Generate a fresh token plaintext. Uses the thread RNG (CSPRNG).
pub fn generate_token() -> String {
    let body = Alphanumeric.sample_string(&mut rand::thread_rng(), 43);
    format!("{TOKEN_PREFIX}{body}")
}

/// Auth middleware. Resolves `Authorization: Bearer <token>` to an [`Actor`],
/// or injects the local user in local mode. Fail-closed: any non-public route
/// without a valid actor is rejected with 401.
pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, ApiError> {
    let actor = if state.config.local {
        // Local mode: implicit single user, no token required.
        Actor::human(state.local_user.expect("local mode sets local_user"))
    } else {
        let token =
            bearer_token(&req).ok_or_else(|| ApiError::unauthorized("missing bearer token"))?;
        state
            .blocking(move |conn| {
                user::resolve_actor(conn, &token)
                    .map_err(ApiError::from)?
                    .ok_or_else(|| ApiError::unauthorized("invalid or revoked token"))
            })
            .await?
    };

    req.extensions_mut().insert(actor);
    Ok(next.run(req).await)
}

/// Extract the bearer token from the Authorization header.
fn bearer_token(req: &Request<axum::body::Body>) -> Option<String> {
    let header = req.headers().get(axum::http::header::AUTHORIZATION)?;
    let value = header.to_str().ok()?;
    value
        .strip_prefix("Bearer ")
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
}

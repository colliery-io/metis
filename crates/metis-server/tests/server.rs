//! Router-level tests via `oneshot`: healthz is public, `/api/v1` requires a
//! valid token, and the admin bootstrap issues working tokens.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use metis_server::config::ServerConfig;
use metis_server::{admin, build_state, router};
use tower::ServiceExt;

fn temp_db() -> (tempfile::TempDir, String) {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("metis.db");
    (dir, path.to_string_lossy().into_owned())
}

async fn body_json(resp: axum::response::Response) -> serde_json::Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn healthz_is_public() {
    let (_dir, url) = temp_db();
    let state = build_state(
        ServerConfig::team(Some(url), Some("127.0.0.1:0".parse().unwrap()), None).unwrap(),
    )
    .unwrap();
    let app = router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_json(resp).await["status"], "ok");
}

#[tokio::test]
async fn protected_route_requires_token() {
    let (_dir, url) = temp_db();
    let state = build_state(
        ServerConfig::team(Some(url), Some("127.0.0.1:0".parse().unwrap()), None).unwrap(),
    )
    .unwrap();
    let app = router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/whoami")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn valid_token_authenticates_and_attributes_agent() {
    let (_dir, url) = temp_db();

    // Bootstrap a user + agent token via the admin path (direct DB).
    let user_id = admin::create_user(&url, "dylan", "Dylan", None, true).unwrap();
    let token = admin::create_token(&url, "dylan", "laptop", Some("claude-code")).unwrap();
    assert!(token.starts_with("mtk_"));

    let state = build_state(
        ServerConfig::team(Some(url), Some("127.0.0.1:0".parse().unwrap()), None).unwrap(),
    )
    .unwrap();
    let app = router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/whoami")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body["user"], user_id.to_string());
    assert_eq!(body["agent"], "claude-code");
}

#[tokio::test]
async fn bad_token_is_rejected() {
    let (_dir, url) = temp_db();
    let state = build_state(
        ServerConfig::team(Some(url), Some("127.0.0.1:0".parse().unwrap()), None).unwrap(),
    )
    .unwrap();
    let app = router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/whoami")
                .header("Authorization", "Bearer mtk_not_a_real_token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn local_mode_needs_no_token() {
    let (_dir, url) = temp_db();
    let state = build_state(ServerConfig::local(
        Some(url),
        Some("127.0.0.1:0".parse().unwrap()),
    ))
    .unwrap();
    let app = router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/whoami")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    // implicit local user, no agent
    assert_eq!(body_json(resp).await["agent"], serde_json::Value::Null);
}

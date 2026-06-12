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

// ----- /api/v1 REST flow (local mode, auth off) -----------------------------

async fn local_app(url: String) -> axum::Router {
    let state = build_state(ServerConfig::local(
        Some(url),
        Some("127.0.0.1:0".parse().unwrap()),
    ))
    .unwrap();
    router(state)
}

fn get(uri: &str) -> Request<Body> {
    Request::builder().uri(uri).body(Body::empty()).unwrap()
}

fn json_req(method: &str, uri: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

#[tokio::test]
async fn rest_project_and_item_lifecycle() {
    let (_dir, url) = temp_db();
    let app = local_app(url).await;

    // create project
    let resp = app
        .clone()
        .oneshot(json_req(
            "POST",
            "/api/v1/projects",
            serde_json::json!({"slug":"metis","name":"Metis"}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let proj = body_json(resp).await;
    assert_eq!(proj["slug"], "metis");
    assert!(proj["config"]["types"]["task"].is_object());

    // create an item
    let resp = app
        .clone()
        .oneshot(json_req(
            "POST",
            "/api/v1/items",
            serde_json::json!({"project":"metis","type":"task","title":"first","body":"# hi","exit_criteria":["done"]}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let item = body_json(resp).await;
    assert_eq!(item["short_code"], "METIS-T-0001");
    assert_eq!(item["body"], "# hi");
    let content_key = item["content_key"].as_str().unwrap().to_string();

    // get it back
    let resp = app
        .clone()
        .oneshot(get("/api/v1/items/METIS-T-0001"))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // transition blocked by unmet criterion -> 409 phase_gate
    let resp = app
        .clone()
        .oneshot(json_req(
            "POST",
            "/api/v1/items/METIS-T-0001/transition",
            serde_json::json!({"phase":"todo"}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    assert_eq!(body_json(resp).await["error"]["code"], "phase_gate");

    // patch with correct If-Match succeeds; mark criterion met
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/api/v1/items/METIS-T-0001")
                .header("content-type", "application/json")
                .header("If-Match", &content_key)
                .body(Body::from(
                    serde_json::json!({"exit_criteria":[{"ordinal":0,"text":"done","met":true}]})
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // now the transition succeeds
    let resp = app
        .clone()
        .oneshot(json_req(
            "POST",
            "/api/v1/items/METIS-T-0001/transition",
            serde_json::json!({"phase":"todo"}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_json(resp).await["phase"], "todo");

    // list filtered by type
    let resp = app
        .clone()
        .oneshot(get("/api/v1/items?project=metis&type=task"))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_json(resp).await.as_array().unwrap().len(), 1);

    // unknown item -> 404
    let resp = app
        .clone()
        .oneshot(get("/api/v1/items/METIS-T-9999"))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn rest_stale_if_match_conflicts() {
    let (_dir, url) = temp_db();
    let app = local_app(url).await;
    app.clone()
        .oneshot(json_req(
            "POST",
            "/api/v1/projects",
            serde_json::json!({"slug":"m","name":"M"}),
        ))
        .await
        .unwrap();
    app.clone()
        .oneshot(json_req(
            "POST",
            "/api/v1/items",
            serde_json::json!({"project":"m","type":"task","title":"t","body":"v1"}),
        ))
        .await
        .unwrap();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/api/v1/items/M-T-0001")
                .header("content-type", "application/json")
                .header("If-Match", "stalehash")
                .body(Body::from(serde_json::json!({"body":"v2"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    assert_eq!(body_json(resp).await["error"]["code"], "conflict");
}

#[tokio::test]
async fn rest_repo_registry_resolve_and_briefing() {
    let (_dir, url) = temp_db();
    let app = local_app(url).await;

    app.clone()
        .oneshot(json_req(
            "POST",
            "/api/v1/projects",
            serde_json::json!({"slug":"metis","name":"Metis"}),
        ))
        .await
        .unwrap();

    // register a repo
    let resp = app
        .clone()
        .oneshot(json_req(
            "POST",
            "/api/v1/projects/metis/repos",
            serde_json::json!({"slug":"core","git_urls":["git@github.com:colliery-io/metis.git"]}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // resolve a different spelling of the same remote
    let resp = app
        .clone()
        .oneshot(get(
            "/api/v1/repos/resolve?remote=https://github.com/colliery-io/metis",
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let ctx = body_json(resp).await;
    assert_eq!(ctx["project"], "metis");
    assert_eq!(ctx["repo"], "core");

    // unregistered remote -> 404
    let resp = app
        .clone()
        .oneshot(get(
            "/api/v1/repos/resolve?remote=git@github.com:other/thing.git",
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    assert_eq!(body_json(resp).await["error"]["code"], "unregistered_repo");

    // create an item touching the repo and move it to active
    app.clone()
        .oneshot(json_req(
            "POST",
            "/api/v1/items",
            serde_json::json!({"project":"metis","type":"task","title":"work","repos":["core"]}),
        ))
        .await
        .unwrap();
    app.clone()
        .oneshot(json_req(
            "POST",
            "/api/v1/items/METIS-T-0001/transition",
            serde_json::json!({"phase":"todo"}),
        ))
        .await
        .unwrap();
    app.clone()
        .oneshot(json_req(
            "POST",
            "/api/v1/items/METIS-T-0001/transition",
            serde_json::json!({"phase":"active"}),
        ))
        .await
        .unwrap();

    // briefing shows it under active
    let resp = app
        .clone()
        .oneshot(get("/api/v1/briefing?repo=core"))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let b = body_json(resp).await;
    assert_eq!(b["project"], "metis");
    assert_eq!(b["active"].as_array().unwrap().len(), 1);
    assert_eq!(b["active"][0]["short_code"], "METIS-T-0001");
    assert_eq!(b["ready"].as_array().unwrap().len(), 0);
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

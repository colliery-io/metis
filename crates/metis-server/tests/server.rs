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

// ----- hosted MCP over HTTP -------------------------------------------------

fn mcp_post(token: Option<&str>, body: serde_json::Value) -> Request<Body> {
    let mut b = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header("content-type", "application/json");
    if let Some(t) = token {
        b = b.header("Authorization", format!("Bearer {t}"));
    }
    b.body(Body::from(body.to_string())).unwrap()
}

#[tokio::test]
async fn mcp_http_requires_token() {
    let (_dir, url) = temp_db();
    let state = build_state(
        ServerConfig::team(Some(url), Some("127.0.0.1:0".parse().unwrap()), None).unwrap(),
    )
    .unwrap();
    let app = router(state);

    let resp = app
        .oneshot(mcp_post(
            None,
            serde_json::json!({"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn mcp_http_get_is_405() {
    // Local mode (auth disabled) so the GET reaches method routing rather than
    // being rejected by the auth layer first — proving there is no SSE handler.
    let (_dir, url) = temp_db();
    let app = local_app(url).await;

    let resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/mcp")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn mcp_http_initialize_list_and_call() {
    let (_dir, url) = temp_db();
    let user_id = admin::create_user(&url, "dev", "Dev", None, true).unwrap();
    let token = admin::create_token(&url, "dev", "laptop", Some("claude-code")).unwrap();
    let state = build_state(
        ServerConfig::team(Some(url), Some("127.0.0.1:0".parse().unwrap()), None).unwrap(),
    )
    .unwrap();
    let app = router(state);

    // initialize
    let resp = app
        .clone()
        .oneshot(mcp_post(
            Some(&token),
            serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let v = body_json(resp).await;
    assert_eq!(v["result"]["serverInfo"]["name"], "Metis");

    // notifications/initialized -> 202, no body
    let resp = app
        .clone()
        .oneshot(mcp_post(
            Some(&token),
            serde_json::json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::ACCEPTED);

    // tools/list includes the renamed tools
    let resp = app
        .clone()
        .oneshot(mcp_post(
            Some(&token),
            serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
        ))
        .await
        .unwrap();
    let v = body_json(resp).await;
    let names: Vec<&str> = v["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"create_item"));
    assert!(names.contains(&"transition_phase"));
    assert!(names.contains(&"briefing"));

    // tools/call: create project + item (attributed to the token's user)
    app.clone()
        .oneshot(mcp_post(
            Some(&token),
            serde_json::json!({"jsonrpc":"2.0","id":3,"method":"tools/call",
                "params":{"name":"create_project","arguments":{"slug":"metis","name":"Metis"}}}),
        ))
        .await
        .unwrap();
    let resp = app
        .clone()
        .oneshot(mcp_post(
            Some(&token),
            serde_json::json!({"jsonrpc":"2.0","id":4,"method":"tools/call",
                "params":{"name":"create_item","arguments":{"project":"metis","type":"task","title":"via http mcp","exit_criteria":["ship"]}}}),
        ))
        .await
        .unwrap();
    let v = body_json(resp).await;
    let text = v["result"]["content"][0]["text"].as_str().unwrap();
    let item: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(item["short_code"], "METIS-T-0001");
    assert_eq!(item["created_by"], user_id.to_string());

    // gated transition -> result with isError true (not a protocol error)
    let resp = app
        .clone()
        .oneshot(mcp_post(
            Some(&token),
            serde_json::json!({"jsonrpc":"2.0","id":5,"method":"tools/call",
                "params":{"name":"transition_phase","arguments":{"short_code":"METIS-T-0001","phase":"todo"}}}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let v = body_json(resp).await;
    assert_eq!(v["result"]["isError"], true);
    assert!(v["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("exit criteria"));

    // unknown method -> JSON-RPC -32601
    let resp = app
        .clone()
        .oneshot(mcp_post(
            Some(&token),
            serde_json::json!({"jsonrpc":"2.0","id":6,"method":"nope/nope","params":{}}),
        ))
        .await
        .unwrap();
    let v = body_json(resp).await;
    assert_eq!(v["error"]["code"], -32601);
}

#[tokio::test]
async fn rest_search_and_saved_views() {
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
    app.clone()
        .oneshot(json_req("POST", "/api/v1/items", serde_json::json!({"project":"metis","type":"task","title":"flux capacitor","body":"time travel"})))
        .await
        .unwrap();
    app.clone()
        .oneshot(json_req("POST", "/api/v1/items", serde_json::json!({"project":"metis","type":"bug","title":"paint shed","body":"unrelated"})))
        .await
        .unwrap();

    // search via /items?q=
    let resp = app
        .clone()
        .oneshot(get("/api/v1/items?q=flux"))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let hits = body_json(resp).await;
    assert_eq!(hits.as_array().unwrap().len(), 1);
    assert_eq!(hits[0]["title"], "flux capacitor");

    // save a view selecting only bugs
    let resp = app
        .clone()
        .oneshot(json_req(
            "POST",
            "/api/v1/projects/metis/views",
            serde_json::json!({"name":"Bugs","query":{"type":"bug"},"shared":true}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let view = body_json(resp).await;
    let view_id = view["id"].as_str().unwrap().to_string();
    assert_eq!(view["name"], "Bugs");

    // list views
    let resp = app
        .clone()
        .oneshot(get("/api/v1/projects/metis/views"))
        .await
        .unwrap();
    assert_eq!(body_json(resp).await.as_array().unwrap().len(), 1);

    // run the view -> only the bug
    let resp = app
        .clone()
        .oneshot(get(&format!("/api/v1/views/{view_id}/items")))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let items = body_json(resp).await;
    assert_eq!(items.as_array().unwrap().len(), 1);
    assert_eq!(items[0]["item_type"], "bug");

    // delete the view
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/views/{view_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    let resp = app
        .clone()
        .oneshot(get("/api/v1/projects/metis/views"))
        .await
        .unwrap();
    assert!(body_json(resp).await.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn oversized_body_is_rejected() {
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

    // a body well over the 1 MiB limit -> 413 (not buffered/processed)
    let huge = "x".repeat(2 * 1024 * 1024);
    let body = serde_json::json!({"project":"metis","type":"task","title":"big","body":huge});
    let resp = app
        .oneshot(json_req("POST", "/api/v1/items", body))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
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

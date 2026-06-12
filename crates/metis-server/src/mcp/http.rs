//! Streamable-HTTP MCP transport, implemented natively on axum (METIS-T-0135).
//!
//! `POST /mcp` speaks MCP over JSON-RPC. It sits behind the same bearer-token
//! middleware as `/api/v1`, so each request carries its own [`Actor`] — giving
//! true per-connection PAT auth (mutations attributed to the connecting user).
//! Tool listing and dispatch are shared with the stdio path
//! ([`super::call_tool`] / [`super::tool_list`]).
//!
//! The server has no server→client messages, so it is stateless: every POST
//! gets an `application/json` response, notifications get `202 Accepted`, and
//! `GET /mcp` is `405` (no SSE stream). Single messages only — JSON-RPC
//! batching was removed from the MCP spec.

use axum::body::Bytes;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use metis_core::actor::Actor;
use rust_mcp_sdk::schema::{schema_utils::CallToolError, CallToolResult, TextContent};
use serde_json::{json, Value};

use super::tools::McpState;
use crate::state::AppState;

/// Handle one MCP JSON-RPC message over HTTP.
pub async fn post_mcp(
    State(app): State<AppState>,
    Extension(actor): Extension<Actor>,
    body: Bytes,
) -> Response {
    let msg: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return Json(rpc_error(Value::Null, -32700, "parse error")).into_response(),
    };
    if !msg.is_object() {
        // No batching: a top-level array (or non-object) is an invalid request.
        return Json(rpc_error(
            Value::Null,
            -32600,
            "invalid request (batches unsupported)",
        ))
        .into_response();
    }

    // No `id` field ⇒ a notification: acknowledge, no body.
    let Some(id) = msg.get("id").cloned() else {
        return StatusCode::ACCEPTED.into_response();
    };
    let method = msg
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let params = msg.get("params").cloned().unwrap_or(Value::Null);

    let state = McpState {
        pool: app.pool.clone(),
        store: app.store.clone(),
        actor,
    };

    let outcome: Result<Value, (i64, String)> = match method {
        "initialize" => Ok(serde_json::to_value(super::server_info()).unwrap_or_default()),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": super::tool_list() })),
        "tools/call" => {
            let name = params
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let args = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            // Per MCP, tool execution failures are an in-band result with
            // isError, not a JSON-RPC protocol error.
            let result = match super::call_tool(&state, &name, args).await {
                Ok(r) => r,
                Err(e) => tool_error_result(&e),
            };
            Ok(serde_json::to_value(result).unwrap_or_default())
        }
        other => Err((-32601, format!("method not found: {other}"))),
    };

    let envelope = match outcome {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err((code, message)) => rpc_error(id, code, &message),
    };
    Json(envelope).into_response()
}

/// A JSON-RPC error envelope.
fn rpc_error(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

/// Wrap a tool error as a `CallToolResult` with `isError`, preserving the
/// message (which already distinguishes not-found / validation / phase-gate /
/// conflict).
fn tool_error_result(e: &CallToolError) -> CallToolResult {
    CallToolResult {
        content: vec![TextContent::new(e.to_string(), None, None).into()],
        is_error: Some(true),
        meta: None,
        structured_content: None,
    }
}

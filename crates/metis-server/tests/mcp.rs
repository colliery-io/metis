//! Exercises the MCP tool layer against a temp SQLite DB. Calls each tool's
//! `run` (exactly what the stdio dispatch invokes) so the service-backed
//! behavior — create, read, gated transition, edit, list — is covered without
//! standing up a stdio transport.

use std::sync::Arc;

use metis_core::actor::Actor;
use metis_core::objects::{DbBlobObjectStore, ObjectStore};
use metis_core::{db, Pool};
use metis_server::mcp::tools::{
    BriefingTool, CreateItemTool, CreateProjectTool, EditItemTool, ListItemsTool, McpState,
    ReadItemTool, ResolveRepoTool, TransitionPhaseTool,
};

fn state() -> (tempfile::TempDir, McpState) {
    let dir = tempfile::TempDir::new().unwrap();
    let url = dir.path().join("metis.db").to_string_lossy().into_owned();
    let pool: Pool = db::connect_and_migrate(&url).unwrap();
    let user = {
        let mut conn = pool.get().unwrap();
        metis_core::service::user::create_user(&mut conn, "dev", "Dev", None, true).unwrap()
    };
    let store: Arc<dyn ObjectStore + Send + Sync> = Arc::new(DbBlobObjectStore::new());
    let st = McpState {
        pool,
        store,
        actor: Actor::agent(user.id.0, "claude-code"),
    };
    (dir, st)
}

/// Parse the single text-content block of a tool result as JSON.
fn result_json(r: rust_mcp_sdk::schema::CallToolResult) -> serde_json::Value {
    let block = serde_json::to_value(&r.content[0]).unwrap();
    let text = block["text"].as_str().expect("text content");
    serde_json::from_str(text).unwrap()
}

#[tokio::test]
async fn mcp_tool_lifecycle() {
    let (_dir, st) = state();

    // create project
    CreateProjectTool {
        slug: "metis".into(),
        name: "Metis".into(),
    }
    .run(&st)
    .await
    .unwrap();

    // create item
    let created = CreateItemTool {
        project: "metis".into(),
        item_type: "task".into(),
        title: "first".into(),
        body: "# hi".into(),
        parent: None,
        assignee: None,
        tags: vec!["x".into()],
        repos: vec![],
        exit_criteria: vec!["done".into()],
    }
    .run(&st)
    .await
    .unwrap();
    let created = result_json(created);
    assert_eq!(created["short_code"], "METIS-T-0001");
    assert_eq!(created["body"], "# hi");

    // read it back
    let read = result_json(
        ReadItemTool {
            short_code: "METIS-T-0001".into(),
        }
        .run(&st)
        .await
        .unwrap(),
    );
    assert_eq!(read["title"], "first");
    assert_eq!(read["tags"][0], "x");

    // transition blocked by unmet criterion -> tool error carrying the gate message
    let err = TransitionPhaseTool {
        short_code: "METIS-T-0001".into(),
        phase: Some("todo".into()),
        force: false,
    }
    .run(&st)
    .await
    .unwrap_err();
    assert!(
        format!("{err}").contains("exit criteria"),
        "expected phase-gate message, got: {err}"
    );

    // mark criterion met via edit, then transition succeeds
    EditItemTool {
        short_code: "METIS-T-0001".into(),
        title: None,
        body: None,
        tags: None,
        exit_criteria: Some(vec![metis_server::mcp::tools::ExitCriterionArg {
            ordinal: 0,
            text: "done".into(),
            met: true,
        }]),
    }
    .run(&st)
    .await
    .unwrap();

    let moved = result_json(
        TransitionPhaseTool {
            short_code: "METIS-T-0001".into(),
            phase: Some("todo".into()),
            force: false,
        }
        .run(&st)
        .await
        .unwrap(),
    );
    assert_eq!(moved["phase"], "todo");

    // list by type
    let listed = result_json(
        ListItemsTool {
            project: Some("metis".into()),
            item_type: Some("task".into()),
            phase: None,
            assignee: None,
            tag: None,
            repo: None,
            q: None,
            include_archived: false,
            limit: None,
            offset: None,
        }
        .run(&st)
        .await
        .unwrap(),
    );
    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert_eq!(listed[0]["short_code"], "METIS-T-0001");

    // read of unknown item -> tool error
    let err = ReadItemTool {
        short_code: "METIS-T-9999".into(),
    }
    .run(&st)
    .await
    .unwrap_err();
    assert!(format!("{err}").contains("not found"));
}

#[tokio::test]
async fn mcp_resolve_repo_and_briefing() {
    let (_dir, st) = state();

    CreateProjectTool {
        slug: "metis".into(),
        name: "Metis".into(),
    }
    .run(&st)
    .await
    .unwrap();

    // register via the service (registration isn't an MCP tool); resolve via MCP
    {
        let mut conn = st.pool.get().unwrap();
        metis_core::service::repo::register(
            &mut conn,
            "metis",
            "core",
            vec!["git@github.com:org/metis.git".into()],
        )
        .unwrap();
    }

    let resolved = result_json(
        ResolveRepoTool {
            remote: "https://github.com/org/metis".into(),
        }
        .run(&st)
        .await
        .unwrap(),
    );
    assert_eq!(resolved["matched"], true);
    assert_eq!(resolved["context"]["repo"], "core");

    let unmatched = result_json(
        ResolveRepoTool {
            remote: "git@github.com:nope/x.git".into(),
        }
        .run(&st)
        .await
        .unwrap(),
    );
    assert_eq!(unmatched["matched"], false);

    // an active item touching the repo shows up in the briefing
    let item = result_json(
        CreateItemTool {
            project: "metis".into(),
            item_type: "task".into(),
            title: "w".into(),
            body: String::new(),
            parent: None,
            assignee: None,
            tags: vec![],
            repos: vec!["core".into()],
            exit_criteria: vec![],
        }
        .run(&st)
        .await
        .unwrap(),
    );
    let sc = item["short_code"].as_str().unwrap().to_string();
    TransitionPhaseTool {
        short_code: sc.clone(),
        phase: Some("todo".into()),
        force: false,
    }
    .run(&st)
    .await
    .unwrap();
    TransitionPhaseTool {
        short_code: sc.clone(),
        phase: Some("active".into()),
        force: false,
    }
    .run(&st)
    .await
    .unwrap();

    let b = result_json(
        BriefingTool {
            repo: "core".into(),
        }
        .run(&st)
        .await
        .unwrap(),
    );
    assert_eq!(b["active"].as_array().unwrap().len(), 1);
    assert_eq!(b["active"][0]["short_code"], sc);
}

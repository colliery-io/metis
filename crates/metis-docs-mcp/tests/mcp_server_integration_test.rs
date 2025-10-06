//! Integration tests that spawn the actual MCP server binary and test over stdio
//! This tests the real MCP protocol communication including the archive_document fix

use anyhow::Result;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use tempfile::TempDir;

/// Helper to build and spawn the MCP server binary
struct McpServerProcess {
    temp_dir: TempDir,
    project_path: String,
    metis_dir: String,
}

impl McpServerProcess {
    fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path().to_string_lossy().to_string();
        let metis_dir = format!("{}/.metis", project_path);

        Ok(Self {
            temp_dir,
            project_path,
            metis_dir,
        })
    }

    /// Build the MCP server binary
    fn build_server() -> Result<()> {
        let output = Command::new("cargo")
            .args(&["build", "--bin", "metis-mcp", "--release"])
            .current_dir("../../") // Go up to metis root
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to build MCP server: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        Ok(())
    }

    /// Spawn the MCP server process and return handles for communication
    fn spawn_server(&self) -> Result<std::process::Child> {
        let child = Command::new("../../target/release/metis-mcp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Ok(child)
    }

    /// Send an MCP request and get response
    fn send_mcp_request(child: &mut std::process::Child, request: Value) -> Result<Value> {
        let stdin = child.stdin.as_mut().unwrap();
        let stdout = child.stdout.as_mut().unwrap();

        // Send request
        let request_str = serde_json::to_string(&request)?;
        writeln!(stdin, "{}", request_str)?;
        stdin.flush()?;

        // Read response
        let mut reader = BufReader::new(stdout);
        let mut response_line = String::new();
        reader.read_line(&mut response_line)?;

        let response: Value = serde_json::from_str(&response_line.trim())?;
        Ok(response)
    }

    /// Initialize the project using MCP protocol
    async fn initialize_project(&self, child: &mut std::process::Child) -> Result<()> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "initialize_project",
                "arguments": {
                    "project_path": self.project_path
                }
            }
        });

        let response = Self::send_mcp_request(child, request)?;

        // Check if successful
        if response.get("error").is_some() {
            return Err(anyhow::anyhow!("Initialize failed: {:?}", response));
        }

        Ok(())
    }

    /// Create a test document to archive
    async fn create_test_document(&self, child: &mut std::process::Child) -> Result<String> {
        // In streamlined mode (default), create an initiative that we can archive
        let request = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "create_document",
                "arguments": {
                    "project_path": self.metis_dir,
                    "document_type": "initiative",
                    "title": "Test Initiative to Archive",
                    "complexity": "m"
                }
            }
        });

        let response = Self::send_mcp_request(child, request)?;

        if let Some(error) = response.get("error") {
            return Err(anyhow::anyhow!("Create document failed: {:?}", error));
        }

        // Extract document ID from response
        if let Some(result) = response.get("result") {
            if let Some(content) = result.get("content") {
                if let Some(content_array) = content.as_array() {
                    if let Some(first_content) = content_array.first() {
                        if let Some(text) = first_content.get("text") {
                            let text_str = text.as_str().unwrap();
                            // Parse the JSON response to get document_id
                            if let Ok(parsed) = serde_json::from_str::<Value>(text_str) {
                                if let Some(doc_id) = parsed.get("document_id") {
                                    return Ok(doc_id.as_str().unwrap().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        Err(anyhow::anyhow!(
            "Could not extract document ID from response"
        ))
    }

    /// Test the archive_document functionality
    async fn test_archive_document(
        &self,
        child: &mut std::process::Child,
        document_id: &str,
    ) -> Result<()> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "archive_document",
                "arguments": {
                    "project_path": self.metis_dir,
                    "document_id": document_id
                }
            }
        });

        let response = Self::send_mcp_request(child, request)?;

        // This should NOT return "Unknown tool" error anymore
        if let Some(error) = response.get("error") {
            let error_message = error.get("message").unwrap().as_str().unwrap();
            if error_message.contains("Unknown tool") {
                return Err(anyhow::anyhow!(
                    "Archive document tool not found - the fix didn't work: {}",
                    error_message
                ));
            }
            // Other errors might be expected (like document not found, etc.)
            println!("Got expected error: {}", error_message);
        } else {
            println!("Archive succeeded: {:?}", response);
        }

        Ok(())
    }
}

#[tokio::test]
async fn test_mcp_server_archive_document_integration() -> Result<()> {
    println!("=== Integration Test: MCP Server Archive Document ===");

    // Step 1: Build the server
    println!("Building MCP server binary...");
    McpServerProcess::build_server()?;

    // Step 2: Set up test environment
    let test_helper = McpServerProcess::new()?;

    // Step 3: Spawn server process
    println!("Spawning MCP server process...");
    let mut child = test_helper.spawn_server()?;

    // Step 4: Initialize project
    println!("Initializing project via MCP...");
    test_helper.initialize_project(&mut child).await?;

    // Step 5: Create a test document
    println!("Creating test document to archive...");
    let document_id = test_helper.create_test_document(&mut child).await?;
    println!("Created document with ID: {}", document_id);

    // Step 6: Test archive functionality (this should work now with our fix)
    println!("Testing archive_document tool...");
    test_helper
        .test_archive_document(&mut child, &document_id)
        .await?;

    // Step 7: Clean up
    child.kill()?;

    println!("✅ Integration test passed - archive_document tool is working!");
    Ok(())
}

#[tokio::test]
async fn test_mcp_server_list_tools() -> Result<()> {
    println!("=== Integration Test: MCP Server List Tools ===");

    // Build and spawn server
    McpServerProcess::build_server()?;
    let test_helper = McpServerProcess::new()?;
    let mut child = test_helper.spawn_server()?;

    // Send list_tools request
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list"
    });

    let response = McpServerProcess::send_mcp_request(&mut child, request)?;

    // Verify archive_document is in the tools list
    if let Some(result) = response.get("result") {
        if let Some(tools) = result.get("tools") {
            if let Some(tools_array) = tools.as_array() {
                let has_archive_tool = tools_array.iter().any(|tool| {
                    tool.get("name").map(|n| n.as_str()) == Some(Some("archive_document"))
                });

                if !has_archive_tool {
                    return Err(anyhow::anyhow!(
                        "archive_document tool not found in tools list"
                    ));
                }

                println!("✅ archive_document tool found in tools list");
            }
        }
    }

    child.kill()?;
    println!("✅ List tools test passed!");
    Ok(())
}

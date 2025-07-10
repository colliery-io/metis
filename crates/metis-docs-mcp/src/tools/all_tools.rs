use super::hello::HelloWorldTool;
use rust_mcp_sdk::tool_box;

// Generate the combined MetisTools enum
tool_box!(
    MetisTools,
    [
        HelloWorldTool
    ]
);
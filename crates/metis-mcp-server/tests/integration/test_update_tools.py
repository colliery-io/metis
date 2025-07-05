#!/usr/bin/env python3
import json
import subprocess
import sys
import os

def send_mcp_request(request):
    env = os.environ.copy()
    env['METIS_WORKSPACE_ROOT'] = '/tmp/test-workspace'
    
    process = subprocess.Popen(
        ['cargo', 'run', '--bin', 'metis-mcp-server'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=env,
        cwd='/Users/dstorey/Desktop/colliery/metis'
    )
    
    request_str = json.dumps(request) + '\n'
    stdout, stderr = process.communicate(request_str)
    
    try:
        return json.loads(stdout.strip())
    except json.JSONDecodeError:
        print(f"Raw stdout: {stdout}")
        return None

# Test tools/list to see all tools
print("Testing tools/list with update tools...")
list_tools_request = {
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/list",
    "params": {}
}

response = send_mcp_request(list_tools_request)
print("Available tools:")
if response and 'result' in response and 'tools' in response['result']:
    for tool in response['result']['tools']:
        print(f"  - {tool['name']}: {tool['description']}")
else:
    print("No tools found")

# Test update_document_content tool
print("\nTesting update_document_content tool...")
update_content_request = {
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
        "name": "update_document_content",
        "arguments": {
            "project_name": "my-project",
            "document_path": "vision.md",
            "section_heading": "Purpose",
            "new_content": "To test the Metis MCP server update functionality and demonstrate document content updates."
        }
    }
}

response = send_mcp_request(update_content_request)
print("Update content response:", json.dumps(response, indent=2))

# Test update_exit_criterion tool
print("\nTesting update_exit_criterion tool...")
update_exit_request = {
    "jsonrpc": "2.0",
    "id": 4,
    "method": "tools/call",
    "params": {
        "name": "update_exit_criterion",
        "arguments": {
            "project_name": "my-project",
            "document_path": "vision.md",
            "criterion_text": "Purpose and success criteria are clearly defined",
            "completed": True
        }
    }
}

response = send_mcp_request(update_exit_request)
print("Update exit criterion response:", json.dumps(response, indent=2))

# Test update_blocked_by tool
print("\nTesting update_blocked_by tool...")
update_blocked_request = {
    "jsonrpc": "2.0",
    "id": 5,
    "method": "tools/call",
    "params": {
        "name": "update_blocked_by",
        "arguments": {
            "project_name": "my-project",
            "document_path": "vision.md",
            "blocked_by": ["[[Strategy Document]]", "[[Initial Requirements]]"]
        }
    }
}

response = send_mcp_request(update_blocked_request)
print("Update blocked_by response:", json.dumps(response, indent=2))
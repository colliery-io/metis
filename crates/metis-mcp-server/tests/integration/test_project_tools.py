#!/usr/bin/env python3
import json
import subprocess
import sys
import os

def send_mcp_request(request):
    process = subprocess.Popen(
        ['cargo', 'run', '--bin', 'metis-mcp-server'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        cwd='/Users/dstorey/Desktop/colliery/metis'
    )
    
    request_str = json.dumps(request) + '\n'
    stdout, stderr = process.communicate(request_str)
    
    try:
        return json.loads(stdout.strip())
    except json.JSONDecodeError:
        print(f"Raw stdout: {stdout}")
        return None

# Test tools/list request
print("Testing tools/list...")
list_tools_request = {
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/list",
    "params": {}
}

response = send_mcp_request(list_tools_request)
print("List tools response:", json.dumps(response, indent=2))

# list_projects tool removed - using direct paths now

# Test initialize_project tool
print("\nTesting initialize_project tool...")
init_project_request = {
    "jsonrpc": "2.0",
    "id": 4,
    "method": "tools/call",
    "params": {
        "name": "initialize_project",
        "arguments": {
            "project_path": "/tmp/test-project",
            "project_name": "test-project",
            "description": "A test project for MCP server"
        }
    }
}

response = send_mcp_request(init_project_request)
print("Initialize project response:", json.dumps(response, indent=2))

# Direct path approach - no project listing needed
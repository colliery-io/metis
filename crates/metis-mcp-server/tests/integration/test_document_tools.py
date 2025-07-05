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

# Test tools/list to see new document tools
print("Testing tools/list with document tools...")
list_tools_request = {
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/list",
    "params": {}
}

response = send_mcp_request(list_tools_request)
print("List tools response:")
if response and 'result' in response and 'tools' in response['result']:
    for tool in response['result']['tools']:
        print(f"  - {tool['name']}: {tool['description']}")
else:
    print("No tools found")

# Test create_document tool
print("\nTesting create_document tool...")
create_doc_request = {
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
        "name": "create_document",
        "arguments": {
            "project_name": "my-project",
            "document_type": "strategy",
            "title": "API Architecture Strategy",
            "risk_level": "medium"
        }
    }
}

response = send_mcp_request(create_doc_request)
print("Create document response:", json.dumps(response, indent=2))

# Test validate_document tool on vision.md
print("\nTesting validate_document tool on existing vision.md...")
validate_doc_request = {
    "jsonrpc": "2.0",
    "id": 4,
    "method": "tools/call",
    "params": {
        "name": "validate_document",
        "arguments": {
            "project_name": "my-project",
            "document_path": "vision.md"
        }
    }
}

response = send_mcp_request(validate_doc_request)
print("Validate document response:", json.dumps(response, indent=2))

# Test validate_document tool on non-existent document
print("\nTesting validate_document tool on non-existent document...")
validate_nonexistent_request = {
    "jsonrpc": "2.0",
    "id": 5,
    "method": "tools/call",
    "params": {
        "name": "validate_document",
        "arguments": {
            "project_name": "my-project",
            "document_path": "nonexistent.md"
        }
    }
}

response = send_mcp_request(validate_nonexistent_request)
print("Validate non-existent document response:", json.dumps(response, indent=2))
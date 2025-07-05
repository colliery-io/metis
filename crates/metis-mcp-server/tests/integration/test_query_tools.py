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

# Test tools/list to see all 11 tools (list_projects removed)
print("Testing tools/list with all tools...")
list_tools_request = {
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/list",
    "params": {}
}

response = send_mcp_request(list_tools_request)
print("Available tools:")
if response and 'result' in response and 'tools' in response['result']:
    for i, tool in enumerate(response['result']['tools'], 1):
        print(f"  {i:2}. {tool['name']}")
    print(f"\nTotal: {len(response['result']['tools'])} tools (list_projects removed)")
else:
    print("No tools found")

# Test list_documents tool
print("\nTesting list_documents tool...")
list_docs_request = {
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
        "name": "list_documents",
        "arguments": {
            "project_path": "/tmp/test-project/.metis"
        }
    }
}

response = send_mcp_request(list_docs_request)
print("List documents response:", json.dumps(response, indent=2))

# Test search_documents tool
print("\nTesting search_documents tool...")
search_docs_request = {
    "jsonrpc": "2.0",
    "id": 4,
    "method": "tools/call",
    "params": {
        "name": "search_documents",
        "arguments": {
            "project_path": "/tmp/test-project/.metis",
            "query": "test",
            "limit": 10
        }
    }
}

response = send_mcp_request(search_docs_request)
print("Search documents response:", json.dumps(response, indent=2))

# Test search for specific content
print("\nTesting search for 'Purpose' content...")
search_purpose_request = {
    "jsonrpc": "2.0",
    "id": 5,
    "method": "tools/call",
    "params": {
        "name": "search_documents",
        "arguments": {
            "project_path": "/tmp/test-project/.metis",
            "query": "Purpose"
        }
    }
}

response = send_mcp_request(search_purpose_request)
print("Search for Purpose response:", json.dumps(response, indent=2))
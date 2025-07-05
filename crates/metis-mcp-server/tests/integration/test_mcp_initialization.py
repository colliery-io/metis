#!/usr/bin/env python3
import json
import subprocess
import sys
import os

def send_mcp_request(request):
    # Start the MCP server - no environment variables needed
    process = subprocess.Popen(
        ['cargo', 'run', '--bin', 'metis-mcp-server'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        cwd='/Users/dstorey/Desktop/colliery/metis'
    )
    
    # Send the request
    request_str = json.dumps(request) + '\n'
    stdout, stderr = process.communicate(request_str)
    
    if stderr:
        print(f"Server stderr: {stderr}", file=sys.stderr)
    
    try:
        return json.loads(stdout.strip())
    except json.JSONDecodeError:
        print(f"Raw stdout: {stdout}")
        return None

# Test 1: Initialize MCP connection
print("Testing MCP server...")

initialize_request = {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {
            "name": "test-client",
            "version": "1.0.0"
        }
    }
}

print("Sending initialize request...")
response = send_mcp_request(initialize_request)
print("Initialize response:", json.dumps(response, indent=2))
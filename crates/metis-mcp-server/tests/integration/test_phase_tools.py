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

# Test tools/list to see all 10 tools
print("Testing tools/list with phase tools...")
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
        print(f"  {i}. {tool['name']}: {tool['description']}")
    print(f"\nTotal: {len(response['result']['tools'])} tools")
else:
    print("No tools found")

# Test validate_exit_criteria tool first
print("\nTesting validate_exit_criteria tool...")
validate_exit_request = {
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
        "name": "validate_exit_criteria",
        "arguments": {
            "project_name": "my-project",
            "document_path": "vision.md"
        }
    }
}

response = send_mcp_request(validate_exit_request)
print("Validate exit criteria response:", json.dumps(response, indent=2))

# Test check_phase_transition tool
print("\nTesting check_phase_transition tool...")
check_phase_request = {
    "jsonrpc": "2.0",
    "id": 4,
    "method": "tools/call",
    "params": {
        "name": "check_phase_transition",
        "arguments": {
            "project_name": "my-project",
            "document_path": "vision.md",
            "target_phase": "review"
        }
    }
}

response = send_mcp_request(check_phase_request)
print("Check phase transition response:", json.dumps(response, indent=2))

# Test transition_phase tool
print("\nTesting transition_phase tool...")
transition_phase_request = {
    "jsonrpc": "2.0",
    "id": 5,
    "method": "tools/call",
    "params": {
        "name": "transition_phase",
        "arguments": {
            "project_name": "my-project",
            "document_path": "vision.md",
            "new_phase": "review",
            "force": False
        }
    }
}

response = send_mcp_request(transition_phase_request)
print("Transition phase response:", json.dumps(response, indent=2))
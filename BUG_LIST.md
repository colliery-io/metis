I'm getting the following error - i believe due to the fact that the id length is truncated and different from the directory structure causing a conflict. This only appears to be happening in the MCP server, the TUI appears functional

⏺ metis - create_document (MCP)(project_path: /Users/dstorey/Desktop/colliery/mimir/.metis, document_type: task, title: Analyze 5etools JSON Schema Structure, parent_id: phase-0-data-ingestion-and-storage)
  ⎿  Error: Not found: Parent initiative 'phase-0-data-ingestion-and-storage' not found

⏺ Let me check if the database is out of sync and resync it:

⏺ Read(.metis/strategies/mvp-development-strategy/initiatives/phase-0-data-ingestion-and-storage/initiative.md)
  ⎿  Read 20 lines (ctrl+r to expand)

⏺ The file exists and is in decompose phase. Let me try searching for the document to see if there's a sync issue:


-------

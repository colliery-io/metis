---
id: enhanced-search-capabilities
level: initiative
title: "Enhanced Search Capabilities"
short_code: "METIS-I-0018"
created_at: 2025-12-31T15:03:23.913810+00:00
updated_at: 2026-01-01T21:28:03.605996+00:00
parent: METIS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/active"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: enhanced-search-capabilities
---

# Enhanced Search Capabilities Initiative

## Context

Metis currently has basic full-text search using SQLite FTS5, searching across document content, title, and type. The search is functional but limited—it cannot search by tags, doesn't support boolean operators, returns unsorted results, and cannot scope searches within document hierarchies.

As workspaces grow with more documents, users need more powerful ways to find relevant information quickly.

### Current State
- FTS5 indexes: content, title, document_type
- Tokenizer: porter unicode61 (stemming + unicode)
- Filters: document_type (post-FTS), include_archived, limit
- No relevance ranking exposed
- No tag search
- No hierarchical scoping

## Goals & Non-Goals

**Goals:**
- Enable tag-based search and filtering
- Support boolean query syntax (AND, OR, NOT, grouping)
- Return results ranked by relevance (BM25)
- Allow scoped/hierarchical search (e.g., "tasks under initiative X")
- Semantic search via vector embeddings (sqlite-vec)
- Knowledge graph / community search via graph relationships (graphqlite)

**Non-Goals:**
- Field-specific query syntax (e.g., `title:planning`)—fields are exposed through existing filters
- Fuzzy/typo-tolerant search
- Search result highlighting/snippets
- Saved searches or search history

## Requirements

### Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| REQ-001 | Search by tag name with `tag:` prefix or `#tagname` syntax | High |
| REQ-002 | Combine multiple tags with AND/OR logic | High |
| REQ-003 | Support boolean operators: AND, OR, NOT (and aliases `+`, `-`) | High |
| REQ-004 | Support grouping with parentheses | Medium |
| REQ-005 | Return results sorted by BM25 relevance score | High |
| REQ-006 | Optionally expose relevance score in results | Low |
| REQ-007 | Scope search to descendants of a document (`under:METIS-I-0001`) | High |
| REQ-008 | Scope search to direct children only (`children:METIS-I-0001`) | Medium |
| REQ-009 | Maintain backward compatibility with current simple queries | High |
| REQ-010 | Semantic search: find documents similar in meaning (`similar:"reduce technical debt"`) | High |
| REQ-011 | Semantic search: find documents similar to another (`like:METIS-T-0012`) | High |
| REQ-012 | Graph search: find related documents via knowledge graph traversal | Medium |
| REQ-013 | Graph search: community detection to cluster related work | Medium |
| REQ-014 | Hybrid search: combine FTS + semantic scores for best results | Medium |

### Non-Functional Requirements

| ID | Requirement |
|----|-------------|
| NFR-001 | Search response time < 100ms for typical workspaces (< 1000 docs) |
| NFR-002 | Query parsing errors return helpful messages, not crashes |
| NFR-003 | Invalid syntax falls back to literal search where reasonable |

## Use Cases

### UC-1: Tag-Based Discovery
- **Actor**: Developer using MCP tool
- **Query**: `tag:backend AND tag:refactor`
- **Expected**: All documents tagged with both #backend and #refactor

### UC-2: Scoped Task Search
- **Actor**: Project manager reviewing initiative
- **Query**: `blocked under:METIS-I-0018`
- **Expected**: All tasks containing "blocked" that are descendants of the initiative

### UC-3: Exclusion Search
- **Actor**: Developer looking for non-completed work
- **Query**: `authentication NOT tag:completed`
- **Expected**: Documents mentioning authentication that aren't tagged as completed

### UC-4: Relevance-Ranked Results
- **Actor**: Any user searching broad terms
- **Query**: `database migration`
- **Expected**: Results ordered by relevance—documents with both terms ranked higher

### UC-5: Semantic Similarity Search
- **Actor**: Developer exploring related work
- **Query**: `similar:"improve API response times"`
- **Expected**: Documents conceptually related to performance optimization, even if they don't contain those exact words

### UC-6: Find Similar Documents
- **Actor**: Project manager looking for related tasks
- **Query**: `like:METIS-T-0012`
- **Expected**: Tasks and initiatives semantically similar to the referenced document

### UC-7: Knowledge Graph Exploration
- **Actor**: Architect understanding system relationships
- **Query**: `related:METIS-I-0018 depth:2`
- **Expected**: Documents connected via references, shared tags, or semantic similarity within 2 hops

### UC-8: Community Discovery
- **Actor**: Team lead organizing work
- **Query**: `communities`
- **Expected**: Clusters of related documents detected via Louvain algorithm, helping identify work themes

## Detailed Design

### Design Decisions

| # | Decision | Choice |
|---|----------|--------|
| Q1 | Parser impl | Parser combinator (nom or winnow) |
| Q2 | Tag query | Combined SQL with JOINs |
| Q3 | Scope filter | Pre-filter descendant IDs |
| Q4 | Embedding model | nomic-embed-text (768 dims), fully embedded |
| Q5 | Embed timing | Background on sync, only changed files |
| Q6 | sqlite-vec load | Bundled into binary |
| Q7 | Default K | Match `limit` param, default 15 |
| Q8 | Distance threshold | Configurable, default 0.5 |
| Q9 | Graph storage | Separate graph.db file |
| Q10 | Extract refs | Yes, regex for METIS-X-NNNN |
| Q11 | Tag modeling | Skip - tags already in table; focus on parent/reference edges |
| Q12 | SIMILAR_TO edges | Yes, cosine > 0.8, on chunks |
| Q13 | PageRank in list | No, separate command (expensive) |
| Q14 | Normalization | Min-max (0-1 range) |
| Q15 | User weights | Query syntax: `weights:0.5,0.3,0.2` |
| Q16 | Debug scores | Yes, optional explain flag |
| Q17 | Chunking strategy | Hybrid: sections + 512 token limit, no merging |
| Q18 | Chunk storage | Embeddings only, regenerate chunks on-demand |
| Q19 | Result granularity | Whole doc + matching chunk snippet |
| Q20 | Min chunk size | No minimum, keep all sections |
| Q21 | Max chunk size | 512 tokens |
| Q22 | Overlap | 10% with 50 token minimum |

---

### Component 0: Chunking Strategy

Documents are chunked for embedding generation and search snippet display.

**Algorithm**:
1. Parse markdown, identify `##` section headers
2. Each section becomes a chunk (no merging of small sections)
3. If section > 512 tokens, split at paragraph boundaries
4. Split chunks get 10% overlap (minimum 50 tokens)
5. Preserve section header metadata with each chunk

**Identity model**: Section is the atomic unit of identity.
- Stable ID: `{doc_short_code}:{section_slug}`
- Example: `METIS-I-0018:detailed-design`
- Sub-chunks within a section share the same section_slug, differ by chunk_index

**Storage**: Chunks are NOT persisted - regenerated on-demand from markdown.
Embeddings ARE persisted, keyed by section identity.

**Schema**:
```sql
CREATE TABLE section_embeddings (
    doc_id TEXT NOT NULL,
    section_slug TEXT NOT NULL,      -- "detailed-design" or "_preamble" for pre-header content
    chunk_index INTEGER NOT NULL,    -- 0, 1, 2... for split sections
    content_hash TEXT NOT NULL,      -- hash of chunk content for change detection
    embedding FLOAT[768],
    PRIMARY KEY (doc_id, section_slug, chunk_index)
);

CREATE INDEX idx_section_embeddings_doc ON section_embeddings(doc_id);
```

**Chunk struct** (in-memory during processing):
```rust
struct Chunk {
    doc_id: String,
    section_slug: String,
    chunk_index: usize,
    content: String,
    section_header: Option<String>,  // original header text for display
    token_count: usize,
    content_hash: String,
}
```

---

### Section Lifecycle Management

**On document sync** (full reconciliation):
```
1. Parse markdown → extract current sections
2. Query stored section_slugs for this doc
3. DELETE orphaned sections:
   DELETE FROM section_embeddings 
   WHERE doc_id = ? AND section_slug NOT IN (current_slugs)
4. For each current section:
   - Chunk the section (split if > 512 tokens)
   - For each chunk, compute content_hash
   - If hash matches stored → skip (embedding still valid)
   - If hash differs or missing → DELETE old chunks for section, INSERT new with fresh embeddings
```

**On document delete**:
```sql
DELETE FROM section_embeddings WHERE doc_id = ?;
```

**Change detection flow**:
```
Document sync
    │
    ▼
Parse sections: [_preamble, goals, design, plan]
    │
    ▼
Query stored: [_preamble, goals, design, alternatives]
    │
    ▼
"alternatives" removed → DELETE
    │
    ▼
For each current section:
    ├─ hash matches? → skip (reuse embedding)
    └─ hash differs? → DELETE section chunks, regenerate, embed
```

**Section rename handling**: Rename = delete old + create new (no rename tracking needed).

---

**Search results** return whole document with matching chunk as snippet:
```
METIS-I-0018: Enhanced Search Capabilities (initiative)
  Match in "## Goals & Non-Goals":
  "...Enable tag-based search and filtering, support boolean 
   query syntax (AND, OR, NOT)..."
  Score: 0.87
```

---

### Component 1: Query Parser

Create a new `SearchQuery` parser that transforms user input into a structured query:

```
Input: "database tag:backend NOT deprecated under:METIS-I-0001"

Output: SearchQuery {
    terms: ["database"],
    tags: [TagFilter::Include("backend")],
    negations: ["deprecated"],
    scope: Some(Scope::Under("METIS-I-0001")),
    operators: [And, And, And]
}
```

**Parser approach**: Use `winnow` parser combinator library for clean, maintainable grammar definition.

### Component 2: Tag Search

Combined SQL approach with JOINs for single round-trip:

```sql
SELECT d.*, bm25(document_search) as rank 
FROM documents d
INNER JOIN document_search ds ON d.filepath = ds.document_filepath
INNER JOIN document_tags dt ON d.id = dt.document_id
WHERE document_search MATCH ?
  AND dt.tag_name IN (?, ?)
GROUP BY d.id
HAVING COUNT(DISTINCT dt.tag_name) = ?  -- for AND logic
ORDER BY rank
```

### Component 3: Hierarchical Scope

Pre-filter approach - get descendant IDs first, then constrain FTS:

```sql
-- Step 1: Get descendants
WITH RECURSIVE descendants AS (
    SELECT child_id FROM document_relationships WHERE parent_id = ?
    UNION ALL
    SELECT r.child_id FROM document_relationships r
    INNER JOIN descendants d ON r.parent_id = d.child_id
)
SELECT child_id FROM descendants;

-- Step 2: FTS with constraint
SELECT d.*, bm25(document_search) as rank 
FROM documents d
INNER JOIN document_search ds ON d.filepath = ds.document_filepath
WHERE document_search MATCH ?
  AND d.id IN (?)  -- descendant IDs
ORDER BY rank
```

### Component 4: Relevance Ranking

FTS5 provides `bm25()` function:
```sql
SELECT *, bm25(document_search) as rank 
FROM document_search 
WHERE document_search MATCH ? 
ORDER BY rank
```

### Component 5: Vector Search (sqlite-vec)

**Dependencies**: 
- `sqlite-vec` extension - bundled/compiled into binary
- `nomic-embed-text` model via ONNX Runtime (768 dimensions, fully embedded)

**Schema addition**:
```sql
CREATE VIRTUAL TABLE document_vectors USING vec0(
    document_id TEXT PRIMARY KEY,
    embedding FLOAT[768]
);
```

**Embedding strategy**:
- Generate embeddings in background during sync
- Only process files with content changes (hash comparison)
- Store embeddings per document (consider chunking for large docs)
- No external API calls - fully local inference

**Query flow**:
1. `similar:"query text"` → embed query locally → KNN search
2. `like:METIS-T-0012` → lookup doc embedding → KNN search

**Default behavior**:
- K = `limit` parameter if provided, else 15
- Distance threshold = configurable, default 0.5 cosine similarity

### Component 6: Knowledge Graph (graphqlite)

**Dependencies**:
- `graphqlite` extension (colliery-io/graphqlite)

**Storage**: Separate `graph.db` file for isolation and independent rebuild

**Graph model**:
- **Nodes**: Documents (with type, phase, short_code as properties)
- **Edges**: 
  - `PARENT_OF` (from document_relationships table)
  - `REFERENCES` (extracted via regex for METIS-X-NNNN patterns in content)
  - `SIMILAR_TO` (cosine > 0.8 from vector similarity, computed on chunks)

**Note**: Tag-based edges skipped - tags already queryable via SQL table; focus on structural and semantic relationships.

**Graph construction**:
- Build graph from existing `document_relationships` table
- Extract references from document content
- Add semantic edges from vector similarity (threshold-based, on chunks)
- Sync incrementally on document changes

**Query capabilities**:
```cypher
-- Find related documents within 2 hops
MATCH (d:Document {short_code: 'METIS-I-0018'})-[*1..2]-(related)
RETURN related

-- Community detection
CALL louvain() YIELD nodeId, communityId
RETURN nodeId, communityId

-- PageRank (separate command, expensive)
CALL pagerank() YIELD nodeId, score
RETURN nodeId, score ORDER BY score DESC
```

### Architecture Changes

```
┌─────────────────┐
│  SearchQuery    │  ← winnow parser
│    Parser       │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  SearchEngine   │  ← Orchestrates search components
└────────┬────────┘
         │
    ┌────┴────┬──────────┬──────────┬──────────┐
    ▼         ▼          ▼          ▼          ▼
┌───────┐ ┌───────┐ ┌─────────┐ ┌────────┐ ┌─────────┐
│ FTS5  │ │ Tags  │ │Hierarchy│ │Vectors │ │  Graph  │
│+bm25  │ │ JOIN  │ │Pre-filt │ │sqlite- │ │graphq-  │
│       │ │       │ │   CTE   │ │vec+ONNX│ │  lite   │
└───────┘ └───────┘ └─────────┘ └────────┘ └─────────┘
                                    │
                              ┌─────┴─────┐
                              │  nomic-   │
                              │embed-text │
                              │  (local)  │
                              └───────────┘
```

### Hybrid Search Strategy

Combine multiple signals for best results:

```
final_score = α * normalize(bm25) + β * normalize(vector_sim) + γ * normalize(graph_proximity)
```

**Normalization**: Min-max scaling to 0-1 range for each component.

**User-configurable weights** via query syntax:
```
database migration weights:0.5,0.3,0.2
```

**Default weights** (auto-detected by query type):
- Keyword query: α=0.7, β=0.2, γ=0.1
- Semantic query (`similar:`, `like:`): α=0.2, β=0.7, γ=0.1
- Graph query (`related:`): α=0.1, β=0.3, γ=0.6

**Debug/explain mode**: Optional flag to show component scores in results.

## Alternatives Considered

### Alt 1: Full Query Language (Lucene-style)
- **Rejected**: Over-engineered for current needs; field queries already covered by filters

### Alt 2: External Search Engine (Meilisearch, Tantivy)
- **Rejected**: Adds deployment complexity; SQLite FTS5 sufficient for workspace scale

### Alt 3: Add all metadata to FTS index
- **Rejected**: Loses structure; harder to do exact tag matching vs. full-text

### Alt 4: External Vector Database (Pinecone, Qdrant, Chroma)
- **Rejected**: Breaks SQLite-only philosophy; sqlite-vec provides native integration

### Alt 5: Neo4j for Graph
- **Rejected**: Requires separate server; graphqlite provides Cypher in SQLite

### Alt 6: API-only Embeddings (OpenAI, Cohere)
- **Partially accepted**: Support as optional backend, but prefer local-first with ONNX models

## Implementation Plan

### Phase 1: Relevance Ranking
- Add `bm25()` scoring to existing search
- Sort results by relevance
- Minimal breaking changes

### Phase 2: Query Parser
- Implement `SearchQuery` struct and parser
- Support AND/OR/NOT and parentheses
- Backward compatible (plain text = current behavior)

### Phase 3: Tag Search
- Add `tag:` prefix support to parser
- Query `document_tags` table
- Intersect with FTS results

### Phase 4: Hierarchical Scoping
- Add `under:` and `children:` support
- Implement recursive CTE for descendants
- Pre-filter or post-filter results

### Phase 5: Vector Search Infrastructure
- Integrate sqlite-vec extension
- Add document_vectors table migration
- Implement EmbeddingService abstraction
- Support pluggable embedding backends (local ONNX, API)

### Phase 6: Semantic Search
- Add `similar:` and `like:` query syntax
- Generate embeddings on document sync
- Implement KNN search via sqlite-vec
- Background embedding generation for existing docs

### Phase 7: Knowledge Graph Infrastructure
- Integrate graphqlite extension
- Build graph from document relationships
- Add tag-based and reference-based edges
- Sync graph on document changes

### Phase 8: Graph Search & Analysis
- Add `related:` query syntax with depth control
- Implement community detection (`communities` query)
- Expose PageRank for document importance
- Add `SIMILAR_TO` edges from vector similarity

### Phase 9: Hybrid Search
- Combine FTS, vector, and graph scores
- Implement configurable weight tuning
- Auto-detect query intent for weight selection
# Exploration Features for RocketIndex

> Design notes from comparing RocketIndex with grepai (January 2025)

## Problem Statement

RocketIndex excels at deterministic navigation: "Where is X defined? Who calls Y?" But it lacks support for exploration scenarios: "I don't know what I'm looking for yet."

grepai solves this with vector embeddings and semantic search. However, this approach has a fundamental flaw: it duplicates semantic understanding that the host AI agent already provides (and does better).

## Design Principle

**Provide richer structural facts. Let the AI do the semantic reasoning.**

The AI's understanding *is* the semantic layer. RocketIndex should give it better structural primitives to reason over — no embeddings, no Ollama dependency.

## Proposed Features

### 1. Multi-Pattern Symbol Search (`explore_domain`)

Search for multiple patterns at once, returning the union of results.

```bash
rkt explore "auth,login,session,token,credential"
```

**Use case:** AI picks patterns based on user's vague query ("authentication stuff"), RocketIndex returns all matching symbols.

**Implementation:** Extend `search_symbols` to accept comma-separated patterns. Low effort.

### 2. Entry Point Detection (`find_entry_points`)

Find call graph roots — symbols that are called but don't call anything else (or call only external/stdlib).

```bash
rkt entry-points
# → main, handleLogin, processWebhook, runMigration
```

**Use case:** Understanding codebase structure, finding where execution starts.

**Implementation:** Query symbols with high in-degree, low out-degree in call graph. Medium effort.

### 3. Public API Surface (`find_public_api`)

Return only public/exported symbols — the external "shape" of the codebase.

```bash
rkt public-api
# → UserService (class), createUser (fn), validateEmail (fn)
```

**Use case:** Understanding what a library/module exposes without reading internals.

**Implementation:** Filter existing symbol index by visibility. Low effort (visibility already tracked).

### 4. Module Dependency Graph (`module_graph`)

Show which modules/namespaces depend on which others.

```bash
rkt modules
# handlers → services → repositories → models
```

**Use case:** Understanding architectural layers, identifying coupling.

**Implementation:** Aggregate symbol-level dependencies to module level. Medium effort.

### 5. Symbol Neighborhood (`symbol_neighborhood`)

Given a symbol, return its immediate context: what it calls, what calls it, siblings in the same file/module.

```bash
rkt neighborhood "UserService.save"
# Calls: Repository.insert, Validator.check
# Called by: Handler.createUser, Handler.updateUser
# Siblings: UserService.find, UserService.delete
```

**Use case:** Quick context around a symbol without full dependency traversal.

**Implementation:** Combine existing `find_callers`, `analyze_dependencies`, and file-level symbol listing. Low effort.

## Example Workflow

User: "How does authentication work in this codebase?"

AI reasoning:
1. `search_symbols("*auth*")` + `search_symbols("*login*")` + `search_symbols("*session*")`
2. Gets: `AuthService`, `LoginHandler`, `SessionManager`, `validateToken`
3. `find_callers("AuthService.authenticate")` — sees it's called from `LoginHandler`
4. `analyze_dependencies("LoginHandler", reverse=false)` — traces the flow
5. Reads the relevant files, synthesizes answer

**RocketIndex provides the map. The AI navigates it.**

## Implementation Priority

| Feature | Effort | Value | Priority |
|---------|--------|-------|----------|
| Multi-pattern search | Low | High | P1 |
| Symbol neighborhood | Low | High | P1 |
| Public API filter | Low | Medium | P2 |
| Entry point detection | Medium | High | P2 |
| Module dependency graph | Medium | Medium | P3 |

## Non-Goals

- **Embedding-based semantic search** — Duplicates host AI capabilities
- **Natural language query parsing** — Let the AI translate queries to symbol patterns
- **Relevance ranking** — Return facts; AI judges relevance

## References

- grepai: https://github.com/yoanbernabeu/grepai
- Comparison analysis: grepai uses local embeddings (Ollama/OpenAI) for semantic search, but this is redundant when the host agent has superior language understanding

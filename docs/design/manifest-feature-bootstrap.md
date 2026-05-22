# Bootstrapping Manifest Feature Trees from Code

> Design notes for using RocketIndex + git history to generate feature trees (January 2025)

## Problem Statement

When adopting Manifest on a large existing codebase, manually creating the feature tree is tedious and error-prone. The codebase already contains signals about what features exist — we should extract them.

## Key Insight

Two complementary signal sources:

| Source | What It Reveals |
|--------|-----------------|
| **Code structure** (RocketIndex) | What exists now — modules, classes, entry points |
| **Git history** | Why it was built — human-written feature names, business intent |

Code structure alone lacks business context. Git history alone lacks current state. Together, they can infer a meaningful feature tree.

## Architectural Fit

| Component | Role |
|-----------|------|
| **Manifest** | Owns feature management, exposes `analyze_codebase` tool |
| **RocketIndex** | Data source for code structure |
| **Git** | Data source for human intent |
| **Host AI** | Semantic reasoning, proposes features from data |

This is a **Manifest MCP tool** that happens to use RocketIndex, not a RocketIndex feature.

## Signal Sources

### Code Structure (via RocketIndex)

| RocketIndex Data | Feature Signal |
|------------------|----------------|
| Modules/namespaces | Top-level feature groups |
| Public API surface | Exposed capabilities |
| Entry points (handlers, CLI commands) | User-facing features |
| Class/type names | Domain concepts |
| Call graph clusters | Related functionality |

### Git History

| Git Signal | Feature Signal |
|------------|----------------|
| Commit messages | Human-language feature names ("Add user authentication") |
| PR titles/descriptions | Business context, ticket references |
| File co-change patterns | Files that change together → same feature |
| Conventional commits | `feat:`, `fix:` prefixes explicitly mark feature work |
| Tags/releases | User-facing summaries of what shipped |
| First commits adding files | Feature inception points |

### Additional Sources

| Source | Signal |
|--------|--------|
| CHANGELOG.md | Curated feature summaries |
| GitHub issues/PRs (via `gh`) | Linked tickets with business context |
| README / docs | High-level feature descriptions |
| Semantic version tags | Major features often coincide with version bumps |

## Proposed Tool

### `manifest.analyze_codebase`

```
analyze_codebase(project_id, directory_path, include_git=true, since="6 months ago")
```

**Process:**
1. Call RocketIndex `describe_project` → module/symbol overview
2. Call RocketIndex `find_entry_points` → user-facing handlers
3. Call RocketIndex `module_graph` → architectural layers
4. Analyze git history → extract feature-related commits per module
5. Return structured analysis for AI reasoning

**Output:**
```json
{
  "modules": [
    {"name": "auth", "path": "src/auth/", "symbols": 24, "public_symbols": 8},
    {"name": "payments", "path": "src/payments/", "symbols": 31, "public_symbols": 12}
  ],
  "entry_points": [
    {"name": "LoginHandler", "module": "auth"},
    {"name": "ProcessPaymentHandler", "module": "payments"}
  ],
  "git_analysis": {
    "auth": {
      "feature_commits": ["Add OAuth support", "Implement 2FA", "Add session management"],
      "first_commit": "2023-04-12",
      "recent_activity": true
    },
    "payments": {
      "feature_commits": ["Add Stripe integration", "Implement refunds", "Add invoice generation"],
      "first_commit": "2023-06-01",
      "recent_activity": true
    }
  },
  "infrastructure_candidates": ["utils", "config", "logging"],
  "inferred_features": [
    {
      "name": "Authentication",
      "confidence": "high",
      "evidence": ["module: auth", "commits: OAuth, 2FA, sessions"],
      "suggested_children": ["OAuth", "Two-Factor Auth", "Session Management"]
    },
    {
      "name": "Payments",
      "confidence": "high",
      "evidence": ["module: payments", "commits: Stripe, refunds, invoices"],
      "suggested_children": ["Stripe Integration", "Refunds", "Invoicing"]
    }
  ]
}
```

## Workflow

```
1. User: "Bootstrap features for this codebase"
2. AI calls manifest.analyze_codebase(project_id, directory)
3. Tool returns structured analysis
4. AI reasons about the data, proposes feature tree
5. AI calls manifest.plan(features=[...], confirm=false) → shows proposal
6. User reviews, adjusts
7. AI calls manifest.plan(features=[...], confirm=true) → creates tree
```

### Alternative: Skill-Based

A `/bootstrap` skill in Manifest that guides this workflow interactively:
- Detects RocketIndex availability
- Gathers structural + git data
- Proposes features module by module
- Handles user feedback loop

## Git History Filtering

Git history is noisy. Filter for signal:

**Include:**
- Commits with `feat:` prefix (conventional commits)
- Commits with ticket references (`#123`, `JIRA-456`)
- First commits that add new files (feature inception)
- Commits touching specific modules
- Merge commits with PR context

**Exclude:**
- "Fix typo", "WIP", "Merge branch main"
- Pure refactoring commits
- Dependency updates
- CI/build changes

**Configurable time window:**
- Full history → complete feature archaeology
- Recent (6 months) → current active features
- Since tag → features since last release

## Challenges

### 1. Granularity
Is `UserService` one feature, or are `create`, `update`, `delete` separate features?

**Heuristic:** Start coarse (module = feature), let user break down if needed.

### 2. Infrastructure vs Product
How to distinguish `utils`, `helpers`, `config` from product features?

**Heuristics:**
- Naming patterns (`*Utils`, `*Helper`, `*Config`)
- No entry points
- Called by many modules but doesn't call them (utility pattern)
- Let AI flag as "infrastructure candidates" for human review

### 3. Feature Naming
Code names (`UserController.createUser`) → Human names ("User Registration")

**Approach:** Extract names from git commits, fall back to code names, let human refine.

### 4. State Inference
Code exists → feature is implemented. But what about partial implementations?

**Heuristic:** If code exists and has tests, mark as `implemented`. Otherwise `in_progress`.

## RocketIndex Dependencies

This workflow benefits from exploration features (see `exploration-features.md`):

| Feature | Status | Value for Bootstrap |
|---------|--------|---------------------|
| `describe_project` | Exists | Module/symbol overview |
| `find_entry_points` | Proposed | User-facing handlers |
| `module_graph` | Proposed | Architectural layers |
| `find_public_api` | Proposed | Exposed capabilities |

## Open Questions

1. **Interactive vs batch?** — Propose all at once, or module by module?
2. **Confidence thresholds?** — Auto-create high-confidence features, prompt for low?
3. **Incremental updates?** — Re-analyze after codebase changes?
4. **Multi-repo?** — How to handle monorepos with multiple products?

## Non-Goals

- Fully automated feature creation (human review is essential)
- Perfect inference (conservative proposals, easy override)
- Replacing human judgment about business intent

## Success Criteria

- Reduces manual feature tree creation from hours to minutes
- Catches features human might miss
- Produces reasonable names that need minimal editing
- Works on codebases with 100+ modules

# RFC 003: Agent Harness Integration Strategy

- **Date**: 2026-05-20
- **Status**: Proposed
- **Area**: RocketIndex Integrations / Agent Booster Pack

## Summary

RocketIndex should expose three integration surfaces with clear ownership:

1. **MCP is the structured query plane.** Agents use MCP tools for symbol
   lookup, callers, references, dependency traversal, and project summaries.
2. **The CLI is the control plane.** Users and agents use `rkt` for indexing,
   health checks, watch mode, setup, and fallback execution.
3. **Skills are the routing layer.** Agent Booster Pack and other instruction
   packs teach agents when to prefer MCP, when to use the CLI, and when to
   avoid RocketIndex.

For Claude, Codex, Cursor, Gemini, Zed, and other MCP-capable harnesses, MCP
should be the default surface for navigation queries. CLI-plus-skill remains a
portable fallback and the required setup path.

## Motivation

RocketIndex already has two useful surfaces:

- `rkt` CLI commands for humans and shell-capable agents.
- An MCP server exposing structured tools for AI assistants.

Agent harnesses differ in what they can load, approve, and persist. Some can
run MCP servers directly. Some only read `AGENTS.md`, `CLAUDE.md`, or skill
files. Some can call shell commands but treat MCP configuration as a user-owned
setup step. Without a clear integration policy, RocketIndex risks duplicating
logic across plugins, skills, shell wrappers, and per-harness extensions.

The goal is to keep the product simple: one native indexer, one structured
server, one CLI, and small harness-specific adapters that do not reimplement
RocketIndex behavior.

## Decision

RocketIndex will treat MCP as the preferred integration for agent navigation
when the harness supports MCP. The CLI remains the source of truth for setup,
index lifecycle, diagnostics, and fallback access. Skills and instruction
files do not wrap RocketIndex behavior; they route the agent to the right
surface.

The guiding contract is:

> MCP answers code-navigation questions. The CLI manages RocketIndex. Skills
> teach tool choice.

## Integration Model

### Query Plane: MCP

Use MCP for fast, typed, structured navigation:

- `find_definition`
- `find_callers`
- `find_references`
- `analyze_dependencies`
- `search_symbols`
- `describe_project`

MCP is preferred because it gives agents named tools with typed parameters and
structured results. This reduces shell permission prompts, avoids ad hoc output
parsing, and keeps navigation intent explicit.

### Control Plane: CLI

Use `rkt` for operational tasks:

- `rkt index`
- `rkt watch`
- `rkt doctor`
- `rkt setup <agent>`
- `rkt serve add/remove/list`
- `rkt update`
- Shell fallback for any MCP-unavailable environment

The CLI should remain stable, scriptable, and JSON-capable. It is the right
place for long-running watch processes, index rebuilds, local diagnostics, and
installer flows.

### Routing Layer: Skills and Instructions

Use ABP skills, `AGENTS.md`, `CLAUDE.md`, or equivalent harness instruction
files to encode policy:

- Prefer RocketIndex MCP tools for definitions, callers, references,
  dependency traversal, and project structure.
- Use `rkt` CLI for indexing, health checks, setup, watch mode, and fallback.
- Use grep or file reads for literal text, comments, logs, strings, or files
  outside the indexed project.
- Run `rkt doctor` or the MCP status equivalent before trusting results when
  the index may be stale.

Skills should be short. They should not duplicate full command reference text
that belongs in RocketIndex docs or `rkt --help`.

## Harness Policy

| Harness | Recommended Default | Setup / Fallback | Notes |
|---------|---------------------|------------------|-------|
| Claude Code | MCP | Claude plugin or `rkt setup claude` | The existing plugin can auto-download `rkt` and configure MCP. Skills should tell agents to prefer MCP tools. |
| Codex | MCP when configured | `rkt` CLI plus ABP skill fallback | Codex should use MCP for navigation and CLI for indexing. ABP can provide routing guidance without owning the indexer. |
| Cursor | MCP when configured | `rkt setup cursor` plus rules | Rules should prefer structured lookups before broad file search. |
| Gemini CLI | MCP when configured | `gemini mcp add rocket-index rkt serve` or `rkt setup gemini` | CLI fallback remains important because Gemini users may vary in MCP setup. |
| Zed | MCP/context server | `rkt setup zed` | Zed integration should remain a context-server configuration, not a separate query implementation. |
| Pi / Agent Booster Pack | Prefer MCP if Pi can consume MCP directly. Otherwise wrap CLI as Pi tools. | ABP skill plus optional Pi extension | A Pi extension should be a thin adapter only. It should call `rkt` or an MCP client, not reimplement RocketIndex. |
| Shell-only agents | CLI | Skill/instruction file | Use `--format json` where available and keep commands narrow. |

## Agent Booster Pack Extension Shape

ABP should treat RocketIndex as an optional external capability, not as core
ABP doctrine. The extension should be small and compositional.

Recommended ABP artifacts:

1. **A RocketIndex skill or reference note** that states the routing policy:
   MCP first for navigation, CLI for setup and fallback.
2. **Optional Pi runtime extension** only if Pi cannot use RocketIndex MCP
   directly. The extension should expose thin tools such as:
   - `rocketindex_doctor`
   - `rocketindex_definition`
   - `rocketindex_symbols`
   - `rocketindex_callers`
   - `rocketindex_references`
   - `rocketindex_dependencies`
3. **No embedded indexer logic.** The extension must shell out to `rkt` or call
   MCP. It must not parse source files, inspect SQLite internals, or maintain
   its own index state.

The ABP skill should be portable across harnesses. The Pi extension is only a
convenience adapter for Pi's tool surface.

## Detailed Design

### MCP Server

RocketIndex MCP remains the primary structured interface. Tool names and
schemas should stay stable. Additive tools are allowed when they serve common
agent workflows.

Potential additions:

- `status` or `doctor` MCP tool if `describe_project` is not sufficient for
  health checks.
- `impact` alias or dedicated tool if `analyze_dependencies(reverse=true)` is
  too indirect for agents.
- Bounded context-building tool if future comparison work shows that agents
  still need too many file reads after structured lookups.

### CLI

The CLI should preserve JSON output for all commands likely to be used by
agents. It should avoid requiring agents to parse pretty output.

Expected behavior:

- Query commands return machine-readable JSON with stable field names.
- Missing index errors include the remediation command.
- `rkt doctor --format json` remains the canonical health signal.
- Long-running `rkt watch` stays an explicit user/session command.

### Skills

The RocketIndex skill text should be concise:

```md
Use RocketIndex for structural code navigation.

If RocketIndex MCP tools are available, prefer them for definitions, callers,
references, dependency traversal, and project summaries.

Use `rkt` CLI for `index`, `watch`, `doctor`, setup, and fallback when MCP is
not configured.

Use normal search/read tools for literal text, comments, logs, and strings.
```

This keeps harness instructions focused on behavior. Full command docs remain
in RocketIndex.

### Pi Extension

If implemented, the Pi extension should:

- Register tools that map directly to existing RocketIndex operations.
- Run in the current project directory from the Pi context.
- Prefer JSON CLI output.
- Return CLI stderr and exit code when commands fail.
- Avoid starting background watchers.
- Avoid writing project files except when the user asks for setup/indexing.

This is an adapter, not a second integration platform.

## Security and Trust Boundaries

RocketIndex reads source files and writes a local SQLite index. Harness
integrations must not expand that trust boundary silently.

Requirements:

- MCP tools should only query registered or current projects, matching current
  project-boundary behavior.
- CLI wrappers must not accept arbitrary shell fragments. They should build
  argv arrays from typed parameters.
- Skills must not instruct agents to run `rkt setup` or edit harness config
  without user intent.
- `rkt watch` should remain explicit because it is a long-running file-system
  watcher.
- Optional semantic features that invoke external tools, such as `dotnet fsi`,
  must remain opt-in.

## Alternatives Considered

### CLI-plus-skill only

This is portable and simple, but it makes every agent route through shell
commands. That increases permission prompts and output parsing. It also hides
the user's intent inside command strings instead of typed tool calls.

Rejected as the default. Kept as fallback and control plane.

### MCP only

This gives the cleanest agent contract, but it does not solve setup, indexing,
watch mode, or non-MCP harnesses. Users still need a CLI.

Rejected as the only surface. Kept as the default query plane.

### Per-harness native implementations

Each harness could get a bespoke plugin that directly reads the SQLite index
or embeds RocketIndex logic.

Rejected. It multiplies maintenance cost and risks behavior drift. Harness
extensions should call MCP or `rkt`.

### ABP-owned RocketIndex behavior

ABP could ship a richer extension that manages indexing, watches files, and
implements query behavior.

Rejected. ABP should route and adapt. RocketIndex should own indexing and
navigation behavior.

## Rollout Plan

1. **Document the policy** in RocketIndex docs and ABP RFC/design notes.
2. **Update RocketIndex setup guidance** so each harness says whether it uses
   MCP, CLI, or both.
3. **Add an ABP skill/reference** with the MCP-first routing policy.
4. **Add a Pi extension only if needed** after confirming Pi cannot consume
   RocketIndex MCP directly.
5. **Add integration tests** for any adapter. Tests should assert argv
   construction and failure handling, not RocketIndex internals.
6. **Run direct harness trials**:
   - MCP configured
   - CLI fallback only
   - missing index
   - stale or unhealthy index

## Success Criteria

- Agents use MCP tools for navigation when available.
- Agents use `rkt` for setup, indexing, watch mode, and diagnostics.
- ABP contains only routing guidance and optional thin adapters.
- No harness duplicates RocketIndex parsing, database, or graph logic.
- CLI fallback works in shell-only environments.
- Missing or stale index states produce clear remediation.

## Open Questions

- Can Pi consume an MCP server directly, or does it need a thin runtime
  extension that wraps `rkt`?
- Should RocketIndex expose an MCP `doctor` tool, or is `describe_project`
  enough?
- Should `analyze_dependencies(reverse=true)` be renamed or aliased as
  `impact` for agent clarity?
- Should `rkt setup <agent>` install skills/instructions, MCP config, or both
  for each harness?
- Should the Claude Code plugin remain the only auto-download path, or should
  ABP/Pi get a similar binary bootstrapper?

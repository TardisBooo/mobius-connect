# Möbius MCP

stdio MCP for the Möbius Agent Session Hub. Same binary as the CLI: `mobius-connect mcp serve`.

The desktop app in [TardisBooo/Mobius](https://github.com/TardisBooo/Mobius) talks to the same local vault. This document is the published agent-facing server.

## How it fits

```
approved local sources (JSONL / OpenCode SQLite)
        │  read-only
        ▼
   local SQLite index (FTS5/BM25)
        │
        ├─ desktop UI
        ├─ mobius-connect CLI
        └─ mobius-connect mcp serve   ← this process
```

The server never writes harness files, never installs SessionStart hooks, and never attaches a PTY.

## Start

```powershell
mobius-connect init
mobius-connect sources add codex $env:USERPROFILE\.codex\sessions
mobius-connect sources add opencode $env:USERPROFILE\.local\share\opencode
mobius-connect sources refresh
mobius-connect mcp serve
```

`mcp serve` uses stdio only. Point your MCP client at the `mobius-connect` executable with the `mcp serve` arguments. Run `init` before serving; the process refuses to start if the database is missing.

## Privilege split

| Open without a token | Requires a human token |
| --- | --- |
| `list_sessions` | `search_sessions` |
| `get_session` | `mome_recall` |
| `get_index_health` | `read_session_range` |
| `get_lineage` | `commit_handoff` |
| `prepare_handoff` | |
| `get_handoff_status` | |
| `request_session_approval` | |
| `resolve_session` | |
| `build_memory_reference` | |

`request_session_approval` describes how a person grants a token. This server cannot mint one. A model saying the user authorized a read is not a credential.

Minting happens in a real terminal:

```powershell
mobius-connect approvals handoff <handoff-id>
```

The operator types `APPROVE`. The token is short-lived, single-use, and scoped to that operation.

## Tools

| Tool | Reads bodies? | Notes |
| --- | --- | --- |
| `list_sessions` | no | Catalogue metadata |
| `get_session` | no | One native identity |
| `get_index_health` | no | Counts and local health |
| `get_lineage` | no | Inherited graph for explicit IDs |
| `build_memory_reference` | no | Same graph, no injection |
| `resolve_session` | no | Approved source locator |
| `prepare_handoff` | no | References-only envelope; does not launch |
| `get_handoff_status` | no | `started` is not identity-bound |
| `request_session_approval` | no | Explains the human grant |
| `search_sessions` | excerpts | Exact-query token |
| `mome_recall` | bounded citations | Token + 3 sessions / ~1200 tokens |
| `read_session_range` | yes | Byte range token |
| `commit_handoff` | no | Single-use launch token |

## What MCP cannot do

- Mint an approval token
- Add or broaden approved source roots
- Attach or type into a PTY
- Rewrite Codex, Claude Code, OpenCode, Pi, Grok, or OMP files
- Inject history at session start
- Treat a launcher PID as a bound native session

## Client config sketch

Exact client JSON varies. The process is always stdio:

```json
{
  "mcpServers": {
    "mobius": {
      "command": "mobius-connect",
      "args": ["mcp", "serve"]
    }
  }
}
```

Use an absolute path to the binary if it is not on `PATH`. Do not wrap this in a login shell that can hang stdio.

## Skill

`plugins/mobius-connect/skills/mobius-memory/SKILL.md` tells an agent how to use these tools. It has no hooks. Loading the skill does not grant transcript access.

## Related

- CLI README: [../README.md](../README.md)
- Desktop: https://github.com/TardisBooo/Mobius
- Approval-token design: https://github.com/TardisBooo/Mobius/blob/main/docs/blog/03-approval-tokens.md

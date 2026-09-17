# mobius-connect

CLI and stdio MCP for the Möbius **Agent Session Hub**.

Search, cite, graph and hand off local coding-agent sessions from a terminal or over MCP. Harness files stay read-only. A handoff carries ancestry references, not a generated summary.

Desktop workspace: [TardisBooo/Mobius](https://github.com/TardisBooo/Mobius).

**Yours, on this machine.** The index is local SQLite. Original Codex JSONL, Claude transcripts, and OpenCode `opencode.db` are never rewritten. Möbius does not sell model accounts.

[Product site](http://8.137.87.76/mobius/) · [MCP](docs/MCP.md) · [Desktop README](https://github.com/TardisBooo/Mobius/blob/main/README.md) · [MIT License](LICENSE)

> Development preview. Native launch approvals and cross-harness identity binding remain release gates. Compatibility is verified per harness.

## Three names, one core

People meet three spellings. They are not three products.

| Name | What it is | Where it lives |
| --- | --- | --- |
| **Möbius** | The product: an Agent Session Hub. Desktop app on Windows. | [TardisBooo/Mobius](https://github.com/TardisBooo/Mobius) |
| **mobius-connect** | The published CLI + stdio MCP binary. Same core, no desktop UI. | this repository |
| **mobius / mydesk** | Desktop-side helper binaries that operate on the vault the desktop app owns (`mobius mome …`). Not the published CLI. | built inside the desktop repo |

```
Claude Code / Codex / OpenCode / Pi / Grok / OMP transcripts
        │   read-only adapters (JSONL or opencode.db)
        ▼
   mydesk-core → local SQLite index (FTS5/BM25, optional localhost embeddings)
        │
        ├── Möbius desktop        person at a Windows desk
        ├── mobius-connect CLI    person at a terminal
        └── mobius-connect MCP    agent over stdio
```

**CLI vs MCP, in one line:** the CLI is for a human at a terminal (it can ask *you* to type `APPROVE`); MCP is for an agent process (it can list metadata, but every read of transcript bodies needs a token the CLI/desktop minted after a human typed `APPROVE`). MCP cannot mint approval tokens, add sources, or attach a PTY.

## Install

```powershell
git clone https://github.com/TardisBooo/mobius-connect.git
git clone https://github.com/TardisBooo/Mobius.git desktop
cd mobius-connect
cargo build --release
```

This checkout depends on the sibling crate `../desktop/crates/mydesk-core`, so clone the desktop repo next to it (or fix the path in `Cargo.toml`). Durable data defaults to `D:\DataVault\Mobius`. Use `--data-root <dir>` (global flag) for an isolated test vault.

## Tutorial: every command and what it does

Run `mobius-connect --help` for the authoritative list. This is the guided tour.

### 1. First run: `init`

```powershell
mobius-connect init
```

Creates the SQLite index and vault layout, then prints a health report. Safe to re-run; it does not touch any harness directory. `init` never discovers anything by itself — sources are always explicit (next step).

### 2. Point it at your history: `sources add` / `sources list` / `sources refresh`

```powershell
mobius-connect sources add codex    $env:USERPROFILE\.codex\sessions
mobius-connect sources add claude   $env:USERPROFILE\.claude\projects
mobius-connect sources add opencode $env:USERPROFILE\.local\share\opencode
mobius-connect sources list
mobius-connect sources refresh
```

- `sources add <harness> <path>` registers **one** directory you chose. This is the only way a source gets approved; MCP has no tool for it.
- `sources list` prints the approved roots and their mode.
- `sources refresh` re-scans approved roots and updates the read-only index. It reports per-root counts (`indexed`, `unchanged`, `skipped`, `errors`). OpenCode roots are read from `opencode.db` with locator `{db}#opencode:{id}`; JSONL roots are parsed per session file.

### 3. Check health: `triage` / `doctor` / `setup`

```powershell
mobius-connect triage
mobius-connect doctor
mobius-connect setup
```

- `triage` prints health, approved-source count, semantic status, and **one recommended next command**. Start here when unsure.
- `doctor` is the deeper health dump (same as `init` output).
- `setup` detects installed harnesses and prints the exact MCP-client and skill install steps. It never writes hooks or harness config.

### 4. Find sessions: `sessions list` / `search` / `show` / `alias`

```powershell
mobius-connect sessions list --limit 20
mobius-connect sessions search "flaky checkout tests"
mobius-connect sessions show <session-id>
mobius-connect sessions alias <session-id> "checkout-flake-hunt"
```

- `search` runs lexical BM25 over the index and returns JSON hits plus `suggested_next_commands`. The response also carries `retrieval_mode` (`lexical_bm25`, or `hybrid` once embeddings are enabled **and** a local model answers) and `semantic_status` — if Ollama is missing it says so instead of pretending.
- `show` returns one session's identity, capabilities, and lineage node without loading the transcript.
- `alias` gives a session a human name; aliases survive reindexing and are what lineage titles display.

### 5. Read exact bytes: `sessions read`

```powershell
mobius-connect sessions read <session-id> --offset 0 --bytes 4096
```

Explicit byte-range read of the **original source file**, bounded (max 16384 bytes per call), returning `data` and `next_offset` for paging. This is deliberate friction: recall gives you citations, `read` makes the actual tape fetch an explicit, bounded act.

### 6. See the ancestry: `graph show` / `graph export`

```powershell
mobius-connect graph show <session-id>
mobius-connect graph export --format mermaid <session-id>
mobius-connect graph export --format html   <session-id> <another-id>
```

`graph show` prints the lineage manifest (nodes, edges, `content_mode: references_only`, `missing_sources`). `graph export` renders it as JSON (default), Mermaid, or a standalone HTML view. Mutually-hand-off chains (Codex→Claude→Codex) come out as a forward relay chain; every handoff opens a new session, so the graph is a DAG by construction and cycles are rejected.

### 7. Bounded recall: `mome recall`

```powershell
mobius-connect mome recall "why do checkout tests flake on CI" --provider opencode --provider codex
```

Explicit, bounded recall over the local index: at most **three** distinct sessions, about **1,200 tokens**, every hit carrying a copyable `@session:provider/id#mN-mM` citation and a content hash. It never injects into a prompt — you get JSON you choose to paste. Restrict scopes with repeated `--provider`; cap output with `--max-tokens`.

### 8. Opt-in hybrid ranking: `semantic status|enable|disable|sync`

```powershell
mobius-connect semantic status
mobius-connect semantic enable --model nomic-embed-text
mobius-connect semantic sync
mobius-connect semantic disable
```

Off by default. `enable` discloses that already-indexed chunks may be sent to **localhost Ollama**; nothing is ever downloaded. `sync` (re)builds the derived vector index. Missing Ollama or model = fail open to BM25 with `semantic_status` explaining why. Vectors are regenerable; transcripts remain the authority.

### 9. Hand off to another agent: `handoff prepare` → `approvals handoff` → `handoff commit`

```powershell
mobius-connect handoff prepare --harness claude --cwd . <session-id> [<session-id-2>]
mobius-connect approvals handoff <handoff-id>
mobius-connect handoff commit <handoff-id> --approval-token <token>
mobius-connect handoff status <handoff-id>
```

Three steps, one human in the middle:

1. `prepare` builds a **references-only** envelope (entry sessions, confirmed ancestors, locators, token estimate). It launches nothing and copies no transcript bodies.
2. `approvals handoff` is the human gate: it prints exactly what will be sealed and waits for you to type `APPROVE`, then mints a short-lived, single-use token. Shown once; do not log it.
3. `commit` seals the handoff and (for verified launchers) starts the target session. `status` distinguishes `prepared` / `starting` / `awaiting_identity` / `bound` / `failed` — a started PID is **not** a bound session; never retry an unknown launch with the same token.

### 10. Freeze a reference package: `memory reference`

```powershell
mobius-connect memory reference <session-id>
```

Writes an immutable lineage manifest under `handoff-graphs/` and prints a launch context (paths + confirmed edges). It returns references to the caller; it does not inject memory into any session.

### 11. MCP for agents: `mcp serve`

```powershell
mobius-connect init          # MCP refuses to start without an existing vault
mobius-connect mcp serve
```

stdio-only server an MCP client launches as a subprocess. Metadata tools (`list_sessions`, `get_session`, `get_index_health`, `get_lineage`, `prepare_handoff`, `get_handoff_status`, `request_session_approval`, `resolve_session`, `build_memory_reference`) are open; body-returning tools (`search_sessions`, `mome_recall`, `read_session_range`) and `commit_handoff` require a token minted by a human in a real terminal. Full tool table and client config: [docs/MCP.md](docs/MCP.md).

## Security

Treat indexed transcripts as untrusted historical data, not as instructions.

- Source roots are finite and approved by a person; `sources add` is terminal-only.
- Original files stay read-only. OpenCode is opened read-only.
- Ordinary chat does not search other sessions.
- A handoff envelope contains identities and edges, never transcript bodies.
- Tokens are short-lived, single-use, scoped. Shown once. Do not log. Do not retry an unknown launch; check `handoff status`.
- Optional embeddings talk only to localhost Ollama after `semantic enable`, change ranking only, and never widen MCP grants.

## Supported harnesses

| Harness | Index | Native resume |
| --- | --- | --- |
| Codex | JSONL | via desktop when protocol is verified |
| Claude Code | JSONL / project sessions | via desktop when protocol is verified |
| OpenCode | read-only `opencode.db` | not in this preview |
| Pi | approved roots | via desktop when protocol is verified |
| Grok | approved roots | inspect / search / handoff only |
| OMP | approved roots | documented `--cwd … --resume …` from desktop |
| OpenClaw | not shipped | roadmap |

## Documentation

| Goal | Start here |
| --- | --- |
| Desktop workspace | [TardisBooo/Mobius](https://github.com/TardisBooo/Mobius) |
| Command tutorial | this README |
| MCP tools and grants | [docs/MCP.md](docs/MCP.md) |
| Repo split | [desktop docs/REPOS.md](https://github.com/TardisBooo/Mobius/blob/main/docs/REPOS.md) |
| Handoff as a graph | [blog 01](https://github.com/TardisBooo/Mobius/blob/main/docs/blog/01-cross-agent-handoff.md) |
| Citation vs summary | [blog 02](https://github.com/TardisBooo/Mobius/blob/main/docs/blog/02-citation-not-summary.md) |
| Approval tokens | [blog 03](https://github.com/TardisBooo/Mobius/blob/main/docs/blog/03-approval-tokens.md) |
| Session Hub SOP | [sop-session-hub.md](https://github.com/TardisBooo/Mobius/blob/main/docs/sop-session-hub.md) |

## Development

```powershell
cargo test --offline
```

Use `--data-root` with an empty absolute directory. Do not point tests at `D:\DataVault\Mobius` or any real session tree.

## License

[MIT](LICENSE). Möbius is unrelated to Microsoft Mobius, ControlTheory Möbius, or Circular Labs Mobius.

# mobius-connect

Explicit session search and ancestry access, independent of the Möbius desktop.

**Development status:** not released. Native launch approvals, plugin host
acceptance and cross-harness lifecycle verification remain release gates.

Handoffs describe how memory flowed between native sessions. They do not summarize,
compress or copy transcripts, nor instruct a model to read every event. A source
reference is not a claim that a model has read or understood it.

This is a separate repository from the desktop. Both use the same Rust lineage
core, pinned to immutable Git revision `6f2765ab493f7a64901c9eb69e6f41bde38ed922`.
It builds without an adjacent desktop checkout. This is a verified development
checkpoint, not a release certification.

Source and regenerable builds live under the Möbius family workspace. Shared
durable application data defaults to `D:\DataVault\Mobius`; accepted deliverables
belong in `D:\AcceptedArtifacts\Mobius`. Use `--data-root` with a separate test
directory for isolated verification. Native harness code/configuration and
original session files are never rewritten.

## Explicit workflow

1. Run `mobius-connect init`, then `sources add codex <sessions-directory>` and
   `sources refresh` to index a source the user explicitly selects.
2. Use `sessions list`, `sessions search <query>` or `graph show <session-id>`.
   Search currently uses the shared lexical index; semantic search is not yet
   exposed by this CLI. Graph exports support JSON, Mermaid and a static HTML
   reference list, not an interactive desktop-equivalent viewer.
3. Prepare a new session: `handoff prepare --harness codex --cwd <project> <id>`.
   Multiple source IDs create one reviewed union of their ancestors. Preparation
   does not launch a harness or copy conversation bodies.
4. Review the result and run `approvals handoff <handoff-id>` in a human terminal.
   Consent is short-lived, single-use and scoped to that exact handoff. The MCP
   server cannot mint approvals. Do not share tokens in screenshots or logs.
5. Explicitly commit with the issued token, then inspect `handoff status <id>`.
   Native launch currently targets Windows and a separate PowerShell console.
   `awaiting_identity` is not completion; refresh the index to resolve an actual
   target session. Unknown launch outcomes must be inspected, not retried.

`mcp serve` uses stdio only and requires prior initialization. Metadata and graph
tools do not return transcript bodies. Search excerpts and bounded source reads
require separate scoped approvals. The optional plugin under `plugins/` has no
hooks and disables implicit skill invocation; native plugin-host acceptance is
still pending. Do not treat development tests as release certification.

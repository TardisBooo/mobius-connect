mod handoff;
use anyhow::{Context, Result, bail, ensure};
use clap::{Parser, Subcommand};
use mydesk_core::*;
use serde_json::{Value, json};
use std::{
    io::{self, BufRead, IsTerminal, Read, Write},
    path::PathBuf,
    str::FromStr,
};

#[derive(Parser)]
#[command(
    version,
    about = "Explicit session lineage and source access; no summarization"
)]
struct Cli {
    #[arg(long, global = true)]
    data_root: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Handoff {
        #[command(subcommand)]
        action: Handoff,
    },
    Init,
    Doctor,
    Sources {
        #[command(subcommand)]
        action: Sources,
    },
    Sessions {
        #[command(subcommand)]
        action: Sessions,
    },
    Graph {
        #[command(subcommand)]
        action: Graph,
    },
    Memory {
        #[command(subcommand)]
        action: Memory,
    },
    Approvals {
        #[command(subcommand)]
        action: Approvals,
    },
    Mcp {
        #[command(subcommand)]
        action: Mcp,
    },
    /// Detect installed harnesses and print MCP/skill install steps. Never writes hooks.
    Setup,
    /// One-shot health and next command for agents.
    Triage,
    Semantic {
        #[command(subcommand)]
        action: Semantic,
    },
    Mome {
        #[command(subcommand)]
        action: Mome,
    },
}
#[derive(Subcommand)]
enum Sources {
    List,
    Add { harness: String, path: PathBuf },
    Refresh,
}
#[derive(Subcommand)]
enum Sessions {
    List {
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    Show {
        id: String,
    },
    Search {
        query: String,
        #[arg(long, default_value_t = 30)]
        limit: usize,
    },
    Read {
        id: String,
        #[arg(long, default_value_t = 0)]
        offset: u64,
        #[arg(long, default_value_t = 4096)]
        bytes: usize,
    },
    Alias {
        id: String,
        alias: String,
    },
}
#[derive(Subcommand)]
enum Graph {
    Show {
        #[arg(required = true)]
        sessions: Vec<String>,
    },
    Export {
        #[arg(long, default_value="json", value_parser=["json","mermaid","html"])]
        format: String,
        #[arg(required = true)]
        sessions: Vec<String>,
    },
}
#[derive(Subcommand)]
enum Memory {
    Reference {
        #[arg(required = true)]
        sessions: Vec<String>,
    },
}
#[derive(Subcommand)]
enum Approvals {
    Review { request: PathBuf },
    Handoff { id: String },
}
#[derive(Subcommand)]
enum Handoff {
    Prepare {
        #[arg(long)]
        harness: String,
        #[arg(long)]
        cwd: PathBuf,
        #[arg(required = true)]
        sessions: Vec<String>,
    },
    Commit {
        id: String,
        #[arg(long)]
        approval_token: String,
    },
    Status {
        id: String,
    },
}
#[derive(Subcommand)]
enum Mcp {
    Serve,
}
#[derive(Subcommand)]
enum Semantic {
    Status,
    Enable {
        #[arg(long, default_value = "nomic-embed-text")]
        model: String,
    },
    Disable,
    Sync,
}
#[derive(Subcommand)]
enum Mome {
    Recall {
        query: String,
        #[arg(long)]
        provider: Vec<String>,
        #[arg(long)]
        max_tokens: Option<usize>,
    },
}

fn print(value: impl serde::Serialize) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}
fn query(desk: &MyDesk, text: &str, limit: usize) -> Result<Vec<SessionSearchHit>> {
    desk.database.query_sessions(&SessionQuery {
        query: text.into(),
        workspace_id: None,
        checkout_id: None,
        providers: vec![],
        limit,
    })
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut paths = WorkspacePaths::from_environment();
    if let Some(root) = cli.data_root {
        ensure!(root.is_absolute(), "data root must be absolute");
        paths.data_root = root;
        paths.workspace_root = paths.data_root.join("workspace-index");
        paths.artifacts_root = paths.data_root.join("artifacts");
        paths.catalog_root = paths.data_root.join("catalog");
    }
    if matches!(cli.command, Command::Mcp { .. }) {
        ensure!(
            paths.database_path().exists(),
            "run mobius-connect init explicitly before starting MCP"
        );
    }
    let desk = MyDesk::open(paths)?;
    match cli.command {
        Command::Handoff { action } => match action {
            Handoff::Prepare {
                harness,
                cwd,
                sessions,
            } => print(handoff::prepare(&desk, &sessions, &harness, &cwd)?),
            Handoff::Commit { id, approval_token } => {
                print(handoff::commit(&desk, &id, &approval_token)?)
            }
            Handoff::Status { id } => print(desk.database.handoff_status(&id)?),
        },
        Command::Init | Command::Doctor => print(desk.health()?),
        Command::Sources { action } => match action {
            Sources::List => print(load_approved_session_sources(&desk.paths)?),
            Sources::Add { harness, path } => print(add_manual_session_source(
                &desk.paths,
                AgentKind::from_str(&harness)?,
                &path,
            )?),
            Sources::Refresh => {
                print(ProviderIndexer::new(&desk.database, &desk.paths).index_approved_roots()?)
            }
        },
        Command::Sessions { action } => match action {
            Sessions::List { limit } => print(
                query(&desk, "", limit)?
                    .into_iter()
                    .map(|hit| hit.session)
                    .collect::<Vec<_>>(),
            ),
            Sessions::Show { id } => print(
                desk.session_lineage(&[id.clone()])?
                    .nodes
                    .into_iter()
                    .find(|n| n.session_id == id),
            ),
            Sessions::Search { query: text, limit } => {
                let hits = query(&desk, &text, limit)?;
                print(json!({
                    "retrieval_mode":"lexical_bm25",
                    "semantic_status": desk.semantic_status()?,
                    "hits": hits,
                    "suggested_next_commands": [
                        "mobius-connect sessions show <session-id>",
                        "mobius-connect graph show <session-id>",
                        "mobius-connect handoff prepare --harness <target> --cwd <dir> <session-id>"
                    ]
                }))
            }
            Sessions::Read { id, offset, bytes } => {
                print(desk.read_session_source_range(&id, offset, bytes)?)
            }
            Sessions::Alias { id, alias } => {
                desk.database.set_session_alias(&id, &alias)?;
                print(json!({"session_id":id,"alias":alias}))
            }
        },
        Command::Graph { action } => match action {
            Graph::Show { sessions } => print(desk.session_lineage(&sessions)?),
            Graph::Export { format, sessions } => {
                let graph = desk.session_lineage(&sessions)?;
                match format.as_str() {
                    "mermaid" => {
                        println!("{}", mermaid(&graph));
                        Ok(())
                    }
                    "html" => {
                        println!("{}", html(&graph));
                        Ok(())
                    }
                    _ => print(graph),
                }
            }
        },
        Command::Memory {
            action: Memory::Reference { sessions },
        } => {
            let graph = desk.save_lineage(&sessions)?;
            print(
                json!({"graph":graph,"manifest_path":desk.lineage_path(&graph.id)?,"context":desk.lineage_launch_context(&graph.id, &sessions[0])?}),
            )
        }
        Command::Approvals {
            action: Approvals::Review { request },
        } => {
            let mut bytes = Vec::new();
            std::fs::File::open(request)?
                .take(16385)
                .read_to_end(&mut bytes)?;
            ensure!(bytes.len() <= 16384, "approval request too large");
            let request: McpApprovalRequest = serde_json::from_slice(&bytes)?;
            review_approval(&desk, request)
        }
        Command::Approvals {
            action: Approvals::Handoff { id },
        } => review_approval(&desk, handoff::approval_request(&desk, &id)?),
        Command::Mcp { action: Mcp::Serve } => serve(&desk),
        Command::Setup => print(setup_report(&desk)?),
        Command::Triage => print(triage_report(&desk)?),
        Command::Semantic { action } => match action {
            Semantic::Status => print(desk.semantic_status()?),
            Semantic::Enable { model } => print(desk.set_semantic_enabled(true, Some(&model))?),
            Semantic::Disable => print(desk.set_semantic_enabled(false, None)?),
            Semantic::Sync => print(desk.sync_mome_embeddings()?),
        },
        Command::Mome { action } => match action {
            Mome::Recall {
                query,
                provider,
                max_tokens,
            } => {
                let providers = provider
                    .iter()
                    .map(|value| AgentKind::from_str(value))
                    .collect::<Result<Vec<_>, _>>()?;
                print(desk.mome_recall(&MomeRecallRequest {
                    query,
                    workspace_id: None,
                    checkout_id: None,
                    providers,
                    max_tokens,
                    retrieval_mode: None,
                })?)
            }
        },
    }
}

fn setup_report(desk: &MyDesk) -> Result<Value> {
    Ok(json!({
        "kind": "mobius_connect_setup",
        "writes_harness_configuration": false,
        "hooks": "none",
        "health": desk.health()?,
        "semantic": desk.semantic_status()?,
        "mcp": {
            "stdio": "mobius-connect mcp serve",
            "note": "Add this binary as an stdio MCP server. Möbius never installs SessionStart hooks."
        },
        "skill": "plugins/mobius-connect/skills/mobius-memory/SKILL.md",
        "next": [
            "mobius-connect sources add <harness> <sessions-directory>",
            "mobius-connect sources refresh",
            "mobius-connect sessions search <query>"
        ]
    }))
}

fn triage_report(desk: &MyDesk) -> Result<Value> {
    let health = desk.health()?;
    let sources = load_approved_session_sources(&desk.paths)?;
    let next = if sources.roots.is_empty() {
        "mobius-connect sources add <harness> <sessions-directory>"
    } else {
        "mobius-connect sessions search <query>"
    };
    Ok(json!({
        "kind": "mobius_connect_triage",
        "health": health,
        "approved_sources": sources.roots.len(),
        "semantic": desk.semantic_status()?,
        "next_command": next,
        "recommended_commands": [
            next,
            "mobius-connect doctor",
            "mobius-connect graph show <session-id>"
        ]
    }))
}

fn review_approval(desk: &MyDesk, request: McpApprovalRequest) -> Result<()> {
    ensure!(
        io::stdin().is_terminal(),
        "approval needs a human terminal, not piped or MCP input"
    );
    eprintln!(
        "Review this exact access request:\n{}\nType APPROVE to grant one short-lived token:",
        serde_json::to_string_pretty(&request)?
    );
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    ensure!(answer.trim() == "APPROVE", "approval cancelled");
    print(McpApprovalStore::for_paths(&desk.paths)?.grant(request)?)
}

fn escaped(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
fn mermaid(graph: &lineage::LineageManifest) -> String {
    let mut out = String::from("flowchart LR\n");
    for (i, n) in graph.nodes.iter().enumerate() {
        out.push_str(&format!(
            "  n{i}[\"{} · {}\"]\n",
            escaped(n.harness.as_deref().unwrap_or("unknown")),
            escaped(n.native_id.as_deref().unwrap_or(&n.session_id))
        ));
    }
    for edge in &graph.edges {
        let source = graph
            .nodes
            .iter()
            .position(|n| n.session_id == edge.source)
            .unwrap();
        let target = graph
            .nodes
            .iter()
            .position(|n| n.session_id == edge.target)
            .unwrap();
        out.push_str(&format!("  n{source} --> n{target}\n"));
    }
    out
}
fn html(graph: &lineage::LineageManifest) -> String {
    let mut out = String::from(
        "<!doctype html><html lang=en><meta charset=utf-8><meta name=viewport content='width=device-width'><title>Session lineage</title><style>body{font:16px system-ui;max-width:1100px;margin:40px auto;padding:20px;color:#18232d;background:#f7faf9}article{padding:18px;border:1px solid #bac6c5;border-radius:10px;margin:12px 0}code,pre{overflow-wrap:anywhere}pre{background:#eef2f1;padding:12px;border-radius:8px}li{margin:8px 0}</style><h1>Session lineage</h1><p>Reference-only DAG export. No transcript excerpts or local source paths are included.</p>",
    );
    out.push_str("<h2>Graph</h2><pre>");
    out.push_str(&escaped(&mermaid(graph)));
    out.push_str("</pre>");
    for n in &graph.nodes {
        out.push_str(&format!("<article id='n{}'><strong>{}</strong><p>{}</p><code>{}</code><p>Updated: {}</p></article>", graph.nodes.iter().position(|item| item.session_id == n.session_id).unwrap(), escaped(n.harness.as_deref().unwrap_or("unknown")), escaped(n.title.as_deref().unwrap_or("Untitled")), escaped(n.native_id.as_deref().unwrap_or(&n.session_id)), escaped(n.updated_at.as_deref().unwrap_or("unknown"))));
    }
    out.push_str("<h2>Handoffs</h2><ul>");
    for e in &graph.edges {
        out.push_str(&format!(
            "<li><code>{}</code> → <code>{}</code></li>",
            escaped(&e.source),
            escaped(&e.target)
        ));
    }
    out.push_str("</ul></html>");
    out
}

fn tools() -> Value {
    let entries = [
        (
            "prepare_handoff",
            "Prepare immutable references only; does not launch",
            json!({"session_ids":{"type":"array","items":{"type":"string"},"minItems":1},"harness":{"type":"string","enum":["codex","claude","pi","grok","omp","opencode"]},"cwd":{"type":"string"}}),
            vec!["session_ids", "harness", "cwd"],
        ),
        (
            "commit_handoff",
            "Launch exactly one reviewed handoff using a human-issued single-use token",
            json!({"handoff_id":{"type":"string"},"approval_token":{"type":"string"}}),
            vec!["handoff_id", "approval_token"],
        ),
        (
            "get_handoff_status",
            "Read lifecycle state; started does not mean identity bound",
            json!({"handoff_id":{"type":"string"}}),
            vec!["handoff_id"],
        ),
        ("get_index_health", "Read index health", json!({}), vec![]),
        (
            "list_sessions",
            "List catalogue metadata; no transcript bodies",
            json!({"limit":{"type":"integer","minimum":1,"maximum":500}}),
            vec![],
        ),
        (
            "get_lineage",
            "Read session ancestry graph; no transcript bodies",
            json!({"session_ids":{"type":"array","items":{"type":"string"},"minItems":1}}),
            vec!["session_ids"],
        ),
        (
            "resolve_session",
            "Resolve an approved native source location",
            json!({"session_id":{"type":"string"}}),
            vec!["session_id"],
        ),
        (
            "search_sessions",
            "Search raw indexed excerpts with an exact-query approval",
            json!({"query":{"type":"string"},"providers":{"type":"array","items":{"type":"string"},"minItems":1},"max_chars":{"type":"integer","minimum":1,"maximum":48000},"approval_token":{"type":"string"}}),
            vec!["query", "providers", "max_chars", "approval_token"],
        ),
        (
            "build_memory_reference",
            "Return graph references without launching or injecting",
            json!({"session_ids":{"type":"array","items":{"type":"string"},"minItems":1}}),
            vec!["session_ids"],
        ),
        (
            "read_session_range",
            "Explicit byte range; requires a human-issued scoped token",
            json!({"session_id":{"type":"string"},"offset":{"type":"integer","minimum":0},"bytes":{"type":"integer","minimum":1,"maximum":16384},"approval_token":{"type":"string"}}),
            vec!["session_id", "offset", "bytes", "approval_token"],
        ),
        (
            "get_session",
            "Get one native session identity without loading its transcript",
            json!({"provider":{"type":"string"},"session_id":{"type":"string"}}),
            vec!["provider", "session_id"],
        ),
        (
            "mome_recall",
            "Bounded cited recall over the regenerable local index; hybrid only when embeddings are enabled and ready",
            json!({"query":{"type":"string"},"providers":{"type":"array","items":{"type":"string"},"minItems":1},"max_tokens":{"type":"integer","minimum":1,"maximum":1200},"approval_token":{"type":"string"}}),
            vec!["query", "providers", "approval_token"],
        ),
        (
            "request_session_approval",
            "Describe how a human grants a short-lived token. This server cannot mint one.",
            json!({"operation":{"type":"string"}}),
            vec!["operation"],
        ),
    ];
    json!({"tools":entries.into_iter().map(|(name,description,properties,required)| json!({"name":name,"description":description,"inputSchema":{"type":"object","properties":properties,"required":required,"additionalProperties":false},"annotations":{"readOnlyHint":!matches!(name,"prepare_handoff"|"commit_handoff"),"destructiveHint":name=="commit_handoff","openWorldHint":name=="commit_handoff"}})).collect::<Vec<_>>()})
}
fn required<'a>(args: &'a Value, field: &str) -> Result<&'a str> {
    args.get(field)
        .and_then(Value::as_str)
        .with_context(|| format!("missing {field}"))
}
fn call(desk: &MyDesk, name: &str, args: Value) -> Result<Value> {
    match name {
        "prepare_handoff" => handoff::prepare(
            desk,
            &serde_json::from_value::<Vec<String>>(args["session_ids"].clone())?,
            required(&args, "harness")?,
            std::path::Path::new(required(&args, "cwd")?),
        ),
        "commit_handoff" => handoff::commit(
            desk,
            required(&args, "handoff_id")?,
            required(&args, "approval_token")?,
        ),
        "get_handoff_status" => desk.database.handoff_status(required(&args, "handoff_id")?),
        "get_index_health" => Ok(serde_json::to_value(desk.health()?)?),
        "list_sessions" => Ok(serde_json::to_value(
            query(
                desk,
                "",
                args["limit"].as_u64().unwrap_or(100).min(500) as usize,
            )?
            .into_iter()
            .map(|h| h.session)
            .collect::<Vec<_>>(),
        )?),
        "get_lineage" | "build_memory_reference" => {
            let ids: Vec<String> = serde_json::from_value(args["session_ids"].clone())?;
            Ok(serde_json::to_value(desk.session_lineage(&ids)?)?)
        }
        "resolve_session" => {
            let id = required(&args, "session_id")?;
            Ok(serde_json::to_value(
                desk.session_lineage(&[id.into()])?
                    .nodes
                    .into_iter()
                    .find(|n| n.session_id == id),
            )?)
        }
        "search_sessions" => {
            let text = required(&args, "query")?;
            let providers: Vec<String> = serde_json::from_value(args["providers"].clone())?;
            let budget = args["max_chars"].as_u64().context("max_chars required")? as usize;
            ensure!(
                !providers.is_empty() && (1..=48000).contains(&budget),
                "invalid search scope or budget"
            );
            let agents = providers
                .iter()
                .map(|p| AgentKind::from_str(p))
                .collect::<Result<Vec<_>, _>>()?;
            McpApprovalStore::for_paths(&desk.paths)?.authorize(
                required(&args, "approval_token")?,
                &McpApprovalAttempt {
                    operation: "search_sessions".into(),
                    query: Some(text.into()),
                    workspace_id: None,
                    checkout_id: None,
                    providers,
                    provider: None,
                    session_id: None,
                    start_ordinal: None,
                    end_ordinal: None,
                    requested_chars: budget,
                },
            )?;
            let hits = desk.database.query_sessions(&SessionQuery {
                query: text.into(),
                workspace_id: None,
                checkout_id: None,
                providers: agents,
                limit: 30,
            })?;
            let mut remaining = budget;
            let hits: Vec<_> = hits.into_iter().map(|hit| {
                let body = hit.message.as_ref().map(|m| m.content.as_str()).unwrap_or("");
                let excerpt: String = body.chars().take(remaining).collect(); remaining -= excerpt.chars().count();
                json!({"session_id":hit.session.id,"harness":hit.session.provider,"title":hit.session.title,
                    "raw_excerpt":excerpt,"excerpt_truncated":excerpt.chars().count() < body.chars().count(),"catalogue_coverage":hit.session.metadata.get("catalogue_coverage")})
            }).collect();
            Ok(json!({"retrieval_mode":"lexical","hits":hits}))
        }
        "read_session_range" => {
            let id = required(&args, "session_id")?;
            let offset = args["offset"].as_u64().context("offset required")?;
            let length = args["bytes"].as_u64().context("bytes required")?;
            ensure!((1..=16384).contains(&length), "request 1–16384 bytes");
            let session = desk
                .database
                .get_session(id)?
                .context("session not found")?;
            McpApprovalStore::for_paths(&desk.paths)?.authorize(
                required(&args, "approval_token")?,
                &McpApprovalAttempt {
                    operation: "read_session_range".into(),
                    query: Some(format!("bytes:{offset}:{length}")),
                    workspace_id: None,
                    checkout_id: None,
                    providers: vec![],
                    provider: Some(session.provider.to_string()),
                    session_id: Some(id.into()),
                    start_ordinal: None,
                    end_ordinal: None,
                    requested_chars: length as usize * 2,
                },
            )?;
            desk.read_session_source_range(id, offset, length as usize)
        }
        "get_session" => {
            let session = desk
                .database
                .get_provider_session(required(&args, "provider")?, required(&args, "session_id")?)?
                .context("session not found")?;
            Ok(serde_json::to_value(session)?)
        }
        "mome_recall" => {
            let text = required(&args, "query")?;
            let providers: Vec<String> = serde_json::from_value(args["providers"].clone())?;
            let agents = providers
                .iter()
                .map(|value| AgentKind::from_str(value))
                .collect::<Result<Vec<_>, _>>()?;
            McpApprovalStore::for_paths(&desk.paths)?.authorize(
                required(&args, "approval_token")?,
                &McpApprovalAttempt {
                    operation: "mome_recall".into(),
                    query: Some(text.into()),
                    workspace_id: None,
                    checkout_id: None,
                    providers: providers.clone(),
                    provider: None,
                    session_id: None,
                    start_ordinal: None,
                    end_ordinal: None,
                    requested_chars: 4800,
                },
            )?;
            Ok(serde_json::to_value(desk.mome_recall(&MomeRecallRequest {
                query: text.into(),
                workspace_id: None,
                checkout_id: None,
                providers: agents,
                max_tokens: args["max_tokens"].as_u64().map(|value| value as usize),
                retrieval_mode: None,
            })?)?)
        }
        "request_session_approval" => Ok(json!({
            "status": "needs_user_approval",
            "operation": required(&args, "operation")?,
            "next_step": "Run mobius-connect approvals review <request.json> in a human terminal. This MCP server cannot mint a token."
        })),
        _ => bail!("unknown tool"),
    }
}
fn serve(desk: &MyDesk) -> Result<()> {
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    loop {
        let mut line = Vec::new();
        let read = (&mut input)
            .take(1024 * 1024 + 1)
            .read_until(b'\n', &mut line)?;
        if read == 0 {
            break;
        }
        ensure!(line.len() <= 1024 * 1024, "MCP request too large");
        let request: Value = match serde_json::from_slice(&line) {
            Ok(value) => value,
            Err(_) => {
                writeln!(
                    output,
                    "{}",
                    json!({"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Parse error"}})
                )?;
                output.flush()?;
                continue;
            }
        };
        let Some(id) = request.get("id") else {
            continue;
        };
        let result: Result<Value> = match request["method"].as_str().unwrap_or("") {
            "initialize" => Ok(
                json!({"protocolVersion":"2025-06-18","capabilities":{"tools":{}},"serverInfo":{"name":"mobius-connect","version":env!("CARGO_PKG_VERSION")}}),
            ),
            "ping" => Ok(json!({})),
            "tools/list" => Ok(tools()),
            "tools/call" => {
                let params = &request["params"];
                match call(
                    desk,
                    params["name"].as_str().unwrap_or(""),
                    params["arguments"].clone(),
                ) {
                    Ok(data) => Ok(
                        json!({"content":[{"type":"text","text":serde_json::to_string(&data)?}],"structuredContent":{"result":data},"isError":false}),
                    ),
                    Err(error) => Ok(
                        json!({"content":[{"type":"text","text":error.to_string()}],"isError":true}),
                    ),
                }
            }
            _ => Err(anyhow::anyhow!("Method not found")),
        };
        let response = match result {
            Ok(result) => json!({"jsonrpc":"2.0","id":id,"result":result}),
            Err(error) => {
                json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":error.to_string()}})
            }
        };
        writeln!(output, "{response}")?;
        output.flush()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn export_escaping_is_safe() {
        assert_eq!(escaped("<script>\"&"), "&lt;script&gt;&quot;&amp;");
    }
    #[test]
    fn tool_contract_has_no_self_grant_or_implicit_launch() {
        let catalogue = tools().to_string();
        assert!(!catalogue.contains("grant_approval"));
        let definition = tools();
        let commit = definition["tools"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["name"] == "commit_handoff")
            .unwrap();
        assert!(
            commit["inputSchema"]["required"]
                .as_array()
                .unwrap()
                .contains(&json!("approval_token"))
        );
        assert_eq!(commit["annotations"]["readOnlyHint"], false);
        assert!(catalogue.contains("read_session_range"));
    }
}

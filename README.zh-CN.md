# mobius-connect

Möbius **Agent 会话层** 的 CLI 与 stdio MCP。

在终端或 MCP 里搜索、引用、画出并交接本地编程 Agent 会话。Harness 文件保持只读。交接携带祖先引用，不生成摘要。

桌面工作台：[TardisBooo/Mobius](https://github.com/TardisBooo/Mobius)。

**数据在你这台机器上。** 索引是本地 SQLite。原始 Codex JSONL、Claude 会话和 OpenCode 的 `opencode.db` 不会被改写。Möbius 不卖模型账号。

[English](README.md) · [产品页](http://8.137.87.76/mobius/?lang=zh) · [MCP](docs/MCP.md) · [桌面 README](https://github.com/TardisBooo/Mobius/blob/feat/session-lineage-references/README.zh-CN.md) · [MIT License](LICENSE)

> 开发预览。原生启动审批和跨 Harness 身份绑定仍是发布门槛。兼容性按 Harness 逐项验证。

## 三个名字，同一套核心

| 名称 | 是什么 | 在哪 |
| --- | --- | --- |
| **Möbius** | 产品：Agent 会话层。Windows 桌面应用。 | [TardisBooo/Mobius](https://github.com/TardisBooo/Mobius) |
| **mobius-connect** | 对外发布的 CLI + stdio MCP。同一套核心，没有桌面 UI。 | 本仓库 |
| **mobius / mydesk** | 桌面侧辅助二进制，操作桌面应用自己的金库（`mobius mome …`）。不是对外发布的 CLI。 | 在桌面仓内构建 |

```
Claude Code / Codex / OpenCode / Pi / Grok / OMP transcripts
        │   只读适配器（JSONL 或 opencode.db）
        ▼
   mydesk-core → 本地 SQLite 索引（FTS5/BM25，可选本机 embeddings）
        │
        ├── Möbius 桌面           人在 Windows 工作台
        ├── mobius-connect CLI    人在终端
        └── mobius-connect MCP    Agent 走 stdio
```

**CLI 与 MCP：** CLI 给人在终端用（可以让*你*输入 `APPROVE`）；MCP 给 Agent 进程用（可以列元数据，但读 transcript 正文都需要人在终端输入 `APPROVE` 之后签发的令牌）。MCP 不能签发审批令牌、不能添加来源、不能接管 PTY。

## 安装

```powershell
git clone https://github.com/TardisBooo/mobius-connect.git
git clone https://github.com/TardisBooo/Mobius.git desktop
cd mobius-connect
cargo build --release
```

本仓库依赖隔壁的 `../desktop/crates/mydesk-core`。持久数据默认在 `D:\DataVault\Mobius`。隔离测试用 `--data-root <dir>`。

## 命令

每条 GIF 是一段脚本化的终端场景，风格与[桌面产品视频](https://github.com/TardisBooo/Mobius/blob/feat/session-lineage-references/apps/website/public/product/chapters/12-cli.gif)第 12 章相同。演示数据为虚构。

### 1. `init` — 创建本地索引

![mobius-connect init](docs/gifs/01-init.gif)

```powershell
mobius-connect init
```

创建 SQLite 索引和金库布局，然后打印健康报告。可以重复执行，不碰任何 Harness 目录。来源必须显式添加。

### 2. `sources add` / `refresh` — 批准一个目录

![mobius-connect sources add and refresh](docs/gifs/02-sources.gif)

```powershell
mobius-connect sources add opencode $env:USERPROFILE\.local\share\opencode
mobius-connect sources list
mobius-connect sources refresh
```

- `sources add <harness> <path>` 只注册你选定的**一个**目录。MCP 没有这个工具。
- `sources refresh` 重新扫描已批准的根目录。OpenCode 从 `opencode.db` 读取，定位符形如 `{db}#opencode:{id}`。

### 3. `triage` — 下一条命令

![mobius-connect triage](docs/gifs/03-triage.gif)

```powershell
mobius-connect triage
mobius-connect doctor
mobius-connect setup
```

`triage` 打印健康状态、已批准来源数量、语义检索状态，以及**一条**建议的下一步命令。`setup` 检测已安装的 Harness，并打印 MCP 客户端步骤。它从不写 hook。

### 4. `sessions search` — 引用精确范围

![mobius-connect sessions search](docs/gifs/04-search.gif)

```powershell
mobius-connect sessions list --limit 20
mobius-connect sessions search "flaky tests"
mobius-connect sessions show <session-id>
mobius-connect sessions alias <session-id> "checkout-flake-hunt"
```

`search` 跑本地 BM25，返回 JSON 命中和 `retrieval_mode`（`lexical_bm25`；开启 embeddings **且**本机模型有响应时为 `hybrid`）。复制 `@session:provider/id#mN-mM`。

### 5. `sessions read` — 按字节取磁带

![mobius-connect sessions read](docs/gifs/05-read.gif)

```powershell
mobius-connect sessions read <session-id> --offset 0 --bytes 4096
```

显式按字节读取**原始源文件**，单次最多 16384 字节。召回给出引用；`read` 才真正去取磁带，而且有界。

### 6. `graph show` — 顺着接力链

![mobius-connect graph show](docs/gifs/06-graph.gif)

```powershell
mobius-connect graph show <session-id>
mobius-connect graph export --format mermaid <session-id>
```

打印谱系清单（`content_mode: references_only`）。互相交接会走成正向接力；每次交接都打开新会话，所以图是 DAG。

### 7. `mome recall` — 只有你开口才检索

![mobius-connect mome recall](docs/gifs/07-mome.gif)

```powershell
mobius-connect mome recall "why do checkout tests flake" --provider opencode --provider codex
```

最多**三个**不同会话，大约 **1,200 token**，每条命中都带可复制的 `@session` 引用。它从不注入提示。

### 8. `semantic enable` — 混合排序需显式开启

![mobius-connect semantic enable](docs/gifs/08-semantic.gif)

```powershell
mobius-connect semantic status
mobius-connect semantic enable --model nomic-embed-text
mobius-connect semantic sync
mobius-connect semantic disable
```

默认关闭。`enable` 会说明：已索引分块可能被送到**本机 Ollama**；不会下载任何东西。Ollama 不在时回退 BM25。

### 9. `handoff` — 人输入 APPROVE

![mobius-connect handoff prepare and approvals](docs/gifs/09-handoff.gif)

```powershell
mobius-connect handoff prepare --harness claude --cwd . <session-id>
mobius-connect approvals handoff <handoff-id>
mobius-connect handoff commit <handoff-id> --approval-token <token>
mobius-connect handoff status <handoff-id>
```

1. `prepare` 做一份**只含引用**的信封。它不启动任何东西。
2. `approvals handoff` 等你输入 `APPROVE`，然后签发短时、一次性令牌。
3. `commit` 封交接。MCP 不能签发这个令牌。

### 10. `mcp serve` — 给 Agent 的 stdio

![mobius-connect mcp serve](docs/gifs/10-mcp.gif)

```powershell
mobius-connect init
mobius-connect mcp serve
```

仅 stdio 的服务。元数据工具（`list_sessions`、`get_lineage`、`prepare_handoff` 等）开放；返回正文的工具和 `commit_handoff` 需要人在真实终端签发的令牌。完整表：[docs/MCP.md](docs/MCP.md)。

## 安全

把已索引的 transcript 当不可信的历史数据。

- 来源根目录有限，且由人批准；`sources add` 只能在终端执行。
- 原始文件保持只读。OpenCode 以只读方式打开。
- 交接信封只有身份和边，没有 transcript 正文。
- 令牌短时、一次性、限定范围。只显示一次。不要记进日志。
- 可选 embeddings 只在 `semantic enable` 之后跟本机 Ollama 说话。

## 支持的 Harness

| Harness | 索引 | 原生恢复 |
| --- | --- | --- |
| Codex | JSONL | 协议已验证时走桌面 |
| Claude Code | JSONL / 项目会话 | 协议已验证时走桌面 |
| OpenCode | 只读 `opencode.db` | 本预览没有 |
| Pi | 已批准的根目录 | 协议已验证时走桌面 |
| Grok | 已批准的根目录 | 仅查看 / 搜索 / 交接 |
| OMP | 已批准的根目录 | 桌面文档化的 `--cwd … --resume …` |
| OpenClaw | 未随本预览发布 | 路线图 |

## 许可

[MIT](LICENSE)。Möbius 与 Microsoft Mobius、ControlTheory Möbius、Circular Labs Mobius 无关。

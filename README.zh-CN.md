# mobius-connect

Möbius **Agent 会话层** 的 CLI 与 stdio MCP。

在终端或 MCP 里搜索、引用、画出并交接本地编程 Agent 会话。Harness 文件保持只读。交接携带祖先引用，不生成摘要。

桌面工作台：[TardisBooo/Mobius](https://github.com/TardisBooo/Mobius)。

**数据在你这台机器上。** 索引是本地 SQLite。原始 Codex JSONL、Claude 会话和 OpenCode 的 `opencode.db` 不会被改写。

[产品页](http://8.137.87.76/mobius/?lang=zh) · [MCP](docs/MCP.md) · [桌面 README](https://github.com/TardisBooo/Mobius/blob/feat/session-lineage-references/README.zh-CN.md) · [MIT License](LICENSE)

## 安装

```powershell
git clone https://github.com/TardisBooo/mobius-connect.git
git clone https://github.com/TardisBooo/Mobius.git desktop
cd mobius-connect
cargo build --release
```

依赖隔壁桌面仓的 `../desktop/crates/mydesk-core`。持久数据默认在 `D:\DataVault\Mobius`。隔离测试用 `--data-root`。

## 命令（每条一条 GIF）

动画风格与桌面成片第 12 章相同。演示数据为虚构。

### 1. `init`

![mobius-connect init](docs/gifs/01-init.gif)

```powershell
mobius-connect init
```

创建 SQLite 索引和金库布局。不碰任何 Harness 目录。来源必须显式添加。

### 2. `sources add` / `refresh`

![mobius-connect sources](docs/gifs/02-sources.gif)

```powershell
mobius-connect sources add opencode $env:USERPROFILE\.local\share\opencode
mobius-connect sources refresh
```

只注册你选定的一个目录。MCP 没有添加工具。OpenCode 从只读 `opencode.db` 读取。

### 3. `triage`

![mobius-connect triage](docs/gifs/03-triage.gif)

```powershell
mobius-connect triage
```

打印健康状态和**一条**建议的下一步命令。

### 4. `sessions search`

![mobius-connect sessions search](docs/gifs/04-search.gif)

```powershell
mobius-connect sessions search "flaky tests"
```

本地 BM25。复制 `@session:provider/id#mN-mM`。

### 5. `sessions read`

![mobius-connect sessions read](docs/gifs/05-read.gif)

```powershell
mobius-connect sessions read <session-id> --offset 0 --bytes 4096
```

按字节读取**原始文件**，单次最多 16384 字节。

### 6. `graph show`

![mobius-connect graph show](docs/gifs/06-graph.gif)

```powershell
mobius-connect graph show <session-id>
```

`content_mode: references_only`。交接链是 DAG：A→B 再「回去」是 A→B→C。

### 7. `mome recall`

![mobius-connect mome recall](docs/gifs/07-mome.gif)

```powershell
mobius-connect mome recall "why do checkout tests flake"
```

最多三个会话、约 1200 token。不注入任何提示。

### 8. `semantic enable`

![mobius-connect semantic enable](docs/gifs/08-semantic.gif)

```powershell
mobius-connect semantic enable --model nomic-embed-text
```

默认关闭。只跟本机 Ollama 说话。Ollama 不在时回退词法。

### 9. `handoff` — 真人输入 APPROVE

![mobius-connect handoff](docs/gifs/09-handoff.gif)

```powershell
mobius-connect handoff prepare --harness claude --cwd . <session-id>
mobius-connect approvals handoff <handoff-id>
```

MCP **不能**签发这个令牌。

### 10. `mcp serve`

![mobius-connect mcp serve](docs/gifs/10-mcp.gif)

```powershell
mobius-connect mcp serve
```

元数据工具开放；读正文和启动需要真人令牌。完整表见 [docs/MCP.md](docs/MCP.md)。

## 安全

- 来源目录由人批准；MCP 不能 `sources add`。
- 原始文件只读。
- 交接信封只有身份和边，没有 transcript 正文。
- 令牌短时、一次性、限定范围。

完整英文教程见 [README.md](README.md)。

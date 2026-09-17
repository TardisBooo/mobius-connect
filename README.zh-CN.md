# mobius-connect

Möbius **Agent 会话层** 的 CLI 与 stdio MCP。

在终端或 MCP 里搜索、引用、画出并交接本地编程 Agent 会话。Harness 文件保持只读。交接携带祖先引用，不生成摘要。

桌面工作台：[TardisBooo/Mobius](https://github.com/TardisBooo/Mobius)。

**数据在你这台机器上。** 索引是本地 SQLite。原始 Codex JSONL、Claude 会话和 OpenCode 的 `opencode.db` 不会被改写。莫比乌斯不卖模型账号。

[产品页](http://8.137.87.76/mobius/?lang=zh) · [MCP](docs/MCP.md) · [桌面 README](https://github.com/TardisBooo/Mobius/blob/main/README.zh-CN.md) · [MIT License](LICENSE)

> 开发预览。原生启动审批与跨 Harness 身份绑定仍是发布门槛。兼容性按 Harness 逐项验证。

## 安装

```powershell
git clone https://github.com/TardisBooo/mobius-connect.git
cd mobius-connect
cargo build --release
```

当前依赖隔壁桌面仓的 `../desktop/crates/mydesk-core`。请把 [TardisBooo/Mobius](https://github.com/TardisBooo/Mobius) 克隆到旁边，或改 `Cargo.toml` 里的路径。持久数据默认在 `D:\DataVault\Mobius`。隔离测试用 `--data-root`。

## 快速开始

```powershell
mobius-connect init
mobius-connect sources add opencode $env:USERPROFILE\.local\share\opencode
mobius-connect sources refresh
mobius-connect triage
mobius-connect sessions search "flaky tests"
```

## 怎么拼在一起

| 你 | 用什么 |
| --- | --- |
| Windows 桌面 | [Möbius](https://github.com/TardisBooo/Mobius) |
| 终端 | `mobius-connect` |
| Agent（MCP） | `mobius-connect mcp serve` |

MCP 不能签发审批令牌、不能添加来源、不能接管 PTY。OpenClaw 历史在路线图上，本预览不索引 `~/.openclaw`。

完整工具表见 [docs/MCP.md](docs/MCP.md)。

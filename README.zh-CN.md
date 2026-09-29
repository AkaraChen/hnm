# hnm

为你的项目安装一套 Claude 与 Codex 共用的文档工作流。

[English](README.md) · **简体中文**

## 快速开始

**macOS / Linux** — 将 `cd` 路径替换为你已有项目的根目录。

```sh
curl -fsSL https://github.com/AkaraChen/hnm/releases/latest/download/install.sh | sh
export PATH="$HOME/.local/bin:$PATH"
cd /path/to/your-project
hnm init
```

**Windows · PowerShell** — 将 `cd` 路径替换为你已有项目的根目录。

```powershell
irm https://github.com/AkaraChen/hnm/releases/latest/download/install.ps1 | iex
cd C:\path\to\your-project
hnm init
```

## 环境要求与安装

使用发布版二进制无需 Rust 工具链。macOS 11+ 和 glibc 2.35+ 的 Linux 支持
x86_64 与 ARM64；Unix 安装器需要 `curl`、`tar`，以及 `sha256sum` 或 `shasum`。
该安装器不支持 Alpine/musl。Windows 需要 x64 和 PowerShell 5.1 或 7；
**运行 `hnm init` 前，请启用开发者模式或使用具备符号链接权限的账户**。

安装器校验 SHA-256 和可执行文件版本后，将程序安装到 `~/.local/bin`。
上面的 Unix `export` 仅设置当前终端的 PATH；请加入 shell 配置以供后续终端使用。
Windows 安装器设置当前会话 PATH，并打印持久配置方法。重新运行安装器即可升级 CLI，
不会修改已有项目的工作流文件。

[安装指南](docs/installation.zh-CN.md)涵盖脚本审阅、固定版本、自定义安装目录、源码构建与故障排查。

## init 之后

1. 阅读输出摘要。已有文件会被合并而非覆盖（`update`）；只有无法安全合并的路径会标记为 `skip`，请检查这些路径，确认工作流完整。
2. 打开 `AGENTS.md` 和 `docs/spec.md`，填写项目名、真实命令和共享规则。
   直接运行 `hnm init` 使用 `generic` 命令预设；默认路径 `.` 写入的项目名是回退值 `project`。
   首次运行时可用 `hnm init --name my-app --stack rust` 显式设置。
3. 在支持加载项目技能与 hook 的 agent 运行环境中打开项目。
   开发下一个功能时，让 agent 使用 `$feature-dev`：先逐项澄清需求，将结论写入
   `docs/prd/`、`docs/adr/` 和 `docs/spec.md`，再实现；准备提交时使用 `$git-commit` 审阅差异并提交。

`hnm` 写入工作流文件，不生成业务代码、不安装项目依赖，也不执行 `git init`。
生成模板保留内置语言；切换 README 语言不会改变生成文件的语言。

## 用法与配置

```sh
# 查看全部参数
hnm init --help

# 只预览，不写文件（目标目录不存在时也不会创建）
hnm init ./my-app --name my-app --stack rust --dry-run

# 初始化指定目录（不存在时创建）
hnm init ./my-app --name my-app --stack rust
```

| 参数 | 默认值 | 作用 |
| --- | --- | --- |
| `[PATH]` | `.` | 目标目录 |
| `--name`、`-n` | 路径末段；`.` / `..` 使用 `project` | 模板中的项目名 |
| `--stack`、`-s` | `generic` | 下表中的命令预设 |
| `--dry-run` | 关闭 | 打印计划，不写文件 |

通过 CLI 参数配置，无单独的 hnm 配置文件。安装器环境变量 `HNM_VERSION` 和
`HNM_INSTALL_DIR` 见[安装指南](docs/installation.zh-CN.md#安装选项)。

init 不会删除或整体替换已有内容，而是尽力合并：`AGENTS.md` 写入 hnm 管理的区块
（`<!-- hnm:begin -->` … `<!-- hnm:end -->`），之后重跑会原地刷新；普通文件 `CLAUDE.md`
会加入 `@AGENTS.md` 导入；`.claude/settings.json` 与 `.codex/hooks.json` 深度合并（保留你的值）；
真实的 `.claude/skills` 目录会在其中逐个链接 skill。其他内容不同的文件、无效 JSON、指向别处的符号链接
保持不动并标记为 `skip`，结尾给出提示。可以放心重跑。
失败时退出码非零，但可能保留此前已写入的文件；修复错误后重新运行即可。即使退出成功，也可能有路径被跳过。

### 技术栈预设

预设只填充 `AGENTS.md` 的 **Commands** 部分，不自动识别技术栈，也不安装依赖。
请根据项目实际情况修改生成的命令。

| `--stack` | Commands 内容 |
| --- | --- |
| `generic`（默认） | 待填写的说明 |
| `rust` | cargo build/test/run/clippy/fmt |
| `node` | npm test/build/lint/typecheck 脚本 |
| `bun` | bun test/build/lint/typecheck 脚本 |
| `go` | go test/build/vet 和 gofmt |
| `python` | 已配置的 pytest/ruff/mypy |

## 生成的文件

```text
project/
├── AGENTS.md                 工作流规则与技术栈命令
├── CLAUDE.md → AGENTS.md
├── docs/
│   ├── prd/                  产品需求
│   ├── adr/                  架构决策
│   └── spec.md               共享约定
├── .agents/skills/
│   ├── feature-dev/          实现前逐项质问
│   └── git-commit/           根据差异生成规范提交
├── .claude/
│   ├── skills → ../.agents/skills
│   └── settings.json         文档审阅与质问 hook
└── .codex/
    ├── hooks.json
    └── hooks/
        ├── spec_doc_review.py
        └── grilling_check.py
```

每个技能包含 `SKILL.md` 和 `agents/openai.yaml`；空的 PRD/ADR 目录包含 `.gitkeep`。
完整路径见[生成布局规范](docs/spec.md#generated-harness-layout)（英文）。

## agent 可执行的工作流

<p align="center">
  <img src="docs/assets/hnm-cover.jpg" alt="hnm — 三个文档文件夹连接成一个系统" width="960">
</p>

- **先澄清，再实现。** `$feature-dev` 每次提出一个产品或技术问题（质问），在实现前记录计划。
- **决策留在仓库。** PRD 描述要做什么，ADR 解释选择，`docs/spec.md` 维护共享约定。
- **提交前审阅文档。** `AGENTS.md`、`$git-commit` 和 Claude/Codex hook 在支持加载它们的运行环境中关联文档审阅与提交。
  英文 质问提醒在每轮用户输入时注入，仅适用于 feature-dev 质问阶段，
  尊重已确认的决策与授权范围，作为上下文提醒，不阻断操作。
  内置 hook 需要 Python 3（Codex 配置使用 `/usr/bin/python3`）；请按运行环境与操作系统调整 hook 命令。

[发布工作流](https://github.com/AkaraChen/hnm/actions/workflows/release.yml) ·
[发布版本](https://github.com/AkaraChen/hnm/releases/latest)

## 项目文档

- [安装指南](docs/installation.zh-CN.md) — 提供中英文用户指南。
- [规范](docs/spec.md) — CLI 行为、生成布局与平台约定（英文）。
- [初始化需求](docs/prd/harness-init.md) — 范围与预期工作流（中文）。
- [实现决策](docs/adr/clap-minijinja-embedded-templates.md) — CLI 与模板设计（中文）。
- [发布流程](docs/releasing.md) — 维护者指南（英文）。

## 开发

使用支持 edition 2024 的 Rust 工具链：

```sh
git clone https://github.com/AkaraChen/hnm.git
cd hnm
cargo test
cargo run -- init --help
```

开发流程见 [AGENTS.md](AGENTS.md)。

<sub>封面为 AI 生成的概念插画。<a href="docs/readme-design.md">设计说明</a>。</sub>

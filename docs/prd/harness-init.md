# Harness 初始化 CLI PRD

## 状态

Accepted，首版实现中。

## 问题与背景

Agent 项目需要一套稳定的 harness：三层文档（`docs/prd`、`docs/adr`、`docs/spec.md`）、`AGENTS.md` 工作流、`feature-dev` 质问 skill、`git-commit` conventional commit skill、以及提交前文档对照门禁。手工复制容易漏文件、漂移和命名不一致。`hnm` 要把这套 harness 一键落到任意目标项目。

## 目标用户与用户故事

目标用户是维护代码仓库、并用 AI agent 做功能开发的开发者或维护者。

- 作为用户，我可以在目标目录运行 `hnm init`，自动生成完整 harness。
- 作为用户，我可以指定项目名，使生成的 `AGENTS.md` / `docs/spec.md` 使用正确名称。
- 作为用户，我可以按技术栈生成合适的 Commands 片段（如 Rust / Node）。
- 作为用户，我可以 dry-run 预览将写入的路径，而不修改磁盘。
- 作为用户，在已有配置的项目中运行 init 时，hnm 尽力把 harness 合并进去，不会删除或整体覆盖我的内容（见 `docs/prd/best-effort-merge.md`）。

## 目标

- 提供 `hnm init [PATH]` CLI，默认目标为当前目录。
- 一次生成 harness 的全部约定文件与目录（见范围）。
- 用 clap 解析参数；用嵌入式模板引擎渲染可参数化文件。
- 创建必要的符号链接（`CLAUDE.md`、`.claude/skills`）。

## 非目标

- 不扫描或改写目标项目的业务代码。
- 不提供交互式 TUI 向导、云端配置同步或多模板市场。
- 不实现 `git init`、CI 配置、依赖安装或语言脚手架。
- 首版不支持从远端拉取模板更新。

## 范围与用户流程

1. 用户在目标仓库根或任意目录执行 `hnm init [PATH]`。
2. CLI 解析 `--name`、`--stack`、`--dry-run`。
3. 未给 `--name` 时，使用传入目标路径的末段作为项目名；默认路径 `.` 或末段缺失时使用 `project`，不会解析当前目录名。
4. 渲染并写入 harness 文件；创建符号链接；打印摘要。
5. dry-run 只打印计划，不写盘。

### 必须生成的路径

| 路径 | 类型 |
|------|------|
| `AGENTS.md` | 模板渲染 |
| `CLAUDE.md` | 符号链接 → `AGENTS.md` |
| `docs/spec.md` | 模板渲染 |
| `docs/prd/.gitkeep` | 静态 |
| `docs/adr/.gitkeep` | 静态 |
| `.agents/skills/feature-dev/SKILL.md` | 静态 |
| `.agents/skills/feature-dev/agents/openai.yaml` | 静态 |
| `.agents/skills/git-commit/SKILL.md` | 静态 |
| `.agents/skills/git-commit/agents/openai.yaml` | 静态 |
| `.claude/skills` | 符号链接 → `../.agents/skills` |
| `.claude/settings.json` | 静态 |
| `.codex/hooks.json` | 静态 |
| `.codex/hooks/spec_doc_review.py` | 静态 |
| `.codex/hooks/grilling_check.py` | 静态 |

## 用户可见状态与失败行为

- 目标路径不存在：创建该目录（含父路径）后继续；无法创建则报错退出非零。
- 目标路径存在但是文件而非目录：报错退出。
- 某输出路径已存在：按 best-effort 合并规则处理（update / unchanged）；无法安全合并时跳过并标记 skip，结束时提示 skip 数量，退出码仍为 0。
- 渲染失败或写盘失败：报错，退出非零；不要求事务回滚已写文件。
- dry-run：列出将执行的 create / update / unchanged / skip / link，退出 0。

## 最小验收标准

- `hnm init <tmpdir>` 后，上表全部路径存在且链接正确。
- `AGENTS.md` 与 `docs/spec.md` 含用户指定或推断的项目名。
- `--stack rust|node|bun|go|python|generic` 影响 `AGENTS.md` 的 Commands 区。
- `--dry-run` 不创建任何文件。
- 已有文件的用户内容永远保留（仅追加或刷新 managed block、合并 JSON）。
- `cargo test` 与 `cargo build` 通过。

## 已解决的产品决策

- 产品名是 `hnm`：把标准 agent 文档 harness 自动落到项目中的工具。
- 子命令为 `init`；首版只有这一主命令（另可有默认 help）。
- 模板引擎选用 Rust 生态中 Jinja 兼容且活跃的 minijinja。
- CLI 框架为 clap derive。
- feature-dev skill、git-commit skill 与 hook 脚本作为静态资源一并打包，不依赖网络。

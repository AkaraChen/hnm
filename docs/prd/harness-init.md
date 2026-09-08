# Harness 初始化 CLI PRD

## 状态

历史首版契约。CLI 迁移后的契约见 `../adr/ctxl-generated-cli.md`；当前 hook 安装与默认文档行为见 `automatic-hooks.md` 和 `../spec.md`。

## 问题与背景

Agent 项目需要一套稳定的 harness：三层文档（`docs/prd`、`docs/adr`、`docs/spec.md`）、`AGENTS.md` 工作流、`feature-dev` 质问 skill、`git-commit` conventional commit skill、以及提交前文档对照门禁。手工复制容易漏文件、漂移和命名不一致。`hnm` 要把这套 harness 一键落到任意目标项目。

## 目标用户与用户故事

目标用户是维护代码仓库、并用 AI agent 做功能开发的开发者或维护者。

- 作为用户，我可以在目标目录运行 `hnm init`，自动生成完整 harness。
- 作为用户，我可以指定项目名，使生成的 `AGENTS.md` / `docs/spec.md` 使用正确名称。
- 作为用户，我可以按技术栈生成合适的 Commands 片段（如 Rust / Node）。
- 作为用户，我可以 dry-run 预览将写入的路径，而不修改磁盘。
- 作为用户，默认不会覆盖已有文件；需要时显式 `--force`。

## 目标

- 提供 `hnm init [PATH]` CLI，默认目标为当前目录。
- 一次生成 harness 的全部约定文件与目录（见范围）。
- 用 cobra 解析参数；用嵌入式模板渲染可参数化文件。
- 创建必要的符号链接（`CLAUDE.md`、`.claude/skills`）。

## 非目标

- 不扫描或改写目标项目的业务代码。
- 不提供交互式 TUI 向导、云端配置同步或多模板市场。
- 不实现 `git init`、CI 配置、依赖安装或语言脚手架。
- 首版不支持从远端拉取模板更新。

## 范围与用户流程

1. 用户在目标仓库根或任意目录执行 `hnm init [PATH]`。
2. CLI 解析 `--name`、`--stack`、`--force`、`--dry-run`。
3. 未给 `--name` 时，使用目标目录名作为项目名。
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

## 用户可见状态与失败行为

- 目标路径不存在：创建该目录（含父路径）后继续；无法创建则报错退出非零。
- 目标路径存在但是文件而非目录：报错退出。
- 某输出路径已存在且未 `--force`：跳过该路径并在摘要中标记 skipped；其余路径继续。若全部应写路径均因冲突跳过且无新建，退出码仍可为 0，但摘要标明无变更。
- 渲染失败或写盘失败：报错，退出非零；不要求事务回滚已写文件。
- dry-run：列出 would-create / would-skip，退出 0。

## 最小验收标准

- `hnm init <tmpdir>` 后，上表全部路径存在且链接正确。
- `AGENTS.md` 与 `docs/spec.md` 含用户指定或推断的项目名。
- `--stack rust|node|bun|go|python|generic` 影响 `AGENTS.md` 的 Commands 区。
- `--dry-run` 不创建任何文件。
- 无 `--force` 时不覆盖已有非链接文件内容。
- `go test ./...` 与 `go build ./cmd/hnm` 通过。

## 已解决的产品决策

- 产品名是 `hnm`：把标准 agent 文档 harness 自动落到项目中的工具。
- 子命令为 `init`；首版只有这一主命令（另可有默认 help）。
- 模板用嵌入资源加占位符替换。
- CLI 框架为 cobra。
- feature-dev skill、git-commit skill 与 hook 脚本作为静态资源一并打包，不依赖网络。

# Go `hnm init` 与 Rust 对照

## 状态

对照完成。Go 实现与 Rust `588caae` 在产品表面上对齐。

## 对照范围

对 `generic`、`rust`、`node`、`bun`、`go`、`python` 六个 stack，分别跑空目录 `init`、第二次 `init`、`--force`、`--dry-run`。项目名固定为 `sample`。

## 必须一致的项

| 项 | 结果 |
|----|------|
| 路径集合 | 一致 |
| `CLAUDE.md` -> `AGENTS.md` | 一致 |
| `.claude/skills` -> `../.agents/skills` | 一致（字面链接文本） |
| `AGENTS.md` / `docs/spec.md` 全部标题 | 一致 |
| 各 stack 的 Commands 片段 | 一致 |
| 空目录首次写入的 `AGENTS.md` 与 `docs/spec.md` | 六个 stack 均与 Rust **字节级相同** |
| 静态 skill / hooks / settings | 与嵌入模板相同 |
| 第二次 `init` 不改已有文件 | 报告 `skip` / `skip-link` |
| `--dry-run` 不写盘 | 只打印 `create` / `link` |
| `--force` 覆盖错误或过期文件 | 报告 `overwrite` / `relink` |

## 可接受差异

- 模板引擎：Rust 用 minijinja，Go 用 `strings.ReplaceAll` 替换 `{{ project_name }}` / `{{ commands }}` / `{{ stack }}`。本次对照没有换行或空白差异。
- CLI 框架：Rust 用 clap，Go 用 cobra。命令、短选项、默认 stack=`generic` 相同；help 排版不同。
- 契约路径：`CLAUDE.md` 的 path/target 来自本仓库 `schema/harness.json`，经 ctxl schema 解析。`.claude/skills` 仍由 hnm 按字面 target 写入，因为 ctxl 会把相对 target 接到项目根上。
- `AGENTS.md` 与 `docs/spec.md` 由 hnm 整文件替换写出，不走 ctxl `WriteSingular`，避免 YAML frontmatter。
- Rust crate 保留，默认安装说明改为 `go install`，cargo 仍可用于对照。

## 未发现的失败项

没有缺路径、断链、缺整节、二次 init 改写、或 dry-run 落盘。

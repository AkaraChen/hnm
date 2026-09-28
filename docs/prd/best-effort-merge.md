# Best-effort merge into existing projects PRD

## 状态

Accepted，v0.2.0 实现。

## 问题

旧版 `hnm init` 对任何已存在路径直接 skip，只有 `--force` 整体覆盖（包括删除真实的 `.claude/skills` 目录）。在已有 `CLAUDE.md`、`.claude/settings.json` 或 skills 目录的项目里，结果是静默的"半套 harness"：规则、skill 或提交门禁未接入，退出码却为 0。

## 目标

- 默认尽可能把 harness 写进已有项目，不需要用户手工合并。
- 永不删除或整体覆盖用户内容。
- 重复运行幂等，并可刷新 hnm 自己管理的内容。
- 无法安全合并的路径明确报告。

## 非目标

- 不提供 `--force`（已移除）。需要替换时由用户手动删除后重跑。
- 不合并用户修改过的 skill / hook 脚本 / `docs/spec.md`；不同即 skip。
- 不修复无效 JSON。

## 行为

见 `docs/spec.md` 的 Generation rules。摘要标签：`create`、`update`、`unchanged`、`skip`、`link`、`skip-link`；存在 skip 时末尾打印提示行。

## 验收标准

- 已有 `AGENTS.md` 保留原内容并追加 managed block；重跑为 `unchanged`，模板变化时原地刷新 block。
- 普通文件 `CLAUDE.md` 获得 `@AGENTS.md` 导入。
- 已有 `.claude/settings.json` 保留原键值并加入 hook；无效 JSON 原样保留并报 skip。
- 真实 `.claude/skills` 目录保留原内容并获得每个 harness skill 的链接。
- 对已初始化目录重跑不写任何文件。
- `--dry-run` 报告相同动作但不写盘。

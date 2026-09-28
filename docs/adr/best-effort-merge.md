# Best-effort merge replaces skip/--force

## 背景

skip/`--force` 二选一：skip 会静默产生不完整 harness，`--force` 会销毁用户配置（覆盖 settings.json、`remove_dir_all` 真实 skills 目录）。

## 决策

移除 `--force`，init 只有一种模式：按路径的 best-effort 合并。策略声明在 `plan::harness_plan` 中（`Update` / `LinkFallback`），合并算法在 `merge` 模块中为纯函数。

- Markdown：`<!-- hnm:begin -->` / `<!-- hnm:end -->` managed block，追加或原地替换。新建的 `AGENTS.md` 本身就包在 block 中，否则用户编辑后重跑会追加第二份模板。标记不成对（缺 end、重复、顺序颠倒）视为冲突，避免每次运行都追加。
- 无 block 的旧版 `AGENTS.md`：与原始模板完全一致时收编进 block；含 harness 规则签名但有改动时 skip，不重复追加。
- `CLAUDE.md` 被占用：使用 Claude Code 的 `@AGENTS.md` 导入语法，而非复制内容，避免两份规则漂移。
- JSON：对象按键递归合并，数组追加缺失项（结构相等去重），标量以用户值为准；需要对象/数组的位置类型不符（如 `null`）视为冲突，而不是静默报 unchanged；用 `serde_json` 的 `preserve_order` 保持用户键顺序。语义不变时不重写文件。
- `.claude/skills` 为真实目录：在其中逐个链接 harness skill。
- 其余差异：保持不动，报 `skip`，末尾汇总提示。

## 备选

- 保留 `--force` 作为逃生口：会继续提供破坏性路径，用户要求默认且唯一为 best-effort。
- 对 skill / hook 脚本做三方合并：需要记录上次安装版本，成本高，暂不做。
- 冲突时非零退出：会让已完成大部分安装的运行显示为失败，改为提示行。

## 代价与失败边界

- 合并后的 JSON 会被重新格式化（仅在有语义变化时）。
- 用户若在 managed block 内手工编辑，会在下次运行时被覆盖。
- 模板升级不会自动更新已被用户修改过的 skill 文件。

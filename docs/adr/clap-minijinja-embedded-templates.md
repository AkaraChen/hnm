# ADR：clap CLI、minijinja 与编译期嵌入模板

## 状态

Accepted。

## 背景与驱动因素

`hnm` 需要稳定、可重复地把 harness 文件集写入目标目录。模板中仅有少量变量（项目名、技术栈命令块），但 skills（`feature-dev`、`git-commit`）与 commit hook 是大段静态正文，必须与工具版本一起分发，不能依赖用户仓库里是否已有副本。

## 决策

- 使用 **clap**（derive API）定义 `init` 子命令与 flags。
- 使用 **minijinja** 渲染参数化模板。选择理由：Jinja2 语法在通用模板引擎中使用面最广；minijinja 是 Rust 侧活跃、依赖面干净的实现。
- 所有模板与静态资源放在仓库 `templates/`，经 `include_str!` 在编译期嵌入二进制，运行时零外部文件依赖。
- 生成计划用显式文件清单（path + 渲染/静态/符号链接），不用运行时扫目录猜测。
- 领域错误用 `thiserror`；`main` 边界用 `anyhow` 打印并映射退出码。
- 模块按职责拆分：`cli`、`error`、`stack`、`plan`、`render`、`init`；`main` 只做解析与调用。

## 考虑过的替代方案

### tera / handlebars

未作为首选。二者可用，但 minijinja 更贴近“当前最广为人知的 Jinja 语法”，且 API 足够小。

### 运行时读取可执行文件旁的 templates 目录

未采用。安装路径不稳定，也容易出现版本与资源不一致。

### 交互式询问项目名与技术栈

首版未做。参数与目录名推断已覆盖主路径；交互可后续加。

## 权衡与后果

- 嵌入模板使发版简单，但改模板必须重新编译。
- 显式清单避免漏文件，新增 harness 路径时要改清单与测试。
- 非 force 时按路径跳过，可能产生“半套 harness”；摘要必须写清 skipped，由用户决定是否 `--force`。

## 边界与失败行为

- 模板变量只信任 CLI 与本地路径推断，不读环境密钥。
- 符号链接在目标已存在且为正确链接时可视为已满足；错误类型链接在 `--force` 下替换。
- Windows 上符号链接可能失败：错误需可读；不在首版承诺 Windows 特权策略。

## 迁移与验证影响

- 增加 init 到临时目录的集成测试，断言路径集合与关键文件内容。
- dry-run 与 skip/force 行为有单测或集成测覆盖。

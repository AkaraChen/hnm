# 一行安装与发布 PRD

## 状态与范围

Accepted，依据 KIT-931 的用户要求。面向不希望安装 Rust 工具链的 hnm 用户。
提供 macOS Intel/Apple Silicon、Linux x86_64/ARM64 和 Windows x64 安装包。
依据后续用户要求，Unix 入口简化为 curl 管道；Windows 提供 PowerShell 一行命令。安装不需管理员权限；init 仍需开发者模式或符号链接权限。

## 用户流程与验收

- README 一行命令从 GitHub Release 获取安装器和匹配平台的二进制，无需克隆或编译。
- 默认安装最新稳定版到 `~/.local/bin`；允许指定版本与绝对安装目录，重复执行完成升级。
- 校验下载完整性并实际执行版本检查后才替换已有程序；下载、版本、平台、校验或运行失败必须非零退出，保留旧程序。
- 不使用 sudo，不修改 shell 配置或项目 harness；需要时提示 PATH。
- 五个平台原生 CI 执行安装与 init smoke test。首次发布需真实资产下载验证。
- 后续推送与 Cargo 版本相同的 vX.Y.Z tag 自动构建发布；PR 也验证全部平台，但无发布权限。

Windows 默认安装到用户目录下的 .local/bin，不修改持久 PATH；当前 PowerShell 会话可直接使用 hnm，后续会话按提示配置 PATH。PowerShell 5.1 与 7 均需验证；校验和版本失败保留旧程序。

## 非目标

包管理器仓库、Windows ARM64 原生产物、自动后台升级，以及修改 init 的行为。

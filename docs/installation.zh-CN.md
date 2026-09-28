# 安装

[← 返回 README](../README.zh-CN.md) · [English](installation.md) · **简体中文**

安装最新发布版，无需 Rust 工具链：

```sh
curl -fsSL https://github.com/AkaraChen/hnm/releases/latest/download/install.sh | sh
```

支持 macOS 11+（Intel / Apple Silicon）和 glibc 2.35+ 的 Linux
（x86_64 / ARM64，例如 Ubuntu 22.04+）。Unix 安装器不支持 Alpine/musl。
CI 在 Ubuntu 22.04、macOS 14 ARM 和 macOS 15 Intel 上测试；最低 macOS
版本是构建目标，不代表该系统版本已实测。

脚本校验 SHA-256 和 `hnm --version`，然后安装到 `~/.local/bin`。
必要时将 `export PATH="$HOME/.local/bin:$PATH"` 加入 shell 配置。
重复运行即可升级，不修改已有项目的工作流文件。
需要 `curl`、`tar`，以及 `sha256sum` 或 `shasum`；不使用 sudo。

## Windows（PowerShell）

```powershell
irm https://github.com/AkaraChen/hnm/releases/latest/download/install.ps1 | iex
```

支持 Windows x64 和 PowerShell 5.1 或 7。安装无需 Rust、Git Bash 或管理员权限。
替换已有程序前会校验 SHA-256 和二进制版本。默认安装到 `$HOME/.local/bin`，
加入当前会话 PATH，并打印后续终端的配置方法。支持 `HNM_VERSION` 和
`HNM_INSTALL_DIR` 环境变量；不提供 Windows ARM64 原生安装包。

`hnm init` 会创建符号链接：请启用 Windows 开发者模式或使用具备符号链接权限的账户。
安装器不会修改系统策略。

## 安装选项

如需先审阅脚本、指定版本或自定义安装目录：

```sh
curl -fSL https://github.com/AkaraChen/hnm/releases/latest/download/install.sh -o install.sh
# 执行前审阅 install.sh。
HNM_VERSION=v0.2.0 HNM_INSTALL_DIR="$HOME/.local/bin" sh install.sh
hnm --version
```

`HNM_VERSION` 接受 `latest`（默认）或稳定标签 `vX.Y.Z`。
`HNM_INSTALL_DIR` 必须为绝对路径；Windows 必须为本地盘符路径。
PowerShell 示例：

```powershell
$env:HNM_VERSION = 'v0.2.0'
$env:HNM_INSTALL_DIR = "$HOME\.local\bin"
irm https://github.com/AkaraChen/hnm/releases/latest/download/install.ps1 | iex
hnm --version
```

版本或资源不存在、平台不支持、校验失败、二进制不兼容时，安装器失败并保留原可执行文件。
校验和用于检测损坏；下载信任仓库与 GitHub HTTPS。
Unix 管道可能掩盖安装脚本首次下载失败的退出码；需要严格失败检测的自动化，应像上例一样先下载成功再执行。

## 源码构建

其他系统可安装支持 edition 2024 的 Rust 工具链，克隆仓库后编译：

```sh
git clone https://github.com/AkaraChen/hnm.git
cd hnm
cargo install --path . --locked
```

确保 Cargo 的 bin 目录（通常为 `~/.cargo/bin`）在 PATH 中。
构建成功后，进入你自己的项目根目录运行 `hnm init`。

## 故障排查

- **找不到 `hnm`：** 检查安装是否成功，确认安装目录在 PATH 中；Unix 可运行
  `export PATH="$HOME/.local/bin:$PATH"`，Windows 按安装器提示配置。
- **Windows 符号链接创建失败：** 启用开发者模式或获得符号链接权限，再运行 init。
- **修改名称或技术栈后文件未变化：** 重跑会刷新 `AGENTS.md` 中的管理区块，但已有的
  `docs/spec.md` 不会被覆盖。先用 `--dry-run` 检查计划，再自行修改或删除该文件后重跑。
- **hook 不运行：** 确认 agent 支持并加载项目配置；审阅 hook 需要 Python 3，
  内置 Codex 路径是 `/usr/bin/python3`，请按系统调整。

完整初始化示例、参数与下一步操作见[中文 README](../README.zh-CN.md)。

# ADR：GitHub Release 二进制与 POSIX 安装器

## 状态

Accepted。

## 决策与原因

利用仓库现有 Cargo 版本作为唯一版本源，原生 GitHub runners 构建四个平台，避免交叉编译工具链。
Linux 使用 Ubuntu 22.04 GNU 目标（glibc 2.35+），macOS 部署目标 11.0；CI 在当前 runner 系统上验证，最低系统版本不声称已实测。
每个平台发布 `hnm-vX.Y.Z-<rust-target>.tar.gz` 和同名 `.sha256`，包内仅 hnm。安装器也作为 Release 资产，避免依赖默认分支上尚未合并的脚本。

POSIX sh 安装器只依赖 curl、tar、常见 Unix 工具及 sha256sum 或 shasum。
latest 先解析为固定 tag，随后所有下载使用该 tag；SHA-256 防止传输损坏，不替代 GitHub HTTPS 信任。
解包后执行 --version 并匹配 tag，成功后在目标目录内原子重命名；不跟随已有 hnm 链接写入，不修改项目配置。

## 替代方案与权衡

cargo install 要求工具链，不满足一行下载二进制的目标。包管理器渠道增加维护成本，暂缓。
GNU 产物使原生测试简单，但不支持 Alpine/musl 与旧 glibc；这些系统保留源码构建途径。
安装器不承担首次下载自身的签名验证；用户可先下载审阅。

## 发布边界

只有 tag 构建的发布 job 有 contents: write。四平台测试和打包全部成功后，先上传到 draft，再公开发布。
已公开版本不可覆盖；失败 draft 可以重跑补齐。tag 必须与 Cargo.toml 的稳定版本完全相同。
初始版本沿用已有 0.1.0；首次发布可在本 PR 的提交上打 tag，PR 合并仍独立接受审查。

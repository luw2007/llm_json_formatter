# Homebrew 分发说明

当前目标：Homebrew 直接下载 GitHub Releases 的预编译二进制包安装 `jf`，避免从源码编译导致用户需要安装 Rust。

## 文件说明

- `jf.rb`：formula 模板，需要复制到 tap 仓库的 `Formula/jf.rb` 并填入正确的 sha256

## 更新流程

1) 确保 GitHub Releases 已包含对应平台资产：

- `jf-vX.Y.Z-aarch64-apple-darwin.tar.gz`
- `jf-vX.Y.Z-x86_64-apple-darwin.tar.gz`
- `jf-vX.Y.Z-aarch64-unknown-linux-gnu.tar.gz`
- `jf-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz`

2) 计算 sha256：

```bash
shasum -a 256 jf-vX.Y.Z-*.tar.gz
```

3) 在 tap 仓库更新 `Formula/jf.rb`：

- `version` 改为 `X.Y.Z`
- 分别替换 macOS arm64/x86_64、Linux arm64/x86_64 的 `sha256`

4) 验证：

```bash
brew uninstall jf || true
brew untap luw2007/tap || true
brew tap luw2007/tap
brew install jf
jf --help
```

# 发布与分发（GitHub Releases + Homebrew）

本文档描述 `jf` 的手动发布流程：先在 GitHub Releases 上传预编译二进制包，再通过 Homebrew 分发。

## 目标产物

Release 资产（assets）建议包含：

- `jf-vX.Y.Z-aarch64-apple-darwin.tar.gz`
- `jf-vX.Y.Z-x86_64-apple-darwin.tar.gz`
- `jf-vX.Y.Z-aarch64-unknown-linux-gnu.tar.gz`
- `jf-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz`
- `jf-vX.Y.Z-x86_64-pc-windows-msvc.zip`
- `SHA256SUMS`

其中 tar.gz/zip 内应直接包含可执行文件 `jf`（Windows 为 `jf.exe`）。

## 发布步骤（手动）

### 1. 更新版本与变更记录

- 修改 [Cargo.toml](file:///Users/luwei.will/ai/llm_json_formatter/Cargo.toml) 的 `package.version`
- 更新 [CHANGELOG.md](file:///Users/luwei.will/ai/llm_json_formatter/CHANGELOG.md)

### 2. 本地验证

```bash
cargo fmt
cargo test
```

### 3. 构建与打包

推荐使用脚本统一打包（构建目标需要你本地已配置好对应 toolchain/交叉编译环境）：

```bash
chmod +x ./scripts/package_release.sh
./scripts/package_release.sh <version>
# 或者省略 version，自动读取 Cargo.toml 的版本号
# ./scripts/package_release.sh
ls -lah dist
```

脚本会生成 `dist/` 目录下的压缩包与 `SHA256SUMS`。

### 4. 创建 Git tag 与 GitHub Release

建议使用 `vX.Y.Z` 作为 tag，例如：

- tag：`v<version>`
- Release 标题：`<version>`

在 GitHub Release 页面上传 `dist/` 下所有产物。

### 5. 更新 Homebrew formula（在 tap 仓库）

Homebrew formula 应改为直接下载 GitHub Releases 资产（而不是 `depends_on "rust" => :build` 从源码编译）。

1) 在 tap 仓库中更新 `Formula/jf.rb`：

- `version` 改为本次版本
- 为 macOS arm64/x86_64、Linux arm64/x86_64 分别填写 url 与 sha256

2) 计算 sha256：

```bash
shasum -a 256 dist/jf-v<version>-*.tar.gz
```

3) 验证：

```bash
brew uninstall jf || true
brew untap luw2007/tap || true
brew tap luw2007/tap
brew install jf
jf --help
```

## macOS 常见问题

若用户下载后遇到“无法打开”或“已损坏”，通常是 Gatekeeper 的隔离属性导致。可以提示用户在解压目录执行：

```bash
xattr -dr com.apple.quarantine jf
```

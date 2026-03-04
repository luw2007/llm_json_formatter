# GitHub Actions 自动化发布配置指南

本文档说明如何配置 GitHub Actions 实现自动化构建、发布和 Homebrew tap 更新。

## Workflows 概览

### 1. `release.yml` - 构建与发布
- **触发条件**：推送 `v*` 标签时自动触发
- **功能**：
  - 构建 4 个平台的二进制文件：
    - macOS ARM64 (aarch64-apple-darwin)
    - macOS x86_64 (x86_64-apple-darwin)
    - Linux ARM64 (aarch64-unknown-linux-gnu)
    - Linux x86_64 (x86_64-unknown-linux-gnu)
  - 为每个平台生成 tar.gz 压缩包
  - 计算所有平台的 SHA256 校验和
  - 自动创建 GitHub Release 并上传所有资产

### 2. `update-tap.yml` - 更新 Homebrew Tap
- **触发条件**：
  - Release 发布后自动触发
  - 可手动触发（需要指定版本号）
- **功能**：
  - 从 GitHub Release 下载并计算所有平台的 SHA256
  - 生成完整的 Homebrew formula 文件
  - 自动提交并推送到 `luw2007/homebrew-tap` 仓库

## 配置步骤

### 1. 创建 Personal Access Token (PAT)

1. 访问 GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
2. 点击 "Generate new token (classic)"
3. 配置 token：
   - **Note**: `tap-push-token` (或任意描述性名称)
   - **Expiration**: 90 days 或 No expiration（根据需要）
   - **Scopes**: 勾选以下权限
     - `repo` (Full control of private repositories)
     - 如果 tap 仓库是公开的，只需 `public_repo` 即可
4. 点击 "Generate token" 并**立即复制** token（只显示一次）

### 2. 添加 Secret 到源码仓库

1. 访问 `https://github.com/luw2007/llm_json_formatter/settings/secrets/actions`
2. 点击 "New repository secret"
3. 配置：
   - **Name**: `TAP_PUSH_TOKEN`
   - **Secret**: 粘贴上面生成的 PAT
4. 点击 "Add secret"

### 3. 验证 Tap 仓库权限

确保生成的 PAT 对 `luw2007/homebrew-tap` 仓库有写入权限：
- 如果是个人仓库，PAT 拥有者必须是仓库 owner 或 collaborator
- 如果是组织仓库，需要授予相应的组织权限

## 使用方法

### 自动发布流程

1. **更新版本号**
   ```bash
   # 在 Cargo.toml 中修改 version = "0.1.3"
   vim Cargo.toml
   ```

2. **更新 CHANGELOG**
   ```bash
   # 补充新版本的变更说明
   vim CHANGELOG.md
   ```

3. **提交更改**
   ```bash
   git add Cargo.toml CHANGELOG.md
   git commit -m "chore: bump version to 0.1.3"
   git push
   ```

4. **创建并推送 tag**
   ```bash
   git tag -a v0.1.3 -m "Release v0.1.3"
   git push origin v0.1.3
   ```

5. **自动化流程开始**
   - GitHub Actions 自动开始构建 4 个平台的二进制
   - 构建完成后自动创建 GitHub Release
   - Release 创建后自动触发 Homebrew tap 更新
   - 几分钟后，用户可通过 `brew upgrade luw2007/tap/jf` 升级

### 手动触发发布（可选）

如果需要重新发布某个版本的 Homebrew formula：

1. 访问 Actions → Update Homebrew Tap → Run workflow
2. 输入版本号（如 `0.1.3`）
3. 点击 "Run workflow"

## 查看构建状态

1. 访问 `https://github.com/luw2007/llm_json_formatter/actions`
2. 查看最近的 workflow 运行状态
3. 如有失败，点击进入查看详细日志

## 故障排查

### TAP_PUSH_TOKEN 权限不足

**错误信息**：
```
remote: Permission to luw2007/homebrew-tap.git denied
```

**解决方法**：
1. 检查 PAT 是否有 `repo` 或 `public_repo` 权限
2. 检查 PAT 拥有者是否是 tap 仓库的 owner/collaborator
3. 重新生成 PAT 并更新 Secret

### 跨平台构建失败

**Linux ARM64 构建错误**：
- 确保 workflow 中已安装 `gcc-aarch64-linux-gnu`
- 检查 `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER` 环境变量设置

**macOS x86_64 构建错误**：
- GitHub Actions 的 `macos-latest` 可能是 ARM64 runner
- workflow 已配置正确的交叉编译 target

### SHA256 校验失败

**症状**：brew 安装时报错 SHA256 不匹配

**原因**：Release 资产被重新上传，但 tap 未更新

**解决方法**：
1. 手动触发 `update-tap.yml` workflow
2. 或删除 tag 和 Release 后重新发布：
   ```bash
   git tag -d v0.1.3
   git push origin :refs/tags/v0.1.3
   gh release delete v0.1.3 -y
   git push origin v0.1.3
   ```

## 高级配置

### 自定义 Tap 仓库

如需使用不同的 tap 仓库，在源码仓库设置 Variables：

1. 访问 Settings → Secrets and variables → Actions → Variables
2. 添加：
   - `TAP_REPO`: `your-username/homebrew-your-tap`
   - `TAP_DEFAULT_BRANCH`: `main` (或其他默认分支)

### 添加更多平台

编辑 `.github/workflows/release.yml`，在 matrix 中添加：

```yaml
- os: windows-latest
  target: x86_64-pc-windows-msvc
```

并相应更新 `update-tap.yml` 中的 SHA256 计算逻辑。

## 本地脚本已弃用

以下本地脚本不再需要（保留作为备用）：
- `scripts/package_release.sh` - 本地打包（被 GitHub Actions 替代）
- `update_tap_formula.sh` - 本地更新 tap（被 GitHub Actions 替代）

所有发布操作现在通过 GitHub Actions 自动完成，确保跨平台一致性。

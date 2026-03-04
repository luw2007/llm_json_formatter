# 快速发布指南

## 首次设置（仅需一次）

1. 创建 GitHub Personal Access Token：https://github.com/settings/tokens
   - 权限：`repo` 或 `public_repo`
   
2. 添加到仓库 Secrets：https://github.com/luw2007/llm_json_formatter/settings/secrets/actions
   - Secret 名称：`TAP_PUSH_TOKEN`
   - Secret 值：粘贴上面的 token

## 日常发布流程（3 步）

```bash
# 1. 更新版本号和 CHANGELOG
vim Cargo.toml CHANGELOG.md
git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to 0.1.3"
git push

# 2. 创建并推送 tag
git tag -a v0.1.3 -m "Release v0.1.3"
git push origin v0.1.3

# 3. 等待 GitHub Actions 完成（约 5-10 分钟）
# 查看进度：https://github.com/luw2007/llm_json_formatter/actions
```

## 完成后自动发生的事情

✅ 构建 4 个平台的二进制（macOS arm64/x86_64, Linux arm64/x86_64）
✅ 创建 GitHub Release 并上传所有文件
✅ 自动更新 Homebrew tap (luw2007/homebrew-tap)
✅ 用户可以 `brew upgrade luw2007/tap/jf` 升级

## 详细文档

参见 [docs/github-actions-setup.md](docs/github-actions-setup.md) 获取完整配置说明和故障排查。

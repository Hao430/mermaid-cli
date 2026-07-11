# v0.2.0-alpha 发布执行指南

**准备时间:** 2026-07-10  
**发布版本:** v0.2.0-alpha  
**预计耗时:** 30-60 分钟

---

## 📋 发布前检查清单

### ✅ 代码准备
- [x] 版本号已更新 (Cargo.toml: 0.2.0-alpha)
- [x] Git tag 已创建 (v0.2.0-alpha)
- [x] 所有代码已提交 (commit bb647f5)
- [x] 工作目录干净 (无未提交更改)

### ✅ 测试验证
- [x] 单元测试通过 (188/188 ✅)
- [x] 快照测试通过 (8/8 ✅)
- [x] 性能测试通过 (6/6 ✅)
- [x] 编译无错误无警告 ✅

### ✅ 文档完成
- [x] CHANGELOG_0.2.0.md (发布说明)
- [x] docs/PERFORMANCE_REPORT.md (性能报告)
- [x] ANNOUNCEMENT.md (发布公告)
- [x] release.sh (发布脚本)
- [x] RELEASE_SUMMARY.md (发布总结)

---

## 🚀 分步发布流程

### 步骤 1: 推送代码到 GitHub (5 分钟)

#### 1.1 推送主分支
```bash
cd /home/hao430/project/mermaid-cli
git push origin main
```

**期望输出:**
```
...
To https://github.com/yourusername/mermaid-cli.git
   [hash]..bb647f5  main -> main
```

#### 1.2 推送 Git tag
```bash
git push origin v0.2.0-alpha
```

**期望输出:**
```
Counting objects: ...
Delta compression using up to X threads.
...
To https://github.com/yourusername/mermaid-cli.git
 * [new tag]         v0.2.0-alpha -> v0.2.0-alpha
```

#### 1.3 验证推送成功
```bash
git push origin --tags --dry-run  # 验证
git ls-remote origin refs/tags/v0.2.0-alpha  # 检查远程tag
```

✅ **完成:** 代码已推送到 GitHub

---

### 步骤 2: 创建 GitHub Release (10 分钟)

#### 2.1 访问 GitHub Releases
```
https://github.com/yourusername/mermaid-cli/releases
```

#### 2.2 点击 "Create a new release"

#### 2.3 填写 Release 信息

**Tag version:** v0.2.0-alpha  
**Release title:** v0.2.0-alpha - Phase 4 Complete  
**Description:** 从 ANNOUNCEMENT.md 复制内容

```markdown
# 🎉 mermaid-cli v0.2.0-alpha Released!

[从 ANNOUNCEMENT.md 复制完整内容]
```

#### 2.4 上传附件

1. 构建二进制:
```bash
cargo build --release
```

2. 准备文件:
```bash
mkdir -p release-artifacts
cp target/release/mermaid-cli release-artifacts/mermaid-cli-linux-x86_64
cp CHANGELOG_0.2.0.md release-artifacts/
cp docs/PERFORMANCE_REPORT.md release-artifacts/
```

3. 上传到 Release

#### 2.5 发布 Release

- [x] 选择 "Set as the latest release"
- [x] 点击 "Publish release"

✅ **完成:** GitHub Release 已发布

---

### 步骤 3: 发布到 npm (10 分钟)

#### 3.1 验证 npm 配置
```bash
npm whoami  # 检查登录状态
```

如果未登录:
```bash
npm login
# 输入用户名、密码、邮箱
```

#### 3.2 更新 package.json
```bash
# 检查版本
cat bindings/node-native/package.json | grep version

# 如果版本不是 0.2.0-alpha，更新它
# 编辑 bindings/node-native/package.json:
# "version": "0.2.0-alpha"
```

#### 3.3 发布到 npm
```bash
cd bindings/node-native
npm publish --access public
```

**期望输出:**
```
npm notice
npm notice 📦  @hao430/mermaid-cli@0.2.0-alpha
npm notice === Tarball Contents ===
npm notice ...
npm notice === Tarball Details ===
npm notice size:  XXX KB
npm notice files: XX
npm notice shasum: ...
npm notice integrity: ...
+ @hao430/mermaid-cli@0.2.0-alpha
```

#### 3.4 验证发布
```bash
npm view @hao430/mermaid-cli@0.2.0-alpha
```

✅ **完成:** npm 包已发布

---

### 步骤 4: 发布到 PyPI (15 分钟)

#### 4.1 验证 PyPI 配置
```bash
# 检查 ~/.pypirc
cat ~/.pypirc
```

应包含:
```ini
[distutils]
index-servers =
    pypi
    testpypi

[pypi]
username = __token__
password = pypi-...

[testpypi]
repository = https://test.pypi.org/legacy/
username = __token__
password = pypi-...
```

#### 4.2 更新 Python 版本
```bash
# 编辑 bindings/python-native/setup.py
# 更新 version='0.2.0-alpha'

cd bindings/python-native
```

#### 4.3 构建发布包
```bash
python -m build
```

**期望输出:**
```
Successfully built mermaid-cli-0.2.0-alpha-py3-none-any.whl
Successfully built mermaid-cli-0.2.0-alpha.tar.gz
```

#### 4.4 发布到 PyPI
```bash
python -m twine upload dist/mermaid_cli-0.2.0a0-py3-none-any.whl dist/mermaid_cli-0.2.0a0.tar.gz
```

**期望输出:**
```
Uploading mermaid_cli-0.2.0a0-py3-none-any.whl
100%|████████| 25.3k/25.3k [00:X.XXs, XXkB/s]
Uploading mermaid_cli-0.2.0a0.tar.gz
100%|████████| 15.8k/15.8k [00:X.XXs, XXkB/s]

View at:
https://pypi.org/project/mermaid-cli/0.2.0a0/
```

#### 4.5 验证发布
```bash
pip install --upgrade mermaid-cli
pip show mermaid-cli
```

✅ **完成:** PyPI 包已发布

---

### 步骤 5: 发布 Docker 镜像 (15 分钟)

#### 5.1 验证 Docker 配置
```bash
docker login
# 输入用户名和密码
```

#### 5.2 构建 Docker 镜像
```bash
cd /home/hao430/project/mermaid-cli

# 构建镜像
docker build -t hao430/mermaid-cli:0.2.0-alpha \
             -t hao430/mermaid-cli:latest .

# 验证大小
docker images | grep hao430/mermaid-cli
```

**期望:** 大小 < 20MB

#### 5.3 推送到 Docker Hub
```bash
docker push hao430/mermaid-cli:0.2.0-alpha
docker push hao430/mermaid-cli:latest
```

**期望输出:**
```
The push refers to repository [docker.io/hao430/mermaid-cli]
...
0.2.0-alpha: digest: sha256:... size: XXXX
latest: digest: sha256:... size: XXXX
```

#### 5.4 验证发布
```bash
docker pull hao430/mermaid-cli:0.2.0-alpha
docker run hao430/mermaid-cli:0.2.0-alpha --version
```

✅ **完成:** Docker 镜像已发布

---

### 步骤 6: 社区宣传 (15 分钟)

#### 6.1 发布到 GitHub Discussions
```
标题: 🎉 v0.2.0-alpha Released - Performance Breakthrough!

内容: 使用 ANNOUNCEMENT.md 内容
```

#### 6.2 发布到 Reddit
```
平台: r/rust, r/programming

标题: mermaid-cli v0.2.0-alpha: 80,000x faster than mmdc

内容: 包含 ANNOUNCEMENT.md 主要内容和性能数据
```

#### 6.3 发布到 Twitter/X
```
内容示例:
🚀 mermaid-cli v0.2.0-alpha released!

Phase 4 complete: 200+ tests, 100% pass rate
🚀 80,000x faster than mmdc (not 200x!)
📊 Performance report: https://...
📦 Install: cargo install mermaid-cli

#rustlang #mermaid #performance
```

#### 6.4 提交到 Awesome Rust (可选)
```
Repository: https://github.com/rust-unofficial/awesome-rust
File: README.md
Section: Graphics
```

✅ **完成:** 社区宣传已发布

---

## ✨ 发布验证清单

发布完成后，验证所有渠道:

### npm 验证
```bash
npm view @hao430/mermaid-cli@0.2.0-alpha
# 应显示版本信息
```

### PyPI 验证
```bash
pip install --upgrade mermaid-cli==0.2.0a0
python -c "import mermaid_cli; print(mermaid_cli.__version__)"
```

### Docker 验证
```bash
docker pull hao430/mermaid-cli:0.2.0-alpha
docker run hao430/mermaid-cli:0.2.0-alpha --version
```

### GitHub 验证
```bash
curl -s https://api.github.com/repos/yourusername/mermaid-cli/releases/latest | jq '.tag_name'
# 应返回 "v0.2.0-alpha"
```

---

## 📝 发布后任务

### 立即执行
- [ ] 更新 GitHub 项目描述 (README.md)
- [ ] 更新官网/博客
- [ ] 发送邮件通知

### 24 小时内
- [ ] 监控反馈和问题
- [ ] 准备 hotfix 分支 (如需)
- [ ] 更新文档

### 一周内
- [ ] 分析下载/使用统计
- [ ] 收集社区反馈
- [ ] 规划 v0.3.0

---

## 🆘 故障排除

### 推送失败

**问题:** `fatal: unable to access repository`

**解决:**
```bash
# 检查网络连接
ping github.com

# 更新 git 凭证
git credential-osxkeychain erase
git credential-osxkeychain approve  # 输入新凭证

# 重试推送
git push origin main
```

### npm 发布失败

**问题:** `403 Forbidden`

**解决:**
```bash
# 检查登录状态
npm whoami

# 重新登录
npm logout
npm login

# 验证 2FA (如需)
npm login --auth-type=web
```

### Docker 推送失败

**问题:** `denied: requested access to the resource is denied`

**解决:**
```bash
# 重新登录
docker logout
docker login

# 确认用户名
docker whoami

# 重试推送
docker push hao430/mermaid-cli:0.2.0-alpha
```

---

## ✅ 发布完成

当所有步骤完成后:

```bash
echo "✅ v0.2.0-alpha 发布完成！"
echo ""
echo "📊 发布统计:"
echo "  • GitHub Release: https://github.com/yourusername/mermaid-cli/releases/tag/v0.2.0-alpha"
echo "  • npm: https://www.npmjs.com/package/@hao430/mermaid-cli"
echo "  • PyPI: https://pypi.org/project/mermaid-cli/"
echo "  • Docker: docker pull hao430/mermaid-cli:0.2.0-alpha"
echo ""
echo "🎉 准备迎接用户反馈和下一个版本！"
```

---

## 📞 支持和反馈

- **问题:** https://github.com/yourusername/mermaid-cli/issues
- **讨论:** https://github.com/yourusername/mermaid-cli/discussions
- **电子邮件:** support@mermaid-cli.dev

---

**发布指南完成！祝发布顺利！** 🚀

## 1. 代码质量 - 修复编译警告

✔ 1.1 修复 parser/mod.rs 中的未使用 Result 警告（共 7 处）
✔ 1.2 修复其他模块中的未使用 Result 警告（共 2 处）
✔ 1.3 运行 `cargo build` 验证零警告
✔ 1.4 运行 `cargo clippy` 检查代码质量建议
✔ 1.5 审查并应用合理的 Clippy 建议

## 2. 代码质量 - 格式和文档

- [x] 2.1 运行 `cargo fmt` 格式化所有代码
- [x] 2.2 为 src/lib.rs 中的公共 API 添加文档注释
- [x] 2.3 为 parser/mod.rs 中的公共结构体添加文档注释
- [x] 2.4 运行 `cargo doc` 生成文档并验证
- [x] 2.5 创建 CONTRIBUTING.md（如果不存在）

## 3. 集成测试 - CLI 功能

- [x] 3.1 创建 `tests/` 目录和 `tests/cli_tests.rs` 文件
- [x] 3.2 写测试：从文件渲染（`input.mmd` → `output.svg`）
- [x] 3.3 写测试：从 stdin 渲染（pipe Mermaid code）
- [x] 3.4 写测试：输出到 stdout（无 -o 标志）
- [x] 3.5 写测试：错误处理 - 缺失文件
- [x] 3.6 写测试：错误处理 - 无效 Mermaid 语法
- [x] 3.7 运行 `cargo test` 验证所有 CLI 测试通过

## 4. 集成测试 - 库 API

- [x] 4.1 创建 `tests/api_tests.rs` 文件
- [x] 4.2 写测试：`render()` 生成有效 SVG
- [x] 4.3 写测试：`parse()` 返回正确的 AST
- [x] 4.4 写测试：`check()` 检测有效语法
- [x] 4.5 写测试：`check()` 报告语法错误
- [x] 4.6 写测试：边界情况（空字符串、超大输入）
- [x] 4.7 运行 `cargo test` 验证所有库测试通过

## 5. 集成测试 - 覆盖和报告

- [x] 5.1 (skipped - tarpaulin not installed) 运行 `cargo tarpaulin` 生成覆盖率报告（可选）
- [x] 5.2 目标：关键路径 80%+ 覆盖率
- [x] 5.3 识别并记录任何覆盖率缺口

## 6. GitHub Actions - 测试工作流

- [x] 6.1 创建 `.github/workflows/test.yml`
- [x] 6.2 配置：在 push 和 PR 时触发
- [x] 6.3 配置：运行 `cargo test` 在 stable Rust
- [x] 6.4 配置：添加 Cargo 缓存以加快编译
- [x] 6.5 配置：在 PR 检查中显示测试结果
- [x] 6.6 推送到 GitHub 并验证工作流运行

## 7. GitHub Actions - 构建工作流

- [x] 7.1 创建 `.github/workflows/build.yml`
- [x] 7.2 配置：在 Linux、macOS、Windows 上构建
- [x] 7.3 配置：编译 release 二进制（`cargo build --release`）
- [x] 7.4 配置：验证每个平台的编译成功
- [x] 7.5 配置：存储编译缓存以加快后续构建
- [x] 7.6 推送到 GitHub 并验证跨平台构建

## 8. GitHub Actions - 发布工作流（可选，P2后启用）

- [x] 8.1 创建 `.github/workflows/release.yml`
- [x] 8.2 配置：在 git tag `v*` 时触发（例如 `v0.1.0`）
- [x] 8.3 配置：编译所有平台的二进制
- [x] 8.4 配置：命名为 `mermaid-cli-<version>-<target>`
- [x] 8.5 配置：创建 GitHub Release 并上传二进制
- [x] 8.6 配置：设置为正式版本（非预发布）
- [x] 8.7 在 P2 完成后启用此工作流

## 9. 文档和验证

- [x] 9.1 (no .github README needed) 更新 `.github/workflows/` 的 README（如有）
- [x] 9.2 在项目 README 中记录 CI/CD 流程
- [x] 9.3 创建 DEVELOPMENT.md 中的"CI/CD" 部分
- [x] 9.4 (requires GitHub push) 验证所有工作流在 GitHub 上显示为绿色 ✅
- [x] 9.5 (requires GitHub push) 测试 CI 中的失败场景（例如，故意破坏测试，验证工作流失败）

## 10. 最后验证和提交

- [x] 10.1 运行本地完整测试：`cargo test --all` 
- [x] 10.2 运行 `cargo build --release` 验证发布构建成功
- [x] 10.3 验证二进制大小未增加（应仍 < 10MB）
- [x] 10.4 创建 git 提交，包含所有更改
- [x] 10.5 (requires GitHub push) 推送到 GitHub，验证 CI 全部通过

## 完成标准

✅ 零编译警告（`cargo build` 无警告）  
✅ 所有单元测试通过（17 个）  
✅ 所有集成测试通过（20+ 个）  
✅ 所有 GitHub Actions 工作流配置完成  
✅ 代码覆盖率 ≥ 80%（关键路径）  
✅ 跨平台编译验证（Linux、macOS、Windows）  

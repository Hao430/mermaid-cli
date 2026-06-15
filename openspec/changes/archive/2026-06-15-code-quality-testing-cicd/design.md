## Context

P1 阶段完成了基础框架：Lexer、Parser、AST、Renderer、SVG 生成和 Fixer 等模块，包含 17 个单元测试（100% 通过）。现有代码有 9 个编译警告（未使用 Result），且缺乏集成测试覆盖。为了为 P2（完整 Flowchart 支持）和最终的 MVP 发布（v0.1-alpha）做准备，需要：

1. 提高代码质量（消除警告）
2. 增加集成测试覆盖
3. 建立自动化 CI/CD 流程

## Goals / Non-Goals

**Goals:**
- 消除所有 9 个编译警告
- 为 CLI 和库 API 添加完整的集成测试
- 设置 GitHub Actions CI/CD，支持自动测试和跨平台编译
- 建立自动发布流程（GitHub Releases）

**Non-Goals:**
- 性能优化（留给 P4）
- 新功能实现（P2 的任务）
- 文档生成系统（后续实现）
- Cargo.io 发布（可选的后续）

## Decisions

### 1. 编译警告修复策略

**决策**：统一使用 `let _ = ...` 模式处理不使用的 Result

**理由**：
- 明确表示 Result 被意识到地忽略
- 不影响逻辑或性能
- 符合 Rust 最佳实践

**替代方案**：
- ❌ `#[allow(unused_must_use)]` — 过于宽泛，隐藏问题
- ❌ 真正处理所有错误 — 在这些情况下不合适（已在上层处理）

### 2. 集成测试框架

**决策**：使用 `tests/` 目录下的独立集成测试，同时保留单元测试

**理由**：
- Rust 标准做法，分离单元测试和集成测试
- 集成测试可测试 CLI 实际行为（文件 I/O、外部接口）
- 单元测试继续验证内部逻辑

**覆盖范围**：
- CLI 工作流：文件输入、stdin、stdout、-o 选项
- 库 API：render()、parse()、check() 函数
- 错误情况：无效输入、缺失文件、解析错误

### 3. CI/CD 架构

**决策**：GitHub Actions + 三个独立的工作流

**工作流设计**：

| 工作流 | 触发 | 任务 |
|------|------|------|
| **Test** | Push + PR | 运行所有测试 |
| **Build** | Push + PR | 编译（debug + release） |
| **Release** | Tag + Manual | 跨平台编译 + GitHub Release |

**理由**：
- 快速反馈（Test + Build 并行）
- 自动化二进制发布（无手工操作）
- 支持多平台（Linux, macOS, Windows）

**平台支持**：
- Linux: `ubuntu-latest` (x86_64)
- macOS: `macos-latest` (ARM64 + Intel)
- Windows: `windows-latest` (x86_64)

### 4. 发布流程

**决策**：使用语义版本标签（v0.1.0）触发自动发布

**工作流**：
1. 提交代码 → Pass 所有测试
2. 创建 Git 标签（`git tag v0.1.0`）
3. GitHub Actions 自动编译三个平台的二进制
4. 创建 GitHub Release 并上传二进制

**文件命名**：
- `mermaid-cli-v0.1.0-x86_64-unknown-linux-gnu`
- `mermaid-cli-v0.1.0-x86_64-apple-darwin`
- `mermaid-cli-v0.1.0-aarch64-apple-darwin` （optional）
- `mermaid-cli-v0.1.0-x86_64-pc-windows-gnu`

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|---------|
| **CI 时间过长** | 使用缓存加快编译，必要时分离工作流 |
| **跨平台编译兼容性** | 早期测试所有平台，保持最小依赖 |
| **发布后发现问题** | 保留 Release 回滚能力，使用预发布版本测试 |
| **测试覆盖不足** | 优先覆盖关键路径（渲染、解析、错误处理） |

## Migration Plan

### 第一步：修复代码质量（当前）
1. 修复 9 个编译警告
2. 验证所有测试仍通过
3. 提交 PR 审查

### 第二步：添加集成测试（随后）
1. 创建 `tests/` 目录结构
2. 写 CLI 集成测试（文件、stdin、stdout）
3. 写库 API 测试（render、parse、check）
4. 写错误情况测试
5. 运行 `cargo test` 验证覆盖

### 第三步：设置 CI/CD（最后）
1. 创建 `.github/workflows/` 目录
2. 创建 `test.yml` 工作流
3. 创建 `build.yml` 工作流
4. 创建 `release.yml` 工作流（可选暂时禁用）
5. 推送到 GitHub，验证工作流运行

### 回滚策略
- 删除 GitHub Release（如有问题）
- 重新标签旧版本（如需要）
- 创建新的修复版本

## Open Questions

1. **CI 并行策略**：Test 和 Build 是否应该串联（Build 等 Test 通过后运行）或并行？
   - **建议**：并行，独立的 Release 工作流确保质量

2. **Windows 编译**：当前代码是否支持 Windows？需要测试。
   - **建议**：代码纯 Rust，应该兼容，CI 中验证

3. **发布自动化时机**：何时启用自动发布工作流？
   - **建议**：P2 完成后，MVP 准备发布时启用

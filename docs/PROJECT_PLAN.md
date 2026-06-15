# 项目总体计划

## 项目信息

| 项目名称 | Mermaid CLI - Rust 实现 |
|---------|----------------------|
| **项目代码** | mermaid-cli-rs |
| **开始日期** | 2026-06-14 |
| **预计 MVP 发布** | 2026-08-01 （第 5-8 周） |
| **首个稳定版本** | 2026-09-15 （第 11-12 周） |
| **维护者** | Hao430 |
| **许可证** | MIT |

---

## 项目愿景

用 Rust 实现高性能、轻量级、AI 友好的 Mermaid 图表命令行工具，为开发者和 AI 系统提供：

✅ **性能优先** — 比 Node.js 版本快 10-50 倍，启动时间 <20ms  
✅ **最小部署** — 单个 <10MB 二进制，无依赖  
✅ **AI 友好** — CLI 接口支持流式输入/输出，自动错误修复  
✅ **社区驱动** — 开源发布，支持社区在 AI 应用中调用  

---

## 目标用户和场景

### 主要用户
1. **开发者** — 本地开发和文档生成
2. **AI 系统** — 通过 subprocess 或 HTTP 调用
3. **CI/CD 流程** — 自动化图表生成
4. **社区开发者** — 集成到自己的 AI 工具中

### 核心应用场景
- LLM 生成 Mermaid 代码，CLI 自动修复并渲染为图片
- 文档自动化：`.mmd` 文件 → SVG/PNG
- AI 系统实时生成架构图、流程图、序列图

---

## 核心功能定义（MVP）

### 必须有 🔴
- [x] 流程图（Flowchart）解析和渲染
- [x] 序列图（Sequence Diagram）解析和渲染
- [x] **自动错误修复** — 智能补全缺失字段、识别拼写错误
- [x] **带位置的诊断信息** — 行号、列号、建议
- [x] SVG 输出（高质量矢量图）
- [x] CLI 接口（文件输入、stdin、stdout）
- [x] 跨平台预编译二进制

### 很好有 🟡
- [ ] 简单的自动布局
- [ ] 节点样式支持
- [ ] 子图支持
- [ ] JSON 格式诊断输出
- [ ] `--width`, `--height` 配置选项

### 不需要（v0.2+） 🟢
- [ ] PNG/PDF 导出
- [ ] 主题系统
- [ ] 高级布局算法
- [ ] 其他图表类型（类图、状态图等）

---

## 开发阶段规划

### 阶段 1: P1 - 基础设施（1-2 周末）

**目标**：搭建项目框架，完成 Flowchart 基础解析

**关键产出**：
- ✅ Cargo 项目初始化
- ✅ Lexer（词法分析）
- ✅ Parser（语法解析）
- ✅ 基础 AST 定义
- ✅ 简单 SVG 生成
- ✅ 单元测试 (70% 覆盖)

**验收标准**：
```bash
$ echo 'graph TD; A[Start]-->B[End]' | mermaid-cli --stdin -o test.svg
# 生成有效的 SVG 文件
```

**时间**：1-2 周末（约 7-14 天）  
**人力**：1 人

---

### 阶段 2: P2 - MVP 发布（2-3 周末）

**目标**：完整的 Flowchart + 错误纠错，发布 v0.1-alpha

**关键产出**：
- ✅ 完整的 Flowchart 支持（所有节点形状、样式、子图）
- ✅ 自动错误修复（Fixer 模块）
- ✅ 带位置的诊断信息
- ✅ 改进的 SVG 渲染（自动布局、标签）
- ✅ 集成测试 (80% 覆盖)
- ✅ GitHub Actions CI/CD
- ✅ 跨平台编译（Linux, macOS, Windows）
- ✅ v0.1-alpha Release

**验收标准**：
```bash
$ mermaid-cli complex.mmd -o output.svg
# 支持多种节点、样式、子图

$ echo 'grpah TD' | mermaid-cli check --show-fixes
# 返回修复建议

$ time mermaid-cli large.mmd -o out.svg
# < 100ms 完成
```

**时间**：2-3 周末（约 14-21 天）  
**人力**：1 人  
**发布时间**：第 5-8 周

---

### 阶段 3: P3 - 扩展（3-4 周末）

**目标**：序列图支持，优化和完善 MVP

**关键产出**：
- ✅ 序列图（Sequence Diagram）完整支持
- ✅ 更好的自动布局
- ✅ 样式系统初步实现
- ✅ 配置选项扩展（--width, --height, --format）
- ✅ 性能优化
- ✅ v0.1.0 正式发布

**时间**：3-4 周末（约 21-28 天）  
**人力**：1 人  
**发布时间**：第 9-12 周

---

### 阶段 4: P4 - 稳定（持续）

**目标**：测试、文档、社区维护

**关键产出**：
- ✅ 测试覆盖率 > 85%
- ✅ 完整的文档（API、架构、开发指南）
- ✅ 示例代码和用例
- ✅ 性能基准测试
- ✅ 社区反馈和 Bug 修复
- ✅ 依赖管理和安全审计

**时间**：持续  
**人力**：社区驱动维护

---

## 技术栈决策

| 组件 | 选择 | 理由 |
|------|------|------|
| **语言** | Rust | 性能、安全、单二进制 |
| **解析策略** | 手动实现 | 完全控制，无依赖 |
| **渲染方式** | 手动 Lexer + Parser → SVG | 精确控制，易维护 |
| **错误恢复** | 错误恢复型解析 | 一次报告多个错误 |
| **修复策略** | 智能补全 | 更好的用户体验 |
| **输出格式** | SVG（MVP）| 轻量、可缩放 |
| **分发方式** | GitHub Releases | 简单、跨平台 |
| **CLI 框架** | clap | 标准、强大 |

---

## 架构概览

```
Input (Mermaid Code)
    ↓
CLI (main.rs)
    ├─→ Lexer (词法分析)
    │    ↓ [Token Stream]
    ├─→ Parser (语法解析)
    │    ↓ [AST + Errors]
    ├─→ Fixer (错误修复)
    │    ↓ [Suggestions]
    ├─→ Renderer (渲染)
    │    ├─→ Flowchart Renderer
    │    └─→ Sequence Renderer
    │    ↓ [Coordinates + Styles]
    └─→ SVG Generator (SVG 生成)
        ↓
Output (SVG String / File)
```

**关键特点**：
- 清晰的模块化设计
- 错误恢复和诊断
- 自动修复建议
- 支持多种图表类型

---

## 项目结构

```
mermaid-cli/
├── docs/                          # 文档（已创建）
│   ├── INDEX.md                  # 文档索引
│   ├── README.md                 # 项目概述
│   ├── QUICK_START.md            # 快速开始
│   ├── ARCHITECTURE.md           # 架构设计
│   ├── DEVELOPMENT.md            # 开发指南
│   ├── ROADMAP.md                # 路线图
│   ├── API.md                    # API 文档
│   └── PROJECT_PLAN.md           # 本文件
├── src/
│   ├── main.rs                   # CLI 入口
│   ├── lib.rs                    # 公共 API
│   ├── parser/
│   │   ├── mod.rs
│   │   ├── lexer.rs
│   │   └── ast.rs
│   ├── renderer/
│   │   ├── mod.rs
│   │   ├── flowchart.rs
│   │   └── sequence.rs
│   ├── fixer/
│   │   ├── mod.rs
│   │   ├── errors.rs
│   │   └── autocorrect.rs
│   └── svg/
│       └── mod.rs
├── tests/
│   ├── parser_tests.rs
│   └── integration_tests.rs
├── examples/
│   └── sample.mmd
├── .github/
│   └── workflows/
│       └── ci.yml
├── Cargo.toml
├── Cargo.lock
├── README.md                     # 项目根 README
└── LICENSE
```

---

## 成功指标

| 指标 | MVP 目标 | v0.1.0 目标 |
|------|---------|----------|
| **性能** | <100ms | <500ms |
| **二进制大小** | <10MB | <15MB |
| **Flowchart 支持** | 100% | 100% |
| **Sequence 支持** | N/A | 100% |
| **测试覆盖率** | 70% | 85% |
| **文档完整度** | 60% | 100% |
| **跨平台支持** | 3 个平台 | 3 个平台 |
| **错误修复准确率** | 80% | 90% |

---

## 风险评估与应对

| 风险 | 影响度 | 概率 | 应对措施 |
|------|--------|------|---------|
| 复杂的布局算法 | 中 | 中 | MVP 使用简单布局，P3+ 优化 |
| 跨平台编译问题 | 高 | 低 | 早期设置 CI/CD，使用容器测试 |
| 性能不达预期 | 高 | 低 | 使用基准测试，及时优化 |
| 依赖安全漏洞 | 中 | 中 | 定期审计，及时更新 |
| 需求变化 | 中 | 中 | 灵活的设计，社区反馈驱动 |

---

## 资源需求

### 人力
- **主开发**：1 人（周末项目）
- **代码审查**：社区贡献者
- **文档**：1 人（部分）
- **维护**：社区驱动

### 基础设施
- GitHub 仓库（免费）
- GitHub Actions CI/CD（免费）
- GitHub Releases（免费）
- 文档托管（GitHub Pages，免费）

### 开发工具
- Rust 工具链（免费）
- VS Code + rust-analyzer（免费）
- 性能分析工具（免费）

**总成本**：0 元 ✅

---

## 沟通和反馈

### 反馈渠道
1. **GitHub Issues** — 报告 Bug 和功能请求
2. **GitHub Discussions** — 讨论设计和想法
3. **Pull Requests** — 代码贡献
4. **项目 Wiki** — 社区文档和最佳实践

### 定期检查点
- **每周末** — 更新进度
- **P1 结束** — 回顾设计，调整 P2 计划
- **P2 结束** — Alpha 发布，收集反馈
- **P3 结束** — v0.1.0 发布，更新路线图

---

## 文档清单

✅ **已完成的文档**（本文档集）：
- [x] INDEX.md — 文档索引和导航
- [x] README.md — 项目概述和快速开始
- [x] QUICK_START.md — 5分钟快速上手
- [x] ARCHITECTURE.md — 架构设计和模块详解
- [x] DEVELOPMENT.md — 开发工作流和代码规范
- [x] ROADMAP.md — 详细的开发路线图
- [x] API.md — 完整的 API 参考
- [x] PROJECT_PLAN.md — 本项目总体计划

**后续需要的文档**（开发过程中补充）：
- [ ] CONTRIBUTING.md — 贡献指南
- [ ] TROUBLESHOOTING.md — 故障排除
- [ ] PERFORMANCE.md — 性能优化指南
- [ ] RELEASE_NOTES.md — 版本发布说明

---

## 立即行动

### 第一步（今天）
1. ✅ **阅读本文档** — 理解整体计划
2. ✅ **浏览 docs/ 目录** — 熟悉文档结构
3. 📝 **创建 Cargo 项目** — 初始化项目框架

### 第二步（本周末）
1. 设置 Git 和 GitHub（如果还没有）
2. 初始化 Cargo 项目和依赖
3. 实现 Lexer（词法分析器）
4. 编写单元测试

### 第三步（下周末）
1. 实现 Parser（语法解析器）
2. 定义 AST 结构
3. 编写集成测试
4. 实现基础 SVG 生成

---

## 关键联系人和资源

### 文档
- 📖 [Rust 官方书籍](https://doc.rust-lang.org/book/)
- 📖 [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- 📖 [Mermaid 官方文档](https://mermaid.js.org/)

### 工具
- 🦀 [rustup - Rust 工具链管理](https://rustup.rs/)
- 📦 [crates.io - Rust 包管理](https://crates.io/)
- ✍️ [VS Code + rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

### 社区
- 💬 [Rust 官方论坛](https://users.rust-lang.org/)
- 💬 [r/rust](https://www.reddit.com/r/rust/)
- 💬 [Rust Discord](https://discord.gg/rust-lang)

---

## 附录：常用命令

```bash
# 创建项目
cargo new mermaid-cli
cd mermaid-cli

# 开发
cargo build
cargo test
cargo run -- --help

# 发布
cargo build --release
cargo doc --open

# 代码质量
cargo clippy
cargo fmt
cargo test --all

# 性能
cargo bench
RUST_LOG=debug cargo run

# 清理
cargo clean
```

---

## 版本历史

| 版本 | 日期 | 更新 |
|------|------|------|
| v1.0 | 2026-06-14 | 初始项目计划文档集 |

---

**项目开始日期**：2026-06-14  
**下次更新**：2026-06-21 （P1 进度检查）  
**维护者**：Hao430

---

## 签署和确认

- [x] 项目愿景和目标明确
- [x] 开发阶段和时间表清晰
- [x] 技术栈和架构已定义
- [x] 成功指标已确定
- [x] 风险评估已完成
- [x] 文档完备

**项目已准备好开始实施！** 🚀

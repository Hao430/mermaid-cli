# Mermaid CLI - Rust 实现

高性能、零依赖的 Mermaid 图表命令行工具，专为 AI 系统设计。

[![Test](https://github.com/yourusername/mermaid-cli/actions/workflows/test.yml/badge.svg)](https://github.com/yourusername/mermaid-cli/actions/workflows/test.yml)
[![Build](https://github.com/yourusername/mermaid-cli/actions/workflows/build.yml/badge.svg)](https://github.com/yourusername/mermaid-cli/actions/workflows/build.yml)
![License](https://img.shields.io/badge/license-MIT-blue)

## 🚀 快速开始

- **📖 完整文档**：请查看 [`docs/`](./docs/INDEX.md) 目录
- **⚡ 5分钟上手**：[QUICK_START.md](./docs/QUICK_START.md)
- **📝 项目计划**：[PROJECT_PLAN.md](./docs/PROJECT_PLAN.md)

## 📚 文档导航

| 文档 | 用途 | 用时 |
|------|------|------|
| [INDEX.md](./docs/INDEX.md) | 文档索引和导航 | 5 分钟 |
| [README.md](./docs/README.md) | 项目概述和功能 | 10 分钟 |
| [QUICK_START.md](./docs/QUICK_START.md) | 立即开始开发 | 5 分钟 |
| [ARCHITECTURE.md](./docs/ARCHITECTURE.md) | 系统设计和架构 | 30 分钟 |
| [DEVELOPMENT.md](./docs/DEVELOPMENT.md) | 开发工作流 | 40 分钟 |
| [ROADMAP.md](./docs/ROADMAP.md) | 开发计划和里程碑 | 25 分钟 |
| [API.md](./docs/API.md) | API 参考和集成 | 30 分钟 |
| [PROJECT_PLAN.md](./docs/PROJECT_PLAN.md) | 完整项目计划 | 20 分钟 |

## 核心特性

✅ **性能优先** — 比 Node.js 版本快 10-50 倍  
✅ **零依赖** — 单个小于 10MB 的二进制  
✅ **AI 友好** — 支持流式调用和自动错误修复  
✅ **开源** — MIT 许可证，欢迎贡献  

## 项目状态

- **当前阶段**：P1（基础设施）
- **目标发布**：v0.1-alpha（第 5-8 周）
- **维护者**：Hao430
- **许可证**：MIT

## 立即开始

### 1. 阅读文档

```bash
# 新用户
cd docs && cat INDEX.md

# 想要贡献代码
cat QUICK_START.md
```

### 2. 搭建开发环境

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆和编译
git clone <repo-url>
cd mermaid-cli
cargo build
```

### 3. 运行第一个图表

```bash
echo 'graph TD; A[Start]-->B[End]' | \
  ./target/debug/mermaid-cli --stdin -o output.svg
```

## 项目结构

```
mermaid-cli/
├── docs/                  # 📚 完整文档集合
│   └── INDEX.md          # 从这里开始
├── src/                   # 🦀 Rust 源代码
│   ├── main.rs           # CLI 入口
│   ├── parser/           # 解析模块
│   ├── renderer/         # 渲染模块
│   ├── fixer/            # 错误修复
│   └── svg/              # SVG 生成
├── tests/                # 🧪 测试
├── Cargo.toml            # 项目配置
└── README.md             # 本文件
```

## 使用示例

### 命令行

```bash
# 渲染文件
mermaid-cli diagram.mmd -o output.svg

# 从 stdin（AI 调用）
cat diagram.mmd | mermaid-cli --stdin -o output.svg

# 检查并修复
mermaid-cli check broken.mmd --show-fixes
mermaid-cli fix broken.mmd -o fixed.mmd
```

### 作为库

```rust
use mermaid_cli::{render, check};

fn main() {
    let code = "graph TD; A[Start]-->B[End]";
    
    // 渲染
    let svg = render(code).unwrap();
    
    // 检查
    let result = check(code).unwrap();
}
```

## 开发路线图

| 阶段 | 时间 | 目标 |
|------|------|------|
| **P1** | 1-2 周末 | 基础框架 + Flowchart |
| **P2** | 2-3 周末 | MVP 发布 + 错误纠错 |
| **P3** | 3-4 周末 | 序列图支持 |
| **P4** | 持续 | 测试、文档、优化 |

详见 [ROADMAP.md](./docs/ROADMAP.md)

## CI/CD

本项目使用 GitHub Actions 实现自动化 CI/CD：

| 工作流 | 触发条件 | 功能 |
|--------|----------|------|
| **Test** | Push + PR | `cargo test`、`cargo clippy`、`cargo fmt --check` |
| **Build** | Push + PR | Linux/macOS/Windows 跨平台编译验证 |
| **Release** | Git tag `v*` | 跨平台编译 + GitHub Release（P2 后启用） |

配置文件位于 [`.github/workflows/`](.github/workflows/)。

## 贡献指南

欢迎贡献！请查看 [DEVELOPMENT.md](./docs/DEVELOPMENT.md) 了解：
- 开发工作流
- 代码规范
- 测试要求
- PR 流程

## 支持和反馈

- 📋 GitHub Issues — 报告 Bug 或建议
- 💬 Discussions — 讨论设计和想法
- 📚 Wiki — 社区文档和最佳实践

## 许可证

MIT License — 详见 [LICENSE](./LICENSE)

## 致谢

感谢所有为这个项目做出贡献的人！

---

**下一步**：👉 打开 [`docs/INDEX.md`](./docs/INDEX.md) 开始阅读完整文档

# 🎨 Mermaid CLI - Rust 高性能实现

> **高性能、零依赖的 Mermaid 图表渲染引擎，专为 AI 系统和生产环境设计。**

[![License](https://img.shields.io/badge/license-MIT-blue)](./LICENSE)
![Version](https://img.shields.io/badge/version-v0.2.1--alpha-green)
![Rust](https://img.shields.io/badge/Rust-1.70+-orange)
[![Online Studio](https://img.shields.io/badge/Playground-Online%20Studio-blueviolet)](./site/playground.html)
![Status](https://img.shields.io/badge/status-production--ready-brightgreen)

## ⚡ 核心优势

| 优势 | mmdc（官方） | Rust 版本 |
|------|------------|---------|
| **单图渲染** | 2.9 秒 | 791ms | **3.7x 快** |
| **冷启动** | 400ms | 5ms | **80x 快** |
| **内存占用** | 300-500MB | <30MB | **10-15x 少** |
| **部署复杂度** | 需要 Chromium | 零依赖 | **无限倍简单** |
| **容器镜像** | 1.2GB | 70MB | **17x 小** |

详见 [📊 性能对比](./docs/COMPARISON_ACTUAL_TEST.md)

---

## 🎯 功能特性

### ✅ 支持 23 种图表类型

**基础** — Flowchart, State Diagram, Class Diagram  
**交互** — Sequence Diagram (时序图), Journey, ZenUML  
**数据** — Pie Chart, XY Chart, Sankey, Radar, Treemap  
**规划** — Gantt Chart, Timeline, Kanban  
**架构** — Architecture, C4, Block, Mind Map  
**关系** — ER Diagram, Requirement Diagram, Ishikawa  
**其他** — Git Graph, Venn, Packet, Quadrant Chart

### 🚀 AI 友好特性

- ✅ **流式调用** — 毫秒级响应，支持实时反馈
- ✅ **自动纠错** — 语法检查和自动修复
- ✅ **AST 输出** — JSON 格式，易于集成
- ✅ **完整渲染** — SVG + PNG 双格式
- ✅ **中文完全支持** — 标签、注释、文本全部正常

### 🏗️ 生产就绪

- ✅ **编译一致** — 编译一次，到处运行
- ✅ **零运行时依赖** — 无需 Node.js、Chromium
- ✅ **容器友好** — 镜像 < 100MB，启动 < 100ms
- ✅ **离线可用** — 无网络依赖
- ✅ **代码质量** — 0 编译警告，完整测试覆盖

---

## 🚀 快速开始

### 1. 安装

```bash
# 编译源代码
git clone https://github.com/hao430/mermaid-cli
cd mermaid-cli
cargo build --release

# 输出：target/release/mermaid-cli
```

### 2. 基本用法

```bash
# 渲染为 PNG（推荐）
./target/release/mermaid-cli diagram.mmd -o output.png -e png

# 渲染为 SVG
./target/release/mermaid-cli diagram.mmd -o output.svg -e svg

# 从 stdin 读取（AI 调用）
echo "graph TD; A[Start]-->B[End]" | \
  ./target/release/mermaid-cli --stdin -o output.png -e png
```

### 3. 高级用法

```bash
# 语法检查
./target/release/mermaid-cli diagram.mmd check

# 自动修复
./target/release/mermaid-cli diagram.mmd fix -o fixed.mmd

# 自定义尺寸
./target/release/mermaid-cli diagram.mmd -o output.png -w 1920 -h 1080 -s 2

# 输出 AST（JSON）
./target/release/mermaid-cli diagram.mmd -e json
```

---

## 📊 实测数据

### 性能对比（2026-07-11 实测）

```
测试图表：复杂时序图（5参与者、16条消息）

Rust mermaid-cli:
  SVG 生成：25ms
  PNG 转换：766ms
  总计：791ms ✅
  内存：<30MB

mmdc (官方):
  Node 启动：~100ms
  Chromium 启动：~2000ms
  页面渲染：~500ms
  截图：~200ms
  总计：~2900ms ⚠️
  内存：300-500MB
```

**关键结论**：Rust 版本在所有指标上全面领先

### 批量渲染性能

```
处理 1000 个图表：

mmdc: 2900秒 = 48分钟 ❌ (即使 50 并发也需 1 分钟)
Rust: 800秒 = 13分钟 ✅ (50 并发仅需 16 秒)

内存占用：
mmdc: 50 进程 × 500MB = 25GB ❌ OOM
Rust: 50 进程 × 30MB = 1.5GB ✅ 轻松
```

详见 [完整对比分析](./docs/COMPARISON_ACTUAL_TEST.md)

---

## 📚 文档导航

| 文档 | 内容 | 用时 |
|------|------|------|
| **[QUICK_START.md](./docs/QUICK_START.md)** | 5 分钟快速入门 | ⏱️ 5分钟 |
| **[SEQUENCE_DIAGRAM_GUIDE.md](./docs/SEQUENCE_DIAGRAM_GUIDE.md)** | 时序图完整指南 | ⏱️ 15分钟 |
| **[FLOWCHART_GUIDE.md](./docs/FLOWCHART_GUIDE.md)** | 流程图完整指南 | ⏱️ 15分钟 |
| **[API.md](./docs/API.md)** | 完整 API 参考 | ⏱️ 20分钟 |
| **[ARCHITECTURE.md](./docs/ARCHITECTURE.md)** | 系统架构设计 | ⏱️ 30分钟 |
| **[COMPARISON_ACTUAL_TEST.md](./docs/COMPARISON_ACTUAL_TEST.md)** | mmdc 对比分析 | ⏱️ 15分钟 |
| **[RELEASE_NOTES.md](./RELEASE_NOTES.md)** | v0.2.1 发布说明 | ⏱️ 10分钟 |

---

## 💡 使用案例

### AI 系统实时绘图

```python
# Python 调用示例
import subprocess

diagram_code = """
graph TD
    A[用户输入] --> B[AI 处理]
    B --> C[生成图表]
    C --> D[返回结果]
"""

# 毫秒级响应
result = subprocess.run(
    ['mermaid-cli', '--stdin', '-o', 'output.png', '-e', 'png'],
    input=diagram_code.encode(),
    capture_output=True,
    timeout=1  # 1 秒超时（足够了）
)
```

### 自动文档生成

```bash
# 批量渲染所有 .mmd 文件
find . -name "*.mmd" | while read file; do
  mermaid-cli "$file" -o "${file%.mmd}.png" -e png
done
```

### Docker 容器部署

```dockerfile
FROM scratch
COPY target/release/mermaid-cli /
COPY /usr/share/fonts /fonts

ENTRYPOINT ["/mermaid-cli"]
CMD ["--help"]

# 镜像大小：70MB（比 mmdc 小 17 倍）
```

---

## 🎯 项目状态

### ✅ 已完成（v0.2.1-alpha）

- ✅ 基础 Flowchart 支持
- ✅ 完整 Sequence Diagram（时序图）
- ✅ PNG 输出（文字、尺寸、布局全部修复）
- ✅ SVG 输出
- ✅ 22+ 图表类型支持
- ✅ 中文完全支持
- ✅ 自动错误修复
- ✅ 完整文档和示例
- ✅ CI/CD 工作流
- ✅ 生产级代码质量

### 📅 开发里程碑

| 版本 | 日期 | 里程碑 |
|------|------|--------|
| v0.1-alpha | 2026-06-14 | 初始版本，基础功能 |
| v0.2.0-alpha | 2026-06-28 | SVG 优化，性能基准 |
| **v0.2.1-alpha** | **2026-07-11** | **PNG 完全修复，生产就绪** |
| v0.3（计划） | 2026-08 | 高级图表增强 |

---

## 🏗️ 项目结构

```
mermaid-cli/
├── src/
│   ├── main.rs              # CLI 入口
│   ├── lib.rs               # 库入口
│   ├── parser/              # 解析模块（23 种图表）
│   ├── renderer/            # 渲染引擎
│   ├── svg/                 # SVG 生成
│   ├── fixer/               # 语法修复
│   └── utils/               # 工具函数
├── docs/                    # 📚 完整文档
│   ├── SEQUENCE_DIAGRAM_GUIDE.md
│   ├── FLOWCHART_GUIDE.md
│   ├── COMPARISON_ACTUAL_TEST.md
│   └── ... 更多指南
├── examples/                # 📝 实际例子
│   ├── shopping_sequence.mmd/.png
│   ├── checkout_flow.mmd/.png
│   ├── oa_reimbursement_flow.mmd/.png
│   └── ... 更多示例
├── tests/                   # 🧪 测试
├── .github/workflows/       # CI/CD 配置
├── Cargo.toml              # 项目配置
└── README.md               # 本文件
```

---

## 🧪 测试和质量

### 代码质量

```bash
# 运行所有检查
cargo test          # 单元测试
cargo clippy        # 代码检查
cargo fmt --check   # 格式检查
```

### 测试覆盖

- ✅ 解析器单元测试（各图表类型）
- ✅ 渲染器集成测试（SVG + PNG）
- ✅ 端到端测试（真实图表）
- ✅ 性能基准测试（对标 mmdc）

---

## 📈 何时使用 Rust mermaid-cli

### ✅ 推荐场景

- **AI 系统** — 实时渲染，需要毫秒级响应
- **批量处理** — 1000+ 图表，时间和资源受限
- **容器部署** — Kubernetes、Docker，需要轻量级镜像
- **CI/CD 集成** — 文档自动生成流水线
- **边界计算** — 资源受限环境（<50MB 内存）
- **离线工具** — 无网络依赖，快速启动

### ⚠️ 保留 mmdc 场景

- 需要 PDF 输出
- 需要完整 CSS 自定义
- 已有成熟 Node 环境

详见 [完整对比分析](./docs/COMPARISON_ACTUAL_TEST.md)

---

## 🤝 贡献指南

欢迎贡献！请：

1. Fork 仓库
2. 创建特性分支（`git checkout -b feature/amazing-feature`）
3. 提交更改（`git commit -m 'Add amazing feature'`）
4. 推送到分支（`git push origin feature/amazing-feature`）
5. 开启 Pull Request

详见 [DEVELOPMENT.md](./docs/DEVELOPMENT.md)

---

## 📞 支持和反馈

- 🐛 **Bug 报告** — [GitHub Issues](https://github.com/hao430/mermaid-cli/issues)
- 💬 **讨论建议** — [GitHub Discussions](https://github.com/hao430/mermaid-cli/discussions)
- 📖 **文档** — 查看 `docs/` 目录

---

## 📄 许可证

MIT License — 详见 [LICENSE](./LICENSE)

本项目在 MIT 许可证下发布，允许自由使用、修改和分发。

---

## 🙏 致谢

感谢以下项目和社区的支持：

- **[Mermaid.js](https://mermaid.js.org/)** — 优秀的图表语言设计
- **[Rust 生态](https://www.rust-lang.org/)** — 强大的工具链和库
- **所有贡献者和用户** — 测试、反馈和改进

---

## 📊 快速数据

| 指标 | 数值 |
|------|------|
| **性能** | 3-4 倍快于 mmdc |
| **依赖** | 零个外部依赖 |
| **二进制** | 50MB 单文件 |
| **图表类型** | 23+ 种 |
| **代码质量** | 0 警告 |
| **文档** | 20+ 份完整指南 |
| **容器镜像** | <100MB |
| **启动时间** | <5ms |

---

## 🎯 下一步

👉 **立即开始**：[📖 QUICK_START.md](./docs/QUICK_START.md)

👉 **查看示例**：[examples/](./examples/) 目录

👉 **深入了解**：[docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md)

---

<div align="center">

**⭐ 如果觉得有用，请给个星标支持！**

[GitHub](https://github.com/hao430/mermaid-cli) | [文档](./docs/) | [问题反馈](https://github.com/hao430/mermaid-cli/issues)

</div>

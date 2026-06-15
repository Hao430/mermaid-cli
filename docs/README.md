# Mermaid CLI - Rust 实现

高性能、零依赖的 Mermaid 图表命令行工具，专为 AI 系统设计。

## 核心特性

- 🚀 **性能优先** — 相比 Node.js 版本更快、更轻
- 📦 **单文件二进制** — 无依赖、开箱即用
- 🤖 **AI 友好** — 支持流式调用、自动错误修复
- 🎨 **精美输出** — 导出高质量 SVG
- 🔧 **智能纠错** — 自动识别和修复常见语法错误
- 📍 **精准诊断** — 返回带位置的错误信息

## 快速开始

### 安装

从 [GitHub Releases](https://github.com/yourusername/mermaid-cli/releases) 下载预编译二进制：

```bash
# macOS
curl -L https://github.com/yourusername/mermaid-cli/releases/download/v0.1.0/mermaid-cli-x86_64-apple-darwin -o mermaid-cli
chmod +x mermaid-cli

# Linux
curl -L https://github.com/yourusername/mermaid-cli/releases/download/v0.1.0/mermaid-cli-x86_64-unknown-linux-gnu -o mermaid-cli
chmod +x mermaid-cli

# Windows
# 下载 mermaid-cli-x86_64-pc-windows-gnu.exe
```

或者从源码编译：

```bash
git clone https://github.com/yourusername/mermaid-cli.git
cd mermaid-cli
cargo build --release
./target/release/mermaid-cli --help
```

### 基本用法

#### 从文件渲染

```bash
mermaid-cli example.mmd -o output.svg
```

#### 从 stdin（适合 AI 调用）

```bash
echo 'graph TD; A[Start]-->B[End]' | mermaid-cli --stdin -o diagram.svg
```

#### 语法检查和修复建议

```bash
# 检查语法错误
mermaid-cli check diagram.mmd

# 显示修复建议
mermaid-cli check diagram.mmd --show-fixes

# 自动修复并输出
mermaid-cli fix diagram.mmd -o fixed.mmd
```

## 支持的图表类型

| 类型 | 状态 | 版本 |
|------|------|------|
| 流程图 (Flowchart) | ✅ MVP | v0.1 |
| 序列图 (Sequence) | ✅ MVP | v0.1 |
| 类图 (Class) | 🔄 规划 | v0.2+ |
| 状态图 (State) | 🔄 规划 | v0.2+ |
| 甘特图 (Gantt) | 🔄 规划 | v0.2+ |

## 项目结构

```
mermaid-cli/
├── docs/                      # 文档
│   ├── README.md             # 项目概述
│   ├── ARCHITECTURE.md        # 架构设计
│   ├── DEVELOPMENT.md         # 开发指南
│   ├── ROADMAP.md            # 路线图
│   └── API.md                # API 文档
├── src/
│   ├── main.rs               # CLI 入口
│   ├── lib.rs                # 库接口
│   ├── parser/               # 解析模块
│   │   ├── mod.rs
│   │   ├── lexer.rs         # 词法分析
│   │   └── ast.rs           # 抽象语法树
│   ├── renderer/             # 渲染模块
│   │   ├── mod.rs
│   │   ├── flowchart.rs     # 流程图渲染
│   │   └── sequence.rs      # 序列图渲染
│   ├── fixer/                # 错误修复模块
│   │   ├── mod.rs
│   │   ├── errors.rs        # 错误定义
│   │   └── autocorrect.rs   # 自动修复
│   └── svg/                  # SVG 生成工具
├── tests/
│   ├── parser_tests.rs
│   └── integration_tests.rs
├── examples/
│   └── sample.mmd
└── Cargo.toml
```

## 用例示例

### 用例 1：文档生成流程

```bash
# 自动将 .mmd 文件转换为 SVG
for file in diagrams/*.mmd; do
  mermaid-cli "$file" -o "docs/$(basename $file .mmd).svg"
done
```

### 用例 2：AI 集成

```python
import subprocess
import json

def generate_diagram(mermaid_code: str) -> str:
    """AI 调用 CLI 生成图表"""
    result = subprocess.run(
        ['mermaid-cli', '--stdin', '-o', 'output.svg'],
        input=mermaid_code.encode(),
        capture_output=True
    )
    
    if result.returncode != 0:
        # 获取诊断信息并重试纠错
        diagnosis = json.loads(result.stderr)
        print(f"Error: {diagnosis}")
    
    return 'output.svg'
```

### 用例 3：CI/CD 集成

```yaml
# GitHub Actions
- name: Generate Diagrams
  run: |
    for file in docs/**/*.mmd; do
      ./mermaid-cli "$file" -o "${file%.mmd}.svg"
    done
    
- name: Commit changes
  run: |
    git add docs/**/*.svg
    git commit -m "Generate diagrams"
```

## 命令行参考

```
USAGE:
    mermaid-cli [COMMAND] [OPTIONS] [INPUT]

COMMANDS:
    render      从文件或 stdin 渲染图表（默认）
    check       检查 Mermaid 语法
    fix         自动修复 Mermaid 代码
    help        显示帮助信息

OPTIONS:
    -o, --output <FILE>        输出文件路径 (默认: stdout)
    --stdin                    从标准输入读取
    --show-fixes               显示修复建议（仅 check 命令）
    -h, --help                 显示帮助
    -V, --version              显示版本

EXAMPLES:
    # 从文件渲染
    mermaid-cli input.mmd -o output.svg
    
    # 从 stdin 渲染
    echo 'graph TD; A-->B' | mermaid-cli --stdin
    
    # 检查语法
    mermaid-cli check diagram.mmd --show-fixes
    
    # 自动修复
    mermaid-cli fix diagram.mmd -o fixed.mmd
```

## 性能对比

| 工具 | 体积 | 启动时间 | 内存 |
|------|------|---------|------|
| mermaid-cli (Rust) | ~5MB | ~10ms | ~20MB |
| mermaid-cli (Node.js) | ~200MB | ~500ms | ~150MB |

## 开发

详见 [DEVELOPMENT.md](./DEVELOPMENT.md)

## 许可证

MIT License

## 贡献

欢迎 PR！请阅读 [DEVELOPMENT.md](./DEVELOPMENT.md) 了解开发流程。

# API 文档

## 公共 API

### 库级别 API

Mermaid CLI 可以作为 Rust 库使用，提供编程接口。

#### 基础函数

##### `render(code: &str) -> Result<String, RenderError>`

将 Mermaid 代码渲染为 SVG。

**参数**：
- `code` — Mermaid 图表代码

**返回**：
- `Ok(String)` — SVG 字符串
- `Err(RenderError)` — 渲染错误

**示例**：

```rust
use mermaid_cli::render;

fn main() {
    let code = "graph TD; A[Start]-->B[End]";
    match render(code) {
        Ok(svg) => println!("{}", svg),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

##### `parse(code: &str) -> Result<Diagram, Vec<ParseError>>`

解析 Mermaid 代码为 AST。

**参数**：
- `code` — Mermaid 图表代码

**返回**：
- `Ok(Diagram)` — 解析得到的图表 AST
- `Err(Vec<ParseError>)` — 所有解析错误

**示例**：

```rust
use mermaid_cli::parse;

fn main() {
    let code = "graph TD; A[Start]-->B[End]";
    match parse(code) {
        Ok(diagram) => println!("Parsed: {:?}", diagram),
        Err(errors) => {
            for error in errors {
                println!("Error at line {}: {}", error.line, error.message);
            }
        }
    }
}
```

##### `check(code: &str) -> Result<CheckResult, CheckError>`

检查 Mermaid 代码的语法，返回详细的诊断信息。

**参数**：
- `code` — Mermaid 图表代码

**返回**：
- `Ok(CheckResult)` — 检查结果（包含错误和修复建议）
- `Err(CheckError)` — 检查过程中的错误

**示例**：

```rust
use mermaid_cli::check;

fn main() {
    let code = "grpah TD; A-->B";
    match check(code) {
        Ok(result) => {
            if result.has_errors() {
                for fix in &result.suggestions {
                    println!("Line {}: {}", fix.line, fix.suggestion);
                }
            }
        }
        Err(e) => eprintln!("Check error: {}", e),
    }
}
```

##### `fix(code: &str) -> Result<(String, Vec<AutoFix>), FixError>`

自动修复 Mermaid 代码中的常见错误。

**参数**：
- `code` — Mermaid 图表代码

**返回**：
- `Ok((String, Vec<AutoFix>))` — 修复后的代码和应用的修复列表
- `Err(FixError)` — 修复过程中的错误

**示例**：

```rust
use mermaid_cli::fix;

fn main() {
    let code = "grpah TD\nA[Start]-->B";
    match fix(code) {
        Ok((fixed, fixes)) => {
            println!("Fixed code:\n{}", fixed);
            for fix in fixes {
                println!("Applied: {} -> {}", fix.original, fix.suggestion);
            }
        }
        Err(e) => eprintln!("Fix error: {}", e),
    }
}
```

---

### 类型定义

#### `Diagram`

表示一个完整的 Mermaid 图表。

```rust
pub struct Diagram {
    pub diagram_type: DiagramType,
    pub statements: Vec<Statement>,
    pub subgraphs: Vec<Subgraph>,
}
```

**字段**：
- `diagram_type` — 图表类型（Flowchart、Sequence 等）
- `statements` — 顶级语句列表
- `subgraphs` — 子图列表

#### `DiagramType`

```rust
pub enum DiagramType {
    Flowchart,
    Sequence,
    // 未来添加其他类型
}
```

#### `Statement`

表示图表中的一个语句。

```rust
pub enum Statement {
    NodeDef {
        id: String,
        label: Option<String>,
        shape: NodeShape,
        style: Option<NodeStyle>,
    },
    EdgeDef {
        from: String,
        to: String,
        label: Option<String>,
        arrow_type: ArrowType,
    },
    StyleDef {
        node_id: String,
        style: NodeStyle,
    },
}
```

#### `NodeShape`

```rust
pub enum NodeShape {
    Rect,           // []
    Circle,         // ()
    Diamond,        // {}
    Rounded,        // [()]
    RoundedRect,    // [[]]
    Named(String),  // [(name)]
}
```

#### `ArrowType`

```rust
pub enum ArrowType {
    Arrow,          // -->
    Line,           // --
    DashedArrow,    // -.->
    DashedLine,     // -.-
    ThickArrow,     // ==>
}
```

#### `ParseError`

表示一个解析错误。

```rust
pub struct ParseError {
    pub error_type: ErrorType,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub suggestion: Option<String>,
    pub context: String,  // 错误发生的那一行
}
```

#### `CheckResult`

语法检查结果。

```rust
pub struct CheckResult {
    pub errors: Vec<ParseError>,
    pub suggestions: Vec<AutoFix>,
    pub is_valid: bool,
}

impl CheckResult {
    pub fn has_errors(&self) -> bool { }
    pub fn to_json(&self) -> String { }
}
```

#### `AutoFix`

自动修复建议。

```rust
pub struct AutoFix {
    pub line: usize,
    pub column: usize,
    pub original: String,
    pub suggestion: String,
    pub fix_type: FixType,
    pub confidence: f32,  // 0.0 - 1.0
}

pub enum FixType {
    AddMissingKeyword,
    FixTypo,
    AddMissingLabel,
    FixArrowSyntax,
    // ...
}
```

#### `RenderError`

渲染错误。

```rust
pub enum RenderError {
    ParseError(Vec<ParseError>),
    LayoutError(String),
    SvgGenerationError(String),
    IoError(std::io::Error),
}
```

---

## CLI API

### 命令和选项

#### 主命令

```
mermaid-cli [COMMAND] [OPTIONS] [INPUT]
```

#### 子命令

##### `render`（默认）

渲染 Mermaid 代码为 SVG。

```bash
mermaid-cli render <INPUT> [OPTIONS]
mermaid-cli <INPUT> [OPTIONS]  # render 是默认命令
```

**选项**：
- `-o, --output <FILE>` — 输出文件路径（默认：stdout）
- `--stdin` — 从标准输入读取
- `--width <PIXELS>` — SVG 宽度（默认：800）
- `--height <PIXELS>` — SVG 高度（默认：600）

**示例**：

```bash
# 从文件输入
mermaid-cli diagram.mmd -o output.svg

# 从 stdin
echo 'graph TD; A-->B' | mermaid-cli --stdin -o output.svg

# 指定尺寸
mermaid-cli diagram.mmd -o output.svg --width 1024 --height 768

# 输出到 stdout
mermaid-cli diagram.mmd
```

##### `check`

检查 Mermaid 代码的语法。

```bash
mermaid-cli check <INPUT> [OPTIONS]
```

**选项**：
- `--show-fixes` — 显示修复建议
- `--format <FORMAT>` — 输出格式（text、json；默认：text）

**示例**：

```bash
# 基础检查
mermaid-cli check diagram.mmd

# 显示修复建议
mermaid-cli check diagram.mmd --show-fixes

# JSON 格式输出
mermaid-cli check diagram.mmd --format json
```

**输出示例（JSON）**：

```json
{
  "valid": false,
  "errors": [
    {
      "line": 1,
      "column": 0,
      "type": "UnknownKeyword",
      "message": "Unknown keyword: 'grpah'",
      "suggestion": "Did you mean 'graph'?",
      "context": "grpah TD"
    }
  ],
  "suggestions": [
    {
      "line": 1,
      "column": 0,
      "original": "grpah",
      "suggestion": "graph",
      "confidence": 0.95
    }
  ]
}
```

##### `fix`

自动修复 Mermaid 代码。

```bash
mermaid-cli fix <INPUT> [OPTIONS]
```

**选项**：
- `-o, --output <FILE>` — 输出文件路径（默认：stdout）
- `--show-changes` — 显示修复的详细信息
- `--dry-run` — 不修改文件，仅预览

**示例**：

```bash
# 修复并输出到文件
mermaid-cli fix broken.mmd -o fixed.mmd

# 预览修复（不输出文件）
mermaid-cli fix broken.mmd --dry-run

# 显示修复详情
mermaid-cli fix broken.mmd --show-changes
```

##### `help`

显示帮助信息。

```bash
mermaid-cli help [COMMAND]
mermaid-cli --help
mermaid-cli -h
```

##### `version`

显示版本信息。

```bash
mermaid-cli --version
mermaid-cli -V
```

---

## 全局选项

| 选项 | 说明 | 默认值 |
|------|------|--------|
| `-v, --verbose` | 详细输出 | 否 |
| `--quiet` | 安静模式（无输出） | 否 |
| `--config <FILE>` | 配置文件路径 | 无 |
| `-h, --help` | 显示帮助 | — |
| `-V, --version` | 显示版本 | — |

---

## 环境变量

| 变量 | 说明 | 示例 |
|------|------|------|
| `MERMAID_THEME` | 默认主题 | `dark`, `light` |
| `MERMAID_OUTPUT_DIR` | 默认输出目录 | `/tmp/diagrams` |
| `RUST_LOG` | 日志级别 | `debug`, `info`, `warn` |

---

## 出错处理

### 错误代码

CLI 返回的错误代码：

| 代码 | 说明 |
|------|------|
| 0 | 成功 |
| 1 | 通用错误 |
| 2 | 参数错误 |
| 3 | 语法错误（代码中） |
| 4 | 文件 I/O 错误 |
| 5 | 渲染错误 |

### 错误消息

错误信息会输出到 stderr，格式为：

```
ERROR [location]: <message>
```

**示例**：

```
ERROR [diagram.mmd:1:0]: Unknown keyword 'grpah'. Did you mean 'graph'?
```

---

## 集成示例

### Python 集成

```python
import subprocess
import json

def render_diagram(mermaid_code: str, output_file: str) -> bool:
    """使用 mermaid-cli 渲染图表"""
    try:
        result = subprocess.run(
            ['mermaid-cli', '--stdin', '-o', output_file],
            input=mermaid_code.encode(),
            capture_output=True,
            timeout=10
        )
        return result.returncode == 0
    except Exception as e:
        print(f"Error: {e}")
        return False

def check_diagram(mermaid_code: str) -> dict:
    """检查图表语法并返回诊断信息"""
    try:
        result = subprocess.run(
            ['mermaid-cli', 'check', '--format', 'json'],
            input=mermaid_code.encode(),
            capture_output=True,
            timeout=10
        )
        return json.loads(result.stdout)
    except Exception as e:
        print(f"Error: {e}")
        return {}
```

### JavaScript/Node.js 集成

```javascript
const { execSync } = require('child_process');
const fs = require('fs');

function renderDiagram(mermaidCode, outputFile) {
  try {
    const result = execSync('mermaid-cli --stdin -o ' + outputFile, {
      input: mermaidCode,
      encoding: 'utf-8'
    });
    return fs.existsSync(outputFile);
  } catch (error) {
    console.error('Error:', error.message);
    return false;
  }
}

function checkDiagram(mermaidCode) {
  try {
    const result = execSync('mermaid-cli check --format json', {
      input: mermaidCode,
      encoding: 'utf-8'
    });
    return JSON.parse(result);
  } catch (error) {
    console.error('Error:', error.message);
    return {};
  }
}
```

---

## 性能特性

### 性能指标

- **启动时间**：~10ms
- **内存占用**：~20MB（基础）
- **处理速度**：平均 1MB/s 的 Mermaid 代码

### 优化建议

1. **批处理** — 处理多个文件时使用脚本，避免重复启动
   ```bash
   for file in *.mmd; do
       mermaid-cli "$file" -o "${file%.mmd}.svg"
   done
   ```

2. **管道处理** — 使用 stdin/stdout 管道
   ```bash
   cat diagram.mmd | mermaid-cli --stdin > output.svg
   ```

3. **缓存** — 缓存不经常变化的图表
   ```bash
   # 仅在输入文件有变化时重新渲染
   if [ diagram.mmd -nt output.svg ]; then
       mermaid-cli diagram.mmd -o output.svg
   fi
   ```

---

## 已知限制

**v0.1 MVP**：
- 仅支持 Flowchart 和 Sequence 图表
- SVG 输出仅（PNG/PDF 在 v0.2+）
- 自动布局算法较简单
- 不支持自定义 CSS

**未来改进**：
- 更复杂的布局算法
- 完整的样式系统
- 其他导出格式
- 主题和插件系统

---

**最后更新**：2026-06-14

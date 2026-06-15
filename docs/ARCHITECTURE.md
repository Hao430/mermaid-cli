# 架构设计文档

## 系统概览

```
┌─────────────────────────────────────────────────────────┐
│                        CLI 入口 (main.rs)               │
│  解析参数、管理 I/O、调用库函数                           │
└────────┬────────────────────────────────────────────────┘
         │
    ┌────▼────────────────────────────────────────────────┐
    │              公共 API (lib.rs)                        │
    │  render() | check() | fix() | parse()               │
    └────┬──────────────────────────────────────────────┬─┘
         │                                              │
    ┌────▼──────────────────┐              ┌───────────▼──┐
    │   Parser 模块          │              │  Fixer 模块  │
    ├─────────────────────┤              ├──────────────┤
    │ lexer.rs - 词法分析  │              │ errors.rs    │
    │ ast.rs - 语法树      │              │ autocorrect  │
    │ mod.rs - 解析协调    │              │              │
    └────┬────────────────┘              └──────┬──────┘
         │                                      │
         └─────────┬──────────────────────────┬─┘
                   │                          │
            ┌──────▼──────┐         ┌─────────▼────┐
            │   Renderer   │         │  Diagnostics │
            ├─────────────┤         ├──────────────┤
            │ flowchart.rs│         │ 错误位置信息 │
            │ sequence.rs │         │ 修复建议     │
            │ mod.rs      │         │ JSON 输出    │
            └──────┬──────┘         └──────────────┘
                   │
            ┌──────▼──────┐
            │  SVG 模块   │
            ├─────────────┤
            │ mod.rs      │
            │ 生成 SVG    │
            │ 节点、边、  │
            │ 标签等      │
            └─────────────┘
```

## 模块详解

### 1. Parser 模块 (`src/parser/`)

**职责**：将 Mermaid 代码转换为 AST

#### `lexer.rs` - 词法分析

```rust
pub struct Lexer {
    input: String,
    position: usize,
    line: usize,
    column: usize,
}

pub enum TokenType {
    Keyword(String),      // graph, flowchart, etc.
    Node(String),         // node id
    Arrow(ArrowType),     // -->, --, etc.
    Shape(ShapeType),     // [], (), {}, etc.
    Label(String),        // text in []
    Comment,
    Semicolon,
    Subgraph,
    End,
    // ...
}

pub struct Token {
    token_type: TokenType,
    line: usize,
    column: usize,
    length: usize,
}
```

**流程**：
1. 逐字符扫描输入
2. 识别关键字、符号、标识符
3. 记录位置信息（行号、列号）
4. 返回 Token 流

#### `ast.rs` - 抽象语法树

```rust
pub struct Diagram {
    pub diagram_type: DiagramType,
    pub statements: Vec<Statement>,
    pub subgraphs: Vec<Subgraph>,
}

pub enum Statement {
    NodeDef {
        id: String,
        label: Option<String>,
        shape: NodeShape,
    },
    EdgeDef {
        from: String,
        to: String,
        label: Option<String>,
        arrow_type: ArrowType,
    },
    // ... 其他语句类型
}

pub enum DiagramType {
    Flowchart,
    Sequence,
}
```

#### `mod.rs` - 解析器主体

```rust
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    errors: Vec<ParseError>,
}

impl Parser {
    pub fn parse(&mut self) -> Result<Diagram, Vec<ParseError>> {
        // 解析并收集错误
        // 错误恢复：允许部分解析
    }
    
    fn parse_statement(&mut self) -> Option<Statement> { }
    fn parse_node(&mut self) -> Option<Statement> { }
    fn parse_edge(&mut self) -> Option<Statement> { }
    // ...
}
```

**错误恢复策略**：
- 尽可能继续解析，而不是立即返回错误
- 记录所有错误及其位置
- 允许部分有效的 AST 生成

---

### 2. Fixer 模块 (`src/fixer/`)

**职责**：识别和修复错误

#### `errors.rs` - 错误定义

```rust
pub enum ErrorType {
    SyntaxError,
    MissingKeyword,
    MissingLabel,
    MissingEnd,
    InvalidArrow,
    TypoInKeyword,
}

pub struct ParseError {
    pub error_type: ErrorType,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub suggestion: Option<String>,  // 修复建议
    pub original_text: String,
}

pub struct DiagnosticInfo {
    pub errors: Vec<ParseError>,
    pub suggestions: Vec<AutoFix>,
}

pub struct AutoFix {
    pub line: usize,
    pub column: usize,
    pub original: String,
    pub suggestion: String,
    pub confidence: f32,  // 修复的确定性
}
```

#### `autocorrect.rs` - 自动修复逻辑

```rust
pub struct Fixer {
    common_typos: HashMap<String, String>,
    keyword_patterns: Vec<Regex>,
}

impl Fixer {
    pub fn fix(&self, code: &str) -> (String, Vec<AutoFix>) {
        // 应用修复规则
        // 返回修复后的代码 + 修复列表
    }
    
    fn fix_missing_end(&self, ast: &Diagram) -> Vec<AutoFix> { }
    fn fix_typos(&self, code: &str) -> Vec<AutoFix> { }
    fn fix_missing_labels(&self, ast: &Diagram) -> Vec<AutoFix> { }
    fn fix_arrow_syntax(&self, code: &str) -> Vec<AutoFix> { }
}

// 修复规则示例
pub const COMMON_TYPOS: &[(&str, &str)] = &[
    ("grpah", "graph"),
    ("flwochart", "flowchart"),
    ("-->>", "-->"),
    ("=>", "->"),
];
```

---

### 3. Renderer 模块 (`src/renderer/`)

**职责**：将 AST 转换为可视化表示

#### 架构

```rust
pub trait DiagramRenderer {
    fn render(&self, diagram: &Diagram) -> Result<String, RenderError>;
}

pub struct FlowchartRenderer {
    config: RenderConfig,
}

pub struct SequenceRenderer {
    config: RenderConfig,
}

pub struct RenderConfig {
    pub width: u32,
    pub height: u32,
    pub font_size: u32,
    pub node_spacing: u32,
    pub colors: ColorScheme,
}
```

#### `flowchart.rs` - 流程图渲染

**布局算法**（MVP）：
1. **节点收集** — 遍历 AST 收集所有节点
2. **分层** — 按拓扑排序分层（流程图从上到下）
3. **坐标计算** — 计算每个节点的位置
4. **边渲染** — 根据坐标绘制连接线

```rust
pub struct FlowchartRenderer {
    nodes: HashMap<String, Node>,
    edges: Vec<Edge>,
    layout: LayoutInfo,
}

struct Node {
    id: String,
    label: String,
    shape: NodeShape,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

struct LayoutInfo {
    layers: Vec<Vec<String>>,  // 按层存储节点 ID
    layer_heights: Vec<f32>,
}

impl FlowchartRenderer {
    pub fn render(&self) -> String {
        // 1. 布局
        let layout = self.compute_layout();
        // 2. 生成 SVG
        let svg = self.generate_svg(&layout);
        svg
    }
    
    fn compute_layout(&self) -> LayoutInfo { }
    fn generate_svg(&self, layout: &LayoutInfo) -> String { }
}
```

#### `sequence.rs` - 序列图渲染

**布局算法**：
1. **参与者排列** — 水平排列参与者
2. **时间线** — 垂直时间轴
3. **消息绘制** — 参与者间的消息箭头
4. **激活框** — 显示参与者的活跃期间

---

### 4. SVG 模块 (`src/svg/`)

**职责**：生成 SVG 代码

```rust
pub struct SvgBuilder {
    elements: Vec<SvgElement>,
    width: u32,
    height: u32,
}

pub enum SvgElement {
    Rect { x: f32, y: f32, width: f32, height: f32, style: String },
    Circle { cx: f32, cy: f32, r: f32, style: String },
    Line { x1: f32, y1: f32, x2: f32, y2: f32, style: String },
    Text { x: f32, y: f32, content: String, style: String },
    Path { d: String, style: String },
}

impl SvgBuilder {
    pub fn add_rect(&mut self, x: f32, y: f32, width: f32, height: f32, style: &str) { }
    pub fn add_text(&mut self, x: f32, y: f32, text: &str, style: &str) { }
    pub fn add_arrow(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) { }
    
    pub fn build(self) -> String {
        // 生成 SVG XML 字符串
    }
}
```

---

## 数据流

### 流程 1: 渲染流程

```
输入 (Mermaid Code)
    ↓
Lexer (词法分析)
    ↓ [Token Stream]
Parser (语法解析)
    ↓ [AST]
Renderer (渲染)
    ↓ [坐标和样式]
SVG 生成
    ↓
输出 (SVG String)
```

### 流程 2: 错误修复流程

```
输入 (Mermaid Code)
    ↓
Lexer + Parser (带错误恢复)
    ↓ [部分 AST + 错误列表]
Fixer (自动修复)
    ↓ [修复建议 + 诊断信息]
输出 (JSON 诊断信息)
```

---

## 关键设计决策

### 1. 为什么分离 Lexer 和 Parser？

- **清晰职责** — Lexer 处理字符流，Parser 处理 Token 流
- **错误位置追踪** — Token 包含行列信息，便于精确诊断
- **可重用性** — Lexer 和 Parser 可独立测试和优化

### 2. 为什么使用错误恢复？

- **部分解析** — 即使有错误也能生成部分 AST
- **全面诊断** — 一次运行发现所有错误，而不是第一个就停止
- **用户体验** — AI 可以基于部分结果做出修复建议

### 3. 为什么 Fixer 独立于 Parser？

- **关注点分离** — 解析和修复是两个不同的问题
- **灵活性** — 可以应用不同的修复策略
- **可测试性** — 独立测试修复逻辑

### 4. SVG 生成与渲染分离？

- **SVG 模块只负责生成** — 格式正确的 SVG 代码
- **Renderer 负责布局和坐标** — Flowchart/Sequence 各自实现
- **便于扩展** — 未来可支持其他输出格式（PNG、PDF）

---

## 依赖关系图

```
main.rs (CLI)
    ↓
lib.rs (Public API)
    ├── parser/
    │   ├── lexer
    │   ├── ast
    │   └── mod (Parser)
    │
    ├── fixer/
    │   ├── errors
    │   └── autocorrect
    │
    ├── renderer/
    │   ├── flowchart
    │   ├── sequence
    │   └── mod (trait DiagramRenderer)
    │
    └── svg/
        └── mod (SvgBuilder)
```

没有循环依赖，下层模块不依赖上层模块。

---

## 性能考量

- **Lexer** — O(n) 线性扫描
- **Parser** — O(n) 单遍解析
- **Renderer** — O(n + e) 其中 n 是节点数，e 是边数
- **SVG 生成** — O(n + e) 生成 SVG 元素

**整体时间复杂度**：O(n)，其中 n 是输入代码的长度。

---

## 错误处理策略

1. **Lexer** — 未知字符跳过，继续扫描
2. **Parser** — 记录错误，尝试恢复并继续解析
3. **Fixer** — 应用修复规则，返回修复建议
4. **Renderer** — 缺失或无效的节点使用默认值

这样确保即使有多个错误，也能提供最大的有用信息。

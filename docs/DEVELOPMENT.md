# 开发指南

## 环境配置

### 前置要求

- Rust 1.70+ （使用 [rustup](https://rustup.rs/) 安装）
- Git
- 文本编辑器（推荐 VSCode + rust-analyzer）

### 快速开始

```bash
# 克隆项目
git clone https://github.com/yourusername/mermaid-cli.git
cd mermaid-cli

# 安装依赖
cargo build

# 运行测试
cargo test

# 构建发布版本
cargo build --release

# 查看帮助
./target/debug/mermaid-cli --help
```

---

## 项目结构与模块

```
src/
├── main.rs              # CLI 入口，解析参数并调用 lib 函数
├── lib.rs               # 公共 API：render()、check()、fix() 等
├── parser/
│   ├── mod.rs          # Parser 结构体和解析入口
│   ├── lexer.rs        # 词法分析器
│   └── ast.rs          # 抽象语法树定义
├── renderer/
│   ├── mod.rs          # DiagramRenderer trait 和通用逻辑
│   ├── flowchart.rs    # 流程图渲染器
│   └── sequence.rs     # 序列图渲染器（P2）
├── fixer/
│   ├── mod.rs          # 修复器主体
│   ├── errors.rs       # 错误类型定义
│   └── autocorrect.rs  # 自动修复规则
└── svg/
    └── mod.rs          # SVG 生成工具
```

---

## 开发工作流

### 1. 创建新功能分支

```bash
git checkout -b feature/add-xyz
```

### 2. 代码开发

#### 添加新的解析规则

**示例**：支持节点边框样式

1. **在 `ast.rs` 中定义新的 AST 节点类型**

```rust
pub struct Node {
    pub id: String,
    pub label: Option<String>,
    pub shape: NodeShape,
    pub style: Option<NodeStyle>,  // 新增
}

pub struct NodeStyle {
    pub border_width: f32,
    pub border_color: String,
    pub fill_color: String,
}
```

2. **在 `lexer.rs` 中识别新的符号**

```rust
fn read_style_directive(&mut self) -> Option<Token> {
    if self.peek_char() == ':' {
        // 读取样式指令
        Token { token_type: TokenType::Style, ... }
    }
    None
}
```

3. **在 `parser/mod.rs` 中添加解析逻辑**

```rust
fn parse_node_definition(&mut self) -> Result<Statement, ParseError> {
    // 解析节点
    let mut node = Node { ... };
    
    // 如果有样式指令，解析样式
    if self.match_token(&TokenType::Colon) {
        node.style = Some(self.parse_node_style()?);
    }
    
    Ok(Statement::NodeDef(node))
}
```

4. **在 `renderer/flowchart.rs` 中应用样式**

```rust
fn render_node(&self, node: &Node) -> SvgElement {
    let style = match &node.style {
        Some(s) => format!("stroke-width:{};fill:{}", s.border_width, s.fill_color),
        None => "stroke:#000;fill:#fff".to_string(),
    };
    
    // 使用 style 生成 SVG
}
```

5. **添加测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_node_with_style() {
        let code = "graph TD\n  A[Start]:border-red:";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        
        // 验证样式被正确解析
    }
}
```

#### 添加新的修复规则

1. **在 `errors.rs` 中定义错误类型**

```rust
pub enum ErrorType {
    MissingEnd,
    InvalidNodeId,
    MissingLabelQuote,  // 新增
}
```

2. **在 `autocorrect.rs` 中实现修复**

```rust
impl Fixer {
    fn fix_missing_label_quotes(&self, code: &str) -> Vec<AutoFix> {
        let mut fixes = Vec::new();
        
        // 扫描找到缺少引号的标签
        for (line_idx, line) in code.lines().enumerate() {
            if let Some(col) = line.find("[") {
                let label_part = &line[col..];
                if !label_part.contains('"') {
                    // 建议添加引号
                    fixes.push(AutoFix {
                        line: line_idx,
                        column: col,
                        suggestion: format!('["{}"{}', ...),
                    });
                }
            }
        }
        
        fixes
    }
}
```

---

### 3. 编写测试

#### 单元测试

放在模块底部的 `#[cfg(test)]` 块中：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lexer_basic_tokens() {
        let mut lexer = Lexer::new("graph TD");
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0].token_type, TokenType::Keyword("graph")));
        assert!(matches!(tokens[1].token_type, TokenType::Keyword("TD")));
    }
    
    #[test]
    fn test_parser_simple_flowchart() {
        let code = "graph TD\n  A[Start]-->B[End]";
        let mut parser = Parser::new(code);
        let diagram = parser.parse().unwrap();
        
        assert_eq!(diagram.diagram_type, DiagramType::Flowchart);
        assert_eq!(diagram.statements.len(), 3);  // 2 nodes + 1 edge
    }
    
    #[test]
    fn test_fixer_missing_end() {
        let code = "graph TD\n  A-->B";
        let fixer = Fixer::new();
        let (fixed, _) = fixer.fix(code);
        
        assert!(fixed.contains("end"));
    }
}
```

#### 集成测试

在 `tests/` 目录下创建测试文件：

```rust
// tests/integration_tests.rs

#[test]
fn test_complete_workflow() {
    let input = "graph TD\n  A[Start]-->B[End]";
    let svg = mermaid_cli::render(input).unwrap();
    
    assert!(svg.contains("<svg"));
    assert!(svg.contains("Start"));
    assert!(svg.contains("End"));
}
```

### 4. 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_lexer_basic_tokens

# 显示输出
cargo test -- --nocapture

# 运行集成测试
cargo test --test integration_tests
```

---

## 代码规范

### 命名约定

- **函数/方法**：snake_case
- **类型/结构体**：PascalCase
- **常量**：SCREAMING_SNAKE_CASE
- **模块**：snake_case

```rust
pub struct DiagramNode { }           // ✅
pub fn parse_flowchart() { }         // ✅
pub const DEFAULT_WIDTH: u32 = 800;  // ✅
```

### 错误处理

使用 `Result` 类型，避免 `panic!`（除非是确实不应该到达的代码）：

```rust
// ❌ 不推荐
fn parse_node(token: &Token) {
    let id = match &token.token_type {
        TokenType::Node(id) => id,
        _ => panic!("Expected node token"),  // 不好
    };
}

// ✅ 推荐
fn parse_node(token: &Token) -> Result<Node, ParseError> {
    let id = match &token.token_type {
        TokenType::Node(id) => id.clone(),
        _ => return Err(ParseError::expected("node token")),
    };
    Ok(Node { id, ... })
}
```

### 文档注释

为公共 API 添加文档注释：

```rust
/// 渲染 Mermaid 图表为 SVG
/// 
/// # Arguments
/// 
/// * `code` - Mermaid 图表代码
/// 
/// # Returns
/// 
/// 返回 SVG 字符串，或返回错误信息
/// 
/// # Example
/// 
/// ```
/// let svg = mermaid_cli::render("graph TD; A-->B")?;
/// ```
pub fn render(code: &str) -> Result<String, RenderError> {
    // ...
}
```

### 导入组织

```rust
// 标准库导入
use std::collections::HashMap;
use std::fmt;

// 第三方库导入
use serde::{Deserialize, Serialize};

// 本地模块导入
use crate::parser::{Parser, Lexer};
use crate::renderer::Renderer;
```

---

## 调试技巧

### 打印调试

```rust
// 基础打印
println!("Debug: {:?}", variable);

// 使用 dbg! 宏（会自动打印变量名和值）
let x = 42;
dbg!(x);  // 输出: [src/main.rs:10] x = 42

// 格式化输出
eprintln!("Error: {}", error_message);
```

### 编译器错误信息

```bash
# 获取详细的编译错误
cargo build 2>&1 | head -50

# 使用 RUST_BACKTRACE 查看完整的栈追踪
RUST_BACKTRACE=1 cargo run input.mmd
```

### 测试特定场景

```bash
# 运行单个文件的所有测试
cargo test --test integration_tests

# 运行名称匹配的测试
cargo test flowchart
```

---

## 性能优化

### 基准测试

```bash
# 使用 criterion（需在 Cargo.toml 添加 dev-dependency）
cargo bench
```

### 性能分析

```bash
# 编译并运行 perf
cargo build --release
perf record ./target/release/mermaid-cli input.mmd -o output.svg
perf report
```

### 常见优化点

1. **避免不必要的克隆**
   ```rust
   // ❌ 不必要的克隆
   let nodes = ast.nodes.clone();
   
   // ✅ 使用引用
   let nodes = &ast.nodes;
   ```

2. **使用 String 而非 &str（当需要所有权时）**
   ```rust
   // ❌ 频繁分配
   let s = format!("node_{}", i);
   
   // ✅ 预分配
   let mut s = String::with_capacity(20);
   s.push_str("node_");
   s.push_str(&i.to_string());
   ```

3. **使用迭代器而非 Vec 中间值**
   ```rust
   // ❌ 创建中间向量
   let ids: Vec<_> = nodes.iter().map(|n| &n.id).collect();
   let filtered: Vec<_> = ids.iter().filter(|id| ...).collect();
   
   // ✅ 链式迭代
   nodes.iter()
       .map(|n| &n.id)
       .filter(|id| ...)
       .for_each(|id| { ... });
   ```

---

## 提交和 PR

### 提交消息格式

遵循 [Conventional Commits](https://www.conventionalcommits.org/)：

```
<type>[optional scope]: <description>

[optional body]

[optional footer]
```

**类型**：
- `feat` — 新功能
- `fix` — 修复 bug
- `docs` — 文档更新
- `style` — 代码格式（不改变功能）
- `refactor` — 重构（既不修复 bug 也不添加功能）
- `test` — 添加或修改测试
- `chore` — 依赖更新等维护工作

**示例**：

```
feat(parser): add support for node styling

- Add NodeStyle struct to AST
- Update lexer to recognize style directives
- Implement style parsing in parser
- Add tests for style parsing
```

### PR 检查清单

提交 PR 前，确保：

- [ ] 代码遵循项目规范
- [ ] 所有测试通过 (`cargo test`)
- [ ] 添加了必要的测试
- [ ] 更新了相关文档
- [ ] 提交消息清晰有意义
- [ ] 没有 `unwrap()` 或 `panic!`（除非合理）

---

## CI/CD 工作流

本项目使用 GitHub Actions 进行持续集成和持续交付。工作流配置文件位于 `.github/workflows/`。

### Test 工作流 (`.github/workflows/test.yml`)

在每次 push 和 pull request 时触发：

- `cargo fmt --check` — 代码格式检查
- `cargo clippy -- -D warnings` — 代码质量检查
- `cargo test --all` — 运行所有单元测试和集成测试
- `cargo doc --no-deps` — 文档构建验证

### Build 工作流 (`.github/workflows/build.yml`)

在每次 push 和 pull request 时触发，在三个平台编译：
- ubuntu-latest (x86_64)
- macos-latest (x86_64 + ARM64)
- windows-latest (x86_64)

编译 debug 和 release 两种配置，并上传 release 二进制作为 artifact。

### Release 工作流 (`.github/workflows/release.yml`)

在推送 git tag `v*` 时触发（例如 `v0.1.0`），自动创建 GitHub Release：

- 编译所有平台的 release 二进制
- 命名为 `mermaid-cli-<version>-<target>`
- 创建 GitHub Release 并上传二进制
- 当前已禁用，等待 P2（完整 Flowchart 支持）完成后启用

### 本地运行 CI 检查

```bash
# 在提交前运行这些命令
cargo fmt --check
cargo clippy -- -D warnings
cargo test --all
```

---

## 常见问题

### Q: 如何快速迭代？

```bash
# 使用 cargo watch 自动重新编译和测试
cargo install cargo-watch
cargo watch -x test -x "build --release"
```

### Q: 如何调试 Parser 错误？

```rust
// 在 Parser 中添加调试打印
#[cfg(debug_assertions)]
fn debug_log(msg: &str) {
    eprintln!("[DEBUG] {}", msg);
}

// 在需要的地方调用
debug_log(&format!("Parsing: {:?}", self.current_token));
```

### Q: 如何处理大文件？

- 使用流式处理而不是一次性加载整个文件
- 分块处理节点
- 考虑使用内存映射（mmap）

### Q: 如何贡献新功能？

1. 开一个 Issue 讨论想法
2. 在 Issue 中获得反馈
3. 创建功能分支
4. 实现 + 测试
5. 提交 PR
6. 审查和迭代

---

## 资源链接

- [Rust 官方书籍](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/)
- [Mermaid 官方文档](https://mermaid.js.org/)

---

## 联系和支持

- 提问：在 GitHub Issues 中发起讨论
- 报告 Bug：使用标签 `bug` 和详细的重现步骤
- 功能请求：使用标签 `enhancement`

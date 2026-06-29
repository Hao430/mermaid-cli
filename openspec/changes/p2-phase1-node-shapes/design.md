## Context

P1 完成了基础 flowchart 解析和渲染，但所有节点都渲染为矩形。AST 中已有 `NodeShape` 枚举（Rect, Circle, Diamond, Rounded, RoundedRect, Named），但解析器始终生成 `NodeShape::Rect`。词法分析器已识别 `(`, `)`, `{`, `}`, `[`, `]` 等单字符 token。

**当前代码状态**：
- `lexer.rs`：已识别 `[`, `]`, `(`, `)`, `{`, `}` 为独立 token
- `ast.rs`：`NodeShape` 枚举已定义 5 种形状 + Named
- `parser/mod.rs`：`parse_optional_label()` 只处理 `[label]`，所有节点固定 `Rect`
- `renderer/mod.rs`：只绘制矩形 + 文本，无形状区分

## Goals / Non-Goals

**Goals:**
- 支持 Mermaid 标准节点形状语法（至少 8 种）
- 解析器正确提取形状信息到 AST
- 渲染器根据形状绘制对应的 SVG 图形
- 保持向后兼容（`A-->B` 和 `A[Start]-->B[End]` 不受影响）

**Non-Goals:**
- 自动布局算法（P2 阶段 2）
- 节点样式（颜色、字体、边框宽度）
- Subgraph（P2 阶段 3）
- 边标签渲染（P2 阶段 3）

## Decisions

### D1: 形状解析策略 — 包围 token 序列匹配

**选择**：在 `parse_optional_label()` 中识别形状 token 序列，返回 `(Option<String>, NodeShape)`。

**备选方案**：
- A) 词法分析器识别完整形状 token → 放弃，因为 `([text])` 和 `[(text)]` 的词法边界模糊
- B) 解析器用 peek/advance 逐字符匹配包围符号 → **选择此方案**

**理由**：形状由外层包围符号决定，词法分析器已经将其拆分为独立 token，解析器只需按模式匹配。这避免了词法层面的复杂度。

### D2: 形状映射表

| 语法 | AST NodeShape | SVG 渲染 |
|------|--------------|----------|
| `[text]` | `Rect` | `<rect>` |
| `(text)` | `Circle` | `<rect rx="10">` |
| `{text}` | `Diamond` | `<polygon>` 旋转45° |
| `([text])` | `Rounded` | `<rect rx="20">` |
| `[[text]]` | `RoundedRect` | 双线矩形 |
| `[(text)]` | `Named("cylinder")` | `<path>` 椭圆顶 |
| `((text))` | `Named("double-circle")` | 双层 `<circle>` |
| `>text]` | `Named("flag")` | `<path>` 旗帜形状 |
| 无包围符号 | `Rect` | `<rect>` |

### D3: NodeShape 枚举重构

保留现有变体，新增以覆盖 Mermaid 标准：
```rust
pub enum NodeShape {
    Rect,           // [text]
    Circle,         // (text) → 圆角矩形
    Diamond,        // {text} → 菱形
    Rounded,        // ([text]) → 大圆角
    Subroutine,     // [[text]] → 双线矩形
    Cylinder,       // [(text)] → 圆柱
    DoubleCircle,   // ((text)) → 双圆
    Flag,           // >text] → 旗帜
    Named(String),  // 未知形状的 fallback
}
```

### D4: 渲染器形状分发

在 `renderer/mod.rs` 中根据 `NodeShape` 分发到不同的 SVG 绘制函数：
- `render_rect()` — 标准矩形
- `render_rounded()` — 圆角矩形
- `render_diamond()` — 菱形多边形
- `render_cylinder()` — 圆柱（矩形 + 顶部椭圆）
- `render_double_circle()` — 双层圆
- `render_flag()` — 旗帜路径

## Risks / Trade-offs

- **风险**：`>` 旗帜语法与 `>` 比较运算符冲突 → 缓解：Mermaid 中 `>` 只在节点包围中出现，上下文足够区分
- **风险**：圆柱和旗帜的 SVG 路径坐标硬编码 → 缓解：先用固定尺寸，P3 再优化
- **权衡**：`Diamond` 渲染为菱形多边形，文本可能溢出 → 接受：P3 再加文本裁剪

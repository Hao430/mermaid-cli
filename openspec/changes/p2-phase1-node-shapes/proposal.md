## Why

当前 MVP 只支持矩形节点，无法表达流程图中常见的菱形判断、圆角操作、圆柱数据库等语义。节点形状是 flowchart 最基础的可视化能力，是 P2 的首要任务。

## What Changes

- 扩展词法分析器，识别 `()`, `{}`, `[[`, `]]`, `[(`, `)]`, `((`, `))` 等形状 token
- 扩展 AST `NodeShape` 枚举，新增 10+ 种形状类型
- 扩展语法解析器的 `parse_node_or_edge_statements()`，解析节点形状语法
- 实现各形状的 SVG 渲染（矩形、圆角、菱形、圆形、圆柱、子程序等）
- 为每种形状添加单元测试和渲染测试

## Capabilities

### New Capabilities
- `node-shapes`: 支持 Mermaid 标准节点形状语法（矩形、圆角、菱形、圆形、圆柱、子程序、旗帜等），并在 SVG 中正确渲染对应形状

### Modified Capabilities
- `node-label-parsing`: 解析器需同时提取形状和标签，形状作为 `NodeDef.shape` 字段传递

## Impact

- **代码**：`src/parser/lexer.rs`（token 类型）、`src/parser/ast.rs`（NodeShape 枚举）、`src/parser/mod.rs`（解析逻辑）、`src/renderer/mod.rs`（渲染逻辑）
- **测试**：新增 15+ 单元测试，3+ API 集成测试
- **无依赖变更**
- **无 API 破坏性变更**：`render()` / `parse()` 接口不变

## Why

当前 Parser 无法正确处理 Mermaid 节点标签语法 `A[Start]-->B[End]`。输入 `A[Start]-->B[End]` 时，Parser 将其解析为 4 个独立节点（A、Start、B、End）而非 2 个带标签的节点。这是 P1 的关键缺口，阻塞 P2（完整 Flowchart 支持）的开发。

## What Changes

- **Parser 修复**：在 `parse_node_or_edge()` 中添加 `[label]` 标签解析逻辑
- **渲染改进**：SVG 节点显示标签文本而非节点 ID
- **边界处理**：支持 `A[Start]`、`A`、`A-->B`、`A[Start]-->B[End]` 等各种语法组合

## Capabilities

### New Capabilities

- `node-label-parsing`: 解析 `id[label]` 标签语法，支持节点 ID 和可选标签

### Modified Capabilities

- `parser`: 修改 `parse_node_or_edge()` 方法，添加标签解析逻辑
- `renderer`: 改进节点渲染，优先显示标签而非节点 ID

## Impact

- **修改文件**：`src/parser/mod.rs`、`src/renderer/mod.rs`、`tests/api_tests.rs`
- **向后兼容**：完全兼容现有 `A-->B` 语法
- **测试**：需要新增标签解析的单元测试和集成测试

## Context

当前 Parser 的 `parse_node_or_edge()` 方法只读取单个 token 作为节点 ID，遇到 `[` 时将其视为无效 token 跳过。这导致 `A[Start]` 被解析为两个独立的 token（`A` 和 `Start`），而非一个带标签的节点。

Mermaid 流程图语法中，节点定义格式为：
- `id` — 无标签节点
- `id[label]` — 带标签节点
- `id[label]-->id2[label2]` — 带标签的边

## Goals / Non-Goals

**Goals:**
- 解析 `id[label]` 标签语法
- 渲染时优先显示标签而非节点 ID
- 保持对 `A-->B` 无标签语法的完全兼容

**Non-Goals:**
- 支持其他节点形状语法 `()`、`{}`（留给 P2）
- 支持节点样式 `:::class`（留给 P2）
- 支持多行标签（留给 P2）

## Decisions

### 1. 标签解析位置

**决策**：在 `parse_node_or_edge()` 中解析标签，而非 `parse_node_id()`

**理由**：
- `parse_node_id()` 应只负责读取节点标识符
- 标签是节点定义的一部分，属于 `parse_node_or_edge()` 的职责
- 保持 `parse_node_id()` 的单一职责

### 2. 标签存储方式

**决策**：扩展 `Statement::NodeDef` 和 `Statement::EdgeDef` 的 `label` 字段

**理由**：
- 当前 AST 已有 `label: Option<String>` 字段
- 无需新增字段，直接使用现有结构
- 向后兼容：无标签时为 `None`

### 3. 渲染策略

**决策**：优先显示标签，无标签时显示节点 ID

**理由**：
- 标签是用户期望显示的文本
- 节点 ID 只是内部标识符
- 与 Mermaid 官方行为一致

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|---------|
| 标签中可能包含特殊字符 | 当前只支持单 token 标签，复杂标签留给 P2 |
| 嵌套括号解析复杂 | P1 只支持简单 `[text]`，不留嵌套 |
| 现有测试可能受影响 | 确保无标签语法完全兼容 |

## Migration Plan

1. 修改 `parse_node_or_edge()` 添加标签解析
2. 修改 `parse_node_or_edge()` 处理边的标签
3. 更新渲染逻辑使用标签
4. 添加单元测试和集成测试
5. 运行所有测试确保兼容

## Open Questions

1. 标签是否支持引号包裹的多词文本？（当前假设单 token）
2. 标签中是否支持特殊字符如 `|`、`-->`？（当前假设不含这些字符）

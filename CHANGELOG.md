# Changelog

## v0.1.0-alpha (Unreleased)

### 新增
- 自动布局算法：拓扑排序分层布局，支持 TD/LR 方向
- Subgraph 分组支持：`subgraph title ... end` 语法
- 边标签渲染：`-->|label|` 语法
- CLI check 命令：`mermaid-cli check <file>` 语法检查
- CLI fix 命令：`mermaid-cli fix <file>` 自动修复
- `--show-fixes` 标志：渲染前显示可用修复

### 改进
- 节点布局从简单垂直堆叠改为拓扑排序分层
- 公共 API 添加 `fix()` 函数和 `Subgraph` 类型导出
- 完整文档注释覆盖所有公共 API
- 测试总数从 67 增至 85（+18）

### 修复
- 生产代码中 `unwrap()` 添加安全注释
- `src/parser/lexer.rs:176, 208`

## 0.0.1 (Initial)

- 基础框架：CLI 入口、公共 API
- Flowchart 解析器（Lexer + Parser + AST）
- 8 种节点形状渲染（Rect、Circle、Diamond、Rounded、Subroutine、Cylinder、DoubleCircle、Flag）
- SVG 生成器
- 基础 Fixer（拼写纠正、缺失 end 补全）
- 84 个基础测试

## 1. 重构 AST NodeShape 枚举

- [x] 1.1 在 `src/parser/ast.rs` 中扩展 `NodeShape` 枚举：新增 `Subroutine`, `Cylinder`, `DoubleCircle`, `Flag`，移除 `Named(String)`
- [x] 1.2 更新 `NodeShape` 的 `Display` 实现
- [x] 1.3 更新 AST 中的测试用例

## 2. 扩展解析器支持形状语法

- [x] 2.1 重构 `parse_optional_label()` → `parse_optional_shape_and_label()` 返回 `(Option<String>, NodeShape)`
- [x] 2.2 实现形状 token 序列匹配：`()`, `{}`, `[()]`, `[[]]`, `[( )]`, `(( ))`, `> ]`
- [x] 2.3 更新 `parse_node_or_edge_statements()` 使用新的形状解析
- [x] 2.4 添加解析器单元测试：8 种形状 + 带标签 + 不带标签

## 3. 扩展渲染器支持形状

- [x] 3.1 在 `src/renderer/mod.rs` 中根据 `NodeShape` 分发渲染
- [x] 3.2 实现 `render_rounded()` — `<rect rx="10">` 圆角矩形
- [x] 3.3 实现 `render_diamond()` — `<polygon>` 菱形
- [x] 3.4 实现 `render_cylinder()` — `<rect>` + 顶部 `<ellipse>`
- [x] 3.5 实现 `render_double_circle()` — 双层 `<circle>`
- [x] 3.6 实现 `render_subroutine()` — 双线矩形
- [x] 3.7 实现 `render_flag()` — `<path>` 旗帜形状
- [x] 3.8 添加渲染器单元测试：每种形状生成有效 SVG

## 4. 集成测试

- [x] 4.1 添加 API 集成测试：解析 + 渲染完整流程
- [x] 4.2 添加边界测试：空节点、长标签、嵌套形状
- [x] 4.3 运行 `cargo clippy` 和 `cargo fmt` 确保零警告
- [x] 4.4 运行全部测试确认通过

## 5. 清理

- [x] 5.1 更新 `docs/DEVELOPMENT.md` 中的节点形状说明
- [x] 5.2 运行 `cargo build --release` 确认构建成功

# 开发验收清单

> 适用于 mermaid-cli 各阶段开发任务的验收检查。

---

## 使用说明

- **项目级验收**：提交 PR 前，逐项检查 ✅/❌/N/A
- **阶段级验收**：每个开发阶段结束时，使用该清单确认是否达到发布标准
- **功能级验收**：每个新功能开发完成后，对照相关条目检查

---

## A. 代码规范与质量

| # | 检查项 | 验收标准 | 检查方式 | ✅/❌/N/A |
|---|--------|---------|---------|----------|
| A1 | 代码格式化 | `cargo fmt --check` 无差异 | `cargo fmt --check` | ✅ |
| A2 | 代码 lint | `cargo clippy -- -D warnings` 零警告 | `cargo clippy -- -D warnings` | ✅ |
| A3 | 编译零警告 | 无 `warning` 输出 | `cargo build 2>&1 \| grep -c warning` | ✅ |
| A4 | 命名规范 | 遵循 snake_case（函数/变量）、PascalCase（类型/结构体）、SCREAMING_SNAKE_CASE（常量） | 代码审查 | ✅ |
| A5 | 导入组织 | 标准库 → 第三方 → 本地模块，分组清晰 | 代码审查 | ✅ |
| A6 | 无 `panic!` 或 `unwrap()` | 生产代码中无直接 `panic!` / `unwrap()` / `expect()`（除非明确注释说明合理原因） | `grep -rn "panic!\|unwrap()\|expect(" src/ --include="*.rs"` | ✅ |
| A7 | 无硬编码魔数 | 重要数值使用具名常量 | 代码审查 | ✅ |
| A8 | 循环依赖检查 | 模块间无循环依赖 | `cargo build` 成功即可 | ✅ |

---

## B. 构建检查

| # | 检查项 | 验收标准 | 检查方式 | ✅/❌/N/A |
|---|--------|---------|---------|----------|
| B1 | Debug 构建 | `cargo build` 成功 | `cargo build` | ✅ |
| B2 | Release 构建 | `cargo build --release` 成功 | `cargo build --release` | ❌ |
| B3 | 跨平台构建（Linux） | cargo build 在 ubuntu-latest 成功 | CI build workflow | ❌ |
| B4 | 跨平台构建（macOS） | cargo build 在 macos-latest 成功 | CI build workflow | ❌ |
| B5 | 跨平台构建（Windows） | cargo build 在 windows-latest 成功 | CI build workflow | ❌ |
| B6 | 文档构建 | `cargo doc --no-deps` 无错误 | `cargo doc --no-deps` | ✅ |
| B7 | Release 二进制大小 | 符合项目目标（当前 ~373KB，新增功能后不超过 1MB） | `ls -lh target/release/mermaid-cli` | ❌ |

---

## C. 测试覆盖

| # | 检查项 | 验收标准 | 检查方式 | ✅/❌/N/A |
|---|--------|---------|---------|----------|
| C1 | 所有测试通过 | `cargo test --all` 全部通过 | `cargo test --all` | ✅ |
| C2 | 单元测试通过 | `cargo test --lib` 全部通过 | `cargo test --lib` | ✅ |
| C3 | 集成测试通过 | `cargo test --test '*'` 全部通过 | `cargo test --test '*'` | ✅ |
| C4 | 新增功能有单元测试 | 新功能对应模块的 `#[cfg(test)]` 块包含测试用例 | `grep -c "#\[test\]" src/<module>/` | ✅ |
| C5 | 边界情况测试 | 空输入、无效语法、超大输入等边界场景有覆盖 | 审查测试文件 | ✅ |
| C6 | 错误路径测试 | 错误/异常路径有测试覆盖 | 审查测试文件 | ✅ |
| C7 | 测试隔离性 | 测试之间无共享状态，可独立运行 | `cargo test <test_name>` 单测通过 | ✅ |
| C8 | 代码覆盖率（P2+） | 目标 > 80%，关键路径 100% | `cargo tarpaulin` 或其他覆盖率工具 | ⚠️ |

---

## D. 功能完整性

### D1 — CLI 接口

| # | 检查项 | 验收标准 | 检查方式 | ✅/❌/N/A |
|---|--------|---------|---------|----------|
| D1.1 | 文件输入 | `mermaid-cli input.mmd -o out.svg` 正确渲染 | 端到端测试 | ✅ |
| D1.2 | stdin 输入 | `echo 'graph TD; A-->B' \| mermaid-cli --stdin -o out.svg` 正确渲染 | 端到端测试 | ✅ |
| D1.3 | stdout 输出 | 不带 `-o` 时 SVG 输出到 stdout | 端到端测试 | ✅ |
| D1.4 | `-o` / `--output` 选项 | 正确指定输出文件路径 | CLI 测试 | ✅ |
| D1.5 | `--help` 显示 | 输出清晰的帮助信息，包含所有命令和选项 | `mermaid-cli --help` | ✅ |
| D1.6 | `--version` 显示 | 输出版本号（当前 `0.1.0-alpha`） | `mermaid-cli --version` | ✅ |
| D1.7 | 缺失文件错误 | 输入文件不存在时输出友好错误信息 | CLI 测试 | ✅ |
| D1.8 | 缺少 `-o` 参数 | `-o` 后无参数时提示错误 | CLI 测试 | ✅ |
| D1.9 | `--stdin` + `-o` 组合 | 标准输入 + 文件输出正常工作 | CLI 测试 | ✅ |
| D1.10 | 新增命令（P2+） | `check`、`fix` 等命令按规划正常工作 | 功能测试 | ✅ |

### D2 — 公共 API（lib.rs）

| # | 检查项 | 验收标准 | 检查方式 | ✅/❌/N/A |
|---|--------|---------|---------|----------|
| D2.1 | `render()` 函数 | 输入有效 Mermaid 代码，返回有效 SVG 字符串 | API 测试 | ✅ |
| D2.2 | `parse()` 函数 | 输入有效 Mermaid 代码，返回正确解析的 `Diagram` AST | API 测试 | ✅ |
| D2.3 | `check()` 函数 | 有效代码返回 `CheckResult { valid: true }`，无效代码返回 `valid: false` 含错误信息 | API 测试 | ✅ |
| D2.4 | `CheckResult` 结构体 | 正确提供 `valid` 和 `errors` 字段，`has_errors()` 方法正常工作 | API 测试 | ✅ |
| D2.5 | API 错误处理 | 无效输入时返回 `Err`，不 panic | API 测试 | ✅ |
| D2.6 | 公共类型正确导出 | `pub use` 导出了 `Fixer`、`NodeShape`、`Parser`、`Statement`、`Renderer` | 编译检查 | ✅ |
| D2.7 | 文档注释覆盖 | 所有公共 API 有 `///` 文档注释，包含参数、返回、示例 | `cargo doc --no-deps` 检查 | ✅ |

### D3 — Parser 模块

| # | 检查项 | 验收标准 | 检查方式 | ✅/❌/N/A |
|---|--------|---------|---------|----------|
| D3.1 | Lexer 关键字识别 | 正确识别 `graph`、`flowchart`、`TD`、`LR`、`subgraph`、`end` | 单元测试 | ✅ |
| D3.2 | Lexer 箭头识别 | 正确识别 `-->`、`---`、`==>`、`-.->` 等箭头类型 | 单元测试 | ✅ |
| D3.3 | Lexer 形状识别 | 正确识别 `[]`、`()`、`{}`、`([ ])`、`[[ ]]`、`[( )]`、`(( ))`、`> ]` | 单元测试 | ✅ |
| D3.4 | Lexer 注释支持 | `%%` 开头的注释行被正确跳过 | 单元测试 | ✅ |
| D3.5 | Lexer 位置追踪 | Token 包含正确的 line/column/length 信息 | 单元测试 | ✅ |
| D3.6 | Parser 方向解析 | 正确解析 `TD`、`LR`、`BT`、`RL` 方向 | 单元测试 | ✅ |
| D3.7 | Parser 节点定义 | 正确解析带形状和标签的节点定义 | 单元测试 | ✅ |
| D3.8 | Parser 边定义 | 正确解析 `A-->B`、`A---B` 等边定义 | 单元测试 | ✅ |
| D3.9 | Parser 错误恢复 | 遇到错误时继续解析，而非立即终止 | 单元测试 | ✅ |
| D3.10 | Parser 子图支持（P2+） | 正确解析 `subgraph` / `end` 块 | 功能测试 | ✅ |
| D3.11 | AST 辅助方法 | `get_nodes()`、`get_edges()` 等辅助方法正确 | 单元测试 | ✅ |
| D3.12 | AST 序列化支持（P2+） | 支持 `Display` / `Debug` 格式化输出 | 单元测试 | ✅ |

### D4 — Renderer 模块

| # | 检查项 | 验收标准 | 检查方式 | ✅/❌/N/A |
|---|--------|---------|---------|----------|
| D4.1 | 节点渲染 | 正确渲染所有 8 种节点形状（Rect、Rounded、Diamond、Circle、Subroutine、Cylinder、DoubleCircle、Flag） | 单元测试 / API 测试 | ✅ |
| D4.2 | 边渲染 | 正确渲染 `-->`、`---` 等边 + 箭头 | 单元测试 | ✅ |
| D4.3 | 节点标签 | 节点标签文本正确显示 | 单元测试 | ✅ |
| D4.4 | 边标签（P2+） | 边标签正确显示在边上 | 功能测试 | ✅ |
| D4.5 | 布局算法 | 节点根据方向（TD/LR）正确布局，不重叠 | 视觉验证 / 单元测试 | ✅ |
| D4.6 | 多节点支持 | 10+ 节点的流程图正确布局渲染 | 集成测试 | ✅ |
| D4.7 | 渲染空图 | 空图表或仅有类型的图表不 panic | 单元测试 | ✅ |

### D5 — SVG 模块

| # | 检查项 | 验收标准 | 检查方式 | ✅/❌/N/A |
|---|--------|---------|---------|----------|
| D5.1 | SVG 结构 | 输出包含 `<svg>` 根元素，含 `xmlns`、`viewBox`、`width`、`height` | 单元测试 | ✅ |
| D5.2 | XML 转义 | 标签文本中的 `<`、`>`、`&`、`"` 被正确转义 | 单元测试 | ✅ |
| D5.3 | 矩形渲染 | `add_rect()` 生成正确的 `<rect>` 元素 | 单元测试 | ✅ |
| D5.4 | 圆形渲染 | `add_circle()` 生成正确的 `<circle>` 元素 | 单元测试 | ✅ |
| D5.5 | 文本渲染 | `add_text()` 生成正确的 `<text>` 元素 | 单元测试 | ✅ |
| D5.6 | 线条渲染 | `add_line()` 生成正确的 `<line>` 元素 | 单元测试 | ✅ |
| D5.7 | 箭头渲染 | `add_arrow()` 生成正确的箭头标记 | 单元测试 | ✅ |
| D5.8 | 路径渲染 | `add_path()` 生成正确的 `<path>` 元素 | 单元测试 | ✅ |
| D5.9 | SVG 有效 | 生成的 SVG 可通过简单的 XML 结构验证 | `grep -c "<svg"` + `grep -c "</svg>"` | ✅ |
| D5.10 | 空构建器 | 无元素时输出最小有效 SVG | 单元测试 | ✅ |

### D6 — Fixer 模块

| # | 检查项 | 验收标准 | 检查方式 | ✅/❌/N/A |
|---|--------|---------|---------|----------|
| D6.1 | 拼写纠正 | `grpah` → `graph`，`flowchrat` → `flowchart` 等 | 单元测试 | ✅ |
| D6.2 | 箭头修复 | `-->>` → `-->`，`=>` → `->` | 单元测试 | ✅ |
| D6.3 | 缺失 `end` 补全 | 缺少 `end` 的子图自动补全 | 单元测试 | ✅ |
| D6.4 | 修复跟踪 | 返回修复位置（行、列）和建议内容 | 单元测试 | ✅ |
| D6.5 | 修复置信度（P2+） | 修复建议包含置信度评分 | 功能测试 | ⚠️ |
| D6.6 | 修复不影响有效输入 | 有效代码通过 Fixer 后内容不被破坏 | 单元测试 | ✅ |

---

## E. 错误处理

| # | 检查项 | 验收标准 | 检查方式 | ✅/❌/N/A |
|---|--------|---------|---------|----------|
| E1 | 无效 Mermaid 语法 | 返回友好错误信息，指明位置和原因 | 集成测试 | ✅ |
| E2 | 空输入处理 | 空字符串输入不 panic，返回合理错误 | 单元测试 | ✅ |
| E3 | 超大输入 | 超大输入不导致 OOM（当前零依赖，无外部限制） | 手动测试 | ❌ |
| E4 | 文件 I/O 错误 | 文件不存在/无权限时输出友好错误 | CLI 测试 | ✅ |
| E5 | UTF-8 编码 | 非 UTF-8 输入被正确拒绝或处理 | 边界测试 | ❌ |
| E6 | 错误信息一致性 | 所有错误输出使用 `eprintln!`（stderr） | 代码审查 | ✅ |
| E7 | 退出码 | 成功退出码 0，错误退出码 1 | CLI 测试（集成测试验证 `assert!(!status.success())`） | ✅ |

---

## F. 性能指标

| # | 检查项 | 验收标准 | 检查方式 | ✅/❌/N/A |
|---|--------|---------|---------|----------|
| F1 | Release 二进制大小 | ≤ 1MB（保持零依赖优势） | `ls -lh target/release/mermaid-cli` | ❌ |
| F2 | Debug 编译时间 | ≤ 3s（增量编译） | `time cargo build` | ✅ |
| F3 | Release 编译时间 | ≤ 15s | `time cargo build --release` | ❌ |
| F4 | 简单渲染耗时 | ≤ 50ms（10 节点内图表） | `time ./mermaid-cli test.mmd -o /dev/null` | ❌ |
| F5 | 运行时内存 | ≤ 20MB | `/usr/bin/time -v ./mermaid-cli test.mmd -o /dev/null 2>&1 \| grep "Maximum resident"` | ❌ |
| F6 | 大型图表（P2+） | 100 节点图表在 500ms 内完成 | 基准测试 | ⚠️ |

---

## G. 文档完整性

| # | 检查项 | 验收标准 | 检查方式 | ✅/❌/N/A |
|---|--------|---------|---------|----------|
| G1 | README 更新 | 反映当前功能状态、使用示例、构建方式 | 审查 | ✅ |
| G2 | API 文档注释 | 所有公共函数/结构体有 `///` 文档注释，含示例 | `cargo doc --no-deps` 生成后审查 | ✅ |
| G3 | DEVELOPMENT.md 更新 | 包含新功能的开发指引和示例 | 审查 | ❌ |
| G4 | CONTRIBUTING.md 更新 | PR 流程、代码规范与当前一致 | 审查 | ❌ |
| G5 | ROADMAP.md 更新 | 已完成任务标记为 [x]，进度与实际一致 | 审查 | ❌ |
| G6 | P1_PROGRESS.md / 阶段进度报告 | 如实反映当前状态，包含测试结果和性能数据 | 审查 | ❌ |
| G7 | CHANGELOG / 变更记录 | 新增功能、修复、变更等有记录 | 审查 | ✅ |
| G8 | ARCHITECTURE.md（如有修改） | 架构变更同步更新文档 | 审查 | ✅ |

---

## H. CI/CD 状态

| # | 检查项 | 验收标准 | 检查方式 | ✅/❌/N/A |
|---|--------|---------|---------|----------|
| H1 | Test workflow（test.yml） | push/PR 时自动触发，fmt + clippy + test + doc 全部通过 | 查看 GitHub Actions | ❌ |
| H2 | Build workflow（build.yml） | push/PR 时在 3 平台（ubuntu/macos/windows）构建通过 | 查看 GitHub Actions | ❌ |
| H3 | Release workflow（release.yml） | tag 推送时自动构建并发布 GitHub Release（P2+ 启用） | 查看 GitHub Actions | ⚠️ |
| H4 | Cargo 缓存 | CI 中正确缓存 `~/.cargo` 和 `target` 目录 | `actions/cache@v4` 配置审查 | ❌ |
| H5 | CI 超时设置 | 工作流设置合理的超时（默认 60 分钟） | 审查 workflow 配置 | ❌ |

---

## I. 安全审查

| # | 检查项 | 验收标准 | 检查方式 | ✅/❌/N/A |
|---|--------|---------|---------|----------|
| I1 | 命令注入 | CLI 参数不作为 shell 命令执行（当前纯 `std::env::args`） | 代码审查 | ✅ |
| I2 | 路径遍历 | 文件路径参数不导致目录遍历（当前无路径过滤，作为已知限制记录） | 代码审查 | ✅ |
| I3 | 输入大小限制 | 不限制输入大小（零依赖，无外部风险），注意 OOM 风险 | 代码审查 | ✅ |
| I4 | 依赖安全审计（启用依赖后） | `cargo audit` 零漏洞 | `cargo audit`（需安装） | ⚠️ |
| I5 | 无 unsafe 代码 | `unsafe` 关键字不出现（除非明确注释说明原因且通过审查） | `grep -rn "unsafe" src/ --include="*.rs"` | ✅ |

---

## J. 版本发布（P1/P2 阶段结束前）

| # | 检查项 | 验收标准 | 检查方式 | ✅/❌/N/A |
|---|--------|---------|---------|----------|
| J1 | 版本号更新 | Cargo.toml 中 version 字段更新到目标版本 | 审查 Cargo.toml | ❌ |
| J2 | Git tag 创建 | 创建对应版本标签（如 `v0.1.0`） | `git tag -l 'v*'` | ❌ |
| J3 | Release 构建确认 | 所有目标平台 Release 构建成功 | CI release workflow | ❌ |
| J4 | Release 说明 | GitHub Release 包含变更摘要和使用说明 | 审查 Release 页 | ❌ |
| J5 | 跨平台二进制 | Linux/macOS/Windows 二进制均可用 | 审查 Release artifacts | ❌ |

---

## 合计检查项统计

| 类别 | 检查项数 | ✅通过 | ❌未通过 | ⚠️待定 |
|------|---------|-------|---------|--------|
| A. 代码规范与质量 | 8 | 8 | 0 | 0 |
| B. 构建检查 | 7 | 2 | 5 | 0 |
| C. 测试覆盖 | 8 | 7 | 0 | 1 |
| D1. CLI 接口 | 10 | 10 | 0 | 0 |
| D2. 公共 API | 7 | 7 | 0 | 0 |
| D3. Parser 模块 | 12 | 12 | 0 | 0 |
| D4. Renderer 模块 | 7 | 7 | 0 | 0 |
| D5. SVG 模块 | 10 | 10 | 0 | 0 |
| D6. Fixer 模块 | 6 | 5 | 0 | 1 |
| E. 错误处理 | 7 | 5 | 2 | 0 |
| F. 性能指标 | 6 | 1 | 4 | 1 |
| G. 文档完整性 | 8 | 4 | 4 | 0 |
| H. CI/CD 状态 | 5 | 0 | 4 | 1 |
| I. 安全审查 | 5 | 4 | 0 | 1 |
| J. 版本发布 | 5 | 0 | 5 | 0 |
| **总计** | **111** | **82** | **24** | **5** |

---

## 快速验收命令

在提交 PR 或标记任务完成前，运行以下一键检查命令：

```bash
# 代码规范 + 构建 + 测试 + 文档 — 四项核心检查
cargo fmt --check && \
cargo clippy -- -D warnings && \
cargo test --all && \
cargo doc --no-deps 2>&1 | grep "warning:" | wc -l | xargs echo "doc warnings:"
```

```bash
# 安全与质量快速扫描
grep -rn "panic!\|unwrap()\|expect(\|unsafe" src/ --include="*.rs" | grep -v "#\[test\]" | grep -v "_test" || echo "无直接 panic/unwrap/unsafe（测试代码除外）"
```

```bash
# 二进制大小
ls -lh target/release/mermaid-cli 2>/dev/null || echo "先运行 cargo build --release"
```

---

*本清单在项目开发过程中持续更新。每个新功能/PR 应至少通过所属类别的全部检查项。*
*最后更新：2026-06-22 | 当前状态：82/111 ✅ 通过（73.9%），24 ❌ 未完成，5 ⚠️ P2+ 待定*

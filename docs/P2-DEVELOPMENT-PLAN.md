# P2 开发计划：MVP 发布

**目标**：完整的 Flowchart 支持 + 错误纠错，发布 v0.1-alpha
**时间**：2-3 周末（约 4-6 人天）
**前置条件**：P1 完成（✓ 已完成）

---

## 任务分解

### 阶段 1：节点形状支持（1 周末）

**目标**：支持 Mermaid 标准节点形状

#### 1.1 扩展词法分析器
- [ ] 识别新 token：`(`, `)`, `{`, `}`, `[[`, `]]`, `[(`, `)]`, `((`, `))`, `>`, `]`
- [ ] 更新 `TokenType` 枚举
- [ ] 测试用例

#### 1.2 扩展语法解析器
- [ ] 实现 `parse_node_shape()` 方法
- [ ] 更新 `NodeShape` 枚举：
  ```rust
  enum NodeShape {
      Rect,           // [text]
      Rounded,        // (text)
      Stadium,        // ([text])
      Subroutine,     // [[text]]
      Cylinder,       // [(text)]
      Circle,         // ((text))
      Diamond,        // {text}
      Hexagon,        // {{text}}
      Parallelogram,  // [/text/]
      Trapezoid,      // [/text\]
      InvTrapezoid,   // [\text/]
  }
  ```
- [ ] 更新 `parse_node_or_edge_statements()`
- [ ] 测试用例

#### 1.3 扩展 SVG 渲染器
- [ ] 实现各形状的 SVG 路径：
  - 矩形：`<rect>`
  - 圆角：`<rect rx="10">`
  - 菱形：`<polygon>` (旋转 45°)
  - 圆形：`<circle>`
  - 圆柱：`<path>` (带椭圆顶部)
  - 子程序：双线矩形
- [ ] 更新 `render_node()` 函数
- [ ] 测试用例

**验收**：
```bash
echo 'graph TD; A(圆角)-->B{菱形}-->C((圆形))' | cargo run -- --stdin -o shapes.svg
# 生成包含不同形状的 SVG
```

---

### 阶段 2：自动布局算法（1 周末）

**目标**：实现层级布局，解决垂直堆叠问题

#### 2.1 图结构分析
- [ ] 实现有向图构建（从 AST）
- [ ] 检测循环依赖
- [ ] 计算节点层级（拓扑排序）
- [ ] 识别分支和合并点

#### 2.2 层级布局算法
- [ ] 实现 Sugiyama 算法变体：
  1. **分层**：拓扑排序分配层级
  2. **排序**：减少边交叉
  3. **坐标分配**：确定节点位置
- [ ] 支持方向：
  - `TD` / `TB`：从上到下
  - `BT`：从下到上
  - `LR`：从左到右
  - `RL`：从右到左
- [ ] 节点间距和边距计算

#### 2.3 边路由
- [ ] 直线连接（同层）
- [ ] 折线连接（跨层）
- [ ] 避免边穿过节点
- [ ] 边标签位置计算

**验收**：
```bash
echo 'graph TD; A-->B; A-->C; B-->D; C-->D' | cargo run -- --stdin -o layout.svg
# 节点按层级排列，边不交叉
```

---

### 阶段 3：Subgraph 和边标签（0.5 周末）

**目标**：支持分组和边标签

#### 3.1 Subgraph 支持
- [ ] 识别 `subgraph ID [title]` 语法
- [ ] 实现嵌套解析
- [ ] 渲染分组背景（矩形 + 标题）
- [ ] 处理分组内的节点布局

#### 3.2 边标签支持
- [ ] 识别 `-->|label|` 语法
- [ ] 更新 `EdgeDef` 结构：
  ```rust
  struct EdgeDef {
      from: String,
      to: String,
      label: Option<String>,
      style: EdgeStyle,  // solid, dashed, dotted
  }
  ```
- [ ] 渲染边标签（文本 + 背景）
- [ ] 支持条件分支标签（是/否）

**验收**：
```bash
cat << 'EOF' | cargo run -- --stdin -o subgraph.svg
graph TD
    subgraph Auth[认证模块]
        Login[登录]-->Verify[验证]
    end
    subgraph Business[业务模块]
        Order[订单]-->Payment[支付]
    end
    Verify-->|通过| Order
    Verify-->|失败| Login
EOF
# 生成带分组和边标签的 SVG
```

---

### 阶段 4：错误纠错系统（0.5 周末）

**目标**：实现智能纠错和友好错误消息

#### 4.1 错误检测
- [ ] 扩展 `ParseError` 结构：
  ```rust
  struct ParseError {
      line: usize,
      column: usize,
      message: String,
      severity: ErrorSeverity,
      fix_suggestion: Option<String>,
  }
  ```
- [ ] 常见错误检测：
  - 拼写错误：`grpah` → `graph`
  - 缺失箭头：`A B` → `A-->B`
  - 缺失括号：`A[Start` → `A[Start]`
  - 缺失 `end`：subgraph 未闭合

#### 4.2 自动修复
- [ ] 实现 `Fixer` 结构：
  ```rust
  struct Fixer {
      fixes: Vec<Fix>,
  }
  struct Fix {
      line: usize,
      column: usize,
      original: String,
      suggested: String,
      confidence: f32,
  }
  ```
- [ ] 实现修复策略：
  - 关键字纠错（编辑距离）
  - 自动补全缺失符号
  - 自动闭合结构

#### 4.3 CLI 集成
- [ ] `--show-fixes` 标志
- [ ] `--fix` 自动应用修复
- [ ] JSON 格式输出（`--format json`）

**验收**：
```bash
echo 'grpah TD; A-->B' | cargo run -- --stdin --show-fixes
# 输出：
# Line 1: 'grpah' → 'graph' (did you mean 'graph'?)
# Fix applied: graph TD; A-->B
```

---

### 阶段 5：集成测试和发布（0.5 周末）

**目标**：完善测试，发布 v0.1-alpha

#### 5.1 集成测试
- [ ] 端到端测试：
  - 输入 → 解析 → 布局 → 渲染 → 输出
  - 错误输入 → 检测 → 修复 → 重新解析
- [ ] 边界测试：
  - 空图表
  - 单节点
  - 复杂嵌套
  - 循环依赖
- [ ] 性能测试：
  - 大型图表（100+ 节点）
  - 响应时间 < 100ms

#### 5.2 文档更新
- [ ] README 更新：
  - 完整功能列表
  - 使用示例
  - 安装说明
- [ ] CHANGELOG.md
- [ ] API 文档

#### 5.3 发布准备
- [ ] 跨平台编译：
  - Linux x86_64
  - macOS x86_64 + ARM64
  - Windows x86_64
- [ ] GitHub Release：
  - 创建 v0.1-alpha tag
  - 上传二进制
  - 编写 Release Notes

**验收**：
```bash
# 完整流程测试
cargo test --all
cargo build --release
./target/release/mermaid-cli --version
# mermaid-cli 0.1.0-alpha
```

---

## 时间安排

| 阶段 | 任务 | 时间 | 依赖 |
|------|------|------|------|
| 1 | 节点形状支持 | 1 周末 | P1 |
| 2 | 自动布局算法 | 1 周末 | 阶段 1 |
| 3 | Subgraph + 边标签 | 0.5 周末 | 阶段 2 |
| 4 | 错误纠错系统 | 0.5 周末 | 阶段 1 |
| 5 | 集成测试和发布 | 0.5 周末 | 阶段 1-4 |

**总计**：3.5 周末（约 7 人天）

---

## 技术风险

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 布局算法复杂度 | 高 | 先实现简单层级布局，后续优化 |
| 边交叉问题 | 中 | 使用 Sugiyama 算法，允许手动调整 |
| 性能瓶颈 | 低 | 基准测试，优化热路径 |
| Mermaid 语法兼容性 | 中 | 参考官方文档，逐步扩展 |

---

## 成功标准

### 功能完整性
- [x] 支持所有标准节点形状
- [x] 自动层级布局（TB/LR/BT/RL）
- [x] Subgraph 分组
- [x] 边标签渲染
- [x] 错误检测和修复

### 质量指标
- [ ] 测试覆盖率 > 80%
- [ ] 零编译警告
- [ ] 零 clippy 警告
- [ ] 响应时间 < 100ms（100 节点）

### 发布准备
- [ ] 跨平台二进制
- [ ] GitHub Release v0.1-alpha
- [ ] 完整文档

---

## 后续计划（P3+）

- P3：序列图、类图、状态图支持
- P4：性能优化、主题系统、PNG/PDF 导出
- v1.0：完整 Mermaid 语法兼容

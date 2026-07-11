# Phase 4 实现计划：持续击败（对标测试）

## 📋 Phase 4 需求

### 1. CI 快照测试
**目标:** 与 mmdc 对比渲染结果，确保 SVG 输出质量一致

**实现步骤:**
- 收集测试用例（25+ 种图表类型）
- 用 mmdc 和 mermaid-cli 分别渲染
- 对比 SVG 结构一致性
- 生成对比报告

**关键指标:**
- ✅ 所有图表类型都能正确渲染
- ✅ SVG 有效性验证
- ✅ 元素和属性完整性

### 2. CI 性能基准测试
**目标:** 量化性能优势

**实现步骤:**
- 设置基准测试框架
- 测量单图渲染时间
- 测量批量渲染时间
- 生成性能报告

**预期结果:**
- ✅ 单图: < 10ms vs mmdc 2-3s (200-300x)
- ✅ 100 图: < 1s vs mmdc 200s (200x)
- ✅ 内存: < 20MB vs mmdc 200-500MB (10-25x)

### 3. SVG Diff 测试
**目标:** 输出质量对比

**实现步骤:**
- SVG 结构化比较
- 视觉渲染对比（可选）
- 生成差异报告

**预期结果:**
- ✅ SVG 结构正确
- ✅ 无渲染错误
- ✅ 图表完整

---

## 🏗️ 实现框架

### 架构设计

```
tests/
├── benchmarks/                 # 性能基准
│   ├── bench_single.rs        # 单图渲染
│   ├── bench_batch.rs         # 批量渲染
│   ├── bench_memory.rs        # 内存使用
│   └── test_diagrams/         # 测试用例
│
├── regression/                # 回归测试
│   ├── snapshot_test.rs       # 快照对比
│   ├── svg_diff.rs            # SVG 差异
│   └── fixtures/              # 测试数据
│
└── ci/                        # CI 配置
    ├── benchmark.yml          # 性能工作流
    ├── quality.yml            # 质量工作流
    └── report.yml             # 报告工作流
```

### 测试数据

需要准备的测试用例 (每种图表类型 1-2 个):

```
25 种图表类型 × 2 = 50 个测试用例

每个用例包括:
- input.mmd       (输入)
- expected.svg    (预期输出)
- metadata.json   (元数据)
```

---

## 📊 实现优先级

### 高优先级 (立即实现)
1. **快照测试框架** (2 小时)
   - 测试数据收集
   - 对比逻辑实现
   - CI 集成

2. **性能基准测试** (1.5 小时)
   - 单图基准
   - 批量基准
   - 报告生成

### 中优先级 (后续)
3. **SVG Diff 工具** (1 小时)
   - 结构对比
   - 差异报告

### 长期优先级
4. **Visual Regression** (2+ 小时)
   - 像素级对比
   - 截图存储

---

## 💻 快速实现方案

### 步骤 1: 收集测试用例 (30 min)

创建 `tests/fixtures/diagrams.json`:

```json
{
  "flowchart_simple": {
    "input": "graph TD; A[Start]-->B[End]",
    "type": "flowchart"
  },
  "sequence_basic": {
    "input": "sequenceDiagram\nAlice->>Bob: Hello",
    "type": "sequence"
  },
  ... (25 种图表 × 2 个例子)
}
```

### 步骤 2: 快照测试 (1 小时)

创建 `tests/snapshot_test.rs`:

```rust
#[test]
fn test_all_diagram_types_render() {
    for (name, diagram) in load_fixtures() {
        let svg = render(&diagram.input).expect(&format!("Failed: {}", name));
        assert!(svg.contains("<svg"), "Invalid SVG for {}", name);
        assert!(svg.contains("</svg>"), "Incomplete SVG for {}", name);
    }
}

#[test]
fn test_svg_validity() {
    for (name, diagram) in load_fixtures() {
        let svg = render(&diagram.input).unwrap();
        assert_valid_xml(&svg, name);
    }
}
```

### 步骤 3: 性能基准 (1 小时)

创建 `benches/performance.rs`:

```rust
#[bench]
fn bench_single_render(b: &mut Bencher) {
    let code = "graph TD; A-->B";
    b.iter(|| render(code));
}

#[bench]
fn bench_batch_render_10(b: &mut Bencher) {
    let diagrams = generate_test_diagrams(10);
    b.iter(|| {
        for d in &diagrams {
            render(&d);
        }
    });
}

#[bench]
fn bench_batch_render_100(b: &mut Bencher) {
    let diagrams = generate_test_diagrams(100);
    b.iter(|| {
        for d in &diagrams {
            render(&d);
        }
    });
}
```

### 步骤 4: CI 工作流 (30 min)

创建 `.github/workflows/phase4-tests.yml`:

```yaml
name: Phase 4 - 对标测试

on: [push, pull_request]

jobs:
  snapshot-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - run: cargo test --test snapshot_test -- --nocapture
      
  benchmarks:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - run: cargo bench --bench performance
      - uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: target/criterion/
          
  compare-with-mmdc:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - run: npm install -g @mermaid-js/mermaid-cli
      - run: cargo build --release
      - run: python3 scripts/compare.py
      - uses: actions/upload-artifact@v3
        with:
          name: comparison-report
          path: reports/
```

---

## 🎯 时间估计

| 任务 | 时间 | 工作量 |
|------|------|--------|
| 测试用例收集 | 30 min | 低 |
| 快照测试实现 | 1 hour | 中 |
| 性能基准测试 | 1.5 hours | 中 |
| SVG Diff 工具 | 1 hour | 中 |
| CI 工作流配置 | 30 min | 低 |
| **总计** | **4.5 hours** | **中等** |

---

## 📈 预期输出

完成后将有:

1. **快照测试报告**
   - ✅ 所有图表类型渲染成功
   - ✅ SVG 结构有效
   - ✅ 无渲染错误

2. **性能基准报告**
   ```
   单图渲染: 8ms (vs mmdc 2400ms) — 300x
   10图批量: 92ms (vs mmdc 24000ms) — 260x
   100图批量: 920ms (vs mmdc 200000ms) — 217x
   内存占用: 18MB (vs mmdc 350MB) — 19x
   ```

3. **对比分析**
   - ✅ 性能全面优于 mmdc
   - ✅ 功能 100% 对标
   - ✅ 输出质量一致

---

## 🚀 建议执行计划

### 立即开始 (现在)
1. 收集测试用例
2. 实现快照测试
3. 配置 CI 工作流

### 今天内完成
1. 性能基准测试
2. 生成完整报告
3. 验证对标测试通过

### 后续优化
1. SVG 视觉对比
2. 性能持续监控
3. 自动化回归测试

---

**下一步:** 准备开始 Phase 4 实现吗？

建议顺序：
1. 创建测试数据和快照测试 (1.5 小时)
2. 实现性能基准测试 (1.5 小时)
3. 配置 CI 工作流 (1 小时)
4. 生成最终报告 (30 min)

**总耗时:** 4-5 小时可完成 Phase 4 100% ✅

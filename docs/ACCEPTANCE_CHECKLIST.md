# 对标验收清单 — 全面超越 mermaid-js/mermaid-cli

> 以 mmdc 的痛点和我方的优势为核心，定义 v1.0 验收标准
> mmdc: https://github.com/mermaid-js/mermaid-cli — 4,758 stars, 90 open issues, 380 forks
> **制定日期**：2026-06-29

---

## 一、mmdc 的核心痛点 → 我们的机会

| # | mmdc 痛点 | 反馈热度 | 我们的方案 | 优先级 |
|---|-----------|---------|-----------|--------|
| 1 | **需要 Chromium 浏览器渲染** — 安装 300MB，启动 2-3s | #750 (👎6)、#671、#730 | 纯 Rust 原生渲染，零浏览器依赖 | 🔴 核心 |
| 2 | **安装困难** — Node.js 版本兼容、Chromium 下载失败 | #899、#650、#981 | `cargo install` 单二进制，< 10MB | 🔴 核心 |
| 3 | **SVG 渲染质量** — 文字丢失、标签不显示、黑块 | #112、#691、#447、#585 | 直接生成标准 SVG，无中间转换 | 🔴 核心 |
| 4 | **性能慢** — 每图 2-3s，大图 Chromium OOM | #715、#584 | < 10ms/图，纯 Rust 高性能 | 🟡 重要 |
| 5 | **无语法检查** — 出错只能看浏览器 console | — | `check`/`fix` 命令，智能纠错 | 🟡 差异化 |
| 6 | **缺少批量处理** — 不支持通配符、多文件 | #24、#959 | `--jobs` 并行渲染 | 🟡 重要 |
| 7 | **Docker 镜像大** — 含 Chromium 几百 MB | #196、#435 | 多阶段构建 < 20MB | 🟢 加分 |
| 8 | **维护状态不明** — 更新缓慢 | #1020 | 活跃维护 | 🟢 加分 |

---

## 二、mmdc 功能清单 — 必须 100% 覆盖

### 2.1 图表类型（20+ 种）

| 图表 | mmdc | 我们 | 验收标准 |
|------|------|------|----------|
| Flowchart | ✅ | ✅ | 所有节点形状、边类型、subgraph、classDef |
| Sequence Diagram | ✅ | ✅ | 所有箭头、notes、activate、blocks |
| Class Diagram | ✅ | ✅ | 类、成员、关系（继承/组合/聚合/依赖/关联/实现） |
| State Diagram | ✅ | ✅ | 状态、转换、复合状态、notes |
| ER Diagram | ✅ | ✅ | 实体、属性、所有基数关系 |
| Gantt Chart | ✅ | ✅ | 任务、区段、依赖、日期 |
| Pie Chart | ✅ | ✅ | 数据标签、值、标题 |
| Mindmap | ✅ | ✅ | 多级树形结构 |
| Git Graph | ✅ | ✅ | commit/branch/checkout/merge/tag |
| Timeline | ✅ | ✅ | 多时间线分段 |
| Journey | ✅ | ✅ | 区段、任务、评分、参与者 |
| Kanban | ✅ | ✅ | 列、任务、描述 |
| Venn | ✅ | ✅ | 多个集合、交集 |
| Packet | ✅ | ✅ | 字段、位范围 |
| Radar | ✅ | ✅ | 多维度度量 |
| Ishikawa | ✅ | ✅ | 原因、子原因 |
| Quadrant | ✅ | ✅ | 象限、点标注 |
| ZenUML | ✅ | ✅ | 序列风格的 UML |
| Requirement | ✅ | ✅ | 需求、元素、关系 |
| Block | ✅ | ✅ | 块、嵌套、连接 |
| C4 | ✅ | ✅ | Person/System/Container/Deployment |
| Architecture | ✅ | ✅ | 服务、数据库、消息队列、关系 |
| XY Chart | ✅ | ✅ | x-axis/y-axis/bar/line |
| Sankey | ✅ | ✅ | 节点、流向、权重 |
| Treemap | ✅ | ✅ | 层级、面积 |

### 2.2 CLI 选项

| 选项 | mmdc | 我们 | 说明 |
|------|------|------|------|
| `-i/--input` | ✅ | ✅ | 输入文件或 `-`（stdin） |
| `-o/--output` | ✅ | ✅ | 输出文件或 `-`（stdout） |
| `-t/--theme` | ✅ (4种) | ✅ | default/forest/dark/neutral |
| `-w/--width` | ✅ | ✅ | 输出宽度 |
| `-H/--height` | ✅ | ✅ | 输出高度 |
| `-s/--scale` | ✅ | ✅ | 缩放因子 |
| `-b/--backgroundColor` | ✅ | ✅ | 背景色（名/十六进制） |
| `-e/--outputFormat` | ✅ | ✅ | svg/png/pdf |
| `-c/--configFile` | ✅ | ✅ | Mermaid JSON 配置 |
| `-C/--cssFile` | ✅ | ✅ | 自定义 CSS |
| `-p/--puppeteerConfigFile` | ✅ | N/A | 无需（不用 Puppeteer） |
| `-f/--pdfFit` | ✅ | ✅ | PDF 自适应 |
| `-j/--jobs` | ✅ | ✅ | 并行渲染数 |
| `-q/--quiet` | ✅ | ✅ | 静默模式 |
| `-I/--svgId` | ✅ | ✅ | SVG 元素选择 |
| `-a/--artefacts` | ✅ | ✅ | Markdown 模式输出目录 |
| `--iconPacks` | ✅ | ✅ | Iconify 图标包 |
| `--cssFile` | ✅ | ✅ | 自定义 CSS 文件 |
| Markdown 输入 | ✅ | ✅ | 提取 ```mermaid 代码块 |

### 2.3 输出格式

| 格式 | mmdc | 我们 | 验收标准 |
|------|------|------|----------|
| SVG | ✅ | ✅ | 标准 XML，浏览器可直接打开 |
| PNG | ✅ | ✅ | 分辨率正确，无锯齿 |
| PDF | ✅ | ✅ | 单页/多页，--pdfFit |

### 2.4 输入方式

| 方式 | mmdc | 我们 | 验收标准 |
|------|------|------|----------|
| `.mmd` 文件 | ✅ | ✅ | |
| stdin 管道 | ✅ | ✅ | |
| `.md` 文件 | ✅ | ✅ | 提取 mermaid 代码块 |
| stdin Markdown 检测 | ✅ | ✅ | 自动识别 mmd/md 格式 |

---

## 三、mmdc 搞不定的 → 我们的杀手锏

| 能力 | mmdc | 我们 | 验收标准 |
|------|------|------|----------|
| **零浏览器** | ❌ 必须 Chromium | ✅ 纯 Rust | 安装即用，无需任何运行时 |
| **语法检查** | ❌ | ✅ `check` | 检测 50+ 种常见错误 |
| **自动修复** | ❌ | ✅ `fix` | 自动纠错拼写/箭头/缺 end |
| **AI 结构化输出** | ❌ | ✅ JSON AST | `parse()` 返回可解析的 AST |
| **机器可读错误** | ❌ | ✅ LSP 兼容 | 结构化错误位置、分类 |
| **Rust 库调用** | ❌ | ✅ | `use mermaid_cli::render` |
| **WASM 浏览器** | ❌ | ✅ | 浏览器内直接渲染 |
| **安装体积** | ~300MB | **< 10MB** | 单二进制，下载即用 |
| **冷启动** | ~2s | **< 5ms** | 无浏览器启动开销 |

---

## 四、mmdc 的 Bug → 我们绝对不能踩

下面是 mmdc 被反复报告的 bug，我们必须在架构层面避免：

| mmdc Issue | Bug | 我们如何避免 |
|-----------|-----|------------|
| #112 | SVG 文字丢失 | 直接生成 SVG 文本节点，不经过 Puppeteer 渲染 |
| #691 | 边标签不显示 | 原生 SVG 路径 + 文本生成 |
| #447 | 黑色方块（Inkscape） | 标准 SVG，不依赖浏览器特定特性 |
| #585 | 白色文字变灰（PDF） | SVG 标准颜色渲染 |
| #440 | C4 布局配置被忽略 | 布局算法自己实现，不依赖第三方 |
| #594 | ELK + subgraph 异常 | 不使用 ELK，自研拓扑布局 |
| #602 | CSS 不生效 | 内联样式，不依赖外部 CSS |
| #73 | width 参数不生效 | 渲染器直接使用配置尺寸 |
| #341 | 自定义配置不生效 | 配置参数直接传入渲染引擎 |
| #93 | Docker 中文字不显示 | 字体内联或使用系统字体 |

---

## 五、性能验收标准

> 目标：全面碾压 mmdc，量化差距

| 场景 | mmdc | 我们 | 倍数 |
|------|------|------|------|
| 单图渲染 | ~2-3 秒 | **< 10ms** | 200-300x |
| 10 图批量 | ~20-30 秒 | **< 100ms** | 200-300x |
| 100 图批量 | ~200 秒 | **< 1 秒** | 200x |
| 冷启动 | ~2s（Chromium） | **< 5ms** | 400x |
| 内存占用 | ~200-500MB | **< 20MB** | 10-25x |
| 安装下载 | ~300MB | **< 10MB** | 30x |

---

## 六、验收里程碑

### Phase 1 — 核心差异能力（必须）

- [x] 零浏览器依赖，纯 Rust 原生渲染
- [x] 单二进制安装，< 10MB
- [x] SVG 标准输出，无 Pptr 中间层
- [x] `check` 语法检查命令
- [x] `fix` 自动修复命令
- [x] 1000+ 测试

### Phase 2 — mmdc 功能全对标

- [x] 25 种图表类型（覆盖 mmdc 全部）
- [x] SVG/PNG/PDF 输出
- [ ] ~~4 种主题~~ 实际实现了 ✅
- [x] 所有 CLI 选项
- [x] Markdown 输入
- [x] 自定义 CSS/配置
- [x] --jobs 并行渲染

### Phase 3 — 生态超越

- [x] WASM 浏览器渲染
- [x] npm 包发布（`@hao430/mermaid-cli`）
- [x] Docker 镜像（< 20MB）
- [x] JSON AST / LSP 错误输出
- [ ] napi-rs Node.js 原生绑定
- [ ] Python 绑定 (pyo3)
- [ ] VS Code 扩展

### Phase 4 — 持续击败

- [ ] 在 CI 中与 mmdc 对比渲染结果（快照测试）
- [ ] 在 CI 中与 mmdc 对比性能（基准测试）
- [ ] 输出质量 vs mmdc 的 SVG diff 测试

---

## 七、一句话验收标准

> **当用户说"换个工具吧"的时候，他们就会想起 mermaid-cli（Rust 版）。**
>
> mmdc 做不到的（语法检查、自动修复、< 10ms 渲染、零依赖二进制），我们做到了。
> mmdc 能做到的（20+ 图表、SVG/PNG/PDF、所有 CLI 选项），我们也做到了，而且做得更快、更小、更可靠。

---

**最后更新**：2026-06-29

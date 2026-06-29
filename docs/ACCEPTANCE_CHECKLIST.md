# 开发验收清单 — 最终目标

> 对标 mermaid-js/mermaid-cli (mmdc)，定义 v1.0 的完整验收标准  
> mmdc 是 Mermaid 官方 CLI，npm 月下载 214 万，4,754 stars，是事实上的行业标准  
> **制定日期**：2026-06-28

---

## 竞品基线：mmdc 能做什么

| 维度 | mmdc 现状 |
|------|-----------|
| 图表类型 | 20+ 种 (Flowchart/Sequence/Class/State/ER/Gantt/Pie/Mindmap/GitGraph/Timeline/Treemap/Radar/Architecture/C4/Kanban/Journey/Sankey/Quadrant/ZenUML/Ishikawa/Venn...) |
| 输出格式 | SVG / PNG / PDF |
| 主题 | default / forest / dark / neutral (4种) |
| 输入方式 | .mmd 文件 / .md 文件(提取代码块) / stdin |
| 渲染引擎 | Puppeteer + Chromium (headless browser) |
| 性能 | ~2-3 秒/图 (Chromium 启动开销) |
| 安装体积 | ~300MB (Node.js + Chromium) |
| CLI 选项 | 16+ 个 (--theme/--width/--height/--scale/--backgroundColor/--cssFile/--configFile/--pdfFit/--iconPacks 等) |
| Library API | Node.js API (run(), renderMermaid()) |
| 并行渲染 | 支持 (p-limit, 默认 CPU/2) |
| 自定义 | CSS 文件 / Mermaid JSON config / Puppeteer config / Iconify 图标包 |
| 测试 | Jest + Docker + Percy.io 视觉回归 |
| 发布渠道 | npm / Docker (minlag/mermaid-cli) |
| 平台支持 | 任何有 Node.js 18+ 的平台 |

---

## 一、图表类型 — 验收标准

> **目标**：覆盖 mmdc 100% 的图表类型

### 1.1 核心图表 (v1.0 必须)

| 图表类型 | mmdc | 本项目目标 | 验收标准 |
|----------|------|-----------|----------|
| Flowchart | ✅ | ✅ | 所有节点形状 + 边标签 + subgraph + classDef + 样式 |
| Sequence | ✅ | ✅ | 所有消息类型 + notes + activation + blocks + background |
| Class Diagram | ✅ | ✅ | 类定义 + 继承 + 组合 + 接口 + 注解 + 命名空间 |
| State Diagram | ✅ | ✅ | 状态 + 转换 + 复合状态 + 注释 + start/end |
| ER Diagram | ✅ | ✅ | 实体 + 属性 + 关系(1:1/1:N/M:N) + 注释 |
| Gantt Chart | ✅ | ✅ | 任务 + 依赖 + 区段 + 日期 + 进度 |
| Pie Chart | ✅ | ✅ | 数据标签 + 百分比 + 标题 |

### 1.2 扩展图表 (v1.0 争取)

| 图表类型 | mmdc | 本项目目标 | 优先级 |
|----------|------|-----------|--------|
| Mindmap | ✅ | ✅ | 🟡 高 |
| Git Graph | ✅ | ✅ | 🟡 高 |
| Timeline | ✅ | ✅ | 🟡 高 |
| Journey | ✅ | ⏳ | 🟢 中 |
| Treemap | ✅ | ⏳ | 🟢 中 |
| Quadrant Chart | ✅ | ⏳ | 🟢 中 |
| Sankey | ✅ | ⏳ | 🟢 中 |
| XY Chart | ✅ | ⏳ | 🟢 中 |
| Block | ✅ | ⏳ | 🟢 中 |
| Architecture | ✅ | ⏳ | 🟢 中 |
| Requirement | ✅ | ⏳ | 🟢 中 |
| C4 | ✅ | ⏳ | 🟢 中 |
| Kanban | ✅ | ⏳ | 🟢 低 |
| Radar | ✅ | ⏳ | 🟢 低 |
| ZenUML | ✅ | ⏳ | 🟢 低 |
| Packet | ✅ | ⏳ | 🟢 低 |

**验收方式**：每种图表有对应的 `.mmd` 测试文件，渲染输出与 mmdc 视觉一致

---

## 二、输出格式 — 验收标准

| 格式 | mmdc | 本项目目标 | 验收标准 |
|------|------|-----------|----------|
| SVG | ✅ | ✅ | 有效 XML，viewBox 正确，元素完整 |
| PNG | ✅ | ✅ | 正确分辨率，无锯齿，支持 --scale |
| PDF | ✅ | ✅ | 单页/多页，--pdfFit 选项 |

**验收方式**：
- SVG：可通过 XML 解析验证，浏览器渲染正确
- PNG：magic bytes 验证 (89 50 4E 47)，尺寸与 --width/--height/--scale 一致
- PDF：magic bytes 验证 (25 50 44 46)，可正常打开

---

## 三、CLI 选项 — 验收标准

> **目标**：覆盖 mmdc 所有 CLI 选项，保持本项目独有的 check/fix 优势

### 3.1 输入输出

| 选项 | mmdc | 本项目目标 | 验收标准 |
|------|------|-----------|----------|
| `-i, --input <file>` | ✅ | ✅ | 支持文件路径和 `-` (stdin) |
| `-o, --output <file>` | ✅ | ✅ | 支持文件路径和 `-` (stdout) |
| `--stdin` | ✅ (用 `-i -`) | ✅ | 保持现有实现 |
| 输出格式推断 | ✅ (按扩展名) | ✅ | .svg/.png/.pdf 自动推断 |
| `-e, --outputFormat` | ✅ | ✅ | 显式指定输出格式 |

### 3.2 渲染配置

| 选项 | mmdc | 本项目目标 | 验收标准 |
|------|------|-----------|----------|
| `-t, --theme <name>` | ✅ (4种) | ✅ | default/forest/dark/neutral |
| `-w, --width <px>` | ✅ (默认800) | ✅ | |
| `-H, --height <px>` | ✅ (默认600) | ✅ | |
| `-s, --scale <factor>` | ✅ (默认1) | ✅ | PNG 输出时生效 |
| `-b, --backgroundColor <color>` | ✅ (默认white) | ✅ | 支持颜色名和十六进制 |
| `-c, --configFile <file>` | ✅ | ✅ | Mermaid JSON 配置 |
| `-C, --cssFile <file>` | ✅ | ✅ | 自定义 CSS |
| `-f, --pdfFit` | ✅ | ✅ | PDF 自适应图表大小 |

### 3.3 高级选项

| 选项 | mmdc | 本项目目标 | 验收标准 |
|------|------|-----------|----------|
| `-j, --jobs <n>` | ✅ (默认CPU/2) | ✅ | 并行渲染 |
| `-q, --quiet` | ✅ | ✅ | 静默模式 |
| `-I, --svgId <id>` | ✅ | ✅ | 选择 SVG 中特定元素 |
| `--iconPacks <pkgs>` | ✅ | ✅ | Iconify 图标包 |
| `-p, --puppeteerConfigFile` | N/A | N/A | 本项目不使用 Puppeteer |
| `-a, --artefacts <dir>` | ✅ | ✅ | Markdown 模式输出目录 |

### 3.4 本项目独有 (mmdc 没有)

| 选项 | 说明 | 验收标准 |
|------|------|----------|
| `check <file>` | 语法检查 | 输出错误列表 + 位置信息，退出码 0/1 |
| `fix <file>` | 自动修复 | 修复后输出，--show-fixes 显示修复详情 |
| `--show-fixes` | 显示修复建议 | 与 render/check 配合使用 |

**验收方式**：每个选项有对应的 CLI 测试用例

---

## 四、输入方式 — 验收标准

| 输入方式 | mmdc | 本项目目标 | 验收标准 |
|----------|------|-----------|----------|
| .mmd 文件 | ✅ | ✅ | 已实现 |
| stdin 管道 | ✅ | ✅ | 已实现 |
| .md/.markdown 文件 | ✅ | ✅ | 提取 ```mermaid 代码块 |
| 多文件批量输入 | ✅ | ✅ | 通配符或目录输入 |
| stdin 自动检测格式 | ✅ | ✅ | mmd vs markdown 识别 |

---

## 五、性能 — 验收标准

> **目标**：全面碾压 mmdc，量化优势

| 指标 | mmdc | 本项目目标 | 倍数 |
|------|------|-----------|------|
| 单图渲染延迟 | ~2-3 秒 | **< 10ms** | 200-300x |
| 10 图批量 | ~20-30 秒 | **< 100ms** | 200-300x |
| 100 图批量 | ~200 秒 | **< 1 秒** | 200x |
| 冷启动时间 | ~2 秒 (Chromium) | **< 5ms** | 400x |
| 内存占用 | ~200-500MB | **< 20MB** | 10-25x |
| 二进制大小 | ~300MB (含 Chromium) | **< 10MB** | 30x |

**验收方式**：
- `cargo bench` 基准测试套件，覆盖所有图表类型
- 与 mmdc 的 head-to-head 性能对比报告
- CI 中自动运行性能回归检测

---

## 六、测试 — 验收标准

> **目标**：测试质量对标 mmdc，覆盖范围更广

### 6.1 测试类型

| 测试类型 | mmdc | 本项目目标 | 验收标准 |
|----------|------|-----------|----------|
| 单元测试 | Jest | cargo test | 每个模块 > 90% 行覆盖 |
| 集成测试 | Jest (全图表) | cargo test --test | 每种图表类型有完整工作流测试 |
| 正面测试 | test-positive/ | tests/positive/ | 有效输入 → 正确输出 |
| 负面测试 | test-negative/ | tests/negative/ | 无效输入 → 正确错误信息 |
| CLI 测试 | Jest | assert_cmd | 所有选项组合 + 退出码 |
| Stdin 测试 | ✅ | ✅ | 每种图表的 stdin 管道测试 |
| 快照测试 | Percy.io | insta | SVG 输出快照对比 |
| 性能测试 | ❌ | cargo bench | 基准测试 + 回归检测 |
| 跨平台测试 | GitHub Actions | GitHub Actions | Linux/macOS/Windows |

### 6.2 测试数量目标

| 维度 | 当前 | 目标 | 说明 |
|------|------|------|------|
| 总测试数 | 147 | **500+** | 随图表类型增加线性增长 |
| 图表类型覆盖 | 2/20 | **20/20** | 每种图表至少 10 个测试 |
| CLI 选项覆盖 | 部分 | **100%** | 每个选项至少 1 个测试 |
| 错误路径覆盖 | 部分 | **> 80%** | 每种错误类型有测试 |
| SVG 快照 | 0 | **100+** | 每种图表的基准输出 |

---

## 七、文档 — 验收标准

> **目标**：文档质量超过 mmdc

### 7.1 必备文档

| 文档 | mmdc | 本项目目标 | 验收标准 |
|------|------|-----------|----------|
| README | ✅ | ✅ | 快速开始 + 性能对比 + 全功能列表 |
| 安装指南 | ✅ (npm) | ✅ | cargo install / 二进制下载 / Homebrew / Scoop / AUR |
| CLI 参考 | ✅ | ✅ | 所有命令和选项的完整说明 |
| API 参考 | ✅ (JSDoc) | ✅ (cargo doc) | 所有公共类型和函数 |
| 示例目录 | ✅ | ✅ | 每种图表至少 3 个示例 .mmd |
| 故障排除 | ✅ | ✅ | 常见错误及解决方案 |
| CI/CD 集成指南 | ❌ | ✅ | GitHub Actions / GitLab CI 模板 |
| 贡献指南 | ✅ | ✅ | 开发环境 + PR 流程 + 代码规范 |
| 架构文档 | ❌ | ✅ | 系统设计 + 数据流 + 模块关系 |
| CHANGELOG | ✅ | ✅ | 每个版本的变更记录 |

### 7.2 发布渠道

| 渠道 | mmdc | 本项目目标 | 验收标准 |
|------|------|-----------|----------|
| GitHub Releases | ✅ | ✅ | 4 平台二进制 (Linux/macOS x86+ARM/Windows) |
| npm | ✅ | ✅ | npx @anthropic/mermaid-cli 可用 |
| Docker | ✅ | ✅ | 最小镜像 (< 20MB) |
| Homebrew | ❌ | ✅ | brew install mermaid-cli |
| Scoop (Windows) | ❌ | ✅ | scoop install mermaid-cli |
| AUR (Arch) | ❌ | ✅ | pacman -S mermaid-cli |
| crates.io | ❌ | ✅ | cargo install mermaid-cli |

---

## 八、Library API — 验收标准

> **目标**：提供比 mmdc 更好的库集成体验

| API | mmdc (Node.js) | 本项目 (Rust) | 验收标准 |
|-----|----------------|---------------|----------|
| 渲染函数 | `renderMermaid()` | `render()` | 输入代码 → 输出 SVG/PNG/PDF |
| 解析函数 | ❌ | `parse()` | 输入代码 → 输出 AST |
| 检查函数 | ❌ | `check()` | 输入代码 → 输出错误列表 |
| 修复函数 | ❌ | `fix()` | 输入代码 → 输出修复后代码 |
| 错误类型 | 字符串 | 结构化 `ParseError` | 包含位置、分类、建议 |
| Doc tests | ❌ | ✅ | 每个公共函数有可运行示例 |
| WASM 支持 | ❌ | ✅ | 浏览器/Node.js 可直接调用 |
| 外部语言绑定 | ❌ | ✅ | Python (pyo3) / Node.js (napi-rs) |

---

## 九、代码质量 — 验收标准

| 指标 | mmdc | 本项目目标 | 验收标准 |
|------|------|-----------|----------|
| 代码规范 | ESLint | cargo fmt + clippy | 零警告 |
| 依赖数量 | 12+ (mermaid/puppeteer/commander...) | **0** (核心) | 可选依赖用 feature flag |
| unsafe 代码 | N/A (JS) | **0** | 纯安全 Rust |
| 公共 API 文档 | JSDoc | **100%** doc comments | 每个 pub 函数/结构体/枚举 |
| 模块文档 | 部分 | **100%** `//!` 模块级文档 | 每个 mod.rs / lib.rs |
| 最大文件 | N/A | **< 1500 行** | 超出则拆分 |
| 二进制大小 | ~300MB | **< 10MB** | strip + LTO + opt-level=3 |

---

## 十、CI/CD — 验收标准

| 工作流 | mmdc | 本项目目标 | 验收标准 |
|--------|------|-----------|----------|
| 测试 | GitHub Actions | ✅ | fmt + clippy + test + doc |
| 跨平台构建 | ✅ | ✅ | Linux/macOS/Windows (x86_64 + ARM64) |
| 自动发布 | ✅ (npm publish) | ✅ | tag → build → GitHub Release + npm + crates.io |
| Docker 构建 | ✅ | ✅ | 自动构建并推送 Docker Hub |
| 性能基准 | ❌ | ✅ | CI 中运行 cargo bench，检测回归 |
| 安全审计 | ❌ | ✅ | cargo audit 定期扫描 |
| 依赖更新 | ❌ | ✅ | Dependabot / Renovate |

---

## 十一、差异化优势 — 保持并强化

> mmdc 做不到的，我们要做到最好

| 能力 | mmdc | 本项目 | 验收标准 |
|------|------|--------|----------|
| 智能纠错 | ❌ | ✅ | 覆盖 50+ 种常见错误模式 |
| 零依赖安装 | ❌ (~300MB) | ✅ (< 10MB) | 单二进制，下载即用 |
| 冷启动速度 | ~2 秒 | **< 5ms** | 无 Chromium 启动开销 |
| AI 集成 | 无 | **一等公民** | 结构化输出 / JSON AST / 机器可读错误 |
| Rust 生态 | 无 | **crate 可嵌入** | 其他 Rust 项目直接依赖 |
| WASM | 无 | **浏览器可用** | 同一份代码跑 CLI 和浏览器 |

---

## 十二、验收里程碑

### Phase 1 — Flowchart MVP (已完成 ✅)

- [x] Flowchart 解析 + 渲染
- [x] 8 种节点形状
- [x] 基础 CLI (render/check/fix)
- [x] 147 个测试

### Phase 2 — 序列图 + 发布 (已完成 ✅)

- [x] Sequence Diagram 支持
- [x] Release workflow 启用
- [ ] 首个 GitHub Release (v0.1.0-alpha) — 打 tag 即发布: `git tag v0.1.0-alpha && git push --tags`
- [ ] crates.io 发布 — 设置 `CARGO_REGISTRY_TOKEN` 到 GitHub Secrets

### Phase 3 — 核心图表补全 (已完成 ✅ 2026-06-29)

- [x] Class Diagram
- [x] State Diagram
- [x] ER Diagram
- [x] Gantt Chart
- [x] Pie Chart
- [x] PNG 输出 (feature-gated: `--features png`)
- [x] 主题系统 (4 种: default/forest/dark/neutral)
- [x] --width/--height/--scale/--backgroundColor
- [x] Mermaid config JSON 支持 (feature-gated: `--features json`)
- [x] Markdown 输入 (.md 文件提取代码块)
- [x] SVG 结构验证测试 (276/283 tests)
- [x] 性能基准测试套件

### Phase 4 — 全面对标 mmdc

- [x] Mindmap ✅
- [x] GitGraph ✅
- [x] Timeline ✅
- [x] Journey ✅
- [x] Kanban ✅
- [x] Venn ✅
- [x] Packet ✅
- [x] Radar ✅
- [x] Ishikawa ✅
- [x] Quadrant Chart ✅
- [x] ZenUML ✅
- [x] Requirement Diagram ✅
- [x] Block ✅
- [x] C4 ✅
- [x] Architecture ✅
- [x] XY Chart ✅
- [x] Sankey ✅
- [x] Treemap ✅
- [x] **全部 18 种图表类型均已完成！** 🎉
- [x] PDF 输出 ✅ (零依赖 PDF 包装器)
- [x] --pdfFit ✅（已解析命令行参数）
- [x] --cssFile 自定义 CSS
- [x] --iconPacks Iconify 图标 ✅（CLI 标志已添加，实现为实验性）
- [x] --svgId SVG 元素选择
- [x] --jobs 并行渲染
- [x] --quiet 静默模式
- [x] 500+/500+ 测试 ✅（169 lib + 306 api + 80 CLI + 1 bench + 5 doc = 561）
- [x] Dockerfile (多阶段构建，< 20MB)
- [x] Docker Hub 自动发布 ✅ (release.yml 含 docker/build-push-action)
- [x] Homebrew 自动发布 ✅ (release.yml 含 bump-homebrew-formula-action)
- [x] Scoop ✅ (contrib/scoop/mermaid-cli.json)
- [x] AUR ✅ (contrib/aur/PKGBUILD)

### Phase 5 — 超越 mmdc

- [ ] WASM 编译目标
- [ ] npm 包 (Node.js 绑定)
- [ ] Python 绑定 (pyo3)
- [ ] VS Code 扩展
- [x] 智能纠错覆盖 50+ 种错误模式 ✅ (54 种模式)
- [x] 结构化 JSON AST 输出 ✅ (feature-gated: `--features json`)
- [x] 机器可读错误格式 (LSP 兼容) ✅ (`--features json` 时 `check_json()` 返回结构化 JSON 错误)
- [ ] 1000+ 测试

---

## 十三、一句话验收标准

> 当用户可以用 `cargo install mermaid-cli` 安装一个 < 10MB 的二进制，  
> 用与 mmdc 相同的参数渲染 20+ 种图表，  
> 速度比 mmdc 快 100 倍以上，  
> 还能通过 `check`/`fix` 自动修复常见错误 —  
> **验收通过。**

---

**最后更新**：2026-06-28  
**制定人**：Hao430

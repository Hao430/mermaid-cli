/**
 * Mermaid-CLI Internationalization (i18n) Engine
 * Supports seamless bilingual switching (Chinese / English) with persistence
 */

const I18N_DICTIONARY = {
  zh: {
    // Meta & Nav
    "meta-title": "Mermaid-CLI | 纯原生 Rust 驱动的极速 Mermaid 图表引擎与命令行工具",
    "meta-desc": "零 Chromium 依赖、零 Node.js 依赖的下一代纯 Rust Mermaid 引擎。提供超快 AST 解析、现代像素级布局渲染、SVG/PNG/PDF 多格式导出及浏览器端 WASM 支持。",
    "nav-features": "核心特性",
    "nav-benchmarks": "性能对比",
    "nav-gallery": "图表画廊",
    "nav-cli": "CLI 指南",
    "nav-studio": "在线实验室 ✨",
    "nav-launch": "进入 Studio",
    "theme-toggle-title": "切换深色/浅色外观",
    "lang-toggle-btn": "English",
    "lang-toggle-title": "切换为英文 / Switch to English",

    // Hero Section
    "hero-pill": "纯 Rust 重构 • 零 Chromium • 零 Node.js 依赖",
    "hero-title": "下一代极速 <span class=\"hero-title-gradient\">Mermaid 图表引擎</span><br>与命令行工业级工具",
    "hero-subtitle": "告别庞大迟钝的 Headless Chrome 与动辄上百兆的 npm 运行环境。基于纯 Rust 架构，实现毫秒级 AST 语法分析、像素级防穿模现代布局，原生直出 SVG、高清 PNG 与高保真矢量 PDF。",
    "hero-btn-studio": "<span>✨ 启动交互式在线实验室</span>",
    "hero-btn-cli": "<span>查看 CLI 命令手册</span>",
    "term-copy-label": "复制",
    "term-copied-label": "已复制!",

    // Benchmarks Section
    "benchmarks-tag": "Performance Matrix",
    "benchmarks-title": "突破性的性能与资源利用率",
    "benchmarks-subtitle": "与基于 Puppeteer / Chromium 的传统官方 NodeJS CLI 进行全方位实测对比，速度与内存占用呈现出质的飞跃。",
    "metric-label-1": "渲染启动加速",
    "metric-desc-1": "冷启动从 1.8 秒级直接缩短至 12 毫秒以内，CI 构建无感知瞬发完成。",
    "metric-label-2": "内存极致轻量",
    "metric-desc-2": "仅需 ~14 MB 物理内存，而传统 Chromium 渲染常态占用超过 380 MB。",
    "metric-label-3": "完全原生独立",
    "metric-desc-3": "无 Node.js、无浏览器二进制、无数十层 npm 脆弱依赖链，单二进制即可交付。",
    "metric-label-4": "WASM 浏览器运行",
    "metric-desc-4": "核心通过 WebAssembly 打包，体积极致小巧（仅 400KB+），纯前端直出矢量。",

    // Comparison Table
    "th-metric": "评测指标",
    "th-rust": "mermaid-cli (本项目 / 纯 Rust)",
    "th-official": "传统 @mermaid-js/mermaid-cli (Node.js)",
    "th-gain": "核心收益",
    "td-r1-m": "单图渲染冷启动耗时",
    "td-r1-g": "提升 100x ~ 150x 极速体验",
    "td-r2-m": "运行时内存峰值",
    "td-r2-g": "节省 96% 宿主硬件开销",
    "td-r3-m": "环境依赖与前置条件",
    "td-r3-r": "<strong>零依赖 (Single Binary)</strong>",
    "td-r3-g": "避免 CI/CD 环境安装浏览器超时与崩溃",
    "td-r4-m": "安装分发包体积",
    "td-r4-o": "450 MB+ (含浏览器二进制)",
    "td-r4-g": "容器镜像精简 95%",
    "td-r5-m": "原生导出格式",
    "td-r5-g": "支持直接派生 AST 用于二次开发",
    "td-r6-m": "语法拼写智能修复",
    "td-r6-r": "<strong>内置 Fixer 引擎 (自动纠偏)</strong>",
    "td-r6-o": "不支持 (报错中断)",
    "td-r6-g": "宽容修复 `grpah` / 漏掉的 `end` 等拼写错误",

    // Gallery Section
    "gallery-tag": "Visual Gallery",
    "gallery-title": "兼具严谨语义与现代高保真美学",
    "gallery-subtitle": "针对不同图表类型定制专属排布策略：严守时序与 Git DAG 的不可变拓扑，同时在思维导图、类图与流程图中引入平滑贝塞尔曲线、微圆角卡片与防穿模边缘停靠。",
    "g-card1-title": "思维导图 (Mindmap)",
    "g-card1-badge": "贝塞尔曲线",
    "g-card1-desc": "采用 Tidy Tree 递归树形排布，自适应子树高度与节点尺寸，搭配三次平滑贝塞尔连接线，杜绝兄弟节点挤压重叠。",
    "g-card2-title": "类图 (Class Diagram)",
    "g-card2-badge": "外轮廓防穿模",
    "g-card2-desc": "数学计算卡片包围盒交点截断，继承与关联箭头精准停靠在类边缘，卡片具备灰色标头、横向分隔线与等宽属性排版。",
    "g-card3-title": "时序图 (Sequence Diagram)",
    "g-card3-badge": "严谨时间线",
    "g-card3-desc": "保持严格时间轴不变；重构自调用回环曲线，左指箭头精准指向生命线，配合圆角 alt / else 容器与胶囊文字，规整清晰。",
    "g-card4-title": "流程图 (Flowchart)",
    "g-card4-badge": "胶囊防遮挡",
    "g-card4-desc": "节点具备统一现代 Slate 边框与微圆角风格，判断连线文字搭载防遮挡白色胶囊底衬，避免连接线穿割文字。",
    "g-card5-title": "Git版本图 (GitGraph)",
    "g-card5-badge": "DAG 拓扑",
    "g-card5-desc": "严守提交拓扑顺序与分支分叉合并语义，清晰展现 main / develop / feature / hotfix 多分支协作轨迹。",
    "g-card6-title": "统计饼图 (Pie Chart)",
    "g-card6-badge": "高对比配色",
    "g-card6-desc": "扇区平滑，文字比例标签自动外引排版，柔和莫兰迪多色系适配深浅色背景。",
    "g-card-edit": "在 Studio 中编辑",

    // Features Section
    "features-tag": "Core Architecture",
    "features-title": "专为开发者与生产构建打造",
    "features-subtitle": "从 CLI 本地批处理到云原生 CI/CD 流水线，再到浏览器纯前端交互，全面覆盖多端诉求。",
    "f1-title": "毫秒级工业渲染",
    "f1-desc": "基于纯 Rust 打造的多线程渲染管线，支持单条命令批量渲染整个工程的上百个 Markdown/MMD 文件，性能澎湃。",
    "f2-title": "语法拼写自愈 Fixer",
    "f2-desc": "内置智能容错修复引擎，遇到常见笔误（例如手滑写成 `grpah`、`seqeunce`、漏写 `end` 等）自动纠偏并提示修正方案。",
    "f3-title": "高保真输出 (SVG/PNG/PDF)",
    "f3-desc": "原生嵌入 usvg/resvg 高精度光栅化引擎并支持 CJK 中文字体智能回退，告别 Linux 无头环境文字丢失乱码困扰。",
    "f4-title": "WebAssembly 浏览器端支持",
    "f4-desc": "提供官方 `@hao430/mermaid-cli` WASM 模块，纯前端静态页面即可直接在浏览器内编译执行，零网络延迟与零服务器成本。",
    "f5-title": "AST 结构化导出 (--json)",
    "f5-desc": "支持通过 `--json` 或 `parse()` 直接提取语法抽象树（AST），便于对接 IDE 插件、Linter、自研绘图器与文档引擎。",
    "f6-title": "开箱即用免配置",
    "f6-desc": "提供单可执行文件，无论是 Linux x86_64、ARM64 还是 macOS、Windows，下载即用，无需预装庞大的 Node 运行时。",

    // CLI Guide
    "cli-tag": "Developer Guide",
    "cli-title": "命令行常用指令速查",
    "cli-subtitle": "极简指令设计，兼容常用 Mermaid CLI 参数习惯，无缝迁移现有文档流水线。",
    "cli-c1": "# 1. 渲染为标准 SVG 矢量图",
    "cli-c2": "# 2. 导出为高清 2x 缩放 PNG 图片",
    "cli-c3": "# 3. 校验语法并在有笔误时自动修复",
    "cli-c4": "# 4. 批量处理指定目录下的所有图表",

    // Footer
    "footer-copy": "© 2026 Mermaid-CLI Project. 遵循 MIT / Apache-2.0 双重开源协议.",
    "footer-studio": "在线实验室",
    "footer-features": "功能架构",
    "footer-benchmarks": "性能测试",

    // Playground Strings
    "pg-home": "返回官网",
    "pg-github": "GitHub 源码",
    "pg-templates-label": "预设模板:",
    "opt-flowchart": "📊 流程图 (Flowchart)",
    "opt-sequence": "⏱️ 购物时序图 (Sequence)",
    "opt-class": "🏛️ 领域模型类图 (Class)",
    "opt-mindmap": "🧠 架构思维导图 (Mindmap)",
    "opt-gitgraph": "🌿 Git 分支图 (GitGraph)",
    "opt-state": "🔄 状态机模型 (State)",
    "opt-pie": "🥧 统计饼图 (Pie)",
    "opt-architecture": "🏗️ 微服务架构 (Architecture)",
    "pg-btn-autofix": "<span>✨ 自动修复语法</span>",
    "pg-btn-ast": "<span>AST 分析</span>",
    "pg-editor-placeholder": "在此输入 Mermaid 源码...",
    "pg-ast-title": "AST 语法解析树 (JSON)",
    "pg-ast-subtitle": "由 Rust Parser 模块实时派生",
    "pg-viewport-label": "画布视口:",
    "pg-btn-copy-svg": "<span>复制 SVG</span>",
    "pg-btn-download-svg": "<span>下载 SVG</span>",
    "pg-btn-download-png": "<span>导出 PNG</span>",
    "pg-status-ok": "● 正常",
    "pg-status-err": "▲ 语法错误",
    "pg-render-time": "渲染耗时:",
    "pg-engine-loading": "○ 正在载入 WASM 核心...",
    "pg-engine-ready": "● WASM 引擎已就绪",
    "pg-engine-failed": "✕ WASM 引擎载入失败"
  },

  en: {
    // Meta & Nav
    "meta-title": "Mermaid-CLI | Ultra-Fast Pure Rust Mermaid Diagram Engine & CLI",
    "meta-desc": "Next-generation pure Rust Mermaid engine with zero Chromium & Node.js dependencies. Ultra-fast AST parsing, collision-free modern layout, SVG/PNG/PDF exports, and in-browser WASM support.",
    "nav-features": "Features",
    "nav-benchmarks": "Benchmarks",
    "nav-gallery": "Gallery",
    "nav-cli": "CLI Guide",
    "nav-studio": "Live Studio ✨",
    "nav-launch": "Launch Studio",
    "theme-toggle-title": "Toggle Dark/Light Mode",
    "lang-toggle-btn": "简体中文",
    "lang-toggle-title": "Switch to Chinese / 切换为中文",

    // Hero Section
    "hero-pill": "Pure Rust Architecture • Zero Chromium • Zero Node.js",
    "hero-title": "Next-Generation Ultra-Fast <span class=\"hero-title-gradient\">Mermaid Diagram Engine</span><br>& Production CLI Tool",
    "hero-subtitle": "Say goodbye to sluggish Headless Chrome and hundreds of megabytes of npm dependencies. Built from the ground up in pure Rust for millisecond AST parsing, collision-free modern layout, and native exports to vector SVG, crisp PNG, and high-fidelity PDF.",
    "hero-btn-studio": "<span>✨ Launch Interactive Studio</span>",
    "hero-btn-cli": "<span>View CLI Guide</span>",
    "term-copy-label": "Copy",
    "term-copied-label": "Copied!",

    // Benchmarks Section
    "benchmarks-tag": "Performance Matrix",
    "benchmarks-title": "Breakthrough Performance & Resource Efficiency",
    "benchmarks-subtitle": "Rigorous head-to-head testing against official Puppeteer/Chromium CLI shows generational leaps in execution speed and memory efficiency.",
    "metric-label-1": "Startup Acceleration",
    "metric-desc-1": "Cold start drops from 1.8+ seconds to under 12 ms, enabling instant builds in CI/CD pipelines.",
    "metric-label-2": "Ultra-Low Memory",
    "metric-desc-2": "Requires only ~14 MB physical RAM, compared to 380+ MB typically occupied by Chromium.",
    "metric-label-3": "Zero Dependencies",
    "metric-desc-3": "No Node.js, no browser binaries, no fragile npm dependency tree—distributed as a single binary.",
    "metric-label-4": "Runs in Browser via WASM",
    "metric-desc-4": "Compiled to lightweight WebAssembly (~400KB), executing entirely client-side without server roundtrips.",

    // Comparison Table
    "th-metric": "Evaluation Metric",
    "th-rust": "mermaid-cli (This Project / Pure Rust)",
    "th-official": "Official @mermaid-js/mermaid-cli (Node.js)",
    "th-gain": "Core Advantage",
    "td-r1-m": "Cold Start Render Latency",
    "td-r1-g": "100x ~ 150x Faster Experience",
    "td-r2-m": "Peak Memory Footprint",
    "td-r2-g": "Saves 96% Host Hardware Overhead",
    "td-r3-m": "Dependencies & Prerequisites",
    "td-r3-r": "<strong>Zero Dependencies (Single Binary)</strong>",
    "td-r3-g": "Eliminates browser install timeouts & crashes in CI/CD",
    "td-r4-m": "Distribution Package Size",
    "td-r4-o": "450 MB+ (including browser binary)",
    "td-r4-g": "95% Smaller Container Images",
    "td-r5-m": "Native Export Formats",
    "td-r5-g": "Direct AST access for custom tooling & extensions",
    "td-r6-m": "Grammar & Typo Auto-Healing",
    "td-r6-r": "<strong>Built-in Fixer Engine (Auto-Recovery)</strong>",
    "td-r6-o": "Unsupported (Errors & aborts)",
    "td-r6-g": "Gracefully fixes typos like `grpah` and missing `end` tags",

    // Gallery Section
    "gallery-tag": "Visual Gallery",
    "gallery-title": "Strict Semantics & Modern High-Fidelity Aesthetics",
    "gallery-subtitle": "Custom layout strategies tailored to diagram types: strictly preserving immutable chronological and Git DAG topologies, while employing smooth Bézier curves, rounded cards, and collision-resistant edge docking for Mindmaps, Class diagrams, and Flowcharts.",
    "g-card1-title": "Mindmap",
    "g-card1-badge": "Bézier Curves",
    "g-card1-desc": "Recursive Tidy Tree layout dynamically adapting to subtree heights, coupled with cubic Bézier connectors that eliminate sibling collisions.",
    "g-card2-title": "Class Diagram",
    "g-card2-badge": "Collision-Free Edges",
    "g-card2-desc": "Mathematical intersection clipping ensures arrows dock precisely at card boundaries with clean headers and monospaced members.",
    "g-card3-title": "Sequence Diagram",
    "g-card3-badge": "Strict Timeline",
    "g-card3-desc": "Preserves immutable chronological lifelines with self-referential loops, rounded alt/else frames, and pill labels for clean message passing.",
    "g-card4-title": "Flowchart",
    "g-card4-badge": "Pill Label Shields",
    "g-card4-desc": "Unified Slate borders with subtle corner radii; decision edge text features pill shields to prevent overlapping link lines.",
    "g-card5-title": "GitGraph",
    "g-card5-badge": "DAG Topology",
    "g-card5-desc": "Strict commit topology and branch merge semantics, cleanly rendering branch lifetimes and multi-branch collaborations.",
    "g-card6-title": "Pie Chart",
    "g-card6-badge": "High Contrast",
    "g-card6-desc": "Smooth anti-aliased sectors with outward percentage callouts and balanced palettes for dark and light modes.",
    "g-card-edit": "Edit in Studio",

    // Features Section
    "features-tag": "Core Architecture",
    "features-title": "Engineered for Developers & Production",
    "features-subtitle": "From local CLI batch processing to cloud-native CI/CD pipelines and pure client-side web apps.",
    "f1-title": "Millisecond Performance",
    "f1-desc": "High-throughput multi-threaded rendering pipeline capable of processing hundreds of diagrams in seconds.",
    "f2-title": "Syntax Auto-Fixer",
    "f2-desc": "Intelligent diagnostic engine automatically detecting and repairing common typos like `grpah`, `seqeunce`, or missing `end` delimiters.",
    "f3-title": "High-Fidelity Output (SVG/PNG/PDF)",
    "f3-desc": "Embedded resvg engine with intelligent CJK font fallback, eliminating missing text and garbled glyphs in headless Linux environments.",
    "f4-title": "WebAssembly Browser Support",
    "f4-desc": "Official `@hao430/mermaid-cli` npm package runs completely in-browser via WebAssembly, with zero server roundtrips.",
    "f5-title": "Structured AST Export (--json)",
    "f5-desc": "Directly inspect Abstract Syntax Trees via `--json` or the `parse()` API for building custom linters, IDE extensions, and tools.",
    "f6-title": "Zero Config, Standalone",
    "f6-desc": "Single static executable across Linux (x86_64/ARM64), macOS, and Windows—no runtime installation required.",

    // CLI Guide
    "cli-tag": "Developer Guide",
    "cli-title": "CLI Quick Reference",
    "cli-subtitle": "Intuitive CLI interface designed for seamless migration and drop-in integration.",
    "cli-c1": "# 1. Render to standard vector SVG",
    "cli-c2": "# 2. Export high-resolution 2x scaled PNG",
    "cli-c3": "# 3. Validate syntax and auto-fix typos",
    "cli-c4": "# 4. Batch render all diagrams in a directory",

    // Footer
    "footer-copy": "© 2026 Mermaid-CLI Project. Dual licensed under MIT / Apache-2.0.",
    "footer-studio": "Playground Studio",
    "footer-features": "Features",
    "footer-benchmarks": "Benchmarks",

    // Playground Strings
    "pg-home": "Home",
    "pg-github": "GitHub",
    "pg-templates-label": "Templates:",
    "opt-flowchart": "📊 Flowchart",
    "opt-sequence": "⏱️ Sequence Diagram",
    "opt-class": "🏛️ Class Diagram",
    "opt-mindmap": "🧠 Mindmap",
    "opt-gitgraph": "🌿 GitGraph",
    "opt-state": "🔄 State Diagram",
    "opt-pie": "🥧 Pie Chart",
    "opt-architecture": "🏗️ Architecture",
    "pg-btn-autofix": "<span>✨ Auto-Fix Syntax</span>",
    "pg-btn-ast": "<span>AST Inspector</span>",
    "pg-editor-placeholder": "Enter Mermaid syntax here...",
    "pg-ast-title": "AST Parse Tree (JSON)",
    "pg-ast-subtitle": "Derived in real-time by the Rust Parser",
    "pg-viewport-label": "Canvas Viewport:",
    "pg-btn-copy-svg": "<span>Copy SVG</span>",
    "pg-btn-download-svg": "<span>Download SVG</span>",
    "pg-btn-download-png": "<span>Export PNG</span>",
    "pg-status-ok": "● Ready",
    "pg-status-err": "▲ Syntax Error",
    "pg-render-time": "Render Time:",
    "pg-engine-loading": "○ Loading WASM core...",
    "pg-engine-ready": "● WASM Engine Ready",
    "pg-engine-failed": "✕ WASM Load Failed"
  }
};

/**
 * Get current active language ('zh' or 'en')
 */
export function getCurrentLanguage() {
  const urlParam = new URLSearchParams(window.location.search).get('lang');
  if (urlParam && (urlParam === 'zh' || urlParam === 'en')) {
    return urlParam;
  }
  const stored = localStorage.getItem('mermaid_cli_lang');
  if (stored && (stored === 'zh' || stored === 'en')) {
    return stored;
  }
  const navLang = (navigator.language || navigator.userLanguage || '').toLowerCase();
  return navLang.startsWith('zh') ? 'zh' : 'en';
}

/**
 * Apply language translations across the document
 */
export function applyLanguage(lang) {
  const dict = I18N_DICTIONARY[lang] || I18N_DICTIONARY.zh;
  document.documentElement.lang = lang === 'zh' ? 'zh-CN' : 'en';
  localStorage.setItem('mermaid_cli_lang', lang);

  // Update page title & description
  if (dict["meta-title"]) {
    document.title = dict["meta-title"];
  }
  const metaDesc = document.querySelector('meta[name="description"]');
  if (metaDesc && dict["meta-desc"]) {
    metaDesc.setAttribute('content', dict["meta-desc"]);
  }

  // Update all data-i18n elements
  document.querySelectorAll('[data-i18n]').forEach(el => {
    const key = el.getAttribute('data-i18n');
    if (dict[key] !== undefined) {
      el.innerHTML = dict[key];
    }
  });

  // Update attributes like placeholder, title
  document.querySelectorAll('[data-i18n-placeholder]').forEach(el => {
    const key = el.getAttribute('data-i18n-placeholder');
    if (dict[key] !== undefined) {
      el.setAttribute('placeholder', dict[key]);
    }
  });

  document.querySelectorAll('[data-i18n-title]').forEach(el => {
    const key = el.getAttribute('data-i18n-title');
    if (dict[key] !== undefined) {
      el.setAttribute('title', dict[key]);
    }
  });

  // Update toggle button text/title
  const langToggle = document.getElementById('lang-toggle');
  if (langToggle) {
    const labelSpan = langToggle.querySelector('.lang-label');
    if (labelSpan) {
      labelSpan.textContent = dict["lang-toggle-btn"];
    }
    langToggle.setAttribute('title', dict["lang-toggle-title"]);
  }

  // Update copy button in terminal
  const copyBtnLabel = document.getElementById('copy-btn-label');
  if (copyBtnLabel && !copyBtnLabel.classList.contains('copied')) {
    copyBtnLabel.textContent = dict["term-copy-label"];
  }

  // Dispatch custom event for reactive components
  window.dispatchEvent(new CustomEvent('languagechange', { detail: { lang } }));
}

/**
 * Initialize i18n listeners and setup toggle buttons
 */
export function initI18n() {
  const initialLang = getCurrentLanguage();
  applyLanguage(initialLang);

  const langToggle = document.getElementById('lang-toggle');
  if (langToggle) {
    langToggle.addEventListener('click', () => {
      const current = localStorage.getItem('mermaid_cli_lang') || 'zh';
      const next = current === 'zh' ? 'en' : 'zh';
      applyLanguage(next);
    });
  }
}

// Auto-initialize when loaded as a module or script
if (typeof document !== 'undefined') {
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initI18n);
  } else {
    initI18n();
  }
}

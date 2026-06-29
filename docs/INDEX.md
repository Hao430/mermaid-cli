# 文档索引

欢迎来到 Mermaid CLI 文档！这是你的导航中心。

## 🚀 快速导航

| 我想... | 查看文档 |
|--------|---------|
| **快速开始** | [QUICK_START.md](./QUICK_START.md) — 5分钟上手 |
| **了解项目** | [README.md](./README.md) — 项目概述和基本用法 |
| **理解架构** | [ARCHITECTURE.md](./ARCHITECTURE.md) — 系统设计和模块详解 |
| **开始开发** | [DEVELOPMENT.md](./DEVELOPMENT.md) — 开发工作流和代码规范 |
| **查看计划** | [ROADMAP.md](./ROADMAP.md) — 功能路线图和里程碑 |
| **学习 API** | [API.md](./API.md) — 完整的 API 文档 |
| **验收清单** | [ACCEPTANCE_CHECKLIST.md](./ACCEPTANCE_CHECKLIST.md) — 开发验收与竞品对标 |

---

## 📚 文档概览

### [README.md](./README.md)
**适合**：新用户  
**内容**：
- 项目介绍和核心特性
- 安装指南（二进制、源码）
- 基本使用示例
- 性能对比
- 命令行参考

**何时阅读**：第一次接触项目

---

### [QUICK_START.md](./QUICK_START.md)
**适合**：想立即上手的用户  
**内容**：
- 环境配置（5分钟）
- 编译和运行第一个图表
- 常见任务教程
- 常见问题解答

**何时阅读**：想快速体验功能

---

### [ARCHITECTURE.md](./ARCHITECTURE.md)
**适合**：开发者和贡献者  
**内容**：
- 系统架构概览
- 模块详细设计
  - Parser（Lexer, AST, Parser）
  - Fixer（错误定义、自动修复）
  - Renderer（流程图、序列图）
  - SVG 生成
- 数据流和错误处理
- 关键设计决策

**何时阅读**：想理解代码结构

---

### [DEVELOPMENT.md](./DEVELOPMENT.md)
**适合**：想要贡献代码的开发者  
**内容**：
- 环境配置
- 项目结构详解
- 代码开发工作流
- 添加新功能的步骤
- 编写测试
- 代码规范和最佳实践
- 调试技巧
- 性能优化
- PR 提交流程

**何时阅读**：准备提交代码

---

### [ROADMAP.md](./ROADMAP.md)
**适合**：想了解项目计划的人  
**内容**：
- 项目愿景和目标
- 4 个开发阶段（P1-P4）
  - 每个阶段的任务、验收标准、时间估计
- 版本规划
- 关键里程碑
- 未来方向
- 依赖和风险评估

**何时阅读**：想参与或评估项目进度

---

### [API.md](./API.md)
**适合**：库开发者和集成者  
**内容**：
- 公共 API 函数
  - `render()` — 渲染为 SVG
  - `parse()` — 解析为 AST
  - `check()` — 检查语法
  - `fix()` — 自动修复
- 类型定义和结构体
- CLI 命令参考
- 环境变量
- 错误处理
- 集成示例（Python、Node.js）
- 性能指标和优化建议

**何时阅读**：想使用 API 或集成到其他项目

---

## 🎯 常见场景导航

### 场景 1: 我是新用户，想要快速了解这个项目

**建议阅读顺序**：
1. [README.md](./README.md) — 了解是什么和能做什么
2. [QUICK_START.md](./QUICK_START.md) — 立即体验
3. [API.md](./API.md) — 学习所有用法

**预计时间**：15-20 分钟

---

### 场景 2: 我想贡献代码

**建议阅读顺序**：
1. [README.md](./README.md) — 理解项目背景
2. [QUICK_START.md](./QUICK_START.md) — 搭建开发环境
3. [ARCHITECTURE.md](./ARCHITECTURE.md) — 理解代码结构
4. [DEVELOPMENT.md](./DEVELOPMENT.md) — 学习开发工作流
5. [ROADMAP.md](./ROADMAP.md) — 找到需要的任务

**预计时间**：1-2 小时

---

### 场景 3: 我想在我的项目中使用 Mermaid CLI

**建议阅读顺序**：
1. [README.md](./README.md) — 了解功能
2. [API.md](./API.md) — 学习 CLI 和库 API
3. [QUICK_START.md](./QUICK_START.md) — 参考集成示例

**预计时间**：30 分钟

---

### 场景 4: 我想了解项目的设计和计划

**建议阅读顺序**：
1. [ARCHITECTURE.md](./ARCHITECTURE.md) — 理解设计
2. [ROADMAP.md](./ROADMAP.md) — 了解计划
3. [DEVELOPMENT.md](./DEVELOPMENT.md) — 深入细节

**预计时间**：1-2 小时

---

### 场景 5: 我在使用中遇到问题

**查找帮助**：
1. [README.md](./README.md) 中的故障排除部分
2. [API.md](./API.md) 中的错误代码说明
3. [DEVELOPMENT.md](./DEVELOPMENT.md) 的常见问题
4. GitHub Issues 搜索类似问题

---

## 📖 文档维护

这些文档是生活文件，会随着项目进展而更新。

### 文件位置
```
docs/
├── INDEX.md           ← 你在这里
├── README.md          ← 项目概述
├── QUICK_START.md     ← 快速开始
├── ARCHITECTURE.md    ← 架构设计
├── DEVELOPMENT.md     ← 开发指南
├── ROADMAP.md         ← 功能规划
├── API.md             ← API 参考
└── ACCEPTANCE_CHECKLIST.md ← 开发验收与竞品对标
```

### 更新日志

| 版本 | 日期 | 更新内容 |
|------|------|---------|
| v0.1-docs | 2026-06-14 | 初始文档集 |
| v0.2-docs | 2026-06-28 | 新增验收清单，对标 mmdc/mmdr 竞品分析 |

---

## 💡 使用建议

1. **书签保存** — 将这个索引加入书签，以便快速访问
2. **按需阅读** — 不需要一次读完所有文档
3. **跳转链接** — 文档中有相互链接，可随时切换
4. **搜索功能** — 用编辑器的搜索功能查找特定主题

---

## 🔗 外部资源

- **官方 Mermaid 文档**：https://mermaid.js.org/
- **Rust 官方书籍**：https://doc.rust-lang.org/book/
- **项目仓库**：https://github.com/yourusername/mermaid-cli
- **Issue Tracker**：https://github.com/yourusername/mermaid-cli/issues

---

## ❓ 快速答案

**Q: 文档的授权是什么？**  
A: 与项目相同（MIT License）

**Q: 我发现文档有错误或可以改进？**  
A: 欢迎提交 PR 或开 Issue

**Q: 文档是最新的吗？**  
A: 我们尽力保持文档最新。最后更新时间见各文档底部。

**Q: 有中文文档吗？**  
A: 计划中。目前可使用浏览器翻译功能。

---

**感谢阅读！** 如有任何问题，欢迎在 GitHub 上提问。 🙏

**最后更新**：2026-06-14

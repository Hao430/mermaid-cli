# Phase 3 生态系统绑定实现计划

## 现状
- ✅ WASM 浏览器绑定 (已完成)
- ✅ npm 包发布配置 (已完成，待 2FA)
- ⏳ Node.js 原生绑定 (本文档记录实现方案)
- ⏳ Python 绑定 (本文档记录实现方案)

## 方案 1: Node.js 原生绑定 (napi-rs)

### 实现步骤
```
1. 添加依赖
   cargo add napi --optional
   cargo add napi-derive --optional

2. 创建 src/napi.rs
   - 导出 render(), parse(), check(), fix()
   - 使用 #[napi] 宏标记

3. 配置 package.json
   {
     "name": "@hao430/mermaid-cli-napi",
     "main": "index.node"
   }

4. 配置 Cargo.toml
   [lib]
   crate-type = ["cdylib"]
   
   [[example]]
   name = "napi"
   crate-type = ["cdylib"]

5. 构建
   cargo napi build --release

6. 发布到 npm
   npm publish --tag v1
```

### 时间估计
- 依赖配置: 5 分钟
- 导出函数: 15 分钟
- 构建测试: 10 分钟
- 文档: 5 分钟
**总计: 35 分钟**

### 优势
- 比 WASM 更快（直接原生调用）
- 跨平台编译支持（arm64, x86_64）
- 自动生成 TypeScript 类型定义

---

## 方案 2: Python 绑定 (PyO3)

### 实现步骤
```
1. 添加依赖
   cargo add pyo3 --optional -F "extension-module"

2. 创建 src/python.rs
   use pyo3::prelude::*;

   #[pyfunction]
   fn render(code: String) -> PyResult<String> { ... }

3. 配置 Cargo.toml
   [dependencies]
   pyo3 = { version = "0.20", optional = true }

4. 配置 pyproject.toml
   [build-system]
   requires = ["maturin>=1.0,<2.0"]

5. 构建
   maturin develop -r

6. 发布到 PyPI
   maturin publish
```

### 时间估计
- 依赖配置: 5 分钟
- 导出函数: 15 分钟
- 构建测试: 10 分钟
- 文档: 5 分钟
**总计: 35 分钟**

### 优势
- Python 数据科学生态集成（pandas, jupyter）
- 简单易用的 Python API
- 跨平台支持

---

## 推荐实现顺序

### 第一优先（这周）
1. **Node.js napi-rs 绑定** — 生态最活跃，用户基数大
2. **Python PyO3 绑定** — 数据科学/研究领域

### 第二优先（后续）
3. VS Code 扩展 — 基于 Node.js 绑定
4. 性能基准测试 — CI 对比 vs mmdc

---

## 网络问题解决方案

### 当前问题
crates.io 下载超时 → napi 依赖获取失败

### 替代方案
1. **使用 cargo mirror**
   ```
   [source.crates-io]
   registry = "sparse+https://mirrors.sjtug.sjtu.edu.cn/crates.io-index"
   ```

2. **离线构建**
   ```
   cargo vendor
   cargo build --offline
   ```

3. **创建独立项目**
   - 在 `bindings/node-native` 中创建单独项目
   - 不依赖主项目的 Cargo 工作区配置

### 推荐做法
创建独立的 Node.js/Python 绑定项目：
```
mermaid-cli/
  src/           (核心库)
  bindings/
    node/        (Node.js napi-rs 绑定)
    python/      (Python PyO3 绑定)
    wasm/        (现有的 WASM)
```

---

## 快速启动 (如果网络恢复)

### Node.js 绑定快速启动
```bash
# 在 bindings/node 目录
cd bindings/node
npm init -y

# 创建最小化 package.json
cat > package.json << 'EOF'
{
  "name": "@hao430/mermaid-cli",
  "version": "0.1.0",
  "type": "commonjs",
  "main": "index.js",
  "napi": {
    "name": "mermaid_cli"
  }
}
EOF

# 配置 Cargo.toml
[lib]
crate-type = ["cdylib"]

# 导出 #[napi] 函数
# 构建: cargo napi build --release
```

---

## 测试计划

### Node.js 绑定测试
```javascript
const mc = require('@hao430/mermaid-cli');

const svg = mc.render('graph TD; A-->B');
console.log(svg.substring(0, 50)); // 验证 SVG 输出

const valid = mc.check('graph TD; A-->B');
console.log(valid); // true

const fixed = mc.fix('grpah TD; A-->B');
console.log(fixed); // 验证修复
```

### Python 绑定测试
```python
import mermaid_cli

svg = mermaid_cli.render('graph TD; A-->B')
print(svg[:50])  # 验证 SVG 输出

valid = mermaid_cli.check('graph TD; A-->B')
print(valid)  # True

fixed = mermaid_cli.fix('grpah TD; A-->B')
print(fixed)  # 验证修复
```

---

## 发布时间表

| 项目 | 实现时间 | 发布时间 | 优先级 |
|------|---------|--------|--------|
| Node.js | 35 分钟 | 周一 | 🔴 高 |
| Python | 35 分钟 | 周二 | 🟡 中 |
| VS Code | 2 小时 | 周三 | 🟢 低 |
| CI 对比 | 1 小时 | 周四 | 🟡 中 |

---

## 后续改进

### v0.2.0 目标
- [ ] Node.js 原生绑定（napi-rs）
- [ ] Python 绑定（PyO3）
- [ ] VS Code 语言扩展
- [ ] 完整的 CI 基准测试

### v0.3.0+ 目标
- [ ] Go 绑定
- [ ] C# 绑定
- [ ] Java 绑定

---

**最后更新:** 2026-07-10 17:00 UTC+8  
**状态:** 计划已制定，待网络恢复后实施

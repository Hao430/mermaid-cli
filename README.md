# 🎨 Mermaid CLI - High-Performance Rust Implementation

> **English** | [简体中文](./README_ZH.md)

> **A high-performance, zero-dependency Mermaid diagram rendering engine built in Rust, engineered for AI systems, automated pipelines, and production environments.**

[![License](https://img.shields.io/badge/license-MIT-blue)](./LICENSE)
![Version](https://img.shields.io/badge/version-v0.2.1--alpha-green)
![Rust](https://img.shields.io/badge/Rust-1.70+-orange)
[![Online Studio](https://img.shields.io/badge/Playground-Online%20Studio-blueviolet)](./site/playground.html)
![Status](https://img.shields.io/badge/status-production--ready-brightgreen)

---

## ⚡ Core Advantages

| Advantage | mmdc (Official Node.js) | Rust Version | Comparison |
|-----------|-------------------------|--------------|------------|
| **Single Diagram Render** | 2.9 s | 791 ms | **3.7x Faster** |
| **Cold Start** | 400 ms | 5 ms | **80x Faster** |
| **Memory Footprint** | 300–500 MB | <30 MB | **10–15x Lower** |
| **Deployment Complexity** | Requires Chromium & Node.js | Single standalone binary | **Zero Dependencies** |
| **Container Image Size** | 1.2 GB | ~70 MB | **17x Smaller** |

See detailed benchmarks in [📊 Actual Performance Test](./docs/COMPARISON_ACTUAL_TEST.md).

---

## 🎯 Features

### ✅ Supports 23 Diagram Types

- **Fundamentals** — Flowchart, State Diagram, Class Diagram  
- **Interaction** — Sequence Diagram, Journey, ZenUML  
- **Data & Metrics** — Pie Chart, XY Chart, Sankey, Radar, Treemap  
- **Planning & Management** — Gantt Chart, Timeline, Kanban  
- **Architecture** — Architecture, C4 Diagram, Block Diagram, Mind Map  
- **Relationships** — ER Diagram, Requirement Diagram, Ishikawa (Fishbone)  
- **Specialized** — Git Graph, Venn Diagram, Packet Diagram, Quadrant Chart  

### 🚀 Built for AI & Agentic Workflows

- ✅ **Sub-Second Streaming** — Millisecond-level rendering suitable for interactive AI agents and real-time streaming feedback.
- ✅ **Auto Error Recovery** — Built-in syntax analysis and automatic healing for common LLM typo patterns.
- ✅ **AST Inspection** — Direct Abstract Syntax Tree export in JSON format for downstream programmatic processing.
- ✅ **Dual-Format Rendering** — Native vector SVG and high-resolution raster PNG output without headless browsers.
- ✅ **Full Multilingual & CJK Support** — Seamless rendering of Chinese, Japanese, Korean, and non-ASCII characters across labels, notes, and styles.

### 🏗️ Production-Ready Reliability

- ✅ **Compile Once, Run Anywhere** — Self-contained binary without dynamic runtime dependencies.
- ✅ **Zero Headless Browsers** — No Chromium, Puppeteer, or Node.js runtime overhead.
- ✅ **Container-Friendly** — Minimal base container image (<100MB) with sub-100ms startup latency.
- ✅ **100% Offline Capable** — Runs completely isolated without external network calls or font CDN dependencies.
- ✅ **Strict Quality Assurance** — 0 compiler warnings, strict clippy standards, and 330+ unit, integration, and snapshot tests.

---

## 🚀 Quick Start

### 1. Installation

#### Build from Source

```bash
# Clone the repository
git clone https://github.com/hao430/mermaid-cli.git
cd mermaid-cli

# Compile with PNG support
cargo build --release --features png

# Standalone binary available at:
# ./target/release/mermaid-cli
```

### 2. Basic Usage

```bash
# Render directly to SVG (default)
./target/release/mermaid-cli input.mmd -o output.svg

# Render to PNG
./target/release/mermaid-cli input.mmd -o output.png

# Stream from standard input (ideal for AI pipelines)
echo "graph TD; A[Input]-->B[Processing]-->C[Output]" | \
  ./target/release/mermaid-cli --stdin -o output.png
```

### 3. Advanced Commands & Flags

```bash
# Syntax verification
./target/release/mermaid-cli check diagram.mmd

# Automatic syntax healing and repair
./target/release/mermaid-cli fix broken.mmd -o fixed.mmd

# Apply built-in themes (default, forest, dark, neutral)
./target/release/mermaid-cli diagram.mmd -t dark -o output.svg

# Custom dimensions and scale factor
./target/release/mermaid-cli diagram.mmd -w 1920 -H 1080 -s 2 -o high_res.png

# Export diagram AST as structured JSON (requires --features json)
./target/release/mermaid-cli diagram.mmd --json
```

---

## 📊 Benchmark Data

### Real-World Sequence Diagram Benchmark

*Benchmark specification: Complex Sequence Diagram with 5 participants and 16 sequential messages.*

```
Rust mermaid-cli:
  SVG Generation : 25 ms
  PNG Rasterize  : 766 ms
  Total Runtime  : 791 ms  ✅
  Peak Memory    : <30 MB

mmdc (Official Node.js CLI):
  Node.js Startup   : ~100 ms
  Chromium Launch   : ~2000 ms
  Page Layout       : ~500 ms
  Screenshot Capture: ~200 ms
  Total Runtime     : ~2900 ms ⚠️
  Peak Memory       : 300–500 MB
```

### High-Concurrency & Batch Rendering

```
Batch Processing: 1,000 Diagrams

mmdc:
  Sequential Time: 2,900 s (~48 minutes) ❌
  50 Workers     : ~60 s, Memory: 50 × 500 MB = 25 GB ❌ (High risk of OOM)

Rust mermaid-cli:
  Sequential Time: ~800 s (~13 minutes) ✅
  50 Workers     : ~16 s, Memory: 50 × 30 MB = 1.5 GB ✅ (Lightweight & stable)
```

Read the full report in [📊 Full Comparison & Benchmarks](./docs/COMPARISON_ACTUAL_TEST.md).

---

## 📚 Documentation

| Document | Description | Est. Reading Time |
|----------|-------------|-------------------|
| **[QUICK_START.md](./docs/QUICK_START.md)** | Quick start tutorial and installation guide | ⏱️ 5 min |
| **[FLOWCHART_GUIDE.md](./docs/FLOWCHART_GUIDE.md)** | Flowcharts, subgraphs, and styling guide | ⏱️ 15 min |
| **[SEQUENCE_DIAGRAM_GUIDE.md](./docs/SEQUENCE_DIAGRAM_GUIDE.md)** | Comprehensive Sequence diagram manual | ⏱️ 15 min |
| **[API.md](./docs/API.md)** | Rust crate and CLI parameter API reference | ⏱️ 20 min |
| **[ARCHITECTURE.md](./docs/ARCHITECTURE.md)** | Internal architecture, lexer, AST, and renderer | ⏱️ 30 min |
| **[COMPARISON_ACTUAL_TEST.md](./docs/COMPARISON_ACTUAL_TEST.md)** | In-depth comparison against official `mmdc` | ⏱️ 15 min |
| **[RELEASE_NOTES.md](./RELEASE_NOTES.md)** | Release notes and version changelogs | ⏱️ 10 min |

---

## 💡 Practical Examples

### AI Agent Real-Time Diagram Generation (Python)

```python
import subprocess

diagram_code = """
graph TD
    User([User Prompt]) --> Agent[AI Agent]
    Agent --> Plan{Task Planning}
    Plan -->|Code| Gen[Code Generator]
    Plan -->|Diagram| Mermaid[Rust mermaid-cli]
    Mermaid --> Result[Rendered PNG / SVG]
"""

# Instant sub-second render
process = subprocess.run(
    ["mermaid-cli", "--stdin", "-o", "workflow.png"],
    input=diagram_code.encode("utf-8"),
    capture_output=True,
    check=True,
    timeout=3,
)
```

### Automated Documentation Generation (Bash)

```bash
#!/usr/bin/env bash
# Recursively render all .mmd diagrams in docs
find docs -type f -name "*.mmd" | while read -r file; do
  out="${file%.mmd}.png"
  echo "Rendering: $file -> $out"
  mermaid-cli "$file" -o "$out" -q
done
```

### Lightweight Scratch Docker Container

```dockerfile
FROM scratch
COPY target/release/mermaid-cli /
COPY /usr/share/fonts /fonts

ENTRYPOINT ["/mermaid-cli"]
CMD ["--help"]

# Image size: ~70 MB (over 17x smaller than official Docker images)
```

---

## 🎯 Project Status & Roadmap

### ✅ Completed Milestones

- ✅ Basic & complex Flowcharts with `subgraph` and `classDef` styling
- ✅ Full Sequence Diagrams with activation, notes, loops, and options
- ✅ Zero-Chromium native PNG & SVG generation
- ✅ 23+ diagram formats supported
- ✅ Multilingual and CJK font rendering
- ✅ Built-in syntax error auto-fixer
- ✅ Online Playground and interactive WebAssembly build
- ✅ Zero compiler warnings, 330+ test suite pass

### 📅 Roadmap

| Version | Date | Key Highlights |
|---------|------|----------------|
| **v0.1.0-alpha** | 2026-06 | Initial prototype and core AST parser |
| **v0.2.0-alpha** | 2026-06 | SVG layout optimization and baseline benchmarks |
| **v0.2.1-alpha** | **2026-07** | **Native PNG rasterization, production ready** |
| **v0.3.0 (Planned)** | 2026-08 | Native PDF generation, extended styling enhancements |

---

## 🏗️ Project Architecture

```
mermaid-cli/
├── src/
│   ├── main.rs              # CLI entry point and argument parsing
│   ├── lib.rs               # Library API interface
│   ├── parser/              # Lexer, AST, and diagram parsers (23 formats)
│   ├── renderer/            # Layout engines and vector positioning
│   ├── svg/                 # High-performance SVG builder
│   ├── fixer/               # Syntax diagnostics and auto-healing
│   └── utils/               # Geometry, typography, and utility helpers
├── docs/                    # 📚 Documentation and design specifications
├── examples/                # 📝 Sample diagrams and rendering outputs
├── site/                    # 🌐 WebAssembly playground and interactive studio
├── tests/                   # 🧪 Integration, benchmark, and snapshot tests
├── Cargo.toml              # Rust package configuration
├── README.md               # English Documentation (Default)
└── README_ZH.md            # Chinese Documentation
```

---

## 🧪 Testing & Code Quality

Run the test suite, linting, and formatting checks:

```bash
# Run unit and integration tests
cargo test --all-features

# Run Clippy static analysis
cargo clippy --all-targets --all-features

# Verify formatting
cargo fmt --check
```

---

## 📈 When to Use Rust `mermaid-cli`

### ✅ Recommended Use Cases

- **AI & LLM Workflows** — Real-time diagram rendering requiring sub-second response times.
- **High-Throughput Batch Processing** — Thousands of diagrams rendered within tight compute constraints.
- **Containerized Workloads** — Kubernetes, Cloud Functions, and Docker deployments requiring ultra-small images (<100MB).
- **CI/CD Pipelines** — Fast, dependable automated documentation builds without installing headless browsers.
- **Resource-Constrained Environments** — Edge servers or embedded systems with limited RAM (<50MB).
- **Air-Gapped / Offline Tooling** — Complete isolation from remote CDNs and external network resources.

### ⚠️ Scenarios to Keep `mmdc`

- Complete browser-engine DOM CSS cascading stylesheet dependencies.
- Legacy pipelines requiring tight coupling with JavaScript browser runtime APIs.

---

## 🤝 Contributing

Contributions are welcome!

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/awesome-feature`)
3. Commit your changes (`git commit -m 'feat: add awesome feature'`)
4. Push to the branch (`git push origin feature/awesome-feature`)
5. Open a Pull Request

Please refer to [CONTRIBUTING.md](./CONTRIBUTING.md) for more details.

---

## 📄 License

This project is licensed under the MIT License — see the [LICENSE](./LICENSE) file for details.

---

## 🙏 Acknowledgments

- **[Mermaid.js](https://mermaid.js.org/)** — For the diagramming specification and ecosystem design.
- **[Rust Community](https://www.rust-lang.org/)** — For the ecosystem, compiler, and tools.
- All contributors, issue reporters, and users.

---

<div align="center">

**⭐ If you find this project helpful, please give it a star on GitHub!**

[GitHub Repository](https://github.com/hao430/mermaid-cli) | [Documentation](./docs/) | [Issues & Feedback](https://github.com/hao430/mermaid-cli/issues)

</div>

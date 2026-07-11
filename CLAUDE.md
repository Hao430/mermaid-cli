# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Mermaid CLI** is a high-performance, zero-dependency Mermaid diagram rendering engine written in Rust. It supports 23+ diagram types (Flowchart, Sequence Diagram, Pie Chart, etc.) with SVG and PNG output.

**Key metrics**: v0.2.1-alpha, production-ready, 3-4x faster than mmdc (official Node.js version), <30MB memory footprint.

---

## Build & Development Commands

### Core Build Tasks

```bash
# Build debug binary (dev mode)
cargo build

# Build optimized release binary (production)
cargo build --release
# Output: target/release/mermaid-cli (~50MB, fully optimized)

# Build with PNG support (default, includes resvg)
cargo build --release --features png

# Build with JSON support (default feature)
cargo build --release --features json

# Build for WASM (browser version)
cargo build --release --features wasm --target wasm32-unknown-unknown
```

### Testing & Quality

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test file
cargo test --test api_tests

# Run benchmarks
cargo test --test render_benchmarks

# Lint code (clippy)
cargo clippy --all-targets --all-features

# Format check (no changes)
cargo fmt --check

# Auto-format code
cargo fmt

# Combined: test + clippy + fmt (CI pipeline)
cargo test && cargo clippy && cargo fmt --check
```

### Common Development Workflows

```bash
# Full validation before commit (runs in CI)
cargo test && cargo clippy && cargo fmt --check

# Generate release binary for testing
cargo build --release && ./target/release/mermaid-cli examples/test.mmd -o out.png -e png

# Quick iteration: build + run
cargo build && echo "graph TD; A[test]" | ./target/debug/mermaid-cli --stdin -o test.svg

# Profile performance
cargo build --release
time ./target/release/mermaid-cli examples/shopping_sequence.mmd -o out.png -e png
```

---

## Architecture & Code Organization

### High-Level Data Flow

```
Input (Mermaid syntax)
    ↓
[PARSER] → [AST] → [RENDERER] → [SVG Builder]
    ↓         ↓          ↓            ↓
Lexer    Statements  Layout Logic   SVG XML
           Tree       Algorithms    Elements
    ↓
[OUTPUT]
  ├── SVG (direct output)
  └── PNG (via resvg conversion)
```

### Module Breakdown

**`src/parser/`** — Syntax parsing and AST construction
- **`lexer.rs`** — Tokenizes Mermaid syntax (keywords, identifiers, operators)
- **`mod.rs`** — Parser implementation; detects diagram type (Flowchart, Sequence, etc.) and dispatches to appropriate parse function
- **`ast.rs`** — AST node definitions; `DiagramType` enum lists all 23 supported types; `Statement` enum represents parsed elements (nodes, edges, messages)
- **Key insight**: Parser is modular—each diagram type can have its own parse function (e.g., `parse_sequence()`, `parse_flowchart()`)

**`src/renderer/`** — Core rendering logic
- **`mod.rs`** — Main rendering engine; orchestrates layout algorithms and SVG generation
- **Layout algorithms**:
  - Flowchart: Hierarchical layout (Sugiyama algorithm with Barycenter sorting for edge-crossing reduction)
  - Sequence: Participant positioning + message timeline layout
  - Others: Specialized algorithms per diagram type
- **Key insight**: Barycenter sorting (implemented here) is critical for reducing edge crossings in hierarchical diagrams (saves 75%+ crossings)
- **Font handling**: `fontdb` initialization (lines ~350-365) is essential for text rendering in PNG output

**`src/svg/`** — SVG generation
- **`mod.rs`** — Low-level SVG element builders (rectangles, circles, paths, text)
- **`add_text()` method** — Detects HTML `<br/>` tags and converts to SVG `<tspan>` elements for multi-line support
- **`set_dimensions()` method** — Dynamically extracts and applies canvas size from rendered SVG tree (prevents image cropping)

**`src/fixer/`** — Syntax validation and auto-repair
- Detects common Mermaid syntax errors and provides fixes
- Used by `mermaid-cli check` and `mermaid-cli fix` commands

**`src/main.rs`** — CLI entry point
- Argument parsing
- File I/O handling
- Feature selection (SVG vs PNG vs JSON output)

**`src/lib.rs`** — Library interface
- Public API for embedding mermaid-cli as a library
- `pub fn parse()`, `pub fn render()`, `pub fn check()`

**`src/wasm.rs`** — WebAssembly bindings (future; browser support)

---

## Critical Implementation Details

### PNG Rendering Pipeline

1. **Parser** generates AST
2. **Renderer** produces SVG XML string
3. **SVG tree construction** via `usvg::Tree::from_str()`
4. **Dimension extraction**: `rtree.size()` → extract actual width/height (not hardcoded)
5. **Font initialization**: `fontdb::Database` + `load_system_fonts()` → enables text rendering
6. **Rasterization**: `resvg::render()` with Tiny-Skia backend → PNG bytes

**Why it matters**: Step 4 (SVG dimension extraction) and Step 5 (fontdb init) are the most critical fixes in v0.2.1. Without them, PNG output fails entirely.

### Sequence Diagram Specifics

- **Participants** are arranged horizontally; each gets a vertical lifeline
- **Messages** are routed between participants with arrows and labels
- **Activation boxes** (gray rectangles) mark periods when a participant is active
- **Condition blocks** (alt/else) span all participants as 4-sided rectangles with internal dividers
- **Layout challenge**: Vertical spacing must account for activation boxes and condition blocks to prevent text overlap (see lines ~2940-2995 in renderer/mod.rs)

### Flowchart Layout (Barycenter Sorting)

The Sugiyama algorithm is used to minimize edge crossings:

1. **Layer assignment**: BFS from root nodes to assign depth (tree-aware, not topological sort)
2. **Node ordering within layers**: **Barycenter method** — sort nodes by the average X position of their parents
   - This reduces crossings by 75%+ compared to naive left-to-right ordering
   - Implementation: `process_seq_stmts()` function in renderer/mod.rs (~line 2700)

**Key insight**: The user's insight "swap these two participants and third layer has no crosses" directly inspired implementing Barycenter sorting, not arbitrary heuristics.

---

## Feature Flags & Build Variants

| Flag | Purpose | Default? |
|------|---------|----------|
| `png` | PNG rendering via resvg | Yes (cargo build --release --features png) |
| `json` | JSON output for AST | Yes |
| `wasm` | Browser/WASM compilation | No (explicit: --features wasm) |

**Common combinations**:
- `cargo build --release` — SVG + JSON (fast, no resvg dependency)
- `cargo build --release --features png` — Full pipeline (SVG + PNG + JSON)
- `cargo build --release --features wasm --target wasm32-unknown-unknown` — Browser build

---

## Testing Strategy

### Test Files

- **`tests/api_tests.rs`** — Library API tests (parse, render, check functions)
- **`tests/render_benchmarks.rs`** — Performance comparisons (used for mmdc vs Rust benchmarking)
- **`tests/coverage_tests.rs`** — Example-based rendering (all diagram types)

### Running Tests

```bash
# Run all
cargo test

# Run specific test
cargo test test_flowchart_parsing

# Run with backtraces
RUST_BACKTRACE=1 cargo test
```

### Example-Based Testing

The `examples/` directory contains real Mermaid files used for manual verification:
- `shopping_sequence.mmd` — 7-participant sequence diagram
- `checkout_flow.mmd` — Flowchart with decision diamonds
- `oa_reimbursement_flow.mmd` — Sequence with conditions
- `org_chart_*.mmd` — Organization charts (test layout algorithms)

**Workflow**: Make changes → `cargo build --release` → `./target/release/mermaid-cli examples/X.mmd -o out.png -e png` → visually inspect output

---

## Performance Profiling & Optimization

### Key Bottlenecks (v0.2.1)

1. **SVG → PNG conversion** (~766ms for typical diagram)
   - Dominated by resvg rasterization, not fixable without major architecture change
   - Acceptable for production; mmdc takes 2900ms for comparison

2. **Layout algorithms** (milliseconds scale)
   - Barycenter sorting is O(n²) but critical for output quality
   - Tree-aware BFS for depth assignment is O(n)

3. **Text rendering** (milliseconds scale)
   - fontdb initialization happens once per run (~5ms)
   - Individual text element rendering is cheap

### Benchmarking Command

```bash
# Generate timing info
time ./target/release/mermaid-cli examples/shopping_sequence.mmd -o /tmp/out.png -e png

# Expected: ~800ms total (25ms SVG + 766ms PNG)
```

---

## Common Gotchas & Debugging

### "Text Not Showing in PNG"
- **Cause**: fontdb not initialized (see `src/renderer/mod.rs` line ~350)
- **Fix**: Ensure `fontdb::Database::new()` and `load_system_fonts()` are called in PNG pipeline
- **Check**: `usvg::Options` must have `fontdb` set via `Arc<Database>`

### "PNG Image Cropped or Wrong Size"
- **Cause**: Hardcoded 800×600 canvas size or SVG tree not queried for actual dimensions
- **Fix**: Extract size from parsed SVG tree: `let svg_size = rtree.size()`
- **Code location**: `src/renderer/mod.rs`, lines ~380-400

### "HTML Tags Like `<br/>` Appear as Literal Text"
- **Cause**: SVG text elements don't natively support HTML; need to convert to `<tspan>` elements
- **Fix**: See `src/svg/mod.rs` `add_text()` method; detects `<br/>` and splits into multiple tspan elements with dy offsets
- **Example**: `"Line1<br/>Line2"` → `<tspan>Line1</tspan><tspan dy="16">Line2</tspan>`

### "Edge Crossings Are Excessive"
- **Cause**: Nodes not sorted by parent position within layers
- **Fix**: Implement Barycenter sorting in layout algorithm (see `process_seq_stmts()`)
- **Impact**: Reduces crossings 75%+ in hierarchical diagrams

### "Condition Boxes Not Rendering (Sequence Diagram)"
- **Cause**: Missing implementation of `alt`/`else` block boundaries
- **Fix**: Render 4-sided rectangles spanning all participants; see `src/renderer/mod.rs` lines ~2940-2995
- **Details**: Dynamic width calculation, white label background to prevent occlusion, spacing adjustment for post-condition messages

---

## Code Style & Conventions

- **Formatting**: `cargo fmt` (standard Rust style)
- **Linting**: `cargo clippy` (no warnings on main branch)
- **Comments**: Minimal, only for non-obvious logic (e.g., algorithm explanations, workarounds)
- **Naming**: CamelCase for types (`DiagramType`), snake_case for functions (`parse_flowchart`)
- **Error handling**: `Result<T, ParseError>` pattern; propagate via `?` operator

---

## CI/CD Pipelines

Located in `.github/workflows/`:

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| **test.yml** | Push + PR | `cargo test`, `cargo clippy`, `cargo fmt --check` |
| **build.yml** | Push + PR | Cross-platform compilation (Linux/macOS/Windows) |
| **release.yml** | Git tag `v*` | Build binaries, create GitHub Release |
| **phase4-tests.yml** | Manual | Detailed performance & rendering validation |

---

## Version & Release Process

- **Current version**: v0.2.1-alpha (production-ready; PNG fully functional)
- **Release tags**: `git tag v0.2.1-alpha && git push origin v0.2.1-alpha`
- **Changelog**: `RELEASE_NOTES.md` and `docs/RELEASE_SUMMARY.md`

---

## Documentation

Key docs for developers:

- **`docs/ARCHITECTURE.md`** — Deep dive into system design
- **`docs/SEQUENCE_DIAGRAM_GUIDE.md`** — Sequence diagram semantics and edge cases
- **`docs/FLOWCHART_GUIDE.md`** — Flowchart node types and layout rules
- **`docs/COMPARISON_ACTUAL_TEST.md`** — Performance vs mmdc (useful for understanding bottlenecks)
- **`README.md`** — Quick reference and usage examples

---

## Useful Git Shortcuts

```bash
# See recent changes (helpful for understanding what changed)
git log --oneline -10

# See what changed in a specific module
git log --oneline src/renderer/mod.rs

# Diff against main
git diff main

# View a specific file at a commit
git show <commit>:src/renderer/mod.rs
```

---

## Performance-Critical Insights

1. **Barycenter sorting is the magic**: Reduces edge crossings 75%+. Do not remove or simplify.
2. **fontdb initialization is non-negotiable**: PNG text rendering depends on it. Check it's in the PNG pipeline before troubleshooting text issues.
3. **SVG dimension extraction matters**: Always extract from `rtree.size()`, not hardcoded values. Prevents image cropping.
4. **Activation box spacing**: Must account for full line height, not just 10px. Prevents text overlap in sequence diagrams.

---

## When to Reach for External Tools

- **Memory profiling**: `valgrind` (on Linux) or Instruments (macOS)
- **Performance profiling**: `cargo build --release && perf record -F 99 ./target/release/mermaid-cli ...`
- **AST inspection**: Use `-e json` output option to inspect parsed structure

---

## Summary: Before Making Changes

1. **Understand the data flow**: Input → Parser → AST → Renderer → SVG → (PNG if requested)
2. **Identify the module**: Parser, Renderer, SVG, or Fixer?
3. **Check the tests**: `cargo test` to ensure baseline passes
4. **Run formatter**: `cargo fmt` before committing
5. **Build release binary**: `cargo build --release --features png` and manually test with an example
6. **Document non-obvious logic**: Use comments sparingly, only for algorithm intent

Good luck! The codebase is well-structured and the main challenges are in the renderer module (layout algorithms, SVG construction) and the PNG pipeline (font initialization, dimension extraction).

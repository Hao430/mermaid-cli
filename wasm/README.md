# mermaid-cli WASM

WebAssembly build of [mermaid-cli](https://github.com/Hao430/mermaid-cli) - a high-performance Mermaid diagram renderer.

## 100x faster than mmdc

- **Zero browser dependency** - No Puppeteer, no Chromium
- **< 1MB binary** - Tiny footprint
- **~1ms per diagram** - Blazing fast
- **All 18 chart types** - Flowchart, Sequence, Class, State, ER, Gantt, Pie, Mindmap, GitGraph, Timeline, Journey, Kanban, Venn, Packet, Radar, Ishikawa, Quadrant, ZenUML, Requirement, Block, C4, Architecture, XY Chart, Sankey, Treemap

## Build

```bash
# Install tools
rustup target add wasm32-unknown-unknown
cargo install wasm-pack

# Build for browser
wasm-pack build --target web --features wasm,json --out-dir wasm/pkg

# Or build for Node.js
wasm-pack build --target nodejs --features wasm,json --out-dir wasm/pkg
```

## Usage (Browser)

```html
<script type="module">
  import init, { render, parse, check } from "./pkg/mermaid_cli.js";
  
  await init();
  
  // Render a flowchart
  const svg = render("graph TD; A-->B");
  document.getElementById("diagram").innerHTML = svg;
  
  // Parse to JSON AST
  const ast = parse("sequenceDiagram\\nA->B: Hello");
  console.log(JSON.parse(ast));
  
  // Check syntax
  const result = check("graph TD; A-->B");  
  console.log(result.valid ? "✓ Valid" : "✗ Invalid");
</script>
```

## Usage (Node.js)

```javascript
const { render, parse, check } = require("./pkg/mermaid_cli.js");

const svg = render("graph TD; A-->B");
console.log(svg);
```

## API

| Function | Description | Returns |
|----------|-------------|---------|
| `render(code)` | Render diagram to SVG | `string` |
| `parse(code)` | Parse to JSON AST | `string` |
| `check(code)` | Check syntax | JSON string |

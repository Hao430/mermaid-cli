---
name: new-chart-types-timeline-gitgraph-mindmap
description: How to implement new chart types - pattern used for Mindmap, GitGraph, and Timeline
metadata:
  type: reference
---

New chart types follow this implementation pattern:

1. **AST** (`src/parser/ast.rs`):
   - Add variant to `DiagramType` enum
   - Add `Statement` variants for the chart type
   - Add match arms in `get_nodes()` and `get_node_labels()` for the new Statement variants

2. **Lexer** (`src/parser/lexer.rs`):
   - Add keyword to `read_identifier()` keyword match list
   - Add keyword to the `test_lexer_diagram_all_keywords` test array

3. **Parser** (`src/parser/mod.rs`):
   - Add keyword dispatch in `parse()` method (before the `_ =>` arm)
   - Update error message string
   - Add `parse_X()` method using `_source` for line-based parsing (unlike token-based parsing for Flowchart/Sequence)
   - Return `Ok(Diagram { diagram_type, statements, direction: None, subgraphs: vec![], title: None })`

4. **Renderer** (`src/renderer/mod.rs`):
   - Add `if diagram.diagram_type == DiagramType::X { return self.render_X(diagram); }` dispatch
   - Implement `render_X()` using `SvgBuilder`

5. **Tests**:
   - Add parse, render, diagram_type, check_valid, and svg_structure tests
   - Add CLI test from stdin

Chart types implemented so far with this pattern:
- **Mindmap**: Tree structure (indentation-based), renders as left-to-right indented tree with colored depth levels
- **GitGraph**: Timeline with branches/commits/merges, renders as horizontal lanes with colored dots and connector lines
- **Timeline**: Events with time descriptions, renders as horizontal axis with evenly-spaced markers and labels

**Why**: Using Statement:: variants rather than separate struct fields keeps Diagram struct stable and avoids modifying existing code.

**How to apply**: Follow the same 5-step pattern for Journey (next simplest remaining chart type).

## MODIFIED Requirements

### Requirement: Node label parsing
The parser SHALL correctly parse node syntax with labels AND shapes. The parser SHALL extract both the label text and the shape from the包围 token sequence, producing a `NodeDef` with the correct `shape` field.

#### Scenario: Parse node with label
- **WHEN** input is `graph TD; A[Start]-->B[End]`
- **THEN** parser produces Diagram with:
  - Node A with label "Start" and shape Rect
  - Node B with label "End" and shape Rect
  - Edge from A to B

#### Scenario: Parse node without label
- **WHEN** input is `graph TD; A-->B`
- **THEN** parser produces Diagram with:
  - Node A with no label (None) and shape Rect
  - Node B with no label (None) and shape Rect
  - Edge from A to B

#### Scenario: Parse mixed nodes
- **WHEN** input is `graph TD; A[Start]-->B`
- **THEN** parser produces Diagram with:
  - Node A with label "Start" and shape Rect
  - Node B with no label (None) and shape Rect
  - Edge from A to B

### Requirement: Label display in rendering
The renderer SHALL display the label text for nodes that have labels, falling back to the node ID for nodes without labels. The renderer SHALL use the node's shape to determine the SVG graphic.

#### Scenario: Render node with label
- **WHEN** rendering a node with id "A" and label "Start"
- **THEN** SVG output contains the text "Start"

#### Scenario: Render node without label
- **WHEN** rendering a node with id "A" and no label
- **THEN** SVG output contains the text "A"

### Requirement: Backward compatibility
The parser SHALL maintain full backward compatibility with existing `A-->B` syntax.

#### Scenario: Existing syntax works unchanged
- **WHEN** input is `graph TD; A-->B`
- **THEN** parser produces the same result as before this change

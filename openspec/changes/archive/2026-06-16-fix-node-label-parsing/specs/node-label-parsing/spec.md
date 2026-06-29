## ADDED Requirements

### Requirement: Node label parsing
The parser SHALL correctly parse `id[label]` syntax as a single node with an identifier and a label.

#### Scenario: Parse node with label
- **WHEN** input is `graph TD; A[Start]-->B[End]`
- **THEN** parser produces Diagram with:
  - Node A with label "Start"
  - Node B with label "End"
  - Edge from A to B

#### Scenario: Parse node without label
- **WHEN** input is `graph TD; A-->B`
- **THEN** parser produces Diagram with:
  - Node A with no label (None)
  - Node B with no label (None)
  - Edge from A to B

#### Scenario: Parse mixed nodes
- **WHEN** input is `graph TD; A[Start]-->B`
- **THEN** parser produces Diagram with:
  - Node A with label "Start"
  - Node B with no label (None)
  - Edge from A to B

### Requirement: Label display in rendering
The renderer SHALL display the label text for nodes that have labels, falling back to the node ID for nodes without labels.

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

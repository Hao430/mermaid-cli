## ADDED Requirements

### Requirement: Parse standard node shapes
The parser SHALL recognize Mermaid standard node shape syntax and assign the correct `NodeShape` variant to each `NodeDef` statement.

#### Scenario: Parse rounded node
- **WHEN** input is `graph TD; A(Rounded)`
- **THEN** parser produces NodeDef with id "A" and shape Circle (rounded rectangle)

#### Scenario: Parse diamond node
- **WHEN** input is `graph TD; A{Diamond}`
- **THEN** parser produces NodeDef with id "A" and shape Diamond

#### Scenario: Parse double-rounded node
- **WHEN** input is `graph TD; A([Double])`
- **THEN** parser produces NodeDef with id "A" and shape Rounded

#### Scenario: Parse subroutine node
- **WHEN** input is `graph TD; A[[Subroutine]]`
- **THEN** parser produces NodeDef with id "A" and shape Subroutine

#### Scenario: Parse cylinder node
- **WHEN** input is `graph TD; A[(Database)]`
- **THEN** parser produces NodeDef with id "A" and shape Cylinder

#### Scenario: Parse double-circle node
- **WHEN** input is `graph TD; A((Circle))`
- **THEN** parser produces NodeDef with id "A" and shape DoubleCircle

#### Scenario: Parse flag node
- **WHEN** input is `graph TD; A>Flag]`
- **THEN** parser produces NodeDef with id "A" and shape Flag

#### Scenario: Parse default rect node
- **WHEN** input is `graph TD; A[label]`
- **THEN** parser produces NodeDef with id "A" and shape Rect

#### Scenario: Parse node without shape syntax
- **WHEN** input is `graph TD; A-->B`
- **THEN** parser produces NodeDef with shape Rect

### Requirement: Render node shapes in SVG
The renderer SHALL produce distinct SVG shapes corresponding to each `NodeShape` variant.

#### Scenario: Render rounded node
- **WHEN** rendering a node with shape Circle
- **THEN** SVG contains a `<rect>` with `rx` attribute (rounded corners)

#### Scenario: Render diamond node
- **WHEN** rendering a node with shape Diamond
- **THEN** SVG contains a `<polygon>` with rotated diamond points

#### Scenario: Render cylinder node
- **WHEN** rendering a node with shape Cylinder
- **THEN** SVG contains a `<rect>` with a top ellipse (cylinder appearance)

#### Scenario: Render double-circle node
- **WHEN** rendering a node with shape DoubleCircle
- **THEN** SVG contains two nested `<circle>` elements

#### Scenario: Render flag node
- **WHEN** rendering a node with shape Flag
- **THEN** SVG contains a `<path>` with flag-like polygon

#### Scenario: Render subroutine node
- **WHEN** rendering a node with shape Subroutine
- **THEN** SVG contains a double-bordered rectangle

### Requirement: Shape with label
The parser SHALL support combining any shape syntax with a label inside the shape brackets.

#### Scenario: Diamond with label
- **WHEN** input is `graph TD; A{Is Valid?}`
- **THEN** parser produces NodeDef with shape Diamond and label "Is Valid?"

#### Scenario: Cylinder with label
- **WHEN** input is `graph TD; A[(PostgreSQL)]`
- **THEN** parser produces NodeDef with shape Cylinder and label "PostgreSQL"

### Requirement: Backward compatibility
All existing `A[label]` and `A-->B` syntax SHALL continue to work identically.

#### Scenario: Existing rect syntax unchanged
- **WHEN** input is `graph TD; A[Start]-->B[End]`
- **THEN** both nodes have shape Rect and correct labels

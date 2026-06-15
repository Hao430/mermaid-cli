## ADDED Requirements

### Requirement: Integration tests for CLI rendering
The system SHALL provide comprehensive integration tests that verify the CLI can correctly render Mermaid diagrams from files and stdin, outputting valid SVG.

#### Scenario: Render from file
- **WHEN** user runs `mermaid-cli input.mmd -o output.svg`
- **THEN** system reads `input.mmd`, renders the diagram, and writes valid SVG to `output.svg`

#### Scenario: Render from stdin
- **WHEN** user pipes Mermaid code to stdin: `echo 'graph TD; A-->B' | mermaid-cli --stdin -o output.svg`
- **THEN** system reads from stdin, renders the diagram, and writes valid SVG to the specified output file

#### Scenario: Output to stdout
- **WHEN** user runs `mermaid-cli input.mmd` without `-o` flag
- **THEN** system renders the diagram and outputs SVG to stdout

#### Scenario: Error handling - missing file
- **WHEN** user runs `mermaid-cli nonexistent.mmd -o output.svg`
- **THEN** system prints an error message and exits with code 1, without creating `output.svg`

#### Scenario: Error handling - invalid Mermaid code
- **WHEN** user provides invalid Mermaid syntax (e.g., `grpah TD; A-->B`)
- **THEN** system prints a parse error with line and column information, and exits with code 1

### Requirement: Integration tests for library API
The system SHALL provide comprehensive integration tests that verify the library API functions (`render()`, `parse()`, `check()`) work correctly end-to-end.

#### Scenario: render() produces valid SVG
- **WHEN** user calls `render("graph TD; A-->B")`
- **THEN** function returns a valid SVG string containing the diagram nodes and edges

#### Scenario: parse() returns correct AST
- **WHEN** user calls `parse("graph TD; A[Start]-->B[End]")`
- **THEN** function returns a Diagram with DiagramType::Flowchart and statements for the two nodes and one edge

#### Scenario: check() detects valid syntax
- **WHEN** user calls `check("graph TD; A-->B")`
- **THEN** function returns CheckResult with `valid: true` and empty errors list

#### Scenario: check() reports syntax errors
- **WHEN** user calls `check("grpah TD; A-->B")`
- **THEN** function returns CheckResult with `valid: false` and a parse error describing the unknown keyword

### Requirement: CI/CD pipeline for automated testing
The system SHALL have GitHub Actions workflows that automatically test code on every push and pull request.

#### Scenario: Test on push
- **WHEN** user pushes a commit to any branch
- **THEN** GitHub Actions runs `cargo test` and reports success/failure

#### Scenario: Test on pull request
- **WHEN** user opens a pull request
- **THEN** GitHub Actions runs `cargo test` and displays results in the PR checks

#### Scenario: Build release binary on tag
- **WHEN** user pushes a git tag matching `v*` (e.g., `v0.1.0`)
- **THEN** GitHub Actions compiles release binaries for Linux, macOS, and Windows, and creates a GitHub Release with the artifacts

### Requirement: Code quality enforcement
The system SHALL eliminate all compiler warnings and enforce code quality standards in CI.

#### Scenario: No compilation warnings
- **WHEN** CI runs `cargo build`
- **THEN** build succeeds with zero warnings

#### Scenario: Clippy lints pass
- **WHEN** CI runs `cargo clippy`
- **THEN** all clippy recommendations are applied or explicitly ignored with justification

pub mod fixer;
pub mod parser;
pub mod renderer;
pub mod svg;

/// Automatically fixes common syntax errors in Mermaid diagrams.
///
/// Detects and corrects typos (`grpah` → `graph`), arrow syntax issues
/// (`-->>` → `-->`), and missing `end` statements.
pub use fixer::Fixer;

/// Shape of a node in a flowchart diagram.
///
/// Variants: `Rect`, `Circle`, `Diamond`, `Rounded`, `Subroutine`,
/// `Cylinder`, `DoubleCircle`, `Flag`.
pub use parser::NodeShape;

/// A grouping of nodes within a subgraph block.
pub use parser::Subgraph;

/// Recursive descent parser that converts Mermaid source code into a
/// [`Diagram`](parser::Diagram) AST.
pub use parser::Parser;

/// A single statement in a diagram — either a node definition (`NodeDef`)
/// or an edge definition (`EdgeDef`).
pub use parser::Statement;

/// Renders a parsed [`Diagram`](parser::Diagram) AST into SVG output.
pub use renderer::Renderer;

/// Render a Mermaid diagram from source code to SVG.
///
/// Parses the Mermaid code and renders it into an SVG string.
/// Returns the SVG string on success, or an error message on failure.
///
/// # Example
///
/// ```
/// let result = mermaid_cli::render("graph TD; A-->B");
/// assert!(result.is_ok());
/// ```
pub fn render(code: &str) -> Result<String, String> {
    let mut parser = Parser::new(code);
    let diagram = parser.parse().map_err(|e| e.to_string())?;

    let renderer = Renderer::new();
    renderer.render(&diagram)
}

/// Parse Mermaid source code into a Diagram AST.
///
/// Returns the parsed Diagram on success, or a list of parse errors.
///
/// # Example
///
/// ```
/// let result = mermaid_cli::parse("graph TD; A-->B");
/// assert!(result.is_ok());
/// ```
pub fn parse(code: &str) -> Result<parser::Diagram, Vec<parser::ParseError>> {
    let mut parser = Parser::new(code);
    parser.parse().map_err(|e| vec![e])
}

/// Check if Mermaid source code has valid syntax.
///
/// Returns a `CheckResult` indicating whether the code is valid
/// and any associated errors.
///
/// # Example
///
/// ```
/// let result = mermaid_cli::check("graph TD; A-->B").unwrap();
/// assert!(result.valid);
/// ```
pub fn check(code: &str) -> Result<CheckResult, String> {
    let mut parser = Parser::new(code);
    match parser.parse() {
        Ok(_) => Ok(CheckResult {
            valid: true,
            errors: vec![],
        }),
        Err(e) => Ok(CheckResult {
            valid: false,
            errors: vec![e],
        }),
    }
}

/// Auto-fix common syntax errors in Mermaid source code.
///
/// Returns the fixed code and a list of fix descriptions.
///
/// # Example
///
/// ```
/// let (fixed, fixes) = mermaid_cli::fix("grpah TD; A-->B");
/// assert!(fixed.contains("graph"));
/// ```
pub fn fix(code: &str) -> (String, Vec<String>) {
    Fixer::new().fix(code)
}

/// Result of syntax checking via [`check`].
///
/// Contains the validation status and any parse errors found.
pub struct CheckResult {
    /// `true` if the Mermaid source code has valid syntax.
    pub valid: bool,
    /// List of parse errors found during syntax checking.
    /// Empty when `valid` is `true`.
    pub errors: Vec<parser::ParseError>,
}

impl CheckResult {
    /// Returns `true` if there are any parse errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

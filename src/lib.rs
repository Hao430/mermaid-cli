pub mod fixer;
pub mod parser;
pub mod renderer;
pub mod svg;

pub use fixer::Fixer;
pub use parser::Parser;
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

/// Result of syntax checking via [`check`].
///
/// Contains the validation status and any parse errors found.
pub struct CheckResult {
    pub valid: bool,
    pub errors: Vec<parser::ParseError>,
}

impl CheckResult {
    /// Returns `true` if there are any parse errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

pub mod fixer;
pub mod pdf;
pub mod parser;
pub mod renderer;
pub mod svg;

#[cfg(feature = "json")]
pub mod config;

/// Automatically fixes common syntax errors in Mermaid diagrams.
///
/// Detects and corrects typos (`grpah` → `graph`), arrow syntax issues
/// (`-->>` → `-->`), and missing `end` statements.
pub use fixer::Fixer;

/// Shape of a node in a flowchart diagram.
///
/// Variants: `Rect`, `Circle`, `Diamond`, `Rounded`, `Subroutine`,
/// `Cylinder`, `DoubleCircle`, `Flag`.
pub use parser::DiagramType;
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

/// Render a Mermaid diagram from source code to PDF.
///
/// Parses the Mermaid code, renders it to SVG, then wraps the SVG
/// in a minimal PDF 1.4 container. Returns PDF bytes.
///
/// # Example
///
/// ```
/// let result = mermaid_cli::render_pdf("graph TD; A-->B", 800, 600);
/// assert!(result.is_ok());
/// let pdf = result.unwrap();
/// assert!(pdf.starts_with(b"%PDF"));
/// ```
pub fn render_pdf(code: &str, width: u32, height: u32) -> Result<Vec<u8>, String> {
    let mut parser = Parser::new(code);
    let diagram = parser.parse().map_err(|e| e.to_string())?;
    let renderer = Renderer::with_dimensions(width, height);
    let svg = renderer.render(&diagram)?;
    let pdf = pdf::PdfWriter::render_svg(&svg, width, height);
    Ok(pdf.into_bytes())
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

/// Serialize a parsed Diagram to JSON (requires `json` feature).
///
/// Returns a JSON string representation of the AST on success.
///
/// # Example
///
/// ```ignore
/// let diagram = mermaid_cli::parse("graph TD; A-->B").unwrap();
/// let json = mermaid_cli::to_json(&diagram).unwrap();
/// assert!(json.contains("Flowchart"));
/// ```
#[cfg(feature = "json")]
pub fn to_json(diagram: &parser::Diagram) -> Result<String, String> {
    serde_json::to_string_pretty(diagram).map_err(|e| e.to_string())
}

/// Check Mermaid source code and return machine-readable JSON errors (requires `json` feature).
///
/// Returns a JSON string with structured error information including line numbers
/// and column positions, compatible with LSP error formats.
///
/// # Example
///
/// ```ignore
/// let json = mermaid_cli::check_json("invalid syntax").unwrap();
/// assert!(json.contains("line"));
/// ```
#[cfg(feature = "json")]
pub fn check_json(code: &str) -> Result<String, String> {
    let mut parser = Parser::new(code);
    match parser.parse() {
        Ok(_) => Ok(r#"{"valid":true,"errors":[]}"#.to_string()),
        Err(e) => {
            let error_json = serde_json::to_string_pretty(&e)
                .map_err(|e| e.to_string())?;
            Ok(format!(r#"{{"valid":false,"errors":[{}]}}"#, error_json))
        }
    }
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

/// Render a Mermaid diagram to PNG bytes (requires `png` feature).
///
/// Parses the code, renders it to SVG, then converts to PNG using resvg.
/// Returns PNG bytes on success.
#[cfg(feature = "png")]
pub fn render_png(code: &str, width: u32, height: u32, scale: f32) -> Result<Vec<u8>, String> {
    let mut parser = Parser::new(code);
    let diagram = parser.parse().map_err(|e| e.to_string())?;
    let renderer = Renderer::with_dimensions(width, height).with_scale(scale);
    renderer.render_png(&diagram)
}

/// Extract mermaid code blocks from a markdown string.
///
/// Returns a list of mermaid code block contents found between
/// ` ```mermaid ` and ` ``` ` fences.
///
/// # Example
///
/// ```
/// let md = "# Title\n\n```mermaid\ngraph TD\nA-->B\n```\n\nText";
/// let blocks = mermaid_cli::extract_mermaid_blocks(md);
/// assert_eq!(blocks.len(), 1);
/// assert!(blocks[0].contains("graph TD"));
/// ```
pub fn extract_mermaid_blocks(markdown: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut in_block = false;
    let mut current_block = String::new();

    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```mermaid") {
            in_block = true;
            current_block.clear();
            continue;
        }
        if in_block && trimmed == "```" {
            in_block = false;
            if !current_block.is_empty() {
                blocks.push(current_block.clone());
            }
            continue;
        }
        if in_block {
            if !current_block.is_empty() {
                current_block.push('\n');
            }
            current_block.push_str(line);
        }
    }

    blocks
}

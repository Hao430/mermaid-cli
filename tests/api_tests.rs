use mermaid_cli::*;

#[test]
fn test_render_produces_valid_svg() {
    let result = render("graph TD; A-->B");
    assert!(result.is_ok(), "render() failed: {:?}", result.err());

    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "SVG should contain <svg tag");
    assert!(svg.contains("</svg>"), "SVG should contain closing </svg> tag");
    assert!(svg.contains("A"), "SVG should contain node A");
    assert!(svg.contains("B"), "SVG should contain node B");
}

#[test]
fn test_render_multiple_nodes_and_edges() {
    let code = "graph TD; A-->B; B-->C";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());

    let svg = result.unwrap();
    assert!(svg.contains("A"));
    assert!(svg.contains("B"));
    assert!(svg.contains("C"));
}

#[test]
fn test_parse_returns_correct_ast() {
    let result = parse("graph TD; A-->B");
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());

    let diagram = result.unwrap();
    assert!(!diagram.statements.is_empty(), "AST should have statements");
}

#[test]
fn test_check_detects_valid_syntax() {
    let result = check("graph TD; A-->B").expect("check() should not fail");
    assert!(result.valid, "Valid code should be reported as valid");
    assert!(result.errors.is_empty(), "Valid code should have no errors");
}

#[test]
fn test_check_reports_syntax_errors() {
    let result = check("grpah TD; A-->B").expect("check() should not fail");
    assert!(!result.valid, "Invalid code should be reported as invalid");
    assert!(!result.errors.is_empty(), "Invalid code should have errors");
}

#[test]
fn test_check_has_errors_method() {
    // Valid code
    let valid_result = check("graph TD; A-->B").unwrap();
    assert!(!valid_result.has_errors(), "Valid code should have no errors");

    // Invalid code
    let invalid_result = check("grpah TD; A-->B").unwrap();
    assert!(invalid_result.has_errors(), "Invalid code should have errors");
}

#[test]
fn test_empty_input_is_invalid() {
    let result = render("");
    assert!(result.is_err(), "Empty input should produce an error");

    let parse_result = parse("");
    assert!(parse_result.is_err(), "Empty input parse should fail");

    let check_result = check("").unwrap();
    assert!(!check_result.valid, "Empty input should be invalid");
}

#[test]
fn test_single_node_flowchart() {
    let result = render("graph TD; A");
    assert!(result.is_ok(), "render() failed: {:?}", result.err());

    let svg = result.unwrap();
    assert!(svg.contains("A"));
}


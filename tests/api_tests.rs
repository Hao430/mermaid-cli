use mermaid_cli::*;

#[test]
fn test_render_produces_valid_svg() {
    let result = render("graph TD; A-->B");
    assert!(result.is_ok(), "render() failed: {:?}", result.err());

    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "SVG should contain <svg tag");
    assert!(
        svg.contains("</svg>"),
        "SVG should contain closing </svg> tag"
    );
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
    assert!(
        !valid_result.has_errors(),
        "Valid code should have no errors"
    );

    // Invalid code
    let invalid_result = check("grpah TD; A-->B").unwrap();
    assert!(
        invalid_result.has_errors(),
        "Invalid code should have errors"
    );
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

#[test]
fn test_render_node_with_label() {
    let code = "graph TD; A[Start]-->B[End]";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());

    let svg = result.unwrap();
    assert!(svg.contains("Start"), "SVG should contain label 'Start'");
    assert!(svg.contains("End"), "SVG should contain label 'End'");
}

#[test]
fn test_render_mixed_labels() {
    let code = "graph TD; A[Start]-->B";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());

    let svg = result.unwrap();
    assert!(svg.contains("Start"), "SVG should contain label 'Start'");
    assert!(
        svg.contains("B"),
        "SVG should contain node ID 'B' (no label)"
    );
}

#[test]
fn test_parse_edge_with_labels() {
    let result = parse("graph TD; A[Start]-->B[End]");
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());

    let diagram = result.unwrap();
    // Should have: NodeDef(A, Start) + NodeDef(B, End) + EdgeDef(A->B)
    assert_eq!(diagram.statements.len(), 3, "Should have 3 statements");
}

// --- 节点形状集成测试 ---

#[test]
fn test_render_rounded_node() {
    let result = render("graph TD; A(Rounded)-->B");
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("rx:10"), "SVG should contain rounded corners");
    assert!(svg.contains("Rounded"));
}

#[test]
fn test_render_diamond_node() {
    let result = render("graph TD; A{Decision}-->B");
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("path"), "Diamond should use path element");
    assert!(svg.contains("Decision"));
}

#[test]
fn test_render_cylinder_node() {
    let result = render("graph TD; A[(Database)]-->B");
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(
        svg.contains("ellipse"),
        "Cylinder should use ellipse element"
    );
    assert!(svg.contains("Database"));
}

#[test]
fn test_render_double_circle_node() {
    let result = render("graph TD; A((Start))-->B");
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    let circle_count = svg.matches("circle").count();
    assert!(
        circle_count >= 2,
        "DoubleCircle should have 2 circles, found {}",
        circle_count
    );
    assert!(svg.contains("Start"));
}

#[test]
fn test_render_subroutine_node() {
    let result = render("graph TD; A[[Process]]-->B");
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    let rect_count = svg.matches("rect").count();
    assert!(
        rect_count >= 2,
        "Subroutine should have 2 rects, found {}",
        rect_count
    );
    assert!(svg.contains("Process"));
}

#[test]
fn test_render_flag_node() {
    let result = render("graph TD; A>Flag]-->B");
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("path"), "Flag should use path element");
    assert!(svg.contains("Flag"));
}

#[test]
fn test_render_mixed_shapes() {
    let code = "graph TD; A[Rect]-->B(Rounded); B-->C{Diamond}; C-->D((Circle))";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("Rect"));
    assert!(svg.contains("Rounded"));
    assert!(svg.contains("Diamond"));
    assert!(svg.contains("Circle"));
}

#[test]
fn test_parse_shape_with_label() {
    let result = parse("graph TD; A{Decision}");
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());
    let diagram = result.unwrap();
    match &diagram.statements[0] {
        mermaid_cli::Statement::NodeDef { id, label, shape } => {
            assert_eq!(id, "A");
            assert_eq!(label.as_deref(), Some("Decision"));
            assert_eq!(*shape, mermaid_cli::NodeShape::Diamond);
        }
        _ => panic!("Expected NodeDef"),
    }
}

#[test]
fn test_check_error_has_location() {
    let result = mermaid_cli::check("invalid_syntax").unwrap();
    assert!(
        !result.valid,
        "invalid_syntax should be reported as invalid"
    );
    assert!(!result.errors.is_empty(), "Should have at least one error");

    let error = &result.errors[0];
    // Display should include line number and column number (numeric)
    let display = format!("{}", error);
    assert!(
        display.contains("line"),
        "Error display should include line number"
    );
    assert!(
        display.contains(':'),
        "Error display should include colon-separated location"
    );
    assert!(!display.is_empty(), "Error message should not be empty");
}

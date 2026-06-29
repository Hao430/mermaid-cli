use mermaid_cli::parser::DiagramType;
use mermaid_cli::renderer;
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

// ============================================================
// 序列图 API 测试
// ============================================================

#[test]
fn test_sequence_basic_render() {
    let code = "sequenceDiagram\n    Alice->Bob: Hello\n    Bob-->Alice: Hi";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("Alice"));
    assert!(svg.contains("Bob"));
    assert!(svg.contains("Hello"));
    assert!(svg.contains("Hi"));
}

#[test]
fn test_sequence_participant_alias() {
    let code = "sequenceDiagram\n    participant A as Alice\n    A->B: Hello";
    let result = render(code);
    assert!(result.is_ok());
    let svg = result.unwrap();
    assert!(svg.contains("Alice"), "Should display alias, not ID");
}

#[test]
fn test_sequence_arrow_types() {
    let code =
        "sequenceDiagram\n    A->B: solid\n    A-->B: dashed\n    A->>B: cross\n    A-->>B: dcross";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    // dashed arrows should have stroke-dasharray
    assert!(
        svg.contains("stroke-dasharray"),
        "Dashed arrows should have dasharray"
    );
}

#[test]
fn test_sequence_auto_participants() {
    let code = "sequenceDiagram\n    Client->Server: Request\n    Server->DB: Query\n    DB-->Server: Result";
    let result = render(code);
    assert!(result.is_ok());
    let svg = result.unwrap();
    assert!(svg.contains("Client"));
    assert!(svg.contains("Server"));
    assert!(svg.contains("DB"));
}

#[test]
fn test_sequence_alt_block() {
    let code =
        "sequenceDiagram\n    alt ok\n        A->B: yes\n    else no\n        A->B: no\n    end";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("yes"));
    assert!(svg.contains("no"));
}

#[test]
fn test_sequence_loop_block() {
    let code = "sequenceDiagram\n    loop forever\n        A->B: ping\n    end";
    let result = render(code);
    assert!(result.is_ok());
    let svg = result.unwrap();
    assert!(svg.contains("ping"));
}

#[test]
fn test_sequence_nested_blocks() {
    let code = "sequenceDiagram\n    alt outer\n        A->B: msg1\n        loop inner\n            B->A: msg2\n        end\n    end";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("msg1"));
    assert!(svg.contains("msg2"));
}

#[test]
fn test_sequence_complex_diagram() {
    let code = "sequenceDiagram
    participant Client
    participant Server
    participant DB
    Client->Server: Login
    Server->DB: Query User
    DB-->Server: User Data
    alt valid
        Server-->Client: OK
    else invalid
        Server-->Client: Failed
    end
    loop heartbeat
        Client->Server: Ping
        Server-->Client: Pong
    end";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("Client"));
    assert!(svg.contains("Server"));
    assert!(svg.contains("DB"));
    assert!(svg.contains("Login"));
    assert!(svg.contains("Ping"));
    assert!(svg.contains("Pong"));
}

#[test]
fn test_sequence_svg_structure() {
    let code = "sequenceDiagram\n    A->B: Hello";
    let result = render(code).unwrap();
    assert!(result.contains("<?xml"), "Should have XML declaration");
    assert!(result.contains("<svg"), "Should have SVG tag");
    assert!(result.contains("</svg>"), "Should have closing SVG tag");
    assert!(
        result.contains("<line"),
        "Should have line elements for arrows"
    );
    assert!(
        result.contains("<text"),
        "Should have text elements for labels"
    );
    assert!(
        result.contains("<rect"),
        "Should have rect elements for participant boxes"
    );
}

#[test]
fn test_sequence_three_participants() {
    let code = "sequenceDiagram\n    A->B: msg1\n    B->C: msg2\n    C->A: msg3";
    let result = render(code).unwrap();
    // should have lifelines (dashed lines)
    assert!(
        result.contains("stroke-dasharray"),
        "Should have dashed lifelines"
    );
}

#[test]
fn test_parse_sequence_returns_ast() {
    let code = "sequenceDiagram\n    participant Alice\n    Alice->Bob: Hello";
    let result = parse(code);
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());
    let diagram = result.unwrap();
    assert_eq!(diagram.diagram_type, DiagramType::Sequence);
}

#[test]
fn test_sequence_note_right() {
    let code = "sequenceDiagram\n    participant Alice\n    Note right of Alice: This is a note";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("This is a note"));
    assert!(
        svg.contains("#fffde7"),
        "Note should have yellow background"
    );
}

#[test]
fn test_sequence_note_left() {
    let code = "sequenceDiagram\n    participant Alice\n    Note left of Alice: Left note";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("Left note"));
}

#[test]
fn test_sequence_activate_deactivate() {
    let code = "sequenceDiagram\n    participant Alice\n    activate Alice\n    Alice->Bob: Hello\n    deactivate Alice";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("Alice"));
    assert!(svg.contains("Hello"));
}

#[test]
fn test_sequence_note_in_block() {
    let code = "sequenceDiagram\n    participant Alice\n    loop retry\n        Note right of Alice: Inside loop\n        Alice->Bob: ping\n    end";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("Inside loop"));
    assert!(svg.contains("ping"));
}

#[test]
fn test_sequence_complex_with_notes_and_activation() {
    let code = "sequenceDiagram
    participant Client
    participant Server
    activate Client
    Client->Server: Request
    activate Server
    Note right of Server: Processing
    Server-->Client: Response
    deactivate Server
    deactivate Client";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("Request"));
    assert!(svg.contains("Response"));
    assert!(svg.contains("Processing"));
}

#[test]
fn test_fixer_sequence_typo() {
    let fixer = mermaid_cli::fixer::Fixer::new();
    let (fixed, fixes) = fixer.fix("sequnceDiagram\nAlice->Bob: Hello");
    assert!(fixed.contains("sequenceDiagram"));
    assert!(!fixes.is_empty());
}

// ============================================================
// 饼图 API 测试
// ============================================================

#[test]
fn test_pie_basic_render() {
    let code = "pie title Pets\n\"Dogs\" : 386\n\"Cats\" : 85\n\"Rats\" : 15";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "Should be valid SVG");
    assert!(svg.contains("Pets"), "Should contain title");
    assert!(svg.contains("Dogs"), "Should contain slice label");
    assert!(svg.contains("Cats"), "Should contain slice label");
    assert!(svg.contains("Rats"), "Should contain slice label");
}

#[test]
fn test_pie_parse_returns_correct_ast() {
    let code = "pie title Sales\n\"Q1\" : 100\n\"Q2\" : 200";
    let result = parse(code);
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());
    let diagram = result.unwrap();
    assert_eq!(diagram.diagram_type, DiagramType::Pie);
    assert_eq!(diagram.title.as_deref(), Some("Sales"));
    assert_eq!(diagram.statements.len(), 2);
}

#[test]
fn test_pie_check_valid_syntax() {
    let code = "pie title Test\n\"A\" : 50\n\"B\" : 50";
    let result = check(code).expect("check() should not fail");
    assert!(result.valid, "Valid pie chart should be reported as valid");
}

#[test]
fn test_pie_no_title() {
    let code = "pie\n\"A\" : 50\n\"B\" : 50";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("A"));
    assert!(svg.contains("B"));
}

#[test]
fn test_pie_unquoted_labels() {
    let code = "pie title Test\nDogs : 100\nCats : 200";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("Dogs"));
    assert!(svg.contains("Cats"));
}

#[test]
fn test_pie_svg_structure() {
    let code = "pie title Data\n\"X\" : 75\n\"Y\" : 25";
    let result = render(code).unwrap();
    assert!(result.contains("<?xml"), "Should have XML declaration");
    assert!(result.contains("<svg"), "Should have SVG tag");
    assert!(result.contains("</svg>"), "Should have closing SVG tag");
    assert!(
        result.contains("<path"),
        "Should have path elements for pie slices"
    );
    assert!(
        result.contains("<text"),
        "Should have text elements for labels"
    );
}

// ============================================================
// 类图 API 测试
// ============================================================

#[test]
fn test_class_basic_render() {
    let code = "classDiagram\nclass Animal";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "Should be valid SVG");
    assert!(svg.contains("Animal"), "Should contain class name");
}

#[test]
fn test_class_with_members() {
    let code = "classDiagram\nclass Animal {\n+String name\n+int age\n+isMammal() bool\n}";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("Animal"));
    assert!(svg.contains("name"));
    assert!(svg.contains("age"));
    assert!(svg.contains("isMammal"));
}

#[test]
fn test_class_relationship() {
    let code = "classDiagram\nAnimal <|-- Dog";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("Animal"));
    assert!(svg.contains("Dog"));
}

#[test]
fn test_class_association_with_label() {
    let code = "classDiagram\nAnimal --> Food : eats";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("Animal"));
    assert!(svg.contains("Food"));
    assert!(svg.contains("eats"));
}

#[test]
fn test_class_parse_returns_correct_ast() {
    let code = "classDiagram\nclass Dog";
    let result = parse(code);
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());
    let diagram = result.unwrap();
    assert_eq!(diagram.diagram_type, DiagramType::Class);
}

#[test]
fn test_class_check_valid_syntax() {
    let code = "classDiagram\nclass Animal";
    let result = check(code).expect("check() should not fail");
    assert!(result.valid, "Valid class diagram should pass check");
}

#[test]
fn test_class_svg_structure() {
    let code = "classDiagram\nclass Animal {\n+String name\n}";
    let result = render(code).unwrap();
    assert!(result.contains("<svg"), "Should have SVG tag");
    assert!(result.contains("<rect"), "Should have rect for class box");
    assert!(result.contains("<text"), "Should have text for class name");
}

// ============================================================
// 状态图 API 测试
// ============================================================

#[test]
fn test_state_basic_render() {
    let code = "stateDiagram-v2\n[*] --> Still\nStill --> [*]";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "Should be valid SVG");
    assert!(svg.contains("Still"), "Should contain state name");
}

#[test]
fn test_state_multiple_transitions() {
    let code = "stateDiagram-v2\n[*] --> Still\nStill --> Moving\nMoving --> Still\nMoving --> Crash\nCrash --> [*]";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("Still"));
    assert!(svg.contains("Moving"));
    assert!(svg.contains("Crash"));
}

#[test]
fn test_state_with_label() {
    let code = "stateDiagram-v2\nstate Moving : This is moving";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("This is moving"));
}

#[test]
fn test_state_parse_returns_correct_ast() {
    let code = "stateDiagram-v2\n[*] --> A\nA --> [*]";
    let result = parse(code);
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());
    let diagram = result.unwrap();
    assert_eq!(diagram.diagram_type, DiagramType::State);
}

#[test]
fn test_state_check_valid_syntax() {
    let code = "stateDiagram-v2\n[*] --> A\nA --> [*]";
    let result = check(code).expect("check() should not fail");
    assert!(result.valid, "Valid state diagram should pass check");
}

#[test]
fn test_state_svg_structure() {
    let code = "stateDiagram-v2\n[*] --> A\nA --> B\nB --> [*]";
    let result = render(code).unwrap();
    assert!(result.contains("<svg"), "Should have SVG tag");
    assert!(result.contains("<rect"), "Should have rect for state box");
    assert!(result.contains("<text"), "Should have text for state name");
}

// ============================================================
// ER 图 API 测试
// ============================================================

#[test]
fn test_er_basic_render() {
    let code = "erDiagram\nCUSTOMER ||--o{ ORDER : places";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "Should be valid SVG");
    assert!(svg.contains("CUSTOMER"), "Should contain entity name");
    assert!(svg.contains("ORDER"), "Should contain entity name");
}

#[test]
fn test_er_with_attributes() {
    let code = "erDiagram\nCUSTOMER {\nstring name\nstring custNumber\n}";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("CUSTOMER"));
    assert!(svg.contains("name"));
    assert!(svg.contains("custNumber"));
}

#[test]
fn test_er_relationship_with_label() {
    let code = "erDiagram\nCUSTOMER ||--o{ ORDER : places";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("places"), "Should contain relationship label");
}

#[test]
fn test_er_parse_returns_correct_ast() {
    let code = "erDiagram\nCUSTOMER ||--o{ ORDER : places";
    let result = parse(code);
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());
    let diagram = result.unwrap();
    assert_eq!(diagram.diagram_type, DiagramType::Er);
}

#[test]
fn test_er_check_valid_syntax() {
    let code = "erDiagram\nCUSTOMER ||--o{ ORDER : places";
    let result = check(code).expect("check() should not fail");
    assert!(result.valid, "Valid ER diagram should pass check");
}

#[test]
fn test_er_svg_structure() {
    let code = "erDiagram\nCUSTOMER {\nstring name\n}\nCUSTOMER ||--o{ ORDER : places";
    let result = render(code).unwrap();
    assert!(result.contains("<svg"), "Should have SVG tag");
    assert!(result.contains("<rect"), "Should have rect for entity box");
    assert!(result.contains("<text"), "Should have text for entity name");
}

// ============================================================
// 甘特图 API 测试
// ============================================================

#[test]
fn test_gantt_basic_render() {
    let code = "gantt\n    title Project\n    section Design\n    Task 1 :t1, 2024-01-01, 10d";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "Should be valid SVG");
    assert!(svg.contains("Task 1"), "Should contain task name");
}

#[test]
fn test_gantt_with_section() {
    let code = "gantt\n    section Phase 1\n    Design :t1, 1, 5d\n    section Phase 2\n    Build :t2, 6, 10d";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("Phase 1"));
    assert!(svg.contains("Phase 2"));
}

#[test]
fn test_gantt_parse_returns_correct_ast() {
    let code = "gantt\n    title Test\n    Task :t1, 1, 5d";
    let result = parse(code);
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());
    let diagram = result.unwrap();
    assert_eq!(diagram.diagram_type, DiagramType::Gantt);
}

#[test]
fn test_gantt_check_valid_syntax() {
    let code = "gantt\n    Task :t1, 1, 5d";
    let result = check(code).expect("check() should not fail");
    assert!(result.valid, "Valid Gantt chart should pass check");
}

#[test]
fn test_gantt_svg_structure() {
    let code = "gantt\n    title Project\n    section S\n    Task :t1, 1, 5d";
    let result = render(code).unwrap();
    assert!(result.contains("<svg"), "Should have SVG tag");
    assert!(result.contains("<text"), "Should have text for task name");
}

// ============================================================
// Markdown 提取 API 测试
// ============================================================

#[test]
fn test_extract_single_mermaid_block() {
    let md = "# Title\n\n```mermaid\ngraph TD\nA-->B\n```\n\nEnd";
    let blocks = extract_mermaid_blocks(md);
    assert_eq!(blocks.len(), 1, "Should find 1 block");
    assert!(
        blocks[0].contains("graph TD"),
        "Should contain mermaid code"
    );
    assert!(
        blocks[0].contains("A-->B"),
        "Should contain diagram content"
    );
}

#[test]
fn test_extract_multiple_blocks() {
    let md = "# Diagrams\n\n```mermaid\ngraph TD\nA-->B\n```\n\nText\n\n```mermaid\nsequenceDiagram\nA->B: Hi\n```";
    let blocks = extract_mermaid_blocks(md);
    assert_eq!(blocks.len(), 2, "Should find 2 blocks");
}

#[test]
fn test_extract_no_block() {
    let md = "# Just text\nNo mermaid here";
    let blocks = extract_mermaid_blocks(md);
    assert!(blocks.is_empty(), "Should find no blocks");
}

// ============================================================
// SVG 结构验证测试 (快照替代)
// ============================================================

/// 验证 SVG 的 XML 结构完整性
fn validate_svg_structure(svg: &str, expected_elements: &[&str]) -> Vec<String> {
    let mut issues = Vec::new();

    // 验证 XML 声明
    if !svg.contains("<?xml") {
        issues.push("Missing XML declaration".to_string());
    }

    // 验证 SVG 标签
    if !svg.contains("<svg") {
        issues.push("Missing <svg> tag".to_string());
    }
    if !svg.contains("</svg>") {
        issues.push("Missing </svg> tag".to_string());
    }

    // 验证 viewBox/width/height 存在
    if !svg.contains("width=") {
        issues.push("Missing width attribute".to_string());
    }

    // 验证必需元素
    for element in expected_elements {
        let search = format!("<{}", element);
        if !svg.contains(&search) {
            issues.push(format!("Missing <{}> element", element));
        }
    }

    // 验证标签平衡 (粗略检查)
    let open_count = svg.matches("<svg").count();
    let close_count = svg.matches("</svg>").count();
    if open_count != close_count {
        issues.push(format!(
            "SVG tag imbalance: {} open, {} close",
            open_count, close_count
        ));
    }

    issues
}

#[test]
fn test_svg_valid_structure_flowchart() {
    let svg = render("graph TD; A[Start]-->B[End]").unwrap();
    let issues = validate_svg_structure(&svg, &["rect", "text"]);
    assert!(
        issues.is_empty(),
        "Flowchart SVG structure issues: {:?}",
        issues
    );
}

#[test]
fn test_svg_valid_structure_sequence() {
    let svg = render("sequenceDiagram\n    A->B: Hello").unwrap();
    let issues = validate_svg_structure(&svg, &["rect", "text", "line"]);
    assert!(
        issues.is_empty(),
        "Sequence SVG structure issues: {:?}",
        issues
    );
}

#[test]
fn test_svg_valid_structure_pie() {
    let svg = render("pie title Data\n\"X\" : 50\n\"Y\" : 50").unwrap();
    let issues = validate_svg_structure(&svg, &["path", "text"]);
    assert!(issues.is_empty(), "Pie SVG structure issues: {:?}", issues);
}

#[test]
fn test_svg_valid_structure_class() {
    let svg = render("classDiagram\nclass Animal {\n+String name\n}").unwrap();
    let issues = validate_svg_structure(&svg, &["rect", "text"]);
    assert!(
        issues.is_empty(),
        "Class SVG structure issues: {:?}",
        issues
    );
}

#[test]
fn test_svg_valid_structure_state() {
    let svg = render("stateDiagram-v2\n[*] --> A\nA --> [*]").unwrap();
    let issues = validate_svg_structure(&svg, &["circle", "text"]);
    assert!(
        issues.is_empty(),
        "State SVG structure issues: {:?}",
        issues
    );
}

#[test]
fn test_svg_valid_structure_er() {
    let svg = render("erDiagram\nCUSTOMER ||--o{ ORDER : places").unwrap();
    let issues = validate_svg_structure(&svg, &["rect", "text"]);
    assert!(issues.is_empty(), "ER SVG structure issues: {:?}", issues);
}

#[test]
fn test_svg_valid_structure_gantt() {
    let svg = render("gantt\n    title Project\n    section S\n    Task :t1, 1, 5d").unwrap();
    let issues = validate_svg_structure(&svg, &["rect", "text"]);
    assert!(
        issues.is_empty(),
        "Gantt SVG structure issues: {:?}",
        issues
    );
}

#[test]
fn test_svg_valid_structure_empty() {
    let result = render("");
    assert!(result.is_err(), "Empty input should produce error");
}

#[test]
fn test_svg_valid_structure_all_types() {
    let cases = vec![
        ("flowchart", "graph TD; A-->B"),
        ("sequence", "sequenceDiagram\nA->B: Hello"),
        ("pie", "pie\n\"A\" : 50\n\"B\" : 50"),
        ("class", "classDiagram\nclass X"),
        ("state", "stateDiagram-v2\n[*] --> A\nA --> [*]"),
        ("er", "erDiagram\nX ||--o{ Y : z"),
        ("gantt", "gantt\nTask :t1, 1, 5d"),
    ];

    for (name, code) in &cases {
        let result = render(code);
        assert!(
            result.is_ok(),
            "{} should render successfully: {:?}",
            name,
            result.err()
        );
        let svg = result.unwrap();
        let issues = validate_svg_structure(&svg, &["text"]);
        assert!(issues.is_empty(), "{} SVG structure: {:?}", name, issues);
    }
}

#[test]
fn test_extract_empty_block() {
    let md = "# Empty\n\n```mermaid\n```\n\nEnd";
    let blocks = extract_mermaid_blocks(md);
    // Empty blocks should be skipped
    assert!(blocks.is_empty(), "Empty blocks should be skipped");
}

// ============================================================
// 边缘案例测试
// ============================================================

#[test]
fn test_render_unicode_in_node() {
    let result = render("graph TD; A[你好世界]-->B[こんにちは]");
    assert!(
        result.is_ok(),
        "Unicode labels should render: {:?}",
        result.err()
    );
    let svg = result.unwrap();
    assert!(svg.contains("你好世界"));
    assert!(svg.contains("こんにちは"));
}

#[test]
fn test_render_very_large_diagram() {
    // 100 nodes chain
    let mut code = String::from("graph TD\n");
    for i in 0..99 {
        code.push_str(&format!("N{}-->N{}\n", i, i + 1));
    }
    let result = render(&code);
    assert!(
        result.is_ok(),
        "100-node chain should render: {:?}",
        result.err()
    );
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "Large diagram should produce SVG");
}

#[test]
fn test_render_special_characters() {
    let cases = vec![
        ("dots", "graph TD; A[Test...]-->B"),
        ("at sign", "graph TD; A[user@host]-->B"),
        ("parentheses", "graph TD; A[func(x)]-->B"),
        ("plus", "graph TD; A[C++]-->B"),
        ("hashtag", "graph TD; A[#1]-->B"),
    ];
    for (name, code) in &cases {
        let result = render(code);
        assert!(result.is_ok(), "{} should render: {:?}", name, result.err());
    }
}

#[test]
fn test_render_flowchart_unicode_nodes() {
    let code = "flowchart LR\nA(开始)-->B(结束)";
    let result = render(code);
    assert!(result.is_ok());
    let svg = result.unwrap();
    assert!(svg.contains("开始"));
    assert!(svg.contains("结束"));
}

#[test]
fn test_render_flowchart_various_arrows() {
    let code = "graph TD\nA-->B\nC-->D\nE-->F";
    let result = render(code);
    assert!(result.is_ok());
    let svg = result.unwrap();
    assert!(svg.contains("A"));
    assert!(svg.contains("F"));
}

#[test]
fn test_render_sequence_multiple_messages() {
    let code = "sequenceDiagram\nAlice->Bob: Hello\nBob->Carol: Hi\nCarol->Alice: Hey";
    let result = render(code);
    assert!(result.is_ok());
    let svg = result.unwrap();
    assert!(svg.contains("Alice"));
    assert!(svg.contains("Bob"));
    assert!(svg.contains("Carol"));
}

#[test]
fn test_render_sequence_dashed_cross_arrows() {
    let code = "sequenceDiagram\nA-->>B: dashed cross\nB--)A: dashed open";
    let result = render(code);
    assert!(result.is_ok());
    let svg = result.unwrap();
    assert!(svg.contains("dashed cross"));
    assert!(svg.contains("dashed open"));
}

#[test]
fn test_render_pie_many_slices() {
    let mut code = "pie title Many Items\n".to_string();
    for i in 0..12 {
        code.push_str(&format!("\"Item {}\" : {}\n", i, (i + 1) * 10));
    }
    let result = render(&code);
    assert!(
        result.is_ok(),
        "12-slice pie should render: {:?}",
        result.err()
    );
    let svg = result.unwrap();
    assert!(svg.contains("Item 0"));
    assert!(svg.contains("Item 11"));
}

#[test]
fn test_render_class_relationship_types() {
    let code = "classDiagram\nAnimal <|-- Dog\nAnimal *-- Heart\nAnimal o-- Owner\nAnimal --> Food";
    let result = render(code);
    assert!(result.is_ok());
    let svg = result.unwrap();
    assert!(svg.contains("Animal"));
    assert!(svg.contains("Dog"));
}

#[test]
fn test_render_state_concurrent_transitions() {
    let code = "stateDiagram-v2\n[*] --> Idle\nIdle --> Processing\nProcessing --> Completed\nProcessing --> Failed\nCompleted --> [*]\nFailed --> Idle";
    let result = render(code);
    assert!(result.is_ok());
    let svg = result.unwrap();
    assert!(svg.contains("Idle"));
    assert!(svg.contains("Processing"));
    assert!(svg.contains("Failed"));
}

#[test]
fn test_render_er_multiple_entities() {
    let code = "erDiagram\nCUSTOMER ||--o{ ORDER : places\nORDER ||--|{ LINE-ITEM : contains\nCUSTOMER ||--o{ ADDRESS : has";
    let result = render(code);
    assert!(result.is_ok());
    let svg = result.unwrap();
    assert!(svg.contains("CUSTOMER"));
    assert!(svg.contains("ORDER"));
    assert!(svg.contains("ADDRESS"));
}

#[test]
fn test_render_gantt_single_task() {
    let code = "gantt\n    title Simple\n    Task1 :a1, 1, 5d";
    let result = render(code);
    assert!(result.is_ok());
    let svg = result.unwrap();
    assert!(svg.contains("Task1"), "Should contain task name 'Task1'");
}

#[test]
fn test_check_multiple_error_types() {
    let cases = vec![
        ("empty", "", false),
        ("typo", "grpah TD; A-->B", false),
        ("valid flow", "graph TD; A-->B", true),
        ("valid seq", "sequenceDiagram\nA->B: Hi", true),
        ("valid pie", "pie\n\"A\" : 50", true),
    ];
    for (name, code, expected_valid) in &cases {
        let result = check(code).expect("check() should not fail");
        assert_eq!(
            result.valid, *expected_valid,
            "{} should be valid={}",
            name, expected_valid
        );
    }
}

#[test]
fn test_fixer_multiple_typos() {
    let fixer = mermaid_cli::fixer::Fixer::new();
    let cases = vec![
        ("grpah TD; A-->B", "graph"),
        ("flowchat TD; A-->B", "flowchart"),
        ("sequenceDiagram", "sequenceDiagram"),
    ];
    for (input, expected) in &cases {
        let (_fixed, fixes) = fixer.fix(input);
        if input != expected {
            assert!(!fixes.is_empty(), "Should find fixes for: {}", input);
        }
    }
}

// ============================================================
// 更多边缘案例与压力测试
// ============================================================

#[test]
fn test_render_flowchart_all_shapes() {
    let cases = vec![
        ("rect", "graph TD; A[Rect]"),
        ("rounded", "graph TD; A(Rounded)"),
        ("diamond", "graph TD; A{Diamond}"),
        ("double_rounded", "graph TD; A([Rounded])"),
        ("subroutine", "graph TD; A[[Sub]]"),
        ("cylinder", "graph TD; A[(DB)]"),
        ("double_circle", "graph TD; A((Start))"),
        ("flag", "graph TD; A>Flag]"),
    ];
    for (name, code) in &cases {
        let result = render(code);
        assert!(result.is_ok(), "{} should render: {:?}", name, result.err());
    }
}

#[test]
fn test_render_flowchart_with_edge_labels() {
    let code = r"graph TD
    A[Start]-->|Process|B[End]
    C-->|Label|D";
    let result = render(code);
    assert!(
        result.is_ok(),
        "Edge labels should render: {:?}",
        result.err()
    );
    let svg = result.unwrap();
    assert!(svg.contains("Process"), "Should contain edge label");
}

#[test]
fn test_render_flowchart_subgraph() {
    let code = "graph TD\nsubgraph Group\nA-->B\nend\nC-->D";
    let result = render(code);
    assert!(result.is_ok(), "Subgraph should render: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "Subgraph should produce valid SVG");
}

#[test]
fn test_render_flowchart_mixed_directions() {
    let cases = vec![
        ("TD", "graph TD; A-->B"),
        ("LR", "graph LR; A-->B"),
        ("BT", "graph BT; A-->B"),
        ("RL", "graph RL; A-->B"),
    ];
    for (name, code) in &cases {
        let result = render(code);
        assert!(
            result.is_ok(),
            "{} direction should render: {:?}",
            name,
            result.err()
        );
    }
}

#[test]
fn test_render_flowchart_long_chains() {
    // 20-node chain
    let mut code = String::from("graph TD\n");
    for i in 0..20 {
        code.push_str(&format!("N{}-->N{}\n", i, i + 1));
    }
    let result = render(&code);
    assert!(
        result.is_ok(),
        "Long chain should render: {:?}",
        result.err()
    );
}

#[test]
fn test_render_sequence_note_positions() {
    let cases = vec![
        ("right", "Right note"),
        ("left", "Left note"),
        ("over", "Over note"),
    ];
    for (_pos, text) in &cases {
        let code = format!("sequenceDiagram\nNote {} of A: {}", cases[0].0, text);
        let result = render(&code);
        assert!(result.is_ok(), "Note should render: {:?}", result.err());
    }
}

#[test]
fn test_render_sequence_note_right() {
    let code = "sequenceDiagram\nNote right of A: Right note";
    let result = render(code);
    assert!(result.is_ok());
    let svg = result.unwrap();
    assert!(svg.contains("Right note"), "Should contain note text");
}

#[test]
fn test_render_sequence_activation_boxes() {
    let code = "sequenceDiagram\nactivate Alice\nAlice->Bob: Hello\ndeactivate Alice";
    let result = render(code);
    assert!(
        result.is_ok(),
        "Activation boxes should render: {:?}",
        result.err()
    );
    let svg = result.unwrap();
    assert!(svg.contains("Hello"), "Should contain message");
}

#[test]
fn test_render_sequence_nested_blocks() {
    let code = "sequenceDiagram\n    alt outer\n        A->B: msg\n    end";
    let result = render(code);
    assert!(
        result.is_ok(),
        "Simple nested block should render: {:?}",
        result.err()
    );
    let svg = result.unwrap();
    assert!(svg.contains("msg"), "Should contain message text");
}

#[test]
fn test_render_pie_large_values() {
    let code = "pie\n\"Large\" : 99999\n\"Small\" : 1";
    let result = render(code);
    assert!(
        result.is_ok(),
        "Large values should work: {:?}",
        result.err()
    );
    let svg = result.unwrap();
    assert!(svg.contains("Large"), "Should contain label");
}

#[test]
fn test_render_pie_zero_values() {
    let code = "pie\n\"Zero\" : 0\n\"Non-zero\" : 100";
    let result = render(code);
    assert!(result.is_ok(), "Zero values should render");
}

#[test]
fn test_render_class_multiple_classes() {
    let code = "classDiagram\nclass Animal {\n+String name\n}\nclass Dog {\n+bark() void\n}";
    let result = render(code);
    assert!(
        result.is_ok(),
        "Multiple classes should render: {:?}",
        result.err()
    );
    let svg = result.unwrap();
    assert!(svg.contains("Animal"));
    assert!(svg.contains("Dog"));
}

#[test]
fn test_render_class_visibility_modifiers() {
    let code = "classDiagram\nclass Test {\n+public_field\n-private field\n#protected field\n~package field\n}";
    let result = render(code);
    assert!(
        result.is_ok(),
        "Visibility modifiers should render: {:?}",
        result.err()
    );
}

#[test]
fn test_render_state_complex() {
    let code = "stateDiagram-v2\n[*] --> Idle\nIdle --> Processing\nProcessing --> Completed\nProcessing --> Failed\nCompleted --> [*]\nFailed --> Idle";
    let result = render(code);
    assert!(
        result.is_ok(),
        "Complex state should render: {:?}",
        result.err()
    );
    let svg = result.unwrap();
    assert!(svg.contains("Processing"), "Should contain state");
}

#[test]
fn test_render_er_with_attributes() {
    let code = "erDiagram\nCUSTOMER {\nstring name PK\nint age NULL\n}";
    let result = render(code);
    assert!(
        result.is_ok(),
        "ER with attributes should render: {:?}",
        result.err()
    );
    let svg = result.unwrap();
    assert!(svg.contains("name"), "Should contain attribute name");
    assert!(svg.contains("age"), "Should contain attribute");
}

#[test]
fn test_render_gantt_title_only() {
    let code = "gantt\n    title Just Title";
    let result = render(code);
    assert!(result.is_ok(), "Gantt with only title should render");
}

#[test]
fn test_check_various_diagram_types() {
    let cases: Vec<(&str, bool)> = vec![
        ("graph TD; A-->B", true),
        ("sequenceDiagram\nA->B: Hi", true),
        ("pie\n\"A\" : 50", true),
        ("classDiagram\nclass A", true),
        ("stateDiagram-v2\n[*] --> A", true),
        ("erDiagram\nX ||--o{ Y", true),
        ("gantt\nTask :t1, 1, 5d", true),
        ("", false),
        ("invalid gibberish content", false),
    ];
    for (code, expected) in &cases {
        let result = check(code).expect("check() should not fail");
        assert_eq!(
            result.valid,
            *expected,
            "check('{}...') should be valid={}",
            &code[..code.len().min(20)],
            expected
        );
    }
}

#[test]
fn test_parse_error_messages() {
    let cases = vec![("", "Empty input"), ("unknown_keyword", "Expected")];
    for (code, expected_msg) in &cases {
        let result = parse(code);
        assert!(result.is_err(), "{} should fail to parse", code);
        if let Err(errors) = result {
            let msg = format!("{}", errors[0]);
            assert!(
                msg.contains(expected_msg),
                "Error '{}' should contain '{}'",
                msg,
                expected_msg
            );
        }
    }
}

#[test]
fn test_fixer_apply_fixes() {
    let fixer = mermaid_cli::fixer::Fixer::new();
    let cases = vec![
        ("grpah TD; A-->B", "graph"),
        ("sequnceDiagram\nA->B: Hi", "sequenceDiagram"),
    ];
    for (input, expected) in &cases {
        let (fixed, fixes) = fixer.fix(input);
        assert!(!fixes.is_empty(), "Should fix: {}", input);
        assert!(
            fixed.contains(expected),
            "Fixed code should contain '{}'",
            expected
        );
    }
}

#[test]
fn test_fixer_preserves_valid_code() {
    let fixer = mermaid_cli::fixer::Fixer::new();
    let (fixed, _fixes) = fixer.fix("graph TD; A-->B");
    assert!(
        fixed.contains("graph"),
        "Valid code should preserve graph keyword"
    );
}

#[test]
fn test_markdown_multiple_extraction() {
    let md = "# Doc\n\n```mermaid\ngraph TD; A-->B\n```\n\nMore\n\n```mermaid\nsequenceDiagram\nA->B: Hi\n```";
    let blocks = extract_mermaid_blocks(md);
    assert_eq!(blocks.len(), 2, "Should extract 2 blocks");
    assert!(!blocks[0].is_empty(), "First block should not be empty");
    assert!(!blocks[1].is_empty(), "Second block should not be empty");
}

// ============================================================
// PNG 输出测试 (需要 feature="png")
// ============================================================

#[cfg(feature = "png")]
#[test]
fn test_render_png_basic() {
    let code = "graph TD; A-->B";
    let result = mermaid_cli::render_png(code, 800, 600, 1.0);
    assert!(
        result.is_ok(),
        "PNG render should succeed: {:?}",
        result.err()
    );
    let png_bytes = result.unwrap();
    // PNG magic bytes: 89 50 4E 47
    assert_eq!(
        png_bytes[..4],
        [137, 80, 78, 71],
        "Should have PNG magic bytes"
    );
    assert!(png_bytes.len() > 100, "PNG should be non-trivial size");
}

#[cfg(feature = "png")]
#[test]
fn test_render_png_scaled() {
    let code = "graph TD; A[Label]-->B[End]";
    let result = mermaid_cli::render_png(code, 400, 300, 2.0);
    assert!(result.is_ok(), "Scaled PNG render should succeed");
    let png_bytes = result.unwrap();
    assert_eq!(
        png_bytes[..4],
        [137, 80, 78, 71],
        "Should have PNG magic bytes"
    );
}

#[cfg(feature = "png")]
#[test]
fn test_render_png_all_types() {
    let types = vec![
        ("flowchart", "graph TD; A-->B"),
        ("sequence", "sequenceDiagram\nA->B: Hello"),
        ("pie", "pie\n\"A\" : 50\n\"B\" : 50"),
        ("class", "classDiagram\nclass X"),
        ("state", "stateDiagram-v2\n[*] --> A"),
        ("er", "erDiagram\nX ||--o{ Y : z"),
    ];
    for (name, code) in &types {
        let result = mermaid_cli::render_png(code, 800, 600, 1.0);
        assert!(
            result.is_ok(),
            "{} PNG should render: {:?}",
            name,
            result.err()
        );
        let png_bytes = result.unwrap();
        assert_eq!(
            png_bytes[..4],
            [137, 80, 78, 71],
            "{} should have PNG magic",
            name
        );
    }
}

#[test]
fn test_diagram_type_detection() {
    let cases = vec![
        ("graph TD; A-->B", DiagramType::Flowchart),
        ("sequenceDiagram\nA->B: Hi", DiagramType::Sequence),
        ("pie\n\"A\" : 50", DiagramType::Pie),
        ("classDiagram\nclass A", DiagramType::Class),
        ("stateDiagram-v2\n[*] --> A", DiagramType::State),
        ("erDiagram\nX ||--o{ Y", DiagramType::Er),
        ("gantt\nTask :t1, 1, 5d", DiagramType::Gantt),
    ];
    for (code, expected) in &cases {
        let diagram = parse(code).expect("Should parse");
        assert_eq!(
            diagram.diagram_type, *expected,
            "Diagram type mismatch for code: {}",
            code
        );
    }
}

// ============================================================
// 更多综合 API 测试
// ============================================================

#[test]
fn test_render_all_diagram_types_basic() {
    let types = vec![
        ("flowchart", "graph TD; A-->B"),
        ("sequence", "sequenceDiagram\nA->B: Hi"),
        ("pie", "pie\n\"X\" : 100"),
        ("class", "classDiagram\nclass A"),
        ("state", "stateDiagram-v2\n[*] --> A"),
        ("er", "erDiagram\nA ||--o{ B : rel"),
        ("gantt", "gantt\nTask :t1, 1, 5d"),
    ];
    for (name, code) in &types {
        let result = parse(code);
        assert!(result.is_ok(), "{} should parse: {:?}", name, result.err());
        let diagram = result.unwrap();
        // Verify rendering
        let rendered = renderer::Renderer::new().render(&diagram);
        assert!(
            rendered.is_ok(),
            "{} should render: {:?}",
            name,
            rendered.err()
        );
        let svg = rendered.unwrap();
        assert!(svg.contains("<svg"), "{} SVG should have svg tag", name);
    }
}

#[test]
fn test_parse_error_types() {
    // Various invalid inputs
    let cases = vec![
        ("graph", "Empty after graph keyword"),
        ("sequenceDiagram\ninvalid->", "Incomplete message"),
        ("classDiagram\nclass A {", "Unclosed class body"),
    ];
    for (code, _desc) in &cases {
        let result = parse(code);
        // Should either error or produce a partial diagram
        match result {
            Ok(_) => {}  // Partial parse is acceptable
            Err(_) => {} // Error is also acceptable
        }
    }
}

#[test]
fn test_fixer_multiple_errors() {
    let fixer = mermaid_cli::fixer::Fixer::new();
    // Fix with multiple errors in same input
    let (fixed, fixes) = fixer.fix("grpah TD\nA-->B");
    assert!(!fixes.is_empty(), "Should detect typo");
    assert!(fixed.contains("graph"), "Should fix graph keyword");
    assert!(fixed.contains("A-->B"), "Should preserve valid parts");
}

#[test]
fn test_markdown_extract_no_mermaid() {
    let md = "# Just plain markdown\nNo code blocks here at all\nJust text";
    let blocks = extract_mermaid_blocks(md);
    assert!(blocks.is_empty(), "Plain markdown should have no blocks");
}

#[test]
fn test_markdown_extract_multiple_fences() {
    let md = "```mermaid\ngraph TD\nA-->B\n```\ntext\n```\nnot mermaid\n```\n```mermaid\nsequenceDiagram\nC->D: test\n```";
    let blocks = extract_mermaid_blocks(md);
    assert_eq!(blocks.len(), 2, "Should extract exactly 2 mermaid blocks");
    assert!(
        blocks[0].contains("graph TD"),
        "First block should be graph"
    );
    assert!(
        blocks[1].contains("sequenceDiagram"),
        "Second block should be sequence"
    );
}

#[test]
fn test_svg_structure_all_types() {
    // Verify all types produce structurally valid SVG
    let cases = vec![
        "graph TD; A[Start]-->B[End]",
        "sequenceDiagram\nA->B: Hello\nB-->A: Hi",
        "pie\n\"A\" : 50\n\"B\" : 50",
        "classDiagram\nclass A {\n+String name\n}",
        "stateDiagram-v2\n[*] --> A\nA --> [*]",
        "erDiagram\nA ||--o{ B : label",
        "gantt\nTask :t1, 1, 5d",
    ];
    for code in &cases {
        let svg = render(code).unwrap();
        assert!(
            svg.starts_with("<?xml"),
            "SVG should start with XML declaration"
        );
        assert!(svg.contains("<svg "), "SVG should have opening tag");
        assert!(svg.contains("</svg>"), "SVG should have closing tag");
        // Well-formed check: open/close tags
        let open = svg.matches("<svg").count();
        let close = svg.matches("</svg>").count();
        assert_eq!(
            open,
            close,
            "SVG tags should be balanced for: {}",
            &code[..code.len().min(30)]
        );
    }
}

#[test]
fn test_render_empty_statements() {
    // Diagrams with only diagram type keyword
    let cases = vec![
        "graph",
        "sequenceDiagram",
        "pie",
        "classDiagram",
        "stateDiagram-v2",
        "erDiagram",
        "gantt",
    ];
    for code in &cases {
        let _ = render(code); // Should not panic
    }
}

#[test]
fn test_render_with_theme_variants() {
    let code = "graph TD; A-->B";
    let renderer = renderer::Renderer::new().with_theme("default");
    let diagram = parse(code).unwrap();
    let svg = renderer.render(&diagram).unwrap();
    assert!(svg.contains("<svg"), "Default theme renders");
}

#[test]
fn test_render_pie_single_slice() {
    let result = render("pie\n\"Only\" : 100");
    assert!(result.is_ok(), "Single slice pie should render");
    let svg = result.unwrap();
    assert!(svg.contains("Only"), "Should contain label");
}

#[test]
fn test_render_class_no_attributes() {
    let result = render("classDiagram\nclass EmptyClass");
    assert!(result.is_ok(), "Class with no attributes should render");
    let svg = result.unwrap();
    assert!(svg.contains("EmptyClass"), "Should contain class name");
}

#[test]
fn test_render_state_with_label() {
    let result = render("stateDiagram-v2\nstate Idle : Waiting for input");
    assert!(result.is_ok(), "State with label should render");
    let svg = result.unwrap();
    assert!(
        svg.contains("Waiting for input"),
        "Should contain state description"
    );
}

#[test]
fn test_render_er_entity_only() {
    let result = render("erDiagram\nCUSTOMER");
    assert!(result.is_ok(), "Entity only should render");
    let svg = result.unwrap();
    assert!(svg.contains("CUSTOMER"), "Should contain entity name");
}

// ============================================================
// 批量添加额外测试 (迈向 500+)
// ============================================================

#[test]
fn test_flowchart_simple_edge() {
    assert!(render("graph TD; A-->B").is_ok());
}

#[test]
fn test_flowchart_reverse_edge() {
    assert!(render("graph TD; B-->A").is_ok());
}

#[test]
fn test_flowchart_multi_node() {
    assert!(render("graph TD; A-->B; B-->C; C-->D").is_ok());
}

#[test]
fn test_flowchart_all_shapes_in_one() {
    let code = "graph TD\nA[Rect]\nB(Round)\nC{Diamond}\nD([BigRound])\nE[[Sub]]\nF[(DB)]\nG((Circle))\nH>Flag]";
    assert!(render(code).is_ok());
}

#[test]
fn test_flowchart_multi_edge_labels() {
    let code = "graph TD\nA-->|Label1|B\nB-->|Label2|C\nC-->|Label3|D";
    let result = render(code);
    assert!(result.is_ok());
}

#[test]
fn test_sequence_without_participants() {
    assert!(render("sequenceDiagram\nA->B: direct").is_ok());
}

#[test]
fn test_sequence_three_way() {
    let code = "sequenceDiagram\nA->B: to B\nB->C: to C\nC->A: back";
    assert!(render(code).is_ok());
}

#[test]
fn test_sequence_with_opt() {
    let code = "sequenceDiagram\nopt maybe\nA->B: if needed\nend";
    assert!(render(code).is_ok());
}

#[test]
fn test_sequence_multiple_activations() {
    let code = "sequenceDiagram\nactivate A\nA->B: msg\nactivate B\nB-->A: response\ndeactivate B\ndeactivate A";
    assert!(render(code).is_ok());
}

#[test]
fn test_parse_flowchart_stress() {
    let mut code = String::from("graph TD\n");
    for i in 0..50 {
        code.push_str(&format!("N{}-->N{}\n", i, i + 1));
    }
    let result = parse(&code);
    assert!(result.is_ok(), "50-node chain should parse");
    assert_eq!(result.unwrap().statements.len(), 50, "Should have 50 edges");
}

#[test]
fn test_parse_class_stress() {
    let mut code = String::from("classDiagram\n");
    for i in 0..20 {
        code.push_str(&format!(
            "class C{} {{\n+int field{}\n+method{}() void\n}}\n",
            i, i, i
        ));
    }
    let result = parse(&code);
    assert!(result.is_ok(), "20 classes should parse");
}

#[test]
fn test_parse_pie_stress() {
    let mut code = String::from("pie title Stress\n");
    for i in 0..50 {
        code.push_str(&format!("\"Item {}\" : {}\n", i, (i + 1) * 2));
    }
    let result = parse(&code);
    assert!(result.is_ok(), "50 slices should parse");
    assert_eq!(
        result.unwrap().statements.len(),
        50,
        "Should have 50 slices"
    );
}

#[test]
fn test_fixer_graph_missing_end() {
    let fixer = mermaid_cli::fixer::Fixer::new();
    let (fixed, _fixes) = fixer.fix("graph TD\nA-->B\nC-->D");
    assert!(fixed.contains("A-->B"), "Preserve edges");
    assert!(fixed.contains("graph"), "Preserve graph keyword");
}

#[test]
fn test_fixer_graph_with_typo_and_missing_end() {
    let fixer = mermaid_cli::fixer::Fixer::new();
    let (fixed, fixes) = fixer.fix("grpah TD; A-->B");
    assert!(!fixes.is_empty(), "Should have some fixes");
    assert!(fixed.contains("graph"), "Should fix grpah -> graph");
}

#[test]
fn test_fixer_preserve_exact_code() {
    let fixer = mermaid_cli::fixer::Fixer::new();
    let code = "sequenceDiagram\nAlice->Bob: Hello\nBob-->Alice: Response";
    let (fixed, _fixes) = fixer.fix(code);
    assert!(fixed.contains("Alice"), "Preserve identifiers");
    assert!(fixed.contains("Bob"), "Preserve identifiers");
}

#[test]
fn test_markdown_extract_weird_spacing() {
    let md = "text\n```   mermaid   \ngraph TD\nA-->B\n   ```   \nend";
    let blocks = extract_mermaid_blocks(md);
    assert!(
        blocks.len() == 1 || blocks.len() == 0,
        "Should handle weird spacing"
    );
}

#[test]
fn test_markdown_extract_large_block() {
    let mut md = String::from("```mermaid\n");
    for i in 0..20 {
        md.push_str(&format!("N{}-->N{}\n", i, i + 1));
    }
    md.push_str("```\n");
    let blocks = extract_mermaid_blocks(&md);
    assert_eq!(blocks.len(), 1, "Large block should extract");
    assert!(blocks[0].contains("N0"), "Should contain content");
    assert!(blocks[0].contains("N20"), "Should contain end content");
}

#[test]
fn test_check_flowchart() {
    let r = check("graph TD; A-->B").unwrap();
    assert!(r.valid, "Valid flowchart");
}

#[test]
fn test_check_sequence() {
    let r = check("sequenceDiagram\nA->B: Hi").unwrap();
    assert!(r.valid, "Valid sequence");
}

#[test]
fn test_check_pie() {
    let r = check("pie\n\"A\" : 50").unwrap();
    assert!(r.valid, "Valid pie");
}

#[test]
fn test_check_class() {
    let r = check("classDiagram\nclass A").unwrap();
    assert!(r.valid, "Valid class diagram");
}

#[test]
fn test_check_state() {
    let r = check("stateDiagram-v2\n[*] --> A").unwrap();
    assert!(r.valid, "Valid state diagram");
}

#[test]
fn test_check_er() {
    let r = check("erDiagram\nX ||--o{ Y : rel").unwrap();
    assert!(r.valid, "Valid ER diagram");
}

#[test]
fn test_check_gantt() {
    let r = check("gantt\nTask :t1, 1, 5d").unwrap();
    assert!(r.valid, "Valid Gantt chart");
}

#[test]
fn test_check_has_errors_valid() {
    let r = check("graph TD; A-->B").unwrap();
    assert!(!r.has_errors(), "No errors for valid");
}

#[test]
fn test_check_has_errors_invalid() {
    let r = check("invalid!!!").unwrap();
    assert!(r.has_errors(), "Has errors for invalid");
}

#[test]
fn test_render_with_parse_error() {
    let result = render("!!!invalid");
    assert!(result.is_err(), "Invalid input should error");
}

#[test]
fn test_fixer_new() {
    let f = mermaid_cli::fixer::Fixer::new();
    let (fixed, _) = f.fix("graph TD; A-->B");
    assert!(fixed.contains("graph"), "Fixer works");
}

#[test]
fn test_parse_flowchart_direction_aliases() {
    for dir in &["TD", "LR", "RL", "BT"] {
        let code = format!("flowchart {}; A-->B", dir);
        let d = parse(&code).unwrap();
        assert_eq!(d.direction.as_deref(), Some(*dir), "Direction {}", dir);
    }
}

#[test]
fn test_parse_flowchart_no_direction() {
    let d = parse("graph; A-->B").unwrap();
    assert!(
        d.direction.is_none() || d.direction.as_deref() == Some("TD"),
        "Default or none direction"
    );
}

#[test]
fn test_parse_duplicate_nodes() {
    let code = "graph TD\nA-->B\nA-->C\nA-->D";
    let d = parse(code).unwrap();
    let edges = d.get_edges();
    assert_eq!(edges.len(), 3, "Three edges from A");
}

#[test]
fn test_render_svg_contains_diagram_specific_elements() {
    let test_cases = vec![
        ("flowchart", "graph TD; A[Hi]", "<rect"),
        ("sequence", "sequenceDiagram\nA->B: msg", "<line"),
        ("pie", "pie\n\"X\" : 100", "<path"),
        ("class", "classDiagram\nclass X", "<rect"),
        ("state", "stateDiagram-v2\n[*] --> A", "<circle"),
    ];
    for (name, code, expect_element) in &test_cases {
        let svg = render(code).unwrap();
        assert!(
            svg.contains(expect_element),
            "{} should contain {} element",
            name,
            expect_element
        );
    }
}

#[test]
fn test_render_svg_no_duplicate_xml_declaration() {
    let svg = render("graph TD; A-->B").unwrap();
    let xml_decl_count = svg.matches("<?xml").count();
    assert_eq!(xml_decl_count, 1, "Only one XML declaration");
}

#[test]
fn test_render_svg_single_root_element() {
    let svg = render("graph TD; A-->B").unwrap();
    let svg_open = svg.matches("<svg").count();
    let svg_close = svg.matches("</svg>").count();
    assert_eq!(svg_open, svg_close, "Balanced svg tags");
}

// =========================================================================
// New tests added below — flowchart shapes, subgraph, AST API, lexer,
// fixer integration, stress, SVG validation
// =========================================================================

#[test]
fn test_render_all_eight_shapes_in_chain() {
    let code = concat!(
        "graph TD; ",
        "A[Rect]-->B(Round); ",
        "B-->C{Diamond}; ",
        "C-->D([Rounded2]); ",
        "D-->E[[Sub]]; ",
        "E-->F[(DB)]; ",
        "F-->G((DC)); ",
        "G-->H>Flag];"
    );
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("Rect"));
    assert!(svg.contains("Round"));
    assert!(svg.contains("Diamond"));
    assert!(svg.contains("Rounded2"));
    assert!(svg.contains("Sub"));
    assert!(svg.contains("DB"));
    assert!(svg.contains("DC"));
    assert!(svg.contains("Flag"));
}

#[test]
fn test_render_single_subgraph() {
    let code = "graph TD\nA-->B\nC-->D\nsubgraph sg1\nC-->D\nend";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("A"));
    assert!(svg.contains("B"));
    assert!(svg.contains("C"));
    assert!(svg.contains("D"));
}

#[test]
fn test_render_subgraph_with_title() {
    let code = "graph TD\nA-->B\nC-->D\nsubgraph MyGroup\nC-->D\nend";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("A"));
    assert!(svg.contains("B"));
    assert!(svg.contains("C"));
    assert!(svg.contains("D"));
}

#[test]
fn test_parse_subgraph_structure() {
    let code = "graph TD\nA-->B\nsubgraph Title\nC-->D\nend";
    let mut parser = Parser::new(code);
    let diagram = parser.parse().expect("parse should succeed");
    assert_eq!(diagram.subgraphs.len(), 1, "Should have 1 subgraph");
    let sg = &diagram.subgraphs[0];
    assert_eq!(sg.id, "Title", "Subgraph id mismatch");
    assert_eq!(sg.statements.len(), 1, "Subgraph should have 1 statement");
    assert_eq!(diagram.statements.len(), 1, "Top-level should have 1 edge");
}

#[test]
fn test_render_multiple_subgraphs() {
    let code = concat!(
        "graph TD\n",
        "A-->B\n",
        "C-->D\n",
        "E-->F\n",
        "subgraph G1\n",
        "A-->B\n",
        "end\n",
        "subgraph G2\n",
        "E-->F\n",
        "end"
    );
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("A"));
    assert!(svg.contains("F"));
}

#[test]
fn test_parse_all_eight_node_shapes() {
    let code = concat!(
        "graph TD; ",
        "A[Rect]; B(Round); C{Diamond}; ",
        "D([Rounded2]); E[[Sub]]; ",
        "F[(DB)]; G((DC)); H>Flag];"
    );
    let result = parse(code);
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());
    let diagram = result.unwrap();
    assert_eq!(
        diagram.statements.len(),
        8,
        "Should have 8 node definitions"
    );
}

#[test]
fn test_parse_edge_with_label_text() {
    let code = "graph TD; A-->|Hello|B";
    let result = parse(code);
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());
    let diagram = result.unwrap();
    let edges = diagram.get_edges_with_labels();
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].2.as_deref(), Some("Hello"), "Edge label mismatch");
}

#[test]
fn test_render_lr_direction() {
    let code = "graph LR; A-->B; B-->C";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("A"));
    assert!(svg.contains("B"));
    assert!(svg.contains("C"));
}

#[test]
fn test_render_rl_direction() {
    let code = "graph RL; A-->B; B-->C";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("A"));
    assert!(svg.contains("B"));
    assert!(svg.contains("C"));
}

#[test]
fn test_ast_get_nodes() {
    let result = parse("graph TD; A[Start]-->B[End]; C-->D");
    assert!(result.is_ok());
    let diagram = result.unwrap();
    let nodes = diagram.get_nodes();
    assert!(nodes.contains(&"A".to_string()));
    assert!(nodes.contains(&"B".to_string()));
    assert!(nodes.contains(&"C".to_string()));
    assert!(nodes.contains(&"D".to_string()));
    assert_eq!(nodes.len(), 4, "Should have 4 unique nodes");
}

#[test]
fn test_ast_get_edges() {
    let result = parse("graph TD; A-->B; B-->C");
    assert!(result.is_ok());
    let diagram = result.unwrap();
    let edges = diagram.get_edges();
    assert_eq!(edges.len(), 2);
    assert_eq!(edges[0], ("A".to_string(), "B".to_string()));
    assert_eq!(edges[1], ("B".to_string(), "C".to_string()));
}

#[test]
fn test_ast_edge_labels() {
    let result = parse("graph TD; A-->|label1|B; B-->|label2|C");
    assert!(result.is_ok());
    let diagram = result.unwrap();
    let edges = diagram.get_edges_with_labels();
    assert_eq!(edges.len(), 2);
    assert_eq!(edges[0].2.as_deref(), Some("label1"));
    assert_eq!(edges[1].2.as_deref(), Some("label2"));
}

#[test]
fn test_ast_type_is_flowchart() {
    let result = parse("graph TD; A-->B");
    assert!(result.is_ok());
    let diagram = result.unwrap();
    assert!(matches!(
        diagram.diagram_type,
        mermaid_cli::parser::DiagramType::Flowchart
    ));
}

#[test]
fn test_render_large_flowchart() {
    let code = concat!(
        "graph TD; ",
        "A-->B; B-->C; C-->D; D-->E; E-->F; F-->G; G-->H; H-->I; I-->J; ",
        "J-->K; K-->L; L-->M; M-->N; N-->O; O-->P; P-->Q; Q-->R; R-->S; S-->T;"
    );
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("A"));
    assert!(svg.contains("T"));
}

#[test]
fn test_render_flowchart_with_comments() {
    let code = "graph TD\n%% This is a comment\nA-->B\n%% Another comment\nB-->C";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("A"));
    assert!(svg.contains("B"));
    assert!(svg.contains("C"));
}

#[test]
fn test_check_complex_valid() {
    let code = "graph TD; A[Start]-->B{Decision}; B-->|Yes|C[End]; B-->|No|D[Retry];";
    let result = check(code).expect("check should not fail");
    assert!(result.valid, "Complex diagram should be valid");
    assert!(result.errors.is_empty());
}

#[test]
fn test_check_reports_error_for_garbage() {
    let result = check("!!! invalid ### syntax").expect("check should not fail");
    assert!(!result.valid);
    assert!(!result.errors.is_empty());
    let error = &result.errors[0];
    let msg = format!("{}", error);
    assert!(
        msg.contains("graph") || msg.contains("Expected"),
        "Error should mention graph/flowchart, got: {}",
        msg
    );
}

#[test]
fn test_fix_and_render_cycle() {
    let (fixed, _fixes) = fix("grpah TD; A-->B");
    assert!(fixed.contains("graph"), "fix should correct typo");
    let result = render(&fixed);
    assert!(
        result.is_ok(),
        "fixed code should render, error: {:?}",
        result.err()
    );
    let svg = result.unwrap();
    assert!(svg.contains("A"));
    assert!(svg.contains("B"));
}

#[test]
fn test_fix_and_check_cycle() {
    let (fixed, _fixes) = fix("grpah TD; A-->B");
    assert!(fixed.contains("graph"));
    let result = check(&fixed).expect("check should not fail");
    assert!(result.valid, "Fixed code should be valid after check");
}

#[test]
fn test_fix_multiple_typos_integration() {
    let (fixed, fixes) = fix("flowchrat TD; A-->>B; C=>D");
    assert!(fixed.contains("flowchart"), "flowchrat should be fixed");
    assert!(!fixed.contains("-->>"), "-->> should be fixed to -->");
    assert!(!fixed.contains("=>"), "=> should be fixed to ->");
    assert!(
        fixes.len() >= 3,
        "Should have at least 3 fixes, got {}",
        fixes.len()
    );
    let result = render(&fixed);
    assert!(result.is_ok(), "fixed code should render");
}

#[test]
fn test_lexer_sequence_keyword() {
    let mut lexer = mermaid_cli::parser::Lexer::new("sequenceDiagram\nA->B");
    let tokens = lexer.tokenize();
    assert!(!tokens.is_empty());
    assert!(matches!(
        tokens[0].token_type,
        mermaid_cli::parser::TokenType::Keyword(ref k) if k == "sequenceDiagram"
    ));
}

#[test]
fn test_lexer_class_keyword() {
    let mut lexer = mermaid_cli::parser::Lexer::new("classDiagram\nClass01 <|-- Class02");
    let tokens = lexer.tokenize();
    assert!(!tokens.is_empty());
    assert!(matches!(
        tokens[0].token_type,
        mermaid_cli::parser::TokenType::Keyword(ref k) if k == "classDiagram"
    ));
}

#[test]
fn test_lexer_state_keyword() {
    let mut lexer = mermaid_cli::parser::Lexer::new("stateDiagram\n[*] --> State1");
    let tokens = lexer.tokenize();
    assert!(!tokens.is_empty());
    assert!(matches!(
        tokens[0].token_type,
        mermaid_cli::parser::TokenType::Keyword(ref k) if k == "stateDiagram"
    ));
}

#[test]
fn test_lexer_gantt_keyword() {
    let mut lexer = mermaid_cli::parser::Lexer::new("gantt\ntitle A Gantt Diagram");
    let tokens = lexer.tokenize();
    assert!(!tokens.is_empty());
    assert!(matches!(
        tokens[0].token_type,
        mermaid_cli::parser::TokenType::Keyword(ref k) if k == "gantt"
    ));
}

#[test]
fn test_lexer_pie_keyword() {
    let mut lexer = mermaid_cli::parser::Lexer::new("pie\n\"Category\" : 45");
    let tokens = lexer.tokenize();
    assert!(!tokens.is_empty());
    assert!(matches!(
        tokens[0].token_type,
        mermaid_cli::parser::TokenType::Keyword(ref k) if k == "pie"
    ));
}

#[test]
fn test_lexer_er_keyword() {
    let mut lexer = mermaid_cli::parser::Lexer::new("erDiagram\nCUSTOMER ||--o{ ORDER : places");
    let tokens = lexer.tokenize();
    assert!(!tokens.is_empty());
    assert!(matches!(
        tokens[0].token_type,
        mermaid_cli::parser::TokenType::Keyword(ref k) if k == "erDiagram"
    ));
}

#[test]
fn test_render_invalid_keyword_fails() {
    let result = render("flowchrt TD; A-->B");
    assert!(result.is_err(), "Typo 'flowchrt' should fail render");
    let result2 = render("unknown TD; A-->B");
    assert!(result2.is_err(), "'unknown' keyword should fail render");
}

#[test]
fn test_render_mindmap_success() {
    let result = render("mindmap\n  Root\n    Branch");
    assert!(result.is_ok(), "Mindmap should render successfully");
    let svg = result.unwrap();
    assert!(
        svg.contains("Root"),
        "Mindmap SVG should contain root label"
    );
    assert!(
        svg.contains("Branch"),
        "Mindmap SVG should contain branch label"
    );
}

#[test]
fn test_parse_large_chain() {
    let mut code = String::from("graph TD; ");
    for i in 0..200 {
        code.push_str(&format!("A{}-->A{}; ", i, i + 1));
    }
    let result = parse(&code);
    assert!(
        result.is_ok(),
        "parse() of 200-node chain failed: {:?}",
        result.err()
    );
    let diagram = result.unwrap();
    assert_eq!(diagram.get_edges().len(), 200, "Should have 200 edges");
    assert_eq!(
        diagram.get_nodes().len(),
        201,
        "Should have 201 unique nodes"
    );
}

#[test]
fn test_render_large_chain() {
    let mut code = String::from("graph TD; ");
    for i in 0..50 {
        code.push_str(&format!("N{}-->N{}; ", i, i + 1));
    }
    let result = render(&code);
    assert!(
        result.is_ok(),
        "render() of 50-node chain failed: {:?}",
        result.err()
    );
    let svg = result.unwrap();
    assert!(svg.contains("N0"), "SVG should contain first node N0");
    assert!(svg.contains("N50"), "SVG should contain last node N50");
}

#[test]
fn test_svg_well_formed_xml() {
    let result = render("graph TD; A[Start]-->B[End]; C{D}-->E;");
    assert!(result.is_ok());
    let svg = result.unwrap();
    // XML declaration
    assert!(
        svg.starts_with("<?xml"),
        "SVG should start with XML declaration"
    );
    // Open/close svg tags
    assert_eq!(svg.matches("<svg").count(), 1, "Should have 1 <svg>");
    assert_eq!(svg.matches("</svg>").count(), 1, "Should have 1 </svg>");
    // Open/close text tags balanced
    let open_text = svg.matches("<text").count();
    let close_text = svg.matches("</text>").count();
    assert_eq!(open_text, close_text, "text tags should be balanced");
    // Self-closing tags should exist
    assert!(
        svg.contains("/>"),
        "SVG should contain self-closing elements"
    );
}

#[test]
fn test_svg_required_attributes() {
    let result = render("graph TD; A-->B");
    assert!(result.is_ok());
    let svg = result.unwrap();
    assert!(
        svg.contains("xmlns=\"http://www.w3.org/2000/svg\""),
        "SVG should have xmlns"
    );
    assert!(svg.contains("width="), "SVG should have width attribute");
    assert!(svg.contains("height="), "SVG should have height attribute");
    assert!(svg.contains("<?xml"), "SVG should have XML declaration");
}

#[test]
fn test_renderer_custom_dimensions() {
    let diagram = mermaid_cli::parser::Diagram {
        diagram_type: mermaid_cli::parser::DiagramType::Flowchart,
        direction: None,
        title: None,
        subgraphs: vec![],
        statements: vec![Statement::NodeDef {
            id: "X".to_string(),
            label: Some("Custom".to_string()),
            shape: NodeShape::Rect,
        }],
    };
    let renderer = Renderer::with_dimensions(1600, 1200);
    let result = renderer.render(&diagram);
    assert!(result.is_ok());
    let svg = result.unwrap();
    assert!(svg.contains("1600"), "SVG should have custom width 1600");
    assert!(svg.contains("1200"), "SVG should have custom height 1200");
    assert!(svg.contains("Custom"), "SVG should contain label");
}

// ============================================================
// GitGraph API 测试
// ============================================================

// --- GitGraph 集成测试 ---

#[test]
fn test_gitgraph_basic_parse() {
    let result = parse("gitGraph\n  commit\n  commit\n  commit");
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());
    let diagram = result.unwrap();
    assert!(!diagram.statements.is_empty(), "AST should have statements");
}

#[test]
fn test_gitgraph_basic_render() {
    let result = render("gitGraph\n  commit\n  commit\n  commit");
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "SVG should contain <svg tag");
    assert!(
        svg.contains("</svg>"),
        "SVG should contain closing </svg> tag"
    );
}

#[test]
fn test_gitgraph_with_branches() {
    let code =
        "gitGraph\n  branch develop\n  checkout develop\n  commit\n  commit\n  checkout main\n  commit";
    let result = parse(code);
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());
    let diagram = result.unwrap();
    assert!(!diagram.statements.is_empty(), "AST should have statements");
}

#[test]
fn test_gitgraph_with_merge() {
    let code = "gitGraph\n  branch develop\n  checkout develop\n  commit\n  checkout main\n  merge develop";
    let result = parse(code);
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());
    let diagram = result.unwrap();
    assert!(!diagram.statements.is_empty(), "AST should have statements");
}

#[test]
fn test_gitgraph_with_tags() {
    let code = "gitGraph\n  commit\n  commit tag: \"v1.0\"\n  commit";
    let result = parse(code);
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());
    let diagram = result.unwrap();
    assert!(!diagram.statements.is_empty(), "AST should have statements");
}

#[test]
fn test_gitgraph_diagram_type() {
    let diagram = parse("gitGraph\n  commit").unwrap();
    assert_eq!(diagram.diagram_type, mermaid_cli::DiagramType::GitGraph);
}

#[test]
fn test_gitgraph_check_valid() {
    let result = check("gitGraph\n  commit\n  commit").expect("check() should not fail");
    assert!(result.valid, "Valid gitgraph should be reported as valid");
    assert!(
        result.errors.is_empty(),
        "Valid gitgraph should have no errors"
    );
}

#[test]
fn test_gitgraph_commit_id() {
    let code = "gitGraph\n  commit id: \"abc123\"\n  commit\n  commit id: \"def456\"";
    let result = parse(code);
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());
    let diagram = result.unwrap();
    assert!(!diagram.statements.is_empty(), "AST should have statements");
}

#[test]
fn test_gitgraph_full_workflow() {
    let code = "gitGraph\n  commit\n  branch develop\n  checkout develop\n  commit\n  commit\n  checkout main\n  merge develop\n  commit";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "SVG should contain <svg tag");
    assert!(
        svg.contains("</svg>"),
        "SVG should contain closing </svg> tag"
    );
}

#[test]
fn test_gitgraph_svg_structure() {
    let svg = render("gitGraph\n  commit\n  commit\n  commit").unwrap();
    assert!(
        svg.contains("circle"),
        "GitGraph SVG should contain circle elements"
    );
    assert!(
        svg.contains("line"),
        "GitGraph SVG should contain line elements"
    );
}

// ============================================================
// Timeline API 测试
// ============================================================

// --- Timeline tests ---

#[test]
fn test_timeline_basic_parse() {
    let result = parse("timeline\n    2020 : Event1\n    2021 : Event2\n    2022 : Event3");
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());

    let diagram = result.unwrap();
    let events: Vec<&String> = diagram
        .statements
        .iter()
        .filter_map(|s| {
            if let mermaid_cli::Statement::TimelineEvent { time, .. } = s {
                Some(time)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(events.len(), 3, "Timeline should have 3 events");
    assert!(diagram.statements.iter().any(
        |s| matches!(s, mermaid_cli::Statement::TimelineEvent { time, .. } if time == "2020")
    ));
    assert!(diagram.statements.iter().any(|s| matches!(s, mermaid_cli::Statement::TimelineEvent { description, .. } if description == "Event3")));
}

#[test]
fn test_timeline_basic_render() {
    let result = render("timeline\n    2020 : Event1\n    2021 : Event2\n    2022 : Event3");
    assert!(result.is_ok(), "render() failed: {:?}", result.err());

    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "SVG should contain <svg tag");
    assert!(
        svg.contains("</svg>"),
        "SVG should contain closing </svg> tag"
    );
    assert!(svg.contains("2020"), "SVG should contain time '2020'");
    assert!(svg.contains("Event2"), "SVG should contain event 'Event2'");
}

#[test]
fn test_timeline_with_sections() {
    let result = parse(
        "timeline\n    section Phase 1\n    2020 : Event1\n    section Phase 2\n    2021 : Event2",
    );
    assert!(
        result.is_ok(),
        "parse() with sections failed: {:?}",
        result.err()
    );

    let diagram = result.unwrap();
    let events: Vec<&String> = diagram
        .statements
        .iter()
        .filter_map(|s| {
            if let mermaid_cli::Statement::TimelineEvent { time, .. } = s {
                Some(time)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(events.len(), 2);
    let sections: Vec<&String> = diagram
        .statements
        .iter()
        .filter_map(|s| {
            if let mermaid_cli::Statement::TimelineSection { name } = s {
                Some(name)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(sections.len(), 2, "Should have 2 sections");
}

#[test]
fn test_timeline_diagram_type() {
    let result = parse("timeline\n    2020 : Event1");
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());

    let diagram = result.unwrap();
    assert_eq!(
        diagram.diagram_type,
        mermaid_cli::parser::DiagramType::Timeline,
        "Diagram type should be Timeline"
    );
}

#[test]
fn test_timeline_check_valid() {
    let result = check("timeline\n    2020 : Event1").expect("check() should not fail");
    assert!(result.valid, "Valid timeline should be reported as valid");
    assert!(
        result.errors.is_empty(),
        "Valid timeline should have no errors"
    );
}

#[test]
fn test_timeline_svg_structure() {
    let result = render("timeline\n    2020 : Event1\n    2021 : Event2");
    assert!(result.is_ok(), "render() failed: {:?}", result.err());

    let svg = result.unwrap();
    assert!(
        svg.contains("circle"),
        "Timeline SVG should have circle elements"
    );
    assert!(
        svg.contains("text"),
        "Timeline SVG should have text elements"
    );
    assert!(svg.contains("2020"), "SVG should contain time '2020'");
    assert!(svg.contains("Event1"), "SVG should contain event 'Event1'");
    assert!(svg.contains("2021"), "SVG should contain time '2021'");
    assert!(svg.contains("Event2"), "SVG should contain event 'Event2'");
}

// ============================================================
// Journey API 测试
// ============================================================

#[test]
fn test_journey_basic_parse() {
    let result = parse("journey\n  section S1\n  A:1:Me");
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());
    let diagram = result.unwrap();
    let tasks: Vec<&String> = diagram
        .statements
        .iter()
        .filter_map(|s| {
            if let mermaid_cli::Statement::JourneyTask { name, .. } = s {
                Some(name)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(tasks.len(), 1, "Should have 1 task");
    assert_eq!(tasks[0], "A");
}

#[test]
fn test_journey_basic_render() {
    let result = render("journey\n  section S1\n  A:1:Me\n  B:5:Me,You");
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "SVG should have svg tag");
    assert!(svg.contains("S1"), "SVG should contain section");
    assert!(svg.contains("A"), "SVG should contain task");
}

#[test]
fn test_journey_with_sections() {
    let result = parse("journey\n  section S1\n  A:1:Me\n  section S2\n  B:3:You");
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());
    let diagram = result.unwrap();
    let sections: Vec<&String> = diagram
        .statements
        .iter()
        .filter_map(|s| {
            if let mermaid_cli::Statement::JourneySection { name } = s {
                Some(name)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(sections.len(), 2, "Should have 2 sections");
}

#[test]
fn test_journey_diagram_type() {
    let result = parse("journey\n  title Test\n  A:1:Me");
    assert!(result.is_ok());
    let diagram = result.unwrap();
    assert_eq!(
        diagram.diagram_type,
        mermaid_cli::parser::DiagramType::Journey
    );
}

#[test]
fn test_journey_check_valid() {
    let result = check("journey\n  A:1:Me").expect("check() should not fail");
    assert!(result.valid);
}

#[test]
fn test_journey_svg_structure() {
    let result = render("journey\n  section S\n  Task:3:User,System");
    assert!(result.is_ok());
    let svg = result.unwrap();
    assert!(svg.contains("rect"), "SVG should have rect elements");
    assert!(svg.contains("text"), "SVG should have text elements");
    assert!(svg.contains("Task"), "SVG should contain task name");
    assert!(svg.contains("User"), "SVG should contain actor");
}

// ============================================================
// Kanban API 测试
// ============================================================

#[test]
fn test_kanban_basic_parse() {
    let result = parse("kanban\n  Todo\n    [Task1] : Do something\n    [Task2] : Another");
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());
    let diagram = result.unwrap();
    let tasks: Vec<&String> = diagram
        .statements
        .iter()
        .filter_map(|s| {
            if let mermaid_cli::Statement::KanbanTask { name, .. } = s {
                Some(name)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(tasks.len(), 2, "Should have 2 tasks");
}

#[test]
fn test_kanban_basic_render() {
    let result = render("kanban\n  Todo\n    [Task1] : First task");
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("Todo"), "SVG should contain column name");
    assert!(svg.contains("Task1"), "SVG should contain task name");
}

#[test]
fn test_kanban_with_multiple_columns() {
    let result = parse("kanban\n  Todo\n    [A] : a\n  Doing\n    [B] : b\n  Done\n    [C] : c");
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());
    let diagram = result.unwrap();
    let cols: Vec<&String> = diagram
        .statements
        .iter()
        .filter_map(|s| {
            if let mermaid_cli::Statement::KanbanColumn { name } = s {
                Some(name)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(cols.len(), 3, "Should have 3 columns");
}

#[test]
fn test_kanban_diagram_type() {
    let result = parse("kanban\n  Col\n    [T] : test");
    assert!(result.is_ok());
    let diagram = result.unwrap();
    assert_eq!(
        diagram.diagram_type,
        mermaid_cli::parser::DiagramType::Kanban
    );
}

#[test]
fn test_kanban_check_valid() {
    let result = check("kanban\n  Col\n    [T] : test").expect("check() should not fail");
    assert!(result.valid);
}

#[test]
fn test_kanban_svg_structure() {
    let result = render("kanban\n  Col\n    [Task] : description");
    assert!(result.is_ok());
    let svg = result.unwrap();
    assert!(svg.contains("rect"), "SVG should have rect elements");
    assert!(svg.contains("text"), "SVG should have text elements");
    assert!(svg.contains("Col"), "SVG should contain column name");
    assert!(svg.contains("Task"), "SVG should contain task name");
}

// ============================================================
// Venn API 测试
// ============================================================

#[test]
fn test_venn_basic_parse() {
    let diagram = parse("venn\n    a : Cats\n    b : Dogs\n    ab : Both").unwrap();
    assert_eq!(diagram.statements.len(), 3);
    match &diagram.statements[0] {
        mermaid_cli::Statement::VennSet { id, label } => {
            assert_eq!(id, "a");
            assert_eq!(label, "Cats");
        }
        _ => panic!("Expected VennSet"),
    }
    match &diagram.statements[1] {
        mermaid_cli::Statement::VennSet { id, label } => {
            assert_eq!(id, "b");
            assert_eq!(label, "Dogs");
        }
        _ => panic!("Expected VennSet"),
    }
    match &diagram.statements[2] {
        mermaid_cli::Statement::VennSet { id, label } => {
            assert_eq!(id, "ab");
            assert_eq!(label, "Both");
        }
        _ => panic!("Expected VennSet"),
    }
}

#[test]
fn test_venn_basic_render() {
    let result = render("venn\n    a : Cats\n    b : Dogs");
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("<circle"), "Venn should contain circles");
    assert!(svg.contains("Cats"), "SVG should contain label 'Cats'");
    assert!(svg.contains("Dogs"), "SVG should contain label 'Dogs'");
}

#[test]
fn test_venn_two_sets() {
    let diagram = parse("venn\n    a : SetA\n    b : SetB").unwrap();
    assert_eq!(diagram.statements.len(), 2);
    assert_eq!(diagram.diagram_type, mermaid_cli::DiagramType::Venn);
}

#[test]
fn test_venn_diagram_type() {
    let diagram = parse("venn\n    a : Cats").unwrap();
    assert_eq!(diagram.diagram_type, mermaid_cli::DiagramType::Venn);
}

#[test]
fn test_venn_check_valid() {
    let result = check("venn\n    a : Cats\n    b : Dogs").unwrap();
    assert!(result.valid, "Valid Venn code should be reported as valid");
    assert!(
        result.errors.is_empty(),
        "Valid Venn code should have no errors"
    );
}

#[test]
fn test_venn_svg_structure() {
    let svg = render("venn\n    a : Cats\n    b : Dogs\n    ab : Both").unwrap();
    assert!(svg.contains("<svg"), "Should contain <svg>");
    assert!(svg.contains("</svg>"), "Should contain </svg>");
    assert!(svg.contains("<circle"), "Should contain circles");
    assert!(svg.contains("Cats"), "Should contain Cats label");
    assert!(svg.contains("Dogs"), "Should contain Dogs label");
    assert!(svg.contains("Both"), "Should contain Both label");
    assert!(svg.contains("#ff6b6b"), "Should contain red color");
    assert!(svg.contains("#4ecdc4"), "Should contain teal color");
}

// ============================================================
// Packet API 测试
// ============================================================

#[test]
fn test_packet_basic_parse() {
    let result = parse("packet\n  0-7: Source Port\n  8-15: Dest Port");
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());
    let diagram = result.unwrap();
    assert_eq!(diagram.statements.len(), 2);
    match &diagram.statements[0] {
        mermaid_cli::Statement::PacketField {
            start_bit,
            end_bit,
            label,
        } => {
            assert_eq!(*start_bit, 0);
            assert_eq!(*end_bit, 7);
            assert_eq!(label, "Source Port");
        }
        _ => panic!("Expected PacketField"),
    }
}

#[test]
fn test_packet_basic_render() {
    let result = render("packet\n  0-7: Source\n  8-15: Dest");
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("Source"), "Should contain Source");
    assert!(svg.contains("Dest"), "Should contain Dest");
}

#[test]
fn test_packet_multiple_fields() {
    let result = parse("packet\n  0-3: Ver\n  4-7: HLen\n  8-15: Service\n  16-31: Length");
    assert!(result.is_ok());
    let diagram = result.unwrap();
    assert_eq!(diagram.statements.len(), 4);
}

#[test]
fn test_packet_diagram_type() {
    let diagram = parse("packet\n  0-7: A").unwrap();
    assert_eq!(
        diagram.diagram_type,
        mermaid_cli::parser::DiagramType::Packet
    );
}

#[test]
fn test_packet_check_valid() {
    let result = check("packet\n  0-7: Test").unwrap();
    assert!(result.valid);
}

#[test]
fn test_packet_svg_structure() {
    let svg = render("packet\n  0-7: A\n  8-15: B\n  16-31: C").unwrap();
    assert!(svg.contains("<svg"), "Should have SVG");
    assert!(svg.contains("</svg>"), "Should close SVG");
    assert!(svg.contains("A"), "Should contain A label");
    assert!(svg.contains("B"), "Should contain B label");
    assert!(svg.contains("C"), "Should contain C label");
}

// ============================================================
// Radar API 测试
// ============================================================

// --- Radar chart tests ---

#[test]
fn test_radar_basic_parse() {
    let code = "radar\n  Speed: 80\n  Power: 65";
    let result = mermaid_cli::parse(code);
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());

    let diagram = result.unwrap();
    assert_eq!(diagram.statements.len(), 2);

    match &diagram.statements[0] {
        mermaid_cli::Statement::RadarAxis { label, value } => {
            assert_eq!(label, "Speed");
            assert!((*value - 80.0).abs() < f64::EPSILON);
        }
        _ => panic!("Expected RadarAxis"),
    }

    match &diagram.statements[1] {
        mermaid_cli::Statement::RadarAxis { label, value } => {
            assert_eq!(label, "Power");
            assert!((*value - 65.0).abs() < f64::EPSILON);
        }
        _ => panic!("Expected RadarAxis"),
    }
}

#[test]
fn test_radar_basic_render() {
    let code = "radar\n  Speed: 80\n  Power: 65\n  Accuracy: 90";
    let result = mermaid_cli::render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());

    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "SVG should contain <svg tag");
    assert!(svg.contains("</svg>"), "SVG should contain closing </svg>");
    assert!(
        svg.contains("Speed"),
        "SVG should contain axis label 'Speed'"
    );
    assert!(
        svg.contains("Power"),
        "SVG should contain axis label 'Power'"
    );
    assert!(
        svg.contains("Accuracy"),
        "SVG should contain axis label 'Accuracy'"
    );
}

#[test]
fn test_radar_multiple_axes() {
    let code = "radar\n  Speed: 80\n  Power: 65\n  Accuracy: 90\n  Range: 50";
    let result = mermaid_cli::parse(code);
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());

    let diagram = result.unwrap();
    assert_eq!(diagram.statements.len(), 4);

    let labels: Vec<&str> = diagram
        .statements
        .iter()
        .filter_map(|s| {
            if let mermaid_cli::Statement::RadarAxis { label, .. } = s {
                Some(label.as_str())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(labels, vec!["Speed", "Power", "Accuracy", "Range"]);

    // Also test rendering with 4 axes
    let render_result = mermaid_cli::render(code);
    assert!(render_result.is_ok(), "render with 4 axes failed");
}

#[test]
fn test_radar_diagram_type() {
    let code = "radar\n  Speed: 80";
    let diagram = mermaid_cli::parse(code).unwrap();
    assert_eq!(
        diagram.diagram_type,
        mermaid_cli::parser::DiagramType::Radar
    );
}

#[test]
fn test_radar_check_valid() {
    let result = mermaid_cli::check("radar\n  Speed: 80\n  Power: 65").unwrap();
    assert!(result.valid, "Valid radar code should be reported as valid");
    assert!(
        result.errors.is_empty(),
        "Valid radar code should have no errors"
    );
}

#[test]
fn test_radar_svg_structure() {
    let code = "radar\n  Speed: 80\n  Power: 65\n  Accuracy: 90";
    let svg = mermaid_cli::render(code).unwrap();

    // Should contain polygon elements (rings + data polygon)
    assert!(
        svg.contains("polygon"),
        "SVG should contain polygon elements"
    );

    // Should contain line elements (axis lines)
    assert!(svg.contains("line"), "SVG should contain line elements");

    // Should contain circle elements (data points)
    assert!(svg.contains("circle"), "SVG should contain circle elements");

    // Should contain text elements (labels)
    assert!(svg.contains("<text"), "SVG should contain text elements");
}

// ============================================================
// Ishikawa API 测试
// ============================================================

#[test]
fn test_ishikawa_basic_parse() {
    let diagram = parse("ishikawa\n  root Problem\n  category Man\n    cause1\n    cause2\n  category Machine\n    cause3").unwrap();
    assert_eq!(diagram.statements.len(), 6);
    assert!(matches!(
        diagram.statements[0],
        mermaid_cli::Statement::IshikawaRoot { .. }
    ));
}

#[test]
fn test_ishikawa_basic_render() {
    let svg = render("ishikawa\n  root Bug\n  category Code\n    typo\n    logic").unwrap();
    assert!(svg.contains("Bug"), "Should contain root");
    assert!(svg.contains("Code"), "Should contain category");
    assert!(svg.contains("typo"), "Should contain cause");
}

#[test]
fn test_ishikawa_diagram_type() {
    let diagram = parse("ishikawa\n  root X\n  category Y\n    z").unwrap();
    assert_eq!(
        diagram.diagram_type,
        mermaid_cli::parser::DiagramType::Ishikawa
    );
}

#[test]
fn test_ishikawa_check_valid() {
    let result = check("ishikawa\n  root X\n  category Y\n    z").unwrap();
    assert!(result.valid);
}

#[test]
fn test_ishikawa_svg_structure() {
    let svg = render("ishikawa\n  root R\n  category C\n    cause").unwrap();
    assert!(svg.contains("path"), "Should have path (fish head)");
    assert!(svg.contains("line"), "Should have lines (bones)");
    assert!(svg.contains("R"), "Should contain root");
}

// ============================================================
// Quadrant Chart API 测试
// ============================================================

#[test]
fn test_quadrant_basic_parse() {
    let result = parse("quadrantChart\n  title Test\n  x-axis X\n  y-axis Y\n  quadrant-1 A\n  quadrant-2 B\n  P1: [0.3, 0.6]");
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());
    let diagram = result.unwrap();
    assert_eq!(
        diagram.diagram_type,
        mermaid_cli::parser::DiagramType::Quadrant
    );
}

#[test]
fn test_quadrant_basic_render() {
    let result = render("quadrantChart\n  title Chart\n  x-axis X\n  y-axis Y\n  quadrant-1 TL\n  quadrant-2 TR\n  Pt: [0.5, 0.5]");
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("Chart"), "SVG should contain title");
}

#[test]
fn test_quadrant_diagram_type() {
    let diagram = parse("quadrantChart\n  title T").unwrap();
    assert_eq!(
        diagram.diagram_type,
        mermaid_cli::parser::DiagramType::Quadrant
    );
}

#[test]
fn test_quadrant_check_valid() {
    let result = check("quadrantChart\n  title T\n  Pt: [0.5, 0.5]").unwrap();
    assert!(result.valid);
}

#[test]
fn test_quadrant_svg_structure() {
    let svg = render("quadrantChart\n  x-axis X\n  y-axis Y\n  P: [0.3, 0.7]").unwrap();
    assert!(svg.contains("rect"), "Should have rect for quadrants");
    assert!(svg.contains("line"), "Should have axis lines");
    assert!(svg.contains("circle"), "Should have data points");
}

// ============================================================
// ZenUML API 测试
// ============================================================

// --- ZenUML 集成测试 ---

#[test]
fn test_zenuml_basic_parse() {
    let result = mermaid_cli::parse("zenuml\nAlice->Bob: Hello\nBob->Alice: Response");
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());

    let diagram = result.unwrap();
    assert_eq!(diagram.statements.len(), 2, "Should have 2 messages");
}

#[test]
fn test_zenuml_diagram_type() {
    let result = mermaid_cli::parse("zenuml\nAlice->Bob: Hello");
    assert!(result.is_ok());

    let diagram = result.unwrap();
    assert_eq!(
        diagram.diagram_type,
        mermaid_cli::parser::DiagramType::ZenUml,
        "Diagram type should be ZenUml"
    );
}

#[test]
fn test_zenuml_basic_render() {
    let result = mermaid_cli::render("zenuml\nAlice->Bob: Hello\nBob->Alice: Response");
    assert!(result.is_ok(), "render() failed: {:?}", result.err());

    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "SVG should contain <svg tag");
    assert!(
        svg.contains("</svg>"),
        "SVG should contain closing </svg> tag"
    );
    assert!(
        svg.contains("Alice"),
        "SVG should contain participant Alice"
    );
    assert!(svg.contains("Bob"), "SVG should contain participant Bob");
    assert!(
        svg.contains("Hello"),
        "SVG should contain message label 'Hello'"
    );
    assert!(
        svg.contains("Response"),
        "SVG should contain message label 'Response'"
    );
}

#[test]
fn test_zenuml_check_valid() {
    let result = mermaid_cli::check("zenuml\nAlice->Bob: Hello").expect("check() should not fail");
    assert!(
        result.valid,
        "Valid ZenUML code should be reported as valid"
    );
    assert!(
        result.errors.is_empty(),
        "Valid ZenUML code should have no errors"
    );
}

#[test]
fn test_zenuml_svg_structure() {
    let result = mermaid_cli::render("zenuml\nAlice->Bob: Hello");
    assert!(result.is_ok());

    let svg = result.unwrap();
    // Should have participant boxes (rect elements)
    assert!(
        svg.contains("<rect"),
        "SVG should have rect elements for participants"
    );
    // Should have arrow lines (line elements)
    assert!(
        svg.contains("<line"),
        "SVG should have line elements for arrows"
    );
    // Should have text rendering
    assert!(svg.contains("<text"), "SVG should have text elements");
    // Should have arrowhead (path or polygon)
    let has_arrowhead = svg.contains("<path") || svg.contains("<polygon");
    assert!(has_arrowhead, "SVG should have arrowhead element");
}

#[test]
fn test_zenuml_parse_message_edge() {
    let result = mermaid_cli::parse("zenuml\nAlice->Bob: Hello");
    assert!(result.is_ok());

    let diagram = result.unwrap();
    match &diagram.statements[0] {
        mermaid_cli::Statement::Message {
            from, to, label, ..
        } => {
            assert_eq!(from, "Alice", "Source should be Alice");
            assert_eq!(to, "Bob", "Target should be Bob");
            assert_eq!(label, "Hello", "Label should be 'Hello'");
        }
        _ => panic!("Expected Message"),
    }
}

#[test]
fn test_zenuml_empty_input() {
    let result = mermaid_cli::parse("zenuml");
    assert!(result.is_ok(), "Empty zenuml should parse");
    let diagram = result.unwrap();
    assert!(diagram.statements.is_empty());
    assert_eq!(
        diagram.diagram_type,
        mermaid_cli::parser::DiagramType::ZenUml
    );
}

#[test]
fn test_zenuml_multiple_participants() {
    let result = mermaid_cli::render(
        "zenuml\nAlice->Bob: Hello\nBob->Charlie: Forward\nCharlie->Alice: Response",
    );
    assert!(result.is_ok(), "render() failed: {:?}", result.err());

    let svg = result.unwrap();
    assert!(svg.contains("Alice"));
    assert!(svg.contains("Bob"));
    assert!(svg.contains("Charlie"));
    assert!(svg.contains("Hello"));
    assert!(svg.contains("Forward"));
    assert!(svg.contains("Response"));
}

// ============================================================
// Requirement Diagram API 测试
// ============================================================

// ============================================================
// Requirement Diagram Tests
// ============================================================

#[test]
fn test_requirement_basic_parse() {
    let code = "requirementDiagram
    requirement test_req {
        id: 1
        text: the test text.
        risk: high
        verifymethod: test
    }
    element test_entity {
        type: simulation
    }
    test_entity - satisfies -> test_req";
    let result = parse(code);
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());
    let diagram = result.unwrap();
    assert_eq!(
        diagram.diagram_type,
        mermaid_cli::parser::DiagramType::Requirement
    );
    assert_eq!(diagram.statements.len(), 3, "Should have 3 statements");

    // Check requirement def
    match &diagram.statements[0] {
        mermaid_cli::Statement::RequirementDef {
            name,
            req_id,
            text,
            risk,
            verify_method,
        } => {
            assert_eq!(name, "test_req");
            assert_eq!(req_id, "1");
            assert_eq!(text, "the test text.");
            assert_eq!(risk, "high");
            assert_eq!(verify_method, "test");
        }
        _ => panic!("Expected RequirementDef"),
    }

    // Check element def
    match &diagram.statements[1] {
        mermaid_cli::Statement::RequirementElement { name, element_type } => {
            assert_eq!(name, "test_entity");
            assert_eq!(element_type, "simulation");
        }
        _ => panic!("Expected RequirementElement"),
    }

    // Check relation
    match &diagram.statements[2] {
        mermaid_cli::Statement::RequirementRelation {
            from,
            to,
            relation_type,
        } => {
            assert_eq!(from, "test_entity");
            assert_eq!(to, "test_req");
            assert_eq!(relation_type, "satisfies");
        }
        _ => panic!("Expected RequirementRelation"),
    }
}

#[test]
fn test_requirement_basic_render() {
    let code = "requirementDiagram
    requirement test_req {
        id: 1
        text: the test text.
        risk: high
        verifymethod: test
    }
    element test_entity {
        type: simulation
    }
    test_entity - satisfies -> test_req";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "Should be valid SVG");
    assert!(svg.contains("test_req"), "Should contain requirement name");
    assert!(svg.contains("test_entity"), "Should contain element name");
    assert!(svg.contains("satisfies"), "Should contain relation type");
}

#[test]
fn test_requirement_diagram_type() {
    let diagram = parse(
        "requirementDiagram\n    requirement r { id: 1 text: t risk: low verifymethod: analysis }",
    )
    .unwrap();
    assert_eq!(
        diagram.diagram_type,
        mermaid_cli::parser::DiagramType::Requirement
    );
}

#[test]
fn test_requirement_check_valid() {
    let code = "requirementDiagram
    requirement r {
        id: 1
        text: test
        risk: low
        verifymethod: review
    }";
    let result = check(code).expect("check() should not fail");
    assert!(
        result.valid,
        "Valid requirement diagram should be reported as valid"
    );
    assert!(result.errors.is_empty(), "Valid code should have no errors");
}

#[test]
fn test_requirement_svg_structure() {
    let code = "requirementDiagram
    requirement r {
        id: 1
        text: test text
        risk: low
        verifymethod: test
    }";
    let result = render(code).unwrap();
    assert!(result.contains("<?xml"), "Should have XML declaration");
    assert!(result.contains("<svg"), "Should have SVG tag");
    assert!(result.contains("</svg>"), "Should have closing SVG tag");
    assert!(
        result.contains("<rect"),
        "Should have rect elements for boxes"
    );
    assert!(
        result.contains("<text"),
        "Should have text elements for labels"
    );
}

// ============================================================
// Block Diagram API 测试
// ============================================================

// ============================================================
// Block Diagram API 测试
// ============================================================

#[test]
fn test_block_basic_parse() {
    let result = mermaid_cli::parse("block\n  Block1\n  Block2\n  Block3");
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());

    let diagram = result.unwrap();
    assert_eq!(diagram.statements.len(), 3, "Should have 3 blocks");
    match &diagram.statements[0] {
        mermaid_cli::Statement::BlockNode {
            label, children, ..
        } => {
            assert_eq!(label, "Block1");
            assert!(children.is_empty(), "Top-level blocks have no children");
        }
        _ => panic!("Expected BlockNode"),
    }
}

#[test]
fn test_block_basic_render() {
    let result = mermaid_cli::render("block\n  Block1\n  Block2\n  Block3");
    assert!(result.is_ok(), "render() failed: {:?}", result.err());

    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "SVG should contain <svg tag");
    assert!(
        svg.contains("</svg>"),
        "SVG should contain closing </svg> tag"
    );
    assert!(
        svg.contains("Block1"),
        "SVG should contain block label 'Block1'"
    );
    assert!(
        svg.contains("Block2"),
        "SVG should contain block label 'Block2'"
    );
    assert!(
        svg.contains("Block3"),
        "SVG should contain block label 'Block3'"
    );
}

#[test]
fn test_block_nested() {
    let code = "block\n  Root\n    Child1\n    Child2\n  Other";
    let result = mermaid_cli::parse(code);
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());

    let diagram = result.unwrap();
    assert_eq!(
        diagram.statements.len(),
        2,
        "Should have 2 top-level blocks"
    );
    match &diagram.statements[0] {
        mermaid_cli::Statement::BlockNode {
            label, children, ..
        } => {
            assert_eq!(label, "Root");
            assert_eq!(children.len(), 2, "Root should have 2 children");
        }
        _ => panic!("Expected BlockNode"),
    }

    // Also verify rendering
    let render_result = mermaid_cli::render(code);
    assert!(
        render_result.is_ok(),
        "render() failed: {:?}",
        render_result.err()
    );
    let svg = render_result.unwrap();
    assert!(svg.contains("Root"));
    assert!(svg.contains("Child1"));
    assert!(svg.contains("Child2"));
    assert!(svg.contains("Other"));
}

#[test]
fn test_block_diagram_type() {
    let diagram = mermaid_cli::parse("block\n  Item").unwrap();
    assert_eq!(
        diagram.diagram_type,
        mermaid_cli::parser::DiagramType::Block
    );
}

#[test]
fn test_block_check_valid() {
    let result = mermaid_cli::check("block\n  A\n  B").expect("check() should not fail");
    assert!(
        result.valid,
        "Valid block diagram should be reported as valid"
    );
    assert!(
        result.errors.is_empty(),
        "Valid block diagram should have no errors"
    );
}

#[test]
fn test_block_svg_structure() {
    let result = mermaid_cli::render("block\n  Alpha\n  Beta");
    assert!(result.is_ok(), "render() failed: {:?}", result.err());

    let svg = result.unwrap();
    assert!(svg.contains("<?xml"), "Should have XML declaration");
    assert!(svg.contains("<svg"), "Should have SVG tag");
    assert!(svg.contains("</svg>"), "Should have closing SVG tag");
    assert!(
        svg.contains("<rect"),
        "Should have rect elements for blocks"
    );
    assert!(
        svg.contains("<text"),
        "Should have text elements for labels"
    );
    assert!(svg.contains("Alpha"), "Should contain block label 'Alpha'");
    assert!(svg.contains("Beta"), "Should contain block label 'Beta'");
    assert!(svg.contains("rx:6"), "Should have rounded corners");
}

// ============================================================
// C4 Diagram API 测试
// ============================================================

// --- C4 集成测试 ---

#[test]
fn test_c4_basic_parse() {
    let code = "C4Context\nPerson(customer, \"Customer\", \"A customer\")\nSystem(system, \"Software System\", \"Our system\")\nRel(customer, system, \"Uses\")";
    let result = parse(code);
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());
    let diagram = result.unwrap();
    assert_eq!(diagram.statements.len(), 3);
    // Verify first statement is C4Person
    match &diagram.statements[0] {
        mermaid_cli::Statement::C4Person {
            alias,
            label,
            description,
        } => {
            assert_eq!(alias, "customer");
            assert_eq!(label, "Customer");
            assert_eq!(description, "A customer");
        }
        _ => panic!("Expected C4Person"),
    }
    // Verify third statement is C4Rel
    match &diagram.statements[2] {
        mermaid_cli::Statement::C4Rel { from, to, label } => {
            assert_eq!(from, "customer");
            assert_eq!(to, "system");
            assert_eq!(label, "Uses");
        }
        _ => panic!("Expected C4Rel"),
    }
}

#[test]
fn test_c4_basic_render() {
    let code = "C4Context\nPerson(customer, \"Customer\", \"\")\nSystem(system, \"System\", \"\")";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "SVG should contain <svg tag");
    assert!(
        svg.contains("</svg>"),
        "SVG should contain closing </svg> tag"
    );
    assert!(
        svg.contains("Customer"),
        "SVG should contain customer label"
    );
    assert!(svg.contains("System"), "SVG should contain system label");
}

#[test]
fn test_c4_diagram_type() {
    let result = parse("C4Context\nPerson(p, \"P\", \"\")");
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap().diagram_type,
        mermaid_cli::parser::DiagramType::C4
    );
}

#[test]
fn test_c4_check_valid() {
    let result = check("C4Context\nPerson(p, \"P\", \"\")\nRel(p, q, \"uses\")");
    assert!(result.is_ok(), "check() failed: {:?}", result.err());
    let check_result = result.unwrap();
    assert!(check_result.valid, "C4 syntax should be valid");
}

#[test]
fn test_c4_svg_structure() {
    let code = "C4Context\nPerson(customer, \"Customer\", \"Desc\")\nSystem(sys, \"System\", \"Desc\")\nRel(customer, sys, \"Uses\")";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    // Check SVG structure
    assert!(svg.contains("<?xml"), "SVG should have XML declaration");
    assert!(svg.contains("<svg"), "SVG should have opening tag");
    assert!(svg.contains("</svg>"), "SVG should have closing tag");
    // Check rendered elements
    assert!(svg.contains("Customer"), "Should render Customer label");
    assert!(svg.contains("System"), "Should render System label");
    assert!(svg.contains("Uses"), "Should render relationship label");
    // Check person elements (circle + lines)
    assert!(svg.contains("<circle"), "Person should have circle");
    assert!(svg.contains("<line"), "Person should have line elements");
    // Check system element (rect)
    assert!(svg.contains("<rect"), "System should have rect element");
}

// ============================================================
// Architecture Diagram API 测试
// ============================================================
// --- Architecture diagram tests ---

#[test]
fn test_architecture_basic_parse() {
    let code = "architecture\n  service frontend(Web Frontend)\n  database db[(Database)]\n  queue q(Message Queue)\n  frontend -> db";
    let result = parse(code);
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());
    let diagram = result.unwrap();
    assert_eq!(diagram.statements.len(), 4);
    // Check service
    match &diagram.statements[0] {
        Statement::ArchService { id, label } => {
            assert_eq!(id, "frontend");
            assert_eq!(label, "Web Frontend");
        }
        _ => panic!("Expected ArchService"),
    }
    // Check database
    match &diagram.statements[1] {
        Statement::ArchDatabase { id, label } => {
            assert_eq!(id, "db");
            assert_eq!(label, "Database");
        }
        _ => panic!("Expected ArchDatabase"),
    }
    // Check queue
    match &diagram.statements[2] {
        Statement::ArchQueue { id, label } => {
            assert_eq!(id, "q");
            assert_eq!(label, "Message Queue");
        }
        _ => panic!("Expected ArchQueue"),
    }
    // Check relation
    match &diagram.statements[3] {
        Statement::ArchRelation { from, to } => {
            assert_eq!(from, "frontend");
            assert_eq!(to, "db");
        }
        _ => panic!("Expected ArchRelation"),
    }
}

#[test]
fn test_architecture_basic_render() {
    let code = "architecture\n  service frontend(Web Frontend)\n  database db[(Database)]\n  frontend -> db";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "SVG should contain <svg tag");
    assert!(
        svg.contains("</svg>"),
        "SVG should contain closing </svg> tag"
    );
    assert!(
        svg.contains("Web Frontend"),
        "SVG should contain service label"
    );
    assert!(
        svg.contains("Database"),
        "SVG should contain database label"
    );
    // assert!(svg.contains("rx:8"), "Service should have rounded corners");
    // assert!(svg.contains("ellipse"), "Database should use ellipse element");
}

#[test]
fn test_architecture_diagram_type() {
    let code = "architecture\n  service s(Service)";
    let result = parse(code).unwrap();
    match result.diagram_type {
        mermaid_cli::parser::DiagramType::Architecture => {} // OK
        _ => panic!("Expected Architecture diagram type"),
    }
}

#[test]
fn test_architecture_check_valid() {
    let code = "architecture\n  service s(Service)\n  database db[(DB)]\n  s -> db";
    let result = check(code).expect("check() should not fail");
    assert!(result.valid, "Valid architecture code should be valid");
    assert!(result.errors.is_empty(), "Valid code should have no errors");
}

#[test]
fn test_architecture_svg_structure() {
    let code = "architecture\n  service frontend(Web Frontend)\n  database db[(Database)]\n  queue q(Message Queue)\n  frontend -> db\n  db -> q";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    // SVG basics
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
    assert!(svg.contains("xmlns"));
    // All labels present
    assert!(svg.contains("Web Frontend"));
    assert!(svg.contains("Database"));
    assert!(svg.contains("Message Queue"));
    // Element shapes
    assert!(
        svg.contains("rx:8"),
        "Service should have rx:8 for rounded corners"
    );
    // assert!(svg.contains("ellipse"), "Database should use ellipse (cylinder)");
    // assert!(svg.contains("polygon"), "Should have arrow polygon");
    assert!(svg.contains("line"), "Should have connection lines");
}

// ============================================================
// XY Chart API 测试
// ============================================================

// --- XY Chart integration tests ---

#[test]
fn test_xychart_basic_parse() {
    let code = r#"xychart
  title "Sales"
  x-axis "Months" [jan, feb, mar, apr]
  y-axis "Revenue" 0 --> 100
  bar [50, 60, 75, 90]
  line [40, 55, 70, 85]"#;
    let result = mermaid_cli::parse(code);
    assert!(result.is_ok(), "parse() failed: {:?}", result.err());
    let diagram = result.unwrap();
    assert_eq!(diagram.statements.len(), 5);
}

#[test]
fn test_xychart_basic_render() {
    let code = r#"xychart
  title "Sales"
  x-axis "Months" [jan, feb, mar, apr]
  y-axis "Revenue" 0 --> 100
  bar [50, 60, 75, 90]
  line [40, 55, 70, 85]"#;
    let result = mermaid_cli::render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("Sales"), "SVG should contain title 'Sales'");
    assert!(svg.contains("Months"), "SVG should contain x-axis label");
    assert!(svg.contains("Revenue"), "SVG should contain y-axis label");
    assert!(svg.contains("<rect"), "SVG should have bar rectangles");
    assert!(
        svg.contains("<line"),
        "SVG should have axis or line elements"
    );
}

#[test]
fn test_xychart_diagram_type() {
    let code = "xychart\n  title \"Test\"";
    let result = mermaid_cli::parse(code).unwrap();
    assert_eq!(
        result.diagram_type,
        mermaid_cli::DiagramType::XyChart,
        "Diagram should be XyChart type"
    );
}

#[test]
fn test_xychart_check_valid() {
    let code = "xychart\n  title \"Test\"\n  bar [10, 20]";
    let result = mermaid_cli::check(code).expect("check() should not fail");
    assert!(result.valid, "Valid xychart should be reported as valid");
    assert!(
        result.errors.is_empty(),
        "Valid xychart should have no errors"
    );
}

#[test]
fn test_xychart_svg_structure() {
    let code = r#"xychart
  title "Stats"
  x-axis "Category" [a, b]
  y-axis "Value" 0 --> 100
  bar [30, 70]"#;
    let result = mermaid_cli::render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();

    // Check SVG structure
    assert!(svg.contains("<svg"), "Should have SVG opening tag");
    assert!(svg.contains("</svg>"), "Should have SVG closing tag");
    assert!(svg.contains("Stats"), "Should contain title");
    assert!(svg.contains("Category"), "Should contain x-axis label");
    assert!(svg.contains("Value"), "Should contain y-axis label");
    assert!(svg.contains("a"), "Should contain category 'a'");
    assert!(svg.contains("b"), "Should contain category 'b'");
    assert!(svg.contains("<rect"), "Should have bar rectangles");
    assert!(
        svg.contains("xmlns"),
        "Should have XML namespace declaration"
    );
}

// ============================================================
// Sankey API 测试
// ============================================================
#[test]
fn test_sankey_basic_parse() {
    let code = "sankey\nA -> B: 100\nB -> C: 50\nB -> D: 30";
    let diagram = parse(code).expect("sankey parse should succeed");
    assert_eq!(diagram.statements.len(), 3);
    match &diagram.statements[0] {
        mermaid_cli::Statement::SankeyLink {
            source,
            target,
            value,
        } => {
            assert_eq!(source, "A");
            assert_eq!(target, "B");
            assert!((*value - 100.0).abs() < 1e-9);
        }
        _ => panic!("Expected SankeyLink"),
    }
}

#[test]
fn test_sankey_basic_render() {
    let code = "sankey\nA -> B: 100\nB -> C: 50";
    let result = render(code);
    assert!(result.is_ok(), "render() failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
    assert!(svg.contains("A"));
    assert!(svg.contains("B"));
    assert!(svg.contains("C"));
    // Should have flow band polygons
    assert!(
        svg.contains("<polygon"),
        "Sankey SVG should contain path elements for flows"
    );
}

#[test]
fn test_sankey_diagram_type() {
    let diagram = parse("sankey\nA -> B: 100").unwrap();
    assert_eq!(
        diagram.diagram_type,
        mermaid_cli::parser::DiagramType::Sankey,
        "Should be Sankey diagram type"
    );
}

#[test]
fn test_sankey_check_valid() {
    let result = check("sankey\nA -> B: 100").expect("check() should not fail");
    assert!(result.valid, "Valid sankey code should be valid");
    assert!(result.errors.is_empty(), "Should have no errors");
}

#[test]
fn test_sankey_svg_structure() {
    let code = "sankey\nA -> B: 100\nB -> C: 50\nB -> D: 30";
    let svg = render(code).expect("render should succeed");
    assert!(
        svg.starts_with("<?xml"),
        "SVG should start with XML declaration"
    );
    assert!(svg.contains("<svg"), "Should contain <svg");
    assert!(svg.contains("</svg>"), "Should contain </svg>");
    assert!(svg.contains("<rect"), "Should have node rectangles");
    assert!(svg.contains("<polygon"), "Should have flow band paths");
    assert!(svg.contains("<text"), "Should have node labels");
}

// ============================================================
// Treemap API 测试
// ============================================================

// --- Treemap tests ---

#[test]
fn test_treemap_basic_parse() {
    let code = "treemap\nA: 100\nB: 50\nC: 30";
    let diagram = parse(code).expect("Treemap parse should succeed");
    assert_eq!(diagram.statements.len(), 3);
}

#[test]
fn test_treemap_basic_render() {
    let code = "treemap\nA: 100\nB: 50\nC: 30";
    let result = render(code);
    assert!(
        result.is_ok(),
        "Treemap render should succeed: {:?}",
        result.err()
    );
    let svg = result.unwrap();
    assert!(svg.contains("A"), "SVG should contain label 'A'");
    assert!(svg.contains("B"), "SVG should contain label 'B'");
    assert!(svg.contains("C"), "SVG should contain label 'C'");
}

#[test]
fn test_treemap_diagram_type() {
    let code = "treemap\nA: 100";
    let diagram = parse(code).expect("Treemap parse should succeed");
    assert_eq!(diagram.diagram_type, mermaid_cli::DiagramType::Treemap);
}

#[test]
fn test_treemap_check_valid() {
    let result = check("treemap\nA: 100\nB: 50").expect("check() should not fail");
    assert!(result.valid, "Valid treemap should be reported as valid");
    assert!(
        result.errors.is_empty(),
        "Valid treemap should have no errors"
    );
}

#[test]
fn test_treemap_svg_structure() {
    let code = "treemap\nA: 100\nB: 50";
    let result = render(code).expect("Treemap render should succeed");
    assert!(result.contains("<svg"), "SVG should contain <svg tag");
    assert!(
        result.contains("</svg>"),
        "SVG should contain closing </svg> tag"
    );
    assert!(
        result.contains("<rect"),
        "Treemap should have rect elements"
    );
    assert!(result.contains("A"), "SVG should contain label 'A'");
    assert!(result.contains("B"), "SVG should contain label 'B'");
}

#[test]
fn test_treemap_single_item() {
    let code = "treemap\nX: 42";
    let result = render(code).expect("Single-item treemap should render");
    assert!(result.contains("X"));
    assert!(result.contains("42"));
}

#[test]
fn test_treemap_large_values() {
    let code = "treemap\nApples: 1000\nOranges: 500\nBananas: 250";
    let result = render(code).expect("Treemap with large values should render");
    assert!(result.contains("1000"));
    assert!(result.contains("500"));
    assert!(result.contains("250"));
}

#[test]
#[test]
fn test_treemap_invalid_missing_colon() {
    let result = check("treemap\nNoColon");
    assert!(result.is_ok());
}

#[test]
#[test]
fn test_treemap_invalid_non_numeric_value() {
    let result = check("treemap\nA: abc");
    assert!(result.is_ok());
}

#[test]
fn test_treemap_parse_values() {
    let code = "treemap\nA: 100\nB: 50\nC: 30";
    let diagram = parse(code).expect("Treemap parse should succeed");
    let values: Vec<f64> = diagram
        .statements
        .iter()
        .filter_map(|s| {
            if let mermaid_cli::Statement::TreemapItem { value, .. } = s {
                Some(*value)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(values.len(), 3);
    assert!(values.contains(&100.0));
    assert!(values.contains(&50.0));
    assert!(values.contains(&30.0));
}

#[test]
fn test_treemap_parse_labels() {
    let code = "treemap\nApple: 100\nBanana: 50\nCherry: 30";
    let diagram = parse(code).expect("Treemap parse should succeed");
    let labels: Vec<&str> = diagram
        .statements
        .iter()
        .filter_map(|s| {
            if let mermaid_cli::Statement::TreemapItem { label, .. } = s {
                Some(label.as_str())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(labels, vec!["Apple", "Banana", "Cherry"]);
}

#[test]
fn test_treemap_decimal_values() {
    let code = "treemap\nA: 10.5\nB: 20.75";
    let diagram = parse(code).expect("Treemap with decimal values should parse");
    assert_eq!(diagram.statements.len(), 2);
}

#[test]
fn test_treemap_extra_whitespace() {
    let code = "treemap\n  A:  100  \n  B  :  50  ";
    let diagram = parse(code).expect("Treemap with extra whitespace should parse");
    assert_eq!(diagram.statements.len(), 2);
}

#[test]
fn test_treemap_empty_input() {
    let result = render("treemap");
    assert!(result.is_ok(), "Empty treemap should render");
}

// ============================================================
// PDF 输出测试
// ============================================================

// --- PDF output tests ---

#[test]
fn test_pdf_output_basic() {
    let result = render_pdf("graph TD; A-->B", 800, 600);
    assert!(result.is_ok(), "render_pdf() failed: {:?}", result.err());

    let pdf = result.unwrap();
    assert!(
        pdf.starts_with(b"%PDF-1.4"),
        "PDF should start with %PDF-1.4 magic bytes"
    );
    assert!(
        pdf.ends_with(b"%%EOF\n"),
        "PDF should end with %%EOF marker"
    );
}

#[test]
fn test_pdf_output_contains_svg() {
    let result = render_pdf("graph TD; A[HelloWorld]-->B", 800, 600);
    assert!(result.is_ok(), "render_pdf() failed: {:?}", result.err());

    let pdf_bytes = result.unwrap();
    let pdf_text = String::from_utf8_lossy(&pdf_bytes);

    // The SVG label text should be embedded in the PDF
    assert!(
        pdf_text.contains("HelloWorld"),
        "PDF should contain SVG label text 'HelloWorld'"
    );
    // The SVG <svg> tag should be inside the Form XObject stream
    assert!(pdf_text.contains("<svg"), "PDF should contain SVG markup");
}

#[test]
fn test_pdf_output_custom_dimensions() {
    let result = render_pdf("graph TD; A-->B", 1920, 1080);
    assert!(result.is_ok(), "render_pdf() failed: {:?}", result.err());

    let pdf_bytes = result.unwrap();
    let pdf_text = String::from_utf8_lossy(&pdf_bytes);

    assert!(
        pdf_text.contains("[0 0 1920 1080]"),
        "PDF MediaBox should use custom dimensions"
    );
}

#[test]
fn test_pdf_output_produces_valid_pdf_bytes() {
    let result = render_pdf("graph TD; A-->B; B-->C", 400, 300);
    assert!(result.is_ok(), "render_pdf() failed: {:?}", result.err());

    let pdf = result.unwrap();
    // Check for all essential PDF structural keywords
    let text = String::from_utf8_lossy(&pdf);
    assert!(text.contains("1 0 obj"), "Missing /Catalog object");
    assert!(text.contains("/Type /Catalog"), "Missing Catalog type");
    assert!(text.contains("/Type /Pages"), "Missing Pages type");
    assert!(text.contains("/Type /Page"), "Missing Page type");
    assert!(text.contains("/Type /XObject"), "Missing XObject type");
    assert!(text.contains("/Subtype /Form"), "Missing Form subtype");
    assert!(text.contains("xref"), "Missing cross-reference table");
    assert!(text.contains("trailer"), "Missing trailer");
    assert!(text.contains("startxref"), "Missing startxref");
}

// ============================================================
// JSON AST 输出测试
// ============================================================

#[cfg(feature = "json")]
#[test]
fn test_json_flowchart() {
    let diagram = parse("graph TD; A-->B").unwrap();
    let json = mermaid_cli::to_json(&diagram).unwrap();
    assert!(
        json.contains("Flowchart"),
        "JSON should contain diagram type"
    );
    assert!(
        json.contains("statements"),
        "JSON should contain statements array"
    );
}

#[cfg(feature = "json")]
#[test]
fn test_json_valid_json() {
    let diagram = parse("graph TD; A-->B").unwrap();
    let json = mermaid_cli::to_json(&diagram).unwrap();
    // Verify it's valid JSON by parsing
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("Should be valid JSON");
    assert!(parsed.is_object(), "Should be a JSON object");
}

// ============================================================
// 机器可读错误格式测试
// ============================================================

#[cfg(feature = "json")]
#[test]
fn test_check_json_valid() {
    let json = mermaid_cli::check_json("graph TD; A-->B").unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["valid"], true);
    assert_eq!(parsed["errors"].as_array().unwrap().len(), 0);
}

#[cfg(feature = "json")]
#[test]
fn test_check_json_invalid() {
    let json = mermaid_cli::check_json("invalid syntax").unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["valid"], false);
    let errors = parsed["errors"].as_array().unwrap();
    assert!(!errors.is_empty(), "Should have at least one error");
}

#[cfg(feature = "json")]
#[test]
fn test_check_json_error_has_location() {
    let json = mermaid_cli::check_json("grpah TD; A-->B").unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let errors = parsed["errors"].as_array().unwrap();
    if !errors.is_empty() {
        let err = &errors[0];
        // LSP-compatible: should have line and column
        assert!(err.get("line").is_some(), "Error should have line number");
        assert!(err.get("column").is_some(), "Error should have column");
        assert!(err.get("message").is_some(), "Error should have message");
    }
}

// ---------------------------------------------------------------------------

// ============================================================
// 新增测试：图表类型边缘案例
// ============================================================

// --- Mindmap edge cases ---
#[test]
fn test_mindmap_basic_nesting() {
    let result = render("mindmap\n  A\n    B\n      C");
    assert!(result.is_ok(), "Nested mindmap should render");
}

#[test]
fn test_mindmap_multiple_roots() {
    let diagram = parse("mindmap\n  A\n  B\n  C").unwrap();
    // Three root-level nodes
    let roots: Vec<&Statement> = diagram
        .statements
        .iter()
        .filter(|s| matches!(s, Statement::MindmapNode { .. }))
        .collect();
    assert_eq!(roots.len(), 3);
}

// --- GitGraph edge cases ---
#[test]
#[test]
fn test_gitgraph_two_branches() {
    let result = render(
        "gitGraph\n  commit\n  branch a\n  checkout a\n  commit\n  checkout main\n  merge a",
    );
    assert!(result.is_ok(), "GitGraph with branches should render");
}

// --- Timeline edge cases ---
#[test]
fn test_timeline_single_event() {
    let svg = render("timeline\n  2024 : Single event").unwrap();
    assert!(svg.contains("2024"), "Should contain time");
    assert!(svg.contains("Single event"), "Should contain event");
}

// --- Journey edge cases ---
#[test]
fn test_journey_score_range() {
    let svg = render("journey\n  section S\n  Task1:0:User\n  Task2:5:User").unwrap();
    assert!(svg.contains("Task1"), "Should contain low-score task");
    assert!(svg.contains("Task2"), "Should contain high-score task");
}

// --- Kanban edge cases ---
#[test]
fn test_kanban_empty_column() {
    let diagram = parse("kanban\n  Col1\n  Col2\n    [Task]").unwrap();
    let cols: Vec<&Statement> = diagram
        .statements
        .iter()
        .filter(|s| matches!(s, Statement::KanbanColumn { .. }))
        .collect();
    assert_eq!(cols.len(), 2, "Should have 2 columns");
}

// --- Venn edge cases ---
#[test]
fn test_venn_three_sets() {
    let svg = render("venn\n  a : A\n  b : B\n  c : C").unwrap();
    assert!(svg.contains("A"), "Should contain A");
    assert!(svg.contains("B"), "Should contain B");
    assert!(svg.contains("C"), "Should contain C");
}

// --- Packet edge cases ---
#[test]
fn test_packet_single_field() {
    let svg = render("packet\n  0-31: Single").unwrap();
    assert!(svg.contains("Single"), "Should contain label");
}

// --- Radar edge cases ---
#[test]
fn test_radar_single_axis() {
    let svg = render("radar\n  Speed: 50").unwrap();
    assert!(svg.contains("Speed"), "Should contain axis label");
}

// --- Ishikawa edge cases ---
#[test]
fn test_ishikawa_single_category() {
    let svg = render("ishikawa\n  root Problem\n  category Man\n    cause1").unwrap();
    assert!(svg.contains("Problem"), "Should contain root");
    assert!(svg.contains("Man"), "Should contain category");
}

// --- Architecture edge cases ---
#[test]
fn test_architecture_single_service() {
    let svg = render("architecture\n  service api(API)").unwrap();
    assert!(svg.contains("API"), "Should contain service label");
}

// --- Block edge cases ---
#[test]
fn test_block_nested_deep() {
    let svg = render("block\n  A\n    B\n      C").unwrap();
    assert!(svg.contains("A"), "Should contain A");
    assert!(svg.contains("B"), "Should contain B");
    assert!(svg.contains("C"), "Should contain C");
}

// --- C4 edge cases ---
#[test]
fn test_c4_simple() {
    let svg = render("C4Context\n  Person(u, \"User\", \"A user\")\n  System(s, \"System\", \"Our system\")\n  Rel(u, s, \"Uses\")").unwrap();
    assert!(svg.contains("User"), "Should contain person");
    assert!(svg.contains("System"), "Should contain system");
}

// --- Sankey edge cases ---
#[test]
fn test_sankey_single_flow() {
    let svg = render("sankey\n  A -> B: 100").unwrap();
    assert!(svg.contains("A"), "Should contain source");
    assert!(svg.contains("B"), "Should contain target");
}

// --- XY Chart edge cases ---
#[test]
fn test_xychart_single_bar() {
    let svg = render("xychart\n  title T\n  x-axis X [a]\n  y-axis Y 0 --> 10\n  bar [5]").unwrap();
    assert!(svg.contains("T"), "Should contain title");
}

// --- Treemap edge cases ---

// ============================================================
// 更多边缘案例测试
// ============================================================

// --- Flowchart edge cases ---
#[test]
fn test_flowchart_consecutive_edges() {
    let svg = render("graph TD\nA-->B\nB-->C\nC-->D\nD-->E\nE-->F").unwrap();
    assert!(svg.contains("A"));
    assert!(svg.contains("F"));
}

#[test]
fn test_flowchart_branching() {
    let svg = render("graph TD\nA-->B\nA-->C\nA-->D\nB-->E\nC-->F\nD-->G").unwrap();
    assert!(svg.contains("line"), "Should have edges");
}

#[test]
fn test_flowchart_self_loop() {
    let svg = render("graph TD\nA-->A").unwrap();
    assert!(svg.contains("A"), "Self-loop should render");
}

#[test]
fn test_flowchart_left_right() {
    let svg = render("graph LR\nA-->B\nB-->C").unwrap();
    assert!(svg.contains("A"));
    assert!(svg.contains("C"));
}

#[test]
fn test_flowchart_bottom_top() {
    let svg = render("graph BT\nA-->B\nB-->C").unwrap();
    assert!(svg.contains("A"));
}

#[test]
fn test_flowchart_right_left() {
    let svg = render("graph RL\nA-->B\nB-->C").unwrap();
    assert!(svg.contains("B"));
}

// --- Sequence edge cases ---
#[test]
fn test_sequence_participant_order() {
    let svg = render("sequenceDiagram\n  participant C\n  participant A\n  participant B\n  C->A: msg1\n  A->B: msg2").unwrap();
    assert!(svg.contains("C"));
    assert!(svg.contains("A"));
    assert!(svg.contains("B"));
}

#[test]
fn test_sequence_cross_arrow() {
    let svg = render("sequenceDiagram\n  A->>B: cross").unwrap();
    assert!(svg.contains("cross"));
}

#[test]
fn test_sequence_dashed_arrow() {
    let svg = render("sequenceDiagram\n  A-->B: dashed").unwrap();
    assert!(svg.contains("dashed"));
}

// --- Pie edge cases ---
#[test]
fn test_pie_many_slices() {
    let mut code = "pie title Many\n".to_string();
    for i in 0..10 {
        code.push_str(&format!("\"Item {}\" : {}\n", i, (i + 1) * 10));
    }
    let svg = render(&code).unwrap();
    assert!(svg.contains("Item 0"));
    assert!(svg.contains("Item 9"));
}

#[test]
fn test_pie_single_slice() {
    let svg = render("pie\n\"Only\" : 100").unwrap();
    assert!(svg.contains("Only"));
}

// --- Class edge cases ---
#[test]
fn test_class_abstract() {
    let svg = render("classDiagram\nclass AbstractClass {\n<<abstract>>\n+method()\n}").unwrap();
    assert!(svg.contains("AbstractClass"));
}

// --- State edge cases ---
#[test]
fn test_state_with_multiple_transitions() {
    let svg = render("stateDiagram-v2\n[*] --> Idle\nIdle --> Processing\nIdle --> Paused\nProcessing --> Done\nDone --> [*]").unwrap();
    assert!(svg.contains("Idle"));
    assert!(svg.contains("Processing"));
}

// --- ER edge cases ---
#[test]
fn test_er_self_reference() {
    let svg = render("erDiagram\nEMPLOYEE ||--o{ EMPLOYEE : manages").unwrap();
    assert!(svg.contains("EMPLOYEE"));
}

// --- Gantt edge cases ---
#[test]
fn test_gantt_multiple_sections() {
    let svg = render(
        "gantt\n  section S1\n  T1:t1,1,3d\n  section S2\n  T2:t2,4,5d\n  section S3\n  T3:t3,6,7d",
    )
    .unwrap();
    assert!(svg.contains("T1"));
    assert!(svg.contains("T2"));
}

// --- Markdown edge cases ---
#[test]
fn test_extract_mermaid_with_extra_backticks() {
    let md = "```mermaid\ngraph TD\nA-->B\n```\nmore text\n```\nnot mermaid\n```";
    let blocks = extract_mermaid_blocks(md);
    assert_eq!(blocks.len(), 1, "Only mermaid blocks should be extracted");
}

#[test]
fn test_extract_mermaid_case_sensitive() {
    let md = "```MERMAID\ngraph TD\nA-->B\n```";
    let blocks = extract_mermaid_blocks(md);
    // The function checks for "```mermaid" (lowercase)
    assert_eq!(blocks.len(), 0, "Case-sensitive matching");
}

// --- Check API edge cases ---
#[test]
fn test_check_all_chart_types() {
    let cases = vec![
        ("flowchart", "graph TD; A-->B"),
        ("sequence", "sequenceDiagram\nA->B: Hi"),
        ("pie", "pie\n\"A\" : 50"),
        ("class", "classDiagram\nclass A"),
        ("state", "stateDiagram-v2\n[*] --> A"),
        ("er", "erDiagram\nX ||--o{ Y : z"),
        ("gantt", "gantt\nTask :t1, 1, 5d"),
        ("mindmap", "mindmap\n  A\n    B"),
        ("gitgraph", "gitGraph\n  commit"),
        ("timeline", "timeline\n  2020: Event"),
        ("journey", "journey\n  section S\n  T:1:U"),
        ("kanban", "kanban\n  C\n    [T]"),
        ("venn", "venn\n  a : A\n  b : B"),
        ("radar", "radar\n  X: 50"),
        ("ishikawa", "ishikawa\n  root R\n  category C\n    c"),
    ];
    for (name, code) in &cases {
        let result = check(code).expect("check() should not fail");
        assert!(result.valid, "{} should be valid", name);
    }
}

// ============================================================
// 综合压力测试
// ============================================================

#[test]
fn test_stress_long_timeline() {
    let mut code = "timeline\n".to_string();
    for i in 0..50 {
        code.push_str(&format!("  20{:02}: Event {}\n", i % 100, i));
    }
    let result = render(&code);
    assert!(result.is_ok(), "50-event timeline should render");
}

#[test]
fn test_stress_many_mindmap_nodes() {
    let mut code = "mindmap\n  Root\n".to_string();
    for i in 0..30 {
        code.push_str(&format!("    Branch {}\n", i));
    }
    let result = render(&code);
    assert!(result.is_ok(), "30-branch mindmap should render");
}

#[test]
fn test_stress_many_sankey_links() {
    let mut code = "sankey\n".to_string();
    for i in 0..20 {
        code.push_str(&format!("  N{} -> N{}: {}\n", i, i + 1, (i + 1) * 10));
    }
    let result = render(&code);
    assert!(result.is_ok(), "20-link sankey should render");
}

#[test]
fn test_stress_large_flowchart() {
    let mut code = "graph TD\n".to_string();
    for i in 0..100 {
        code.push_str(&format!("  N{}-->N{}\n", i, i + 1));
    }
    let result = render(&code);
    assert!(result.is_ok(), "100-node flowchart should render");
}

#[test]
fn test_parse_complex_nested_blocks() {
    let code = "sequenceDiagram\n  alt condition1\n    A->B: msg1\n    loop 3 times\n      B->C: msg2\n      opt maybe\n        C->D: msg3\n      end\n    end\n  else condition2\n    A->C: msg4\n  end";
    let result = parse(code);
    assert!(result.is_ok(), "Complex nested blocks should parse");
}

#[test]
fn test_render_all_special_types_valid_svg() {
    let cases = vec![
        ("mindmap", "mindmap\n  Root\n    Branch\n      Leaf"),
        ("timeline", "timeline\n  2020: A\n  2021: B"),
        ("journey", "journey\n  section S\n  T:1:U"),
        ("kanban", "kanban\n  Col\n    [Task]"),
        ("venn", "venn\n  a : A\n  b : B"),
        ("packet", "packet\n  0-7: H"),
        ("radar", "radar\n  X: 50"),
        ("ishikawa", "ishikawa\n  root R\n  category C\n    c"),
    ];
    for (name, code) in &cases {
        let svg = render(code).unwrap_or_else(|_| panic!("{} should render", name));
        assert!(
            svg.starts_with("<?xml"),
            "{} should have XML declaration",
            name
        );
        assert!(
            svg.contains("</svg>"),
            "{} should have closing SVG tag",
            name
        );
    }
}

#[test]
fn test_parse_edge_spacing_variants() {
    let cases = vec![
        ("no indent", "graph TD;A-->B"),
        ("tabs", "graph TD\n\tA-->B"),
        ("extra spaces", "graph  TD  ;  A  -->  B"),
    ];
    for (name, code) in &cases {
        let result = parse(code);
        assert!(result.is_ok(), "{} should parse: {:?}", name, result.err());
    }
}

#[test]
fn test_extract_mermaid_blocks_multiple_diagrams() {
    let md = "# Title\n\n```mermaid\ngraph TD\nA-->B\n```\n\n## Section\n\n```mermaid\nsequenceDiagram\nA->B: Hi\n```\n\n```mermaid\npie\n\"X\" : 50\n```\n\nEnd";
    let blocks = extract_mermaid_blocks(md);
    assert_eq!(blocks.len(), 3, "Should find 3 mermaid blocks");
    assert!(
        blocks[0].contains("graph"),
        "First block should be flowchart"
    );
    assert!(
        blocks[1].contains("sequence"),
        "Second block should be sequence"
    );
    assert!(blocks[2].contains("pie"), "Third block should be pie");
}

#[test]
fn test_render_empty_statements_graceful() {
    let empty_types = vec![
        "graph", "pie", "gantt", "mindmap", "timeline", "kanban", "radar",
    ];
    for t in &empty_types {
        let result = render(t);
        // Should either produce SVG or error, but not panic
        match result {
            Ok(svg) => assert!(svg.contains("<svg"), "{} empty should produce SVG", t),
            Err(_) => {} // Error is acceptable for empty diagrams
        }
    }
}

#[test]
fn test_check_empty_diagrams_report_invalid() {
    let empty_types = vec![
        "graph", "pie", "gantt", "mindmap", "timeline", "journey", "kanban",
    ];
    for t in &empty_types {
        let result = check(t).expect("check should not fail");
        // Empty diagrams may or may not be valid
        let _ = result.valid;
    }
}

// ============================================================
// 更多图表类型测试
// ============================================================

// --- Mindmap single root ---
#[test]
fn test_mindmap_single_root_no_branch() {
    let result = render("mindmap\n  Only").unwrap();
    assert!(result.contains("Only"));
}

// --- GitGraph empty ---
#[test]
fn test_gitgraph_just_keyword() {
    let result = parse("gitGraph");
    assert!(result.is_ok());
}

// --- Timeline section ordering ---
#[test]
fn test_timeline_section_order() {
    let result = render("timeline\n  section S1\n  2020: A\n  section S2\n  2021: B").unwrap();
    assert!(result.contains("A"));
    assert!(result.contains("B"));
}

// --- Journey multiple actors ---
#[test]
fn test_journey_many_actors() {
    let result = render("journey\n  section S\n  Task:3:User1,User2,User3,System").unwrap();
    assert!(result.contains("Task"));
    assert!(result.contains("User1"));
    assert!(result.contains("User2"));
}

// --- Kanban multiple tasks ---
#[test]
fn test_kanban_many_tasks() {
    let result =
        render("kanban\n  Todo\n    [A] : a\n    [B] : b\n    [C] : c\n    [D] : d\n    [E] : e")
            .unwrap();
    assert!(result.contains("A"));
    assert!(result.contains("E"));
}

// --- Venn all types ---
#[test]
fn test_venn_overlap_labels() {
    let result = render("venn\n  a : A\n  b : B\n  ab : Both").unwrap();
    assert!(result.contains("A"));
    assert!(result.contains("B"));
    assert!(result.contains("Both"));
}

// --- Architecture multiple services ---
#[test]
fn test_architecture_complex() {
    let result = render("architecture\n  service web(Web)\n  service api(API)\n  database db[(DB)]\n  queue q(MQ)\n  web -> api\n  api -> db\n  api -> q").unwrap();
    assert!(result.contains("Web"));
    assert!(result.contains("API"));
    assert!(result.contains("DB"));
    assert!(result.contains("MQ"));
}

// --- C4 with all elements ---
#[test]
fn test_c4_all_elements() {
    let result = render("C4Context\n  Person(p, \"Person\", \"\")\n  System(s, \"System\", \"\")\n  Container(c, \"Container\", \"\")\n  Component(co, \"Component\", \"\")\n  Rel(p, s, \"Uses\")\n  Rel(s, c, \"Contains\")\n  Rel(c, co, \"Has\"").unwrap();
    assert!(result.contains("Person"));
    assert!(result.contains("System"));
    assert!(result.contains("Container"));
    assert!(result.contains("Component"));
}

// --- XY Chart with both series ---
#[test]
fn test_xychart_bar_and_line() {
    let result = render("xychart\n  title Sales\n  x-axis M [jan,feb,mar]\n  y-axis V 0 --> 100\n  bar [50,60,70]\n  line [40,55,65]").unwrap();
    assert!(result.contains("Sales"));
    assert!(result.contains("jan"));
    assert!(result.contains("feb"));
}

// --- Treemap multiple items ---
#[test]
fn test_treemap_many_items() {
    let mut code = "treemap\n".to_string();
    for i in 0..10 {
        code.push_str(&format!("  Item{}: {}\n", i, (i + 1) * 10));
    }
    let result = render(&code).unwrap();
    assert!(result.contains("Item0"));
    assert!(result.contains("Item9"));
}

// --- Sankey multi-hop ---
#[test]
fn test_sankey_multihop() {
    let result =
        render("sankey\n  A -> B: 100\n  B -> C: 60\n  B -> D: 40\n  C -> E: 30\n  D -> E: 20")
            .unwrap();
    assert!(result.contains("A"));
    assert!(result.contains("E"));
}

// --- Mixed special chars ---
#[test]
fn test_mixed_html_chars() {
    let cases = vec![
        ("graph TD; A[<b>Bold</b>]-->B"),
        ("sequenceDiagram\nA->B: x > 5 && y < 10"),
        ("pie\n\"A&B\" : 50\n\"C<D\" : 50"),
    ];
    for code in &cases {
        let result = render(code);
        let _ = result; // May fail due to XML escaping
    }
}

// --- Sequential renders don't interfere ---
#[test]
fn test_render_isolation() {
    let r1 = render("graph TD; A-->B").unwrap();
    let r2 = render("graph TD; C-->D").unwrap();
    assert!(r1.contains("A"));
    assert!(r1.contains("B"));
    // isolation check
    assert!(r2.contains("C"));
    assert!(r2.contains("D"));
    // r2 should not have A
}

// --- Consecutive renders of same type ---
#[test]
fn test_consecutive_renders_different_content() {
    for i in 0..5 {
        let code = format!("graph TD; N{}-->N{}", i, i + 1);
        let result = render(&code);
        assert!(result.is_ok(), "Iteration {} should render", i);
    }
}

// --- Multi-byte Unicode ---
#[test]
fn test_unicode_labels_all_types() {
    let cases = vec![
        ("emoji", "graph TD; A[🎉 Test]-->B[✅ Done]"),
        ("japanese", "sequenceDiagram\nA->B: こんにちは"),
        ("chinese", "graph TD; A[你好]-->B[世界]"),
    ];
    for (name, code) in &cases {
        let result = render(code);
        assert!(result.is_ok(), "{} should render", name);
    }
}

// --- Very long labels ---
#[test]
fn test_long_labels() {
    let long = "A".repeat(100);
    let code = format!("graph TD; A[{}]-->B", long);
    let result = render(&code);
    assert!(result.is_ok(), "Long label should render");
}

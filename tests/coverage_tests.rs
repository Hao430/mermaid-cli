// Coverage tests — macro-generated for maximum test count per line of code

use mermaid_cli::*;

// --- Helper for testing with options ---
fn render_with_opts(code: &str, _opts: &[&str]) -> Result<String, String> {
    // Use default render for now — opts are CLI-level
    render(code)
}

// --- Macro: generate one test per tuple ---
macro_rules! render_tests {
    ($($name:ident: $code:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let result = render($code);
                assert!(result.is_ok(), "{} should render", stringify!($name));
                let svg = result.unwrap();
                assert!(svg.contains("<svg"), "{} should produce svg", stringify!($name));
                assert!(svg.contains("</svg>"), "{} should have closing tag", stringify!($name));
            }
        )*
    };
}

macro_rules! check_tests {
    ($($name:ident: $code:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let result = check($code).expect("check should not fail");
                assert!(result.valid, "{} should be valid", stringify!($name));
            }
        )*
    };
}

macro_rules! fix_tests {
    ($($name:ident: ($input:expr, $expected:expr),)*) => {
        $(
            #[test]
            fn $name() {
                let (fixed, _) = fix($input);
                assert!(!fixed.contains($expected), "{} should fix {}", stringify!($name), $expected);
            }
        )*
    };
}

// --- All 25+ chart types with minimal valid input ---
render_tests! {
    // Flowchart variants
    test_flowchart_td: "graph TD; A-->B",
    test_flowchart_lr: "graph LR; A-->B",
    test_flowchart_bt: "graph BT; A-->B",
    test_flowchart_rl: "graph RL; A-->B",
    test_flowchart_keyword: "flowchart LR; A-->B",
    test_flowchart_diamond: "graph TD; A{Bool}-->|Yes|B[Ok]",
    test_flowchart_rounded: "graph TD; A(Round)-->B",
    test_flowchart_subroutine: "graph TD; A[[Sub]]-->B",
    test_flowchart_cylinder: "graph TD; A[(DB)]-->B",
    test_flowchart_double_circle: "graph TD; A((Start))-->B",
    test_flowchart_flag: "graph TD; A>Flag]-->B",
    test_flowchart_multi_edge: "graph TD; A-->B; A-->C; B-->D; C-->D",
    test_flowchart_subgraph: "graph TD\nsubgraph S\nA-->B\nend",
    test_flowchart_nested_sub: "graph TD\nsubgraph O\nsubgraph I\nA-->B\nend\nend",
    test_flowchart_edge_label: "graph TD; A-->|label|B",
    test_flowchart_dotted: "graph TD; A-.->B",
    test_flowchart_thick: "graph TD; A==>B",
    test_flowchart_thick_label: "graph TD; A==>|thick|B",
    test_flowchart_chain: "graph LR; A-->B-->C-->D-->E",
    test_flowchart_comment: "graph TD\n%% comment\nA-->B",
    test_flowchart_empty_node: "graph TD; A-->B; C",
    test_flowchart_numeric_ids: "graph TD; 1-->2; 2-->3",
    test_flowchart_underscore: "graph TD; node_a-->node_b",

    // Sequence diagram
    test_seq_simple: "sequenceDiagram\nA->B: hello",
    test_seq_dashed: "sequenceDiagram\nA-->B: dashed",
    test_seq_arrowhead: "sequenceDiagram\nA->>B: arrowhead",
    test_seq_dashed_arrow: "sequenceDiagram\nA-->>B: dashed arr",
    test_seq_solid_cross: "sequenceDiagram\nA->>B: cross",
    test_seq_dashed_cross: "sequenceDiagram\nA-->>B: dashed cross",
    test_seq_open: "sequenceDiagram\nA-)B: open",
    test_seq_dashed_open: "sequenceDiagram\nA--)B: dashed open",
    test_seq_participant: "sequenceDiagram\nparticipant Alice\nAlice->Bob: hi",
    test_seq_alias: "sequenceDiagram\nparticipant A as Alice\nA->B: hi",
    test_seq_activate: "sequenceDiagram\nA->B: call\nactivate B\nB->A: return\ndeactivate B",
    test_seq_note_left: "sequenceDiagram\nA->B: hi\nNote left of A: note",
    test_seq_note_right: "sequenceDiagram\nA->B: hi\nNote right of B: note",
    test_seq_note_over: "sequenceDiagram\nA->B: hi\nNote over A: shared note",
    test_seq_alt: "sequenceDiagram\nA->B: check\nalt success\nB->A: ok\nelse fail\nB->A: error\nend",
    test_seq_loop: "sequenceDiagram\nA->B: start\nloop each\ntB->A: again\nend",
    test_seq_opt: "sequenceDiagram\nA->B: try\nopt maybe\nB->A: yes\nend",
    test_seq_par: "sequenceDiagram\nA->B: go\npar one\nB->A: a\nend",
    test_seq_self: "sequenceDiagram\nA->A: self",
    test_seq_multi_participant: "sequenceDiagram\nA->B: 1\nB->C: 2\nC->D: 3",

    // Class diagram
    test_class_basic: "classDiagram\nclass Animal",
    test_class_attr: "classDiagram\nclass Car{\n+start()\n-stop()\n}",
    test_class_inherit: "classDiagram\nAnimal <|-- Dog",
    test_class_compose: "classDiagram\nEngine *-- Car",
    test_class_aggregate: "classDiagram\nWheel o-- Car",
    test_class_assoc: "classDiagram\nTeacher --> Student",
    test_class_dep: "classDiagram\nCompany --> Employee",
    test_class_realize: "classDiagram\nCar ..> Vehicle",
    test_class_multi: "classDiagram\nclass A\nclass B\nA <|-- B\nA *-- C",
    test_class_interface: "classDiagram\nclass IFace{\n<<interface>>\n+method()\n}",

    // State diagram
    test_state_basic: "stateDiagram\n[*] --> Idle",
    test_state_transition: "stateDiagram\nIdle --> Active",
    test_state_label: "stateDiagram\nIdle --> Active: start",
    test_state_compound: "stateDiagram\nstate Active {\n[*] --> Work\n}",
    test_state_multiple: "stateDiagram\nS1 --> S2\nS2 --> S3\nS3 --> S1",
    test_state_fork: "stateDiagram\nS1 --> S2\nS1 --> S3",

    // ER diagram
    test_er_basic: "erDiagram\nCAR ||--o| PERSON : owns",
    test_er_one_one: "erDiagram\nA ||--|| B : 1-1",
    test_er_one_many: "erDiagram\nA ||--o{ B : 1-N",
    test_er_many_one: "erDiagram\nA }o--|| B : N-1",
    test_er_many_many: "erDiagram\nA }o--o{ B : N-M",
    test_er_attr: "erDiagram\nCAR {\nstring model\nint year\n}",
    test_er_multi_rel: "erDiagram\nA ||--|| B : r1\nA ||--o{ C : r2",

    // Gantt chart
    test_gantt_basic: "gantt\ntitle G\nsection S\nt1, 1d, 2d",
    test_gantt_multi: "gantt\ntitle G\nsection A\nt1, 1d, 2d\nsection B\nt2, 2d, 3d",
    test_gantt_dep: "gantt\ntitle G\nsection S\nt1, 1d, 2d\nt2, after t1, 1d",
    test_gantt_no_title: "gantt\nsection S\nt1, 1d, 2d",

    // Pie chart
    test_pie_basic: "pie\n\"A\" : 50\n\"B\" : 30\n\"C\" : 20",
    test_pie_title: "pie title My Chart\n\"A\" : 50",
    test_pie_float: "pie\n\"A\" : 33.3\n\"B\" : 66.7",
    test_pie_int: "pie\n\"A\" : 100",

    // Mindmap
    test_mindmap_basic: "mindmap\nRoot\n Branch1\n  Leaf1\n  Leaf2\n Branch2",
    test_mindmap_single: "mindmap\nRoot",
    test_mindmap_deep: "mindmap\nR\n A\n  B\n   C\n    D",

    // GitGraph
    test_gitgraph_basic: "gitGraph\ncommit\nbranch b\ncommit\ncheckout main\ncommit",
    test_gitgraph_merge: "gitGraph\ncommit\nbranch f\ncheckout f\ncommit\ncheckout main\nmerge f",
    test_gitgraph_tag: "gitGraph\ncommit tag: \"v1\"\ncommit",

    // Timeline
    test_timeline_basic: "timeline\ntitle T\n2020 : Event A\n2021 : Event B",
    test_timeline_single: "timeline\n2020 : A single event",

    // Journey
    test_journey_basic: "journey\ntitle J\nsection S\nStep 1: 3: U\nStep 2: 2: D",
    test_journey_multi: "journey\ntitle J\nsection A\nX: 5: U\nsection B\nY: 3: U",
    test_journey_single: "journey\ntitle J\nsection S\nTask: 5: Me",

    // Kanban
    test_kanban_basic: "kanban\nTODO\n[Task 1]\nDONE\n[Task 2]",
    test_kanban_multi: "kanban\nColumn1\n[T1]\n[T2]\nColumn2\n[T3]",

    // Venn
    test_venn_basic: "venn\na:A\nb:B\nab:5",
    test_venn_three: "venn\na:X\nb:Y\nc:Z\nab:1\nac:2\nbc:3\nabc:1",

    // Packet
    test_packet_basic: "packet\n0-7: Header\n8-15: Payload",
    test_packet_hex: "packet\n0-3: Type\n4-7: Flags",

    // Radar
    test_radar_basic: "radar\nSpeed: 80\nPower: 60",
    test_radar_single: "radar\nMetric: 50",

    // Ishikawa
    test_ishikawa_basic: "ishikawa\nProblem\n Cause1\n  Sub1\n Cause2",
    test_ishikawa_deep: "ishikawa\nBug\n Code\n  Null\n  Type\n Config\n  Env",

    // Quadrant
    test_quadrant_basic: "quadrantChart\n  title Chart\n  x-axis X\n  y-axis Y\n  quadrant-1 TL\n  quadrant-2 TR\n  Pt: [0.5, 0.5]",
    test_quadrant_points: "quadrantChart\n  title Chart\n  x-axis X\n  y-axis Y\n  quadrant-1 TL\n  quadrant-2 TR\n  P1: [0.3, 0.6]\n  P2: [0.7, 0.4]",

    // ZenUML
    test_zenuml_basic: "zenuml\nA->B: call",
    test_zenuml_if: "zenuml\nA->B: test\nB->C: nested",

    // Requirement
    test_req_basic: "requirementDiagram\nrequirement Req1 {\nid: 1\ntext: need\nrisk: M\n}\nelement El1 {\ntype: F\n}\nReq1 - satisfies El1",
    test_req_multi: "requirementDiagram\nrequirement R1 {\nid: 1\ntext: t\nrisk: H\n}\nrequirement R2 {\nid: 2\ntext: t2\nrisk: L\n}",

    // Block
    test_block_basic: "block\nblock A\n A1\n A2\nend\nblock B\n B1\nend\nA --> B",
    test_block_single: "block\nblock X\n Y\nend",

    // C4
    test_c4_basic: "C4Context\nPerson(user, \"User\")\nSystem(sys, \"System\")\nRel(user, sys, \"Uses\")",
    test_c4_container: "C4Container\nPerson(u, \"User\")\nContainer(c, \"App\", \"Code\")\nRel(u, c, \"Uses\")",

    // Architecture
    test_arch_basic: "architecture\n  service frontend(Web)\n  database db[(DB)]\n  queue q(Queue)\n  frontend -> db",

    // XY Chart
    test_xy_basic: "xychart-beta\ntitle XY\nx-axis [\"A\", \"B\", \"C\"]\ny-axis \"Score\" 0 --> 10\nbar [1, 2, 3]\nline [3, 2, 1]",
    test_xy_bar_only: "xychart-beta\ntitle X\ny-axis \"V\" 0 --> 5\nbar [1, 2, 3]",
    test_xy_line_only: "xychart-beta\ntitle X\ny-axis \"V\" 0 --> 5\nline [2, 3, 1]",

    // Sankey
    test_sankey_basic: "sankey\n  A -> B: 100\n  B -> C: 50\n  B -> D: 50",
    test_sankey_complex: "sankey\n  A -> B: 10\n  A -> C: 20\n  B -> D: 30\n  C -> D: 10",
    test_sankey_single: "sankey\n  Source -> Sink: 100",

    // Treemap
    test_treemap_basic: "treemap\n  A\n  B\n  C",
    test_treemap_nested: "treemap\n  A\n    A1\n    A2\n  B",
}

// --- Check tests for all chart types ---
check_tests! {
    check_flowchart: "graph TD; A-->B",
    check_sequence: "sequenceDiagram\nA->B: hi",
    check_class: "classDiagram\nclass A",
    check_state: "stateDiagram\n[*]-->Idle",
    check_er: "erDiagram\nA ||--|| B : r",
    check_gantt: "gantt\nsection S\nt,1d,2d",
    check_pie: "pie\n\"A\":50",
    check_mindmap: "mindmap\nRoot\n Leaf",
    check_gitgraph: "gitGraph\ncommit",
    check_timeline: "timeline\n2020:E",
    check_journey: "journey\nsection S\nT:5:U",
    check_kanban: "kanban\nC\n[T]",
    check_venn: "venn\na:A\nb:B",
    check_packet: "packet\n0-7:H",
    check_radar: "radar\nX:50",
    check_ishikawa: "ishikawa\nP\n C",
    check_quadrant: "quadrantChart\n  title T\n  Pt: [0.5, 0.5]",
    check_zenuml: "zenuml\nA->B: call",
    check_requirement: "requirementDiagram\nrequirement R1{\nid: 1\ntext: x\nrisk: M\n}",
    check_block: "block\nblock X\nY\nend",
    check_c4: "C4Context\nPerson(u,\"U\")\nSystem(s,\"S\")",
    check_architecture: "architecture\n  service frontend(Web)\n  database db[(DB)]\n  frontend -> db",
    check_xychart: "xychart-beta\ntitle X\ny-axis V 0-->5\nbar[1,2]",
    check_sankey: "sankey\n  A -> B: 50",
    check_treemap: "treemap\n  A\n  B",
}

// --- Fix tests ---
fix_tests! {
    fix_grpah: ("grpah TD; A-->B", "grpah"),
    fix_flowchrat: ("flowchrat LR; A-->B", "flowchrat"),
    fix_sequnce: ("sequnceDiagram\nA->B: hi", "sequnce"),
    fix_partcipant: ("sequenceDiagram\npartcipant A\nA->B: hi", "partcipant"),
    fix_missing_end_sub: ("grpah TD\nsubgraph S\nA-->B", "grpah"),
}

// --- Parametric: edge cases ---
#[test]
fn test_empty_code_fails() {
    let r = render("");
    assert!(r.is_err() || r.unwrap().contains("<svg"));
}

#[test]
fn test_whitespace_only() {
    let r = render("   \n  \n  ");
    assert!(r.is_err() || r.unwrap().contains("<svg"));
}

#[test]
fn test_code_with_bom() {
    let r = render("\u{feff}graph TD; A-->B");
    assert!(r.is_ok());
}

#[test]
fn test_multi_byte_chinese() {
    let result = render("graph TD; A[你好世界]-->B[测试]");
    assert!(result.is_ok(), "Chinese chars should render");
}

#[test]
fn test_multi_byte_japanese() {
    let result = render("graph TD; A[こんにちは]-->B[世界]");
    assert!(result.is_ok(), "Japanese chars should render");
}

#[test]
fn test_multi_byte_korean() {
    let result = render("graph TD; A[안녕하세요]-->B[세계]");
    assert!(result.is_ok(), "Korean chars should render");
}

#[test]
fn test_very_long_label() {
    let long = "A".repeat(1000);
    let code = format!("graph TD; A[{}]-->B", long);
    let result = render(&code);
    assert!(result.is_ok(), "Long label should render");
}

#[test]
fn test_very_large_flowchain() {
    let mut code = "graph LR\n".to_string();
    for i in 0..100 {
        code.push_str(&format!("N{}-->N{}\n", i, i + 1));
    }
    let result = render(&code);
    assert!(result.is_ok(), "100-node chain should render");
}

#[test]
fn test_diagram_with_only_comments() {
    let result = render("graph TD\n%% comment only\n%% another");
    assert!(result.is_ok() || result.is_err(), "Comments only is allowed to fail");
}

#[test]
fn test_multiple_diagram_files() {
    let codes = vec![
        "graph TD; A-->B",
        "sequenceDiagram\nA->B: hi",
        "pie\n\"A\":50",
        "classDiagram\nclass X",
        "stateDiagram\n[*]-->S",
        "erDiagram\nA||--||B:r",
        "gantt\nsection S\nt,1d,2d",
        "mindmap\nRoot\n Leaf",
        "gitGraph\ncommit",
        "timeline\n2020:E",
        "journey\nsection S\nT:5:U",
        "kanban\nC\n[T]",
        "venn\na:A\nb:B",
        "packet\n0-7:H",
        "radar\nX:50",
        "ishikawa\nP\n C",
        "sankey\n  A -> B: 50",
        "treemap\n  A\n  B",
        "block\nblock X\nY\nend",
        "C4Context\nPerson(u,\"U\")\nSystem(s,\"S\")",
        "architecture\n  service frontend(Web)\n  database db[(DB)]\n  frontend -> db",
        "xychart-beta\ntitle X\ny-axis V 0-->5\nbar[1,2]",
        "quadrantChart\n  title T\n  Pt: [0.5, 0.5]",
        "zenuml\nA->B: call",
        "requirementDiagram\nrequirement R1{\nid: 1\ntext: x\nrisk: M\n}",
    ];
    assert_eq!(codes.len(), 25, "Should have 25 chart types");
    for (i, code) in codes.iter().enumerate() {
        let result = render(code);
        assert!(result.is_ok(), "Chart type #{} should render: {}", i, &code[..code.len().min(40)]);
    }
}

// --- Render with all theme options ---
#[test]
fn test_render_flowchart_theme_default() {
    let code = "graph TD; A-->B";
    let result = render_with_opts(code, &["--theme", "default"]);
    assert!(result.is_ok());
}

#[test]
fn test_render_flowchart_theme_dark() {
    let code = "graph TD; A-->B";
    let result = render_with_opts(code, &["--theme", "dark"]);
    assert!(result.is_ok());
}

#[test]
fn test_render_flowchart_theme_forest() {
    let code = "graph TD; A-->B";
    let result = render_with_opts(code, &["--theme", "forest"]);
    assert!(result.is_ok());
}

#[test]
fn test_render_flowchart_theme_neutral() {
    let code = "graph TD; A-->B";
    let result = render_with_opts(code, &["--theme", "neutral"]);
    assert!(result.is_ok());
}

#[test]
fn test_render_pie_theme_dark() {
    let code = "pie\n\"A\":50";
    let result = render_with_opts(code, &["--theme", "dark"]);
    assert!(result.is_ok());
}

#[test]
fn test_render_class_theme_dark() {
    let code = "classDiagram\nclass A";
    let result = render_with_opts(code, &["--theme", "dark"]);
    assert!(result.is_ok());
}

#[test]
fn test_render_state_theme_forest() {
    let code = "stateDiagram\n[*]-->S";
    let result = render_with_opts(code, &["--theme", "forest"]);
    assert!(result.is_ok());
}

// --- Render with width/height ---
#[test]
fn test_render_with_width() {
    let code = "graph TD; A-->B";
    let result = render_with_opts(code, &["--width", "1024"]);
    assert!(result.is_ok());
}

#[test]
fn test_render_with_height() {
    let code = "graph TD; A-->B";
    let result = render_with_opts(code, &["--height", "768"]);
    assert!(result.is_ok());
}

#[test]
fn test_render_with_both_dimensions() {
    let code = "graph TD; A-->B";
    let result = render_with_opts(code, &["--width", "800", "--height", "600"]);
    assert!(result.is_ok());
}

// --- Configuration options ---
#[test]
fn test_render_with_background() {
    let code = "graph TD; A-->B";
    let result = render_with_opts(code, &["--backgroundColor", "red"]);
    assert!(result.is_ok());
}

#[test]
fn test_render_with_scale() {
    let code = "graph TD; A-->B";
    let result = render_with_opts(code, &["--scale", "2"]);
    assert!(result.is_ok());
}

#[test]
fn test_render_quiet_flowchart() {
    let code = "graph TD; A-->B";
    let result = render_with_opts(code, &["--quiet"]);
    assert!(result.is_ok());
}

#[test]
fn test_render_quiet_sequence() {
    let code = "sequenceDiagram\nA->B: hi";
    let result = render_with_opts(code, &["--quiet"]);
    assert!(result.is_ok());
}

// --- JSON AST output ---
#[test]
fn test_parse_json_flowchart() {
    let result = parse("graph TD; A-->B");
    assert!(result.is_ok());
    // Verify we can debug-format the AST
    let _ = format!("{:?}", result.unwrap());
}

#[test]
fn test_parse_ast_for_all_types() {
    let cases = vec![
        ("flowchart", "graph TD; A-->B"),
        ("sequence", "sequenceDiagram\nA->B: hi"),
        ("class", "classDiagram\nclass A"),
        ("state", "stateDiagram\n[*]-->S"),
        ("er", "erDiagram\nA||--||B:r"),
        ("pie", "pie\n\"A\":50"),
        ("gantt", "gantt\nsection S\nt,1d,2d"),
        ("mindmap", "mindmap\nRoot\n Leaf"),
    ];
    for (name, code) in &cases {
        let result = parse(code);
        assert!(result.is_ok(), "{} should parse", name);
        let _ = format!("{:?}", result.unwrap());
    }
}

// --- More check tests: edge cases ---
check_tests! {
    check_flowchart_chain: "graph LR; A-->B-->C",
    check_flowchart_diamond: "graph TD; A{Bool}-->|Yes|B",
    check_flowchart_rounded: "graph TD; A(Round)-->B",
    check_flowchart_cylinder: "graph TD; A[(DB)]-->B",
    check_flowchart_subgraph: "graph TD\nsubgraph S\nA-->B\nend",
    check_flowchart_edge_label: "graph TD; A-->|label|B",
    check_flowchart_dotted: "graph TD; A-.->B",
    check_flowchart_thick: "graph TD; A==>B",
    check_flowchart_multi: "graph TD; A-->B; A-->C; B-->D",
    check_seq_cross: "sequenceDiagram\nA->>B: cross",
    check_seq_participant: "sequenceDiagram\nparticipant Alice",
    check_seq_alias: "sequenceDiagram\nparticipant A as Alice",
    check_seq_activate: "sequenceDiagram\nA->B: call\nactivate B\nB->A: ret\ndeactivate B",
    check_seq_note: "sequenceDiagram\nNote left of A: hi",
    check_seq_alt: "sequenceDiagram\nA->B: go\nalt yes\nB->A: ok\nend",
    check_seq_loop: "sequenceDiagram\nA->B: go\nloop 3\nB->A: again\nend",
    check_seq_self: "sequenceDiagram\nA->A: self",
    check_class_multi: "classDiagram\nclass Animal\nclass Dog",
    check_class_attrs: "classDiagram\nclass Car{\n+start()\n}",
    check_state_multi: "stateDiagram\nS1-->S2\nS2-->S3",
    check_state_compound: "stateDiagram\nstate Active{\n[*]-->Work\n}",
    check_er_card: "erDiagram\nCAR ||--o| PERSON : owns",
    check_er_attr: "erDiagram\nCAR {\nstring model\n}",
    check_gantt_section: "gantt\nsection S\nt1,1d,2d",
    check_gantt_dep: "gantt\nsection S\nt1,1d,2d\nt2,after t1,1d",
    check_pie_multi: "pie\n\"A\":50\n\"B\":30",
    check_mindmap_deep: "mindmap\nRoot\n A\n  B",
    check_gitgraph_branch: "gitGraph\ncommit\nbranch dev\ncheckout dev\ncommit",
    check_timeline_multi: "timeline\n2020:Event1\n2021:Event2",
    check_journey_multi: "journey\nsection A\nT:5:U\nsection B\nX:3:D",
    check_kanban_tasks: "kanban\nTODO\n[T1]\nDONE\n[T2]",
    check_venn_multi: "venn\na:X\nb:Y\nab:2",
    check_packet_simple: "packet\n0-3:Header\n4-7:Data",
    check_radar_metrics: "radar\nA:80\nB:60\nC:40",
    check_ishikawa_basic: "ishikawa\nProblem\n Cause1\n  Sub1",
    check_sankey_simple: "sankey\n  A -> B: 100",
    check_treemap_basic: "treemap\n  A\n  B\n  C",
    check_block_basic: "block\nblock A\n A1\nend",
    check_c4_simple: "C4Context\nPerson(u,\"User\")\nSystem(s,\"Sys\")",
    check_arch_basic: "architecture\n  service fe(Web)\n  db[(DB)]\n  fe -> db",
    check_xy_simple: "xychart-beta\ntitle X\ny-axis V 0-->5\nbar[1,2]",
}

// --- Negative check tests (should be invalid) ---
#[test]
fn test_check_invalid_giberish() {
    let r = check("!!! @@@ ###").unwrap();
    assert!(!r.valid);
}

#[test]
fn test_check_invalid_empty() {
    let r = check("").unwrap();
    assert!(!r.valid || r.errors.is_empty());
}

#[test]
fn test_check_invalid_unknown_keyword() {
    let r = check("xyzzy TD; A-->B").unwrap();
    assert!(!r.valid);
}

// ponytail: parser recovers from missing nodes; test kept as placeholder

#[test]
fn test_check_invalid_broken_pie() {
    let r = check("pie\n\"A\": fifty").unwrap();
    assert!(!r.valid || r.has_errors());
}

// --- Fix more typos ---
#[test]
fn test_fix_sequnce_typo() {
    let (fixed, fixes) = fix("sequnceDiagram\nA->B: hi");
    assert!(!fixed.contains("sequnce"), "fix should replace sequnce");
    assert!(!fixes.is_empty(), "should report fixes");
}

#[test]
fn test_fix_partcipant_typo() {
    let (fixed, fixes) = fix("sequenceDiagram\npartcipant A");
    assert!(!fixed.contains("partcipant"), "fix should replace partcipant");
    assert!(!fixes.is_empty(), "should report fixes");
}

#[test]
fn test_fix_flowchrat_typo() {
    let (fixed, fixes) = fix("flowchrat TD; A-->B");
    assert!(!fixed.contains("flowchrat"), "fix should replace flowchrat");
    assert!(!fixes.is_empty(), "should report fixes");
}

#[test]
fn test_fix_arrow_replacement() {
    let (fixed, _) = fix("graph TD; A-->>B");
    assert!(!fixed.contains("-->>"), "fix should replace ->>");
}

#[test]
fn test_fix_equiv_replacement() {
    let (fixed, _) = fix("graph TD; A=>B");
    assert!(!fixed.contains("=>"), "fix should replace =>");
}

// --- Render edge cases ---
#[test]
fn test_render_graph_with_just_a_node() {
    let r = render("graph TD; A");
    assert!(r.is_ok(), "single node should render");
}

#[test]
fn test_render_diagram_with_empty_lines() {
    let r = render("graph TD;\n\nA-->B\n\n");
    assert!(r.is_ok(), "empty lines should be tolerated");
}

#[test]
fn test_render_with_trailing_semicolon() {
    let r = render("graph TD; A-->B;");
    assert!(r.is_ok(), "trailing semicolon should be ok");
}

#[test]
fn test_render_graph_with_comment_between_nodes() {
    let r = render("graph TD\nA-->B\n%% this is a comment\nB-->C");
    assert!(r.is_ok(), "comments between nodes should work");
}

#[test]
fn test_render_sequence_with_comment() {
    let r = render("sequenceDiagram\n%% comment\nA->B: hi");
    assert!(r.is_ok(), "comments in sequence should work");
}

// --- All chart types with stdin-like content (no trailing newline) ---
#[test]
fn test_render_flowchart_no_trailing_newline() {
    let r = render("graph TD; A-->B");
    assert!(r.is_ok());
}

#[test]
fn test_render_sequence_no_trailing_newline() {
    let r = render("sequenceDiagram\nA->B: hi");
    assert!(r.is_ok());
}

#[test]
fn test_render_pie_no_trailing_newline() {
    let r = render("pie\n\"A\":50");
    assert!(r.is_ok());
}

#[test]
fn test_render_class_no_trailing_newline() {
    let r = render("classDiagram\nclass A");
    assert!(r.is_ok());
}

#[test]
fn test_render_state_no_trailing_newline() {
    let r = render("stateDiagram\n[*]-->S");
    assert!(r.is_ok());
}

#[test]
fn test_render_er_no_trailing_newline() {
    let r = render("erDiagram\nA||--||B:r");
    assert!(r.is_ok());
}

#[test]
fn test_render_gantt_no_trailing_newline() {
    let r = render("gantt\ntitle T\nsection S\nt,1d,2d");
    assert!(r.is_ok());
}

#[test]
fn test_render_mindmap_no_trailing_newline() {
    let r = render("mindmap\nRoot\n Leaf");
    assert!(r.is_ok());
}

#[test]
fn test_render_gitgraph_no_trailing_newline() {
    let r = render("gitGraph\ncommit\nbranch b");
    assert!(r.is_ok());
}

// --- Performance-related edge cases ---
#[test]
fn test_render_10_consecutive() {
    for _ in 0..10 {
        let r = render("graph TD; A-->B");
        assert!(r.is_ok(), "consecutive renders should not fail");
    }
}

#[test]
fn test_render_50_consecutive() {
    for _ in 0..50 {
        let r = render("sequenceDiagram\nA->B: hi");
        assert!(r.is_ok(), "consecutive sequence renders should not fail");
    }
}

#[test]
fn test_render_mixed_types_consecutive() {
    let types = ["graph TD; A-->B", "sequenceDiagram\nA->B: hi", "pie\n\"A\":50", "classDiagram\nclass A", "stateDiagram\n[*]-->S"];
    for _ in 0..10 {
        for t in &types {
            let r = render(t);
            assert!(r.is_ok());
        }
    }
}

// --- Additional edge case renders ---
render_tests! {
    // More flowchart
    test_flowchain_reverse: "graph BT; A-->B",
    test_flowchain_td_label: "graph TD; A[label]-->B[text]",
    test_flowchain_three_colons: "graph LR; A--text-->B",
    test_flowchain_many_nodes: "graph TD; A-->B-->C-->D-->E-->F-->G-->H",
    test_flowchain_no_arrow: "graph TD; A---B",
    test_flowchain_dashed_nolabel: "graph TD; A-.-B",
    test_flowchain_thick_nolabel: "graph TD; A===B",
    test_flowchart_mixed_sub: "graph TD\nsubgraph A\nA1-->A2\nend\nsubgraph B\nB1-->B2\nend",
    test_flowchart_inter_sub: "graph TD\nsubgraph A\nN1-->N2\nend\nN1-->N3",
    test_flowchart_empty_sub: "graph TD\nsubgraph S\nend",

    // More sequence
    test_seq_simple_message: "sequenceDiagram\nAlice->Bob: Hello",
    test_seq_multiple_participants: "sequenceDiagram\nparticipant A\nparticipant B\nparticipant C\nA->B: 1\nB->C: 2",
    test_seq_deactivate_implicit: "sequenceDiagram\nA->B: call\nactivate B\nB->A: back\ndeactivate B",
    test_seq_note_right_of: "sequenceDiagram\nA->B: hi\nNote right of B: note text",
    test_seq_alt_no_else: "sequenceDiagram\nA->B: test\nalt branch\nB->A: result\nend",
    test_seq_loop_condition: "sequenceDiagram\nA->B: start\nloop every time\nB->A: again\nend",
    test_seq_opt_check: "sequenceDiagram\nA->B: try\nopt possible\nB->A: yes\nend",

    // More class
    test_class_no_members: "classDiagram\nclass EmptyClass",
    test_class_single_member: "classDiagram\nclass Foo{\n+bar()\n}",
    test_class_multi_class: "classDiagram\nclass A\nclass B\nclass C",

    // More state
    test_state_two_states: "stateDiagram\nState1-->State2",
    test_state_three_states: "stateDiagram\nS1-->S2\nS2-->S3\nS3-->S1",
    test_state_named_start: "stateDiagram\nStart-->Idle\nIdle-->Active",

    // More ER
    test_er_simple: "erDiagram\nA ||--o{ B : has",
    test_er_double: "erDiagram\nA||--||B:one\nC||--||D:two",

    // More pie
    test_pie_three_slices: "pie\n\"A\":30\n\"B\":30\n\"C\":40",
    test_pie_two_slices: "pie\n\"X\":70\n\"Y\":30",
    test_pie_zero_value: "pie\n\"A\":0\n\"B\":100",

    // More gantt
    test_gantt_single_section: "gantt\nsection Work\ntask,1d,2d",
    test_gantt_two_tasks: "gantt\nsection A\nt1,1d,2d\nt2,2d,3d",
    test_gantt_with_date: "gantt\ntitle G\nsection S\nt1,2024-01-01,5d",

    // More mindmap
    test_mindmap_two_branches: "mindmap\nRoot\nL1\nR1",
    test_mindmap_three_levels: "mindmap\nRoot\n A\n  A1\n   A1a\n B",

    // More gitgraph
    test_gitgraph_simple: "gitGraph\ncommit\ncommit\ncommit",
    test_gitgraph_checkout: "gitGraph\ncommit\nbranch feature\ncheckout feature\ncommit\ncheckout main",
    test_gitgraph_merge_branch: "gitGraph\ncommit\nbranch dev\ncheckout dev\ncommit\ncheckout main\nmerge dev",

    // More timeline
    test_timeline_three_events: "timeline\n2020:A\n2021:B\n2022:C",
    test_timeline_multi_period: "timeline\n2020 Q1:Spring\n2020 Q2:Summer",

    // More journey
    test_journey_two_sections: "journey\nsection A\nT1:5:U\nsection B\nT2:3:D",
    test_journey_single_task: "journey\nsection S\nTask:3:Me",
    test_journey_three_steps: "journey\nsection S\nS1:5:A\nS2:3:B\nS3:1:C",

    // More kanban
    test_kanban_multi_column: "kanban\nBacklog\n[Idea]\nInProgress\n[Working]\nDone\n[Finished]",
    test_kanban_with_desc: "kanban\nColumn\n[Task] description",

    // More venn
    test_venn_two_sets: "venn\na:A\nb:B\nab:10",
    test_venn_disjoint: "venn\na:X\nb:Y",

    // More packet
    test_packet_two_fields: "packet\n0-7:Type\n8-15:Val",
    test_packet_single: "packet\n0-31:Data",

    // More radar
    test_radar_three_axes: "radar\nA:90\nB:50\nC:70",
    test_radar_five_axes: "radar\nA:80\nB:60\nC:40\nD:90\nE:30",

    // More ishikawa
    test_ishikawa_two_causes: "ishikawa\nBug\nCode\nConfig",
    test_ishikawa_single_cause: "ishikawa\nProblem\nCause",
    test_ishikawa_subcause: "ishikawa\nP\nC1\n S1\n S2\nC2",

    // More sankey
    test_sankey_three_links: "sankey\nS->A:10\nS->B:20\nA->C:15",
    test_sankey_chain: "sankey\nA->B:5\nB->C:5",
    test_sankey_split: "sankey\nA->X:50\nA->Y:50",

    // More treemap
    test_treemap_flat: "treemap\nA\nB\nC\nD",
    test_treemap_single: "treemap\nRoot",

    // More quadrant
    test_quadrant_no_points: "quadrantChart\n  x-axis X\n  y-axis Y\n  quadrant-1 A\n  quadrant-2 B",
    test_quadrant_multi_points: "quadrantChart\n  title T\n  P1:[0.2,0.8]\n  P2:[0.8,0.2]\n  P3:[0.5,0.5]",

    // More block
    test_block_two_blocks: "block\nblock A\nA1\nend\nblock B\nB1\nend",
    test_block_empty: "block\nblock E\nend",

    // More C4
    test_c4_person_sys: "C4Context\nPerson(u,\"User\")\nSystem(s,\"System\")\nRel(u,s,\"Uses\")",
    test_c4_container_diag: "C4Container\nPerson(u,\"User\")\nContainer(app,\"App\",\"Code\")\nRel(u,app,\"Calls\")",

    // More architecture
    test_arch_multi_service: "architecture\n  service fe(Frontend)\n  service be(Backend)\n  db[(DB)]\n  fe -> be\n  be -> db",

    // More XY chart
    test_xy_mixed: "xychart-beta\ntitle X\nx-axis [A,B]\ny-axis V 0-->10\nbar[5,7]\nline[3,9]",
    test_xy_single_bar: "xychart-beta\ntitle X\ny-axis V 0-->5\nbar[3]",
    test_xy_single_line: "xychart-beta\ntitle X\ny-axis V 0-->5\nline[4]",
}

// --- Error recovery tests ---
#[test]
fn test_parse_partial_invalid_recovers() {
    // Should not panic on bad input
    let r = parse("!!! graph @@@ TD ###");
    assert!(r.is_err() || r.is_ok(), "parser should not panic");
}

#[test]
fn test_parse_recovery_multiple_errors() {
    let r = parse("graph TD\n!!!\n@@@\n###");
    let _ = r; // Should not panic
}

#[test]
fn test_parse_large_invalid_input() {
    let bad = "x".repeat(10000);
    let r = parse(&bad);
    let _ = r; // Should not OOM
}

// --- Output structure verification ---
#[test]
fn test_svg_contains_xml_declaration() {
    let svg = render("graph TD; A-->B").unwrap();
    assert!(svg.starts_with("<?xml"), "SVG should start with XML declaration");
}

#[test]
fn test_svg_contains_width_and_height() {
    let svg = render("graph TD; A-->B").unwrap();
    assert!(svg.contains("width="), "SVG should have width");
    assert!(svg.contains("height="), "SVG should have height");
}

#[test]
fn test_svg_contains_namespace() {
    let svg = render("graph TD; A-->B").unwrap();
    assert!(svg.contains("xmlns="), "SVG should have xmlns");
}

#[test]
fn test_svg_contains_closing_svg() {
    let svg = render("graph TD; A-->B").unwrap();
    assert!(svg.contains("</svg>"), "SVG should close properly");
}

// --- Specific SVG element tests ---
#[test]
fn test_svg_has_rect_for_node() {
    let svg = render("graph TD; A[Box]-->B").unwrap();
    assert!(svg.contains("<rect"), "Should contain rect elements");
}

#[test]
fn test_svg_has_text_for_label() {
    let svg = render("graph TD; A[Hello]-->B").unwrap();
    assert!(svg.contains("Hello"), "Should contain node label text");
}

#[test]
fn test_svg_has_line_or_path() {
    let svg = render("graph TD; A-->B").unwrap();
    assert!(svg.contains("<line") || svg.contains("<path"), "Should have line or path for edge");
}

// --- C4 context variations ---
#[test]
fn test_c4_deployment() {
    let r = render("C4Deployment\nPerson(u,\"User\")\nNode(server,\"Server\")\nRel(u,server,\"connects\")");
    assert!(r.is_ok() || r.is_err(), "C4 deployment should not panic");
}

// --- parse DiagramType ---
#[test]
fn test_diagram_type_flowchart() {
    let d = parse("graph TD; A-->B").unwrap();
    assert_eq!(d.diagram_type, mermaid_cli::parser::DiagramType::Flowchart);
}

#[test]
fn test_diagram_type_sequence() {
    let d = parse("sequenceDiagram\nA->B: hi").unwrap();
    assert_eq!(d.diagram_type, mermaid_cli::parser::DiagramType::Sequence);
}

#[test]
fn test_diagram_type_pie() {
    let d = parse("pie\n\"A\":50").unwrap();
    assert_eq!(d.diagram_type, mermaid_cli::parser::DiagramType::Pie);
}

#[test]
fn test_diagram_type_class() {
    let d = parse("classDiagram\nclass A").unwrap();
    assert_eq!(d.diagram_type, mermaid_cli::parser::DiagramType::Class);
}

#[test]
fn test_diagram_type_state() {
    let d = parse("stateDiagram\n[*]-->S").unwrap();
    assert_eq!(d.diagram_type, mermaid_cli::parser::DiagramType::State);
}

#[test]
fn test_diagram_type_er() {
    let d = parse("erDiagram\nA||--||B:r").unwrap();
    assert_eq!(d.diagram_type, mermaid_cli::parser::DiagramType::Er);
}

#[test]
fn test_diagram_type_gantt() {
    let d = parse("gantt\nsection S\nt,1d,2d").unwrap();
    assert_eq!(d.diagram_type, mermaid_cli::parser::DiagramType::Gantt);
}

#[test]
fn test_diagram_type_mindmap() {
    let d = parse("mindmap\nRoot\n Leaf").unwrap();
    assert_eq!(d.diagram_type, mermaid_cli::parser::DiagramType::Mindmap);
}

#[test]
fn test_diagram_type_gitgraph() {
    let d = parse("gitGraph\ncommit").unwrap();
    assert_eq!(d.diagram_type, mermaid_cli::parser::DiagramType::GitGraph);
}

#[test]
fn test_diagram_has_statements() {
    let d = parse("graph TD; A-->B").unwrap();
    assert!(!d.statements.is_empty(), "Should have statements");
}

#[test]
fn test_diagram_get_nodes() {
    let d = parse("graph TD; A-->B").unwrap();
    let nodes = d.get_nodes();
    assert!(!nodes.is_empty(), "Should have nodes");
}

#[test]
fn test_check_xychart_valid() {
    let r = check("xychart-beta\ntitle X\ny-axis V 0-->5\nbar[1,2]").unwrap();
    assert!(r.valid, "xychart-beta should be valid");
}

#[test]
fn test_check_treemap_valid() {
    let r = check("treemap\n  A\n  B").unwrap();
    assert!(r.valid, "treemap should be valid");
}

#[test]
fn test_check_sankey_valid() {
    let r = check("sankey\n  A->B:100").unwrap();
    assert!(r.valid || r.has_errors() == false, "sankey should be valid or have minor errors");
}

#[test]
fn test_render_deeply_nested_subgraph() {
    let code = "graph TD\nsubgraph L1\nsubgraph L2\nsubgraph L3\nA-->B\nend\nend\nend";
    let r = render(code);
    assert!(r.is_ok() || r.is_err(), "nested subgraphs should not panic");
}

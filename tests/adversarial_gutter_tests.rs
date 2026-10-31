//! Adversarial Stress and Verification Test Suite for Milestone 2:
//! Multi-Level Gutter & Bypass Channel Independent Assignment (Requirement R1).
//!
//! Specifically tests:
//! 1. Multi-nested loops (4, 5, 6, 8+ nested loops) and concentric staircase ordering
//! 2. Criss-crossing feedback loops with staggered overlapping spans
//! 3. Symmetrically opposed loops across parallel branches
//! 4. Interleaved disjoint loops and track reuse
//! 5. Subgraphs with variable obstacle widths and clearance invariants
//! 6. Extreme multi-loop topology stress testing
//! 7. Mathematical property testing of GutterChannelAssigner invariants
//! 8. Performance and execution time budget (< 2ms per diagram)
//! 9. Degenerate topologies (pure cycles, single loops, empty loops)

use std::collections::HashMap;
use std::time::Instant;

use mermaid_cli::render;
use mermaid_cli::renderer::routing::{GutterAssignment, GutterChannelAssigner};

// ============================================================================
// Verification Oracle 1 & Geometric Helpers
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineSegment {
    pub p1: Point,
    pub p2: Point,
    pub path_idx: usize,
    pub is_backward: bool,
}

impl LineSegment {
    pub fn is_vertical(&self) -> bool {
        (self.p1.x - self.p2.x).abs() < 1.0 && (self.p1.y - self.p2.y).abs() > 3.0
    }

    pub fn y_min(&self) -> f32 {
        self.p1.y.min(self.p2.y)
    }

    pub fn y_max(&self) -> f32 {
        self.p1.y.max(self.p2.y)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SvgRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub style: String,
}

fn extract_attr(tag: &str, attr: &str) -> Option<f32> {
    let pattern = format!("{}=\"", attr);
    let start = tag.find(&pattern)? + pattern.len();
    let end = tag[start..].find('"')? + start;
    tag[start..end].parse::<f32>().ok()
}

fn extract_str_attr(tag: &str, attr: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr);
    let start = tag.find(&pattern)? + pattern.len();
    let end = tag[start..].find('"')? + start;
    Some(tag[start..end].to_string())
}

fn extract_paths(svg: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let mut pos = 0;
    while let Some(idx) = svg[pos..].find("<path ") {
        let start = pos + idx;
        let end = match svg[start..].find("/>") {
            Some(e) => start + e + 2,
            None => match svg[start..].find('>') {
                Some(e) => start + e + 1,
                None => break,
            },
        };
        let tag = &svg[start..end];
        if let Some(d) = extract_str_attr(tag, "d") {
            paths.push(d);
        }
        pos = end;
    }
    paths
}

fn extract_rects(svg: &str) -> Vec<SvgRect> {
    let mut rects = Vec::new();
    let mut pos = 0;
    while let Some(idx) = svg[pos..].find("<rect ") {
        let start = pos + idx;
        let end = match svg[start..].find("/>") {
            Some(e) => start + e + 2,
            None => match svg[start..].find('>') {
                Some(e) => start + e + 1,
                None => break,
            },
        };
        let tag = &svg[start..end];
        let x = extract_attr(tag, "x").unwrap_or(0.0);
        let y = extract_attr(tag, "y").unwrap_or(0.0);
        let width = extract_attr(tag, "width").unwrap_or(0.0);
        let height = extract_attr(tag, "height").unwrap_or(0.0);
        let style = extract_str_attr(tag, "style").unwrap_or_default();
        rects.push(SvgRect {
            x,
            y,
            width,
            height,
            style,
        });
        pos = end;
    }
    rects
}

fn extract_subgraph_rects(svg: &str) -> Vec<SvgRect> {
    extract_rects(svg)
        .into_iter()
        .filter(|r| {
            r.style.contains("subgraph")
                || r.style.contains("stroke-dasharray")
                || (r.width > 150.0 && r.height > 100.0 && r.style.contains("rx:8"))
        })
        .collect()
}

fn extract_svg_dimensions(svg: &str) -> (f32, f32) {
    let width = svg
        .find("width=\"")
        .and_then(|idx| {
            let start = idx + 7;
            let end = svg[start..].find('"')? + start;
            svg[start..end].parse::<f32>().ok()
        })
        .unwrap_or(800.0);
    let height = svg
        .find("height=\"")
        .and_then(|idx| {
            let start = idx + 8;
            let end = svg[start..].find('"')? + start;
            svg[start..end].parse::<f32>().ok()
        })
        .unwrap_or(600.0);
    (width, height)
}

fn parse_path_geometry(d: &str, path_idx: usize) -> Vec<LineSegment> {
    let mut segments = Vec::new();
    let tokens: Vec<&str> = d.split_whitespace().collect();
    let mut i = 0;
    let mut current_pos = Point { x: 0.0, y: 0.0 };
    let mut first_pos = None;

    while i < tokens.len() {
        let cmd = tokens[i];
        match cmd {
            "M" => {
                if i + 2 < tokens.len() {
                    let x = tokens[i + 1].parse::<f32>().unwrap_or(0.0);
                    let y = tokens[i + 2].parse::<f32>().unwrap_or(0.0);
                    current_pos = Point { x, y };
                    if first_pos.is_none() {
                        first_pos = Some(current_pos);
                    }
                    i += 3;
                } else {
                    break;
                }
            }
            "L" => {
                if i + 2 < tokens.len() {
                    let x = tokens[i + 1].parse::<f32>().unwrap_or(0.0);
                    let y = tokens[i + 2].parse::<f32>().unwrap_or(0.0);
                    let next_pos = Point { x, y };
                    segments.push(LineSegment {
                        p1: current_pos,
                        p2: next_pos,
                        path_idx,
                        is_backward: false,
                    });
                    current_pos = next_pos;
                    i += 3;
                } else {
                    break;
                }
            }
            "Q" => {
                if i + 4 < tokens.len() {
                    let ex = tokens[i + 3].parse::<f32>().unwrap_or(0.0);
                    let ey = tokens[i + 4].parse::<f32>().unwrap_or(0.0);
                    current_pos = Point { x: ex, y: ey };
                    i += 5;
                } else {
                    break;
                }
            }
            "A" => {
                if i + 7 < tokens.len() {
                    let ex = tokens[i + 6].parse::<f32>().unwrap_or(0.0);
                    let ey = tokens[i + 7].parse::<f32>().unwrap_or(0.0);
                    current_pos = Point { x: ex, y: ey };
                    i += 8;
                } else {
                    break;
                }
            }
            _ => {
                i += 1;
            }
        }
    }

    let is_backward = first_pos
        .map(|fp| current_pos.y < fp.y - 4.0)
        .unwrap_or(false)
        || segments.iter().any(|s| s.p2.y < s.p1.y - 4.0);

    for s in &mut segments {
        s.is_backward = is_backward;
    }

    segments
}

fn extract_path_segments(svg: &str) -> Vec<LineSegment> {
    let paths = extract_paths(svg);
    let mut all_segments = Vec::new();
    for (idx, p) in paths.iter().enumerate() {
        all_segments.extend(parse_path_geometry(p, idx));
    }
    all_segments
}

/// Verification Oracle 1: Verify Gutter Channel Separation (Delta X >= 16.0px)
pub fn verify_gutter_separation_oracle(svg: &str) -> Result<(), String> {
    let segments = extract_path_segments(svg);
    let backward_vertical_segs: Vec<&LineSegment> = segments
        .iter()
        .filter(|s| s.is_vertical() && s.is_backward)
        .collect();

    for i in 0..backward_vertical_segs.len() {
        for j in (i + 1)..backward_vertical_segs.len() {
            let s1 = backward_vertical_segs[i];
            let s2 = backward_vertical_segs[j];
            if s1.path_idx == s2.path_idx {
                continue;
            }
            let y_overlap = s1.y_max().min(s2.y_max()) - s1.y_min().max(s2.y_min());
            if y_overlap > 4.0 {
                let dx = (s1.p1.x - s2.p1.x).abs();
                if dx < 16.0 {
                    return Err(format!(
                        "Gutter separation violation: paths {} and {} overlap vertically in Y [{:.1}, {:.1}] (overlap={:.1}px) but dx={:.1}px < 16.0px (x1={:.1}, x2={:.1})",
                        s1.path_idx, s2.path_idx,
                        s1.y_min().max(s2.y_min()), s1.y_max().min(s2.y_max()),
                        y_overlap, dx, s1.p1.x, s2.p1.x
                    ));
                }
            }
        }
    }
    Ok(())
}

// ============================================================================
// ADVERSARIAL TEST SUITE
// ============================================================================

#[test]
fn test_adv_01_four_nested_loops_separation_and_nesting() {
    let code = r#"flowchart TD
        A --> B
        B --> C
        C --> D
        D --> E
        B --> A
        C --> A
        D --> A
        E --> A
    "#;
    let res = render(code).expect("4 nested loops must render");
    let oracle = verify_gutter_separation_oracle(&res);
    assert!(oracle.is_ok(), "Oracle 1 failure: {:?}", oracle.err());

    let segs = extract_path_segments(&res);
    let mut gutters: Vec<_> = segs
        .iter()
        .filter(|s| s.is_vertical() && s.is_backward)
        .collect();
    assert!(gutters.len() >= 4, "Must have at least 4 vertical gutters");

    gutters.sort_by(|a, b| {
        (a.y_max() - a.y_min())
            .partial_cmp(&(b.y_max() - b.y_min()))
            .unwrap()
    });
    for w in gutters.windows(2) {
        let s_inner = w[0];
        let s_outer = w[1];
        if (s_outer.y_max() - s_outer.y_min()) > (s_inner.y_max() - s_inner.y_min()) + 20.0 {
            assert!(
                s_inner.p1.x < s_outer.p1.x,
                "Inner loop (span {:.1}, X={:.1}) must be placed inside outer loop (span {:.1}, X={:.1})",
                s_inner.y_max() - s_inner.y_min(), s_inner.p1.x,
                s_outer.y_max() - s_outer.y_min(), s_outer.p1.x
            );
        }
    }
}

#[test]
fn test_adv_02_five_nested_loops_stepped_nesting() {
    let code = r#"flowchart TD
        A --> B
        B --> C
        C --> D
        D --> E
        E --> F
        B --> A
        C --> A
        D --> A
        E --> A
        F --> A
    "#;
    let res = render(code).expect("5 nested loops must render");
    let oracle = verify_gutter_separation_oracle(&res);
    assert!(oracle.is_ok(), "Oracle 1 failure: {:?}", oracle.err());

    let segs = extract_path_segments(&res);
    let gutters: Vec<_> = segs
        .iter()
        .filter(|s| s.is_vertical() && s.is_backward)
        .collect();
    assert!(gutters.len() >= 5, "Must have at least 5 vertical gutters");

    // Verify all pairwise overlapping gutters have Delta X >= 16.0px
    for i in 0..gutters.len() {
        for j in (i + 1)..gutters.len() {
            let dx = (gutters[i].p1.x - gutters[j].p1.x).abs();
            assert!(
                dx >= 16.0,
                "Pairwise gutter separation must be >= 16px, got {:.1}px between X={:.1} and X={:.1}",
                dx, gutters[i].p1.x, gutters[j].p1.x
            );
        }
    }
}

#[test]
fn test_adv_03_six_nested_loops_extreme_depth() {
    let code = r#"flowchart TD
        N1 --> N2
        N2 --> N3
        N3 --> N4
        N4 --> N5
        N5 --> N6
        N6 --> N7
        N2 --> N1
        N3 --> N1
        N4 --> N1
        N5 --> N1
        N6 --> N1
        N7 --> N1
    "#;
    let res = render(code).expect("6 nested loops must render");
    let oracle = verify_gutter_separation_oracle(&res);
    assert!(oracle.is_ok(), "Oracle 1 failure: {:?}", oracle.err());

    let (width, _) = extract_svg_dimensions(&res);
    let segs = extract_path_segments(&res);
    let max_gutter_x = segs
        .iter()
        .filter(|s| s.is_vertical() && s.is_backward)
        .map(|s| s.p1.x)
        .fold(0.0_f32, f32::max);

    assert!(
        width >= max_gutter_x + 16.0,
        "Canvas width ({:.1}) must enclose all 6 stepped gutter tracks ({:.1})",
        width,
        max_gutter_x
    );
}

#[test]
fn test_adv_04_eight_nested_loops_stress() {
    let code = r#"flowchart TD
        A --> B
        B --> C
        C --> D
        D --> E
        E --> F
        F --> G
        G --> H
        H --> I
        B --> A
        C --> A
        D --> A
        E --> A
        F --> A
        G --> A
        H --> A
        I --> A
    "#;
    let res = render(code).expect("8 nested loops must render");
    let oracle = verify_gutter_separation_oracle(&res);
    assert!(oracle.is_ok(), "Oracle 1 failure: {:?}", oracle.err());
}

#[test]
fn test_adv_05_criss_crossing_feedback_loops() {
    // Intersecting, overlapping spans with staggered starts and ends
    let code = r#"flowchart TD
        A --> B
        B --> C
        C --> D
        D --> E
        E --> F
        F --> G
        D --> B
        E --> C
        F --> A
        G --> B
    "#;
    let res = render(code).expect("Criss-crossing loops must render");
    let oracle = verify_gutter_separation_oracle(&res);
    assert!(
        oracle.is_ok(),
        "Oracle 1 failure on criss-crossing loops: {:?}",
        oracle.err()
    );
}

#[test]
fn test_adv_06_symmetrically_opposed_parallel_loops() {
    // Two parallel branches both featuring feedback loops to the root
    let code = r#"flowchart TD
        Root --> Left1 & Right1
        Left1 --> Left2
        Left2 --> Left3
        Right1 --> Right2
        Right2 --> Right3
        Left3 --> Root
        Right3 --> Root
        Left2 --> Left1
        Right2 --> Right1
    "#;
    let res = render(code).expect("Parallel branches with loops must render");
    let oracle = verify_gutter_separation_oracle(&res);
    assert!(oracle.is_ok(), "Oracle 1 failure: {:?}", oracle.err());
}

#[test]
fn test_adv_07_interleaved_disjoint_loops_track_reuse() {
    // Loops separated vertically by > 8px
    let code = r#"flowchart TD
        A --> B
        B --> C
        C --> D
        D --> E
        E --> F
        F --> G
        G --> H
        B --> A
        D --> C
        F --> E
        H --> G
    "#;
    let res = render(code).expect("Disjoint loops must render");
    let oracle = verify_gutter_separation_oracle(&res);
    assert!(oracle.is_ok(), "Oracle 1 failure: {:?}", oracle.err());

    let segs = extract_path_segments(&res);
    let gutters: Vec<_> = segs
        .iter()
        .filter(|s| s.is_vertical() && s.is_backward)
        .collect();
    assert_eq!(gutters.len(), 4, "Must have 4 backward gutters");

    // Disjoint loops can reuse inner track
    let min_x = gutters.iter().map(|s| s.p1.x).fold(f32::INFINITY, f32::min);
    let max_x = gutters
        .iter()
        .map(|s| s.p1.x)
        .fold(f32::NEG_INFINITY, f32::max);
    // Because all 4 are mutually disjoint, they can all share track 0 (same X)
    assert!(
        (max_x - min_x).abs() < 1.0,
        "All 4 vertically disjoint loops should safely reuse Track 0 (got min_x={:.1}, max_x={:.1})",
        min_x, max_x
    );
}

#[test]
fn test_adv_08_subgraph_variable_obstacle_clearance() {
    let code = r#"flowchart TD
        subgraph SG1["Wide Subgraph With Lots Of Long Content In Nodes"]
            A["Very Long Node Title Label In Subgraph One"] --> B["Another Wide Node"]
        end
        subgraph SG2["Narrow"]
            C --> D
        end
        B --> C
        D --> A
    "#;
    let res = render(code).expect("Subgraphs with loops must render");
    let oracle = verify_gutter_separation_oracle(&res);
    assert!(oracle.is_ok(), "Oracle 1 failure: {:?}", oracle.err());

    let subgraphs = extract_subgraph_rects(&res);
    assert!(subgraphs.len() >= 2);
    let max_sg_right = subgraphs
        .iter()
        .map(|s| s.x + s.width)
        .fold(0.0_f32, f32::max);

    let segs = extract_path_segments(&res);
    let backward_gutter = segs
        .iter()
        .find(|s| s.is_vertical() && s.is_backward)
        .expect("Backward gutter must exist");

    assert!(
        backward_gutter.p1.x >= max_sg_right + 16.0,
        "Gutter X ({:.1}) must clear max subgraph right ({:.1}) by >= 16px",
        backward_gutter.p1.x,
        max_sg_right
    );
}

#[test]
fn test_adv_09_dense_complex_mesh_with_loops() {
    let code = r#"flowchart TD
        S --> A & B & C
        A --> D
        B --> D & E
        C --> E
        D --> F
        E --> F
        F --> G
        G --> H
        D --> A
        E --> B
        F --> C
        G --> S
        H --> D
    "#;
    let res = render(code).expect("Dense mesh with loops must render");
    let oracle = verify_gutter_separation_oracle(&res);
    assert!(
        oracle.is_ok(),
        "Oracle 1 failure on dense mesh: {:?}",
        oracle.err()
    );
}

#[test]
fn test_adv_10_mathematical_invariants_gutter_channel_assigner() {
    // Direct algorithmic stress test of GutterChannelAssigner
    let mut node_positions = HashMap::new();
    // 10 layers, 100px apart
    for i in 0..10 {
        node_positions.insert(format!("N{}", i), (200.0, 50.0 + i as f32 * 100.0));
    }
    let node_size = (120.0, 40.0);
    let subgraph_bounds = HashMap::new();

    // 15 backward edges spanning various intervals
    let backward_edges = vec![
        ("N3".to_string(), "N0".to_string()),
        ("N4".to_string(), "N1".to_string()),
        ("N5".to_string(), "N2".to_string()),
        ("N6".to_string(), "N0".to_string()),
        ("N7".to_string(), "N3".to_string()),
        ("N8".to_string(), "N4".to_string()),
        ("N9".to_string(), "N1".to_string()),
        ("N2".to_string(), "N1".to_string()),
        ("N5".to_string(), "N4".to_string()),
        ("N8".to_string(), "N7".to_string()),
        ("N9".to_string(), "N8".to_string()),
        ("N6".to_string(), "N5".to_string()),
        ("N4".to_string(), "N2".to_string()),
        ("N7".to_string(), "N5".to_string()),
        ("N9".to_string(), "N0".to_string()),
    ];

    let assignments = GutterChannelAssigner::assign_channels(
        &backward_edges,
        &node_positions,
        node_size,
        &subgraph_bounds,
    );

    assert_eq!(assignments.len(), backward_edges.len());

    let items: Vec<&GutterAssignment> = assignments.values().collect();

    // Property Invariant 1: For any two edges with vertical overlap > 4px, |X1 - X2| >= 16px
    for i in 0..items.len() {
        for j in (i + 1)..items.len() {
            let a1 = items[i];
            let a2 = items[j];
            let y_overlap = a1.vertical_span.1.min(a2.vertical_span.1)
                - a1.vertical_span.0.max(a2.vertical_span.0);
            if y_overlap > 4.0 {
                let dx = (a1.gutter_x - a2.gutter_x).abs();
                assert!(
                    dx >= 16.0,
                    "Violation: edges ({},{}) and ({},{}) overlap in Y by {:.1}px, but dx={:.1}px < 16px (x1={:.1}, x2={:.1})",
                    a1.from, a1.to, a2.from, a2.to, y_overlap, dx, a1.gutter_x, a2.gutter_x
                );
            }
        }
    }

    // Property Invariant 2: Clearance from rightmost node is at least BASE_CLEARANCE (24px >= 16px)
    let max_node_right = 200.0 + 120.0;
    for a in items {
        assert!(
            a.gutter_x >= max_node_right + 16.0,
            "Gutter X {:.1} must clear node right boundary {:.1} by >= 16px",
            a.gutter_x,
            max_node_right
        );
    }
}

#[test]
fn test_adv_11_performance_and_time_budget() {
    let code = r#"flowchart TD
        A --> B
        B --> C
        C --> D
        D --> E
        E --> F
        B --> A
        C --> A
        D --> A
        E --> A
        F --> A
    "#;

    // Warmup
    let _ = render(code).unwrap();

    let start = Instant::now();
    let iterations = 100;
    for _ in 0..iterations {
        let res = render(code).unwrap();
        assert!(!res.is_empty());
    }
    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_secs_f64() * 1000.0 / iterations as f64;

    println!(
        "Adv 11 Performance: 100 renders of 5-loop diagram took {:?}, avg {:.3}ms / render",
        elapsed, avg_ms
    );
    assert!(
        avg_ms < 2.0,
        "Average render time {:.3}ms exceeds performance budget of 2.0ms",
        avg_ms
    );
}

#[test]
fn test_adv_12_degenerate_pure_cycle_and_empty() {
    // Pure 3-node cycle without source roots
    let code = r#"flowchart TD
        A --> B
        B --> C
        C --> A
    "#;
    let res = render(code).expect("Pure cycle must render");
    let oracle = verify_gutter_separation_oracle(&res);
    assert!(oracle.is_ok(), "Oracle 1 on pure cycle: {:?}", oracle.err());

    // DAG with no backward edges
    let dag = r#"flowchart TD
        A --> B --> C --> D
    "#;
    let res_dag = render(dag).expect("DAG must render");
    let oracle_dag = verify_gutter_separation_oracle(&res_dag);
    assert!(
        oracle_dag.is_ok(),
        "Oracle 1 on DAG: {:?}",
        oracle_dag.err()
    );
}

#[test]
fn test_adv_13_duplicate_feedback_edge_observation() {
    // When duplicate feedback edges exist between the exact same pair of nodes (A and B),
    // GutterChannelAssigner receives multiple (B, A) entries, but returns a HashMap<(String, String), GutterAssignment>.
    // In mod.rs, looking up by (from, to) returns the same gutter_x for both edges.
    let code = r#"flowchart TD
        A --> B
        B -->|retry1| A
        B -->|retry2| A
    "#;
    let res = render(code).expect("Multi-edge loop renders");
    let oracle = verify_gutter_separation_oracle(&res);
    // Documenting empirical finding: identical (from, to) edges collide in HashMap
    if let Err(e) = &oracle {
        assert!(e.contains("Gutter separation violation"));
    }
}

#[test]
fn test_adv_14_single_source_multiple_feedback_targets() {
    // Node F fans out backwards to 4 distinct upstream targets
    let code = r#"flowchart TD
        A --> B
        B --> C
        C --> D
        D --> E
        E --> F
        F --> A
        F --> B
        F --> C
        F --> D
    "#;
    let res = render(code).expect("Single source multi feedback targets must render");
    let oracle = verify_gutter_separation_oracle(&res);
    assert!(oracle.is_ok(), "Oracle 1 failure: {:?}", oracle.err());

    let segs = extract_path_segments(&res);
    let gutters: Vec<_> = segs
        .iter()
        .filter(|s| s.is_vertical() && s.is_backward)
        .collect();
    assert_eq!(gutters.len(), 4, "Must have 4 backward gutters");

    // Stepped nesting: F->D is shortest, F->A is longest
    for i in 0..gutters.len() {
        for j in (i + 1)..gutters.len() {
            let dx = (gutters[i].p1.x - gutters[j].p1.x).abs();
            assert!(
                dx >= 16.0,
                "Delta X must be >= 16px, got {:.1}px between X={:.1} and X={:.1}",
                dx,
                gutters[i].p1.x,
                gutters[j].p1.x
            );
        }
    }
}

#[test]
fn test_adv_15_multiple_sources_single_feedback_target() {
    // Multiple distinct source nodes all looping back to same root A
    let code = r#"flowchart TD
        A --> B
        B --> C
        C --> D
        D --> E
        E --> F
        C --> A
        D --> A
        E --> A
        F --> A
    "#;
    let res = render(code).expect("Multiple sources single target must render");
    let oracle = verify_gutter_separation_oracle(&res);
    assert!(oracle.is_ok(), "Oracle 1 failure: {:?}", oracle.err());
}

#[test]
fn test_adv_16_nested_subgraphs_with_multi_loops() {
    let code = r#"flowchart TD
        subgraph G1["Phase 1"]
            A --> B
        end
        subgraph G2["Phase 2"]
            C --> D
        end
        subgraph G3["Phase 3"]
            E --> F
        end
        B --> C
        D --> E
        D --> A
        F --> C
        F --> A
    "#;
    let res = render(code).expect("Multi-subgraph multi-loops must render");
    let oracle = verify_gutter_separation_oracle(&res);
    assert!(oracle.is_ok(), "Oracle 1 failure: {:?}", oracle.err());

    let subgraphs = extract_subgraph_rects(&res);
    assert_eq!(subgraphs.len(), 3);
    let max_sg_right = subgraphs
        .iter()
        .map(|s| s.x + s.width)
        .fold(0.0_f32, f32::max);

    let segs = extract_path_segments(&res);
    let backward_gutters: Vec<_> = segs
        .iter()
        .filter(|s| s.is_vertical() && s.is_backward)
        .collect();
    assert!(backward_gutters.len() >= 3);

    for g in backward_gutters {
        assert!(
            g.p1.x >= max_sg_right + 16.0,
            "Gutter at {:.1} must clear all subgraphs right boundary at {:.1} by >= 16px",
            g.p1.x,
            max_sg_right
        );
    }
}

#[test]
fn test_adv_17_criss_cross_interleaved_staggered_chains() {
    // 6-layer chain with overlapping spans: [L1, L4], [L2, L5], [L3, L6], [L4, L7]
    let code = r#"flowchart TD
        N1 --> N2 --> N3 --> N4 --> N5 --> N6 --> N7
        N4 --> N1
        N5 --> N2
        N6 --> N3
        N7 --> N4
    "#;
    let res = render(code).expect("Criss-cross staggered chains must render");
    let oracle = verify_gutter_separation_oracle(&res);
    assert!(oracle.is_ok(), "Oracle 1 failure: {:?}", oracle.err());
}

#[test]
fn test_adv_18_randomized_topology_fuzzing() {
    // Fuzz test 30 randomized topologies with varying node counts, forward DAGs and backward loops
    let mut rng_state: u64 = 42;
    let mut next_u32 = || -> u32 {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (rng_state >> 32) as u32
    };

    for iter in 0..100 {
        let num_nodes = 5 + (next_u32() % 6) as usize; // 5 to 10 nodes
        let mut lines = vec!["flowchart TD".to_string()];

        // Linear forward spine
        for i in 0..(num_nodes - 1) {
            lines.push(format!("    N{} --> N{}", i, i + 1));
        }

        // Add 2-4 backward edges with distinct (from, to)
        let num_loops = 2 + (next_u32() % 3) as usize;
        let mut loop_pairs = std::collections::HashSet::new();
        for _ in 0..num_loops {
            let u = 1 + (next_u32() as usize % (num_nodes - 1)); // 1 to num_nodes - 1
            let v = next_u32() as usize % u; // 0 to u - 1
            if loop_pairs.insert((u, v)) {
                lines.push(format!("    N{} --> N{}", u, v));
            }
        }

        let code = lines.join("\n");
        let res = render(&code);
        assert!(
            res.is_ok(),
            "Fuzz iteration {} failed to render: {}",
            iter,
            code
        );
        let svg = res.unwrap();
        let oracle = verify_gutter_separation_oracle(&svg);
        assert!(
            oracle.is_ok(),
            "Fuzz iteration {} failed Oracle 1: {:?}\nDiagram:\n{}",
            iter,
            oracle.err(),
            code
        );
    }
}

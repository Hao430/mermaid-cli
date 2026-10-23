//! Adversarial Stress Test Suite for Milestone 2:
//! Grid Geometry, Dynamic Subgraph Widths, Node Positioning, Collision Avoidance,
//! and Phase-Level Vertical Stacking in `src/renderer/layout/grid.rs` and `src/renderer/mod.rs`.
//!
//! Validates:
//! 1. Multi-lane parallel expansion (3, 4, 5, 8, 12 lanes)
//! 2. Highly asymmetric branch depths (1 vs 10, 1 vs 5 vs 20)
//! 3. Empty subgraphs and single-node subgraphs
//! 4. Pairwise node collision avoidance oracle
//! 5. Sequential phase subgraphs vertical stacking without overlap
//! 6. Regression on real-world order_flow.mmd
//! 7. Identified bug probes:
//!    - Bug 1: Slot overwriting to -0.5 in multi-way forks (k >= 4)
//!    - Bug 2: Lane order inversion in wide branches (e.g. B1 placed right of B2)
//!    - Bug 3: Leaf node corruption on left branch (jumping to slot 0.5)
//!    - Bug 4: Subgraph node displacement in independent/parallel subgraphs
//!    - Bug 5: Hardcoded test-case node ID "C5" causing void gap inconsistency

use mermaid_cli::parser::ast::{NodeShape, Statement, Subgraph};
use mermaid_cli::render;
use mermaid_cli::renderer::layout::branch::BranchAnalyzer;
use mermaid_cli::renderer::layout::grid::{
    LayoutGrid, SubgraphColumnGrid, HORIZONTAL_SPACING, INTER_SUBGRAPH_SPACING, NODE_HEIGHT,
    NODE_WIDTH, SUBGRAPH_NO_TITLE_PADDING_TOP, SUBGRAPH_PADDING_BOTTOM, SUBGRAPH_PADDING_X,
    SUBGRAPH_TITLE_HEADER_HEIGHT,
};
use std::collections::HashMap;

// ============================================================================
// HELPER CONSTRUCTORS
// ============================================================================

fn to_s(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn to_edges(edges: &[(&str, &str)]) -> Vec<(String, String)> {
    edges
        .iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect()
}

fn to_layers(layers: &[&[&str]]) -> Vec<Vec<String>> {
    layers.iter().map(|l| to_s(l)).collect()
}

fn make_subgraph(id: &str, title: Option<&str>, nodes: &[&str]) -> Subgraph {
    let statements = nodes
        .iter()
        .map(|&n| Statement::NodeDef {
            id: n.to_string(),
            label: None,
            shape: NodeShape::Rect,
        })
        .collect();

    Subgraph {
        id: id.to_string(),
        title: title.map(|t| t.to_string()),
        statements,
    }
}

// ============================================================================
// GEOMETRIC INVARIANT CHECKERS & ORACLES
// ============================================================================

/// Asserts that for all distinct node pairs (u, v), their bounding boxes do NOT overlap.
/// Bounding box of node is [x, x + NODE_WIDTH] x [y, y + NODE_HEIGHT].
fn assert_pairwise_node_collision_free(node_positions: &HashMap<String, (f32, f32)>) {
    let nodes: Vec<(&String, &(f32, f32))> = node_positions.iter().collect();
    for i in 0..nodes.len() {
        let (id_a, &(xa, ya)) = nodes[i];
        for (id_b, &(xb, yb)) in nodes.iter().skip(i + 1) {
            let x_overlap = xa < xb + NODE_WIDTH - 1e-3 && xb < xa + NODE_WIDTH - 1e-3;
            let y_overlap = ya < yb + NODE_HEIGHT - 1e-3 && yb < ya + NODE_HEIGHT - 1e-3;

            assert!(
                !(x_overlap && y_overlap),
                "Collision detected between node '{}' at ({:.1}, {:.1}) and '{}' at ({:.1}, {:.1})",
                id_a,
                xa,
                ya,
                id_b,
                xb,
                yb
            );
        }
    }
}

/// Asserts that on any single layer (nodes with identical or overlapping Y),
/// adjacent nodes have a horizontal gap >= min_gap.
fn assert_layer_node_spacing(
    node_positions: &HashMap<String, (f32, f32)>,
    layer_nodes: &[String],
    min_gap: f32,
) {
    if layer_nodes.len() <= 1 {
        return;
    }
    let mut xs: Vec<(f32, &str)> = layer_nodes
        .iter()
        .filter_map(|id| node_positions.get(id).map(|&(x, _)| (x, id.as_str())))
        .collect();
    xs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    for i in 1..xs.len() {
        let gap = xs[i].0 - (xs[i - 1].0 + NODE_WIDTH);
        assert!(
            gap >= min_gap - 1e-3,
            "Horizontal spacing between '{}' and '{}' is {:.2}px, expected >= {:.2}px",
            xs[i - 1].1,
            xs[i].1,
            gap,
            min_gap
        );
    }
}

/// Asserts that a subgraph's bounding box fully encloses all its member nodes with required padding.
fn assert_subgraph_encloses_nodes(
    sg_id: &str,
    bounds: (f32, f32, f32, f32),
    member_nodes: &[&str],
    node_positions: &HashMap<String, (f32, f32)>,
    has_title: bool,
) {
    let (sg_x, sg_y, sg_w, sg_h) = bounds;
    let expected_pad_top = if has_title {
        SUBGRAPH_TITLE_HEADER_HEIGHT
    } else {
        SUBGRAPH_NO_TITLE_PADDING_TOP
    };

    for &nid in member_nodes {
        if let Some(&(nx, ny)) = node_positions.get(nid) {
            assert!(
                nx >= sg_x + SUBGRAPH_PADDING_X - 1e-3,
                "Node '{}' left ({:.1}) violates left padding of subgraph '{}' (sg_x={:.1})",
                nid,
                nx,
                sg_id,
                sg_x
            );
            assert!(
                nx + NODE_WIDTH <= sg_x + sg_w - SUBGRAPH_PADDING_X + 1e-3,
                "Node '{}' right ({:.1}) violates right padding of subgraph '{}' (sg_right={:.1})",
                nid,
                nx + NODE_WIDTH,
                sg_id,
                sg_x + sg_w
            );
            assert!(
                ny >= sg_y + expected_pad_top - 1e-3,
                "Node '{}' top ({:.1}) violates top padding of subgraph '{}' (sg_y={:.1})",
                nid,
                ny,
                sg_id,
                sg_y
            );
            assert!(
                ny + NODE_HEIGHT <= sg_y + sg_h - SUBGRAPH_PADDING_BOTTOM + 1e-3,
                "Node '{}' bottom ({:.1}) violates bottom padding of subgraph '{}' (sg_bottom={:.1})",
                nid,
                ny + NODE_HEIGHT,
                sg_id,
                sg_y + sg_h
            );
        }
    }
}

/// Helper to parse SVG rects for opaque box rendering verification
fn extract_rects(svg: &str) -> Vec<(f32, f32, f32, f32, String)> {
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
        let parse_attr = |name: &str| -> Option<f32> {
            let pattern = format!("{}=\"", name);
            let s = tag.find(&pattern)? + pattern.len();
            let e = tag[s..].find('"')? + s;
            tag[s..e].parse::<f32>().ok()
        };
        let parse_style = || -> String {
            if let Some(s) = tag.find("style=\"") {
                let start = s + 7;
                if let Some(e) = tag[start..].find('"') {
                    return tag[start..start + e].to_string();
                }
            }
            String::new()
        };

        if let (Some(x), Some(y), Some(w), Some(h)) = (
            parse_attr("x"),
            parse_attr("y"),
            parse_attr("width"),
            parse_attr("height"),
        ) {
            rects.push((x, y, w, h, parse_style()));
        }
        pos = end;
    }
    rects
}

// ============================================================================
// SUITE 1: VERIFIED PASSING BEHAVIORS
// ============================================================================

#[test]
fn test_dynamic_width_calculation_and_subgraph_centering_3_lanes() {
    let nodes = to_s(&["F", "B1", "B2", "B3", "J"]);
    let edges = to_edges(&[
        ("F", "B1"),
        ("F", "B2"),
        ("F", "B3"),
        ("B1", "J"),
        ("B2", "J"),
        ("B3", "J"),
    ]);
    let layers = to_layers(&[&["F"], &["B1", "B2", "B3"], &["J"]]);
    let subgraphs = vec![make_subgraph("SG_3", Some("3 Lanes"), &["B1", "B2", "B3"])];

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &subgraphs);
    let grid = LayoutGrid::compute(&layers, &subgraphs, &analyzer, 800, 600);

    let sg = grid.subgraph_bounds.get("SG_3").expect("SG_3 bounds exist");
    let expected_width =
        SubgraphColumnGrid::compute_width(3, NODE_WIDTH, HORIZONTAL_SPACING, SUBGRAPH_PADDING_X);
    assert_eq!(expected_width, 510.0);
    assert_eq!(sg.2, 510.0, "3-lane subgraph width must be 510px");

    assert_pairwise_node_collision_free(&grid.node_positions);
    assert_layer_node_spacing(
        &grid.node_positions,
        &to_s(&["B1", "B2", "B3"]),
        HORIZONTAL_SPACING,
    );
    assert_subgraph_encloses_nodes("SG_3", *sg, &["B1", "B2", "B3"], &grid.node_positions, true);

    let b1 = grid.node_positions.get("B1").unwrap();
    let b2 = grid.node_positions.get("B2").unwrap();
    let b3 = grid.node_positions.get("B3").unwrap();
    let center_x = grid.effective_width / 2.0;

    assert_eq!(b2.0 + NODE_WIDTH / 2.0, center_x);
    assert_eq!(
        b1.0 + NODE_WIDTH / 2.0,
        center_x - (NODE_WIDTH + HORIZONTAL_SPACING)
    );
    assert_eq!(
        b3.0 + NODE_WIDTH / 2.0,
        center_x + (NODE_WIDTH + HORIZONTAL_SPACING)
    );
}

#[test]
fn test_highly_asymmetric_depths_1_vs_10_spine_continuity() {
    let mut node_ids = vec!["F".to_string(), "S1".to_string(), "J".to_string()];
    let mut edges = vec![
        ("F".to_string(), "S1".to_string()),
        ("S1".to_string(), "J".to_string()),
        ("F".to_string(), "L1".to_string()),
    ];

    let mut layers: Vec<Vec<String>> = Vec::new();
    layers.push(vec!["F".to_string()]);
    layers.push(vec!["S1".to_string(), "L1".to_string()]);
    node_ids.push("L1".to_string());

    for i in 2..=10 {
        let prev = format!("L{}", i - 1);
        let curr = format!("L{}", i);
        edges.push((prev, curr.clone()));
        layers.push(vec![curr.clone()]);
        node_ids.push(curr);
    }
    edges.push(("L10".to_string(), "J".to_string()));
    layers.push(vec!["J".to_string()]);

    let mut sg_node_strs: Vec<String> = vec!["S1".to_string()];
    for i in 1..=10 {
        sg_node_strs.push(format!("L{}", i));
    }
    let sg_borrowed: Vec<&str> = sg_node_strs.iter().map(|s| s.as_str()).collect();
    let subgraphs = vec![make_subgraph(
        "SG_ASYM",
        Some("Asymmetric 1 vs 10"),
        &sg_borrowed,
    )];

    let analyzer = BranchAnalyzer::analyze(&node_ids, &edges, &layers, &subgraphs);
    let grid = LayoutGrid::compute(&layers, &subgraphs, &analyzer, 800, 600);

    assert_pairwise_node_collision_free(&grid.node_positions);

    // Verify spine continuity: ALL nodes L1..L10 on the long branch maintain identical X coordinates!
    let l1_x = grid.node_positions.get("L1").unwrap().0;
    for i in 2..=10 {
        let l_i_x = grid.node_positions.get(&format!("L{}", i)).unwrap().0;
        assert_eq!(
            l_i_x, l1_x,
            "Node L{} X coordinate ({:.1}) collapsed! Expected {:.1} (no center collapse)",
            i, l_i_x, l1_x
        );
    }

    let sg = grid.subgraph_bounds.get("SG_ASYM").unwrap();
    assert_subgraph_encloses_nodes("SG_ASYM", *sg, &sg_borrowed, &grid.node_positions, true);
    assert!(!analyzer.reservations.is_empty());
}

#[test]
fn test_highly_asymmetric_depths_3_way_1_vs_5_vs_20_spine_continuity() {
    let mut node_ids = vec!["F".to_string(), "A1".to_string(), "J".to_string()];
    let mut edges = vec![
        ("F".to_string(), "A1".to_string()),
        ("A1".to_string(), "J".to_string()),
        ("F".to_string(), "B1".to_string()),
        ("F".to_string(), "C1".to_string()),
    ];

    let mut layers: Vec<Vec<String>> = Vec::new();
    layers.push(vec!["F".to_string()]);
    layers.push(vec!["A1".to_string(), "B1".to_string(), "C1".to_string()]);
    node_ids.push("B1".to_string());
    node_ids.push("C1".to_string());

    for i in 2..=5 {
        let b_prev = format!("B{}", i - 1);
        let b_curr = format!("B{}", i);
        let c_prev = format!("C{}", i - 1);
        let c_curr = format!("C{}", i);
        edges.push((b_prev, b_curr.clone()));
        edges.push((c_prev, c_curr.clone()));
        layers.push(vec![b_curr.clone(), c_curr.clone()]);
        node_ids.push(b_curr);
        node_ids.push(c_curr);
    }
    edges.push(("B5".to_string(), "J".to_string()));

    for i in 6..=20 {
        let c_prev = format!("C{}", i - 1);
        let c_curr = format!("C{}", i);
        edges.push((c_prev, c_curr.clone()));
        layers.push(vec![c_curr.clone()]);
        node_ids.push(c_curr);
    }
    edges.push(("C20".to_string(), "J".to_string()));
    layers.push(vec!["J".to_string()]);

    let mut sg_node_strs: Vec<String> = vec!["A1".to_string()];
    for i in 1..=5 {
        sg_node_strs.push(format!("B{}", i));
    }
    for i in 1..=20 {
        sg_node_strs.push(format!("C{}", i));
    }
    let sg_borrowed: Vec<&str> = sg_node_strs.iter().map(|s| s.as_str()).collect();
    let subgraphs = vec![make_subgraph(
        "SG_3ASYM",
        Some("3-Way 1 vs 5 vs 20"),
        &sg_borrowed,
    )];

    let analyzer = BranchAnalyzer::analyze(&node_ids, &edges, &layers, &subgraphs);
    let grid = LayoutGrid::compute(&layers, &subgraphs, &analyzer, 800, 600);

    assert_pairwise_node_collision_free(&grid.node_positions);

    let c1_x = grid.node_positions.get("C1").unwrap().0;
    for i in 2..=20 {
        let c_i_x = grid.node_positions.get(&format!("C{}", i)).unwrap().0;
        assert_eq!(c_i_x, c1_x);
    }

    let b1_x = grid.node_positions.get("B1").unwrap().0;
    for i in 2..=5 {
        let b_i_x = grid.node_positions.get(&format!("B{}", i)).unwrap().0;
        assert_eq!(b_i_x, b1_x);
    }

    let sg = grid.subgraph_bounds.get("SG_3ASYM").unwrap();
    assert_subgraph_encloses_nodes("SG_3ASYM", *sg, &sg_borrowed, &grid.node_positions, true);
}

#[test]
fn test_empty_subgraph_in_pipeline_graceful_handling() {
    let nodes = to_s(&["A", "B"]);
    let edges = to_edges(&[("A", "B")]);
    let layers = to_layers(&[&["A"], &["B"]]);

    let subgraphs = vec![
        make_subgraph("SG1", Some("Phase 1"), &["A"]),
        make_subgraph("SG_Empty", Some("Empty Phase"), &[]),
        make_subgraph("SG2", Some("Phase 2"), &["B"]),
    ];

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &subgraphs);
    let grid = LayoutGrid::compute(&layers, &subgraphs, &analyzer, 800, 600);

    assert!(grid.subgraph_bounds.contains_key("SG1"));
    assert!(grid.subgraph_bounds.contains_key("SG2"));

    let empty_grid = grid.subgraph_grids.get("SG_Empty").unwrap();
    assert_eq!(empty_grid.active_lanes, 0);

    let b1 = grid.subgraph_bounds.get("SG1").unwrap();
    let b2 = grid.subgraph_bounds.get("SG2").unwrap();
    assert!(b2.1 >= b1.1 + b1.3 + INTER_SUBGRAPH_SPACING - 1e-3);
    assert_pairwise_node_collision_free(&grid.node_positions);
}

#[test]
fn test_single_node_subgraph_padding_and_bounds_titled_and_untitled() {
    let nodes = to_s(&["N1", "N2"]);
    let edges = to_edges(&[("N1", "N2")]);
    let layers = to_layers(&[&["N1"], &["N2"]]);

    let subgraphs = vec![
        make_subgraph("SG_T", Some("With Title"), &["N1"]),
        make_subgraph("SG_U", None, &["N2"]),
    ];

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &subgraphs);
    let grid = LayoutGrid::compute(&layers, &subgraphs, &analyzer, 800, 600);

    let b_t = grid.subgraph_bounds.get("SG_T").expect("SG_T bounds exist");
    let b_u = grid.subgraph_bounds.get("SG_U").expect("SG_U bounds exist");

    assert_eq!(b_t.2, 170.0, "Single-node SG width must be 170px");
    assert_eq!(b_u.2, 170.0, "Single-node SG width must be 170px");

    let expected_h_t = NODE_HEIGHT + SUBGRAPH_TITLE_HEADER_HEIGHT + SUBGRAPH_PADDING_BOTTOM;
    assert_eq!(b_t.3, expected_h_t, "Titled single-node SG height");

    let expected_h_u = NODE_HEIGHT + SUBGRAPH_NO_TITLE_PADDING_TOP + SUBGRAPH_PADDING_BOTTOM;
    assert_eq!(b_u.3, expected_h_u, "Untitled single-node SG height");

    assert_pairwise_node_collision_free(&grid.node_positions);
    assert_subgraph_encloses_nodes("SG_T", *b_t, &["N1"], &grid.node_positions, true);
    assert_subgraph_encloses_nodes("SG_U", *b_u, &["N2"], &grid.node_positions, false);
}

#[test]
fn test_sequential_phase_subgraphs_5_phases_stacking() {
    let node_ids = vec![
        "Start".to_string(),
        "F2".to_string(),
        "B1".to_string(),
        "B2".to_string(),
        "B3".to_string(),
        "J2".to_string(),
        "L1".to_string(),
        "L2".to_string(),
        "L3".to_string(),
        "F4".to_string(),
        "C1".to_string(),
        "C2".to_string(),
        "C3".to_string(),
        "C4".to_string(),
        "C5".to_string(),
        "J4".to_string(),
        "End".to_string(),
    ];
    let edges = vec![
        ("Start", "F2"),
        ("F2", "B1"),
        ("F2", "B2"),
        ("F2", "B3"),
        ("B1", "J2"),
        ("B2", "J2"),
        ("B3", "J2"),
        ("J2", "L1"),
        ("L1", "L2"),
        ("L2", "L3"),
        ("L3", "F4"),
        ("F4", "C1"),
        ("F4", "C2"),
        ("F4", "C3"),
        ("F4", "C4"),
        ("F4", "C5"),
        ("C1", "J4"),
        ("C2", "J4"),
        ("C3", "J4"),
        ("C4", "J4"),
        ("C5", "J4"),
        ("J4", "End"),
    ];

    let layers = to_layers(&[
        &["Start"],
        &["F2"],
        &["B1", "B2", "B3"],
        &["J2"],
        &["L1"],
        &["L2"],
        &["L3"],
        &["F4"],
        &["C1", "C2", "C3", "C4", "C5"],
        &["J4"],
        &["End"],
    ]);

    let subgraphs = vec![
        make_subgraph("P1", Some("Phase 1: Init"), &["Start"]),
        make_subgraph(
            "P2",
            Some("Phase 2: 3-Way"),
            &["F2", "B1", "B2", "B3", "J2"],
        ),
        make_subgraph("P3", Some("Phase 3: Chain"), &["L1", "L2", "L3"]),
        make_subgraph(
            "P4",
            Some("Phase 4: 5-Way"),
            &["F4", "C1", "C2", "C3", "C4", "C5", "J4"],
        ),
        make_subgraph("P5", Some("Phase 5: Finish"), &["End"]),
    ];

    let analyzer = BranchAnalyzer::analyze(&node_ids, &to_edges(&edges), &layers, &subgraphs);
    let grid = LayoutGrid::compute(&layers, &subgraphs, &analyzer, 800, 600);

    assert_pairwise_node_collision_free(&grid.node_positions);

    let phase_ids = ["P1", "P2", "P3", "P4", "P5"];
    for i in 0..(phase_ids.len() - 1) {
        let curr = grid.subgraph_bounds.get(phase_ids[i]).unwrap();
        let next = grid.subgraph_bounds.get(phase_ids[i + 1]).unwrap();

        assert!(
            next.1 >= curr.1 + curr.3 + INTER_SUBGRAPH_SPACING - 1e-3,
            "Phase {} bottom ({:.1}) overlaps with Phase {} top ({:.1})",
            phase_ids[i],
            curr.1 + curr.3,
            phase_ids[i + 1],
            next.1
        );
    }
}

#[test]
fn test_phase_stacking_with_long_bypass_edge() {
    let nodes = to_s(&["A1", "B1", "C1"]);
    let edges = to_edges(&[("A1", "B1"), ("B1", "C1"), ("A1", "C1")]);
    let layers = to_layers(&[&["A1"], &["B1"], &["C1"]]);
    let subgraphs = vec![
        make_subgraph("G1", Some("Phase 1"), &["A1"]),
        make_subgraph("G2", Some("Phase 2"), &["B1"]),
        make_subgraph("G3", Some("Phase 3"), &["C1"]),
    ];

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &subgraphs);
    let grid = LayoutGrid::compute(&layers, &subgraphs, &analyzer, 800, 600);

    let g1 = grid.subgraph_bounds.get("G1").unwrap();
    let g2 = grid.subgraph_bounds.get("G2").unwrap();
    let g3 = grid.subgraph_bounds.get("G3").unwrap();

    assert!(g2.1 >= g1.1 + g1.3 + INTER_SUBGRAPH_SPACING - 1e-3);
    assert!(g3.1 >= g2.1 + g2.3 + INTER_SUBGRAPH_SPACING - 1e-3);
    assert_pairwise_node_collision_free(&grid.node_positions);
}

#[test]
fn test_order_flow_realworld_geometric_invariants() {
    let mmd_path = "examples/order_flow.mmd";
    let content = std::fs::read_to_string(mmd_path).expect("order_flow.mmd must exist");
    let svg = render(&content).expect("order_flow.mmd must render cleanly");

    let height = svg
        .find("height=\"")
        .and_then(|idx| {
            let start = idx + 8;
            let end = svg[start..].find('"')? + start;
            svg[start..end].parse::<f32>().ok()
        })
        .unwrap();

    assert!(
        height <= 3300.0,
        "Generated SVG height ({:.1}px) must satisfy compression target <= 3300px",
        height
    );

    let rects = extract_rects(&svg);
    let node_rects: Vec<&(f32, f32, f32, f32, String)> = rects
        .iter()
        .filter(|r| (r.2 - NODE_WIDTH).abs() < 1.0 && (r.3 - NODE_HEIGHT).abs() < 1.0)
        .collect();

    assert!(
        node_rects.len() >= 25,
        "Must render at least 25 nodes in order_flow"
    );

    for i in 0..node_rects.len() {
        let (xa, ya, wa, ha, _) = *node_rects[i];
        for &&(xb, yb, wb, hb, _) in node_rects.iter().skip(i + 1) {
            let x_overlap = xa < xb + wb - 1e-3 && xb < xa + wa - 1e-3;
            let y_overlap = ya < yb + hb - 1e-3 && yb < ya + ha - 1e-3;
            assert!(
                !(x_overlap && y_overlap),
                "Order flow node collision between ({:.1}, {:.1}) and ({:.1}, {:.1})",
                xa,
                ya,
                xb,
                yb
            );
        }
    }
}

// ============================================================================
// SUITE 2: EMPIRICALLY CONFIRMED DEFECT & VULNERABILITY PROBES
// ============================================================================

/// Bug 1 Probe: Multi-lane fork slot corruption (k >= 4).
///
/// In `src/renderer/layout/grid.rs` line 344:
/// `if leads_to_join && slot < -1.0 { slot = -0.5; }`
/// overwrites the slot of any branch with slot < -1.0 (e.g. branch 0 in a 4-way fork,
/// which was assigned -1.5) to -0.5, colliding with branch 1!
#[test]
fn test_bug_wide_parallel_branches_4_lanes_slot_overwritten_to_minus_half() {
    let nodes = to_s(&["F", "B0", "B1", "B2", "B3", "J"]);
    let edges = to_edges(&[
        ("F", "B0"),
        ("F", "B1"),
        ("F", "B2"),
        ("F", "B3"),
        ("B0", "J"),
        ("B1", "J"),
        ("B2", "J"),
        ("B3", "J"),
    ]);
    let layers = to_layers(&[&["F"], &["B0", "B1", "B2", "B3"], &["J"]]);
    let subgraphs = vec![make_subgraph(
        "SG_4",
        Some("4 Lanes"),
        &["B0", "B1", "B2", "B3"],
    )];

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &subgraphs);
    let grid = LayoutGrid::compute(&layers, &subgraphs, &analyzer, 800, 600);

    let b0_x = grid.node_positions.get("B0").unwrap().0;
    let b1_x = grid.node_positions.get("B1").unwrap().0;
    let b2_x = grid.node_positions.get("B2").unwrap().0;
    let b3_x = grid.node_positions.get("B3").unwrap().0;

    let center_x = grid.effective_width / 2.0;
    let col_w = NODE_WIDTH + HORIZONTAL_SPACING;

    // Expected symmetric slots: B0 = -1.5, B1 = -0.5, B2 = +0.5, B3 = +1.5
    let expected_b0_x = center_x + (-1.5) * col_w - NODE_WIDTH / 2.0;
    let expected_b1_x = center_x + (-0.5) * col_w - NODE_WIDTH / 2.0;

    assert_eq!(
        b0_x, expected_b0_x,
        "CRITICAL BUG: B0 slot -1.5 was overwritten to -0.5 by line 344 in grid.rs! b0_x={:.1}, expected={:.1}",
        b0_x, expected_b0_x
    );
    assert_eq!(
        b1_x, expected_b1_x,
        "B1 slot -0.5 corrupted! b1_x={:.1}, expected={:.1}",
        b1_x, expected_b1_x
    );
    assert!(
        b0_x < b1_x && b1_x < b2_x && b2_x < b3_x,
        "Lane ordering must be strictly preserved"
    );
}

/// Bug 2 Probe: Lane order inversion in 5-way branches.
///
/// In a 5-way fork, branch 0 (slot -2.0) has its slot overwritten to -0.5,
/// causing branch 0 to jump between branch 1 (slot -1.0) and branch 2 (slot 0.0),
/// rendering branches out-of-order: B2, B1, B3, B4, B5.
#[test]
fn test_bug_wide_parallel_branches_5_lanes_lane_inversion() {
    let branch_names = ["B1", "B2", "B3", "B4", "B5"];
    let mut node_list = vec!["F"];
    node_list.extend_from_slice(&branch_names);
    node_list.push("J");
    let nodes = to_s(&node_list);

    let mut edge_tuples = Vec::new();
    for &b in &branch_names {
        edge_tuples.push(("F", b));
        edge_tuples.push((b, "J"));
    }
    let edges = to_edges(&edge_tuples);
    let layers = to_layers(&[&["F"], &branch_names, &["J"]]);
    let subgraphs = vec![make_subgraph("SG_5", Some("5 Lanes"), &branch_names)];

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &subgraphs);
    let grid = LayoutGrid::compute(&layers, &subgraphs, &analyzer, 800, 600);

    let b1_x = grid.node_positions.get("B1").unwrap().0;
    let b2_x = grid.node_positions.get("B2").unwrap().0;
    let b3_x = grid.node_positions.get("B3").unwrap().0;
    let b4_x = grid.node_positions.get("B4").unwrap().0;
    let b5_x = grid.node_positions.get("B5").unwrap().0;

    // Branches must be monotonically ordered from left to right: B1 < B2 < B3 < B4 < B5
    assert!(
        b1_x < b2_x,
        "CRITICAL BUG: B1 ({:.1}) was positioned to the right of B2 ({:.1})! Lane ordering inverted due to slot overwrite at line 344 in grid.rs",
        b1_x, b2_x
    );
    assert!(b2_x < b3_x && b3_x < b4_x && b4_x < b5_x);

    let sg = grid.subgraph_bounds.get("SG_5").unwrap();
    assert_eq!(
        sg.2, 850.0,
        "5-lane subgraph width must be 850px, but found {:.1} due to collision inflation",
        sg.2
    );
}

/// Bug 3 Probe: 8-way fork cascading collision and width inflation.
///
/// In an 8-way fork, branches 0, 1, 2 all have slot < -1.0, so all 3 slots
/// are overwritten to -0.5, collapsing into a 4-way collision with branch 3
/// and inflating subgraph width to 1870px instead of 1360px.
#[test]
fn test_bug_wide_parallel_branches_8_lanes_width_and_symmetry() {
    let branch_names = ["B1", "B2", "B3", "B4", "B5", "B6", "B7", "B8"];
    let mut node_list = vec!["F"];
    node_list.extend_from_slice(&branch_names);
    node_list.push("J");
    let nodes = to_s(&node_list);

    let mut edge_tuples = Vec::new();
    for &b in &branch_names {
        edge_tuples.push(("F", b));
        edge_tuples.push((b, "J"));
    }
    let edges = to_edges(&edge_tuples);
    let layers = to_layers(&[&["F"], &branch_names, &["J"]]);
    let subgraphs = vec![make_subgraph("SG_8", Some("8 Lanes"), &branch_names)];

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &subgraphs);
    let grid = LayoutGrid::compute(&layers, &subgraphs, &analyzer, 800, 600);

    let sg = grid.subgraph_bounds.get("SG_8").unwrap();
    let expected_width =
        SubgraphColumnGrid::compute_width(8, NODE_WIDTH, HORIZONTAL_SPACING, SUBGRAPH_PADDING_X);
    assert_eq!(expected_width, 1360.0);
    assert_eq!(
        sg.2, 1360.0,
        "CRITICAL BUG: 8-lane subgraph width exploded to {:.1}px (expected 1360px) due to cascading collisions from line 344",
        sg.2
    );
}

/// Bug 4 Probe: Unconditional slot corruption of left-branch leaf nodes.
///
/// In lines 346-358:
/// `else if branch_analyzer.forward_neighbors(nid).is_none_or(|n| n.is_empty()) && slot.abs() < 1.0 ... { slot = 0.5; }`
/// forces any leaf node with |slot| < 1.0 (such as a failure exit on the left branch, slot -0.5)
/// to jump across to slot +0.5, colliding with the right branch and displacing it!
#[test]
fn test_bug_leaf_node_on_left_branch_lane_continuity() {
    let nodes = to_s(&[
        "Outside", "Fork", "Left", "Right", "Fail", "Success", "Join",
    ]);
    let edges = to_edges(&[
        ("Fork", "Left"),
        ("Fork", "Right"),
        ("Left", "Fail"),
        ("Right", "Success"),
        ("Outside", "Join"),
        ("Success", "Join"),
    ]);
    let layers = to_layers(&[
        &["Outside", "Fork"],
        &["Left", "Right"],
        &["Fail", "Success"],
        &["Join"],
    ]);
    let subgraphs = vec![make_subgraph(
        "SG_Leaf",
        Some("Leaf Test"),
        &["Left", "Right", "Fail", "Success"],
    )];

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &subgraphs);
    let grid = LayoutGrid::compute(&layers, &subgraphs, &analyzer, 800, 600);

    let fail_x = grid.node_positions.get("Fail").unwrap().0;
    let left_x = grid.node_positions.get("Left").unwrap().0;

    assert_eq!(
        fail_x, left_x,
        "CRITICAL BUG: Fail node on Left branch (slot -0.5) jumped to Right column (slot +0.5, x={:.1}) due to line 357 in grid.rs! Expected left_x={:.1}",
        fail_x, left_x
    );
}

/// Bug 5 Probe: Independent / parallel subgraphs displace nodes outside bounding box.
///
/// In `src/renderer/layout/grid.rs` lines 564-568:
/// `for pos in node_positions.values_mut() { if pos.1 >= next_y - 1e-4 { pos.1 += shift_y; } }`
/// shifts nodes based on a global Y threshold without checking subgraph ownership.
/// When two subgraphs start at layer 0 without inter-subgraph edges, SG1's nodes are shifted
/// down into SG2 while SG1's bounding box remains empty at the top of the canvas.
#[test]
fn test_bug_independent_subgraphs_do_not_displace_nodes_outside_bounds() {
    let nodes = to_s(&["A1", "A2", "B1", "B2"]);
    let edges = to_edges(&[("A1", "A2"), ("B1", "B2")]);
    let layers = to_layers(&[&["A1", "B1"], &["A2", "B2"]]);
    let subgraphs = vec![
        make_subgraph("SG1", Some("Independent 1"), &["A1", "A2"]),
        make_subgraph("SG2", Some("Independent 2"), &["B1", "B2"]),
    ];

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &subgraphs);
    let grid = LayoutGrid::compute(&layers, &subgraphs, &analyzer, 800, 600);

    let sg1 = *grid.subgraph_bounds.get("SG1").expect("SG1 bounds exist");
    assert_subgraph_encloses_nodes("SG1", sg1, &["A1", "A2"], &grid.node_positions, true);
}

/// Bug 6 Probe: Hardcoded test-case node ID "C5" causes inconsistent width and void gaps.
///
/// In `src/renderer/layout/grid.rs` line 176:
/// `if n == "C5" && slot < -1.0 { slot = -0.5; }`
/// specifically checks string literal "C5". If the node is named "C5",
/// line 176 does not fire, causing SG dynamic width to be calculated as 510px (3 lanes)
/// while node positioning uses 340px (2 lanes), leaving a dead blank void column.
#[test]
fn test_bug_hardcoded_node_name_c5_inconsistency() {
    // Full G3 payment topology with node renamed from C5 to C5
    let nodes = to_s(&[
        "C1", "C2", "C3", "C4", "C5", "C6", "C7", "C8", "C9", "C10", "D1",
    ]);
    let edges = to_edges(&[
        ("C1", "C2"),
        ("C2", "C3"),
        ("C2", "C9"),
        ("C3", "C4"),
        ("C4", "C10"),
        ("C10", "C4"),
        ("C4", "C6"),
        ("C6", "C7"),
        ("C7", "C3"),
        ("C7", "C8"),
        ("C4", "C5"),
        ("C5", "D1"),
        ("C9", "D1"),
    ]);
    let layers = to_layers(&[
        &["C1"],
        &["C2"],
        &["C3", "C9"],
        &["C4"],
        &["C10", "C6", "C5"],
        &["C7"],
        &["C8", "D1"],
    ]);

    let subgraphs = vec![make_subgraph(
        "G3",
        Some("Payment"),
        &["C1", "C2", "C3", "C4", "C5", "C6", "C7", "C8", "C9", "C10"],
    )];

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &subgraphs);
    let grid = LayoutGrid::compute(&layers, &subgraphs, &analyzer, 800, 600);

    let sg = grid.subgraph_bounds.get("G3").unwrap();
    eprintln!("Full G3 with C5 width: {}", sg.2);
    assert_eq!(
        sg.2, 595.0,
        "G3 dynamic width must be 595px to accommodate parallel lanes without void gaps (observed: {:.1}px)",
        sg.2
    );
}

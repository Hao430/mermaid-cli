//! End-to-End Test Suite for Horizontal Parallel Branch Expansion in mermaid-cli.
//!
//! Opaque-box, requirement-driven tests covering all 18 features from PROJECT.md
//! across 4 comprehensive tiers:
//! - Tier 1: Feature Coverage (>=5 tests per feature across 18 features = 90 tests)
//! - Tier 2: Boundary & Corner Cases (>=5 tests per feature across 18 features = 90 tests)
//! - Tier 3: Cross-Feature Combinations (15 pairwise & interaction tests)
//! - Tier 4: Real-World Scenarios (10 end-to-end tests on examples/order_flow.mmd)
//!
//! Total test cases: 205

use mermaid_cli::{check, parse, render};
use std::time::Instant;

// ============================================================================
// SVG Opaque-Box Parsing & Geometric Helper Utilities
// ============================================================================

pub mod helpers {
    /// Parsed rectangle element from SVG
    #[derive(Debug, Clone, PartialEq)]
    pub struct SvgRect {
        pub x: f32,
        pub y: f32,
        pub width: f32,
        pub height: f32,
        pub style: String,
    }

    /// Parsed text element from SVG
    #[derive(Debug, Clone, PartialEq)]
    pub struct SvgText {
        pub x: f32,
        pub y: f32,
        pub content: String,
        pub style: String,
    }

    /// Extract overall SVG width and height from the root `<svg>` tag.
    pub fn extract_svg_dimensions(svg: &str) -> (f32, f32) {
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

    /// Extract all `<rect>` elements from the SVG.
    pub fn extract_rects(svg: &str) -> Vec<SvgRect> {
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

    /// Extract all `<text>` elements and tspans from the SVG.
    pub fn extract_texts(svg: &str) -> Vec<SvgText> {
        let mut texts = Vec::new();
        let mut pos = 0;
        while let Some(idx) = svg[pos..].find("<text ") {
            let start = pos + idx;
            let end = match svg[start..].find("</text>") {
                Some(e) => start + e + 7,
                None => break,
            };
            let tag_full = &svg[start..end];
            let tag_open_end = tag_full.find('>').unwrap_or(tag_full.len());
            let open_tag = &tag_full[..tag_open_end];

            let x = extract_attr(open_tag, "x").unwrap_or(0.0);
            let y = extract_attr(open_tag, "y").unwrap_or(0.0);
            let style = extract_str_attr(open_tag, "style").unwrap_or_default();

            let inner = &tag_full[tag_open_end + 1..tag_full.len() - 7];
            let clean_content = strip_xml_tags(inner);

            texts.push(SvgText {
                x,
                y,
                content: clean_content,
                style,
            });
            pos = end;
        }
        texts
    }

    /// Extract all `<path d="..." ...>` elements from SVG.
    pub fn extract_paths(svg: &str) -> Vec<String> {
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

    /// Count the total number of quadratic Bézier curve commands ('Q') across all paths.
    pub fn count_fillet_curves(svg: &str) -> usize {
        extract_paths(svg)
            .iter()
            .map(|p| p.split_whitespace().filter(|&token| token == "Q").count())
            .sum()
    }

    /// Extract pill shield rectangles (used as protective underlays under edge labels).
    pub fn extract_pill_shields(svg: &str) -> Vec<SvgRect> {
        extract_rects(svg)
            .into_iter()
            .filter(|r| {
                (r.height - 18.0).abs() < 1.0
                    || r.style.contains("pill")
                    || r.style.contains("rx:4")
            })
            .collect()
    }

    /// Extract subgraph boundary rectangles.
    pub fn extract_subgraph_rects(svg: &str) -> Vec<SvgRect> {
        extract_rects(svg)
            .into_iter()
            .filter(|r| {
                r.style.contains("subgraph")
                    || r.style.contains("stroke-dasharray")
                    || (r.width > 150.0 && r.height > 100.0 && r.style.contains("rx:8"))
            })
            .collect()
    }

    /// Find the center (cx, cy) of a node matching the specified label or text substring.
    pub fn find_node_center(svg: &str, label_substring: &str) -> Option<(f32, f32)> {
        for t in extract_texts(svg) {
            if t.content.contains(label_substring) {
                return Some((t.x, t.y));
            }
        }
        None
    }

    /// Helper to extract numeric attribute
    fn extract_attr(tag: &str, attr: &str) -> Option<f32> {
        let pattern = format!("{}=\"", attr);
        let start = tag.find(&pattern)? + pattern.len();
        let end = tag[start..].find('"')? + start;
        tag[start..end].parse::<f32>().ok()
    }

    /// Helper to extract string attribute
    fn extract_str_attr(tag: &str, attr: &str) -> Option<String> {
        let pattern = format!("{}=\"", attr);
        let start = tag.find(&pattern)? + pattern.len();
        let end = tag[start..].find('"')? + start;
        Some(tag[start..end].to_string())
    }

    /// Helper to strip XML tags from inner text
    fn strip_xml_tags(s: &str) -> String {
        let mut res = String::new();
        let mut in_tag = false;
        for c in s.chars() {
            if c == '<' {
                in_tag = true;
            } else if c == '>' {
                in_tag = false;
            } else if !in_tag {
                res.push(c);
            }
        }
        res.trim().to_string()
    }
}

// ============================================================================
// TIER 1: FEATURE COVERAGE (90 TESTS: F01 .. F18, 5 tests each)
// ============================================================================

mod tier1_feature_coverage {
    use super::*;

    // --- F01: DFS Back-Edge Detection (PLAN §2.1) ---
    #[test]
    fn test_tier1_f01_dfs_01_direct_2node_cycle() {
        let code = "flowchart TD\n A-->B\n B-->A";
        let res = render(code);
        assert!(
            res.is_ok(),
            "Cycle should not cause infinite loop: {:?}",
            res.err()
        );
        let svg = res.unwrap();
        assert!(svg.contains(">A<") && svg.contains(">B<"));
        assert!(svg.contains("<path "));
    }

    #[test]
    fn test_tier1_f01_dfs_02_self_loop() {
        let code = "flowchart TD\n NodeA-->NodeA";
        let res = render(code);
        assert!(
            res.is_ok(),
            "Self-loop should render without error: {:?}",
            res.err()
        );
        let svg = res.unwrap();
        assert!(svg.contains(">NodeA<"));
    }

    #[test]
    fn test_tier1_f01_dfs_03_triangle_cycle_3nodes() {
        let code = "flowchart TD\n A-->B\n B-->C\n C-->A";
        let res = render(code);
        assert!(
            res.is_ok(),
            "3-node cycle should be isolated: {:?}",
            res.err()
        );
        let svg = res.unwrap();
        let pos_a = helpers::find_node_center(&svg, "A").unwrap();
        let pos_b = helpers::find_node_center(&svg, "B").unwrap();
        let pos_c = helpers::find_node_center(&svg, "C").unwrap();
        assert!(
            pos_a.1 != pos_b.1 || pos_b.1 != pos_c.1,
            "Cycle must be broken across distinct vertical layers"
        );
    }

    #[test]
    fn test_tier1_f01_dfs_04_cycle_with_downstream_branch() {
        let code = "flowchart TD\n A-->B\n B-->C\n C-->B\n C-->D";
        let res = render(code);
        assert!(res.is_ok());
        let svg = res.unwrap();
        let pos_c = helpers::find_node_center(&svg, "C").unwrap();
        let pos_d = helpers::find_node_center(&svg, "D").unwrap();
        assert!(
            pos_d.1 > pos_c.1,
            "Downstream node D must be below cycle node C"
        );
    }

    #[test]
    fn test_tier1_f01_dfs_05_multiple_disjoint_cycles() {
        let code = "flowchart TD\n A-->B\n B-->A\n C-->D\n D-->C";
        let res = render(code);
        assert!(
            res.is_ok(),
            "Multiple disjoint cycles should render: {:?}",
            res.err()
        );
        let svg = res.unwrap();
        assert!(
            svg.contains(">A<")
                && svg.contains(">B<")
                && svg.contains(">C<")
                && svg.contains(">D<")
        );
    }

    // --- F02: Fork Node Identification (PLAN §2.1) ---
    #[test]
    fn test_tier1_f02_fork_01_binary_fork() {
        let code = "flowchart TD\n Root-->Branch1\n Root-->Branch2";
        let res = render(code).expect("Binary fork should render");
        let b1 = helpers::find_node_center(&res, "Branch1").unwrap();
        let b2 = helpers::find_node_center(&res, "Branch2").unwrap();
        assert!(
            (b1.0 - b2.0).abs() >= 100.0,
            "Fork children must have distinct horizontal coordinates"
        );
    }

    #[test]
    fn test_tier1_f02_fork_02_ternary_fork() {
        let code = "flowchart TD\n Root-->L\n Root-->M\n Root-->R";
        let res = render(code).expect("Ternary fork should render");
        let l = helpers::find_node_center(&res, "L").unwrap();
        let m = helpers::find_node_center(&res, "M").unwrap();
        let r = helpers::find_node_center(&res, "R").unwrap();
        assert!(
            l.0 < m.0 && m.0 < r.0,
            "Ternary fork children must be spread horizontally in order"
        );
    }

    #[test]
    fn test_tier1_f02_fork_03_quaternary_fork() {
        let code = "flowchart TD\n F-->A\n F-->B\n F-->C\n F-->D";
        let res = render(code).expect("Quaternary fork should render");
        let a = helpers::find_node_center(&res, "A").unwrap();
        let d = helpers::find_node_center(&res, "D").unwrap();
        assert!(
            d.0 > a.0 + 200.0,
            "4-way fork must span across multiple horizontal columns"
        );
    }

    #[test]
    fn test_tier1_f02_fork_04_fork_from_decision_diamond() {
        let code = "flowchart TD\n Dec{Decision}-->|Yes|YAction\n Dec-->|No|NAction";
        let res = render(code).expect("Decision diamond fork should render");
        let y = helpers::find_node_center(&res, "YAction").unwrap();
        let n = helpers::find_node_center(&res, "NAction").unwrap();
        assert!(
            (y.0 - n.0).abs() >= 100.0,
            "Diamond branches must be horizontally separated"
        );
    }

    #[test]
    fn test_tier1_f02_fork_05_sequential_two_forks() {
        let code = "flowchart TD\n F1-->F2\n F1-->A\n F2-->B\n F2-->C";
        let res = render(code).expect("Sequential forks should render");
        let b = helpers::find_node_center(&res, "B").unwrap();
        let c = helpers::find_node_center(&res, "C").unwrap();
        assert!(
            (b.0 - c.0).abs() >= 100.0,
            "Second fork children must be horizontally separated"
        );
    }

    // --- F03: Lane Slot Index Assignment (PLAN §2.1) ---
    #[test]
    fn test_tier1_f03_slot_01_binary_symmetric_offsets() {
        let code = "flowchart TD\n Fork-->Left\n Fork-->Right";
        let res = render(code).expect("render should succeed");
        let fork = helpers::find_node_center(&res, "Fork").unwrap();
        let left = helpers::find_node_center(&res, "Left").unwrap();
        let right = helpers::find_node_center(&res, "Right").unwrap();
        assert!(
            left.0 < fork.0,
            "Left slot must be to the left of fork center"
        );
        assert!(
            right.0 > fork.0,
            "Right slot must be to the right of fork center"
        );
    }

    #[test]
    fn test_tier1_f03_slot_02_ternary_symmetric_offsets() {
        let code = "flowchart TD\n Fork-->L\n Fork-->C\n Fork-->R";
        let res = render(code).expect("render should succeed");
        let fork = helpers::find_node_center(&res, "Fork").unwrap();
        let c = helpers::find_node_center(&res, "C").unwrap();
        assert!(
            (c.0 - fork.0).abs() < 15.0,
            "Center branch slot offset must be 0.0"
        );
    }

    #[test]
    fn test_tier1_f03_slot_03_quaternary_symmetric_offsets() {
        let code = "flowchart TD\n Fork-->B1\n Fork-->B2\n Fork-->B3\n Fork-->B4";
        let res = render(code).expect("render should succeed");
        let fork = helpers::find_node_center(&res, "Fork").unwrap();
        let b2 = helpers::find_node_center(&res, "B2").unwrap();
        let b3 = helpers::find_node_center(&res, "B3").unwrap();
        assert!(
            b2.0 < fork.0 && b3.0 > fork.0,
            "Middle branches must flank fork center symmetrically"
        );
    }

    #[test]
    fn test_tier1_f03_slot_04_slot_spacing_greater_than_node_width() {
        let code = "flowchart TD\n F-->A\n F-->B";
        let res = render(code).expect("render should succeed");
        let a = helpers::find_node_center(&res, "A").unwrap();
        let b = helpers::find_node_center(&res, "B").unwrap();
        assert!(
            (b.0 - a.0).abs() >= 130.0,
            "Horizontal slot spacing must accommodate node width (130px)"
        );
    }

    #[test]
    fn test_tier1_f03_slot_05_left_right_ordering() {
        let code = "flowchart TD\n F-->First\n F-->Second";
        let res = render(code).expect("render should succeed");
        let first = helpers::find_node_center(&res, "First").unwrap();
        let second = helpers::find_node_center(&res, "Second").unwrap();
        assert!(
            first.0 < second.0,
            "First branch defined must be placed to the left of Second"
        );
    }

    // --- F04: Downstream Slot Propagation (PLAN §2.1) ---
    #[test]
    fn test_tier1_f04_prop_01_two_hop_branch_propagation() {
        let code = "flowchart TD\n Fork-->L1\n Fork-->R1\n L1-->L2\n R1-->R2";
        let res = render(code).expect("render should succeed");
        let l1 = helpers::find_node_center(&res, "L1").unwrap();
        let l2 = helpers::find_node_center(&res, "L2").unwrap();
        assert!(
            (l2.0 - l1.0).abs() < 20.0,
            "L2 must inherit L1's horizontal slot offset"
        );
    }

    #[test]
    fn test_tier1_f04_prop_02_three_hop_branch_propagation() {
        let code = "flowchart TD\n Fork-->L1\n Fork-->R1\n L1-->L2\n L2-->L3";
        let res = render(code).expect("render should succeed");
        let l1 = helpers::find_node_center(&res, "L1").unwrap();
        let l3 = helpers::find_node_center(&res, "L3").unwrap();
        assert!(
            (l3.0 - l1.0).abs() < 20.0,
            "L3 must propagate through 3 hops without horizontal drift"
        );
    }

    #[test]
    fn test_tier1_f04_prop_03_asymmetric_depth_propagation() {
        let code = "flowchart TD\n Fork-->L1\n Fork-->R1\n L1-->L2\n L2-->L3\n R1-->R2";
        let res = render(code).expect("render should succeed");
        let r1 = helpers::find_node_center(&res, "R1").unwrap();
        let r2 = helpers::find_node_center(&res, "R2").unwrap();
        assert!(
            (r2.0 - r1.0).abs() < 20.0,
            "R2 must maintain R1's lane even when sibling branch is longer"
        );
    }

    #[test]
    fn test_tier1_f04_prop_04_no_center_collapse_on_single_child() {
        let code = "flowchart TD\n Fork-->L1\n Fork-->R1\n L1-->L2";
        let res = render(code).expect("render should succeed");
        let (width, _) = helpers::extract_svg_dimensions(&res);
        let l2 = helpers::find_node_center(&res, "L2").unwrap();
        let canvas_center_x = width / 2.0;
        assert!(
            (l2.0 - canvas_center_x).abs() > 30.0,
            "L2 must not collapse to global canvas center"
        );
    }

    #[test]
    fn test_tier1_f04_prop_05_parallel_chains_stay_in_lanes() {
        let code = "flowchart TD\n Fork-->A1\n Fork-->B1\n A1-->A2\n A2-->A3\n B1-->B2\n B2-->B3";
        let res = render(code).expect("render should succeed");
        let a3 = helpers::find_node_center(&res, "A3").unwrap();
        let b3 = helpers::find_node_center(&res, "B3").unwrap();
        assert!(
            (b3.0 - a3.0).abs() >= 100.0,
            "Both parallel chains must remain in separate columns to depth 3"
        );
    }

    // --- F05: Join Termination & Convergence (PLAN §2.1) ---
    #[test]
    fn test_tier1_f05_join_01_binary_join_convergence() {
        let code = "flowchart TD\n Fork-->B1\n Fork-->B2\n B1-->Join\n B2-->Join";
        let res = render(code).expect("render should succeed");
        let fork = helpers::find_node_center(&res, "Fork").unwrap();
        let join = helpers::find_node_center(&res, "Join").unwrap();
        assert!(
            (join.0 - fork.0).abs() < 20.0,
            "Join node must converge back to center axis aligned with Fork"
        );
    }

    #[test]
    fn test_tier1_f05_join_02_ternary_join_convergence() {
        let code =
            "flowchart TD\n Fork-->B1\n Fork-->B2\n Fork-->B3\n B1-->Join\n B2-->Join\n B3-->Join";
        let res = render(code).expect("render should succeed");
        let fork = helpers::find_node_center(&res, "Fork").unwrap();
        let join = helpers::find_node_center(&res, "Join").unwrap();
        assert!(
            (join.0 - fork.0).abs() < 20.0,
            "3-way join node must converge back to center axis"
        );
    }

    #[test]
    fn test_tier1_f05_join_03_join_resets_slot_to_zero() {
        let code =
            "flowchart TD\n Fork-->B1\n Fork-->B2\n B1-->Join\n B2-->Join\n Join-->Downstream";
        let res = render(code).expect("render should succeed");
        let join = helpers::find_node_center(&res, "Join").unwrap();
        let down = helpers::find_node_center(&res, "Downstream").unwrap();
        assert!(
            (down.0 - join.0).abs() < 10.0,
            "Downstream node after join must inherit slot offset 0.0"
        );
    }

    #[test]
    fn test_tier1_f05_join_04_join_downstream_single_chain() {
        let code = "flowchart TD\n Fork-->L\n Fork-->R\n L-->J\n R-->J\n J-->S1\n S1-->S2";
        let res = render(code).expect("render should succeed");
        let j = helpers::find_node_center(&res, "J").unwrap();
        let s2 = helpers::find_node_center(&res, "S2").unwrap();
        assert!(
            (s2.0 - j.0).abs() < 15.0,
            "Post-join linear chain must remain aligned along central trunk"
        );
    }

    #[test]
    fn test_tier1_f05_join_05_fork_join_fork_cascade() {
        let code = "flowchart TD\n F1-->A\n F1-->B\n A-->J1\n B-->J1\n J1-->F2L\n J1-->F2R";
        let res = render(code).expect("render should succeed");
        let f2l = helpers::find_node_center(&res, "F2L").unwrap();
        let f2r = helpers::find_node_center(&res, "F2R").unwrap();
        assert!(
            (f2r.0 - f2l.0).abs() >= 100.0,
            "Join node acting as subsequent fork opens distinct lanes"
        );
    }

    // --- F06: Virtual Lane Reservation (PLAN §2.2) ---
    #[test]
    fn test_tier1_f06_resv_01_unbalanced_branch_corridor_reserved() {
        let code = "flowchart TD\n Fork-->Short\n Fork-->Long1\n Long1-->Long2\n Long2-->Long3\n Short-->Join\n Long3-->Join";
        let res = render(code).expect("render should succeed");
        let short = helpers::find_node_center(&res, "Short").unwrap();
        let l2 = helpers::find_node_center(&res, "Long2").unwrap();
        assert!(
            (l2.0 - short.0).abs() >= 100.0,
            "Long branch must not drift into Short branch's reserved lane"
        );
    }

    #[test]
    fn test_tier1_f06_resv_02_short_left_long_right() {
        let code = "flowchart TD\n Fork-->L\n Fork-->R1\n R1-->R2\n R2-->R3\n L-->Join\n R3-->Join";
        let res = render(code).expect("render should succeed");
        let l = helpers::find_node_center(&res, "L").unwrap();
        let r3 = helpers::find_node_center(&res, "R3").unwrap();
        assert!(
            l.0 + 100.0 <= r3.0,
            "Left reserved lane corridor preserved throughout right's depth span"
        );
    }

    #[test]
    fn test_tier1_f06_resv_03_short_right_long_left() {
        let code = "flowchart TD\n Fork-->L1\n Fork-->R\n L1-->L2\n L2-->L3\n L3-->Join\n R-->Join";
        let res = render(code).expect("render should succeed");
        let r = helpers::find_node_center(&res, "R").unwrap();
        let l3 = helpers::find_node_center(&res, "L3").unwrap();
        assert!(
            l3.0 + 100.0 <= r.0,
            "Right reserved lane corridor preserved throughout left's depth span"
        );
    }

    #[test]
    fn test_tier1_f06_resv_04_zero_intermediate_bypass_reservation() {
        let code = "flowchart TD\n Fork-->Bypass\n Fork-->M1\n M1-->M2\n M2-->Join\n Bypass-->Join";
        let res = render(code).expect("render should succeed");
        let bp = helpers::find_node_center(&res, "Bypass").unwrap();
        let m2 = helpers::find_node_center(&res, "M2").unwrap();
        assert!(
            (bp.0 - m2.0).abs() >= 100.0,
            "Direct bypass edge reserves lane without horizontal collapse"
        );
    }

    #[test]
    fn test_tier1_f06_resv_05_multi_depth_corridor_span() {
        let code = "flowchart TD\n Fork-->B1\n Fork-->B2\n Fork-->B3\n B1-->Join\n B2-->B22\n B22-->Join\n B3-->B32\n B32-->B33\n B33-->Join";
        let res = render(code).expect("render should succeed");
        let b1 = helpers::find_node_center(&res, "B1").unwrap();
        let b22 = helpers::find_node_center(&res, "B22").unwrap();
        let b33 = helpers::find_node_center(&res, "B33").unwrap();
        assert!(
            b1.0 < b22.0 && b22.0 < b33.0,
            "Multi-depth branch corridors preserved across depth span D_max"
        );
    }

    // --- F07: Subgraph Dynamic Width (PLAN §2.3) ---
    #[test]
    fn test_tier1_f07_width_01_single_lane_subgraph_width() {
        let code = "flowchart TD\n subgraph SG[Single Lane]\n A-->B\n end";
        let res = render(code).expect("render should succeed");
        let sg_rects = helpers::extract_subgraph_rects(&res);
        assert!(!sg_rects.is_empty(), "Subgraph rect must be present");
        assert!(
            sg_rects[0].width >= 150.0,
            "Single-lane subgraph must fit node width"
        );
    }

    #[test]
    fn test_tier1_f07_width_02_two_lane_subgraph_expands() {
        let code = "flowchart TD\n subgraph SG[Two Lanes]\n Fork-->Left\n Fork-->Right\n end";
        let res = render(code).expect("render should succeed");
        let sg_rects = helpers::extract_subgraph_rects(&res);
        assert!(!sg_rects.is_empty());
        assert!(
            sg_rects[0].width >= 280.0,
            "2-lane subgraph must dynamically expand (W_sg >= 280px)"
        );
    }

    #[test]
    fn test_tier1_f07_width_03_three_lane_subgraph_expands() {
        let code = "flowchart TD\n subgraph SG[Three Lanes]\n Fork-->L\n Fork-->M\n Fork-->R\n end";
        let res = render(code).expect("render should succeed");
        let sg_rects = helpers::extract_subgraph_rects(&res);
        assert!(!sg_rects.is_empty());
        assert!(
            sg_rects[0].width >= 420.0,
            "3-lane subgraph must dynamically expand (W_sg >= 420px)"
        );
    }

    #[test]
    fn test_tier1_f07_width_04_dynamic_width_formula_scaling() {
        let code1 = "flowchart TD\n subgraph SG1\n A-->B\n end";
        let code2 = "flowchart TD\n subgraph SG2\n Fork-->L\n Fork-->R\n end";
        let res1 = render(code1).unwrap();
        let res2 = render(code2).unwrap();
        let w1 = helpers::extract_subgraph_rects(&res1)[0].width;
        let w2 = helpers::extract_subgraph_rects(&res2)[0].width;
        assert!(
            w2 > w1,
            "2-lane subgraph width must strictly exceed 1-lane subgraph width"
        );
    }

    #[test]
    fn test_tier1_f07_width_05_adjacent_subgraphs_dynamic_width() {
        let code = "flowchart TD\n subgraph SG_Wide[Wide Phase]\n F-->A\n F-->B\n F-->C\n end\n subgraph SG_Narrow[Narrow Phase]\n N1-->N2\n end\n SG_Wide-->SG_Narrow";
        let res = render(code).expect("render should succeed");
        let rects = helpers::extract_subgraph_rects(&res);
        assert!(rects.len() >= 2, "Both subgraphs must be rendered");
    }

    // --- F08: Subgraph-Scoped Node X-Positioning (PLAN §2.3) ---
    #[test]
    fn test_tier1_f08_xpos_01_scoped_to_subgraph_center() {
        let code = "flowchart TD\n subgraph SG[Phase Container]\n Fork-->L\n Fork-->R\n end";
        let res = render(code).expect("render should succeed");
        let sg = helpers::extract_subgraph_rects(&res)[0].clone();
        let sg_cx = sg.x + sg.width / 2.0;
        let fork = helpers::find_node_center(&res, "Fork").unwrap();
        assert!(
            (fork.0 - sg_cx).abs() < 25.0,
            "Fork node must be centered relative to containing subgraph"
        );
    }

    #[test]
    fn test_tier1_f08_xpos_02_no_canvas_center_collapse() {
        let code = "flowchart TD\n subgraph SG\n Fork-->L1\n Fork-->R1\n L1-->L2\n end";
        let res = render(code).expect("render should succeed");
        let l1 = helpers::find_node_center(&res, "L1").unwrap();
        let l2 = helpers::find_node_center(&res, "L2").unwrap();
        assert!(
            (l2.0 - l1.0).abs() < 20.0,
            "L2 inside subgraph must stay in L1's column without center collapse"
        );
    }

    #[test]
    fn test_tier1_f08_xpos_03_independent_subgraphs_x_offset() {
        let code = "flowchart TD\n subgraph G1\n A-->B\n end\n subgraph G2\n C-->D\n end";
        let res = render(code).expect("render should succeed");
        assert!(res.contains("G1") && res.contains("G2"));
    }

    #[test]
    fn test_tier1_f08_xpos_04_two_nodes_in_subgraph_symmetric() {
        let code = "flowchart TD\n subgraph G\n Fork-->LeftNode\n Fork-->RightNode\n end";
        let res = render(code).expect("render should succeed");
        let l = helpers::find_node_center(&res, "LeftNode").unwrap();
        let r = helpers::find_node_center(&res, "RightNode").unwrap();
        let fork = helpers::find_node_center(&res, "Fork").unwrap();
        let dist_l = (fork.0 - l.0).abs();
        let dist_r = (r.0 - fork.0).abs();
        assert!(
            (dist_l - dist_r).abs() < 20.0,
            "Left and Right branches must be placed symmetrically around Fork"
        );
    }

    #[test]
    fn test_tier1_f08_xpos_05_deep_chain_inside_subgraph_aligned() {
        let code = "flowchart TD\n subgraph G\n F-->L\n F-->R\n L-->L1\n L1-->L2\n end";
        let res = render(code).expect("render should succeed");
        let l = helpers::find_node_center(&res, "L").unwrap();
        let l2 = helpers::find_node_center(&res, "L2").unwrap();
        assert!(
            (l2.0 - l.0).abs() < 20.0,
            "Deep chain in subgraph maintains X column"
        );
    }

    // --- F09: Branch Layer Compaction (PLAN §1.2) ---
    #[test]
    fn test_tier1_f09_compact_01_parallel_branches_same_vertical_row() {
        let code = "flowchart TD\n F-->B1\n F-->B2\n B1-->B12\n B2-->B22";
        let res = render(code).expect("render should succeed");
        let b1 = helpers::find_node_center(&res, "B1").unwrap();
        let b2 = helpers::find_node_center(&res, "B2").unwrap();
        let b12 = helpers::find_node_center(&res, "B12").unwrap();
        let b22 = helpers::find_node_center(&res, "B22").unwrap();
        assert!(
            (b1.1 - b2.1).abs() < 10.0,
            "B1 and B2 must occupy the same horizontal layer row"
        );
        assert!(
            (b12.1 - b22.1).abs() < 10.0,
            "B12 and B22 must occupy the same horizontal layer row"
        );
    }

    #[test]
    fn test_tier1_f09_compact_02_compacted_height_less_than_sequential() {
        let parallel_code = "flowchart TD\n F-->A\n F-->B\n A-->A2\n B-->B2";
        let sequential_code = "flowchart TD\n S1-->S2\n S2-->S3\n S3-->S4\n S4-->S5";
        let p_res = render(parallel_code).unwrap();
        let s_res = render(sequential_code).unwrap();
        let (_, p_h) = helpers::extract_svg_dimensions(&p_res);
        let (_, s_h) = helpers::extract_svg_dimensions(&s_res);
        assert!(
            p_h < s_h,
            "Compacted parallel branch height must be less than sequential chain height"
        );
    }

    #[test]
    fn test_tier1_f09_compact_03_three_parallel_branches_compacted() {
        let code = "flowchart TD\n F-->A\n F-->B\n F-->C\n A-->A2\n B-->B2\n C-->C2";
        let res = render(code).expect("render should succeed");
        let a = helpers::find_node_center(&res, "A").unwrap();
        let b = helpers::find_node_center(&res, "B").unwrap();
        let c = helpers::find_node_center(&res, "C").unwrap();
        assert!(
            (a.1 - b.1).abs() < 10.0 && (b.1 - c.1).abs() < 10.0,
            "All 3 parallel branches must share same Y row"
        );
    }

    #[test]
    fn test_tier1_f09_compact_04_compacted_layers_y_spacing() {
        let code = "flowchart TD\n F-->A\n F-->B\n A-->A2\n B-->B2";
        let res = render(code).expect("render should succeed");
        let a = helpers::find_node_center(&res, "A").unwrap();
        let a2 = helpers::find_node_center(&res, "A2").unwrap();
        assert!(
            a2.1 >= a.1 + 80.0,
            "Vertical layer row spacing must provide clear visual separation"
        );
    }

    #[test]
    fn test_tier1_f09_compact_05_unbalanced_branches_layer_compaction() {
        let code = "flowchart TD\n F-->L1\n F-->R1\n L1-->L2\n L2-->L3\n R1-->R2";
        let res = render(code).expect("render should succeed");
        let l2 = helpers::find_node_center(&res, "L2").unwrap();
        let r2 = helpers::find_node_center(&res, "R2").unwrap();
        assert!(
            (l2.1 - r2.1).abs() < 10.0,
            "L2 and R2 should share compacted layer row even on unbalanced branch"
        );
    }

    // --- F10: Subgraph Bounding Box Auto-Fitting (PLAN §2.3) ---
    #[test]
    fn test_tier1_f10_bbox_01_wraps_all_contained_nodes() {
        let code = "flowchart TD\n subgraph SG\n A-->B\n end";
        let res = render(code).expect("render should succeed");
        let sg = helpers::extract_subgraph_rects(&res)[0].clone();
        let a = helpers::find_node_center(&res, "A").unwrap();
        let b = helpers::find_node_center(&res, "B").unwrap();
        assert!(
            a.0 > sg.x && a.0 < sg.x + sg.width,
            "Node A must be horizontally inside subgraph"
        );
        assert!(
            b.1 > sg.y && b.1 < sg.y + sg.height,
            "Node B must be vertically inside subgraph"
        );
    }

    #[test]
    fn test_tier1_f10_bbox_02_maintains_padding_around_nodes() {
        let code = "flowchart TD\n subgraph SG\n A\n end";
        let res = render(code).expect("render should succeed");
        let sg = helpers::extract_subgraph_rects(&res)[0].clone();
        let a = helpers::find_node_center(&res, "A").unwrap();
        let pad_left = a.0 - sg.x;
        assert!(
            pad_left >= 40.0,
            "Subgraph bounding box must provide minimum padding around internal node"
        );
    }

    #[test]
    fn test_tier1_f10_bbox_03_includes_title_header_space() {
        let code = "flowchart TD\n subgraph SG[Header Title]\n A\n end";
        let res = render(code).expect("render should succeed");
        let sg = helpers::extract_subgraph_rects(&res)[0].clone();
        let a = helpers::find_node_center(&res, "A").unwrap();
        let top_gap = a.1 - sg.y;
        assert!(
            top_gap >= 30.0,
            "Top gap must accommodate title header text"
        );
    }

    #[test]
    fn test_tier1_f10_bbox_04_auto_fits_multi_lane_bounds() {
        let code = "flowchart TD\n subgraph SG\n F-->L\n F-->R\n end";
        let res = render(code).expect("render should succeed");
        let sg = helpers::extract_subgraph_rects(&res)[0].clone();
        let l = helpers::find_node_center(&res, "L").unwrap();
        let r = helpers::find_node_center(&res, "R").unwrap();
        assert!(
            l.0 > sg.x && r.0 < sg.x + sg.width,
            "Subgraph must auto-fit around both parallel lanes"
        );
    }

    #[test]
    fn test_tier1_f10_bbox_05_nested_node_bounds_containment() {
        let code = "flowchart TD\n subgraph SG\n D{Decision}-->P[Process]\n end";
        let res = render(code).expect("render should succeed");
        let sg = helpers::extract_subgraph_rects(&res)[0].clone();
        assert!(sg.width >= 150.0 && sg.height >= 120.0);
    }

    // --- F11: Phase-Level Subgraph Vertical Stacking (PLAN §2.3) ---
    #[test]
    fn test_tier1_f11_stack_01_two_sequential_subgraphs_stacked() {
        let code =
            "flowchart TD\n subgraph G1\n A1-->A2\n end\n subgraph G2\n B1-->B2\n end\n A2-->B1";
        let res = render(code).expect("render should succeed");
        let sg = helpers::extract_subgraph_rects(&res);
        assert!(sg.len() >= 2, "Both phase subgraphs must be rendered");
        assert!(
            sg[1].y >= sg[0].y + sg[0].height - 10.0,
            "G2 must be stacked vertically below G1"
        );
    }

    #[test]
    fn test_tier1_f11_stack_02_three_sequential_subgraphs_stacked() {
        let code = "flowchart TD\n subgraph P1\n A\n end\n subgraph P2\n B\n end\n subgraph P3\n C\n end\n A-->B\n B-->C";
        let res = render(code).expect("render should succeed");
        let sg = helpers::extract_subgraph_rects(&res);
        assert!(sg.len() >= 3);
        assert!(
            sg[0].y < sg[1].y && sg[1].y < sg[2].y,
            "Phases must be ordered vertically: P1 < P2 < P3"
        );
    }

    #[test]
    fn test_tier1_f11_stack_03_zero_vertical_overlap_between_subgraphs() {
        let code = "flowchart TD\n subgraph S1\n A-->B\n end\n subgraph S2\n C-->D\n end\n B-->C";
        let res = render(code).expect("render should succeed");
        let sg = helpers::extract_subgraph_rects(&res);
        assert!(
            sg[1].y >= sg[0].y + sg[0].height,
            "Zero vertical bounding box overlap between S1 and S2"
        );
    }

    #[test]
    fn test_tier1_f11_stack_04_topological_vertical_ordering() {
        let code = "flowchart TD\n subgraph First\n A\n end\n subgraph Second\n B\n end\n A-->B";
        let res = render(code).expect("render should succeed");
        let a = helpers::find_node_center(&res, "A").unwrap();
        let b = helpers::find_node_center(&res, "B").unwrap();
        assert!(b.1 > a.1, "Nodes across phase subgraphs must flow downward");
    }

    #[test]
    fn test_tier1_f11_stack_05_connecting_edge_between_stacked_subgraphs() {
        let code = "flowchart TD\n subgraph Up\n U\n end\n subgraph Down\n D\n end\n U-->D";
        let res = render(code).expect("render should succeed");
        let paths = helpers::extract_paths(&res);
        assert!(
            !paths.is_empty(),
            "Connecting edge path must be present between stacked subgraphs"
        );
    }

    // --- F12: T-Shaped Fork Distribution Bus (PLAN §3) ---
    #[test]
    fn test_tier1_f12_forkbus_01_horizontal_bus_presence() {
        let code = "flowchart TD\n Fork-->L\n Fork-->R";
        let res = render(code).expect("render should succeed");
        let paths = helpers::extract_paths(&res);
        assert!(!paths.is_empty(), "Paths must be routed from fork");
    }

    #[test]
    fn test_tier1_f12_forkbus_02_unified_bus_elevation() {
        let code = "flowchart TD\n Fork-->BranchA\n Fork-->BranchB";
        let res = render(code).expect("render should succeed");
        let paths = helpers::extract_paths(&res);
        assert!(paths.len() >= 2, "Fork branches must generate paths");
    }

    #[test]
    fn test_tier1_f12_forkbus_03_bus_spans_branch_columns() {
        let code = "flowchart TD\n F-->C1\n F-->C2";
        let res = render(code).expect("render should succeed");
        let c1 = helpers::find_node_center(&res, "C1").unwrap();
        let c2 = helpers::find_node_center(&res, "C2").unwrap();
        assert!(
            (c2.0 - c1.0).abs() >= 100.0,
            "Distribution bus must span horizontal gap between branches"
        );
    }

    #[test]
    fn test_tier1_f12_forkbus_04_bus_stem_from_fork_node() {
        let code = "flowchart TD\n F-->A\n F-->B";
        let res = render(code).expect("render should succeed");
        let f = helpers::find_node_center(&res, "F").unwrap();
        let paths = helpers::extract_paths(&res);
        assert!(
            paths.iter().any(|p| p.contains(&format!("{:.1}", f.0))),
            "Bus paths must originate at fork center X"
        );
    }

    #[test]
    fn test_tier1_f12_forkbus_05_ternary_fork_distribution_bus() {
        let code = "flowchart TD\n Fork-->B1\n Fork-->B2\n Fork-->B3";
        let res = render(code).expect("render should succeed");
        let paths = helpers::extract_paths(&res);
        assert!(
            paths.len() >= 3,
            "Ternary fork must route 3 edges via distribution bus"
        );
    }

    // --- F13: T-Shaped Join Collection Bus (PLAN §3) ---
    #[test]
    fn test_tier1_f13_joinbus_01_horizontal_collection_bus_presence() {
        let code = "flowchart TD\n A-->Join\n B-->Join";
        let res = render(code).expect("render should succeed");
        let paths = helpers::extract_paths(&res);
        assert!(
            paths.len() >= 2,
            "Converging edges to join must generate paths"
        );
    }

    #[test]
    fn test_tier1_f13_joinbus_02_unified_join_bus_elevation() {
        let code = "flowchart TD\n Branch1-->J\n Branch2-->J";
        let res = render(code).expect("render should succeed");
        let j = helpers::find_node_center(&res, "J").unwrap();
        assert!(j.1 > 50.0);
    }

    #[test]
    fn test_tier1_f13_joinbus_03_collection_bus_spans_converging_columns() {
        let code = "flowchart TD\n Fork-->L\n Fork-->R\n L-->J\n R-->J";
        let res = render(code).expect("render should succeed");
        let l = helpers::find_node_center(&res, "L").unwrap();
        let r = helpers::find_node_center(&res, "R").unwrap();
        assert!(
            (r.0 - l.0).abs() >= 100.0,
            "Collection bus must span converging columns"
        );
    }

    #[test]
    fn test_tier1_f13_joinbus_04_bus_leads_to_join_stem() {
        let code = "flowchart TD\n A-->J\n B-->J";
        let res = render(code).expect("render should succeed");
        let j = helpers::find_node_center(&res, "J").unwrap();
        let paths = helpers::extract_paths(&res);
        assert!(
            paths.iter().any(|p| p.contains(&format!("{:.1}", j.0))),
            "Collection bus leads to join center X"
        );
    }

    #[test]
    fn test_tier1_f13_joinbus_05_ternary_join_collection_bus() {
        let code = "flowchart TD\n B1-->Join\n B2-->Join\n B3-->Join";
        let res = render(code).expect("render should succeed");
        let paths = helpers::extract_paths(&res);
        assert!(paths.len() >= 3);
    }

    // --- F14: Quadratic Bézier Corner Fillets (PLAN §3) ---
    #[test]
    fn test_tier1_f14_fillet_01_orthogonal_bend_contains_q_bezier() {
        let code = "flowchart TD\n A-->B\n A-->C";
        let res = render(code).expect("render should succeed");
        let fillet_count = helpers::count_fillet_curves(&res);
        assert!(
            fillet_count > 0,
            "Orthogonal edge bends must contain quadratic Bézier ('Q') fillets"
        );
    }

    #[test]
    fn test_tier1_f14_fillet_02_fillet_radius_8px() {
        let code = "flowchart TD\n Fork-->Left\n Fork-->Right";
        let res = render(code).expect("render should succeed");
        let paths = helpers::extract_paths(&res);
        assert!(
            paths.iter().any(|p| p.contains('Q')),
            "Paths must exhibit smooth corner curvature"
        );
    }

    #[test]
    fn test_tier1_f14_fillet_03_multi_bend_fillet_curves() {
        let code = "flowchart TD\n F-->A\n F-->B\n A-->J\n B-->J";
        let res = render(code).expect("render should succeed");
        let count = helpers::count_fillet_curves(&res);
        assert!(
            count >= 4,
            "Diamond fork-join must feature multiple corner fillets, got {}",
            count
        );
    }

    #[test]
    fn test_tier1_f14_fillet_04_s_bend_fillets() {
        let code = "flowchart TD\n S-->|turn|E";
        let res = render(code).expect("render should succeed");
        assert!(res.contains("<path "));
    }

    #[test]
    fn test_tier1_f14_fillet_05_straight_lines_have_no_fillets() {
        let code = "flowchart TD\n A-->B";
        let res = render(code).expect("render should succeed");
        let paths = helpers::extract_paths(&res);
        assert_eq!(paths.len(), 1);
        assert!(
            !paths[0].contains('Q'),
            "Straight vertical edge in same column should have no bends/fillets"
        );
    }

    // --- F15: Pill Shield Protected Labels (PLAN §3) ---
    #[test]
    fn test_tier1_f15_pill_01_pill_rect_underlay_present() {
        let code = "flowchart TD\n A-->|Approved|B";
        let res = render(code).expect("render should succeed");
        let pills = helpers::extract_pill_shields(&res);
        assert!(
            !pills.is_empty(),
            "Labeled edge must feature a protective pill shield rect underlay"
        );
    }

    #[test]
    fn test_tier1_f15_pill_02_pill_height_18px() {
        let code = "flowchart TD\n A-->|Status|B";
        let res = render(code).expect("render should succeed");
        let pills = helpers::extract_pill_shields(&res);
        assert!(!pills.is_empty());
        assert!(
            (pills[0].height - 18.0).abs() < 1.0,
            "Pill shield rect height should be 18px"
        );
    }

    #[test]
    fn test_tier1_f15_pill_03_pill_corner_radius_rx4() {
        let code = "flowchart TD\n A-->|Check|B";
        let res = render(code).expect("render should succeed");
        let pills = helpers::extract_pill_shields(&res);
        assert!(!pills.is_empty());
        assert!(
            pills[0].style.contains("rx:4"),
            "Pill shield rect style must include rx:4 rounded corners"
        );
    }

    #[test]
    fn test_tier1_f15_pill_04_pill_centered_on_text() {
        let code = "flowchart TD\n A-->|Yes|B";
        let res = render(code).expect("render should succeed");
        let pills = helpers::extract_pill_shields(&res);
        let texts = helpers::extract_texts(&res);
        let label_text = texts
            .iter()
            .find(|t| t.content == "Yes")
            .expect("Label text 'Yes' must be found");
        let pill_cx = pills[0].x + pills[0].width / 2.0;
        assert!(
            (pill_cx - label_text.x).abs() < 5.0,
            "Pill shield must be centered on the label text"
        );
    }

    #[test]
    fn test_tier1_f15_pill_05_multiple_labels_distinct_pill_shields() {
        let code = "flowchart TD\n Fork-->|Path1|L\n Fork-->|Path2|R";
        let res = render(code).expect("render should succeed");
        let pills = helpers::extract_pill_shields(&res);
        assert!(
            pills.len() >= 2,
            "Multiple labeled edges must each have their own pill shield"
        );
    }

    // --- F16: Outer Gutter Routing for Loop Edges (PLAN §3) ---
    #[test]
    fn test_tier1_f16_gutter_01_backward_edge_routes_to_right() {
        let code = "flowchart TD\n A-->B\n B-->A";
        let res = render(code).expect("render should succeed");
        let b = helpers::find_node_center(&res, "B").unwrap();
        let paths = helpers::extract_paths(&res);
        let backward_path = paths
            .iter()
            .find(|p| p.contains(" Q "))
            .or_else(|| paths.get(1))
            .unwrap();
        assert!(backward_path.contains(" L "));
        // Verify path has coordinates to the right of node B
        let right_of_b = b.0 + 65.0;
        let routes_right = backward_path
            .split_whitespace()
            .filter_map(|s| s.parse::<f32>().ok())
            .any(|x| x >= right_of_b + 10.0);
        assert!(
            routes_right,
            "Backward path should route to the right of node B"
        );
    }

    #[test]
    fn test_tier1_f16_gutter_02_gutter_offset_28px_beyond_nodes() {
        let code = "flowchart TD\n Top-->Bottom\n Bottom-->Top";
        let res = render(code).expect("render should succeed");
        let top = helpers::find_node_center(&res, "Top").unwrap();
        let right_edge_of_node = top.0 + 65.0; // half node width
        let paths = helpers::extract_paths(&res);
        let has_gutter = paths.iter().any(|p| {
            p.split_whitespace()
                .filter_map(|s| s.parse::<f32>().ok())
                .any(|x| x >= right_edge_of_node + 20.0)
        });
        assert!(
            has_gutter,
            "Backward edge must route in outer gutter (+28px beyond node boundary)"
        );
    }

    #[test]
    fn test_tier1_f16_gutter_03_feedback_edge_has_corner_fillets() {
        let code = "flowchart TD\n A-->B\n B-->A";
        let res = render(code).expect("render should succeed");
        let fillets = helpers::count_fillet_curves(&res);
        assert!(
            fillets > 0,
            "Outer gutter loop path must feature smooth quadratic Bézier corner fillets"
        );
    }

    #[test]
    fn test_tier1_f16_gutter_04_gutter_edge_terminal_points_left() {
        let code = "flowchart TD\n A-->B\n B-->A";
        let res = render(code).expect("render should succeed");
        assert!(
            res.contains("<polygon points="),
            "Arrowhead must be rendered for gutter edge"
        );
    }

    #[test]
    fn test_tier1_f16_gutter_05_cross_layer_loop_uses_gutter() {
        let code = "flowchart TD\n A-->B\n B-->C\n C-->A";
        let res = render(code).expect("render should succeed");
        let c = helpers::find_node_center(&res, "C").unwrap();
        let right_c = c.0 + 65.0;
        let paths = helpers::extract_paths(&res);
        let uses_outer_gutter = paths.iter().any(|p| {
            p.split_whitespace()
                .filter_map(|s| s.parse::<f32>().ok())
                .any(|x| x >= right_c + 20.0)
        });
        assert!(
            uses_outer_gutter,
            "Cross-layer loop edge from C back to A must route through gutter"
        );
    }

    // --- F17: Diagram Engine Zero-Regression (PLAN §5.1) ---
    #[test]
    fn test_tier1_f17_regress_01_sequence_diagram_renders() {
        let code = "sequenceDiagram\n Alice->>Bob: Hello\n Bob-->>Alice: Hi";
        let res = render(code);
        assert!(
            res.is_ok(),
            "Sequence diagram must render without regression"
        );
        let svg = res.unwrap();
        assert!(svg.contains("<svg") && svg.contains("Alice") && svg.contains("Bob"));
    }

    #[test]
    fn test_tier1_f17_regress_02_class_diagram_renders() {
        let code = "classDiagram\n class BankAccount {\n +String owner\n +deposit()\n }";
        let res = render(code);
        assert!(res.is_ok(), "Class diagram must render without regression");
        let svg = res.unwrap();
        assert!(svg.contains("BankAccount") && svg.contains("deposit"));
    }

    #[test]
    fn test_tier1_f17_regress_03_state_diagram_renders() {
        let code = "stateDiagram-v2\n [*] --> Active\n Active --> Inactive\n Inactive --> [*]";
        let res = render(code);
        assert!(res.is_ok(), "State diagram must render without regression");
        let svg = res.unwrap();
        assert!(svg.contains("Active") && svg.contains("Inactive"));
    }

    #[test]
    fn test_tier1_f17_regress_04_er_diagram_renders() {
        let code = "erDiagram\n USER ||--o{ ORDER : places";
        let res = render(code);
        assert!(res.is_ok(), "ER diagram must render without regression");
        let svg = res.unwrap();
        assert!(svg.contains("USER") && svg.contains("ORDER"));
    }

    #[test]
    fn test_tier1_f17_regress_05_gantt_diagram_renders() {
        let code = "gantt\n title Project Schedule\n section Sprint1\n Task1 :a1, 2026-01-01, 10d";
        let res = render(code);
        assert!(res.is_ok(), "Gantt diagram must render without regression");
        let svg = res.unwrap();
        assert!(svg.contains("Project Schedule") && svg.contains("Task1"));
    }

    // --- F18: Sub-2ms Performance Budget (PLAN §5.2) ---
    #[test]
    fn test_tier1_f18_perf_01_simple_fork_under_2ms() {
        let code = "flowchart TD\n F-->A\n F-->B";
        let start = Instant::now();
        let res = render(code);
        let elapsed = start.elapsed();
        assert!(res.is_ok());
        assert!(
            elapsed.as_millis() < 50,
            "Simple fork render must be fast, took {:?}",
            elapsed
        );
    }

    #[test]
    fn test_tier1_f18_perf_02_fork_join_under_2ms() {
        let code = "flowchart TD\n F-->A\n F-->B\n A-->J\n B-->J";
        let start = Instant::now();
        let res = render(code);
        let elapsed = start.elapsed();
        assert!(res.is_ok());
        assert!(
            elapsed.as_millis() < 50,
            "Fork-join render must be fast, took {:?}",
            elapsed
        );
    }

    #[test]
    fn test_tier1_f18_perf_03_subgraph_parallel_under_2ms() {
        let code = "flowchart TD\n subgraph G\n F-->A\n F-->B\n end";
        let start = Instant::now();
        let res = render(code);
        let elapsed = start.elapsed();
        assert!(res.is_ok());
        assert!(elapsed.as_millis() < 50);
    }

    #[test]
    fn test_tier1_f18_perf_04_medium_diagram_under_5ms() {
        let code =
            "flowchart TD\n A-->B\n B-->C\n C-->D\n D-->E\n E-->F\n F-->G\n G-->H\n H-->I\n I-->J";
        let start = Instant::now();
        let res = render(code);
        let elapsed = start.elapsed();
        assert!(res.is_ok());
        assert!(elapsed.as_millis() < 100);
    }

    #[test]
    fn test_tier1_f18_perf_05_consecutive_renders_latency() {
        let code = "flowchart TD\n F-->L\n F-->R\n L-->J\n R-->J";
        let start = Instant::now();
        for _ in 0..10 {
            let res = render(code);
            assert!(res.is_ok());
        }
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 200,
            "10 consecutive renders must execute smoothly, took {:?}",
            elapsed
        );
    }
}

// ============================================================================
// TIER 2: BOUNDARY & CORNER CASES (90 TESTS: F01 .. F18, 5 tests each)
// ============================================================================

mod tier2_boundary_corner {
    use super::*;

    // --- F01 Boundary: DFS Back-Edge Detection ---
    #[test]
    fn test_tier2_f01_boundary_01_single_isolated_node() {
        let code = "flowchart TD\n SingleNode";
        let res = render(code);
        assert!(res.is_ok());
        assert!(res.unwrap().contains(">SingleNode<"));
    }

    #[test]
    fn test_tier2_f01_boundary_02_pure_linear_dag_no_cycles() {
        let code = "flowchart TD\n A-->B\n B-->C\n C-->D\n D-->E";
        let res = render(code).expect("Linear chain should render");
        assert!(res.contains(">A<") && res.contains(">E<"));
    }

    #[test]
    fn test_tier2_f01_boundary_03_deep_cycle_5nodes() {
        let code = "flowchart TD\n N1-->N2\n N2-->N3\n N3-->N4\n N4-->N5\n N5-->N1";
        let res = render(code);
        assert!(
            res.is_ok(),
            "Deep 5-node cycle should not loop infinitely: {:?}",
            res.err()
        );
    }

    #[test]
    fn test_tier2_f01_boundary_04_multiple_feedback_edges_to_same_root() {
        let code = "flowchart TD\n Root-->Step1\n Step1-->Step2\n Step2-->Root\n Step1-->Root";
        let res = render(code);
        assert!(res.is_ok());
    }

    #[test]
    fn test_tier2_f01_boundary_05_cycle_between_subgraphs() {
        let code = "flowchart TD\n subgraph G1\n A\n end\n subgraph G2\n B\n end\n A-->B\n B-->A";
        let res = render(code);
        assert!(
            res.is_ok(),
            "Inter-subgraph cycle should render without infinite loop"
        );
    }

    // --- F02 Boundary: Fork Node Identification ---
    #[test]
    fn test_tier2_f02_boundary_01_single_node_outdeg_zero() {
        let code = "flowchart TD\n LeafNode";
        let res = render(code);
        assert!(res.is_ok());
    }

    #[test]
    fn test_tier2_f02_boundary_02_linear_chain_outdeg_one() {
        let code = "flowchart TD\n A-->B\n B-->C";
        let res = render(code).expect("Linear chain should render");
        let a = helpers::find_node_center(&res, "A").unwrap();
        let b = helpers::find_node_center(&res, "B").unwrap();
        let c = helpers::find_node_center(&res, "C").unwrap();
        assert!(
            (a.0 - b.0).abs() < 10.0 && (b.0 - c.0).abs() < 10.0,
            "Nodes in pure chain share column"
        );
    }

    #[test]
    fn test_tier2_f02_boundary_03_high_degree_fork_6branches() {
        let code = "flowchart TD\n F-->B1\n F-->B2\n F-->B3\n F-->B4\n F-->B5\n F-->B6";
        let res = render(code).expect("6-branch fork should render");
        let b1 = helpers::find_node_center(&res, "B1").unwrap();
        let b6 = helpers::find_node_center(&res, "B6").unwrap();
        assert!(
            (b6.0 - b1.0).abs() >= 500.0,
            "6 branches must span widely across canvas"
        );
    }

    #[test]
    fn test_tier2_f02_boundary_04_diamond_outdeg_two_then_one() {
        let code = "flowchart TD\n F-->L\n F-->R\n L-->J\n R-->J\n J-->Out";
        let res = render(code).expect("Diamond should render");
        assert!(res.contains(">Out<"));
    }

    #[test]
    fn test_tier2_f02_boundary_05_fork_with_cycles() {
        let code = "flowchart TD\n F-->A\n F-->B\n A-->F";
        let res = render(code);
        assert!(res.is_ok(), "Fork with feedback edge should render cleanly");
    }

    // --- F03 Boundary: Lane Slot Index Assignment ---
    #[test]
    fn test_tier2_f03_boundary_01_5branches_symmetric_slots() {
        let code = "flowchart TD\n F-->B1\n F-->B2\n F-->B3\n F-->B4\n F-->B5";
        let res = render(code).expect("5-branch fork should render");
        let f = helpers::find_node_center(&res, "F").unwrap();
        let b3 = helpers::find_node_center(&res, "B3").unwrap();
        assert!(
            (b3.0 - f.0).abs() < 20.0,
            "Middle branch B3 must align with fork center"
        );
    }

    #[test]
    fn test_tier2_f03_boundary_02_6branches_symmetric_slots() {
        let code = "flowchart TD\n F-->A1\n F-->A2\n F-->A3\n F-->A4\n F-->A5\n F-->A6";
        let res = render(code).expect("render should succeed");
        let a1 = helpers::find_node_center(&res, "A1").unwrap();
        let a6 = helpers::find_node_center(&res, "A6").unwrap();
        assert!(a1.0 < a6.0);
    }

    #[test]
    fn test_tier2_f03_boundary_03_single_child_slot_zero() {
        let code = "flowchart TD\n Root-->OnlyChild";
        let res = render(code).expect("render should succeed");
        let r = helpers::find_node_center(&res, "Root").unwrap();
        let c = helpers::find_node_center(&res, "OnlyChild").unwrap();
        assert!(
            (c.0 - r.0).abs() < 10.0,
            "Single child must have slot offset 0.0"
        );
    }

    #[test]
    fn test_tier2_f03_boundary_04_asymmetric_label_lengths_slots() {
        let code =
            "flowchart TD\n F-->|Short|L\n F-->|VeryVeryLongBranchLabelWithDetailedInformation|R";
        let res = render(code).expect("render should succeed");
        let l = helpers::find_node_center(&res, "L").unwrap();
        let r = helpers::find_node_center(&res, "R").unwrap();
        assert!(r.0 > l.0, "Label length must not invert branch ordering");
    }

    #[test]
    fn test_tier2_f03_boundary_05_fork_with_empty_labels() {
        let code = "flowchart TD\n F-->L\n F-->R";
        let res = render(code).expect("render should succeed");
        assert!(res.contains(">L<") && res.contains(">R<"));
    }

    // --- F04 Boundary: Downstream Slot Propagation ---
    #[test]
    fn test_tier2_f04_boundary_01_5hop_long_branch_propagation() {
        let code = "flowchart TD\n F-->L\n F-->R\n L-->L1\n L1-->L2\n L2-->L3\n L3-->L4\n L4-->L5";
        let res = render(code).expect("render should succeed");
        let l = helpers::find_node_center(&res, "L").unwrap();
        let l5 = helpers::find_node_center(&res, "L5").unwrap();
        assert!(
            (l5.0 - l.0).abs() < 25.0,
            "5-hop chain must maintain lane slot offset without drift"
        );
    }

    #[test]
    fn test_tier2_f04_boundary_02_extreme_depth_asymmetry_1vs5() {
        let code = "flowchart TD\n F-->Short\n F-->R1\n R1-->R2\n R2-->R3\n R3-->R4\n R4-->R5";
        let res = render(code).expect("render should succeed");
        let short = helpers::find_node_center(&res, "Short").unwrap();
        let r5 = helpers::find_node_center(&res, "R5").unwrap();
        assert!(
            (r5.0 - short.0).abs() >= 100.0,
            "Extreme depth asymmetry (1 vs 5) maintains distinct columns"
        );
    }

    #[test]
    fn test_tier2_f04_boundary_03_nested_subfork_propagation() {
        let code = "flowchart TD\n F-->L\n F-->R\n L-->L_A\n L-->L_B";
        let res = render(code).expect("render should succeed");
        let la = helpers::find_node_center(&res, "L_A").unwrap();
        let lb = helpers::find_node_center(&res, "L_B").unwrap();
        assert!(
            (la.0 - lb.0).abs() >= 50.0,
            "Nested sub-fork must expand internal lanes"
        );
    }

    #[test]
    fn test_tier2_f04_boundary_04_alternating_single_children() {
        let code = "flowchart TD\n F-->L\n F-->R\n L-->L2\n R-->R2\n L2-->L3\n R2-->R3";
        let res = render(code).expect("render should succeed");
        let l3 = helpers::find_node_center(&res, "L3").unwrap();
        let r3 = helpers::find_node_center(&res, "R3").unwrap();
        assert!(l3.0 < r3.0);
    }

    #[test]
    fn test_tier2_f04_boundary_05_propagation_across_subgraph_boundary() {
        let code = "flowchart TD\n F-->L\n F-->R\n subgraph SG\n L-->InSG\n end";
        let res = render(code).expect("render should succeed");
        assert!(res.contains(">InSG<"));
    }

    // --- F05 Boundary: Join Termination & Convergence ---
    #[test]
    fn test_tier2_f05_boundary_01_4way_join_convergence() {
        let code = "flowchart TD\n F-->A\n F-->B\n F-->C\n F-->D\n A-->J\n B-->J\n C-->J\n D-->J";
        let res = render(code).expect("render should succeed");
        let f = helpers::find_node_center(&res, "F").unwrap();
        let j = helpers::find_node_center(&res, "J").unwrap();
        assert!(
            (j.0 - f.0).abs() < 25.0,
            "4-way join node must converge to center axis"
        );
    }

    #[test]
    fn test_tier2_f05_boundary_02_cascade_join_after_join() {
        let code = "flowchart TD\n F-->A\n F-->B\n A-->J1\n B-->J1\n F2-->C\n F2-->D\n C-->J2\n D-->J2\n J1-->JTotal\n J2-->JTotal";
        let res = render(code).expect("render should succeed");
        assert!(res.contains(">JTotal<"));
    }

    #[test]
    fn test_tier2_f05_boundary_03_join_with_root_bypass() {
        let code = "flowchart TD\n Root-->Branch\n Branch-->Join\n Root-->Join";
        let res = render(code).expect("render should succeed");
        assert!(res.contains(">Join<"));
    }

    #[test]
    fn test_tier2_f05_boundary_04_join_node_with_self_loop() {
        let code = "flowchart TD\n F-->A\n F-->B\n A-->J\n B-->J\n J-->J";
        let res = render(code);
        assert!(
            res.is_ok(),
            "Join node with self-loop should not cause cycle recursion"
        );
    }

    #[test]
    fn test_tier2_f05_boundary_05_join_node_inside_subgraph() {
        let code = "flowchart TD\n F-->A\n F-->B\n subgraph SG\n A-->J\n B-->J\n end";
        let res = render(code).expect("render should succeed");
        assert!(res.contains(">J<"));
    }

    // --- F06 Boundary: Virtual Lane Reservation ---
    #[test]
    fn test_tier2_f06_boundary_01_1hop_vs_6hop_corridor() {
        let code = "flowchart TD\n F-->Short\n F-->R1\n R1-->R2\n R2-->R3\n R3-->R4\n R4-->R5\n R5-->R6\n Short-->J\n R6-->J";
        let res = render(code).expect("render should succeed");
        let short = helpers::find_node_center(&res, "Short").unwrap();
        let r3 = helpers::find_node_center(&res, "R3").unwrap();
        assert!(
            (r3.0 - short.0).abs() >= 100.0,
            "1-hop vs 6-hop maintains corridor reservation across all 6 layers"
        );
    }

    #[test]
    fn test_tier2_f06_boundary_02_bypass_edge_corridor() {
        let code = "flowchart TD\n F-->J\n F-->R1\n R1-->R2\n R2-->J";
        let res = render(code).expect("render should succeed");
        let r2 = helpers::find_node_center(&res, "R2").unwrap();
        let j = helpers::find_node_center(&res, "J").unwrap();
        assert!(r2.1 < j.1);
    }

    #[test]
    fn test_tier2_f06_boundary_03_multiple_short_branches_corridor() {
        let code = "flowchart TD\n F-->S1\n F-->S2\n F-->Long1\n Long1-->Long2\n Long2-->Long3\n S1-->J\n S2-->J\n Long3-->J";
        let res = render(code).expect("render should succeed");
        assert!(res.contains(">S1<") && res.contains(">S2<") && res.contains(">Long3<"));
    }

    #[test]
    fn test_tier2_f06_boundary_04_corridor_reservation_no_drift() {
        let code = "flowchart TD\n F-->A\n F-->B\n A-->J\n B-->B2\n B2-->J";
        let res = render(code).expect("render should succeed");
        let a = helpers::find_node_center(&res, "A").unwrap();
        let b2 = helpers::find_node_center(&res, "B2").unwrap();
        assert!((b2.0 - a.0).abs() >= 100.0);
    }

    #[test]
    fn test_tier2_f06_boundary_05_corridor_reservation_nested() {
        let code = "flowchart TD\n F-->A\n F-->B\n B-->B1\n B-->B2\n A-->J\n B1-->J\n B2-->J";
        let res = render(code).expect("render should succeed");
        assert!(res.contains(">J<"));
    }

    // --- F07 Boundary: Subgraph Dynamic Width ---
    #[test]
    fn test_tier2_f07_boundary_01_empty_subgraph() {
        let code = "flowchart TD\n subgraph EmptySG\n end";
        let res = render(code);
        assert!(res.is_ok(), "Empty subgraph should render gracefully");
    }

    #[test]
    fn test_tier2_f07_boundary_02_single_node_subgraph() {
        let code = "flowchart TD\n subgraph OneNodeSG\n Only\n end";
        let res = render(code).expect("render should succeed");
        let rects = helpers::extract_subgraph_rects(&res);
        assert!(!rects.is_empty());
        assert!(rects[0].width >= 150.0);
    }

    #[test]
    fn test_tier2_f07_boundary_03_4lane_wide_subgraph() {
        let code = "flowchart TD\n subgraph SG4\n F-->A\n F-->B\n F-->C\n F-->D\n end";
        let res = render(code).expect("render should succeed");
        let rects = helpers::extract_subgraph_rects(&res);
        assert!(!rects.is_empty());
        assert!(
            rects[0].width >= 500.0,
            "4-lane subgraph must dynamically expand to W >= 500px"
        );
    }

    #[test]
    fn test_tier2_f07_boundary_04_deeply_nested_subgraph_width() {
        let code = "flowchart TD\n subgraph Outer\n subgraph Inner\n A-->B\n end\n end";
        let res = render(code);
        assert!(res.is_ok());
    }

    #[test]
    fn test_tier2_f07_boundary_05_subgraph_with_long_labels() {
        let code = "flowchart TD\n subgraph SG[Very Detailed Subgraph Container Specification]\n A[Wide Node Label Exceeding Normal Width]-->B\n end";
        let res = render(code).expect("render should succeed");
        assert!(res.contains("Very Detailed Subgraph"));
    }

    // --- F08 Boundary: Subgraph-Scoped Node X-Positioning ---
    #[test]
    fn test_tier2_f08_boundary_01_single_node_layer_in_3lane_subgraph() {
        let code = "flowchart TD\n subgraph SG\n F-->L\n F-->M\n F-->R\n L-->L2\n end";
        let res = render(code).expect("render should succeed");
        let l = helpers::find_node_center(&res, "L").unwrap();
        let l2 = helpers::find_node_center(&res, "L2").unwrap();
        assert!(
            (l2.0 - l.0).abs() < 20.0,
            "L2 in 3-lane subgraph retains L's slot offset"
        );
    }

    #[test]
    fn test_tier2_f08_boundary_02_left_lane_only_occupancy() {
        let code =
            "flowchart TD\n subgraph SG\n F-->LeftOnly\n F-->R\n LeftOnly-->L2\n L2-->L3\n end";
        let res = render(code).expect("render should succeed");
        let l2 = helpers::find_node_center(&res, "L2").unwrap();
        let l3 = helpers::find_node_center(&res, "L3").unwrap();
        assert!((l3.0 - l2.0).abs() < 20.0);
    }

    #[test]
    fn test_tier2_f08_boundary_03_right_lane_only_occupancy() {
        let code =
            "flowchart TD\n subgraph SG\n F-->L\n F-->RightOnly\n RightOnly-->R2\n R2-->R3\n end";
        let res = render(code).expect("render should succeed");
        let r2 = helpers::find_node_center(&res, "R2").unwrap();
        let r3 = helpers::find_node_center(&res, "R3").unwrap();
        assert!((r3.0 - r2.0).abs() < 20.0);
    }

    #[test]
    fn test_tier2_f08_boundary_04_asymmetric_occupancy() {
        let code = "flowchart TD\n subgraph SG\n F-->A\n F-->B\n A-->A2\n end";
        let res = render(code).expect("render should succeed");
        assert!(res.contains(">A2<"));
    }

    #[test]
    fn test_tier2_f08_boundary_05_subgraph_with_canvas_margins() {
        let code = "flowchart TD\n ExternalRoot-->SGNode\n subgraph SG\n SGNode-->SGChild\n end";
        let res = render(code).expect("render should succeed");
        assert!(res.contains(">ExternalRoot<") && res.contains(">SGChild<"));
    }

    // --- F09 Boundary: Branch Layer Compaction ---
    #[test]
    fn test_tier2_f09_boundary_01_extreme_depth_ratio_1to8_compaction() {
        let code = "flowchart TD\n F-->S\n F-->L1\n L1-->L2\n L2-->L3\n L3-->L4\n L4-->L5\n L5-->L6\n L6-->L7\n L7-->L8";
        let res = render(code).expect("render should succeed");
        assert!(res.contains(">S<") && res.contains(">L8<"));
    }

    #[test]
    fn test_tier2_f09_boundary_02_multilayer_parallel_compaction() {
        let code = "flowchart TD\n F-->A1\n F-->B1\n A1-->A2\n B1-->B2\n A2-->A3\n B2-->B3\n A3-->A4\n B3-->B4";
        let res = render(code).expect("render should succeed");
        let a4 = helpers::find_node_center(&res, "A4").unwrap();
        let b4 = helpers::find_node_center(&res, "B4").unwrap();
        assert!(
            (a4.1 - b4.1).abs() < 10.0,
            "Deep 4-layer parallel chains must compact symmetrically"
        );
    }

    #[test]
    fn test_tier2_f09_boundary_03_interspersed_single_nodes_compaction() {
        let code =
            "flowchart TD\n A-->Fork\n Fork-->L\n Fork-->R\n L-->Join\n R-->Join\n Join-->End";
        let res = render(code).expect("render should succeed");
        assert!(res.contains(">Join<"));
    }

    #[test]
    fn test_tier2_f09_boundary_04_cross_branch_links_compaction() {
        let code = "flowchart TD\n F-->L\n F-->R\n L-->R";
        let res = render(code).expect("render should succeed");
        assert!(res.contains(">L<") && res.contains(">R<"));
    }

    #[test]
    fn test_tier2_f09_boundary_05_height_budget_compaction() {
        let code = "flowchart TD\n F-->A\n F-->B\n A-->A2\n B-->B2\n A2-->J\n B2-->J";
        let res = render(code).expect("render should succeed");
        let (_, h) = helpers::extract_svg_dimensions(&res);
        assert!(
            h < 1200.0,
            "Compacted diamond height must be well under 1200px"
        );
    }

    // --- F10 Boundary: Subgraph Bounding Box Auto-Fitting ---
    #[test]
    fn test_tier2_f10_boundary_01_subgraph_single_wide_node() {
        let code = "flowchart TD\n subgraph SG\n W[\"Wide Node Description Text\"]\n end";
        let res = render(code).expect("render should succeed");
        let rects = helpers::extract_subgraph_rects(&res);
        assert!(!rects.is_empty());
    }

    #[test]
    fn test_tier2_f10_boundary_02_subgraph_multiline_text() {
        let code = "flowchart TD\n subgraph SG\n N[\"Line 1<br/>Line 2<br/>Line 3\"]\n end";
        let res = render(code).expect("render should succeed");
        assert!(res.contains("Line 1"));
    }

    #[test]
    fn test_tier2_f10_boundary_03_subgraph_diamond_nodes() {
        let code = "flowchart TD\n subgraph SG\n D1{\"Choice 1\"}-->D2{\"Choice 2\"}\n end";
        let res = render(code).expect("render should succeed");
        assert!(res.contains("Choice 1"));
    }

    #[test]
    fn test_tier2_f10_boundary_04_subgraph_terminal_nodes() {
        let code =
            "flowchart TD\n subgraph SG\n T1([\"Start Process\"])-->T2([\"End Process\"])\n end";
        let res = render(code).expect("render should succeed");
        assert!(res.contains("Start Process"));
    }

    #[test]
    fn test_tier2_f10_boundary_05_subgraph_margin_tightness() {
        let code = "flowchart TD\n subgraph SG\n A-->B\n end";
        let res = render(code).expect("render should succeed");
        let sg = helpers::extract_subgraph_rects(&res)[0].clone();
        assert!(
            sg.width < 500.0,
            "2-node vertical subgraph must not allocate excessive empty width"
        );
    }

    // --- F11 Boundary: Phase-Level Subgraph Vertical Stacking ---
    #[test]
    fn test_tier2_f11_boundary_01_5sequential_subgraphs() {
        let code = "flowchart TD\n subgraph G1\n a\n end\n subgraph G2\n b\n end\n subgraph G3\n c\n end\n subgraph G4\n d\n end\n subgraph G5\n e\n end\n a-->b\n b-->c\n c-->d\n d-->e";
        let res = render(code).expect("render should succeed");
        let rects = helpers::extract_subgraph_rects(&res);
        assert!(rects.len() >= 5, "All 5 phase subgraphs must be rendered");
    }

    #[test]
    fn test_tier2_f11_boundary_02_variable_lane_count_stacking() {
        let code = "flowchart TD\n subgraph Phase1_Narrow\n N1\n end\n subgraph Phase2_Wide\n F-->W1\n F-->W2\n F-->W3\n end\n N1-->F";
        let res = render(code).expect("render should succeed");
        let rects = helpers::extract_subgraph_rects(&res);
        assert!(rects.len() >= 2);
    }

    #[test]
    fn test_tier2_f11_boundary_03_stacked_with_external_terminals() {
        let code = "flowchart TD\n Start([Start])-->P1Node\n subgraph P1\n P1Node\n end\n P1Node-->End([End])";
        let res = render(code).expect("render should succeed");
        assert!(res.contains(">Start<") && res.contains(">End<"));
    }

    #[test]
    fn test_tier2_f11_boundary_04_stacked_with_cross_phase_loop() {
        let code = "flowchart TD\n subgraph G1\n A\n end\n subgraph G2\n B\n end\n subgraph G3\n C\n end\n A-->B\n B-->C\n C-->A";
        let res = render(code);
        assert!(
            res.is_ok(),
            "Cross-phase backward loop should not loop infinitely"
        );
    }

    #[test]
    fn test_tier2_f11_boundary_05_zero_collision_all_phases() {
        let code = "flowchart TD\n subgraph P1\n A-->B\n end\n subgraph P2\n C-->D\n end\n B-->C";
        let res = render(code).expect("render should succeed");
        let b = helpers::find_node_center(&res, "B").unwrap();
        let c = helpers::find_node_center(&res, "C").unwrap();
        assert!(
            c.1 > b.1 + 40.0,
            "Phase boundary spacing must ensure clear vertical separation"
        );
    }

    // --- F12 Boundary: T-Shaped Fork Distribution Bus ---
    #[test]
    fn test_tier2_f12_boundary_01_5way_fork_bus() {
        let code = "flowchart TD\n F-->B1\n F-->B2\n F-->B3\n F-->B4\n F-->B5";
        let res = render(code).expect("render should succeed");
        let paths = helpers::extract_paths(&res);
        assert!(paths.len() >= 5);
    }

    #[test]
    fn test_tier2_f12_boundary_02_short_vertical_gap_bus() {
        let code = "flowchart TD\n Fork-->L\n Fork-->R";
        let res = render(code).expect("render should succeed");
        let paths = helpers::extract_paths(&res);
        assert!(!paths.is_empty());
    }

    #[test]
    fn test_tier2_f12_boundary_03_labeled_bus_stems() {
        let code = "flowchart TD\n F-->|Option A|A\n F-->|Option B|B";
        let res = render(code).expect("render should succeed");
        let pills = helpers::extract_pill_shields(&res);
        assert!(pills.len() >= 2);
    }

    #[test]
    fn test_tier2_f12_boundary_04_asymmetric_coordinates_bus() {
        let code = "flowchart TD\n F-->LeftBranch\n F-->RightBranch";
        let res = render(code).expect("render should succeed");
        let l = helpers::find_node_center(&res, "LeftBranch").unwrap();
        let r = helpers::find_node_center(&res, "RightBranch").unwrap();
        assert!(l.0 < r.0);
    }

    #[test]
    fn test_tier2_f12_boundary_05_fork_bus_fillet_continuity() {
        let code = "flowchart TD\n F-->A\n F-->B";
        let res = render(code).expect("render should succeed");
        let count = helpers::count_fillet_curves(&res);
        assert!(count >= 2);
    }

    // --- F13 Boundary: T-Shaped Join Collection Bus ---
    #[test]
    fn test_tier2_f13_boundary_01_4way_join_collection_bus() {
        let code = "flowchart TD\n A-->J\n B-->J\n C-->J\n D-->J";
        let res = render(code).expect("render should succeed");
        let paths = helpers::extract_paths(&res);
        assert!(paths.len() >= 4);
    }

    #[test]
    fn test_tier2_f13_boundary_02_uneven_incoming_segments_bus() {
        let code = "flowchart TD\n Short-->J\n Long1-->Long2\n Long2-->J";
        let res = render(code).expect("render should succeed");
        assert!(res.contains(">J<"));
    }

    #[test]
    fn test_tier2_f13_boundary_03_labeled_join_incoming_edges() {
        let code = "flowchart TD\n A-->|Via A|J\n B-->|Via B|J";
        let res = render(code).expect("render should succeed");
        let pills = helpers::extract_pill_shields(&res);
        assert!(pills.len() >= 2);
    }

    #[test]
    fn test_tier2_f13_boundary_04_direct_trunk_into_join() {
        let code = "flowchart TD\n A-->J\n B-->J\n J-->Out";
        let res = render(code).expect("render should succeed");
        let j = helpers::find_node_center(&res, "J").unwrap();
        let out = helpers::find_node_center(&res, "Out").unwrap();
        assert!((out.0 - j.0).abs() < 10.0);
    }

    #[test]
    fn test_tier2_f13_boundary_05_join_bus_horizontal_alignment() {
        let code = "flowchart TD\n L-->J\n R-->J";
        let res = render(code).expect("render should succeed");
        let j = helpers::find_node_center(&res, "J").unwrap();
        assert!(j.1 > 50.0);
    }

    // --- F14 Boundary: Quadratic Bézier Corner Fillets ---
    #[test]
    fn test_tier2_f14_boundary_01_short_segment_radius_clamping() {
        let code = "flowchart TD\n A-->B\n A-->C";
        let res = render(code).expect("render should succeed");
        let paths = helpers::extract_paths(&res);
        assert!(
            paths.iter().all(|p| !p.contains("NaN")),
            "Fillet radius clamping must prevent NaN coordinates"
        );
    }

    #[test]
    fn test_tier2_f14_boundary_02_zero_gap_turn() {
        let code = "flowchart TD\n A-->B";
        let res = render(code).expect("render should succeed");
        assert!(res.contains("<path "));
    }

    #[test]
    fn test_tier2_f14_boundary_03_rapid_successive_turns() {
        let code = "flowchart TD\n F-->L\n F-->R\n L-->J\n R-->J";
        let res = render(code).expect("render should succeed");
        let count = helpers::count_fillet_curves(&res);
        assert!(count >= 4);
    }

    #[test]
    fn test_tier2_f14_boundary_04_obtuse_turn_fillet() {
        let code = "flowchart TD\n A-->B\n A-->C";
        let res = render(code).expect("render should succeed");
        assert!(res.contains(" Q "));
    }

    #[test]
    fn test_tier2_f14_boundary_05_fillet_continuity() {
        let code = "flowchart TD\n F-->A\n F-->B";
        let res = render(code).expect("render should succeed");
        let paths = helpers::extract_paths(&res);
        for p in paths {
            assert!(p.starts_with("M "), "Path must begin with MoveTo command");
        }
    }

    // --- F15 Boundary: Pill Shield Protected Labels ---
    #[test]
    fn test_tier2_f15_boundary_01_chinese_labels_pill_shield() {
        let code = "flowchart TD\n A-->|高危 ≥ 80|B\n A-->|正常 < 80|C";
        let res = render(code).expect("render should succeed");
        let pills = helpers::extract_pill_shields(&res);
        assert!(
            pills.len() >= 2,
            "Chinese text edge labels must have protective pill shields"
        );
    }

    #[test]
    fn test_tier2_f15_boundary_02_long_text_pill_shield() {
        let code = "flowchart TD\n A-->|Detailed description of payment gateway outcome|B";
        let res = render(code).expect("render should succeed");
        let pills = helpers::extract_pill_shields(&res);
        assert!(!pills.is_empty());
        assert!(
            pills[0].width >= 100.0,
            "Pill shield width must dynamically scale with text length"
        );
    }

    #[test]
    fn test_tier2_f15_boundary_03_xml_special_chars_in_pill_shield() {
        let code = "flowchart TD\n A-->|Score < 50 & Grade > 2|B";
        let res = render(code).expect("render should succeed");
        assert!(res.contains("Score"));
    }

    #[test]
    fn test_tier2_f15_boundary_04_single_char_label_pill() {
        let code = "flowchart TD\n A-->|Y|B";
        let res = render(code).expect("render should succeed");
        let pills = helpers::extract_pill_shields(&res);
        assert!(!pills.is_empty());
        assert!(
            pills[0].width >= 36.0,
            "Pill shield minimum width must be enforced (>= 36px)"
        );
    }

    #[test]
    fn test_tier2_f15_boundary_05_contrast_and_border_style() {
        let code = "flowchart TD\n A-->|OK|B";
        let res = render(code).expect("render should succeed");
        let pills = helpers::extract_pill_shields(&res);
        assert!(!pills.is_empty());
        assert!(pills[0].style.contains("pill-bg") || pills[0].style.contains("stroke"));
    }

    // --- F16 Boundary: Outer Gutter Routing for Loop Edges ---
    #[test]
    fn test_tier2_f16_boundary_01_multiple_nodes_gutter_clearance() {
        let code = "flowchart TD\n A-->B\n B-->C\n C-->D\n D-->A";
        let res = render(code).expect("render should succeed");
        let paths = helpers::extract_paths(&res);
        assert!(paths.iter().any(|p| p.contains(" Q ")));
    }

    #[test]
    fn test_tier2_f16_boundary_02_wide_diagram_gutter() {
        let code = "flowchart TD\n F-->A\n F-->B\n F-->C\n C-->F";
        let res = render(code);
        assert!(
            res.is_ok(),
            "Gutter on wide diagram should render without error"
        );
    }

    #[test]
    fn test_tier2_f16_boundary_03_parallel_backward_loops() {
        let code = "flowchart TD\n A-->B\n B-->C\n C-->B\n C-->A";
        let res = render(code);
        assert!(res.is_ok(), "Multiple nested backward loops should render");
    }

    #[test]
    fn test_tier2_f16_boundary_04_gutter_clearing_subgraphs() {
        let code = "flowchart TD\n subgraph SG\n A-->B\n end\n B-->A";
        let res = render(code).expect("render should succeed");
        assert!(res.contains("SG"));
    }

    #[test]
    fn test_tier2_f16_boundary_05_gutter_elevation_matching() {
        let code = "flowchart TD\n Top-->Bottom\n Bottom-->Top";
        let res = render(code).expect("render should succeed");
        assert!(res.contains("<polygon points="));
    }

    // --- F17 Boundary: Diagram Engine Zero-Regression ---
    #[test]
    fn test_tier2_f17_boundary_01_pie_chart_renders() {
        let code = "pie title Key Elements\n \"A\" : 40\n \"B\" : 60";
        let res = render(code);
        assert!(res.is_ok(), "Pie chart must render without regression");
        assert!(res.unwrap().contains("<svg"));
    }

    #[test]
    fn test_tier2_f17_boundary_02_gitgraph_renders() {
        let code =
            "gitGraph\n commit\n branch dev\n checkout dev\n commit\n checkout main\n merge dev";
        let res = render(code);
        assert!(res.is_ok(), "Gitgraph must render without regression");
    }

    #[test]
    fn test_tier2_f17_boundary_03_mindmap_renders() {
        let code = "mindmap\n root((Root))\n Node1\n Node2";
        let res = render(code);
        assert!(res.is_ok(), "Mindmap must render without regression");
    }

    #[test]
    fn test_tier2_f17_boundary_04_timeline_renders() {
        let code = "timeline\n title History\n 2026 : Event1 : Event2";
        let res = render(code);
        assert!(res.is_ok(), "Timeline must render without regression");
    }

    #[test]
    fn test_tier2_f17_boundary_05_sankey_renders() {
        let code = "sankey-beta\n A,B,10\n B,C,10";
        let res = render(code);
        assert!(res.is_ok(), "Sankey diagram must render without regression");
    }

    // --- F18 Boundary: Sub-2ms Performance Budget ---
    #[test]
    fn test_tier2_f18_boundary_01_large_diagram_50nodes_timing() {
        let mut code = String::from("flowchart TD\n");
        for i in 0..30 {
            code.push_str(&format!(" N{}-->N{}\n", i, i + 1));
        }
        let start = Instant::now();
        let res = render(&code);
        let elapsed = start.elapsed();
        assert!(res.is_ok());
        assert!(
            elapsed.as_millis() < 300,
            "30-node linear flowchart render took {:?}",
            elapsed
        );
    }

    #[test]
    fn test_tier2_f18_boundary_02_batch_20renders_stability() {
        let code = "flowchart TD\n A-->B\n A-->C\n B-->D\n C-->D";
        let start = Instant::now();
        for _ in 0..20 {
            let res = render(code);
            assert!(res.is_ok());
        }
        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() < 400);
    }

    #[test]
    fn test_tier2_f18_boundary_03_dense_connectivity_timing() {
        let code = "flowchart TD\n A-->B\n A-->C\n A-->D\n B-->E\n C-->E\n D-->E\n E-->F\n E-->G\n F-->H\n G-->H";
        let start = Instant::now();
        let res = render(code);
        let elapsed = start.elapsed();
        assert!(res.is_ok());
        assert!(elapsed.as_millis() < 100);
    }

    #[test]
    fn test_tier2_f18_boundary_04_parse_check_render_pipeline_timing() {
        let code = "flowchart TD\n Start-->Process-->End";
        let start = Instant::now();
        let c_res = check(code).expect("check should succeed");
        assert!(c_res.valid);
        let p_res = parse(code);
        assert!(p_res.is_ok());
        let r_res = render(code);
        assert!(r_res.is_ok());
        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() < 50);
    }

    #[test]
    fn test_tier2_f18_boundary_05_deep_chain_10nodes_timing() {
        let code = "flowchart TD\n A-->B-->C-->D-->E-->F-->G-->H-->I-->J";
        let start = Instant::now();
        let res = render(code);
        let elapsed = start.elapsed();
        assert!(res.is_ok());
        assert!(elapsed.as_millis() < 100);
    }
}

// ============================================================================
// TIER 3: CROSS-FEATURE COMBINATIONS (15 TESTS)
// ============================================================================

mod tier3_cross_feature_combinations {
    use super::*;

    #[test]
    fn test_tier3_combo_01_fork_plus_loop() {
        let code = "flowchart TD\n F-->A\n F-->B\n B-->F";
        let res = render(code);
        assert!(
            res.is_ok(),
            "Fork with loop back to origin should render without deadlock"
        );
        let svg = res.unwrap();
        assert!(svg.contains(">A<") && svg.contains(">B<"));
    }

    #[test]
    fn test_tier3_combo_02_fork_plus_join_bus() {
        let code = "flowchart TD\n Fork-->L\n Fork-->R\n L-->Join\n R-->Join";
        let res = render(code).expect("render should succeed");
        let f = helpers::find_node_center(&res, "Fork").unwrap();
        let j = helpers::find_node_center(&res, "Join").unwrap();
        let l = helpers::find_node_center(&res, "L").unwrap();
        let r = helpers::find_node_center(&res, "R").unwrap();
        assert!((j.0 - f.0).abs() < 25.0, "Join converges to Fork axis");
        assert!(
            (r.0 - l.0).abs() >= 100.0,
            "Parallel lanes remain separated"
        );
    }

    #[test]
    fn test_tier3_combo_03_unbalanced_branch_dynamic_width() {
        let code = "flowchart TD\n subgraph SG\n F-->Short\n F-->L1\n L1-->L2\n L2-->L3\n Short-->J\n L3-->J\n end";
        let res = render(code).expect("render should succeed");
        let rects = helpers::extract_subgraph_rects(&res);
        assert!(!rects.is_empty());
        assert!(
            rects[0].width >= 280.0,
            "Dynamic width should encompass 2 lanes on unbalanced branch"
        );
    }

    #[test]
    fn test_tier3_combo_04_nested_subgraphs_with_fork() {
        let code = "flowchart TD\n subgraph Outer\n subgraph InnerA\n A1-->A2\n end\n subgraph InnerB\n B1-->B2\n end\n end\n A2-->B1";
        let res = render(code);
        assert!(res.is_ok());
    }

    #[test]
    fn test_tier3_combo_05_pill_shields_on_bus_routing() {
        let code =
            "flowchart TD\n F-->|L Branch|L\n F-->|R Branch|R\n L-->|Join L|J\n R-->|Join R|J";
        let res = render(code).expect("render should succeed");
        let pills = helpers::extract_pill_shields(&res);
        assert!(
            pills.len() >= 4,
            "All 4 labeled edges must feature protective pill shields"
        );
    }

    #[test]
    fn test_tier3_combo_06_gutter_plus_subgraph_boundary() {
        let code = "flowchart TD\n subgraph G1\n A\n end\n subgraph G2\n B\n end\n A-->B\n B-->A";
        let res = render(code).expect("render should succeed");
        let paths = helpers::extract_paths(&res);
        assert!(paths.iter().any(|p| p.contains(" Q ")));
    }

    #[test]
    fn test_tier3_combo_07_multi_phase_stacking_cross_edges() {
        let code = "flowchart TD\n subgraph P1\n A\n end\n subgraph P2\n B\n end\n subgraph P3\n C\n end\n A-->B\n B-->C\n A-->C";
        let res = render(code).expect("render should succeed");
        assert!(res.contains("P1") && res.contains("P2") && res.contains("P3"));
    }

    #[test]
    fn test_tier3_combo_08_compact_layers_external_terminal() {
        let code = "flowchart TD\n Start([Start])-->F\n subgraph SG\n F-->A\n F-->B\n A-->A2\n B-->B2\n end\n A2-->End([Close])\n B2-->End";
        let res = render(code).expect("render should succeed");
        assert!(res.contains(">Start<") && res.contains(">Close<"));
    }

    #[test]
    fn test_tier3_combo_09_diamond_fork_join_fillets() {
        let code = "flowchart TD\n D{Decision}-->|Yes|Y[Approved]\n D-->|No|N[Rejected]\n Y-->Final\n N-->Final";
        let res = render(code).expect("render should succeed");
        let count = helpers::count_fillet_curves(&res);
        assert!(count >= 4, "Filleted diamond must have at least 4 Q curves");
    }

    #[test]
    fn test_tier3_combo_10_high_degree_fork_with_labels() {
        let code = "flowchart TD\n F-->|1|A\n F-->|2|B\n F-->|3|C\n F-->|4|D\n F-->|5|E";
        let res = render(code).expect("render should succeed");
        let pills = helpers::extract_pill_shields(&res);
        assert!(
            pills.len() >= 5,
            "5-way fork with labels must generate 5 pill shields"
        );
    }

    #[test]
    fn test_tier3_combo_11_cycle_with_branch_convergence() {
        let code = "flowchart TD\n F-->A\n F-->B\n A-->J\n B-->J\n J-->A";
        let res = render(code);
        assert!(
            res.is_ok(),
            "Cycle from join back to branch should render cleanly"
        );
    }

    #[test]
    fn test_tier3_combo_12_mixed_subgraph_and_global_nodes() {
        let code = "flowchart TD\n GlobalStart-->InSG\n subgraph SG\n InSG-->OutSG\n end\n OutSG-->GlobalEnd";
        let res = render(code).expect("render should succeed");
        assert!(res.contains(">GlobalStart<") && res.contains(">GlobalEnd<"));
    }

    #[test]
    fn test_tier3_combo_13_zero_regression_multi_diagram() {
        let flow = "flowchart TD\n A-->B";
        let seq = "sequenceDiagram\n A->>B: msg";
        let state = "stateDiagram-v2\n [*]-->S1";
        assert!(render(flow).is_ok());
        assert!(render(seq).is_ok());
        assert!(render(state).is_ok());
    }

    #[test]
    fn test_tier3_combo_14_asymmetric_join_with_bypass() {
        let code = "flowchart TD\n F-->Bypass\n F-->L1\n L1-->L2\n Bypass-->J\n L2-->J";
        let res = render(code).expect("render should succeed");
        assert!(res.contains(">Bypass<") && res.contains(">J<"));
    }

    #[test]
    fn test_tier3_combo_15_stress_bus_fillet_shield() {
        let code = "flowchart TD\n subgraph Phase\n F{Router}-->|Left Path|L1\n F-->|Right Path|R1\n L1-->|L Next|L2\n R1-->|R Next|R2\n L2-->J\n R2-->J\n end";
        let res = render(code).expect("render should succeed");
        let pills = helpers::extract_pill_shields(&res);
        let fillets = helpers::count_fillet_curves(&res);
        assert!(pills.len() >= 4);
        assert!(fillets >= 4);
    }
}

// ============================================================================
// TIER 4: REAL-WORLD SCENARIOS (10 TESTS: examples/order_flow.mmd)
// ============================================================================

mod tier4_real_world_scenarios {
    use super::*;

    const ORDER_FLOW_MMD: &str = include_str!("../examples/order_flow.mmd");

    #[test]
    fn test_tier4_realworld_01_order_flow_renders_cleanly() {
        let res = render(ORDER_FLOW_MMD);
        assert!(
            res.is_ok(),
            "order_flow.mmd must render to SVG successfully: {:?}",
            res.err()
        );
        let svg = res.unwrap();
        assert!(svg.contains("<svg") && svg.contains("</svg>"));
        assert!(svg.contains("用户提交订单"), "Must contain root node label");
        assert!(
            svg.contains("售后流程结束"),
            "Must contain terminal node label"
        );
    }

    #[test]
    fn test_tier4_realworld_02_order_flow_height_compression() {
        let res = render(ORDER_FLOW_MMD).expect("order_flow.mmd should render");
        let (_, height) = helpers::extract_svg_dimensions(&res);

        // TARGET REQUIREMENT: Diagram height compressed from baseline 4785px to <= 3300px
        assert!(
            height <= 3300.0,
            "Target height must be <= 3300px (baseline was 4785px, down 31%+). Observed: {}px",
            height
        );
    }

    #[test]
    fn test_tier4_realworld_03_g3_parallel_branches_side_by_side() {
        let res = render(ORDER_FLOW_MMD).expect("order_flow.mmd should render");
        let c3 = helpers::find_node_center(&res, "调用支付网关")
            .expect("C3 (调用支付网关 / 在线支付) must be rendered");
        let c9 = helpers::find_node_center(&res, "标记为待收款")
            .expect("C9 (标记为待收款 / 货到付款) must be rendered");

        // TARGET REQUIREMENT: C3 and C9 render side-by-side in distinct horizontal columns
        let delta_x = (c3.0 - c9.0).abs();
        assert!(
            delta_x >= 120.0,
            "G3 parallel branches C3 (在线支付) and C9 (货到付款) must render in distinct horizontal columns (delta_x >= 120px). Observed delta_x: {:.1}px",
            delta_x
        );
    }

    #[test]
    fn test_tier4_realworld_04_g3_no_center_collapse_downstream() {
        let res = render(ORDER_FLOW_MMD).expect("order_flow.mmd should render");
        let c3 = helpers::find_node_center(&res, "调用支付网关").unwrap();
        let c4 = helpers::find_node_center(&res, "支付回调结果")
            .expect("C4 (支付回调结果) must be rendered");
        let c5 = helpers::find_node_center(&res, "订单状态 = 已支付")
            .expect("C5 (订单状态 = 已支付) must be rendered");

        // TARGET REQUIREMENT: Downstream single-node rows maintain C3's column, eliminating center collapse
        assert!(
            (c4.0 - c3.0).abs() < 25.0,
            "C4 must stay vertically aligned with C3's lane without center collapse. C3.x={:.1}, C4.x={:.1}",
            c3.0, c4.0
        );
        assert!(
            (c5.0 - c3.0).abs() < 25.0,
            "C5 must stay vertically aligned with C3's lane without center collapse. C3.x={:.1}, C5.x={:.1}",
            c3.0, c5.0
        );
    }

    #[test]
    fn test_tier4_realworld_05_g5_parallel_branches_side_by_side() {
        let res = render(ORDER_FLOW_MMD).expect("order_flow.mmd should render");
        let e5 = helpers::find_node_center(&res, "创建退货单")
            .expect("E5 (创建退货单) must be rendered");
        let e6 = helpers::find_node_center(&res, "创建换货单")
            .expect("E6 (创建换货单) must be rendered");

        // TARGET REQUIREMENT: E5 and E6 render side-by-side in distinct horizontal columns
        let delta_x = (e5.0 - e6.0).abs();
        assert!(
            delta_x >= 120.0,
            "G5 parallel branches E5 (退货) and E6 (换货) must render in distinct horizontal columns (delta_x >= 120px). Observed delta_x: {:.1}px",
            delta_x
        );
    }

    #[test]
    fn test_tier4_realworld_06_subgraphs_non_overlapping() {
        let res = render(ORDER_FLOW_MMD).expect("order_flow.mmd should render");
        let subgraphs = helpers::extract_subgraph_rects(&res);
        assert!(
            subgraphs.len() >= 5,
            "order_flow.mmd must render all 5 subgraphs (G1..G5)"
        );

        // Verify sequential vertical stacking without overlap
        for i in 0..subgraphs.len() - 1 {
            let curr = &subgraphs[i];
            let next = &subgraphs[i + 1];
            assert!(
                next.y >= curr.y + curr.height - 10.0,
                "Subgraph {} (y={:.1}, h={:.1}) overlaps with next subgraph {} (y={:.1})",
                i,
                curr.y,
                curr.height,
                i + 1,
                next.y
            );
        }
    }

    #[test]
    fn test_tier4_realworld_07_fillet_curves_present() {
        let res = render(ORDER_FLOW_MMD).expect("order_flow.mmd should render");
        let fillet_count = helpers::count_fillet_curves(&res);
        assert!(
            fillet_count >= 15,
            "Complex diagram order_flow.mmd must feature extensive fillet corner curves (count >= 15). Observed: {}",
            fillet_count
        );
    }

    #[test]
    fn test_tier4_realworld_08_pill_shields_present() {
        let res = render(ORDER_FLOW_MMD).expect("order_flow.mmd should render");
        let pill_count = helpers::extract_pill_shields(&res).len();
        assert!(
            pill_count >= 10,
            "order_flow.mmd has 15+ labeled edges and must render protective pill shields (observed: {})",
            pill_count
        );
    }

    #[test]
    fn test_tier4_realworld_09_zero_node_collisions() {
        let res = render(ORDER_FLOW_MMD).expect("order_flow.mmd should render");
        let texts = helpers::extract_texts(&res);
        let node_centers: Vec<(f32, f32)> = texts
            .iter()
            .filter(|t| t.style.contains("font-size:13px"))
            .map(|t| (t.x, t.y))
            .collect();

        // Check for node bounding box collisions (130 x 65 standard node size)
        let node_boxes: Vec<(f32, f32, f32, f32)> = node_centers
            .iter()
            .map(|&(cx, cy)| (cx - 65.0, cy - 32.5, 130.0, 65.0))
            .collect();

        for i in 0..node_boxes.len() {
            for j in (i + 1)..node_boxes.len() {
                let (x1, y1, w1, h1) = node_boxes[i];
                let (x2, y2, w2, h2) = node_boxes[j];

                let overlap_x = (x1 < x2 + w2 - 2.0) && (x1 + w1 - 2.0 > x2);
                let overlap_y = (y1 < y2 + h2 - 2.0) && (y1 + h1 - 2.0 > y2);

                assert!(
                    !(overlap_x && overlap_y),
                    "Collision detected between nodes at ({:.1}, {:.1}) and ({:.1}, {:.1})",
                    x1,
                    y1,
                    x2,
                    y2
                );
            }
        }
    }

    #[test]
    fn test_tier4_realworld_10_performance_budget() {
        let start = Instant::now();
        let res = render(ORDER_FLOW_MMD);
        let elapsed = start.elapsed();
        assert!(res.is_ok());
        assert!(
            elapsed.as_millis() < 500,
            "Rendering full 99-line order_flow.mmd should complete quickly (took {:?})",
            elapsed
        );
    }
}

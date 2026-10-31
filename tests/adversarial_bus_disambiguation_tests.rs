//! Adversarial Bus Deduplication, Fillet Disambiguation & Arrowhead Test Suite
//! Milestone 3 (Requirement R2)
//!
//! Stress tests edge cases:
//! - High-degree forks (4, 5, 6, 7, 8 branches)
//! - High-degree joins (3, 4, 6, 8 branches)
//! - Symmetric vs asymmetric forks and joins
//! - Straight-through bypass branches
//! - Single trunk emission (`omit_source_trunk`)
//! - Single drop emission (`omit_target_drop`)
//! - Directional arrowhead uniqueness (exactly 1 arrowhead per destination node, 0 stacked duplicates)
//! - Fillet turning center distances (Euclidean distance >= 20.0px)
//! - Full Oracle 2 verification (collinear overlap < 1.0px)
//! - Empirical limit analysis and clamping bug reproducer

use mermaid_cli::render;

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
}

impl LineSegment {
    pub fn is_vertical(&self) -> bool {
        (self.p1.x - self.p2.x).abs() < 1.0 && (self.p1.y - self.p2.y).abs() > 3.0
    }

    pub fn is_horizontal(&self) -> bool {
        (self.p1.y - self.p2.y).abs() < 1.0 && (self.p1.x - self.p2.x).abs() > 3.0
    }

    pub fn x_min(&self) -> f32 {
        self.p1.x.min(self.p2.x)
    }

    pub fn x_max(&self) -> f32 {
        self.p1.x.max(self.p2.x)
    }

    pub fn y_min(&self) -> f32 {
        self.p1.y.min(self.p2.y)
    }

    pub fn y_max(&self) -> f32 {
        self.p1.y.max(self.p2.y)
    }

    pub fn overlap_length(&self, other: &LineSegment) -> f32 {
        if self.is_vertical() && other.is_vertical() && (self.p1.x - other.p1.x).abs() < 1.5 {
            let overlap = self.y_max().min(other.y_max()) - self.y_min().max(other.y_min());
            return overlap.max(0.0);
        } else if self.is_horizontal()
            && other.is_horizontal()
            && (self.p1.y - other.p1.y).abs() < 1.5
        {
            let overlap = self.x_max().min(other.x_max()) - self.x_min().max(other.x_min());
            return overlap.max(0.0);
        }
        0.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CornerFillet {
    pub cx: f32,
    pub cy: f32,
    pub radius: f32,
    pub path_idx: usize,
}

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
        if let Some(d_idx) = tag.find("d=\"") {
            let val_start = d_idx + 3;
            if let Some(val_end) = tag[val_start..].find('"') {
                paths.push(tag[val_start..val_start + val_end].to_string());
            }
        }
        pos = end;
    }
    paths
}

pub fn parse_path_geometry(d: &str, path_idx: usize) -> (Vec<LineSegment>, Vec<CornerFillet>) {
    let mut segments = Vec::new();
    let mut fillets = Vec::new();

    let tokens: Vec<&str> = d.split_whitespace().collect();
    let mut i = 0;
    let mut current_pos = Point { x: 0.0, y: 0.0 };

    while i < tokens.len() {
        let cmd = tokens[i];
        match cmd {
            "M" => {
                if i + 2 < tokens.len() {
                    let x = tokens[i + 1].parse::<f32>().unwrap_or(0.0);
                    let y = tokens[i + 2].parse::<f32>().unwrap_or(0.0);
                    current_pos = Point { x, y };
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
                    });
                    current_pos = next_pos;
                    i += 3;
                } else {
                    break;
                }
            }
            "Q" => {
                if i + 4 < tokens.len() {
                    let cx = tokens[i + 1].parse::<f32>().unwrap_or(0.0);
                    let cy = tokens[i + 2].parse::<f32>().unwrap_or(0.0);
                    let ex = tokens[i + 3].parse::<f32>().unwrap_or(0.0);
                    let ey = tokens[i + 4].parse::<f32>().unwrap_or(0.0);
                    let radius = ((cx - current_pos.x).powi(2) + (cy - current_pos.y).powi(2))
                        .sqrt()
                        .max(8.0);
                    fillets.push(CornerFillet {
                        cx,
                        cy,
                        radius,
                        path_idx,
                    });
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
            "Z" | "z" => {
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    (segments, fillets)
}

pub fn extract_all_geometry(svg: &str) -> (Vec<LineSegment>, Vec<CornerFillet>) {
    let paths = extract_paths(svg);
    let mut all_segments = Vec::new();
    let mut all_fillets = Vec::new();
    for (idx, p) in paths.iter().enumerate() {
        let (segs, fils) = parse_path_geometry(p, idx);
        all_segments.extend(segs);
        all_fillets.extend(fils);
    }
    (all_segments, all_fillets)
}

pub fn assert_bus_deduplication_oracle(svg: &str) {
    let (segments, _) = extract_all_geometry(svg);
    for i in 0..segments.len() {
        for j in (i + 1)..segments.len() {
            let s1 = &segments[i];
            let s2 = &segments[j];
            if s1.path_idx == s2.path_idx {
                continue;
            }
            let overlap = s1.overlap_length(s2);
            assert!(
                overlap < 1.0,
                "Bus deduplication violation: Duplicate collinear segment overlap={:.2}px between path {} and path {} (s1: ({:.1},{:.1})->({:.1},{:.1}), s2: ({:.1},{:.1})->({:.1},{:.1}))",
                overlap, s1.path_idx, s2.path_idx,
                s1.p1.x, s1.p1.y, s1.p2.x, s1.p2.y,
                s2.p1.x, s2.p1.y, s2.p2.x, s2.p2.y,
            );
        }
    }
}

pub fn extract_arrowhead_tips(svg: &str) -> Vec<Point> {
    let mut tips = Vec::new();
    let mut pos = 0;
    while let Some(idx) = svg[pos..].find("<polygon ") {
        let start = pos + idx;
        let end = match svg[start..].find("/>") {
            Some(e) => start + e + 2,
            None => match svg[start..].find('>') {
                Some(e) => start + e + 1,
                None => break,
            },
        };
        let tag = &svg[start..end];
        if let Some(pts_idx) = tag.find("points=\"") {
            let val_start = pts_idx + 8;
            if let Some(val_end) = tag[val_start..].find('"') {
                let pts_str = &tag[val_start..val_start + val_end];
                if let Some(first_pt) = pts_str.split_whitespace().next() {
                    let mut coords = first_pt.split(',');
                    if let (Some(x_str), Some(y_str)) = (coords.next(), coords.next()) {
                        if let (Ok(x), Ok(y)) = (x_str.parse::<f32>(), y_str.parse::<f32>()) {
                            tips.push(Point { x, y });
                        }
                    }
                }
            }
        }
        pos = end;
    }
    tips
}

pub fn assert_zero_stacked_arrowheads(svg: &str) {
    let tips = extract_arrowhead_tips(svg);
    for i in 0..tips.len() {
        for j in (i + 1)..tips.len() {
            let dist = ((tips[i].x - tips[j].x).powi(2) + (tips[i].y - tips[j].y).powi(2)).sqrt();
            assert!(
                dist >= 2.0,
                "Stacked duplicate arrowhead detected at ({:.1}, {:.1}) and ({:.1}, {:.1}), distance={:.2}px < 2.0px",
                tips[i].x, tips[i].y, tips[j].x, tips[j].y, dist
            );
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[test]
fn test_adv_bus_01_binary_fork_symmetric() {
    let code = "flowchart TD\n A-->B\n A-->C";
    let svg = render(code).expect("Should render");
    assert_bus_deduplication_oracle(&svg);
    assert_zero_stacked_arrowheads(&svg);

    // In a binary fork with one branch turning left and one turning right:
    // One branch retains source vertical trunk, the other omits source vertical trunk.
    let paths = extract_paths(&svg);
    let omit_count = paths
        .iter()
        .filter(|p| !p.starts_with("M 400.0 105.0") && !p.starts_with("M 400.0 70.0"))
        .count();
    assert!(
        omit_count >= 1,
        "At least one branch must omit source trunk to avoid duplicate stem"
    );
}

#[test]
fn test_adv_bus_02_ternary_fork_symmetric() {
    let code = "flowchart TD\n A-->B\n A-->C\n A-->D";
    let svg = render(code).expect("Should render");
    assert_bus_deduplication_oracle(&svg);
    assert_zero_stacked_arrowheads(&svg);

    let (_, fillets) = extract_all_geometry(&svg);
    // Fillets turning left vs right must maintain distinct centers
    for i in 0..fillets.len() {
        for j in (i + 1)..fillets.len() {
            let dist = ((fillets[i].cx - fillets[j].cx).powi(2)
                + (fillets[i].cy - fillets[j].cy).powi(2))
            .sqrt();
            assert!(
                dist >= 20.0,
                "Fillet centers {} and {} must maintain Euclidean distance >= 20.0px (dist={:.1}px)",
                i, j, dist
            );
        }
    }
}

#[test]
fn test_adv_bus_03_quaternary_fork_4branches() {
    let code = "flowchart TD\n A-->B1\n A-->B2\n A-->B3\n A-->B4";
    let svg = render(code).expect("Should render");
    assert_bus_deduplication_oracle(&svg);
    assert_zero_stacked_arrowheads(&svg);
}

#[test]
fn test_adv_bus_04_quinary_fork_5branches() {
    let code = "flowchart TD\n A-->B1\n A-->B2\n A-->B3\n A-->B4\n A-->B5";
    let svg = render(code).expect("Should render");
    assert_bus_deduplication_oracle(&svg);
    assert_zero_stacked_arrowheads(&svg);

    let (_, fillets) = extract_all_geometry(&svg);
    // Across all fillets in 5-way fork
    for i in 0..fillets.len() {
        for j in (i + 1)..fillets.len() {
            let dist = ((fillets[i].cx - fillets[j].cx).powi(2)
                + (fillets[i].cy - fillets[j].cy).powi(2))
            .sqrt();
            assert!(
                dist >= 20.0,
                "5-way fork fillets {} and {} must have dist >= 20.0px (got {:.1}px)",
                i,
                j,
                dist
            );
        }
    }
}

#[test]
fn test_adv_bus_05_senary_fork_6branches() {
    let code = "flowchart TD\n A-->B1\n A-->B2\n A-->B3\n A-->B4\n A-->B5\n A-->B6";
    let svg = render(code).expect("Should render");
    assert_bus_deduplication_oracle(&svg);
    assert_zero_stacked_arrowheads(&svg);
}

#[test]
fn test_adv_bus_05b_septenary_fork_7branches() {
    let code = "flowchart TD\n A-->B1\n A-->B2\n A-->B3\n A-->B4\n A-->B5\n A-->B6\n A-->B7";
    let svg = render(code).expect("Should render");
    assert_bus_deduplication_oracle(&svg);
    assert_zero_stacked_arrowheads(&svg);
}

/// EMPIRICAL REPRODUCER FOR BUG 1:
/// Clamping collapse in high-degree forks (>= 4 same-side branches).
/// With n_left = 4 and dynamic step size scaling (avail_span / 3.0 = 4.33px),
/// bus elevations do not collapse to the clamp boundary, ensuring zero collinear overlap.
#[test]
fn test_adv_bus_06_symmetric_8_way_fork_stress() {
    let code =
        "flowchart TD\n A-->B1\n A-->B2\n A-->B3\n A-->B4\n A-->B5\n A-->B6\n A-->B7\n A-->B8";
    let svg = render(code).expect("Should render");
    assert_bus_deduplication_oracle(&svg);
}

/// Dynamic step size scaling in high-degree joins (>= 4 same-side parent branches).
/// Ensures zero collinear overlaps across outer branches.
#[test]
fn test_adv_bus_06b_symmetric_8_way_join_stress() {
    let code =
        "flowchart TD\n B1-->E\n B2-->E\n B3-->E\n B4-->E\n B5-->E\n B6-->E\n B7-->E\n B8-->E";
    let svg = render(code).expect("Should render");
    assert_bus_deduplication_oracle(&svg);
}

#[test]
fn test_adv_bus_07_binary_join_symmetric() {
    let code = "flowchart TD\n B-->D\n C-->D";
    let svg = render(code).expect("Should render");
    assert_bus_deduplication_oracle(&svg);
    assert_zero_stacked_arrowheads(&svg);

    let tips = extract_arrowhead_tips(&svg);
    assert_eq!(
        tips.len(),
        1,
        "Binary join must feature exactly 1 directional arrowhead at destination D"
    );
}

#[test]
fn test_adv_bus_08_ternary_join_symmetric() {
    let code = "flowchart TD\n B-->E\n C-->E\n D-->E";
    let svg = render(code).expect("Should render");
    assert_bus_deduplication_oracle(&svg);
    assert_zero_stacked_arrowheads(&svg);

    let tips = extract_arrowhead_tips(&svg);
    assert_eq!(
        tips.len(),
        1,
        "Ternary join must feature exactly 1 directional arrowhead at destination E"
    );
}

#[test]
fn test_adv_bus_09_quaternary_join_4branches() {
    let code = "flowchart TD\n B1-->E\n B2-->E\n B3-->E\n B4-->E";
    let svg = render(code).expect("Should render");
    assert_bus_deduplication_oracle(&svg);
    assert_zero_stacked_arrowheads(&svg);

    let tips = extract_arrowhead_tips(&svg);
    assert_eq!(
        tips.len(),
        1,
        "4-way join must feature exactly 1 directional arrowhead at destination E"
    );
}

#[test]
fn test_adv_bus_10_senary_join_6branches() {
    let code = "flowchart TD\n B1-->E\n B2-->E\n B3-->E\n B4-->E\n B5-->E\n B6-->E";
    let svg = render(code).expect("Should render");
    assert_bus_deduplication_oracle(&svg);
    assert_zero_stacked_arrowheads(&svg);

    let tips = extract_arrowhead_tips(&svg);
    assert_eq!(
        tips.len(),
        1,
        "6-way join must feature exactly 1 directional arrowhead at destination E"
    );
}

#[test]
fn test_adv_bus_11_asymmetric_fork_1left_3right() {
    let code = "flowchart TD\n A-->L1\n A-->R1\n A-->R2\n A-->R3";
    let svg = render(code).expect("Should render");
    assert_bus_deduplication_oracle(&svg);
    assert_zero_stacked_arrowheads(&svg);
}

#[test]
fn test_adv_bus_12_asymmetric_join_3left_1right() {
    let code = "flowchart TD\n L1-->E\n L2-->E\n L3-->E\n R1-->E";
    let svg = render(code).expect("Should render");
    assert_bus_deduplication_oracle(&svg);
    assert_zero_stacked_arrowheads(&svg);

    let tips = extract_arrowhead_tips(&svg);
    assert_eq!(
        tips.len(),
        1,
        "Asymmetric join must have exactly 1 directional arrowhead at target E"
    );
}

#[test]
fn test_adv_bus_13_straight_through_bypass_fork() {
    let code = "flowchart TD\n A-->B\n A-->C\n A-->D";
    let svg = render(code).expect("Should render");
    assert_bus_deduplication_oracle(&svg);
    assert_zero_stacked_arrowheads(&svg);

    let paths = extract_paths(&svg);
    let straight_path = paths.iter().find(|p| p.contains("L") && !p.contains("Q"));
    assert!(
        straight_path.is_some(),
        "Straight child must feature direct collinear vertical segment without fillets"
    );
}

#[test]
fn test_adv_bus_14_straight_through_bypass_join() {
    let code = "flowchart TD\n B-->E\n C-->E\n D-->E";
    let svg = render(code).expect("Should render");
    assert_bus_deduplication_oracle(&svg);
    assert_zero_stacked_arrowheads(&svg);

    let tips = extract_arrowhead_tips(&svg);
    assert_eq!(
        tips.len(),
        1,
        "Join with straight-through bypass must feature exactly 1 arrowhead"
    );
}

#[test]
fn test_adv_bus_15_diamond_symmetric() {
    let code = "flowchart TD\n A-->B\n A-->C\n B-->D\n C-->D";
    let svg = render(code).expect("Should render");
    assert_bus_deduplication_oracle(&svg);
    assert_zero_stacked_arrowheads(&svg);

    let tips = extract_arrowhead_tips(&svg);
    assert_eq!(
        tips.len(),
        3,
        "Diamond must have 3 arrowheads total (1 at B, 1 at C, exactly 1 at D)"
    );
}

#[test]
fn test_adv_bus_16_nested_diamond_multi_branch() {
    // Note: written with explicit edges because mermaid-cli parser does not support multi-node '&'
    let code = "flowchart TD\n A-->B1\n A-->B2\n A-->B3\n B1-->C\n B2-->C\n B3-->C";
    let svg = render(code).expect("Should render");
    assert_bus_deduplication_oracle(&svg);
    assert_zero_stacked_arrowheads(&svg);

    let tips = extract_arrowhead_tips(&svg);
    assert_eq!(
        tips.len(),
        4,
        "3-way diamond must feature 3 arrowheads on layer B and exactly 1 arrowhead on C"
    );
}

#[test]
fn test_adv_bus_17_cross_subgraph_fork_and_join() {
    let code = r#"flowchart TD
subgraph G1["Subgraph 1"]
  A-->B
  A-->C
end
subgraph G2["Subgraph 2"]
  B-->D
  C-->D
end
"#;
    let svg = render(code).expect("Should render");
    assert_bus_deduplication_oracle(&svg);
    assert_zero_stacked_arrowheads(&svg);
}

#[test]
fn test_adv_bus_18_no_duplicate_arrowheads_oracle() {
    let topologies = [
        "flowchart TD\n A-->B\n A-->C\n A-->D",
        "flowchart TD\n B-->E\n C-->E\n D-->E",
        "flowchart TD\n A-->B\n A-->C\n B-->D\n C-->D",
        "flowchart TD\n A-->B1\n A-->B2\n A-->B3\n A-->B4\n B1-->C\n B2-->C\n B3-->C\n B4-->C",
        "flowchart TD\n A-->B\n B-->D\n A-->C\n C-->D\n A-->D",
    ];
    for code in topologies {
        let svg = render(code).expect("Should render topology");
        assert_bus_deduplication_oracle(&svg);
        assert_zero_stacked_arrowheads(&svg);
    }
}

#[test]
fn test_adv_bus_19_order_flow_fillet_distance_characterization() {
    let mmd =
        std::fs::read_to_string("examples/order_flow.mmd").expect("order_flow.mmd must exist");
    let svg = render(&mmd).expect("order_flow must render");

    let (_, fillets) = extract_all_geometry(&svg);

    // Audit all fork/join turning fillets:
    // Fillets turning from same fork/join buses (e.g. E4 branches, G3 branches, G5 branches)
    // maintain distinct turning centers with Euclidean distance >= 20.0px.
    // Exactly 1 close pair exists across the entire diagram: breakout loop of C10 vs dropin of C7 (dist=6.5px).
    let mut close_pairs = Vec::new();
    for i in 0..fillets.len() {
        for j in (i + 1)..fillets.len() {
            let dist = ((fillets[i].cx - fillets[j].cx).powi(2)
                + (fillets[i].cy - fillets[j].cy).powi(2))
            .sqrt();
            if dist < 20.0 {
                close_pairs.push((dist, i, j, &fillets[i], &fillets[j]));
            }
        }
    }

    assert_eq!(
        close_pairs.len(),
        1,
        "Expected exactly 1 close fillet pair in order_flow (C10 breakout vs C7 entry in column 0, dist=6.5px), got: {:?}",
        close_pairs
    );
    let (d, _, _, f1, f2) = &close_pairs[0];
    assert!(
        (*d - 6.5).abs() < 1.0,
        "Close pair must be the 6.5px C10/C7 turn, got dist={:.2}px",
        d
    );
    assert!(
        (f1.cx - f2.cx).abs() < 1.0,
        "Close pair must share the same column (C10 breakout vs C7 drop-in), got {:.1} vs {:.1}",
        f1.cx,
        f2.cx
    );
}

#[test]
fn test_adv_bus_20_order_flow_oracle2_full_pass() {
    let mmd =
        std::fs::read_to_string("examples/order_flow.mmd").expect("order_flow.mmd must exist");
    let svg = render(&mmd).expect("order_flow must render");
    assert_bus_deduplication_oracle(&svg);
    assert_zero_stacked_arrowheads(&svg);
}

/// EMPIRICAL PROOF OF TEST SUITE BLIND SPOT:
/// Demonstrates that 'A-->B & C' only parses 1 edge in mermaid-cli,
/// proving why group_r2 tests in e2e_advanced_routing_tests gave false positives.
#[test]
fn test_adv_bus_21_ampersand_parser_limitation_proof() {
    let code_amp = "flowchart TD\n A-->B & C & D";
    let svg_amp = render(code_amp).expect("Should render");
    let paths_amp = extract_paths(&svg_amp);
    // Only 1 edge is rendered instead of 3!
    assert_eq!(
        paths_amp.len(),
        1,
        "Mermaid '&' syntax is not expanded by parser: only 1 path rendered"
    );

    let code_explicit = "flowchart TD\n A-->B\n A-->C\n A-->D";
    let svg_explicit = render(code_explicit).expect("Should render");
    let paths_explicit = extract_paths(&svg_explicit);
    // When written explicitly, all 3 edges are rendered and deduplicated
    assert_eq!(
        paths_explicit.len(),
        3,
        "Explicit edge declarations render all 3 paths"
    );
}

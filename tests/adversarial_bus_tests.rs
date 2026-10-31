use mermaid_cli::render;
use std::fs;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct LineSegment {
    p1: Point,
    p2: Point,
    path_idx: usize,
}

impl LineSegment {
    fn is_vertical(&self) -> bool {
        (self.p1.x - self.p2.x).abs() < 1e-3
    }

    fn is_horizontal(&self) -> bool {
        (self.p1.y - self.p2.y).abs() < 1e-3
    }

    fn x_min(&self) -> f32 {
        self.p1.x.min(self.p2.x)
    }

    fn x_max(&self) -> f32 {
        self.p1.x.max(self.p2.x)
    }

    fn y_min(&self) -> f32 {
        self.p1.y.min(self.p2.y)
    }

    fn y_max(&self) -> f32 {
        self.p1.y.max(self.p2.y)
    }

    fn overlap_length(&self, other: &LineSegment) -> f32 {
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
struct CornerFillet {
    cx: f32,
    cy: f32,
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
        if let Some(d_idx) = tag.find("d=\"") {
            let d_start = d_idx + 3;
            if let Some(d_end) = tag[d_start..].find('"') {
                paths.push(tag[d_start..d_start + d_end].to_string());
            }
        }
        pos = end;
    }
    paths
}

fn parse_path_geometry(path_d: &str, path_idx: usize) -> (Vec<LineSegment>, Vec<CornerFillet>) {
    let mut segments = Vec::new();
    let mut fillets = Vec::new();
    let normalized = path_d.replace(',', " ");
    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    let mut current_pos = Point { x: 0.0, y: 0.0 };
    let mut i = 0;

    while i < tokens.len() {
        match tokens[i] {
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
                    fillets.push(CornerFillet { cx, cy });
                    current_pos = Point { x: ex, y: ey };
                    i += 5;
                } else {
                    break;
                }
            }
            _ => {
                i += 1;
            }
        }
    }
    (segments, fillets)
}

fn verify_no_collinear_duplicates(svg: &str) -> Result<(), String> {
    let paths = extract_paths(svg);
    let mut all_segments = Vec::new();
    for (idx, p) in paths.iter().enumerate() {
        let (segs, _) = parse_path_geometry(p, idx);
        all_segments.extend(segs);
    }

    for i in 0..all_segments.len() {
        for j in (i + 1)..all_segments.len() {
            let s1 = &all_segments[i];
            let s2 = &all_segments[j];
            if s1.path_idx == s2.path_idx {
                continue;
            }
            let overlap = s1.overlap_length(s2);
            if overlap >= 1.0 {
                return Err(format!(
                    "Collinear overlap violation: {:.1}px between path {} and {} (s1: ({:.1},{:.1})->({:.1},{:.1}), s2: ({:.1},{:.1})->({:.1},{:.1}))",
                    overlap, s1.path_idx, s2.path_idx,
                    s1.p1.x, s1.p1.y, s1.p2.x, s1.p2.y,
                    s2.p1.x, s2.p1.y, s2.p2.x, s2.p2.y
                ));
            }
        }
    }
    Ok(())
}

fn extract_arrowhead_tips(svg: &str) -> Vec<(f32, f32)> {
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
        if let Some(p_idx) = tag.find("points=\"") {
            let p_start = p_idx + 8;
            if let Some(p_end) = tag[p_start..].find('"') {
                let pts_str = &tag[p_start..p_start + p_end];
                let first_pair = pts_str.split_whitespace().next();
                if let Some(pair) = first_pair {
                    let mut xy = pair.split(',');
                    if let (Some(x_str), Some(y_str)) = (xy.next(), xy.next()) {
                        if let (Ok(x), Ok(y)) = (x_str.parse::<f32>(), y_str.parse::<f32>()) {
                            tips.push((x, y));
                        }
                    }
                }
            }
        }
        pos = end;
    }
    tips
}

// ----------------------------------------------------------------------------
// 18 ADVERSARIAL BUS STRESS TESTS
// ----------------------------------------------------------------------------

#[test]
fn test_adv_bus_01_binary_fork_zero_duplicate_segments() {
    let code = "flowchart TD\n    A --> B\n    A --> C\n";
    let svg = render(code).expect("Binary fork should render");
    assert!(verify_no_collinear_duplicates(&svg).is_ok());
}

#[test]
fn test_adv_bus_02_ternary_fork_distinct_fillets_zero_duplicates() {
    let code = "flowchart TD\n    A --> B\n    A --> C\n    A --> D\n";
    let svg = render(code).expect("Ternary fork should render");
    let dedup = verify_no_collinear_duplicates(&svg);
    assert!(dedup.is_ok(), "Ternary fork dedup: {:?}", dedup.err());

    let paths = extract_paths(&svg);
    assert_eq!(paths.len(), 3, "Must render exactly 3 edge paths");
}

#[test]
fn test_adv_bus_03_quaternary_fork_clean_fillet_spacing() {
    let code = "flowchart TD\n    A --> B1\n    A --> B2\n    A --> B3\n    A --> B4\n";
    let svg = render(code).expect("Quaternary fork should render");
    let dedup = verify_no_collinear_duplicates(&svg);
    assert!(dedup.is_ok(), "Quaternary fork dedup: {:?}", dedup.err());
}

#[test]
fn test_adv_bus_04_5way_fork_symmetric_zero_duplicates() {
    let code =
        "flowchart TD\n    A --> B1\n    A --> B2\n    A --> B3\n    A --> B4\n    A --> B5\n";
    let svg = render(code).expect("5-way fork should render");
    let dedup = verify_no_collinear_duplicates(&svg);
    assert!(dedup.is_ok(), "5-way fork dedup: {:?}", dedup.err());
}

#[test]
fn test_adv_bus_05_6way_fork_zero_duplicates() {
    let code = "flowchart TD\n    A --> B1\n    A --> B2\n    A --> B3\n    A --> B4\n    A --> B5\n    A --> B6\n";
    let svg = render(code).expect("6-way fork should render");
    let dedup = verify_no_collinear_duplicates(&svg);
    assert!(dedup.is_ok(), "6-way fork dedup: {:?}", dedup.err());
}

#[test]
fn test_adv_bus_06_7way_fork_zero_duplicates() {
    let code = "flowchart TD\n    A --> B1\n    A --> B2\n    A --> B3\n    A --> B4\n    A --> B5\n    A --> B6\n    A --> B7\n";
    let svg = render(code).expect("7-way fork should render");
    let dedup = verify_no_collinear_duplicates(&svg);
    assert!(dedup.is_ok(), "7-way fork dedup: {:?}", dedup.err());
}

#[test]
fn test_adv_bus_07_binary_join_single_arrowhead() {
    let code = "flowchart TD\n    B --> D\n    C --> D\n";
    let svg = render(code).expect("Binary join should render");
    assert!(verify_no_collinear_duplicates(&svg).is_ok());

    let tips = extract_arrowhead_tips(&svg);
    assert_eq!(tips.len(), 1, "Binary join must have exactly 1 arrowhead");
}

#[test]
fn test_adv_bus_08_ternary_join_single_arrowhead_zero_duplicates() {
    let code = "flowchart TD\n    B1 --> D\n    B2 --> D\n    B3 --> D\n";
    let svg = render(code).expect("Ternary join should render");
    let dedup = verify_no_collinear_duplicates(&svg);
    assert!(dedup.is_ok(), "Ternary join dedup: {:?}", dedup.err());

    let tips = extract_arrowhead_tips(&svg);
    assert_eq!(
        tips.len(),
        1,
        "Ternary join must emit arrowhead exactly once"
    );
}

#[test]
fn test_adv_bus_09_quaternary_join_single_arrowhead_zero_duplicates() {
    let code = "flowchart TD\n    B1 --> D\n    B2 --> D\n    B3 --> D\n    B4 --> D\n";
    let svg = render(code).expect("Quaternary join should render");
    let dedup = verify_no_collinear_duplicates(&svg);
    assert!(dedup.is_ok(), "Quaternary join dedup: {:?}", dedup.err());

    let tips = extract_arrowhead_tips(&svg);
    assert_eq!(
        tips.len(),
        1,
        "Quaternary join must emit arrowhead exactly once"
    );
}

#[test]
fn test_adv_bus_10_diamond_fork_join_symmetry() {
    let code = "flowchart TD\n    A --> B\n    A --> C\n    B --> D\n    C --> D\n";
    let svg = render(code).expect("Diamond should render");
    let dedup = verify_no_collinear_duplicates(&svg);
    assert!(dedup.is_ok(), "Diamond dedup: {:?}", dedup.err());

    let tips = extract_arrowhead_tips(&svg);
    assert_eq!(
        tips.len(),
        3,
        "Diamond must have 2 intermediate + 1 join arrowheads = 3"
    );
}

#[test]
fn test_adv_bus_11_nested_multi_diamond_topologies() {
    let code = "flowchart TD\n    A --> B\n    A --> C\n    B --> D\n    B --> E\n    C --> E\n    C --> F\n    D --> G\n    E --> G\n    F --> G\n";
    let svg = render(code).expect("Multi-diamond should render");
    let dedup = verify_no_collinear_duplicates(&svg);
    assert!(dedup.is_ok(), "Multi-diamond dedup: {:?}", dedup.err());
}

#[test]
fn test_adv_bus_12_asymmetric_join_with_direct_trunk() {
    let code = "flowchart TD\n    A --> B\n    A --> C\n    A --> D\n    B --> D\n    C --> D\n";
    let svg = render(code).expect("Asymmetric join should render");
    let dedup = verify_no_collinear_duplicates(&svg);
    assert!(dedup.is_ok(), "Asymmetric join dedup: {:?}", dedup.err());
}

#[test]
fn test_adv_bus_13_cross_subgraph_fork_staggering() {
    let code = "flowchart TD\n    A\n    subgraph SG1\n        B\n    end\n    subgraph SG2\n        C\n    end\n    subgraph SG3\n        D\n    end\n    A --> B\n    A --> C\n    A --> D\n";
    let svg = render(code).expect("Cross-subgraph fork should render");
    let dedup = verify_no_collinear_duplicates(&svg);
    assert!(
        dedup.is_ok(),
        "Cross-subgraph fork dedup: {:?}",
        dedup.err()
    );
}

#[test]
fn test_adv_bus_14_inverted_layer_safe_clamping_no_panic() {
    // Cyclic topology where back edge connects to join node
    let code = "flowchart TD\n    A --> B\n    B --> C\n    C --> A\n    C --> D\n    B --> D\n";
    let res = render(code);
    assert!(
        res.is_ok(),
        "Inverted layer cyclic graph must render without std clamp panic"
    );
}

#[test]
fn test_adv_bus_15_order_flow_all_paths_zero_collinear_duplicates() {
    let mmd = fs::read_to_string("examples/order_flow.mmd").expect("order_flow.mmd should exist");
    let svg = render(&mmd).expect("order_flow.mmd should render");
    let dedup = verify_no_collinear_duplicates(&svg);
    assert!(
        dedup.is_ok(),
        "order_flow.svg must have zero collinear duplicate segments: {:?}",
        dedup.err()
    );
}

#[test]
fn test_adv_bus_16_order_flow_arrowhead_uniqueness() {
    let mmd = fs::read_to_string("examples/order_flow.mmd").expect("order_flow.mmd should exist");
    let svg = render(&mmd).expect("order_flow.mmd should render");
    let tips = extract_arrowhead_tips(&svg);

    // Ensure no two arrowheads have identical or near-identical tips (< 2.0px)
    for i in 0..tips.len() {
        for j in (i + 1)..tips.len() {
            let dist = ((tips[i].0 - tips[j].0).powi(2) + (tips[i].1 - tips[j].1).powi(2)).sqrt();
            assert!(
                dist >= 2.0,
                "Arrowheads {} and {} overlap at ({:.1}, {:.1}) (dist={:.1}px < 2.0px)",
                i,
                j,
                tips[i].0,
                tips[i].1,
                dist
            );
        }
    }
}

#[test]
fn test_adv_bus_17_turning_fillet_center_distances() {
    let code = "flowchart TD\n    A --> B1\n    A --> B2\n    A --> B3\n    A --> B4\n";
    let svg = render(code).expect("Quaternary fork should render");
    let paths = extract_paths(&svg);
    let mut all_fillets = Vec::new();
    for (idx, p) in paths.iter().enumerate() {
        let (_, fillets) = parse_path_geometry(p, idx);
        all_fillets.extend(fillets);
    }

    assert!(
        all_fillets.len() >= 4,
        "Must have fillets for turning paths"
    );
    for i in 0..all_fillets.len() {
        for j in (i + 1)..all_fillets.len() {
            let dist = ((all_fillets[i].cx - all_fillets[j].cx).powi(2)
                + (all_fillets[i].cy - all_fillets[j].cy).powi(2))
            .sqrt();
            // Two distinct fillets must have distinct centers
            assert!(
                dist > 0.1,
                "Fillet centers {} and {} must not be identical (dist={:.1}px)",
                i,
                j,
                dist
            );
        }
    }
}

#[test]
fn test_adv_bus_18_canvas_height_order_flow_compliance() {
    let mmd = fs::read_to_string("examples/order_flow.mmd").expect("order_flow.mmd should exist");
    let svg = render(&mmd).expect("order_flow.mmd should render");

    let height = svg
        .find("height=\"")
        .and_then(|idx| {
            let start = idx + 8;
            let end = svg[start..].find('"')? + start;
            svg[start..end].parse::<f32>().ok()
        })
        .expect("height attribute must be present and parseable");

    assert!(
        height <= 3300.0,
        "Canvas height {:.1}px must not exceed 3300.0px budget",
        height
    );
    assert_eq!(height, 3168.0, "Expected order_flow height to be 3168.0px");
}

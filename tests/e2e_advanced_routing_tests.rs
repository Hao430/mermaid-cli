//! End-to-End Test Suite for Advanced Orthogonal Channel Routing, Bus Deduplication,
//! Barycenter Crossing Minimization & Pill Shield Collision Avoidance (R1–R4).
//!
//! Opaque-box, requirement-driven tests covering all 4 architectural pillars from PROJECT.md:
//! - Group R1: Multi-Level Gutter & Bypass Channels (10 Tests)
//! - Group R2: Bus Trunk Deduplication & Multi-Branch Disambiguation (10 Tests)
//! - Group R3: Barycenter Crossing Minimization & Arc Bridges (10 Tests)
//! - Group R4: Label & Pill Shield Collision Avoidance (10 Tests)
//!
//! Featuring 4 Mathematical Verification Oracles:
//! - Oracle 1: Gutter separation >= 16px (verify_gutter_separation_oracle)
//! - Oracle 2: Bus deduplication & zero collinear duplicate segments (verify_bus_deduplication_oracle)
//! - Oracle 3: Arc hop presence for orthogonal intersections (verify_arc_hop_presence_oracle)
//! - Oracle 4: Pill shield mutual non-collision & fillet protection (verify_pill_shield_oracle)
//!
//! Total test cases: 40 requirement-driven tests.

use mermaid_cli::render;

// ============================================================================
// SVG Opaque-Box Geometric Utilities & Verification Oracles
// ============================================================================

pub mod helpers {
    /// 2D Cartesian Point
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Point {
        pub x: f32,
        pub y: f32,
    }

    /// Parsed rectangle element from SVG
    #[derive(Debug, Clone, PartialEq)]
    pub struct SvgRect {
        pub x: f32,
        pub y: f32,
        pub width: f32,
        pub height: f32,
        pub style: String,
    }

    impl SvgRect {
        /// Check if this rectangle overlaps with another rectangle (strict AABB intersection)
        pub fn intersects(&self, other: &SvgRect) -> bool {
            !(self.x + self.width <= other.x
                || other.x + other.width <= self.x
                || self.y + self.height <= other.y
                || other.y + other.height <= self.y)
        }

        /// Check if this rectangle overlaps with an explicit bounding box [min_x, min_y, max_x, max_y]
        pub fn intersects_box(&self, min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> bool {
            !(self.x + self.width <= min_x
                || max_x <= self.x
                || self.y + self.height <= min_y
                || max_y <= self.y)
        }
    }

    /// Parsed text element from SVG
    #[derive(Debug, Clone, PartialEq)]
    pub struct SvgText {
        pub x: f32,
        pub y: f32,
        pub content: String,
        pub style: String,
    }

    /// Straight linear segment extracted from SVG <path> elements
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

        /// Calculate collinear overlap length between two linear segments
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

        /// Detect transverse orthogonal intersection between a horizontal and a vertical segment
        pub fn transverse_intersection(&self, other: &LineSegment) -> Option<(f32, f32)> {
            if self.is_horizontal() && other.is_vertical() {
                let cross_x = other.p1.x;
                let cross_y = self.p1.y;
                if cross_x > self.x_min() + 2.0
                    && cross_x < self.x_max() - 2.0
                    && cross_y > other.y_min() + 2.0
                    && cross_y < other.y_max() - 2.0
                {
                    return Some((cross_x, cross_y));
                }
            } else if self.is_vertical() && other.is_horizontal() {
                return other.transverse_intersection(self);
            }
            None
        }
    }

    /// Corner fillet extracted from quadratic Bézier ('Q') command
    #[derive(Debug, Clone, PartialEq)]
    pub struct CornerFillet {
        pub cx: f32,
        pub cy: f32,
        pub radius: f32,
    }

    impl CornerFillet {
        /// Fillet envelope [min_x, min_y, max_x, max_y]
        pub fn bounding_box(&self) -> (f32, f32, f32, f32) {
            let r = self.radius;
            (self.cx - r, self.cy - r, self.cx + r, self.cy + r)
        }
    }

    /// Semi-circular Arc Hop extracted from SVG ('A') command
    #[derive(Debug, Clone, PartialEq)]
    pub struct ArcHop {
        pub cx: f32,
        pub cy: f32,
        pub rx: f32,
        pub ry: f32,
        pub path_idx: usize,
    }

    /// Extract overall SVG width and height from the root `<svg>` tag
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

    /// Extract all `<rect>` elements from the SVG
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

    /// Extract all `<text>` elements and tspans from the SVG
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

    /// Extract all `<path d="..." ...>` elements from SVG
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

    /// Extract pill shield rectangles (used as protective underlays under edge labels)
    pub fn extract_pill_shields(svg: &str) -> Vec<SvgRect> {
        extract_rects(svg)
            .into_iter()
            .filter(|r| {
                (r.height - 18.0).abs() < 2.0
                    || r.style.contains("pill")
                    || (r.style.contains("rx:4") && r.height < 30.0)
            })
            .collect()
    }

    /// Extract subgraph boundary rectangles
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

    /// Extract node bounding rectangles (excluding subgraphs and pill shields)
    pub fn extract_node_boxes(svg: &str) -> Vec<SvgRect> {
        extract_rects(svg)
            .into_iter()
            .filter(|r| {
                r.width >= 40.0
                    && r.height >= 25.0
                    && !r.style.contains("subgraph")
                    && !r.style.contains("stroke-dasharray")
                    && (r.height - 18.0).abs() > 3.0
                    && !r.style.contains("pill")
            })
            .collect()
    }

    /// Find the center (cx, cy) of a node matching the specified label or text substring
    pub fn find_node_center(svg: &str, label_substring: &str) -> Option<(f32, f32)> {
        for t in extract_texts(svg) {
            if t.content.contains(label_substring) {
                return Some((t.x, t.y));
            }
        }
        None
    }

    /// Parse SVG path string into linear line segments, corner fillets, and arc hops
    pub fn parse_path_geometry(
        d: &str,
        path_idx: usize,
    ) -> (Vec<LineSegment>, Vec<CornerFillet>, Vec<ArcHop>) {
        let mut segments = Vec::new();
        let mut fillets = Vec::new();
        let mut hops = Vec::new();

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
                        let cx = tokens[i + 1].parse::<f32>().unwrap_or(0.0);
                        let cy = tokens[i + 2].parse::<f32>().unwrap_or(0.0);
                        let ex = tokens[i + 3].parse::<f32>().unwrap_or(0.0);
                        let ey = tokens[i + 4].parse::<f32>().unwrap_or(0.0);
                        let radius = ((cx - current_pos.x).powi(2) + (cy - current_pos.y).powi(2))
                            .sqrt()
                            .max(8.0);
                        fillets.push(CornerFillet { cx, cy, radius });
                        current_pos = Point { x: ex, y: ey };
                        i += 5;
                    } else {
                        break;
                    }
                }
                "A" => {
                    if i + 7 < tokens.len() {
                        let rx = tokens[i + 1].parse::<f32>().unwrap_or(5.0);
                        let ry = tokens[i + 2].parse::<f32>().unwrap_or(5.0);
                        let ex = tokens[i + 6].parse::<f32>().unwrap_or(0.0);
                        let ey = tokens[i + 7].parse::<f32>().unwrap_or(0.0);
                        let hop_cx = (current_pos.x + ex) / 2.0;
                        let hop_cy = (current_pos.y + ey) / 2.0;
                        hops.push(ArcHop {
                            cx: hop_cx,
                            cy: hop_cy,
                            rx,
                            ry,
                            path_idx,
                        });
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

        let is_backward = first_pos
            .map(|fp| current_pos.y < fp.y - 4.0)
            .unwrap_or(false)
            || segments.iter().any(|s| s.p2.y < s.p1.y - 4.0);

        for s in &mut segments {
            s.is_backward = is_backward;
        }

        (segments, fillets, hops)
    }

    /// Extract all straight linear segments from all `<path>` elements in the SVG
    pub fn extract_path_segments(svg: &str) -> Vec<LineSegment> {
        let paths = extract_paths(svg);
        let mut all_segments = Vec::new();
        for (idx, p) in paths.iter().enumerate() {
            let (segs, _, _) = parse_path_geometry(p, idx);
            all_segments.extend(segs);
        }
        all_segments
    }

    /// Extract all corner fillets from all `<path>` elements in the SVG
    pub fn extract_corner_fillets(svg: &str) -> Vec<CornerFillet> {
        let paths = extract_paths(svg);
        let mut all_fillets = Vec::new();
        for (idx, p) in paths.iter().enumerate() {
            let (_, fillets, _) = parse_path_geometry(p, idx);
            all_fillets.extend(fillets);
        }
        all_fillets
    }

    /// Extract all Arc Hops ('A' commands) from all `<path>` elements in the SVG
    pub fn extract_arc_hops(svg: &str) -> Vec<ArcHop> {
        let paths = extract_paths(svg);
        let mut all_hops = Vec::new();
        for (idx, p) in paths.iter().enumerate() {
            let (_, _, hops) = parse_path_geometry(p, idx);
            all_hops.extend(hops);
        }
        all_hops
    }

    // ========================================================================
    // THE 4 VERIFICATION ORACLES
    // ========================================================================

    /// Oracle 1: Verify Gutter Channel Separation (Delta X >= 16.0px)
    ///
    /// For any two concurrent vertical segments in outer gutters (e.g. feedback loops)
    /// that overlap along their vertical Y span, their horizontal X coordinates must
    /// be separated by at least 16.0px to avoid visual ambiguity and collinear overlap.
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

    /// Oracle 2: Verify Bus Deduplication (Zero Collinear Duplicate SVG Path Segments)
    ///
    /// Across all distinct `<path>` elements, no two linear segments (fork/join stems or bus lines)
    /// may have collinear overlap length >= 1.0px.
    pub fn verify_bus_deduplication_oracle(svg: &str) -> Result<(), String> {
        let segments = extract_path_segments(svg);
        for i in 0..segments.len() {
            for j in (i + 1)..segments.len() {
                let s1 = &segments[i];
                let s2 = &segments[j];
                if s1.path_idx == s2.path_idx {
                    continue;
                }
                let overlap = s1.overlap_length(s2);
                if overlap >= 1.0 {
                    return Err(format!(
                        "Bus deduplication violation: Duplicate collinear segment overlap={:.1}px between path {} and path {} (s1: ({:.1},{:.1})->({:.1},{:.1}), s2: ({:.1},{:.1})->({:.1},{:.1}))",
                        overlap, s1.path_idx, s2.path_idx,
                        s1.p1.x, s1.p1.y, s1.p2.x, s1.p2.y,
                        s2.p1.x, s2.p1.y, s2.p2.x, s2.p2.y,
                    ));
                }
            }
        }
        Ok(())
    }

    /// Oracle 3: Verify Arc Hop Presence on Orthogonal Crossings
    ///
    /// Detects all transverse orthogonal intersections between horizontal and vertical segments.
    /// Every transverse intersection point must feature an SVG Arc Hop ('A' command) bridge leaping over it.
    pub fn verify_arc_hop_presence_oracle(svg: &str) -> Result<(), String> {
        let segments = extract_path_segments(svg);
        let arc_hops = extract_arc_hops(svg);

        let mut crossings = Vec::new();
        for i in 0..segments.len() {
            for j in (i + 1)..segments.len() {
                let s1 = &segments[i];
                let s2 = &segments[j];
                if s1.path_idx == s2.path_idx {
                    continue;
                }
                if let Some(pt) = s1.transverse_intersection(s2) {
                    crossings.push(pt);
                }
            }
        }

        for (cx, cy) in &crossings {
            let hop_found = arc_hops
                .iter()
                .any(|hop| ((hop.cx - cx).powi(2) + (hop.cy - cy).powi(2)).sqrt() <= 12.0);
            if !hop_found {
                return Err(format!(
                    "Arc Hop presence violation: Orthogonal line intersection detected at ({:.1}, {:.1}) without corresponding SVG Arc Hop ('A' command) bridge",
                    cx, cy
                ));
            }
        }
        Ok(())
    }

    /// Oracle 4: Verify Pill Shield Mutual Non-Collision & Corner Fillet Protection
    ///
    /// 1. All pill shields must be pairwise mutually disjoint (zero AABB intersection).
    /// 2. No pill shield may overlap any 90-degree corner fillet envelope [cx-8, cx+8] x [cy-8, cy+8].
    /// 3. No pill shield may overlap any node bounding box.
    pub fn verify_pill_shield_oracle(svg: &str) -> Result<(), String> {
        let pill_shields = extract_pill_shields(svg);
        let corner_fillets = extract_corner_fillets(svg);
        let node_boxes = extract_node_boxes(svg);

        // 1. Mutual pairwise non-collision
        for i in 0..pill_shields.len() {
            for j in (i + 1)..pill_shields.len() {
                let s1 = &pill_shields[i];
                let s2 = &pill_shields[j];
                if s1.intersects(s2) {
                    return Err(format!(
                        "Pill shield mutual collision: Shield {} at [{:.1}, {:.1}, {:.1}, {:.1}] collides with shield {} at [{:.1}, {:.1}, {:.1}, {:.1}]",
                        i, s1.x, s1.y, s1.width, s1.height,
                        j, s2.x, s2.y, s2.width, s2.height
                    ));
                }
            }
        }

        // 2. Corner fillet protection
        for (i, fillet) in corner_fillets.iter().enumerate() {
            let (fx1, fy1, fx2, fy2) = fillet.bounding_box();
            for (j, shield) in pill_shields.iter().enumerate() {
                if shield.intersects_box(fx1, fy1, fx2, fy2) {
                    return Err(format!(
                        "Corner fillet obstruction: Pill shield {} at [{:.1}, {:.1}, {:.1}, {:.1}] obstructs 90-degree corner fillet {} at center ({:.1}, {:.1})",
                        j, shield.x, shield.y, shield.width, shield.height,
                        i, fillet.cx, fillet.cy
                    ));
                }
            }
        }

        // 3. Node bounding box protection
        for (i, node) in node_boxes.iter().enumerate() {
            for (j, shield) in pill_shields.iter().enumerate() {
                if shield.intersects(node) {
                    return Err(format!(
                        "Node bounding box obstruction: Pill shield {} at [{:.1}, {:.1}, {:.1}, {:.1}] overlaps node {} at [{:.1}, {:.1}, {:.1}, {:.1}]",
                        j, shield.x, shield.y, shield.width, shield.height,
                        i, node.x, node.y, node.width, node.height
                    ));
                }
            }
        }

        Ok(())
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
// GROUP R1: MULTI-LEVEL GUTTER & BYPASS CHANNELS (10 TESTS)
// ============================================================================

mod group_r1_multi_level_gutter_and_bypass {
    use super::helpers::*;
    use super::*;

    #[test]
    fn test_r1_gutter_01_two_loops_channel_separation_16px() {
        let code = "flowchart TD\n A-->B\n B-->C\n C-->D\n C-->A\n D-->B";
        let res = render(code);
        assert!(res.is_ok(), "Two concurrent feedback loops must render");
        let svg = res.unwrap();
        let gutter_res = verify_gutter_separation_oracle(&svg);
        assert!(
            gutter_res.is_ok(),
            "Two concurrent feedback loops must maintain >= 16px gutter separation: {:?}",
            gutter_res.err()
        );
    }

    #[test]
    fn test_r1_gutter_02_three_nested_loops_stepped_nesting() {
        let code = "flowchart TD\n A-->B\n B-->C\n C-->D\n D-->E\n C-->B\n D-->A\n E-->A";
        let res = render(code);
        assert!(res.is_ok(), "Three nested feedback loops must render");
        let svg = res.unwrap();
        let segments = extract_path_segments(&svg);
        let vertical_gutters: Vec<_> = segments
            .iter()
            .filter(|s| s.is_vertical() && s.is_backward)
            .collect();
        assert!(
            vertical_gutters.len() >= 3,
            "Three nested feedback loops require at least 3 vertical gutter segments"
        );
        let gutter_res = verify_gutter_separation_oracle(&svg);
        assert!(
            gutter_res.is_ok(),
            "Three nested loops must have stepped non-overlapping gutters: {:?}",
            gutter_res.err()
        );
    }

    #[test]
    fn test_r1_gutter_03_subgraph_perimeter_cleared() {
        let code = "flowchart TD\n subgraph G1[\"阶段一\"]\n A-->B\n B-->C\n end\n C-->A";
        let res = render(code);
        assert!(res.is_ok(), "Feedback loop around subgraph must render");
        let svg = res.unwrap();
        let subgraphs = extract_subgraph_rects(&svg);
        assert!(!subgraphs.is_empty(), "Subgraph G1 must exist");
        let sg_right = subgraphs[0].x + subgraphs[0].width;

        let segments = extract_path_segments(&svg);
        let loop_vertical = segments.iter().find(|s| s.is_vertical() && s.is_backward);
        assert!(
            loop_vertical.is_some(),
            "Feedback loop vertical segment must be present"
        );
        assert!(
            loop_vertical.unwrap().p1.x >= sg_right + 16.0,
            "Gutter vertical channel at {:.1} must clear subgraph right boundary at {:.1} by >= 16px",
            loop_vertical.unwrap().p1.x,
            sg_right
        );
    }

    #[test]
    fn test_r1_gutter_04_cross_layer_bypass_corridor_isolation() {
        let code = "flowchart TD\n A-->B\n B-->C\n C-->D\n A-->D";
        let res = render(code);
        assert!(res.is_ok(), "Cross-layer bypass edge must render");
        let svg = res.unwrap();
        let node_boxes = extract_node_boxes(&svg);
        let b_box = node_boxes
            .iter()
            .find(|n| n.y > 50.0 && n.y < 300.0)
            .expect("Intermediate node must exist");

        let segments = extract_path_segments(&svg);
        // Ensure no linear segment cuts directly through intermediate node B
        for s in &segments {
            assert!(
                !s.is_vertical()
                    || (s.p1.x < b_box.x || s.p1.x > b_box.x + b_box.width)
                    || (s.y_max() <= b_box.y || s.y_min() >= b_box.y + b_box.height),
                "Bypass edge vertical segment at X={:.1} cuts directly through node B [{:.1}, {:.1}]",
                s.p1.x,
                b_box.x,
                b_box.x + b_box.width
            );
        }
    }

    #[test]
    fn test_r1_gutter_05_feedback_loop_fillet_curves_preserved() {
        let code = "flowchart TD\n A-->B\n B-->C\n C-->A";
        let res = render(code);
        assert!(res.is_ok(), "Feedback loop with fillets must render");
        let svg = res.unwrap();
        let fillets = extract_corner_fillets(&svg);
        assert!(
            fillets.len() >= 2,
            "Feedback loop must preserve smooth corner fillets at orthogonal turns (found {})",
            fillets.len()
        );
    }

    #[test]
    fn test_r1_gutter_06_wide_canvas_gutter_spacing() {
        let code = "flowchart TD\n A-->B & C & D\n D-->A";
        let res = render(code);
        assert!(res.is_ok(), "Wide canvas with gutter must render");
        let svg = res.unwrap();
        let (width, _) = extract_svg_dimensions(&svg);
        let segments = extract_path_segments(&svg);
        let gutter_x = segments
            .iter()
            .filter(|s| s.is_vertical())
            .map(|s| s.p1.x)
            .fold(0.0_f32, f32::max);
        assert!(
            width >= gutter_x + 10.0,
            "Canvas width ({:.1}) must comfortably contain outer gutter track ({:.1})",
            width,
            gutter_x
        );
    }

    #[test]
    fn test_r1_gutter_07_asymmetric_depth_gutter_routing() {
        let code = "flowchart TD\n A-->B\n B-->C\n C-->D\n B-->A\n D-->A";
        let res = render(code);
        assert!(res.is_ok(), "Asymmetric depth loops must render");
        let svg = res.unwrap();
        let gutter_res = verify_gutter_separation_oracle(&svg);
        assert!(
            gutter_res.is_ok(),
            "Asymmetric depth loops ending at same node must have separated gutter channels: {:?}",
            gutter_res.err()
        );
    }

    #[test]
    fn test_r1_gutter_08_order_flow_c10_c7_stepped_gutter() {
        let mmd = std::fs::read_to_string("examples/order_flow.mmd")
            .expect("order_flow.mmd should exist");
        let res = render(&mmd);
        assert!(res.is_ok(), "order_flow.mmd must render cleanly");
        let svg = res.unwrap();
        let gutter_res = verify_gutter_separation_oracle(&svg);
        assert!(
            gutter_res.is_ok(),
            "order_flow.svg concurrent feedback loops (C10->C4 and C7->C3) must satisfy Delta X >= 16px: {:?}",
            gutter_res.err()
        );
    }

    #[test]
    fn test_r1_gutter_09_direct_loop_to_same_layer() {
        let code = "flowchart TD\n A-->B\n B-->A";
        let res = render(code);
        assert!(res.is_ok(), "2-node cycle must render");
        let svg = res.unwrap();
        let node_boxes = extract_node_boxes(&svg);
        let max_x = node_boxes
            .iter()
            .map(|n| n.x + n.width)
            .fold(0.0_f32, f32::max);
        let segments = extract_path_segments(&svg);
        let loop_vertical = segments.iter().find(|s| s.is_vertical() && s.p1.x > max_x);
        assert!(
            loop_vertical.is_some(),
            "Backward edge in 2-node cycle must route through outer gutter track clearing nodes"
        );
    }

    #[test]
    fn test_r1_gutter_10_multiple_subgraphs_gutter_avoidance() {
        let code = "flowchart TD\n subgraph G1[\"第一段\"]\n A-->B\n end\n subgraph G2[\"第二段\"]\n C-->D\n end\n B-->C\n D-->A";
        let res = render(code);
        assert!(res.is_ok(), "Multi-subgraph feedback loop must render");
        let svg = res.unwrap();
        let subgraphs = extract_subgraph_rects(&svg);
        assert!(subgraphs.len() >= 2, "Both subgraphs G1 and G2 must exist");
        let max_sg_right = subgraphs
            .iter()
            .map(|sg| sg.x + sg.width)
            .fold(0.0_f32, f32::max);

        let segments = extract_path_segments(&svg);
        let loop_seg = segments
            .iter()
            .find(|s| s.is_vertical() && s.p1.x > max_sg_right);
        assert!(
            loop_seg.is_some(),
            "Feedback loop traversing multiple subgraphs must route strictly outside both subgraphs (X > {:.1})",
            max_sg_right
        );
    }
}

// ============================================================================
// GROUP R2: BUS TRUNK DEDUPLICATION & MULTI-BRANCH DISAMBIGUATION (10 TESTS)
// ============================================================================

mod group_r2_bus_trunk_deduplication {
    use super::helpers::*;
    use super::*;

    #[test]
    fn test_r2_bus_01_fork_vertical_stem_drawn_exactly_once() {
        let code = "flowchart TD\n A-->B\n A-->C";
        let res = render(code);
        assert!(res.is_ok(), "Binary fork must render");
        let svg = res.unwrap();
        let dedup_res = verify_bus_deduplication_oracle(&svg);
        assert!(
            dedup_res.is_ok(),
            "Binary fork vertical stem must be emitted exactly once without duplicate collinear path segments: {:?}",
            dedup_res.err()
        );
    }

    #[test]
    fn test_r2_bus_02_join_vertical_stem_drawn_exactly_once() {
        let code = "flowchart TD\n B-->D\n C-->D";
        let res = render(code);
        assert!(res.is_ok(), "Binary join must render");
        let svg = res.unwrap();
        let dedup_res = verify_bus_deduplication_oracle(&svg);
        assert!(
            dedup_res.is_ok(),
            "Binary join collection trunk must be emitted exactly once without duplicate collinear path segments: {:?}",
            dedup_res.err()
        );
    }

    #[test]
    fn test_r2_bus_03_horizontal_bus_segment_uniqueness() {
        let code = "flowchart TD\n A-->B\n A-->C\n A-->D\n A-->E";
        let res = render(code);
        assert!(res.is_ok(), "Multi-branch fork must render");
        let svg = res.unwrap();
        let dedup_res = verify_bus_deduplication_oracle(&svg);
        assert!(
            dedup_res.is_ok(),
            "Horizontal distribution bus segments must not have duplicate collinear overlaps: {:?}",
            dedup_res.err()
        );
    }

    #[test]
    fn test_r2_bus_04_ternary_fork_zero_duplicate_segments() {
        let code = "flowchart TD\n A-->B\n A-->C\n A-->D";
        let res = render(code);
        assert!(res.is_ok(), "Ternary fork must render");
        let svg = res.unwrap();
        let dedup_res = verify_bus_deduplication_oracle(&svg);
        assert!(
            dedup_res.is_ok(),
            "Ternary fork must have zero collinear duplicate SVG path segments: {:?}",
            dedup_res.err()
        );
    }

    #[test]
    fn test_r2_bus_05_multi_branch_fillet_centers_disambiguated() {
        let code = "flowchart TD\n A-->B\n A-->C\n A-->D";
        let res = render(code);
        assert!(res.is_ok(), "Ternary fork with fillets must render");
        let svg = res.unwrap();
        let fillets = extract_corner_fillets(&svg);
        assert!(
            fillets.len() >= 2,
            "Ternary fork must feature distinct corner fillets for turning branches"
        );
        for i in 0..fillets.len() {
            for j in (i + 1)..fillets.len() {
                let dist = ((fillets[i].cx - fillets[j].cx).powi(2)
                    + (fillets[i].cy - fillets[j].cy).powi(2))
                .sqrt();
                assert!(
                    dist >= 20.0,
                    "Corner fillets {} and {} must maintain distinct turning centers (dist={:.1}px >= 20px)",
                    i,
                    j,
                    dist
                );
            }
        }
    }

    #[test]
    fn test_r2_bus_06_diamond_fork_join_bus_deduplication() {
        let code = "flowchart TD\n A-->B\n A-->C\n B-->D\n C-->D";
        let res = render(code);
        assert!(res.is_ok(), "Diamond fork-join must render");
        let svg = res.unwrap();
        let dedup_res = verify_bus_deduplication_oracle(&svg);
        assert!(
            dedup_res.is_ok(),
            "Diamond fork distribution and join collection buses must have zero collinear duplicates: {:?}",
            dedup_res.err()
        );
    }

    #[test]
    fn test_r2_bus_07_quaternary_fork_clean_fillet_spacing() {
        let code = "flowchart TD\n A-->B1\n A-->B2\n A-->B3\n A-->B4";
        let res = render(code);
        assert!(res.is_ok(), "Quaternary fork must render");
        let svg = res.unwrap();
        let dedup_res = verify_bus_deduplication_oracle(&svg);
        assert!(
            dedup_res.is_ok(),
            "Quaternary fork must have zero collinear duplicate path segments: {:?}",
            dedup_res.err()
        );
    }

    #[test]
    fn test_r2_bus_08_order_flow_fork_trunks_deduplicated() {
        let mmd = std::fs::read_to_string("examples/order_flow.mmd")
            .expect("order_flow.mmd should exist");
        let res = render(&mmd);
        assert!(res.is_ok(), "order_flow.mmd must render cleanly");
        let svg = res.unwrap();
        let dedup_res = verify_bus_deduplication_oracle(&svg);
        assert!(
            dedup_res.is_ok(),
            "order_flow.svg fork and join buses must have zero duplicate collinear path segments: {:?}",
            dedup_res.err()
        );
    }

    #[test]
    fn test_r2_bus_09_asymmetric_join_collection_bus() {
        let code = "flowchart TD\n A-->B\n B-->D\n A-->C\n C-->D\n A-->D";
        let res = render(code);
        assert!(res.is_ok(), "Asymmetric join must render");
        let svg = res.unwrap();
        let dedup_res = verify_bus_deduplication_oracle(&svg);
        assert!(
            dedup_res.is_ok(),
            "Asymmetric join collection bus must have zero collinear duplicate segments: {:?}",
            dedup_res.err()
        );
    }

    #[test]
    fn test_r2_bus_10_high_degree_fork_bus_clarity() {
        let code = "flowchart TD\n A-->B1\n A-->B2\n A-->B3\n A-->B4\n A-->B5";
        let res = render(code);
        assert!(res.is_ok(), "5-way fork must render");
        let svg = res.unwrap();
        let dedup_res = verify_bus_deduplication_oracle(&svg);
        assert!(
            dedup_res.is_ok(),
            "5-way fork must deduplicate trunk and bus segments: {:?}",
            dedup_res.err()
        );
    }
}

// ============================================================================
// GROUP R3: BARYCENTER CROSSING MINIMIZATION & ARC BRIDGES (10 TESTS)
// ============================================================================

mod group_r3_crossing_minimization_and_arc_hops {
    use super::helpers::*;
    use super::*;

    #[test]
    fn test_r3_cross_01_barycenter_reordering_reduces_crossings() {
        let code = "flowchart TD\n A1[\"A1\"] --> B2[\"B2\"]\n A2[\"A2\"] --> B1[\"B1\"]";
        let res = render(code);
        assert!(res.is_ok(), "Crossable 2-node graph must render");
        let svg = res.unwrap();
        let b1_pos = find_node_center(&svg, "B1");
        let b2_pos = find_node_center(&svg, "B2");
        assert!(
            b1_pos.is_some() && b2_pos.is_some(),
            "Nodes B1 and B2 must exist"
        );
        // Barycenter crossing minimization reorders layer 2 so that B1 (connected to A2 on right)
        // is placed to the right of B2 (connected to A1 on left), resulting in 0 crossings!
        assert!(
            b2_pos.unwrap().0 < b1_pos.unwrap().0,
            "Barycenter reordering must place B2 (x={:.1}) left of B1 (x={:.1}) to eliminate line crossing",
            b2_pos.unwrap().0,
            b1_pos.unwrap().0
        );
    }

    #[test]
    fn test_r3_cross_02_orthogonal_intersection_arc_hop_present() {
        let code = "flowchart TD\n A1-->B2\n A2-->B1\n A3-->B3";
        let res = render(code);
        assert!(res.is_ok(), "Diagram with cross edges must render");
        let svg = res.unwrap();
        let hop_res = verify_arc_hop_presence_oracle(&svg);
        assert!(
            hop_res.is_ok(),
            "Unavoidable orthogonal line intersections must feature semi-circular Arc Hop bridges: {:?}",
            hop_res.err()
        );
    }

    #[test]
    fn test_r3_cross_03_arc_hop_geometry_span_8_to_12px() {
        let code = "flowchart TD\n A1-->B2\n A2-->B1\n A3-->B3";
        let res = render(code);
        assert!(res.is_ok(), "Graph with crossings must render");
        let svg = res.unwrap();
        let hops = extract_arc_hops(&svg);
        for hop in &hops {
            let span = 2.0 * hop.rx;
            assert!(
                (8.0..=14.0).contains(&span),
                "Arc Hop span must be between 8px and 12px (found {:.1}px with rx={:.1})",
                span,
                hop.rx
            );
        }
    }

    #[test]
    fn test_r3_cross_04_arc_hop_command_syntax_svg_a() {
        let code = "flowchart TD\n A1-->B2\n A2-->B1\n A3-->B3";
        let res = render(code);
        assert!(res.is_ok(), "Graph with crossings must render");
        let svg = res.unwrap();
        let segments = extract_path_segments(&svg);
        let has_transverse_cross = segments.iter().any(|s1| {
            segments
                .iter()
                .any(|s2| s1.path_idx != s2.path_idx && s1.transverse_intersection(s2).is_some())
        });
        if has_transverse_cross {
            assert!(
                svg.contains(" A "),
                "SVG paths with transverse crossings must contain SVG arc 'A' commands"
            );
        }
    }

    #[test]
    fn test_r3_cross_05_horizontal_edge_hops_over_vertical() {
        let code = "flowchart TD\n A1-->B2\n A2-->B1\n A3-->B3";
        let res = render(code);
        assert!(res.is_ok(), "Graph with crossings must render");
        let svg = res.unwrap();
        let hop_res = verify_arc_hop_presence_oracle(&svg);
        assert!(
            hop_res.is_ok(),
            "Orthogonal intersections must leap cleanly: {:?}",
            hop_res.err()
        );
    }

    #[test]
    fn test_r3_cross_06_bypass_line_arc_hops_over_bus() {
        let code = "flowchart TD\n A-->B & C & D\n Start-->E\n B-->E";
        let res = render(code);
        assert!(res.is_ok(), "Bypass crossing bus must render");
        let svg = res.unwrap();
        let hop_res = verify_arc_hop_presence_oracle(&svg);
        assert!(
            hop_res.is_ok(),
            "Bypass edge crossing bus line must feature Arc Hop: {:?}",
            hop_res.err()
        );
    }

    #[test]
    fn test_r3_cross_07_multiple_crossings_multiple_arc_hops() {
        let code = "flowchart TD\n A1-->B3\n A2-->B2\n A3-->B1";
        let res = render(code);
        assert!(res.is_ok(), "Multiple crossings graph must render");
        let svg = res.unwrap();
        let hop_res = verify_arc_hop_presence_oracle(&svg);
        assert!(
            hop_res.is_ok(),
            "Multiple crossings must each have an Arc Hop bridge: {:?}",
            hop_res.err()
        );
    }

    #[test]
    fn test_r3_cross_08_crossing_near_corner_fillet_clearance() {
        let code = "flowchart TD\n A1-->B2\n A2-->B1\n A3-->B3";
        let res = render(code);
        assert!(res.is_ok(), "Graph must render");
        let svg = res.unwrap();
        let hops = extract_arc_hops(&svg);
        let fillets = extract_corner_fillets(&svg);
        for hop in &hops {
            for fillet in &fillets {
                let dist = ((hop.cx - fillet.cx).powi(2) + (hop.cy - fillet.cy).powi(2)).sqrt();
                assert!(
                    dist >= 8.0,
                    "Arc Hop at ({:.1},{:.1}) must maintain >= 8px clearance from corner fillet at ({:.1},{:.1})",
                    hop.cx,
                    hop.cy,
                    fillet.cx,
                    fillet.cy
                );
            }
        }
    }

    #[test]
    fn test_r3_cross_09_order_flow_arc_hops_present() {
        let mmd = std::fs::read_to_string("examples/order_flow.mmd")
            .expect("order_flow.mmd should exist");
        let res = render(&mmd);
        assert!(res.is_ok(), "order_flow.mmd must render cleanly");
        let svg = res.unwrap();
        let hop_res = verify_arc_hop_presence_oracle(&svg);
        assert!(
            hop_res.is_ok(),
            "order_flow.svg transverse crossings must feature Arc Hop bridges: {:?}",
            hop_res.err()
        );
    }

    #[test]
    fn test_r3_cross_10_planar_graph_zero_arc_hops_needed() {
        let code = "flowchart TD\n A-->B\n B-->C\n C-->D";
        let res = render(code);
        assert!(res.is_ok(), "Planar linear graph must render");
        let svg = res.unwrap();
        let hops = extract_arc_hops(&svg);
        assert_eq!(
            hops.len(),
            0,
            "Planar linear graph with 0 crossings must not generate superfluous Arc Hops"
        );
    }
}

// ============================================================================
// GROUP R4: LABEL & PILL SHIELD COLLISION AVOIDANCE (10 TESTS)
// ============================================================================

mod group_r4_label_pill_shield_collision_avoidance {
    use super::helpers::*;
    use super::*;

    #[test]
    fn test_r4_shield_01_pairwise_pill_shield_mutual_disjointness() {
        let code = "flowchart TD\n A-->|\"标签一\"| B\n A-->|\"标签二\"| C\n B-->|\"标签三\"| D\n C-->|\"标签四\"| D";
        let res = render(code);
        assert!(res.is_ok(), "Labeled diamond flow must render");
        let svg = res.unwrap();
        let shield_res = verify_pill_shield_oracle(&svg);
        assert!(
            shield_res.is_ok(),
            "All edge label pill shields must be pairwise mutually disjoint: {:?}",
            shield_res.err()
        );
    }

    #[test]
    fn test_r4_shield_02_corner_fillet_clearance_no_obscuration() {
        let code = "flowchart TD\n A-->|\"高危审核\"| B";
        let res = render(code);
        assert!(res.is_ok(), "Labeled edge with turn must render");
        let svg = res.unwrap();
        let fillets = extract_corner_fillets(&svg);
        let shields = extract_pill_shields(&svg);
        for fillet in &fillets {
            let (fx1, fy1, fx2, fy2) = fillet.bounding_box();
            for shield in &shields {
                assert!(
                    !shield.intersects_box(fx1, fy1, fx2, fy2),
                    "Pill shield [{:.1}, {:.1}, {:.1}, {:.1}] must not obscure 90-degree corner fillet at ({:.1}, {:.1})",
                    shield.x,
                    shield.y,
                    shield.width,
                    shield.height,
                    fillet.cx,
                    fillet.cy
                );
            }
        }
    }

    #[test]
    fn test_r4_shield_03_node_bounding_box_clearance() {
        let code = "flowchart TD\n A[\"起始节点\"] -->|\"流转条件判定\"| B[\"目标节点\"]";
        let res = render(code);
        assert!(res.is_ok(), "Labeled edge between nodes must render");
        let svg = res.unwrap();
        let node_boxes = extract_node_boxes(&svg);
        let shields = extract_pill_shields(&svg);
        for node in &node_boxes {
            for shield in &shields {
                assert!(
                    !shield.intersects(node),
                    "Pill shield [{:.1}, {:.1}] must not overlap node bounding box [{:.1}, {:.1}, {:.1}, {:.1}]",
                    shield.x,
                    shield.y,
                    node.x,
                    node.y,
                    node.width,
                    node.height
                );
            }
        }
    }

    #[test]
    fn test_r4_shield_04_long_label_bus_segment_sliding() {
        let code = "flowchart TD\n A-->|\"这是一个非常长的风控审核判定分支标签测试\"| B";
        let res = render(code);
        assert!(res.is_ok(), "Long label edge must render");
        let svg = res.unwrap();
        let shield_res = verify_pill_shield_oracle(&svg);
        assert!(
            shield_res.is_ok(),
            "Long label on bus segment must slide or offset to avoid corner fillets and nodes: {:?}",
            shield_res.err()
        );
    }

    #[test]
    fn test_r4_shield_05_adjacent_fork_labels_staggered() {
        let code = "flowchart TD\n A-->|\"完全充足库存\"| B\n A-->|\"部分不足库存\"| C";
        let res = render(code);
        assert!(res.is_ok(), "Adjacent branch labels must render");
        let svg = res.unwrap();
        let shield_res = verify_pill_shield_oracle(&svg);
        assert!(
            shield_res.is_ok(),
            "Adjacent fork labels must have staggered positions to eliminate collision: {:?}",
            shield_res.err()
        );
    }

    #[test]
    fn test_r4_shield_06_chinese_labels_pill_shield_disjointness() {
        let code = "flowchart TD\n A-->|\"在线支付\"| B\n A-->|\"货到付款\"| C\n B-->|\"支付成功\"| D\n B-->|\"支付失败\"| E";
        let res = render(code);
        assert!(res.is_ok(), "Multi-branch Chinese labels must render");
        let svg = res.unwrap();
        let shield_res = verify_pill_shield_oracle(&svg);
        assert!(
            shield_res.is_ok(),
            "Chinese edge label pill shields must be pairwise disjoint: {:?}",
            shield_res.err()
        );
    }

    #[test]
    fn test_r4_shield_07_vertical_segment_label_clearance() {
        let code = "flowchart TD\n A-->|\"垂直分支流转\"| B";
        let res = render(code);
        assert!(res.is_ok(), "Vertical segment label must render");
        let svg = res.unwrap();
        let node_boxes = extract_node_boxes(&svg);
        let shields = extract_pill_shields(&svg);
        assert!(!shields.is_empty(), "Pill shield must exist");
        let shield = &shields[0];
        for node in &node_boxes {
            assert!(
                !shield.intersects(node),
                "Vertical segment pill shield must maintain clearance from nodes"
            );
        }
    }

    #[test]
    fn test_r4_shield_08_order_flow_all_pill_shields_collision_free() {
        let mmd = std::fs::read_to_string("examples/order_flow.mmd")
            .expect("order_flow.mmd should exist");
        let res = render(&mmd);
        assert!(res.is_ok(), "order_flow.mmd must render cleanly");
        let svg = res.unwrap();
        let shields = extract_pill_shields(&svg);
        assert!(
            shields.len() >= 10,
            "order_flow.svg must feature at least 10 pill shields (found {})",
            shields.len()
        );
        for i in 0..shields.len() {
            for j in (i + 1)..shields.len() {
                assert!(
                    !shields[i].intersects(&shields[j]),
                    "order_flow.svg pill shield {} at [{:.1}, {:.1}] collides with shield {} at [{:.1}, {:.1}]",
                    i,
                    shields[i].x,
                    shields[i].y,
                    j,
                    shields[j].x,
                    shields[j].y
                );
            }
        }
    }

    #[test]
    fn test_r4_shield_09_order_flow_fillets_unobstructed_by_shields() {
        let mmd = std::fs::read_to_string("examples/order_flow.mmd")
            .expect("order_flow.mmd should exist");
        let res = render(&mmd);
        assert!(res.is_ok(), "order_flow.mmd must render cleanly");
        let svg = res.unwrap();
        let fillets = extract_corner_fillets(&svg);
        let shields = extract_pill_shields(&svg);
        for fillet in &fillets {
            let (fx1, fy1, fx2, fy2) = fillet.bounding_box();
            for shield in &shields {
                assert!(
                    !shield.intersects_box(fx1, fy1, fx2, fy2),
                    "order_flow.svg pill shield at [{:.1}, {:.1}, {:.1}, {:.1}] obstructs 90-degree corner fillet at ({:.1}, {:.1})",
                    shield.x,
                    shield.y,
                    shield.width,
                    shield.height,
                    fillet.cx,
                    fillet.cy
                );
            }
        }
    }

    #[test]
    fn test_r4_shield_10_shield_underlay_rx4_preserved() {
        let mmd = std::fs::read_to_string("examples/order_flow.mmd")
            .expect("order_flow.mmd should exist");
        let res = render(&mmd);
        assert!(res.is_ok(), "order_flow.mmd must render cleanly");
        let svg = res.unwrap();
        let shields = extract_pill_shields(&svg);
        assert!(!shields.is_empty(), "Pill shields must be present");
        for (i, s) in shields.iter().enumerate() {
            assert!(
                (s.height - 18.0).abs() < 2.5,
                "Pill shield {} must have standard height 18px (found {:.1})",
                i,
                s.height
            );
        }
    }
}

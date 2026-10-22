/**
 * Rounded Orthogonal Edge Router
 * Inspired by Ondine (ondine.sh)
 * Generates clean rectilinear Manhattan paths with smooth quadratic Bézier corner fillets.
 */

#[derive(Debug, Clone)]
pub struct RoutedEdge {
    /// SVG path string: "M ... L ... Q ... L ..."
    pub path_data: String,
    /// Arrowhead placement: (tip_x, tip_y, dir_dx, dir_dy)
    pub arrow: (f32, f32, f32, f32),
    /// Optimal edge label position: (x, y)
    pub label_pos: (f32, f32),
}

/// Compute waypoints and rounded orthogonal path between source and target node bounding boxes
pub fn route_orthogonal_edge(
    from_box: (f32, f32, f32, f32), // (x, y, width, height)
    to_box: (f32, f32, f32, f32),
    fillet_radius: f32,
) -> RoutedEdge {
    let (x1, y1, w1, h1) = from_box;
    let (x2, y2, w2, h2) = to_box;

    let cx1 = x1 + w1 / 2.0;
    let bottom1 = y1 + h1;
    let cx2 = x2 + w2 / 2.0;
    let top2 = y2;

    let waypoints: Vec<(f32, f32)> = if top2 >= bottom1 - 4.0 {
        // Forward edge (target is below source)
        if (cx1 - cx2).abs() < 2.0 {
            // Straight vertical line in same column
            vec![(cx1, bottom1), (cx2, top2)]
        } else {
            // Two-step orthogonal bend
            let y_mid = bottom1 + (top2 - bottom1) / 2.0;
            vec![(cx1, bottom1), (cx1, y_mid), (cx2, y_mid), (cx2, top2)]
        }
    } else {
        // Backward / Feedback / Cross-layer loop edge
        let right1 = x1 + w1;
        let cy1 = y1 + h1 / 2.0;
        let right2 = x2 + w2;
        let cy2 = y2 + h2 / 2.0;

        // Choose gutter on the outer right side
        let gutter_x = right1.max(right2) + 28.0;

        vec![
            (right1, cy1),
            (gutter_x, cy1),
            (gutter_x, cy2),
            (right2, cy2),
        ]
    };

    let path_data = generate_rounded_path(&waypoints, fillet_radius);

    // Compute incoming direction for the terminal arrowhead
    let n = waypoints.len();
    let p_last = waypoints[n - 1];
    let p_prev = waypoints[n - 2];
    let dx = p_last.0 - p_prev.0;
    let dy = p_last.1 - p_prev.1;
    let len = (dx * dx + dy * dy).sqrt().max(0.001);
    let arrow = (p_last.0, p_last.1, dx / len, dy / len);

    // Compute optimal label position: midpoint of the longest rectilinear segment
    let mut max_seg_len = -1.0_f32;
    let mut label_pos = (
        (waypoints[0].0 + p_last.0) / 2.0,
        (waypoints[0].1 + p_last.1) / 2.0,
    );

    for i in 0..waypoints.len() - 1 {
        let p_a = waypoints[i];
        let p_b = waypoints[i + 1];
        let seg_len = ((p_b.0 - p_a.0).powi(2) + (p_b.1 - p_a.1).powi(2)).sqrt();
        if seg_len > max_seg_len {
            max_seg_len = seg_len;
            label_pos = ((p_a.0 + p_b.0) / 2.0, (p_a.1 + p_b.1) / 2.0);
        }
    }

    RoutedEdge {
        path_data,
        arrow,
        label_pos,
    }
}

/// Generate SVG path string with quadratic Bézier fillets at all 90-degree corners
pub fn generate_rounded_path(waypoints: &[(f32, f32)], radius: f32) -> String {
    if waypoints.len() < 2 {
        return String::new();
    }
    if waypoints.len() == 2 {
        return format!(
            "M {:.1} {:.1} L {:.1} {:.1}",
            waypoints[0].0, waypoints[0].1, waypoints[1].0, waypoints[1].1
        );
    }

    let mut d = format!("M {:.1} {:.1}", waypoints[0].0, waypoints[0].1);

    for i in 1..waypoints.len() - 1 {
        let p_prev = waypoints[i - 1];
        let p_curr = waypoints[i];
        let p_next = waypoints[i + 1];

        let v1_x = p_curr.0 - p_prev.0;
        let v1_y = p_curr.1 - p_prev.1;
        let len1 = (v1_x * v1_x + v1_y * v1_y).sqrt();

        let v2_x = p_next.0 - p_curr.0;
        let v2_y = p_next.1 - p_curr.1;
        let len2 = (v2_x * v2_x + v2_y * v2_y).sqrt();

        if len1 < 0.001 || len2 < 0.001 {
            continue;
        }

        // Limit fillet radius to at most half of the adjacent segments
        let r = radius.min(len1 / 2.0).min(len2 / 2.0);

        let f_start_x = p_curr.0 - (v1_x / len1) * r;
        let f_start_y = p_curr.1 - (v1_y / len1) * r;

        let f_end_x = p_curr.0 + (v2_x / len2) * r;
        let f_end_y = p_curr.1 + (v2_y / len2) * r;

        // Line to start of the fillet arc
        d.push_str(&format!(" L {:.1} {:.1}", f_start_x, f_start_y));
        // Quadratic Bézier curve through the corner vertex to the end of fillet arc
        d.push_str(&format!(
            " Q {:.1} {:.1} {:.1} {:.1}",
            p_curr.0, p_curr.1, f_end_x, f_end_y
        ));
    }

    let last = waypoints[waypoints.len() - 1];
    d.push_str(&format!(" L {:.1} {:.1}", last.0, last.1));
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_straight_orthogonal() {
        let edge =
            route_orthogonal_edge((100.0, 50.0, 100.0, 50.0), (100.0, 150.0, 100.0, 50.0), 6.0);
        assert!(edge.path_data.starts_with("M 150.0 100.0"));
        assert!(edge.path_data.contains("L 150.0 150.0"));
    }

    #[test]
    fn test_filleted_bends() {
        let edge =
            route_orthogonal_edge((50.0, 50.0, 100.0, 50.0), (250.0, 200.0, 100.0, 50.0), 8.0);
        assert!(edge.path_data.contains('Q'));
    }

    #[test]
    fn test_feedback_loop() {
        let edge =
            route_orthogonal_edge((100.0, 200.0, 100.0, 50.0), (100.0, 50.0, 100.0, 50.0), 8.0);
        assert!(edge.path_data.contains('Q'));
        assert_eq!(edge.arrow.2, -1.0); // Points left into target node
    }
}

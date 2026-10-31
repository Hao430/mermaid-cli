/*!
 * Rounded Orthogonal Edge Router
 * Inspired by Ondine (ondine.sh)
 * Generates clean rectilinear Manhattan paths with smooth quadratic Bézier corner fillets.
 */

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RoutedEdge {
    /// SVG path string: "M ... L ... Q ... L ..."
    pub path_data: String,
    /// Arrowhead placement: (tip_x, tip_y, dir_dx, dir_dy)
    pub arrow: (f32, f32, f32, f32),
    /// Optimal edge label position: (x, y)
    pub label_pos: (f32, f32),
}

/// Options for routing edges through shared horizontal distribution or collection buses,
/// multi-track outer gutters, or inter-column bypass corridors.
#[derive(Debug, Clone, Copy, Default)]
pub struct EdgeBusOptions {
    /// Shared horizontal distribution bus Y coordinate for fork branches
    pub fork_bus_y: Option<f32>,
    /// Shared horizontal collection bus Y coordinate for join branches
    pub join_bus_y: Option<f32>,
    /// Lane column center X coordinate for vertical transit bypass
    pub bypass_lane_x: Option<f32>,
    /// Assigned outer gutter channel X coordinate for backward/feedback loop edges
    pub gutter_x: Option<f32>,
    /// Exit from bottom with vertical drop because horizontal breakout to gutter is obstructed
    pub breakout_obstructed: bool,
    /// Enter into top with vertical drop because horizontal drop-in from gutter is obstructed
    pub drop_in_obstructed: bool,
    /// Omit emitting the vertical trunk from fork node to fork bus (M3 - R2)
    pub omit_source_trunk: bool,
    /// Omit emitting the vertical drop from join bus to join node (M3 - R2)
    pub omit_target_drop: bool,
}

/// Compute waypoints and rounded orthogonal path between source and target node bounding boxes
pub fn route_orthogonal_edge(
    from_box: (f32, f32, f32, f32), // (x, y, width, height)
    to_box: (f32, f32, f32, f32),
    fillet_radius: f32,
) -> RoutedEdge {
    route_orthogonal_edge_with_bus(from_box, to_box, fillet_radius, None)
}

/// Extended orthogonal router with support for T-shaped fork distribution
/// and join collection buses.
pub fn route_orthogonal_edge_with_bus(
    from_box: (f32, f32, f32, f32),
    to_box: (f32, f32, f32, f32),
    fillet_radius: f32,
    bus_options: Option<&EdgeBusOptions>,
) -> RoutedEdge {
    let (x1, y1, w1, h1) = from_box;
    let (x2, y2, w2, h2) = to_box;

    let cx1 = x1 + w1 / 2.0;
    let bottom1 = y1 + h1;
    let cx2 = x2 + w2 / 2.0;
    let top2 = y2;

    let is_gutter = bus_options.and_then(|b| b.gutter_x).is_some();
    let mut waypoints: Vec<(f32, f32)> = if !is_gutter && top2 >= bottom1 - 4.0 {
        let fork_bus = bus_options.and_then(|b| b.fork_bus_y);
        let join_bus = bus_options.and_then(|b| b.join_bus_y);
        let bypass_x = bus_options.and_then(|b| b.bypass_lane_x);

        if let Some(lx) = bypass_x {
            // Dedicated inter-column bypass corridor routing
            let has_top_bend = (cx1 - lx).abs() >= 2.0;
            let has_bottom_bend = (lx - cx2).abs() >= 2.0;

            if !has_top_bend && !has_bottom_bend {
                vec![(cx1, bottom1), (cx2, top2)]
            } else {
                let mut y_upper = fork_bus
                    .filter(|&y| y > bottom1 + 2.0 && y < top2 - 2.0)
                    .unwrap_or((bottom1 + 12.0).min(top2 - 4.0));
                let mut y_lower = join_bus
                    .filter(|&y| y > bottom1 + 2.0 && y < top2 - 2.0)
                    .unwrap_or((top2 - 12.0).max(bottom1 + 4.0));

                if has_top_bend && has_bottom_bend && y_upper >= y_lower {
                    let y_mid = (bottom1 + top2) / 2.0;
                    y_upper = bottom1 + (y_mid - bottom1) / 2.0;
                    y_lower = y_mid + (top2 - y_mid) / 2.0;
                }

                let mut pts = vec![(cx1, bottom1)];
                if has_top_bend {
                    pts.push((cx1, y_upper));
                    pts.push((lx, y_upper));
                }
                if has_bottom_bend {
                    pts.push((lx, y_lower));
                    pts.push((cx2, y_lower));
                }
                pts.push((cx2, top2));
                pts
            }
        } else if (cx1 - cx2).abs() < 2.0 {
            // Straight vertical line in same column
            vec![(cx1, bottom1), (cx2, top2)]
        } else {
            match (fork_bus, join_bus) {
                (Some(y_dist), Some(y_join))
                    if y_dist < y_join && y_dist > bottom1 && y_join < top2 =>
                {
                    vec![(cx1, bottom1), (cx1, y_dist), (cx2, y_dist), (cx2, top2)]
                }
                (Some(y_dist), _) if y_dist > bottom1 && y_dist < top2 => {
                    vec![(cx1, bottom1), (cx1, y_dist), (cx2, y_dist), (cx2, top2)]
                }
                (_, Some(y_join)) if y_join > bottom1 && y_join < top2 => {
                    vec![(cx1, bottom1), (cx1, y_join), (cx2, y_join), (cx2, top2)]
                }
                _ => {
                    // Standard two-step orthogonal bend at midpoint
                    let y_mid = bottom1 + (top2 - bottom1) / 2.0;
                    vec![(cx1, bottom1), (cx1, y_mid), (cx2, y_mid), (cx2, top2)]
                }
            }
        }
    } else {
        // Backward / Feedback / Cross-layer loop edge
        let right1 = x1 + w1;
        let cy1 = y1 + h1 / 2.0;
        let right2 = x2 + w2;
        let cy2 = y2 + h2 / 2.0;

        // Choose gutter on the outer right side: use assigned gutter_x if present,
        // otherwise fallback to default clearance for backwards compatibility.
        let gutter_x = bus_options
            .and_then(|b| b.gutter_x)
            .unwrap_or_else(|| right1.max(right2) + 28.0);

        let breakout_obstructed = bus_options.map(|b| b.breakout_obstructed).unwrap_or(false);
        let drop_in_obstructed = bus_options.map(|b| b.drop_in_obstructed).unwrap_or(false);

        let mut waypoints = Vec::with_capacity(6);
        if breakout_obstructed {
            let breakout_y = bus_options
                .and_then(|b| b.fork_bus_y)
                .unwrap_or(bottom1 + 12.0);
            waypoints.push((cx1, bottom1));
            waypoints.push((cx1, breakout_y));
            waypoints.push((gutter_x, breakout_y));
        } else {
            waypoints.push((right1, cy1));
            waypoints.push((gutter_x, cy1));
        }

        if drop_in_obstructed {
            let dropin_y = top2 - 12.0;
            waypoints.push((gutter_x, dropin_y));
            waypoints.push((cx2, dropin_y));
            waypoints.push((cx2, top2));
        } else {
            waypoints.push((gutter_x, cy2));
            waypoints.push((right2, cy2));
        }

        waypoints
    };

    // Apply T-bus trunk deduplication (M3 - R2):
    // When omit_source_trunk is true, omit initial vertical stem waypoint (cx1, bottom1)
    // so waypoints start directly at (cx1, fork_bus_y).
    // When omit_target_drop is true, omit final vertical drop waypoint (cx2, top2)
    // so waypoints end directly at (cx2, join_bus_y).
    if !is_gutter && top2 >= bottom1 - 4.0 {
        let omit_src = bus_options.map(|b| b.omit_source_trunk).unwrap_or(false);
        let omit_dst = bus_options.map(|b| b.omit_target_drop).unwrap_or(false);

        if omit_src && waypoints.len() >= 3 && (waypoints[0].0 - waypoints[1].0).abs() < 1.0 {
            waypoints.remove(0);
        }
        if omit_dst
            && waypoints.len() >= 3
            && (waypoints[waypoints.len() - 1].0 - waypoints[waypoints.len() - 2].0).abs() < 1.0
        {
            waypoints.pop();
        }
    }

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

// ============================================================================
// Multi-Level Gutter Channel Assigner (Milestone 2 - Requirement R1)
// ============================================================================

/// Represents an assigned outer gutter channel for a backward/feedback loop edge.
#[derive(Debug, Clone, PartialEq)]
pub struct GutterAssignment {
    /// Source node identifier
    pub from: String,
    /// Target node identifier
    pub to: String,
    /// Track index (0 is innermost track closest to nodes, 1 is next outer track, etc.)
    pub track_index: usize,
    /// Calculated physical X coordinate for the vertical gutter line
    pub gutter_x: f32,
    /// Vertical interval span (y_min, y_max)
    pub vertical_span: (f32, f32),
    /// True if an obstacle (sibling node) obstructs the direct horizontal breakout corridor
    pub breakout_obstructed: bool,
    /// True if an obstacle (sibling node) obstructs the direct horizontal drop-in corridor
    pub drop_in_obstructed: bool,
}

/// Assigner for obstacle-aware multi-channel outer gutters using interval graph coloring.
pub struct GutterChannelAssigner;

impl GutterChannelAssigner {
    /// Safe clearance distance from the rightmost obstacle boundary to base gutter track 0
    pub const BASE_CLEARANCE: f32 = 24.0;
    /// Horizontal separation between consecutive outer gutter tracks (>= 16px required)
    pub const TRACK_SPACING: f32 = 18.0;
    /// Vertical safety margin for interval overlap detection
    pub const VERTICAL_MARGIN: f32 = 8.0;

    /// Helper: check if horizontal breakout from `from` is obstructed by sibling nodes to its right
    pub fn is_breakout_obstructed(
        from_node: &str,
        node_positions: &HashMap<String, (f32, f32)>,
        node_size: (f32, f32),
    ) -> bool {
        let (node_width, node_height) = node_size;
        let &(x1, y1) = match node_positions.get(from_node) {
            Some(p) => p,
            None => return false,
        };
        let cy1 = y1 + node_height / 2.0;
        let right1 = x1 + node_width;

        node_positions.iter().any(|(k, &(xk, yk))| {
            if k == from_node {
                return false;
            }
            let right_k = xk + node_width;
            let v_hit = yk <= cy1 + 0.5 && cy1 <= yk + node_height + 0.5;
            let h_hit = right_k >= right1 - 0.5;
            v_hit && h_hit
        })
    }

    /// Helper: check if horizontal drop-in to `to` is obstructed by sibling nodes to its right
    pub fn is_drop_in_obstructed(
        to_node: &str,
        node_positions: &HashMap<String, (f32, f32)>,
        node_size: (f32, f32),
    ) -> bool {
        let (node_width, node_height) = node_size;
        let &(x2, y2) = match node_positions.get(to_node) {
            Some(p) => p,
            None => return false,
        };
        let cy2 = y2 + node_height / 2.0;
        let right2 = x2 + node_width;

        node_positions.iter().any(|(k, &(xk, yk))| {
            if k == to_node {
                return false;
            }
            let right_k = xk + node_width;
            let v_hit = yk <= cy2 + 0.5 && cy2 <= yk + node_height + 0.5;
            let h_hit = right_k >= right2 - 0.5;
            v_hit && h_hit
        })
    }

    /// Assign discrete, non-overlapping outer gutter tracks for all backward edges.
    ///
    /// # Arguments
    /// * `backward_edges` - List of (from, to) backward/feedback edge identifiers
    /// * `node_positions` - Calculated top-left (x, y) coordinates for all nodes
    /// * `node_size` - Standard (node_width, node_height)
    /// * `subgraph_bounds` - Calculated bounding boxes (x, y, w, h) for all subgraphs
    ///
    /// # Returns
    /// Map from (from, to) to `GutterAssignment`
    pub fn assign_channels(
        backward_edges: &[(String, String)],
        node_positions: &HashMap<String, (f32, f32)>,
        node_size: (f32, f32),
        subgraph_bounds: &HashMap<String, (f32, f32, f32, f32)>,
    ) -> HashMap<(String, String), GutterAssignment> {
        let mut results = HashMap::new();
        if backward_edges.is_empty() {
            return results;
        }

        let (node_width, node_height) = node_size;

        // Step 1: Collect edge spans and filter valid edges
        #[derive(Debug, Clone)]
        struct EdgeMeta {
            from: String,
            to: String,
            y_min: f32,
            y_max: f32,
            span_len: f32,
            breakout_obstructed: bool,
            drop_in_obstructed: bool,
        }

        let mut edge_metas = Vec::new();
        for (from, to) in backward_edges {
            if let (Some(&(_x1, y1)), Some(&(_x2, y2))) =
                (node_positions.get(from), node_positions.get(to))
            {
                let cy1 = y1 + node_height / 2.0;
                let cy2 = y2 + node_height / 2.0;

                let breakout_obstructed =
                    Self::is_breakout_obstructed(from, node_positions, node_size);
                let drop_in_obstructed = Self::is_drop_in_obstructed(to, node_positions, node_size);

                let span_y_start = if breakout_obstructed {
                    y1 + node_height + 12.0
                } else {
                    cy1
                };
                let span_y_end = if drop_in_obstructed { y2 - 12.0 } else { cy2 };

                let y_min = span_y_start.min(span_y_end);
                let y_max = span_y_start.max(span_y_end);
                let span_len = (y_max - y_min).abs();
                edge_metas.push(EdgeMeta {
                    from: from.clone(),
                    to: to.clone(),
                    y_min,
                    y_max,
                    span_len,
                    breakout_obstructed,
                    drop_in_obstructed,
                });
            }
        }

        if edge_metas.is_empty() {
            return results;
        }

        // Step 2: Sort backward edges by vertical span length ascending (inner loops first)
        edge_metas.sort_by(|a, b| {
            a.span_len
                .partial_cmp(&b.span_len)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.from.cmp(&b.from))
                .then_with(|| a.to.cmp(&b.to))
        });

        // Step 3: Group overlapping edges into connected components to ensure a shared X_base
        let n = edge_metas.len();
        let mut parent: Vec<usize> = (0..n).collect();

        fn find(parent: &mut [usize], i: usize) -> usize {
            if parent[i] == i {
                i
            } else {
                let p = parent[i];
                parent[i] = find(parent, p);
                parent[i]
            }
        }

        fn union(parent: &mut [usize], i: usize, j: usize) {
            let root_i = find(parent, i);
            let root_j = find(parent, j);
            if root_i != root_j {
                parent[root_i] = root_j;
            }
        }

        for i in 0..n {
            for j in (i + 1)..n {
                let overlap = edge_metas[i].y_max.min(edge_metas[j].y_max)
                    - edge_metas[i].y_min.max(edge_metas[j].y_min);
                if overlap >= -Self::VERTICAL_MARGIN {
                    union(&mut parent, i, j);
                }
            }
        }

        let mut components: HashMap<usize, Vec<usize>> = HashMap::new();
        for i in 0..n {
            let root = find(&mut parent, i);
            components.entry(root).or_default().push(i);
        }

        // Step 4: For each component, compute obstacle clearance boundary X_base and assign tracks
        for (_root, mut member_indices) in components {
            // Sort member edges by span length ascending (inner loops first)
            member_indices.sort_by(|&a, &b| {
                edge_metas[a]
                    .span_len
                    .partial_cmp(&edge_metas[b].span_len)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| edge_metas[a].from.cmp(&edge_metas[b].from))
                    .then_with(|| edge_metas[a].to.cmp(&edge_metas[b].to))
            });

            let comp_y_min = member_indices
                .iter()
                .map(|&i| edge_metas[i].y_min)
                .fold(f32::INFINITY, f32::min)
                - Self::VERTICAL_MARGIN;
            let comp_y_max = member_indices
                .iter()
                .map(|&i| edge_metas[i].y_max)
                .fold(f32::NEG_INFINITY, f32::max)
                + Self::VERTICAL_MARGIN;

            let mut max_obstacle_right = 0.0f32;

            // Check all nodes overlapping the component's vertical span
            for &(nx, ny) in node_positions.values() {
                let node_bottom = ny + node_height;
                if ny <= comp_y_max && node_bottom >= comp_y_min {
                    let right = nx + node_width;
                    if right > max_obstacle_right {
                        max_obstacle_right = right;
                    }
                }
            }

            // Check all subgraphs overlapping the component's vertical span
            for &(sg_x, sg_y, sg_w, sg_h) in subgraph_bounds.values() {
                let sg_bottom = sg_y + sg_h;
                if sg_y <= comp_y_max && sg_bottom >= comp_y_min {
                    let right = sg_x + sg_w;
                    if right > max_obstacle_right {
                        max_obstacle_right = right;
                    }
                }
            }

            let x_base = max_obstacle_right + Self::BASE_CLEARANCE;

            // Step 5: Greedy first-fit track coloring for member edges
            let mut tracks: Vec<Vec<(f32, f32)>> = Vec::new();

            for &idx in &member_indices {
                let meta = &edge_metas[idx];
                let span_y_min = meta.y_min;
                let span_y_max = meta.y_max;

                let mut assigned_track = None;
                for (t_idx, track_intervals) in tracks.iter().enumerate() {
                    let collides = track_intervals.iter().any(|&(i_min, i_max)| {
                        let overlap = span_y_max.min(i_max) - span_y_min.max(i_min);
                        overlap >= -Self::VERTICAL_MARGIN
                    });
                    if !collides {
                        assigned_track = Some(t_idx);
                        break;
                    }
                }

                let track_index = match assigned_track {
                    Some(t) => {
                        tracks[t].push((span_y_min, span_y_max));
                        t
                    }
                    None => {
                        let new_t = tracks.len();
                        tracks.push(vec![(span_y_min, span_y_max)]);
                        new_t
                    }
                };

                let gutter_x = x_base + track_index as f32 * Self::TRACK_SPACING;

                results.insert(
                    (meta.from.clone(), meta.to.clone()),
                    GutterAssignment {
                        from: meta.from.clone(),
                        to: meta.to.clone(),
                        track_index,
                        gutter_x,
                        vertical_span: (meta.y_min, meta.y_max),
                        breakout_obstructed: meta.breakout_obstructed,
                        drop_in_obstructed: meta.drop_in_obstructed,
                    },
                );
            }
        }

        results
    }
}

// ============================================================================
// Arc Hop Bridges (Milestone 4 - Requirement R3)
// ============================================================================

/// Radius of the semi-circular arc hop bridge (span = 2r = 10px, within the 8-12px spec)
const ARC_HOP_RADIUS: f32 = 5.0;

/// A straight "L" segment extracted from a path's token list.
#[derive(Debug, Clone, Copy, PartialEq)]
struct PathSeg {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

impl PathSeg {
    fn is_horizontal(&self) -> bool {
        (self.y1 - self.y2).abs() < 1.0 && (self.x1 - self.x2).abs() > 3.0
    }
    fn is_vertical(&self) -> bool {
        (self.x1 - self.x2).abs() < 1.0 && (self.y1 - self.y2).abs() > 3.0
    }
}

/// Post-process routed edge paths so every transverse orthogonal crossing between
/// two distinct edges is bridged by a semi-circular Arc Hop ("A" command).
///
/// The horizontal edge leaps over the vertical track: the crossing "L" command is
/// split and the bridge arc bulges upward (toward smaller Y) in both travel
/// directions. Crossings closer than one hop span to a segment endpoint or to a
/// previously placed hop are skipped to avoid degenerate geometry.
pub fn apply_arc_hops(path_datas: &mut [String]) {
    // 1. Tokenize every path
    let mut tokens: Vec<Vec<String>> = path_datas
        .iter()
        .map(|d| d.split_whitespace().map(str::to_string).collect())
        .collect();

    // 2. Extract straight "L" segments with their token index
    let mut segments: Vec<(usize, usize, PathSeg)> = Vec::new();
    for (pi, tok) in tokens.iter().enumerate() {
        let mut i = 0;
        let (mut cx, mut cy) = (0.0_f32, 0.0_f32);
        while i < tok.len() {
            match tok[i].as_str() {
                "M" | "L" => {
                    if i + 2 >= tok.len() {
                        break;
                    }
                    let nx: f32 = tok[i + 1].parse().unwrap_or(0.0);
                    let ny: f32 = tok[i + 2].parse().unwrap_or(0.0);
                    if tok[i] == "L" {
                        segments.push((
                            pi,
                            i,
                            PathSeg {
                                x1: cx,
                                y1: cy,
                                x2: nx,
                                y2: ny,
                            },
                        ));
                    }
                    cx = nx;
                    cy = ny;
                    i += 3;
                }
                "Q" => {
                    if i + 4 >= tok.len() {
                        break;
                    }
                    cx = tok[i + 3].parse().unwrap_or(0.0);
                    cy = tok[i + 4].parse().unwrap_or(0.0);
                    i += 5;
                }
                "A" => {
                    if i + 7 >= tok.len() {
                        break;
                    }
                    cx = tok[i + 6].parse().unwrap_or(0.0);
                    cy = tok[i + 7].parse().unwrap_or(0.0);
                    i += 8;
                }
                _ => i += 1,
            }
        }
    }

    // 3. Detect transverse crossings between distinct paths (strict interior, 2px margin),
    //    keyed by the horizontal segment's (path_idx, token_idx).
    let mut crossings: HashMap<(usize, usize), (PathSeg, Vec<f32>)> = HashMap::new();
    for &(h_pi, h_ti, h) in &segments {
        if !h.is_horizontal() {
            continue;
        }
        for &(v_pi, _, v) in &segments {
            if !v.is_vertical() || v_pi == h_pi {
                continue;
            }
            let cross_x = v.x1;
            let cross_y = h.y1;
            let (hx_min, hx_max) = (h.x1.min(h.x2), h.x1.max(h.x2));
            let (vy_min, vy_max) = (v.y1.min(v.y2), v.y1.max(v.y2));
            if cross_x > hx_min + 2.0
                && cross_x < hx_max - 2.0
                && cross_y > vy_min + 2.0
                && cross_y < vy_max - 2.0
            {
                let entry = crossings
                    .entry((h_pi, h_ti))
                    .or_insert_with(|| (h, Vec::new()));
                entry.1.push(cross_x);
            }
        }
    }
    if crossings.is_empty() {
        return;
    }

    // 4. Compute hop windows (absolute X spans) per horizontal segment
    let r = ARC_HOP_RADIUS;
    let mut hop_windows: HashMap<(usize, usize), Vec<(f32, f32)>> = HashMap::new();
    for (&(pi, ti), (seg, xs)) in &crossings {
        let (x_min, x_max) = (seg.x1.min(seg.x2), seg.x1.max(seg.x2));
        let mut sorted = xs.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mut windows: Vec<(f32, f32)> = Vec::new();
        let mut last_end = x_min;
        for &c in &sorted {
            let mut s = c - r;
            let mut e = c + r;
            // Clamp inside the segment interior
            if e > x_max - 1.0 {
                e = x_max - 1.0;
                s = e - 2.0 * r;
            }
            if s < x_min + 1.0 {
                s = x_min + 1.0;
                e = s + 2.0 * r;
            }
            if e > x_max - 1.0 {
                continue;
            }
            // Avoid overlapping a previously placed hop on the same segment
            if s < last_end + 1.0 {
                s = last_end + 1.0;
                e = s + 2.0 * r;
                if e > x_max - 1.0 {
                    continue;
                }
            }
            // The bridge center must stay within tolerance of the true crossing
            let center = (s + e) / 2.0;
            if (center - c).abs() > 11.0 {
                continue;
            }
            last_end = e;
            windows.push((s, e));
        }
        if !windows.is_empty() {
            hop_windows.insert((pi, ti), windows);
        }
    }
    if hop_windows.is_empty() {
        return;
    }

    // 5. Rewrite affected paths, replacing the crossing "L x y" with hop tokens
    for (&(pi, ti), windows) in &hop_windows {
        let seg = crossings[&(pi, ti)].0;
        let y = seg.y1;
        let forward = seg.x2 > seg.x1;
        let tok = &mut tokens[pi];
        let mut rebuilt: Vec<String> = Vec::with_capacity(tok.len() + windows.len() * 6);
        let mut i = 0;
        while i < tok.len() {
            if i == ti {
                for &(s, e) in windows {
                    // sweep=1 bulges upward for +X travel, sweep=0 for -X travel
                    let (t1, t2, sweep) = if forward { (s, e, "1") } else { (e, s, "0") };
                    rebuilt.push("L".to_string());
                    rebuilt.push(format!("{:.1}", t1));
                    rebuilt.push(format!("{:.1}", y));
                    rebuilt.push("A".to_string());
                    rebuilt.push(format!("{:.1}", ARC_HOP_RADIUS));
                    rebuilt.push(format!("{:.1}", ARC_HOP_RADIUS));
                    rebuilt.push("0".to_string());
                    rebuilt.push("0".to_string());
                    rebuilt.push(sweep.to_string());
                    rebuilt.push(format!("{:.1}", t2));
                    rebuilt.push(format!("{:.1}", y));
                }
                rebuilt.push("L".to_string());
                rebuilt.push(format!("{:.1}", seg.x2));
                rebuilt.push(format!("{:.1}", seg.y2));
                i += 3; // skip the original "L x y"
            } else {
                rebuilt.push(tok[i].clone());
                i += 1;
            }
        }
        path_datas[pi] = rebuilt.join(" ");
    }
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

    #[test]
    fn test_fork_distribution_bus() {
        let bus = EdgeBusOptions {
            fork_bus_y: Some(130.0),
            join_bus_y: None,
            bypass_lane_x: None,
            ..Default::default()
        };
        let edge = route_orthogonal_edge_with_bus(
            (100.0, 50.0, 100.0, 50.0),
            (250.0, 200.0, 100.0, 50.0),
            8.0,
            Some(&bus),
        );
        assert!(edge.path_data.contains("130.0"));
        assert!(edge.path_data.contains('Q'));
    }

    #[test]
    fn test_join_collection_bus() {
        let bus = EdgeBusOptions {
            fork_bus_y: None,
            join_bus_y: Some(170.0),
            bypass_lane_x: None,
            ..Default::default()
        };
        let edge = route_orthogonal_edge_with_bus(
            (50.0, 50.0, 100.0, 50.0),
            (200.0, 200.0, 100.0, 50.0),
            8.0,
            Some(&bus),
        );
        assert!(edge.path_data.contains("170.0"));
        assert!(edge.path_data.contains('Q'));
    }

    #[test]
    fn test_direct_bypass_bus() {
        let bus = EdgeBusOptions {
            fork_bus_y: Some(120.0),
            join_bus_y: Some(280.0),
            bypass_lane_x: Some(500.0),
            ..Default::default()
        };
        let edge = route_orthogonal_edge_with_bus(
            (100.0, 50.0, 100.0, 50.0),
            (100.0, 350.0, 100.0, 50.0),
            8.0,
            Some(&bus),
        );
        assert!(edge.path_data.contains("500.0"));
        assert!(edge.path_data.contains('Q'));
    }

    #[test]
    fn test_gutter_assigner_two_overlapping_loops_separation() {
        let mut positions = HashMap::new();
        positions.insert("A".to_string(), (100.0, 50.0));
        positions.insert("B".to_string(), (100.0, 150.0));
        positions.insert("C".to_string(), (100.0, 250.0));
        positions.insert("D".to_string(), (100.0, 350.0));

        let backward_edges = vec![
            ("C".to_string(), "A".to_string()),
            ("D".to_string(), "B".to_string()),
        ];

        let assignments = GutterChannelAssigner::assign_channels(
            &backward_edges,
            &positions,
            (130.0, 65.0),
            &HashMap::new(),
        );

        let g_ca = assignments
            .get(&("C".to_string(), "A".to_string()))
            .unwrap();
        let g_db = assignments
            .get(&("D".to_string(), "B".to_string()))
            .unwrap();

        assert_ne!(g_ca.track_index, g_db.track_index);
        let dx = (g_ca.gutter_x - g_db.gutter_x).abs();
        assert!(dx >= 18.0, "Delta X must be >= 18px (found {:.1})", dx);
    }

    #[test]
    fn test_gutter_assigner_clears_subgraph() {
        let mut positions = HashMap::new();
        positions.insert("A".to_string(), (100.0, 50.0));
        positions.insert("B".to_string(), (100.0, 150.0));

        let mut subgraphs = HashMap::new();
        // Subgraph extends to X = 400.0
        subgraphs.insert("G1".to_string(), (50.0, 30.0, 350.0, 220.0));

        let backward_edges = vec![("B".to_string(), "A".to_string())];

        let assignments = GutterChannelAssigner::assign_channels(
            &backward_edges,
            &positions,
            (130.0, 65.0),
            &subgraphs,
        );

        let g = assignments
            .get(&("B".to_string(), "A".to_string()))
            .unwrap();
        assert!(
            g.gutter_x >= 400.0 + 24.0,
            "Gutter must clear subgraph right bound 400.0 by >= 24px (found {:.1})",
            g.gutter_x
        );
    }

    #[test]
    fn test_gutter_assigner_disjoint_loops_track_reuse() {
        let mut positions = HashMap::new();
        positions.insert("A".to_string(), (100.0, 50.0));
        positions.insert("B".to_string(), (100.0, 120.0));
        positions.insert("C".to_string(), (100.0, 400.0));
        positions.insert("D".to_string(), (100.0, 480.0));

        let backward_edges = vec![
            ("B".to_string(), "A".to_string()),
            ("D".to_string(), "C".to_string()),
        ];

        let assignments = GutterChannelAssigner::assign_channels(
            &backward_edges,
            &positions,
            (130.0, 65.0),
            &HashMap::new(),
        );

        let g1 = assignments
            .get(&("B".to_string(), "A".to_string()))
            .unwrap();
        let g2 = assignments
            .get(&("D".to_string(), "C".to_string()))
            .unwrap();

        // Disjoint in Y: can safely reuse track 0
        assert_eq!(g1.track_index, 0);
        assert_eq!(g2.track_index, 0);
    }

    #[test]
    fn test_bypass_corridor_without_fork_bus() {
        let bus = EdgeBusOptions {
            fork_bus_y: None,
            join_bus_y: Some(300.0),
            bypass_lane_x: Some(320.0),
            ..Default::default()
        };
        let edge = route_orthogonal_edge_with_bus(
            (100.0, 50.0, 100.0, 50.0),  // cx = 150.0, bottom = 100.0
            (100.0, 350.0, 100.0, 50.0), // cx = 150.0, top = 350.0
            8.0,
            Some(&bus),
        );
        // Breakout top at 112.0, corridor transit at 320.0, drop at 300.0
        assert!(edge.path_data.contains("320.0"));
        assert!(edge.path_data.contains("112.0"));
        assert!(edge.path_data.contains("300.0"));
        assert!(edge.path_data.contains('Q'));
    }

    #[test]
    fn test_bypass_corridor_already_aligned() {
        let bus = EdgeBusOptions {
            fork_bus_y: None,
            join_bus_y: Some(300.0),
            bypass_lane_x: Some(320.0),
            ..Default::default()
        };
        let edge = route_orthogonal_edge_with_bus(
            (270.0, 50.0, 100.0, 50.0),  // cx = 320.0, already in corridor!
            (100.0, 350.0, 100.0, 50.0), // cx = 150.0
            8.0,
            Some(&bus),
        );
        // Starts at 320.0, no top bend, drops to 150.0 at 300.0
        assert!(edge.path_data.starts_with("M 320.0 100.0"));
        assert!(edge.path_data.contains("300.0"));
        assert!(edge.path_data.contains("150.0"));
    }

    #[test]
    fn test_obstructed_breakout_routing() {
        let bus = EdgeBusOptions {
            gutter_x: Some(300.0),
            breakout_obstructed: true,
            drop_in_obstructed: false,
            ..Default::default()
        };
        // Source node: [50.0, 100.0, 100.0, 50.0], Target node: [50.0, 20.0, 100.0, 50.0]
        let edge = route_orthogonal_edge_with_bus(
            (50.0, 100.0, 100.0, 50.0),
            (50.0, 20.0, 100.0, 50.0),
            8.0,
            Some(&bus),
        );
        // Breakout exits from bottom (cx=100.0, bottom=150.0) -> (100.0, 162.0) -> (300.0, 162.0)
        assert!(edge.path_data.starts_with("M 100.0 150.0"));
        assert!(edge.path_data.contains("162.0"));
    }

    #[test]
    fn test_obstructed_drop_in_routing() {
        let bus = EdgeBusOptions {
            gutter_x: Some(300.0),
            breakout_obstructed: false,
            drop_in_obstructed: true,
            ..Default::default()
        };
        let edge = route_orthogonal_edge_with_bus(
            (50.0, 100.0, 100.0, 50.0),
            (50.0, 20.0, 100.0, 50.0),
            8.0,
            Some(&bus),
        );
        // Drop-in enters into top (cx=100.0, top=20.0) from (100.0, 8.0)
        assert!(edge.path_data.contains(" 8.0"));
        assert!(edge.path_data.ends_with("100.0 20.0"));
        assert_eq!(edge.arrow, (100.0, 20.0, 0.0, 1.0)); // Downward arrowhead
    }

    #[test]
    fn test_omit_source_trunk() {
        let bus = EdgeBusOptions {
            fork_bus_y: Some(130.0),
            omit_source_trunk: true,
            ..Default::default()
        };
        // Source node: cx=150.0, bottom=100.0. Target: cx=300.0, top=200.0
        let edge = route_orthogonal_edge_with_bus(
            (100.0, 50.0, 100.0, 50.0),
            (250.0, 200.0, 100.0, 50.0),
            8.0,
            Some(&bus),
        );
        // Should start at (cx1=150.0, fork_bus_y=130.0), omitting (150.0, 100.0)
        assert!(
            edge.path_data.starts_with("M 150.0 130.0"),
            "Path data must start at fork bus Y: {}",
            edge.path_data
        );
        assert!(
            edge.path_data.ends_with("300.0 200.0"),
            "Path data must still end at target top: {}",
            edge.path_data
        );
    }

    #[test]
    fn test_omit_target_drop() {
        let bus = EdgeBusOptions {
            join_bus_y: Some(170.0),
            omit_target_drop: true,
            ..Default::default()
        };
        // Source node: cx=100.0, bottom=100.0. Target: cx=250.0, top=200.0
        let edge = route_orthogonal_edge_with_bus(
            (50.0, 50.0, 100.0, 50.0),
            (200.0, 200.0, 100.0, 50.0),
            8.0,
            Some(&bus),
        );
        // Should start at (cx1=100.0, bottom=100.0) and end at (cx2=250.0, join_bus_y=170.0)
        assert!(
            edge.path_data.starts_with("M 100.0 100.0"),
            "Path data must start at source bottom: {}",
            edge.path_data
        );
        assert!(
            edge.path_data.ends_with("250.0 170.0"),
            "Path data must end at join bus Y: {}",
            edge.path_data
        );
    }

    #[test]
    fn test_omit_both_source_trunk_and_target_drop() {
        let bus = EdgeBusOptions {
            fork_bus_y: Some(150.0),
            join_bus_y: Some(150.0),
            omit_source_trunk: true,
            omit_target_drop: true,
            ..Default::default()
        };
        let edge = route_orthogonal_edge_with_bus(
            (100.0, 50.0, 100.0, 50.0),
            (250.0, 200.0, 100.0, 50.0),
            8.0,
            Some(&bus),
        );
        // Waypoints pruned to [(150.0, 150.0), (300.0, 150.0)]
        assert_eq!(edge.path_data, "M 150.0 150.0 L 300.0 150.0");
    }
}

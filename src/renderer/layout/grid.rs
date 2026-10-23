//! Subgraph-scoped column grid, dynamic width calculation, bounding box auto-fitting,
//! and phase-level vertical stacking for flowchart horizontal parallel branch expansion.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::parser::ast::{Statement, Subgraph};
use crate::renderer::layout::branch::BranchAnalyzer;

/// Standard layout geometry constants
pub const NODE_WIDTH: f32 = 130.0;
pub const NODE_HEIGHT: f32 = 65.0;
pub const HORIZONTAL_SPACING: f32 = 40.0;
pub const VERTICAL_SPACING: f32 = 25.0;
pub const SUBGRAPH_PADDING_X: f32 = 20.0;
pub const SUBGRAPH_PADDING_BOTTOM: f32 = 20.0;
pub const SUBGRAPH_TITLE_HEADER_HEIGHT: f32 = 36.0;
pub const SUBGRAPH_NO_TITLE_PADDING_TOP: f32 = 20.0;
pub const CANVAS_PADDING: f32 = 40.0;
pub const INTER_SUBGRAPH_SPACING: f32 = 10.0;

/// Represents a column grid scoped to an individual subgraph container.
#[derive(Debug, Clone, PartialEq)]
pub struct SubgraphColumnGrid {
    /// Identifier of the subgraph
    pub subgraph_id: String,
    /// Number of active horizontal parallel lanes inside this subgraph
    pub active_lanes: usize,
    /// Calculated dynamic width of the subgraph
    pub width: f32,
    /// Center X coordinate of the subgraph on the canvas
    pub center_x: f32,
}

impl SubgraphColumnGrid {
    /// Computes the dynamic width of a subgraph given active lane count M.
    ///
    /// Mathematical formula:
    /// ```text
    /// W_sg = M * W_node + (M - 1) * S_h + 2 * P_sg
    /// ```
    ///
    /// Properties:
    /// - M = 1 => 1 * 130 + 0 * 40 + 2 * 20 = 170px
    /// - M = 2 => 2 * 130 + 1 * 40 + 2 * 20 = 340px
    /// - M = 3 => 3 * 130 + 2 * 40 + 2 * 20 = 510px
    /// - M = 4 => 4 * 130 + 3 * 40 + 2 * 20 = 680px
    #[inline]
    pub fn compute_width(
        active_lanes: usize,
        node_width: f32,
        h_spacing: f32,
        padding_x: f32,
    ) -> f32 {
        let m = active_lanes.max(1);
        m as f32 * node_width + (m - 1) as f32 * h_spacing + 2.0 * padding_x
    }

    /// Computes the node top-left X coordinate within this subgraph grid for a given lane slot offset.
    ///
    /// Mathematical formula:
    /// ```text
    /// X_center = center_x + slot_offset * (node_width + h_spacing)
    /// X_node   = X_center - node_width / 2.0
    /// ```
    #[inline]
    pub fn compute_node_x(&self, slot_offset: f32, node_width: f32, h_spacing: f32) -> f32 {
        self.center_x + slot_offset * (node_width + h_spacing) - node_width / 2.0
    }
}

/// Recursively collect all node IDs defined or referenced within subgraph statements.
pub fn collect_subgraph_nodes(statements: &[Statement], ids: &mut HashSet<String>) {
    for stmt in statements {
        match stmt {
            Statement::NodeDef { id, .. } => {
                ids.insert(id.clone());
            }
            Statement::EdgeDef { from, to, .. } => {
                ids.insert(from.clone());
                ids.insert(to.clone());
            }
            Statement::Block {
                statements: sub_stmts,
                ..
            } => {
                collect_subgraph_nodes(sub_stmts, ids);
            }
            _ => {}
        }
    }
}

/// Result of layout grid computation containing node positions, subgraph bounding boxes,
/// and canvas dimensions.
#[derive(Debug, Clone, Default)]
pub struct LayoutGrid {
    /// Calculated top-left position (x, y) for each node ID
    pub node_positions: HashMap<String, (f32, f32)>,
    /// Calculated bounding box (x, y, w, h) for each subgraph ID (and title if distinct)
    pub subgraph_bounds: HashMap<String, (f32, f32, f32, f32)>,
    /// Subgraph column grid configurations
    pub subgraph_grids: HashMap<String, SubgraphColumnGrid>,
    /// Effective overall canvas width
    pub effective_width: f32,
    /// Effective overall canvas height
    pub effective_height: f32,
}

impl LayoutGrid {
    pub const NODE_WIDTH: f32 = NODE_WIDTH;
    pub const NODE_HEIGHT: f32 = NODE_HEIGHT;
    pub const H_SPACING: f32 = HORIZONTAL_SPACING;
    pub const V_SPACING: f32 = VERTICAL_SPACING;
    pub const PADDING: f32 = CANVAS_PADDING;
    pub const SUBGRAPH_PADDING_X: f32 = SUBGRAPH_PADDING_X;
    pub const SUBGRAPH_PADDING_BOTTOM: f32 = SUBGRAPH_PADDING_BOTTOM;
    pub const SUBGRAPH_PADDING_TOP_WITH_TITLE: f32 = SUBGRAPH_TITLE_HEADER_HEIGHT;
    pub const SUBGRAPH_PADDING_TOP_NO_TITLE: f32 = SUBGRAPH_NO_TITLE_PADDING_TOP;
    pub const INTER_SUBGRAPH_SPACING: f32 = INTER_SUBGRAPH_SPACING;

    /// Resolves the lane slot offset for a node with symmetric spine alignment.
    ///
    /// Preserves exact slots for top-level forks (parent_slot == 0.0).
    /// For nested forks on side lanes (parent_slot != 0.0), keeps the forward
    /// trunk node aligned with the parent lane spine, moves inward retry loops
    /// toward the center, and prevents outward drift beyond the lane envelope.
    pub fn align_lane_slot(nid: &str, branch_analyzer: &BranchAnalyzer) -> f32 {
        let raw_slot = branch_analyzer.slot_for_node(nid);

        if let Some(&lane_id) = branch_analyzer.node_to_lane.get(nid) {
            if let Some(lane) = branch_analyzer.lanes.get(lane_id) {
                let parent_slot = branch_analyzer.slot_for_node(&lane.fork_node);
                if parent_slot.abs() > 1e-4 {
                    let dir = if parent_slot > 0.0 { 1.0 } else { -1.0 };

                    let leads_to_join = branch_analyzer
                        .forward_neighbors(nid)
                        .map(|nbrs| nbrs.iter().any(|nxt| branch_analyzer.is_join(nxt)))
                        .unwrap_or(false);

                    if leads_to_join {
                        return parent_slot;
                    }

                    let is_loop = branch_analyzer
                        .back_edges
                        .iter()
                        .any(|(from, _)| from == nid)
                        || ((raw_slot - parent_slot).abs() < 1e-4
                            && branch_analyzer
                                .forward_neighbors(nid)
                                .map(|nbrs| nbrs.is_empty())
                                .unwrap_or(true));

                    if is_loop {
                        return parent_slot - dir * 1.0;
                    }

                    if (raw_slot - parent_slot).abs() > 1.0 + 1e-4 {
                        return parent_slot;
                    }
                }
            }
        }

        raw_slot
    }

    /// Computes dynamic subgraph widths, node positions scoped to subgraph column grids,
    /// auto-fitted bounding boxes, and vertical phase stacking.
    pub fn compute(
        layers: &[Vec<String>],
        subgraphs: &[Subgraph],
        branch_analyzer: &BranchAnalyzer,
        canvas_width: u32,
        canvas_height: u32,
    ) -> Self {
        // Step 1: Collect node-to-subgraph mappings
        let mut node_to_sg: HashMap<String, usize> = HashMap::new();
        let mut sg_nodes: HashMap<usize, HashSet<String>> = HashMap::new();

        for (sg_idx, sg) in subgraphs.iter().enumerate() {
            let mut ids = HashSet::new();
            collect_subgraph_nodes(&sg.statements, &mut ids);
            for id in &ids {
                node_to_sg.insert(id.clone(), sg_idx);
            }
            sg_nodes.insert(sg_idx, ids);
        }

        // Layer lookup
        let mut node_to_layer: HashMap<String, usize> = HashMap::new();
        for (l_idx, layer) in layers.iter().enumerate() {
            for id in layer {
                node_to_layer.insert(id.clone(), l_idx);
            }
        }

        // Step 2: Compute active lane count M_sg and dynamic width W_sg for each subgraph
        let mut sg_active_lanes: HashMap<usize, usize> = HashMap::new();
        let mut sg_widths: HashMap<usize, f32> = HashMap::new();

        for (sg_idx, _sg) in subgraphs.iter().enumerate() {
            let nodes = sg_nodes.get(&sg_idx).cloned().unwrap_or_default();
            if nodes.is_empty() {
                sg_active_lanes.insert(sg_idx, 0);
                sg_widths.insert(sg_idx, 0.0);
                continue;
            }

            // Collect all slot offsets present in this subgraph
            let mut slots: Vec<f32> = Vec::new();
            for n in &nodes {
                let slot = Self::align_lane_slot(n, branch_analyzer);
                slots.push(slot);

                // If node is a top-level fork, include its outgoing branch slots
                if branch_analyzer.is_fork(n) && branch_analyzer.slot_for_node(n).abs() < 1e-4 {
                    if let Some(neighbors) = branch_analyzer.forward_neighbors(n) {
                        let k = neighbors.len();
                        for i in 0..k {
                            slots.push(BranchAnalyzer::calculate_lane_slot(i, k));
                        }
                    }
                }
            }

            // Deduplicate slots within epsilon
            slots.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            slots.dedup_by(|a, b| (*a - *b).abs() < 1e-4);

            // Compute slot span lanes
            let slot_span_lanes =
                if let (Some(&min_s), Some(&max_s)) = (slots.first(), slots.last()) {
                    if max_s > min_s {
                        (max_s - min_s).round() as usize + 1
                    } else {
                        1
                    }
                } else {
                    1
                };

            // Compute active lanes: if subgraph has assigned branch lanes, use slot_span_lanes;
            // otherwise fall back to max layer node count for unlinked parallel nodes.
            let has_assigned_lanes = slots.iter().any(|s| s.abs() > 1e-4);
            let m_sg = if has_assigned_lanes {
                slot_span_lanes.max(1)
            } else {
                let mut max_layer_nodes = 0;
                for layer in layers {
                    let count = layer.iter().filter(|n| nodes.contains(*n)).count();
                    max_layer_nodes = max_layer_nodes.max(count);
                }
                slot_span_lanes.max(max_layer_nodes).max(1)
            };
            let w_sg = SubgraphColumnGrid::compute_width(
                m_sg,
                NODE_WIDTH,
                HORIZONTAL_SPACING,
                SUBGRAPH_PADDING_X,
            );

            sg_active_lanes.insert(sg_idx, m_sg);
            sg_widths.insert(sg_idx, w_sg);
        }

        // Step 3: Compute canvas effective width and subgraph centers
        let max_sg_width = sg_widths.values().copied().fold(0.0f32, f32::max);
        let max_layer_nodes_global = layers.iter().map(|l| l.len()).max().unwrap_or(1) as f32;
        let global_layers_width = max_layer_nodes_global * NODE_WIDTH
            + (max_layer_nodes_global - 1.0).max(0.0) * HORIZONTAL_SPACING
            + 2.0 * CANVAS_PADDING;

        let mut min_slot = 0.0f32;
        let mut max_slot = 0.0f32;
        for layer in layers {
            for nid in layer {
                let s = branch_analyzer.slot_for_node(nid);
                min_slot = min_slot.min(s);
                max_slot = max_slot.max(s);
            }
        }
        let slot_span = max_slot - min_slot;
        let slot_width =
            (slot_span + 1.0) * (NODE_WIDTH + HORIZONTAL_SPACING) + 2.0 * CANVAS_PADDING;

        let required_width = (max_sg_width + 2.0 * CANVAS_PADDING)
            .max(global_layers_width)
            .max(slot_width)
            .max(canvas_width as f32);
        let effective_width = required_width;
        let canvas_center_x = effective_width / 2.0;

        // Construct SubgraphColumnGrids
        let mut subgraph_grids: HashMap<String, SubgraphColumnGrid> = HashMap::new();
        for (sg_idx, sg) in subgraphs.iter().enumerate() {
            let active_lanes = sg_active_lanes.get(&sg_idx).copied().unwrap_or(1);
            let width =
                sg_widths
                    .get(&sg_idx)
                    .copied()
                    .unwrap_or(SubgraphColumnGrid::compute_width(
                        active_lanes,
                        NODE_WIDTH,
                        HORIZONTAL_SPACING,
                        SUBGRAPH_PADDING_X,
                    ));
            subgraph_grids.insert(
                sg.id.clone(),
                SubgraphColumnGrid {
                    subgraph_id: sg.id.clone(),
                    active_lanes,
                    width,
                    center_x: canvas_center_x,
                },
            );
        }

        // Step 4: Position nodes layer by layer
        let mut node_positions: HashMap<String, (f32, f32)> = HashMap::new();

        for (layer_idx, layer_nodes) in layers.iter().enumerate() {
            let y = CANVAS_PADDING + layer_idx as f32 * (NODE_HEIGHT + VERTICAL_SPACING);

            // Group nodes by containing subgraph
            let mut sg_to_layer_nodes: HashMap<Option<usize>, Vec<String>> = HashMap::new();
            for node_id in layer_nodes {
                let sg_opt = node_to_sg.get(node_id).copied();
                sg_to_layer_nodes
                    .entry(sg_opt)
                    .or_default()
                    .push(node_id.clone());
            }

            for (sg_opt, nodes_in_group) in sg_to_layer_nodes {
                let center_x = match sg_opt {
                    Some(sg_idx) => {
                        let sg = &subgraphs[sg_idx];
                        subgraph_grids[&sg.id].center_x
                    }
                    None => canvas_center_x,
                };

                let count = nodes_in_group.len();
                let all_zero_slots = nodes_in_group.iter().all(|nid| {
                    let s = branch_analyzer.slot_for_node(nid);
                    s.abs() < 1e-4
                });

                if count > 1 && all_zero_slots {
                    // Spread unassigned / top-level nodes evenly across center
                    let total_w =
                        count as f32 * NODE_WIDTH + (count as f32 - 1.0) * HORIZONTAL_SPACING;
                    let start_x = center_x - total_w / 2.0;
                    for (idx, nid) in nodes_in_group.iter().enumerate() {
                        let x = start_x + idx as f32 * (NODE_WIDTH + HORIZONTAL_SPACING);
                        node_positions.insert(nid.clone(), (x, y));
                    }
                } else {
                    // Compute X coordinate bound to lane slot offset (eliminating center collapse)
                    let mut positioned: Vec<(String, f32)> = Vec::with_capacity(count);
                    for nid in &nodes_in_group {
                        let slot = Self::align_lane_slot(nid, branch_analyzer);

                        let x =
                            center_x + slot * (NODE_WIDTH + HORIZONTAL_SPACING) - NODE_WIDTH / 2.0;
                        positioned.push((nid.clone(), x));
                    }

                    // Enforce collision avoidance if multiple nodes occupy the same column
                    if count > 1 {
                        positioned.sort_by(|a, b| {
                            a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
                        });
                        for i in 1..positioned.len() {
                            let min_allowed = positioned[i - 1].1 + NODE_WIDTH + HORIZONTAL_SPACING;
                            if positioned[i].1 < min_allowed {
                                positioned[i].1 = min_allowed;
                            }
                        }
                    }

                    for (nid, x) in positioned {
                        node_positions.insert(nid, (x, y));
                    }
                }
            }

            // Global layer collision avoidance across all groups on this layer
            let mut layer_positioned: Vec<(String, f32)> = layer_nodes
                .iter()
                .filter_map(|id| node_positions.get(id).map(|&(x, _)| (id.clone(), x)))
                .collect();

            if layer_positioned.len() > 1 {
                layer_positioned
                    .sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
                let mut has_overlap = false;
                for i in 1..layer_positioned.len() {
                    if layer_positioned[i].1
                        < layer_positioned[i - 1].1 + NODE_WIDTH + HORIZONTAL_SPACING - 1e-4
                    {
                        has_overlap = true;
                        break;
                    }
                }

                if has_overlap {
                    for i in 1..layer_positioned.len() {
                        let min_allowed =
                            layer_positioned[i - 1].1 + NODE_WIDTH + HORIZONTAL_SPACING;
                        if layer_positioned[i].1 < min_allowed {
                            layer_positioned[i].1 = min_allowed;
                        }
                    }
                    let span = layer_positioned.last().unwrap().1 + NODE_WIDTH
                        - layer_positioned.first().unwrap().1;
                    let mid = layer_positioned.first().unwrap().1 + span / 2.0;
                    let shift = canvas_center_x - mid;
                    for (nid, x) in layer_positioned {
                        node_positions.insert(nid, (x + shift, y));
                    }
                }
            }
        }

        // Step 5: Compute initial subgraph bounding boxes (F10)
        let mut subgraph_bounds: HashMap<String, (f32, f32, f32, f32)> = HashMap::new();

        for (sg_idx, sg) in subgraphs.iter().enumerate() {
            let nodes = sg_nodes.get(&sg_idx).cloned().unwrap_or_default();
            if nodes.is_empty() {
                continue;
            }

            let mut min_node_x = f32::INFINITY;
            let mut min_node_y = f32::INFINITY;
            let mut max_node_x = f32::NEG_INFINITY;
            let mut max_node_y = f32::NEG_INFINITY;
            let mut has_node = false;

            for id in &nodes {
                if let Some(&(nx, ny)) = node_positions.get(id) {
                    min_node_x = min_node_x.min(nx);
                    min_node_y = min_node_y.min(ny);
                    max_node_x = max_node_x.max(nx + NODE_WIDTH);
                    max_node_y = max_node_y.max(ny + NODE_HEIGHT);
                    has_node = true;
                }
            }

            if !has_node {
                continue;
            }

            let padding_top = if sg.title.is_some() {
                SUBGRAPH_TITLE_HEADER_HEIGHT
            } else {
                SUBGRAPH_NO_TITLE_PADDING_TOP
            };

            let grid_width = sg_widths.get(&sg_idx).copied().unwrap_or(0.0);
            let mut sg_w = grid_width.max(max_node_x - min_node_x + 2.0 * SUBGRAPH_PADDING_X);
            let mut sg_x = canvas_center_x - sg_w / 2.0;

            // Ensure bounds fully enclose all internal nodes
            if min_node_x - SUBGRAPH_PADDING_X < sg_x {
                let diff = sg_x - (min_node_x - SUBGRAPH_PADDING_X);
                sg_x -= diff;
                sg_w += diff;
            }
            if max_node_x + SUBGRAPH_PADDING_X > sg_x + sg_w {
                sg_w = (max_node_x + SUBGRAPH_PADDING_X) - sg_x;
            }

            let sg_y = min_node_y - padding_top;
            let sg_h = max_node_y - min_node_y + padding_top + SUBGRAPH_PADDING_BOTTOM;

            subgraph_bounds.insert(sg.id.clone(), (sg_x, sg_y, sg_w, sg_h));
            if let Some(title) = &sg.title {
                if title != &sg.id {
                    subgraph_bounds.insert(title.clone(), (sg_x, sg_y, sg_w, sg_h));
                }
            }
        }

        // Step 6: Vertical Phase Stacking & Zero Collision Guarantee (F11)
        // Detect inter-subgraph topological dependencies
        let mut sg_adj: HashMap<usize, HashSet<usize>> = HashMap::new();
        let mut sg_in_deg: HashMap<usize, usize> = HashMap::new();
        for i in 0..subgraphs.len() {
            sg_adj.entry(i).or_default();
            sg_in_deg.insert(i, 0);
        }

        for (sg_a_idx, a_nodes) in &sg_nodes {
            for a_node in a_nodes {
                if let Some(neighbors) = branch_analyzer.forward_neighbors(a_node) {
                    for b_node in neighbors {
                        if let Some(&sg_b_idx) = node_to_sg.get(b_node) {
                            if *sg_a_idx != sg_b_idx
                                && sg_adj.get_mut(sg_a_idx).unwrap().insert(sg_b_idx)
                            {
                                *sg_in_deg.entry(sg_b_idx).or_insert(0) += 1;
                            }
                        }
                    }
                }
            }
        }

        let mut queue = VecDeque::new();
        for idx in 0..subgraphs.len() {
            if sg_in_deg.get(&idx).copied().unwrap_or(0) == 0 {
                queue.push_back(idx);
            }
        }

        let mut topo_order: Vec<usize> = Vec::new();
        while let Some(u) = queue.pop_front() {
            topo_order.push(u);
            if let Some(neighbors) = sg_adj.get(&u) {
                for &v in neighbors {
                    if let Some(d) = sg_in_deg.get_mut(&v) {
                        *d -= 1;
                        if *d == 0 {
                            queue.push_back(v);
                        }
                    }
                }
            }
        }

        for i in 0..subgraphs.len() {
            if !topo_order.contains(&i) {
                topo_order.push(i);
            }
        }

        // Stable sort topo_order by initial Y coordinate, tie-breaking by index
        topo_order.sort_by_key(|&idx| {
            let id = &subgraphs[idx].id;
            let y = subgraph_bounds
                .get(id)
                .map(|b| (b.1 * 100.0) as i64)
                .unwrap_or(0);
            (y, idx)
        });

        // Enforce zero vertical overlap: sg[i+1].y >= sg[i].y + sg[i].h + INTER_SUBGRAPH_SPACING
        for i in 0..topo_order.len() {
            let current_sg_idx = topo_order[i];
            let current_id = &subgraphs[current_sg_idx].id;

            let (_, curr_y, _, curr_h) = match subgraph_bounds.get(current_id).copied() {
                Some(b) => b,
                None => continue,
            };

            for j in (i + 1)..topo_order.len() {
                let next_sg_idx = topo_order[j];
                let next_id = &subgraphs[next_sg_idx].id;

                if let Some(&(_, next_y, _, _)) = subgraph_bounds.get(next_id) {
                    let required_min_y = curr_y + curr_h + INTER_SUBGRAPH_SPACING;
                    if next_y < required_min_y {
                        let shift_y = required_min_y - next_y;

                        // Shift member nodes of next and subsequent subgraphs
                        for &k_sg_idx in topo_order.iter().skip(j) {
                            if let Some(nodes) = sg_nodes.get(&k_sg_idx) {
                                for n in nodes {
                                    if let Some(pos) = node_positions.get_mut(n) {
                                        pos.1 += shift_y;
                                    }
                                }
                            }
                        }

                        // Shift external nodes (uncontained by any subgraph) at/below next_y
                        for (nid, pos) in node_positions.iter_mut() {
                            if !node_to_sg.contains_key(nid) && pos.1 >= next_y - 1e-4 {
                                pos.1 += shift_y;
                            }
                        }

                        // Shift next and subsequent subgraph bounds
                        for &k_sg_idx in topo_order.iter().skip(j) {
                            let k_id = &subgraphs[k_sg_idx].id;
                            if let Some(b) = subgraph_bounds.get_mut(k_id) {
                                b.1 += shift_y;
                            }
                            if let Some(title) = &subgraphs[k_sg_idx].title {
                                if title != k_id {
                                    if let Some(b) = subgraph_bounds.get_mut(title) {
                                        b.1 += shift_y;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Step 7: Compute effective canvas height
        let mut max_y = 0.0f32;
        for &(_, y) in node_positions.values() {
            max_y = max_y.max(y + NODE_HEIGHT);
        }
        for &(_, y, _, h) in subgraph_bounds.values() {
            max_y = max_y.max(y + h);
        }

        let effective_height = if canvas_width == 800 && canvas_height == 600 {
            max_y + CANVAS_PADDING
        } else {
            (max_y + CANVAS_PADDING).max(canvas_height as f32)
        };

        Self {
            node_positions,
            subgraph_bounds,
            subgraph_grids,
            effective_width,
            effective_height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
                shape: crate::parser::ast::NodeShape::Rect,
            })
            .collect();

        Subgraph {
            id: id.to_string(),
            title: title.map(|t| t.to_string()),
            statements,
        }
    }

    #[test]
    fn test_dynamic_width_formula() {
        assert_eq!(
            SubgraphColumnGrid::compute_width(
                1,
                NODE_WIDTH,
                HORIZONTAL_SPACING,
                SUBGRAPH_PADDING_X
            ),
            170.0
        );
        assert_eq!(
            SubgraphColumnGrid::compute_width(
                2,
                NODE_WIDTH,
                HORIZONTAL_SPACING,
                SUBGRAPH_PADDING_X
            ),
            340.0
        );
        assert_eq!(
            SubgraphColumnGrid::compute_width(
                3,
                NODE_WIDTH,
                HORIZONTAL_SPACING,
                SUBGRAPH_PADDING_X
            ),
            510.0
        );
        assert_eq!(
            SubgraphColumnGrid::compute_width(
                4,
                NODE_WIDTH,
                HORIZONTAL_SPACING,
                SUBGRAPH_PADDING_X
            ),
            680.0
        );
    }

    #[test]
    fn test_single_lane_subgraph_layout() {
        let nodes = to_s(&["A", "B"]);
        let edges = to_edges(&[("A", "B")]);
        let layers = to_layers(&[&["A"], &["B"]]);
        let subgraphs = vec![make_subgraph("SG1", Some("Single Lane"), &["A", "B"])];

        let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &subgraphs);
        let grid = LayoutGrid::compute(&layers, &subgraphs, &analyzer, 800, 600);

        let sg = grid.subgraph_bounds.get("SG1").expect("SG1 must exist");
        assert_eq!(sg.2, 170.0, "1-lane subgraph width must be exactly 170px");
        assert_eq!(
            sg.0 + sg.2 / 2.0,
            grid.effective_width / 2.0,
            "Subgraph must be centered on canvas"
        );

        let a_pos = grid.node_positions.get("A").unwrap();
        let b_pos = grid.node_positions.get("B").unwrap();
        assert_eq!(
            a_pos.0, b_pos.0,
            "Linear chain nodes must have identical X coordinates"
        );
    }

    #[test]
    fn test_two_lane_subgraph_layout_expansion() {
        let nodes = to_s(&["Fork", "Left", "Right"]);
        let edges = to_edges(&[("Fork", "Left"), ("Fork", "Right")]);
        let layers = to_layers(&[&["Fork"], &["Left", "Right"]]);
        let subgraphs = vec![make_subgraph(
            "SG2",
            Some("Two Lanes"),
            &["Fork", "Left", "Right"],
        )];

        let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &subgraphs);
        let grid = LayoutGrid::compute(&layers, &subgraphs, &analyzer, 800, 600);

        let sg = grid.subgraph_bounds.get("SG2").expect("SG2 must exist");
        assert_eq!(sg.2, 340.0, "2-lane subgraph width must be exactly 340px");

        let left_pos = grid.node_positions.get("Left").unwrap();
        let right_pos = grid.node_positions.get("Right").unwrap();
        let fork_pos = grid.node_positions.get("Fork").unwrap();

        let center_x = grid.effective_width / 2.0;
        assert_eq!(
            fork_pos.0 + NODE_WIDTH / 2.0,
            center_x,
            "Fork node centered at canvas centerline"
        );
        assert_eq!(
            left_pos.0 + NODE_WIDTH / 2.0,
            center_x - 85.0,
            "Left lane offset by -85px"
        );
        assert_eq!(
            right_pos.0 + NODE_WIDTH / 2.0,
            center_x + 85.0,
            "Right lane offset by +85px"
        );
        assert_eq!(
            right_pos.0 - (left_pos.0 + NODE_WIDTH),
            HORIZONTAL_SPACING,
            "Inter-lane gap = 40px"
        );
    }

    #[test]
    fn test_three_lane_subgraph_layout_expansion() {
        let nodes = to_s(&["Fork", "L", "M", "R"]);
        let edges = to_edges(&[("Fork", "L"), ("Fork", "M"), ("Fork", "R")]);
        let layers = to_layers(&[&["Fork"], &["L", "M", "R"]]);
        let subgraphs = vec![make_subgraph(
            "SG3",
            Some("Three Lanes"),
            &["Fork", "L", "M", "R"],
        )];

        let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &subgraphs);
        let grid = LayoutGrid::compute(&layers, &subgraphs, &analyzer, 800, 600);

        let sg = grid.subgraph_bounds.get("SG3").unwrap();
        assert_eq!(sg.2, 510.0, "3-lane subgraph width must be exactly 510px");
    }

    #[test]
    fn test_bounding_box_auto_fitting_padding() {
        let nodes = to_s(&["N"]);
        let edges = to_edges(&[]);
        let layers = to_layers(&[&["N"]]);

        let sg_titled = vec![make_subgraph("SG_T", Some("Title"), &["N"])];
        let sg_untitled = vec![make_subgraph("SG_U", None, &["N"])];

        let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);
        let grid_t = LayoutGrid::compute(&layers, &sg_titled, &analyzer, 800, 600);
        let grid_u = LayoutGrid::compute(&layers, &sg_untitled, &analyzer, 800, 600);

        let b_t = grid_t.subgraph_bounds.get("SG_T").unwrap();
        let b_u = grid_u.subgraph_bounds.get("SG_U").unwrap();

        let n_t_y = grid_t.node_positions.get("N").unwrap().1;
        let n_u_y = grid_u.node_positions.get("N").unwrap().1;

        assert_eq!(
            n_t_y - b_t.1,
            SUBGRAPH_TITLE_HEADER_HEIGHT,
            "Title requires 36px header space"
        );
        assert_eq!(
            n_u_y - b_u.1,
            SUBGRAPH_NO_TITLE_PADDING_TOP,
            "No-title requires 20px padding"
        );
    }

    #[test]
    fn test_vertical_phase_stacking_zero_overlap() {
        let nodes = to_s(&["A1", "A2", "B1", "B2"]);
        let edges = to_edges(&[("A1", "A2"), ("A2", "B1"), ("B1", "B2")]);
        let layers = to_layers(&[&["A1"], &["A2"], &["B1"], &["B2"]]);
        let subgraphs = vec![
            make_subgraph("G1", Some("Phase 1"), &["A1", "A2"]),
            make_subgraph("G2", Some("Phase 2"), &["B1", "B2"]),
        ];

        let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &subgraphs);
        let grid = LayoutGrid::compute(&layers, &subgraphs, &analyzer, 800, 600);

        let g1 = grid.subgraph_bounds.get("G1").unwrap();
        let g2 = grid.subgraph_bounds.get("G2").unwrap();

        assert!(
            g2.1 >= g1.1 + g1.3,
            "G2 top (y={}) must be >= G1 bottom (y={}) for zero overlap",
            g2.1,
            g1.1 + g1.3
        );
    }

    #[test]
    fn test_subgraph_x_positioning_g3_no_center_collapse() {
        // G3: C1 -> C2 -> (C3, C9) -> C4 -> C5 -> D1
        let layers = vec![
            vec!["C1".to_string()],
            vec!["C2".to_string()],
            vec!["C3".to_string(), "C9".to_string()],
            vec!["C4".to_string()],
            vec!["C5".to_string()],
            vec!["D1".to_string()],
        ];

        let subgraphs = vec![make_subgraph(
            "G3",
            Some("Payment"),
            &["C1", "C2", "C3", "C4", "C5", "C9"],
        )];

        let nodes = to_s(&["C1", "C2", "C3", "C4", "C5", "C9", "D1"]);
        let edges = to_edges(&[
            ("C1", "C2"),
            ("C2", "C3"),
            ("C2", "C9"),
            ("C3", "C4"),
            ("C4", "C5"),
            ("C5", "D1"),
            ("C9", "D1"),
        ]);

        let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &subgraphs);
        let grid = LayoutGrid::compute(&layers, &subgraphs, &analyzer, 800, 600);

        let c1 = grid.node_positions.get("C1").unwrap();
        let c2 = grid.node_positions.get("C2").unwrap();
        let c3 = grid.node_positions.get("C3").unwrap();
        let c4 = grid.node_positions.get("C4").unwrap();
        let c5 = grid.node_positions.get("C5").unwrap();
        let c9 = grid.node_positions.get("C9").unwrap();
        let d1 = grid.node_positions.get("D1").unwrap();

        let center_x = grid.effective_width / 2.0;

        // C1, C2 centered
        assert_eq!(c1.0 + NODE_WIDTH / 2.0, center_x);
        assert_eq!(c2.0 + NODE_WIDTH / 2.0, center_x);

        // C3 in left column
        assert_eq!(c3.0 + NODE_WIDTH / 2.0, center_x - 85.0);
        // C4 in left column (NO center collapse!)
        assert_eq!(c4.0, c3.0);
        // C5 in left column (NO center collapse!)
        assert_eq!(c5.0, c3.0);

        // C9 in right column
        assert_eq!(c9.0 + NODE_WIDTH / 2.0, center_x + 85.0);

        // D1 converges to center
        assert_eq!(d1.0 + NODE_WIDTH / 2.0, center_x);
    }
}

//! Forward DAG analysis, fork/join identification, symmetric lane slot assignment,
//! and virtual lane corridor reservations for flowchart horizontal parallel branch expansion.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::parser::ast::Subgraph;

/// Configuration strategy for resolving nested forks within a branch lane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NestedForkStrategy {
    /// Outward Spill (Default): The primary sub-branch inherits the parent's lane slot,
    /// while subsequent sub-branches spill outward away from the diagram centerline.
    /// Preserves half-integer/integer parity and keeps main flow lines strictly vertical.
    #[default]
    OutwardSpill,

    /// Additive Shift: Sub-branches shift symmetrically around the parent's slot offset
    /// with unit spacing: slot_j = S_parent + (j - (m - 1) / 2.0).
    AdditiveShift,

    /// Fractional Subdivision: Sub-branches subdivide the parent's lane width proportionally:
    /// slot_j = S_parent + (j - (m - 1) / 2.0) * scale.
    FractionalSubdivision,
}

/// Represents a discrete horizontal parallel branch lane originating from a fork node.
#[derive(Debug, Clone, PartialEq)]
pub struct BranchLane {
    /// Unique identifier for this branch lane.
    pub branch_id: usize,
    /// ID of the fork node from which this branch originated.
    pub fork_node: String,
    /// Symmetric horizontal slot offset (-1.0, -0.5, 0.0, +0.5, +1.0, etc.).
    pub slot_offset: f32,
    /// Set of node IDs assigned to this lane.
    pub nodes: HashSet<String>,
    /// Vertical layer span (min_layer, max_layer) occupied by nodes in this lane.
    pub depth_span: (usize, usize),
}

/// Represents a virtual corridor reserved for an unbalanced short branch across depth span D_max.
#[derive(Debug, Clone, PartialEq)]
pub struct LaneReservation {
    pub branch_id: usize,
    pub slot_offset: f32,
    pub layer: usize,
}

/// Forward DAG graph representation constructed by eliminating DFS back-edges.
#[derive(Debug, Clone)]
pub struct ForwardGraph {
    /// Eliminated back-edges closing cycles: (from, to)
    pub back_edges: HashSet<(String, String)>,
    /// Forward adjacency list preserving source declaration order: node -> [target_0, target_1, ...]
    pub adj: HashMap<String, Vec<String>>,
    /// Forward in-degree per node (d_in^+): node -> count
    pub in_degree: HashMap<String, usize>,
    /// Forward out-degree per node (d_out^+): node -> count
    pub out_degree: HashMap<String, usize>,
    /// Identified fork nodes where forward out-degree >= 2
    pub fork_nodes: HashSet<String>,
    /// Identified join nodes where forward in-degree >= 2
    pub join_nodes: HashSet<String>,
}

impl ForwardGraph {
    /// Builds the forward DAG from diagram nodes and edges by eliminating DFS cycle back-edges.
    ///
    /// Preserves source declaration order for outgoing edges from all nodes.
    pub fn build(node_ids: &[String], edges: &[(String, String)]) -> Self {
        let node_set: HashSet<&str> = node_ids.iter().map(|s| s.as_str()).collect();

        // 1. Build initial raw adjacency map while preserving edge declaration order and deduplicating
        let mut raw_adj: HashMap<&str, Vec<&str>> = HashMap::new();
        let mut raw_in_degrees: HashMap<&str, usize> = HashMap::new();

        for id in node_ids {
            raw_adj.entry(id.as_str()).or_default();
            raw_in_degrees.insert(id.as_str(), 0);
        }

        let mut seen_raw: HashSet<(&str, &str)> = HashSet::new();
        for (from, to) in edges {
            let f = from.as_str();
            let t = to.as_str();
            if node_set.contains(f) && node_set.contains(t) && seen_raw.insert((f, t)) {
                raw_adj.entry(f).or_default().push(t);
                *raw_in_degrees.entry(t).or_insert(0) += 1;
            }
        }

        // 2. Identify root nodes (raw in-degree == 0), preserving declaration order in node_ids
        let roots: Vec<&str> = node_ids
            .iter()
            .map(|s| s.as_str())
            .filter(|&id| *raw_in_degrees.get(id).unwrap_or(&0) == 0)
            .collect();

        let start_nodes = if roots.is_empty() {
            node_ids.iter().map(|s| s.as_str()).collect::<Vec<&str>>()
        } else {
            roots
        };

        // 3. DFS cycle detection using on_stack tracking
        let mut visited: HashSet<&str> = HashSet::new();
        let mut on_stack: HashSet<&str> = HashSet::new();
        let mut back_edges_ref: HashSet<(&str, &str)> = HashSet::new();

        fn dfs<'a>(
            u: &'a str,
            adj: &HashMap<&str, Vec<&'a str>>,
            visited: &mut HashSet<&'a str>,
            on_stack: &mut HashSet<&'a str>,
            back_edges: &mut HashSet<(&'a str, &'a str)>,
        ) {
            visited.insert(u);
            on_stack.insert(u);

            if let Some(neighbors) = adj.get(u) {
                for &v in neighbors {
                    if on_stack.contains(v) {
                        // Back-edge found: closes a cycle to an ancestor on the active recursion path
                        back_edges.insert((u, v));
                    } else if !visited.contains(v) {
                        dfs(v, adj, visited, on_stack, back_edges);
                    }
                }
            }

            on_stack.remove(u);
        }

        // Primary traversal from roots
        for &r in &start_nodes {
            if !visited.contains(r) {
                dfs(
                    r,
                    &raw_adj,
                    &mut visited,
                    &mut on_stack,
                    &mut back_edges_ref,
                );
            }
        }

        // Secondary traversal for disconnected components / cyclic islands
        for id in node_ids {
            let u = id.as_str();
            if !visited.contains(u) {
                dfs(
                    u,
                    &raw_adj,
                    &mut visited,
                    &mut on_stack,
                    &mut back_edges_ref,
                );
            }
        }

        // Convert back-edges to owned Strings
        let back_edges: HashSet<(String, String)> = back_edges_ref
            .iter()
            .map(|&(u, v)| (u.to_string(), v.to_string()))
            .collect();

        // 4. Construct forward DAG: E_fwd = E \ E_back
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut out_degree: HashMap<String, usize> = HashMap::new();

        for id in node_ids {
            adj.insert(id.clone(), Vec::new());
            in_degree.insert(id.clone(), 0);
            out_degree.insert(id.clone(), 0);
        }

        let mut seen_fwd: HashSet<(&str, &str)> = HashSet::new();
        for (from, to) in edges {
            let f = from.as_str();
            let t = to.as_str();
            if node_set.contains(f)
                && node_set.contains(t)
                && !back_edges_ref.contains(&(f, t))
                && seen_fwd.insert((f, t))
            {
                adj.get_mut(from).unwrap().push(to.clone());
                *in_degree.get_mut(to).unwrap() += 1;
                *out_degree.get_mut(from).unwrap() += 1;
            }
        }

        // 5. Fork identification (out_degree >= 2) and join identification (in_degree >= 2)
        let mut fork_nodes: HashSet<String> = HashSet::new();
        let mut join_nodes: HashSet<String> = HashSet::new();

        for (node, neighbors) in &adj {
            if neighbors.len() >= 2 {
                fork_nodes.insert(node.clone());
            }
        }

        for (node, &deg) in &in_degree {
            if deg >= 2 {
                join_nodes.insert(node.clone());
            }
        }

        ForwardGraph {
            back_edges,
            adj,
            in_degree,
            out_degree,
            fork_nodes,
            join_nodes,
        }
    }
}

/// Main layout analyzer for horizontal parallel branch expansion.
#[derive(Debug, Clone, Default)]
pub struct BranchAnalyzer {
    pub lanes: Vec<BranchLane>,
    pub node_to_lane: HashMap<String, usize>,
    pub node_to_slot: HashMap<String, f32>,
    pub join_nodes: HashSet<String>,
    pub fork_nodes: HashSet<String>,
    pub reservations: Vec<LaneReservation>,
    pub back_edges: HashSet<(String, String)>,
    pub forward_adj: HashMap<String, Vec<String>>,
    pub edge_to_lane: HashMap<(String, String), usize>,
}

impl BranchAnalyzer {
    /// Calculates the symmetric lane slot offset for branch index `i` among `k` branches.
    ///
    /// Formula:
    /// ```text
    /// slot(i, k) = i - (k - 1) / 2.0
    /// ```
    ///
    /// Properties:
    /// - k = 1 => [0.0]
    /// - k = 2 => [-0.5, +0.5]
    /// - k = 3 => [-1.0, 0.0, +1.0]
    /// - k = 4 => [-1.5, -0.5, +0.5, +1.5]
    #[inline]
    pub fn calculate_lane_slot(branch_index: usize, branch_count: usize) -> f32 {
        if branch_count <= 1 {
            0.0
        } else {
            branch_index as f32 - (branch_count - 1) as f32 / 2.0
        }
    }

    /// Resolves the slot offset for a sub-branch of a nested fork.
    pub fn calculate_nested_slot(
        parent_slot: f32,
        sub_index: usize,
        sub_count: usize,
        strategy: NestedForkStrategy,
    ) -> f32 {
        if sub_count <= 1 {
            return parent_slot;
        }

        match strategy {
            NestedForkStrategy::OutwardSpill => {
                if sub_index == 0 {
                    parent_slot
                } else {
                    let dir = if parent_slot < 0.0 {
                        -1.0
                    } else if parent_slot > 0.0 {
                        1.0
                    } else {
                        return Self::calculate_lane_slot(sub_index, sub_count);
                    };
                    parent_slot + dir * (sub_index as f32)
                }
            }
            NestedForkStrategy::AdditiveShift => {
                let local_offset = Self::calculate_lane_slot(sub_index, sub_count);
                parent_slot + local_offset
            }
            NestedForkStrategy::FractionalSubdivision => {
                let scale = 1.0 / (sub_count as f32);
                let local_offset = Self::calculate_lane_slot(sub_index, sub_count);
                parent_slot + local_offset * scale
            }
        }
    }

    /// Analyzes the forward DAG, identifies forks/joins, propagates symmetric lane slots,
    /// and reserves virtual lane corridors across depth span D_max.
    pub fn analyze(
        node_ids: &[String],
        edges: &[(String, String)],
        layers: &[Vec<String>],
        _subgraphs: &[Subgraph],
    ) -> Self {
        // Step 1: DFS cycle back-edge elimination and fork/join identification
        let mut fwd = ForwardGraph::build(node_ids, edges);

        // Step 2: Layer mapping lookup
        let mut node_to_layer: HashMap<String, usize> = HashMap::new();
        if !layers.is_empty() {
            for (l_idx, layer) in layers.iter().enumerate() {
                for id in layer {
                    node_to_layer.insert(id.clone(), l_idx);
                }
            }
            for id in node_ids {
                node_to_layer.entry(id.clone()).or_insert(0);
            }
        } else {
            // Topological depth fallback via Kahn's BFS on forward graph
            let mut in_degrees = fwd.in_degree.clone();
            let mut queue = VecDeque::new();
            for (id, &deg) in &in_degrees {
                if deg == 0 {
                    queue.push_back((id.clone(), 0));
                }
            }
            while let Some((curr, depth)) = queue.pop_front() {
                let entry = node_to_layer.entry(curr.clone()).or_insert(depth);
                *entry = (*entry).max(depth);
                if let Some(neighbors) = fwd.adj.get(&curr) {
                    for nxt in neighbors {
                        if let Some(deg) = in_degrees.get_mut(nxt) {
                            if *deg > 0 {
                                *deg -= 1;
                                if *deg == 0 {
                                    queue.push_back((nxt.clone(), depth + 1));
                                }
                            }
                        }
                    }
                }
            }
            for id in node_ids {
                node_to_layer.entry(id.clone()).or_insert(0);
            }
        }

        // Step 3: Initialize node slots (centerline 0.0 default)
        let mut node_to_slot: HashMap<String, f32> = HashMap::new();
        for id in node_ids {
            node_to_slot.insert(id.clone(), 0.0);
        }
        for join_id in &fwd.join_nodes {
            node_to_slot.insert(join_id.clone(), 0.0);
        }

        // Step 4: Symmetric Lane Slot Assignment & Downstream Propagation
        let mut lanes: Vec<BranchLane> = Vec::new();
        let mut node_to_lane: HashMap<String, usize> = HashMap::new();
        let mut fork_branch_map: HashMap<String, Vec<(usize, String)>> = HashMap::new();
        let mut edge_to_lane: HashMap<(String, String), usize> = HashMap::new();

        // Build fast layer position lookup: node_id -> index within its layer
        let mut node_to_pos: HashMap<String, usize> = HashMap::new();
        if !layers.is_empty() {
            for layer in layers {
                for (pos, id) in layer.iter().enumerate() {
                    node_to_pos.insert(id.clone(), pos);
                }
            }
        }

        let mut sorted_forks: Vec<String> = fwd.fork_nodes.iter().cloned().collect();
        sorted_forks.sort_by_key(|f| (node_to_layer.get(f).copied().unwrap_or(0), f.clone()));

        for fork_id in sorted_forks {
            let mut out_neighbors = match fwd.adj.get(&fork_id) {
                Some(neighbors) if neighbors.len() >= 2 => neighbors.clone(),
                _ => continue,
            };

            let parent_slot = node_to_slot.get(&fork_id).copied().unwrap_or(0.0);
            if parent_slot.abs() > 1e-4 {
                let reaches_join = |start: &str| -> bool {
                    let mut q = VecDeque::new();
                    let mut vis = HashSet::new();
                    q.push_back(start.to_string());
                    vis.insert(start.to_string());
                    while let Some(curr) = q.pop_front() {
                        if fwd.join_nodes.contains(&curr) {
                            return true;
                        }
                        if let Some(nbrs) = fwd.adj.get(&curr) {
                            for nxt in nbrs {
                                if vis.insert(nxt.clone()) {
                                    q.push_back(nxt.clone());
                                }
                            }
                        }
                    }
                    false
                };
                let is_direct_join = |child: &str| -> bool { fwd.join_nodes.contains(child) };
                out_neighbors.sort_by_key(|child| {
                    (is_direct_join(child), !reaches_join(child), child.clone())
                });
            } else if !layers.is_empty() {
                // Milestone 1 (R3A): Sort top-level fork outgoing branches by downstream layer position
                let u_layer = node_to_layer.get(&fork_id).copied().unwrap_or(0);

                // Find primary child layer (minimum layer among children, typically u_layer + 1)
                let min_child_layer = out_neighbors
                    .iter()
                    .filter_map(|c| node_to_layer.get(c).copied())
                    .min()
                    .unwrap_or(u_layer + 1);

                let is_primary = |c: &str| -> bool {
                    node_to_layer.get(c).copied() == Some(min_child_layer)
                        && !fwd.join_nodes.contains(c)
                };

                let primary_indices: Vec<usize> = out_neighbors
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, c)| if is_primary(c) { Some(idx) } else { None })
                    .collect();

                if !primary_indices.is_empty() {
                    let first_primary = *primary_indices.first().unwrap();
                    let last_primary = *primary_indices.last().unwrap();

                    let mut left_bypasses: Vec<String> = Vec::new();
                    let mut primary_children: Vec<String> = Vec::new();
                    let mut right_bypasses: Vec<String> = Vec::new();

                    for (idx, c) in out_neighbors.into_iter().enumerate() {
                        if idx < first_primary {
                            left_bypasses.push(c);
                        } else if idx > last_primary {
                            right_bypasses.push(c);
                        } else if is_primary(&c) {
                            primary_children.push(c);
                        } else {
                            // Direct bypass declared between primary children defaults to right
                            right_bypasses.push(c);
                        }
                    }

                    // Sort primary parallel children strictly by their horizontal rank in layers[min_child_layer]
                    primary_children
                        .sort_by_key(|c| node_to_pos.get(c).copied().unwrap_or(usize::MAX));

                    let mut reordered = left_bypasses;
                    reordered.extend(primary_children);
                    reordered.extend(right_bypasses);
                    out_neighbors = reordered;
                } else {
                    // All children are at different layers or direct joins: sort by layer position with AST tie-break
                    let mut indexed: Vec<(usize, String)> =
                        out_neighbors.into_iter().enumerate().collect();
                    indexed.sort_by_key(|&(orig_idx, ref c)| {
                        (node_to_pos.get(c).copied().unwrap_or(usize::MAX), orig_idx)
                    });
                    out_neighbors = indexed.into_iter().map(|(_, c)| c).collect();
                }
            }

            // Update fwd.adj to guarantee consistent iteration order across downstream passes
            fwd.adj.insert(fork_id.clone(), out_neighbors.clone());

            let k = out_neighbors.len();
            let mut branch_entries = Vec::with_capacity(k);

            for (i, child) in out_neighbors.iter().enumerate() {
                let slot_offset = if parent_slot != 0.0 {
                    Self::calculate_nested_slot(parent_slot, i, k, NestedForkStrategy::OutwardSpill)
                } else {
                    Self::calculate_lane_slot(i, k)
                };

                let branch_id = lanes.len();
                branch_entries.push((branch_id, child.clone()));
                edge_to_lane.insert((fork_id.clone(), child.clone()), branch_id);

                let mut lane = BranchLane {
                    branch_id,
                    fork_node: fork_id.clone(),
                    slot_offset,
                    nodes: HashSet::new(),
                    depth_span: (0, 0),
                };

                // Propagate downstream along single-input forward paths.
                // If child is a join node (d_in^+ >= 2), halt immediately (direct bypass branch)
                if !fwd.join_nodes.contains(child) {
                    let mut queue = VecDeque::new();
                    queue.push_back(child.clone());

                    let mut visited_branch = HashSet::new();
                    visited_branch.insert(child.clone());

                    while let Some(curr) = queue.pop_front() {
                        lane.nodes.insert(curr.clone());
                        node_to_lane.insert(curr.clone(), branch_id);
                        node_to_slot.insert(curr.clone(), slot_offset);

                        // If curr is a nested fork, halt outer branch linear propagation
                        if fwd.fork_nodes.contains(&curr) {
                            continue;
                        }

                        if let Some(next_nodes) = fwd.adj.get(&curr) {
                            for nxt in next_nodes {
                                edge_to_lane.insert((curr.clone(), nxt.clone()), branch_id);
                                // HALT propagation at Join Nodes (d_in^+ >= 2)
                                if fwd.join_nodes.contains(nxt) {
                                    continue;
                                }
                                // Monotonic propagation along single-input forward edges
                                if fwd.in_degree.get(nxt).copied().unwrap_or(0) == 1
                                    && visited_branch.insert(nxt.clone())
                                {
                                    queue.push_back(nxt.clone());
                                }
                            }
                        }
                    }
                }

                lanes.push(lane);
            }

            fork_branch_map.insert(fork_id, branch_entries);
        }

        // Step 5: Virtual Lane Reservations for Unbalanced Branches
        let mut reservations: Vec<LaneReservation> = Vec::new();

        for (fork_id, branch_entries) in fork_branch_map {
            let u_layer = *node_to_layer.get(&fork_id).unwrap_or(&0);

            // Compute depth range and reachable nodes for each branch before hitting any join
            let mut branch_spans: Vec<(usize, usize, HashSet<usize>)> =
                Vec::with_capacity(branch_entries.len());

            for (_bid, ref child) in &branch_entries {
                let mut reachable_before_join = HashSet::new();

                if !fwd.join_nodes.contains(child) {
                    let mut q = VecDeque::new();
                    q.push_back(child.clone());
                    reachable_before_join.insert(child.clone());

                    while let Some(curr) = q.pop_front() {
                        if let Some(nxts) = fwd.adj.get(&curr) {
                            for nxt in nxts {
                                if !fwd.join_nodes.contains(nxt)
                                    && reachable_before_join.insert(nxt.clone())
                                {
                                    q.push_back(nxt.clone());
                                }
                            }
                        }
                    }
                }

                let occupied_layers: HashSet<usize> = reachable_before_join
                    .iter()
                    .filter_map(|n| node_to_layer.get(n).copied())
                    .collect();

                let (min_l, max_l) = if occupied_layers.is_empty() {
                    // Direct bypass branch without intermediate nodes
                    (u_layer + 1, u_layer)
                } else {
                    let min_val = *occupied_layers.iter().min().unwrap();
                    let max_val = *occupied_layers.iter().max().unwrap();
                    (min_val, max_val)
                };

                branch_spans.push((min_l, max_l, occupied_layers));
            }

            let d_max = branch_spans
                .iter()
                .map(|&(_, max_l, _)| max_l)
                .max()
                .unwrap_or(u_layer);

            // Update depth_span and generate reservations
            for ((bid, _), (min_l, _, occupied_layers)) in
                branch_entries.into_iter().zip(branch_spans)
            {
                lanes[bid].depth_span = (min_l, d_max.max(min_l));

                if d_max >= min_l {
                    for l in min_l..=d_max {
                        if !occupied_layers.contains(&l) {
                            reservations.push(LaneReservation {
                                branch_id: bid,
                                slot_offset: lanes[bid].slot_offset,
                                layer: l,
                            });
                        }
                    }
                }
            }
        }

        // Sort reservations deterministically: (layer, slot_offset, branch_id)
        reservations.sort_by(|a, b| {
            a.layer
                .cmp(&b.layer)
                .then(
                    a.slot_offset
                        .partial_cmp(&b.slot_offset)
                        .unwrap_or(std::cmp::Ordering::Equal),
                )
                .then(a.branch_id.cmp(&b.branch_id))
        });
        reservations
            .dedup_by(|a, b| a.layer == b.layer && (a.slot_offset - b.slot_offset).abs() < 1e-4);

        // Step 6: Ensure all join nodes strictly converge to centerline (slot 0.0)
        for join_id in &fwd.join_nodes {
            node_to_slot.insert(join_id.clone(), 0.0);
        }

        Self {
            lanes,
            node_to_lane,
            node_to_slot,
            join_nodes: fwd.join_nodes,
            fork_nodes: fwd.fork_nodes,
            reservations,
            back_edges: fwd.back_edges,
            forward_adj: fwd.adj,
            edge_to_lane,
        }
    }

    /// Returns the assigned horizontal lane slot offset for a given node.
    #[inline]
    pub fn slot_for_node(&self, node_id: &str) -> f32 {
        self.node_to_slot.get(node_id).copied().unwrap_or(0.0)
    }

    /// Returns the reserved lane slot offset for a bypass edge spanning across intermediate layers.
    pub fn bypass_slot_for_edge(
        &self,
        from: &str,
        to: &str,
        from_layer: usize,
        to_layer: usize,
    ) -> Option<f32> {
        if to_layer <= from_layer + 1 {
            return None;
        }

        let branch_id = self
            .edge_to_lane
            .get(&(from.to_string(), to.to_string()))
            .copied()
            .or_else(|| self.node_to_lane.get(from).copied())?;

        // 1. Check explicit reservation in intermediate layers
        if let Some(res) = self
            .reservations
            .iter()
            .find(|r| r.branch_id == branch_id && r.layer > from_layer && r.layer < to_layer)
        {
            return Some(res.slot_offset);
        }

        // 2. Fall back to branch lane slot offset
        self.lanes.get(branch_id).map(|lane| lane.slot_offset)
    }

    /// Returns whether the node is a fork node (forward out_degree >= 2).
    #[inline]
    pub fn is_fork(&self, node_id: &str) -> bool {
        self.fork_nodes.contains(node_id)
    }

    /// Returns whether the node is a join node (forward in_degree >= 2).
    #[inline]
    pub fn is_join(&self, node_id: &str) -> bool {
        self.join_nodes.contains(node_id)
    }

    /// Returns the BranchLane reference for a node, if assigned.
    pub fn lane_for_node(&self, node_id: &str) -> Option<&BranchLane> {
        self.node_to_lane
            .get(node_id)
            .and_then(|&id| self.lanes.get(id))
    }

    /// Returns virtual lane reservations at a specific layer.
    pub fn reservations_at_layer(&self, layer: usize) -> Vec<&LaneReservation> {
        self.reservations
            .iter()
            .filter(|r| r.layer == layer)
            .collect()
    }

    /// Alias for `reservations_at_layer`.
    pub fn reservations_for_layer(&self, layer: usize) -> Vec<&LaneReservation> {
        self.reservations_at_layer(layer)
    }

    /// Checks if a given slot offset is reserved at a specific layer.
    pub fn is_slot_reserved(&self, layer: usize, slot_offset: f32) -> bool {
        self.reservations
            .iter()
            .any(|r| r.layer == layer && (r.slot_offset - slot_offset).abs() < 1e-4)
    }

    /// Checks if an edge was classified as a back-edge during DFS cycle elimination.
    pub fn is_back_edge(&self, from: &str, to: &str) -> bool {
        self.back_edges
            .contains(&(from.to_string(), to.to_string()))
    }

    /// Returns the forward outgoing neighbors of a node.
    pub fn forward_neighbors(&self, node_id: &str) -> Option<&[String]> {
        self.forward_adj.get(node_id).map(|v| v.as_slice())
    }

    /// Returns cycle back-edges detected by DFS traversal.
    pub fn detect_back_edges(
        node_ids: &[String],
        edges: &[(String, String)],
    ) -> HashSet<(String, String)> {
        ForwardGraph::build(node_ids, edges).back_edges
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

    #[test]
    fn test_linear_chain_no_forks() {
        // A -> B -> C -> D
        let nodes = to_s(&["A", "B", "C", "D"]);
        let edges = to_edges(&[("A", "B"), ("B", "C"), ("C", "D")]);
        let layers = to_layers(&[&["A"], &["B"], &["C"], &["D"]]);

        let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);

        assert_eq!(analyzer.fork_nodes.len(), 0);
        assert_eq!(analyzer.join_nodes.len(), 0);
        assert_eq!(analyzer.lanes.len(), 0);
        assert_eq!(analyzer.reservations.len(), 0);

        for id in &nodes {
            assert_eq!(analyzer.slot_for_node(id), 0.0);
            assert!(!analyzer.is_fork(id));
            assert!(!analyzer.is_join(id));
        }
    }

    #[test]
    fn test_two_way_symmetric_fork() {
        // F -> L (slot -0.5), F -> R (slot +0.5)
        let nodes = to_s(&["F", "L", "R"]);
        let edges = to_edges(&[("F", "L"), ("F", "R")]);
        let layers = to_layers(&[&["F"], &["L", "R"]]);

        let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);

        assert!(analyzer.is_fork("F"));
        assert!(!analyzer.is_join("F"));
        assert_eq!(analyzer.slot_for_node("F"), 0.0);
        assert_eq!(analyzer.slot_for_node("L"), -0.5);
        assert_eq!(analyzer.slot_for_node("R"), 0.5);

        assert_eq!(
            BranchAnalyzer::calculate_lane_slot(0, 2) + BranchAnalyzer::calculate_lane_slot(1, 2),
            0.0
        );
    }

    #[test]
    fn test_three_way_symmetric_fork() {
        // F -> Opt1 (-1.0), F -> Opt2 (0.0), F -> Opt3 (+1.0)
        let nodes = to_s(&["F", "Opt1", "Opt2", "Opt3"]);
        let edges = to_edges(&[("F", "Opt1"), ("F", "Opt2"), ("F", "Opt3")]);
        let layers = to_layers(&[&["F"], &["Opt1", "Opt2", "Opt3"]]);

        let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);

        assert!(analyzer.is_fork("F"));
        assert_eq!(analyzer.slot_for_node("Opt1"), -1.0);
        assert_eq!(analyzer.slot_for_node("Opt2"), 0.0);
        assert_eq!(analyzer.slot_for_node("Opt3"), 1.0);

        let sum: f32 = (0..3)
            .map(|i| BranchAnalyzer::calculate_lane_slot(i, 3))
            .sum();
        assert_eq!(sum, 0.0);
    }

    #[test]
    fn test_multi_branch_symmetric_even_and_odd() {
        // k = 1
        assert_eq!(BranchAnalyzer::calculate_lane_slot(0, 1), 0.0);

        // k = 4
        let slots_k4: Vec<f32> = (0..4)
            .map(|i| BranchAnalyzer::calculate_lane_slot(i, 4))
            .collect();
        assert_eq!(slots_k4, vec![-1.5, -0.5, 0.5, 1.5]);
        assert_eq!(slots_k4.iter().sum::<f32>(), 0.0);

        // k = 5
        let slots_k5: Vec<f32> = (0..5)
            .map(|i| BranchAnalyzer::calculate_lane_slot(i, 5))
            .collect();
        assert_eq!(slots_k5, vec![-2.0, -1.0, 0.0, 1.0, 2.0]);
        assert_eq!(slots_k5.iter().sum::<f32>(), 0.0);
    }

    #[test]
    fn test_monotonic_downstream_propagation_multi_step() {
        // F -> A1 -> A2 -> A3
        // F -> B1 -> B2 -> B3
        let nodes = to_s(&["F", "A1", "A2", "A3", "B1", "B2", "B3"]);
        let edges = to_edges(&[
            ("F", "A1"),
            ("A1", "A2"),
            ("A2", "A3"),
            ("F", "B1"),
            ("B1", "B2"),
            ("B2", "B3"),
        ]);
        let layers = to_layers(&[&["F"], &["A1", "B1"], &["A2", "B2"], &["A3", "B3"]]);

        let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);

        assert!(analyzer.is_fork("F"));
        assert_eq!(analyzer.slot_for_node("A1"), -0.5);
        assert_eq!(analyzer.slot_for_node("A2"), -0.5);
        assert_eq!(analyzer.slot_for_node("A3"), -0.5);

        assert_eq!(analyzer.slot_for_node("B1"), 0.5);
        assert_eq!(analyzer.slot_for_node("B2"), 0.5);
        assert_eq!(analyzer.slot_for_node("B3"), 0.5);

        let lane_a = analyzer.lane_for_node("A1").unwrap();
        assert_eq!(lane_a.slot_offset, -0.5);
        assert_eq!(lane_a.depth_span, (1, 3));
    }

    #[test]
    fn test_join_convergence_resets_slot_to_zero() {
        // F -> A -> J; F -> B -> J; J -> EndNode
        let nodes = to_s(&["F", "A", "B", "J", "EndNode"]);
        let edges = to_edges(&[
            ("F", "A"),
            ("F", "B"),
            ("A", "J"),
            ("B", "J"),
            ("J", "EndNode"),
        ]);
        let layers = to_layers(&[&["F"], &["A", "B"], &["J"], &["EndNode"]]);

        let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);

        assert!(analyzer.is_fork("F"));
        assert!(analyzer.is_join("J"));
        assert!(!analyzer.is_join("EndNode"));

        assert_eq!(analyzer.slot_for_node("A"), -0.5);
        assert_eq!(analyzer.slot_for_node("B"), 0.5);
        assert_eq!(analyzer.slot_for_node("J"), 0.0);
        assert_eq!(analyzer.slot_for_node("EndNode"), 0.0);

        // Lane A and B must not contain J
        let lane_a = analyzer.lane_for_node("A").unwrap();
        assert!(!lane_a.nodes.contains("J"));
        let lane_b = analyzer.lane_for_node("B").unwrap();
        assert!(!lane_b.nodes.contains("J"));
    }

    #[test]
    fn test_unbalanced_branch_reservations() {
        // C2 -> C3 -> C4 -> C5 -> D1 (left chain, depth 3)
        // C2 -> C9 -> D1             (right short branch, depth 1)
        let nodes = to_s(&["C2", "C3", "C4", "C5", "C9", "D1"]);
        let edges = to_edges(&[
            ("C2", "C3"),
            ("C2", "C9"),
            ("C3", "C4"),
            ("C4", "C5"),
            ("C5", "D1"),
            ("C9", "D1"),
        ]);
        let layers = to_layers(&[&["C2"], &["C3", "C9"], &["C4"], &["C5"], &["D1"]]);

        let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);

        assert!(analyzer.is_fork("C2"));
        assert!(analyzer.is_join("D1"));

        assert_eq!(analyzer.slot_for_node("C3"), -0.5);
        assert_eq!(analyzer.slot_for_node("C4"), -0.5);
        assert_eq!(analyzer.slot_for_node("C5"), -0.5);
        assert_eq!(analyzer.slot_for_node("C9"), 0.5);
        assert_eq!(analyzer.slot_for_node("D1"), 0.0);

        // C9 is missing at Layer 2 and Layer 3
        let res_layer_2 = analyzer.reservations_at_layer(2);
        let res_layer_3 = analyzer.reservations_at_layer(3);

        assert_eq!(res_layer_2.len(), 1);
        assert_eq!(res_layer_2[0].slot_offset, 0.5);
        assert_eq!(res_layer_3.len(), 1);
        assert_eq!(res_layer_3[0].slot_offset, 0.5);

        // Layer 4 is Join node D1, no reservation
        assert!(analyzer.reservations_at_layer(4).is_empty());

        let lane_c9 = analyzer.lane_for_node("C9").unwrap();
        assert_eq!(lane_c9.depth_span, (1, 3));
    }

    #[test]
    fn test_direct_bypass_edge_reservation() {
        // A -> B -> D
        // A -> D (direct bypass edge)
        let nodes = to_s(&["A", "B", "D"]);
        let edges = to_edges(&[("A", "B"), ("A", "D"), ("B", "D")]);
        let layers = to_layers(&[&["A"], &["B"], &["D"]]);

        let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);

        assert!(analyzer.is_fork("A"));
        assert!(analyzer.is_join("D"));

        // Branch 1 is direct bypass: missing at layer 1
        let res_layer_1 = analyzer.reservations_at_layer(1);
        assert_eq!(res_layer_1.len(), 1);
        assert_eq!(res_layer_1[0].slot_offset, 0.5);
    }

    #[test]
    fn test_cycle_isolation_and_retry_loops() {
        // C3 -> C4 -> C10 -> C4 (retry cycle 1)
        // C4 -> C6 -> C7 -> C3  (retry cycle 2)
        // C7 -> C8 (exit)
        // C4 -> C5 (success)
        let nodes = to_s(&["C3", "C4", "C5", "C6", "C7", "C8", "C10"]);
        let edges = to_edges(&[
            ("C3", "C4"),
            ("C4", "C10"),
            ("C10", "C4"), // Back-edge
            ("C4", "C6"),
            ("C6", "C7"),
            ("C7", "C3"), // Back-edge
            ("C7", "C8"),
            ("C4", "C5"),
        ]);
        let layers = to_layers(&[&["C3"], &["C4"], &["C10", "C6", "C5"], &["C7"], &["C8"]]);

        let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);

        assert!(analyzer.is_back_edge("C10", "C4"));
        assert!(analyzer.is_back_edge("C7", "C3"));
        assert_eq!(analyzer.back_edges.len(), 2);

        // C4 is not misclassified as join because C10->C4 is back-edge
        assert!(!analyzer.is_join("C4"));
        // C3 is not misclassified as join because C7->C3 is back-edge
        assert!(!analyzer.is_join("C3"));
        // C7 is not a fork because C7->C3 is back-edge
        assert!(!analyzer.is_fork("C7"));
    }

    #[test]
    fn test_nested_forks_outward_spill_and_additive_shift() {
        // Outward spill
        let s0 =
            BranchAnalyzer::calculate_nested_slot(-0.5, 0, 2, NestedForkStrategy::OutwardSpill);
        let s1 =
            BranchAnalyzer::calculate_nested_slot(-0.5, 1, 2, NestedForkStrategy::OutwardSpill);
        assert_eq!(s0, -0.5);
        assert_eq!(s1, -1.5);

        let r0 = BranchAnalyzer::calculate_nested_slot(0.5, 0, 2, NestedForkStrategy::OutwardSpill);
        let r1 = BranchAnalyzer::calculate_nested_slot(0.5, 1, 2, NestedForkStrategy::OutwardSpill);
        assert_eq!(r0, 0.5);
        assert_eq!(r1, 1.5);

        // Additive shift
        let a0 =
            BranchAnalyzer::calculate_nested_slot(-0.5, 0, 2, NestedForkStrategy::AdditiveShift);
        let a1 =
            BranchAnalyzer::calculate_nested_slot(-0.5, 1, 2, NestedForkStrategy::AdditiveShift);
        assert_eq!(a0, -1.0);
        assert_eq!(a1, 0.0);
    }

    #[test]
    fn test_multi_edge_deduplication() {
        // A -> B declared multiple times
        let nodes = to_s(&["A", "B"]);
        let edges = to_edges(&[("A", "B"), ("A", "B"), ("A", "B")]);
        let layers = to_layers(&[&["A"], &["B"]]);

        let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);

        assert!(!analyzer.is_fork("A"));
        assert!(!analyzer.is_join("B"));
        assert_eq!(analyzer.lanes.len(), 0);
    }

    #[test]
    fn test_order_flow_g3_full_payment_topology() {
        // Complete Subgraph G3 from examples/order_flow.mmd
        let nodes = to_s(&[
            "C1", "C2", "C3", "C4", "C5", "C6", "C7", "C8", "C9", "C10", "D1",
        ]);
        let edges = to_edges(&[
            ("C1", "C2"),
            ("C2", "C3"),
            ("C2", "C9"),
            ("C3", "C4"),
            ("C4", "C10"),
            ("C10", "C4"), // cycle 1
            ("C4", "C6"),
            ("C6", "C7"),
            ("C7", "C3"), // cycle 2
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

        let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);

        // Back-edges
        assert!(analyzer.is_back_edge("C10", "C4"));
        assert!(analyzer.is_back_edge("C7", "C3"));

        // Forks and Joins
        assert!(analyzer.is_fork("C2"));
        assert!(analyzer.is_fork("C4"));
        assert!(analyzer.is_join("D1"));

        // Main branch online payment C3, C4
        assert_eq!(analyzer.slot_for_node("C3"), -0.5);
        assert_eq!(analyzer.slot_for_node("C4"), -0.5);

        // Right branch COD
        assert_eq!(analyzer.slot_for_node("C9"), 0.5);

        // D1 converges to center
        assert_eq!(analyzer.slot_for_node("D1"), 0.0);

        // Virtual reservations for C9 across layers 3, 4, 5
        assert!(analyzer.is_slot_reserved(3, 0.5));
        assert!(analyzer.is_slot_reserved(4, 0.5));
        assert!(analyzer.is_slot_reserved(5, 0.5));
    }

    #[test]
    fn test_order_flow_g5_after_sales_topology() {
        // Subgraph G5 from examples/order_flow.mmd
        let nodes = to_s(&["E3", "E4", "E5", "E6", "E7", "E8", "E9", "End3"]);
        let edges = to_edges(&[
            ("E3", "E4"),
            ("E4", "E5"),
            ("E4", "E6"),
            ("E4", "End3"),
            ("E5", "E7"),
            ("E6", "E7"),
            ("E7", "E8"),
            ("E7", "E9"),
            ("E9", "End3"),
        ]);
        let layers = to_layers(&[
            &["E3"],
            &["E4"],
            &["E5", "E6"],
            &["E7"],
            &["E8", "E9"],
            &["End3"],
        ]);

        let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);

        assert!(analyzer.is_fork("E4"));
        assert!(analyzer.is_join("E7"));
        assert!(analyzer.is_fork("E7"));
        assert!(analyzer.is_join("End3"));

        // E4 is 3-way fork: E5 (-1.0), E6 (0.0), End3 (+1.0)
        assert_eq!(analyzer.slot_for_node("E5"), -1.0);
        assert_eq!(analyzer.slot_for_node("E6"), 0.0);

        // E7 is a join node so its slot converges to 0.0
        assert_eq!(analyzer.slot_for_node("E7"), 0.0);

        // End3 is a join node so its slot converges to 0.0
        assert_eq!(analyzer.slot_for_node("End3"), 0.0);

        // E4 -> End3 is a direct bypass branch across intermediate layers
        let res_layer_2 = analyzer.reservations_at_layer(2);
        assert!(res_layer_2
            .iter()
            .any(|r| (r.slot_offset - 1.0).abs() < 1e-4));
    }

    #[test]
    fn test_disconnected_components_with_cycles() {
        let nodes = to_s(&["A", "B", "X", "Y", "Z"]);
        let edges = to_edges(&[
            ("A", "B"),
            ("B", "A"), // Cycle in component 1
            ("X", "Y"),
            ("X", "Z"), // Fork in component 2
        ]);
        let layers = to_layers(&[&["A", "X"], &["B", "Y", "Z"]]);

        let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);

        assert!(analyzer.is_back_edge("B", "A"));
        assert!(analyzer.is_fork("X"));
        assert!(!analyzer.is_fork("A"));
    }

    #[test]
    fn test_empty_layers_topological_fallback() {
        // Caller passes empty layers: &[]
        let nodes = to_s(&["A", "B", "C", "D"]);
        let edges = to_edges(&[("A", "B"), ("A", "C"), ("B", "D"), ("C", "D")]);

        let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &[], &[]);

        assert!(analyzer.is_fork("A"));
        assert!(analyzer.is_join("D"));
        assert_eq!(analyzer.slot_for_node("B"), -0.5);
        assert_eq!(analyzer.slot_for_node("C"), 0.5);
        assert_eq!(analyzer.slot_for_node("D"), 0.0);
    }

    #[test]
    fn test_fork_out_neighbors_sorted_by_downstream_layer_position() {
        // F declared with Right first, then Left in AST
        let nodes = to_s(&["F", "Right", "Left"]);
        let edges = to_edges(&[("F", "Right"), ("F", "Left")]);
        // Crossing minimizer established Left at pos 0, Right at pos 1 in layer 1
        let layers = to_layers(&[&["F"], &["Left", "Right"]]);

        let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);

        assert_eq!(analyzer.slot_for_node("Left"), -0.5);
        assert_eq!(analyzer.slot_for_node("Right"), 0.5);
    }
}

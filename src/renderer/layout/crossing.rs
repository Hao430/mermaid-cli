//! In-layer two-sided Sugiyama barycenter ordering heuristic for flowchart DAGs.
//!
//! Reorders nodes within layers to minimize edge crossings while strictly preserving
//! subgraph containment boundaries and component stability.

use std::collections::{HashMap, HashSet};

use crate::parser::ast::Subgraph;
use crate::renderer::layout::grid::collect_subgraph_nodes;

/// Number of two-sided sweep passes (downward + upward) executed for convergence.
pub const DEFAULT_SWEEP_PASSES: usize = 2;

/// Minimizer implementing the Sugiyama two-sided in-layer barycenter heuristic.
pub struct BarycenterCrossingMinimizer;

impl BarycenterCrossingMinimizer {
    /// Reorders nodes within each layer of `layers` to minimize edge crossings while strictly
    /// preserving subgraph containment boundaries.
    ///
    /// # Arguments
    /// * `layers` - Mutable slice of layer node vectors.
    /// * `edges` - All graph edges `(from, to)`.
    /// * `back_edges` - DFS cycle back-edges to exclude from forward ordering.
    /// * `subgraphs` - Subgraph definitions for partition grouping.
    pub fn minimize_crossings(
        layers: &mut [Vec<String>],
        edges: &[(String, String)],
        back_edges: &HashSet<(String, String)>,
        subgraphs: &[Subgraph],
    ) {
        if layers.len() <= 1 {
            return;
        }

        // 1. Collect all valid node IDs present in layers
        let mut all_nodes: HashSet<&str> = HashSet::new();
        for layer in layers.iter() {
            for id in layer {
                all_nodes.insert(id.as_str());
            }
        }

        // 2. Build forward adjacency and parent maps (excluding cycle back-edges and self-loops)
        let mut forward_adj: HashMap<String, Vec<String>> = HashMap::new();
        let mut forward_parents: HashMap<String, Vec<String>> = HashMap::new();
        let mut seen_edges: HashSet<(&str, &str)> = HashSet::new();

        for (from, to) in edges {
            let f = from.as_str();
            let t = to.as_str();
            if f != t
                && all_nodes.contains(f)
                && all_nodes.contains(t)
                && !back_edges.contains(&(from.clone(), to.clone()))
                && seen_edges.insert((f, t))
            {
                forward_adj
                    .entry(from.clone())
                    .or_default()
                    .push(to.clone());
                forward_parents
                    .entry(to.clone())
                    .or_default()
                    .push(from.clone());
            }
        }

        // 3. Map each node to its containing subgraph index (if any)
        let mut node_to_sg: HashMap<String, usize> = HashMap::new();
        for (sg_idx, sg) in subgraphs.iter().enumerate() {
            let mut ids = HashSet::new();
            collect_subgraph_nodes(&sg.statements, &mut ids);
            for id in ids {
                node_to_sg.insert(id, sg_idx);
            }
        }

        // 4. Execute iterative two-sided Sugiyama sweeps
        for _pass in 0..DEFAULT_SWEEP_PASSES {
            // Downward sweep: layer 1 down to layers.len() - 1
            for d in 1..layers.len() {
                if layers[d].len() <= 1 {
                    continue;
                }
                let ref_positions: HashMap<String, f32> = layers[d - 1]
                    .iter()
                    .enumerate()
                    .map(|(idx, id)| (id.clone(), idx as f32))
                    .collect();

                let mut barycenters: HashMap<String, f32> = HashMap::with_capacity(layers[d].len());
                for (orig_idx, id) in layers[d].iter().enumerate() {
                    let bary = if let Some(parents) = forward_parents.get(id) {
                        let valid_positions: Vec<f32> = parents
                            .iter()
                            .filter_map(|p| ref_positions.get(p).copied())
                            .collect();
                        if !valid_positions.is_empty() {
                            valid_positions.iter().sum::<f32>() / (valid_positions.len() as f32)
                        } else {
                            orig_idx as f32
                        }
                    } else {
                        orig_idx as f32
                    };
                    barycenters.insert(id.clone(), bary);
                }

                Self::sort_layer_subgraph_preserving(&mut layers[d], &barycenters, &node_to_sg);
            }

            // Upward sweep: layer layers.len() - 2 down to 0
            for d in (0..layers.len().saturating_sub(1)).rev() {
                if layers[d].len() <= 1 {
                    continue;
                }
                let ref_positions: HashMap<String, f32> = layers[d + 1]
                    .iter()
                    .enumerate()
                    .map(|(idx, id)| (id.clone(), idx as f32))
                    .collect();

                let mut barycenters: HashMap<String, f32> = HashMap::with_capacity(layers[d].len());
                for (orig_idx, id) in layers[d].iter().enumerate() {
                    let bary = if let Some(children) = forward_adj.get(id) {
                        let valid_positions: Vec<f32> = children
                            .iter()
                            .filter_map(|c| ref_positions.get(c).copied())
                            .collect();
                        if !valid_positions.is_empty() {
                            valid_positions.iter().sum::<f32>() / (valid_positions.len() as f32)
                        } else {
                            orig_idx as f32
                        }
                    } else {
                        orig_idx as f32
                    };
                    barycenters.insert(id.clone(), bary);
                }

                Self::sort_layer_subgraph_preserving(&mut layers[d], &barycenters, &node_to_sg);
            }
        }
    }

    /// Sorts an individual layer by barycenter while strictly preserving subgraph partitions.
    pub fn sort_layer_subgraph_preserving(
        layer: &mut Vec<String>,
        barycenters: &HashMap<String, f32>,
        node_to_sg: &HashMap<String, usize>,
    ) {
        if layer.len() <= 1 {
            return;
        }

        // 1. Group nodes by containing subgraph Option<usize>
        #[derive(Debug, Clone)]
        struct NodeBaryEntry {
            id: String,
            barycenter: f32,
            orig_index: usize,
        }

        let mut group_map: HashMap<Option<usize>, Vec<NodeBaryEntry>> = HashMap::new();
        for (orig_index, nid) in layer.iter().enumerate() {
            let sg_opt = node_to_sg.get(nid).copied();
            let barycenter = barycenters.get(nid).copied().unwrap_or(orig_index as f32);
            group_map.entry(sg_opt).or_default().push(NodeBaryEntry {
                id: nid.clone(),
                barycenter,
                orig_index,
            });
        }

        // 2. Sort nodes within each subgraph group by their individual barycenters
        struct SubgraphGroup {
            nodes: Vec<String>,
            avg_barycenter: f32,
            min_orig_index: usize,
        }

        let mut groups: Vec<SubgraphGroup> = Vec::with_capacity(group_map.len());
        for (_key, mut entries) in group_map {
            entries.sort_by(|a, b| {
                a.barycenter
                    .partial_cmp(&b.barycenter)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.orig_index.cmp(&b.orig_index))
            });

            let sum_bary: f32 = entries.iter().map(|e| e.barycenter).sum();
            let avg_barycenter = sum_bary / (entries.len().max(1) as f32);
            let min_orig_index = entries.iter().map(|e| e.orig_index).min().unwrap_or(0);
            let node_ids: Vec<String> = entries.into_iter().map(|e| e.id).collect();

            groups.push(SubgraphGroup {
                nodes: node_ids,
                avg_barycenter,
                min_orig_index,
            });
        }

        // 3. Sort the subgraph groups by group average barycenter (with min_orig_index tie-breaker)
        groups.sort_by(|a, b| {
            a.avg_barycenter
                .partial_cmp(&b.avg_barycenter)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.min_orig_index.cmp(&b.min_orig_index))
        });

        // 4. Reconstruct the layer contiguously
        let mut reconstructed = Vec::with_capacity(layer.len());
        for group in groups {
            reconstructed.extend(group.nodes);
        }

        *layer = reconstructed;
    }

    /// Helper utility to count total geometric crossings between all consecutive layers.
    ///
    /// Two forward edges (u1, v1) and (u2, v2) between layer d and layer d+1 cross if and only if:
    /// (pos(u1) < pos(u2) && pos(v1) > pos(v2)) || (pos(u1) > pos(u2) && pos(v1) < pos(v2))
    pub fn count_crossings(
        layers: &[Vec<String>],
        edges: &[(String, String)],
        back_edges: &HashSet<(String, String)>,
    ) -> usize {
        if layers.len() <= 1 {
            return 0;
        }

        let mut node_to_layer_pos: HashMap<String, (usize, usize)> = HashMap::new();
        for (l_idx, layer) in layers.iter().enumerate() {
            for (p_idx, id) in layer.iter().enumerate() {
                node_to_layer_pos.insert(id.clone(), (l_idx, p_idx));
            }
        }

        let forward_edges: Vec<((usize, usize), (usize, usize))> = edges
            .iter()
            .filter(|(f, t)| f != t && !back_edges.contains(&(f.clone(), t.clone())))
            .filter_map(|(f, t)| {
                let p_from = node_to_layer_pos.get(f)?;
                let p_to = node_to_layer_pos.get(t)?;
                if p_from.0 < p_to.0 {
                    Some((*p_from, *p_to))
                } else {
                    None
                }
            })
            .collect();

        let mut crossings = 0;
        let m = forward_edges.len();
        for i in 0..m {
            for j in (i + 1)..m {
                let (u1, v1) = forward_edges[i];
                let (u2, v2) = forward_edges[j];

                if u1.0 == u2.0
                    && v1.0 == v2.0
                    && ((u1.1 < u2.1 && v1.1 > v2.1) || (u1.1 > u2.1 && v1.1 < v2.1))
                {
                    crossings += 1;
                }
            }
        }

        crossings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::{NodeShape, Statement};

    fn make_test_subgraph(id: &str, nodes: &[&str]) -> Subgraph {
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
            title: None,
            statements,
        }
    }

    #[test]
    fn test_x_crossing_elimination() {
        // Layer 0: [P0, P1]
        // Layer 1: [C0, C1]
        // Edges: P0 -> C1, P1 -> C0 (Crosses!)
        let mut layers = vec![
            vec!["P0".to_string(), "P1".to_string()],
            vec!["C0".to_string(), "C1".to_string()],
        ];
        let edges = vec![
            ("P0".to_string(), "C1".to_string()),
            ("P1".to_string(), "C0".to_string()),
        ];
        let back_edges = HashSet::new();
        let subgraphs = vec![];

        let initial_crossings =
            BarycenterCrossingMinimizer::count_crossings(&layers, &edges, &back_edges);
        assert_eq!(
            initial_crossings, 1,
            "Initial configuration must have 1 crossing"
        );

        BarycenterCrossingMinimizer::minimize_crossings(
            &mut layers,
            &edges,
            &back_edges,
            &subgraphs,
        );

        let final_crossings =
            BarycenterCrossingMinimizer::count_crossings(&layers, &edges, &back_edges);
        assert_eq!(
            final_crossings, 0,
            "Barycenter ordering must eliminate crossing"
        );
        assert_eq!(layers[1], vec!["C1".to_string(), "C0".to_string()]);
    }

    #[test]
    fn test_subgraph_containment_preserved() {
        // Layer 1 contains nodes [A1, B1, A2] where A1, A2 belong to SG0, B1 belongs to None
        let mut layers = vec![
            vec!["P0".to_string(), "P1".to_string(), "P2".to_string()],
            vec!["A1".to_string(), "B1".to_string(), "A2".to_string()],
        ];
        // P0 -> A1, P1 -> B1, P2 -> A2
        let edges = vec![
            ("P0".to_string(), "A1".to_string()),
            ("P1".to_string(), "B1".to_string()),
            ("P2".to_string(), "A2".to_string()),
        ];
        let back_edges = HashSet::new();
        let sg = make_test_subgraph("SG0", &["A1", "A2"]);

        BarycenterCrossingMinimizer::minimize_crossings(&mut layers, &edges, &back_edges, &[sg]);

        // A1 and A2 must remain contiguous in layers[1]
        let pos_a1 = layers[1].iter().position(|x| x == "A1").unwrap();
        let pos_a2 = layers[1].iter().position(|x| x == "A2").unwrap();
        assert_eq!(
            (pos_a1 as isize - pos_a2 as isize).abs(),
            1,
            "Subgraph nodes must be contiguous"
        );
    }

    #[test]
    fn test_subgraph_contiguity_preserved_under_interleaved_barycenters() {
        // SG1: A1, A2; SG2: B1, B2
        let mut node_to_sg = HashMap::new();
        node_to_sg.insert("A1".to_string(), 0);
        node_to_sg.insert("A2".to_string(), 0);
        node_to_sg.insert("B1".to_string(), 1);
        node_to_sg.insert("B2".to_string(), 1);

        // Interleaved barycenters: A1=0.0, B1=1.0, A2=2.0, B2=3.0
        // Avg SG1 = 1.0, Avg SG2 = 2.0
        let mut barycenters = HashMap::new();
        barycenters.insert("A1".to_string(), 0.0);
        barycenters.insert("B1".to_string(), 1.0);
        barycenters.insert("A2".to_string(), 2.0);
        barycenters.insert("B2".to_string(), 3.0);

        let mut layer = vec![
            "A1".to_string(),
            "B1".to_string(),
            "A2".to_string(),
            "B2".to_string(),
        ];

        BarycenterCrossingMinimizer::sort_layer_subgraph_preserving(
            &mut layer,
            &barycenters,
            &node_to_sg,
        );

        // Must be [A1, A2, B1, B2] - SG1 contiguous, SG2 contiguous, SG1 precedes SG2
        assert_eq!(layer, vec!["A1", "A2", "B1", "B2"]);
    }

    #[test]
    fn test_top_level_node_not_interleaved_in_subgraph() {
        let mut node_to_sg = HashMap::new();
        node_to_sg.insert("A1".to_string(), 0);
        node_to_sg.insert("A2".to_string(), 0);

        // Barycenters: A1=0.0, C=1.0, A2=2.0
        let mut barycenters = HashMap::new();
        barycenters.insert("A1".to_string(), 0.0);
        barycenters.insert("C".to_string(), 1.0);
        barycenters.insert("A2".to_string(), 2.0);

        let mut layer = vec!["A1".to_string(), "C".to_string(), "A2".to_string()];

        BarycenterCrossingMinimizer::sort_layer_subgraph_preserving(
            &mut layer,
            &barycenters,
            &node_to_sg,
        );

        // Subgraph SG1 nodes [A1, A2] must be strictly adjacent
        let pos_a1 = layer.iter().position(|x| x == "A1").unwrap();
        let pos_a2 = layer.iter().position(|x| x == "A2").unwrap();
        assert_eq!(
            (pos_a1 as isize - pos_a2 as isize).abs(),
            1,
            "A1 and A2 must be contiguous"
        );
    }

    #[test]
    fn test_disconnected_components_stability() {
        let mut layers = vec![
            vec!["A".to_string(), "C".to_string()],
            vec!["B".to_string(), "D".to_string()],
        ];
        let edges = vec![
            ("A".to_string(), "B".to_string()),
            ("C".to_string(), "D".to_string()),
        ];
        let back_edges = HashSet::new();

        BarycenterCrossingMinimizer::minimize_crossings(&mut layers, &edges, &back_edges, &[]);

        assert_eq!(layers[0], vec!["A".to_string(), "C".to_string()]);
        assert_eq!(layers[1], vec!["B".to_string(), "D".to_string()]);
    }

    #[test]
    fn test_cycle_back_edge_excluded() {
        // Back edge C1 -> P0 must not invert upward sweep
        let mut layers = vec![
            vec!["P0".to_string(), "P1".to_string()],
            vec!["C0".to_string(), "C1".to_string()],
        ];
        let edges = vec![
            ("P0".to_string(), "C0".to_string()),
            ("P1".to_string(), "C1".to_string()),
            ("C1".to_string(), "P0".to_string()), // Cycle edge
        ];
        let mut back_edges = HashSet::new();
        back_edges.insert(("C1".to_string(), "P0".to_string()));

        BarycenterCrossingMinimizer::minimize_crossings(&mut layers, &edges, &back_edges, &[]);

        assert_eq!(layers[0], vec!["P0".to_string(), "P1".to_string()]);
        assert_eq!(layers[1], vec!["C0".to_string(), "C1".to_string()]);
    }

    #[test]
    fn test_empty_and_single_layer_safety() {
        let mut empty_layers: Vec<Vec<String>> = vec![];
        BarycenterCrossingMinimizer::minimize_crossings(
            &mut empty_layers,
            &[],
            &HashSet::new(),
            &[],
        );
        assert!(empty_layers.is_empty());

        let mut single_layer = vec![vec!["A".to_string()]];
        BarycenterCrossingMinimizer::minimize_crossings(
            &mut single_layer,
            &[],
            &HashSet::new(),
            &[],
        );
        assert_eq!(single_layer, vec![vec!["A".to_string()]]);
    }
}

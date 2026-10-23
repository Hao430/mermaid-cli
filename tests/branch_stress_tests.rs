//! Adversarial Stress Test Suite for `src/renderer/layout/branch.rs`.
//!
//! Validates:
//! 1. Deep branching chains (50+, 100+, 500+ nodes)
//! 2. Wide multi-way forks (k = 5, 8, 10, 20, 50, 100)
//! 3. Nested multi-level forks (2-level, 3-level, 5-level, symmetric nested)
//! 4. Heavily unbalanced branch lengths (1 vs 25 nodes, 1 vs 50 nodes, direct bypass vs 25 nodes)
//! 5. Multiple joins converging from unbalanced depths (3-way unbalanced, cascading joins)
//! 6. Adversarial edge cases: empty, single node, pure cycles, self-loops, chained diamonds
//!
//! Invariant Verification:
//! - No stack overflow or panic across all stress scenarios
//! - Invariant 1: Symmetric slot allocation satisfies sum(slot) == 0.0
//! - Invariant 2: Virtual reservations span ALL intermediate missing layers across depth span D_max

use mermaid_cli::renderer::layout::branch::BranchAnalyzer;

/// Helper: convert string slice to owned String Vec
fn to_s(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

/// Helper: convert edge tuple slice to owned (String, String) Vec
fn to_edges(edges: &[(&str, &str)]) -> Vec<(String, String)> {
    edges
        .iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect()
}

/// Helper: convert layer slices to Vec<Vec<String>>
fn to_layers(layers: &[&[&str]]) -> Vec<Vec<String>> {
    layers.iter().map(|l| to_s(l)).collect()
}

// ============================================================================
// 1. Deep Branching Chains (50+, 100+, 500+ nodes)
// ============================================================================

#[test]
fn test_stress_deep_branching_chain_50_nodes() {
    // Fork F -> Branch A (50 nodes: A1..A50) -> Join J
    //        -> Branch B (50 nodes: B1..B50) -> Join J
    let n = 50;
    let mut node_ids = vec!["F".to_string(), "J".to_string()];
    let mut edges = Vec::new();
    let mut layer_vecs: Vec<Vec<String>> = Vec::with_capacity(n + 2);

    layer_vecs.push(vec!["F".to_string()]);

    edges.push(("F".to_string(), "A1".to_string()));
    edges.push(("F".to_string(), "B1".to_string()));

    for i in 1..=n {
        let a = format!("A{}", i);
        let b = format!("B{}", i);
        node_ids.push(a.clone());
        node_ids.push(b.clone());
        layer_vecs.push(vec![a.clone(), b.clone()]);

        if i < n {
            edges.push((a, format!("A{}", i + 1)));
            edges.push((b, format!("B{}", i + 1)));
        } else {
            edges.push((a, "J".to_string()));
            edges.push((b, "J".to_string()));
        }
    }

    layer_vecs.push(vec!["J".to_string()]);

    let analyzer = BranchAnalyzer::analyze(&node_ids, &edges, &layer_vecs, &[]);

    // 1. No panic & correct classification
    assert!(analyzer.is_fork("F"));
    assert!(analyzer.is_join("J"));
    assert_eq!(analyzer.lanes.len(), 2);

    // 2. Invariant: sum(slot) == 0.0 for fork F
    let slot_a = analyzer.slot_for_node("A1");
    let slot_b = analyzer.slot_for_node("B1");
    assert_eq!(slot_a, -0.5);
    assert_eq!(slot_b, 0.5);
    assert!((slot_a + slot_b).abs() < 1e-6);

    // 3. Monotonic slot propagation across all 50 levels
    for i in 1..=n {
        let a = format!("A{}", i);
        let b = format!("B{}", i);
        assert_eq!(analyzer.slot_for_node(&a), -0.5, "Node {} slot mismatch", a);
        assert_eq!(analyzer.slot_for_node(&b), 0.5, "Node {} slot mismatch", b);
    }

    // 4. Join node converges to 0.0
    assert_eq!(analyzer.slot_for_node("J"), 0.0);

    // 5. Balanced branches occupy all layers 1..50, so 0 virtual reservations
    assert_eq!(analyzer.reservations.len(), 0);

    // 6. Depth span check
    let lane_a = analyzer.lane_for_node("A1").unwrap();
    let lane_b = analyzer.lane_for_node("B1").unwrap();
    assert_eq!(lane_a.depth_span, (1, 50));
    assert_eq!(lane_b.depth_span, (1, 50));
    assert_eq!(lane_a.nodes.len(), 50);
    assert_eq!(lane_b.nodes.len(), 50);
}

#[test]
fn test_stress_deep_branching_chain_100_nodes() {
    let n = 100;
    let mut node_ids = vec!["F".to_string(), "J".to_string()];
    let mut edges = Vec::new();

    edges.push(("F".to_string(), "A1".to_string()));
    edges.push(("F".to_string(), "B1".to_string()));

    for i in 1..=n {
        let a = format!("A{}", i);
        let b = format!("B{}", i);
        node_ids.push(a.clone());
        node_ids.push(b.clone());

        if i < n {
            edges.push((a, format!("A{}", i + 1)));
            edges.push((b, format!("B{}", i + 1)));
        } else {
            edges.push((a, "J".to_string()));
            edges.push((b, "J".to_string()));
        }
    }

    // Pass empty layers to stress Kahn's BFS topological fallback on 100-node depth
    let analyzer = BranchAnalyzer::analyze(&node_ids, &edges, &[], &[]);

    assert!(analyzer.is_fork("F"));
    assert!(analyzer.is_join("J"));
    assert_eq!(analyzer.slot_for_node("F"), 0.0);
    assert_eq!(analyzer.slot_for_node("A100"), -0.5);
    assert_eq!(analyzer.slot_for_node("B100"), 0.5);
    assert_eq!(analyzer.slot_for_node("J"), 0.0);
    assert_eq!(analyzer.reservations.len(), 0);
}

#[test]
fn test_stress_linear_chain_recursion_depth_500() {
    // Stress test DFS recursion limit: linear chain of 500 nodes
    let n = 500;
    let mut node_ids = Vec::with_capacity(n);
    let mut edges = Vec::with_capacity(n - 1);

    for i in 1..=n {
        node_ids.push(format!("N{}", i));
        if i < n {
            edges.push((format!("N{}", i), format!("N{}", i + 1)));
        }
    }

    let analyzer = BranchAnalyzer::analyze(&node_ids, &edges, &[], &[]);

    assert_eq!(analyzer.fork_nodes.len(), 0);
    assert_eq!(analyzer.join_nodes.len(), 0);
    assert_eq!(analyzer.lanes.len(), 0);
    assert_eq!(analyzer.reservations.len(), 0);
    assert_eq!(analyzer.slot_for_node("N1"), 0.0);
    assert_eq!(analyzer.slot_for_node("N500"), 0.0);
}

// ============================================================================
// 2. Wide Multi-Way Forks (k = 5, 8, 10, 20, 50, 100)
// ============================================================================

#[test]
fn test_stress_wide_forks_k_values() {
    let k_values = [5, 8, 10, 20, 50, 100];

    for &k in &k_values {
        // Direct calculation verification
        let slots: Vec<f32> = (0..k)
            .map(|i| BranchAnalyzer::calculate_lane_slot(i, k))
            .collect();

        // Invariant 1: sum(slot) == 0.0
        let sum: f32 = slots.iter().sum();
        assert!(
            sum.abs() < 1e-4,
            "Sum for k={} was {} (expected 0.0)",
            k,
            sum
        );

        // Symmetry: slot(i) == -slot(k - 1 - i)
        for i in 0..k {
            let opposite = slots[k - 1 - i];
            assert!(
                (slots[i] + opposite).abs() < 1e-5,
                "Symmetry violation at i={}, k={}: {} vs {}",
                i,
                k,
                slots[i],
                opposite
            );
        }

        // Unit spacing between consecutive lanes
        for i in 0..(k - 1) {
            let diff = slots[i + 1] - slots[i];
            assert!(
                (diff - 1.0).abs() < 1e-5,
                "Spacing violation between i={} and i+1={} for k={}: diff={}",
                i,
                i + 1,
                k,
                diff
            );
        }
    }
}

#[test]
fn test_stress_wide_fork_k8_graph_topology() {
    let k = 8;
    let mut node_ids = vec!["Fork".to_string(), "Join".to_string()];
    let mut edges = Vec::new();
    let mut layer1 = Vec::new();

    for i in 0..k {
        let child = format!("Branch{}", i);
        node_ids.push(child.clone());
        edges.push(("Fork".to_string(), child.clone()));
        edges.push((child.clone(), "Join".to_string()));
        layer1.push(child);
    }

    let layers = vec![vec!["Fork".to_string()], layer1, vec!["Join".to_string()]];
    let analyzer = BranchAnalyzer::analyze(&node_ids, &edges, &layers, &[]);

    assert!(analyzer.is_fork("Fork"));
    assert!(analyzer.is_join("Join"));
    assert_eq!(analyzer.lanes.len(), k);

    // Verify all 8 lane slots and sum invariant
    let mut sum = 0.0;
    let expected = [-3.5, -2.5, -1.5, -0.5, 0.5, 1.5, 2.5, 3.5];
    for (i, &exp) in expected.iter().enumerate().take(k) {
        let child = format!("Branch{}", i);
        let slot = analyzer.slot_for_node(&child);
        assert!(
            (slot - exp).abs() < 1e-5,
            "Branch{} slot {} != expected {}",
            i,
            slot,
            exp
        );
        sum += slot;
    }
    assert!(sum.abs() < 1e-5, "Sum of slots for k=8 must be 0.0");
    assert_eq!(analyzer.slot_for_node("Join"), 0.0);
}

#[test]
fn test_stress_wide_fork_k10_graph_topology() {
    let k = 10;
    let mut node_ids = vec!["Fork".to_string(), "Join".to_string()];
    let mut edges = Vec::new();

    for i in 0..k {
        let child = format!("Branch{}", i);
        node_ids.push(child.clone());
        edges.push(("Fork".to_string(), child.clone()));
        edges.push((child.clone(), "Join".to_string()));
    }

    let analyzer = BranchAnalyzer::analyze(&node_ids, &edges, &[], &[]);

    assert!(analyzer.is_fork("Fork"));
    assert!(analyzer.is_join("Join"));
    assert_eq!(analyzer.lanes.len(), k);

    let mut sum = 0.0;
    let expected = [-4.5, -3.5, -2.5, -1.5, -0.5, 0.5, 1.5, 2.5, 3.5, 4.5];
    for (i, &exp) in expected.iter().enumerate().take(k) {
        let child = format!("Branch{}", i);
        let slot = analyzer.slot_for_node(&child);
        assert!(
            (slot - exp).abs() < 1e-5,
            "Branch{} slot {} != expected {}",
            i,
            slot,
            exp
        );
        sum += slot;
    }
    assert!(sum.abs() < 1e-5, "Sum of slots for k=10 must be 0.0");
    assert_eq!(analyzer.slot_for_node("Join"), 0.0);
}

// ============================================================================
// 3. Nested Multi-Level Forks
// ============================================================================

#[test]
fn test_stress_nested_forks_multi_level() {
    // Level 0: F0 -> L0, R0
    // Level 1: R0 (fork) -> R1a, R1b
    // Level 2: R1b (fork) -> R2a, R2b
    // All branches eventually converge to J
    let nodes = to_s(&["F0", "L0", "R0", "R1a", "R1b", "R2a", "R2b", "J"]);
    let edges = to_edges(&[
        ("F0", "L0"),
        ("F0", "R0"),
        ("L0", "J"),
        ("R0", "R1a"),
        ("R0", "R1b"),
        ("R1a", "J"),
        ("R1b", "R2a"),
        ("R1b", "R2b"),
        ("R2a", "J"),
        ("R2b", "J"),
    ]);
    let layers = to_layers(&[
        &["F0"],
        &["L0", "R0"],
        &["R1a", "R1b"],
        &["R2a", "R2b"],
        &["J"],
    ]);

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);

    // Forks & Joins
    assert!(analyzer.is_fork("F0"));
    assert!(analyzer.is_fork("R0"));
    assert!(analyzer.is_fork("R1b"));
    assert!(analyzer.is_join("J"));

    // Level 0 slots
    assert_eq!(analyzer.slot_for_node("L0"), -0.5);
    assert_eq!(analyzer.slot_for_node("R0"), 0.5);

    // Level 1: R0 has parent_slot = 0.5 (positive side)
    // OutwardSpill: sub_index 0 stays at parent_slot (+0.5), sub_index 1 spills outward (+1.5)
    assert_eq!(analyzer.slot_for_node("R1a"), 0.5);
    assert_eq!(analyzer.slot_for_node("R1b"), 1.5);

    // Level 2: R1b has parent_slot = 1.5 (positive side)
    // OutwardSpill: sub_index 0 stays at 1.5, sub_index 1 spills outward (+2.5)
    assert_eq!(analyzer.slot_for_node("R2a"), 1.5);
    assert_eq!(analyzer.slot_for_node("R2b"), 2.5);

    // Join converges
    assert_eq!(analyzer.slot_for_node("J"), 0.0);
}

#[test]
fn test_stress_symmetric_nested_forks_both_sides() {
    // F0 -> L0 (-0.5), R0 (+0.5)
    // L0 -> L1a (-0.5), L1b (-1.5)
    // R0 -> R1a (+0.5), R1b (+1.5)
    // All converge to J
    let nodes = to_s(&["F0", "L0", "R0", "L1a", "L1b", "R1a", "R1b", "J"]);
    let edges = to_edges(&[
        ("F0", "L0"),
        ("F0", "R0"),
        ("L0", "L1a"),
        ("L0", "L1b"),
        ("R0", "R1a"),
        ("R0", "R1b"),
        ("L1a", "J"),
        ("L1b", "J"),
        ("R1a", "J"),
        ("R1b", "J"),
    ]);
    let layers = to_layers(&[
        &["F0"],
        &["L0", "R0"],
        &["L1a", "L1b", "R1a", "R1b"],
        &["J"],
    ]);

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);

    assert_eq!(analyzer.slot_for_node("L1a"), -0.5);
    assert_eq!(analyzer.slot_for_node("L1b"), -1.5);
    assert_eq!(analyzer.slot_for_node("R1a"), 0.5);
    assert_eq!(analyzer.slot_for_node("R1b"), 1.5);

    // Reflectional symmetry: sum of all 4 sub-branches is exactly 0.0
    let total_sub_slot = analyzer.slot_for_node("L1a")
        + analyzer.slot_for_node("L1b")
        + analyzer.slot_for_node("R1a")
        + analyzer.slot_for_node("R1b");
    assert!((total_sub_slot).abs() < 1e-6);
    assert_eq!(analyzer.slot_for_node("J"), 0.0);
}

// ============================================================================
// 4. Heavily Unbalanced Branch Lengths (1 vs 25 nodes, 1 vs 50 nodes)
// ============================================================================

#[test]
fn test_stress_heavily_unbalanced_1_vs_25_nodes() {
    // Fork F -> Branch A (1 node: A1) -> Join J
    //        -> Branch B (25 nodes: B1..B25) -> Join J
    let n = 25;
    let mut node_ids = vec!["F".to_string(), "A1".to_string(), "J".to_string()];
    let mut edges = Vec::new();
    let mut layers: Vec<Vec<String>> = Vec::with_capacity(n + 2);

    // Layer 0: Fork
    layers.push(vec!["F".to_string()]);

    // Layer 1: A1, B1
    node_ids.push("B1".to_string());
    edges.push(("F".to_string(), "A1".to_string()));
    edges.push(("F".to_string(), "B1".to_string()));
    edges.push(("A1".to_string(), "J".to_string()));
    layers.push(vec!["A1".to_string(), "B1".to_string()]);

    // Layers 2..=25: B2..B25
    for i in 2..=n {
        let b = format!("B{}", i);
        node_ids.push(b.clone());
        edges.push((format!("B{}", i - 1), b.clone()));
        layers.push(vec![b]);
    }
    edges.push((format!("B{}", n), "J".to_string()));

    // Layer 26: Join J
    layers.push(vec!["J".to_string()]);

    let analyzer = BranchAnalyzer::analyze(&node_ids, &edges, &layers, &[]);

    // 1. Structural checks
    assert!(analyzer.is_fork("F"));
    assert!(analyzer.is_join("J"));
    assert_eq!(analyzer.slot_for_node("A1"), -0.5);
    assert_eq!(analyzer.slot_for_node("B1"), 0.5);
    assert_eq!(analyzer.slot_for_node("J"), 0.0);

    // 2. Invariant 1: sum(slot) == 0.0
    assert_eq!(
        analyzer.slot_for_node("A1") + analyzer.slot_for_node("B1"),
        0.0
    );

    // 3. Invariant 2: Virtual reservations span ALL intermediate missing layers
    // D_max = 25. A1 is at layer 1. Intermediate missing layers for Branch A: 2..=25 (24 layers!).
    // Branch B occupies layers 1..=25, so no missing layers for Branch B.
    // Layer 26 is Join J, which must NOT have reservations.
    for l in 2..=n {
        assert!(
            analyzer.is_slot_reserved(l, -0.5),
            "Missing virtual reservation for Branch A (slot -0.5) at layer {}",
            l
        );
        assert!(
            !analyzer.is_slot_reserved(l, 0.5),
            "Spurious virtual reservation for Branch B (slot 0.5) at layer {}",
            l
        );
    }

    // Exactly 24 reservations generated, all on slot -0.5
    assert_eq!(analyzer.reservations.len(), n - 1);
    for res in &analyzer.reservations {
        assert_eq!(res.slot_offset, -0.5);
        assert!(res.layer >= 2 && res.layer <= n);
    }

    // Join layer (26) must have zero reservations
    assert_eq!(analyzer.reservations_at_layer(n + 1).len(), 0);
}

#[test]
fn test_stress_heavily_unbalanced_1_vs_50_nodes() {
    let n = 50;
    let mut node_ids = vec!["F".to_string(), "A1".to_string(), "J".to_string()];
    let mut edges = Vec::new();
    let mut layers: Vec<Vec<String>> = Vec::with_capacity(n + 2);

    layers.push(vec!["F".to_string()]);
    node_ids.push("B1".to_string());
    edges.push(("F".to_string(), "A1".to_string()));
    edges.push(("F".to_string(), "B1".to_string()));
    edges.push(("A1".to_string(), "J".to_string()));
    layers.push(vec!["A1".to_string(), "B1".to_string()]);

    for i in 2..=n {
        let b = format!("B{}", i);
        node_ids.push(b.clone());
        edges.push((format!("B{}", i - 1), b.clone()));
        layers.push(vec![b]);
    }
    edges.push((format!("B{}", n), "J".to_string()));
    layers.push(vec!["J".to_string()]);

    let analyzer = BranchAnalyzer::analyze(&node_ids, &edges, &layers, &[]);

    // Exactly 49 virtual reservations spanning layers 2..=50
    assert_eq!(analyzer.reservations.len(), 49);
    for l in 2..=50 {
        assert!(analyzer.is_slot_reserved(l, -0.5));
    }
}

#[test]
fn test_stress_direct_bypass_vs_25_nodes() {
    // Fork F -> Join J (direct bypass: 0 intermediate nodes)
    //        -> Branch B (25 nodes: B1..B25) -> Join J
    let n = 25;
    let mut node_ids = vec!["F".to_string(), "J".to_string()];
    let mut edges = Vec::new();
    let mut layers: Vec<Vec<String>> = Vec::with_capacity(n + 2);

    layers.push(vec!["F".to_string()]);
    edges.push(("F".to_string(), "J".to_string())); // Direct bypass

    for i in 1..=n {
        let b = format!("B{}", i);
        node_ids.push(b.clone());
        if i == 1 {
            edges.push(("F".to_string(), b.clone()));
        } else {
            edges.push((format!("B{}", i - 1), b.clone()));
        }
        layers.push(vec![b]);
    }
    edges.push((format!("B{}", n), "J".to_string()));
    layers.push(vec!["J".to_string()]);

    let analyzer = BranchAnalyzer::analyze(&node_ids, &edges, &layers, &[]);

    assert!(analyzer.is_fork("F"));
    assert!(analyzer.is_join("J"));

    // Direct bypass branch is missing across all intermediate layers 1..=25 (25 layers!)
    assert_eq!(analyzer.reservations.len(), 25);
    for l in 1..=25 {
        assert!(
            analyzer.is_slot_reserved(l, -0.5),
            "Direct bypass corridor missing reservation at layer {}",
            l
        );
    }
}

// ============================================================================
// 5. Multiple Joins Converging from Unbalanced Depths
// ============================================================================

#[test]
fn test_stress_three_way_unbalanced_depths() {
    // Fork F -> Branch 1 (length 1: A1) -> J
    //        -> Branch 2 (length 5: B1..B5) -> J
    //        -> Branch 3 (length 25: C1..C25) -> J
    let mut node_ids = vec!["F".to_string(), "J".to_string()];
    let mut edges = Vec::new();
    let mut layers: Vec<Vec<String>> = Vec::with_capacity(27);

    // Layer 0: Fork
    layers.push(vec!["F".to_string()]);

    // Layer 1: A1, B1, C1
    node_ids.push("A1".to_string());
    node_ids.push("B1".to_string());
    node_ids.push("C1".to_string());
    edges.push(("F".to_string(), "A1".to_string()));
    edges.push(("F".to_string(), "B1".to_string()));
    edges.push(("F".to_string(), "C1".to_string()));
    edges.push(("A1".to_string(), "J".to_string()));
    layers.push(vec!["A1".to_string(), "B1".to_string(), "C1".to_string()]);

    // Layers 2..5: B2..B5 and C2..C5
    for i in 2..=5 {
        let b = format!("B{}", i);
        let c = format!("C{}", i);
        node_ids.push(b.clone());
        node_ids.push(c.clone());
        edges.push((format!("B{}", i - 1), b.clone()));
        edges.push((format!("C{}", i - 1), c.clone()));
        layers.push(vec![b, c]);
    }
    edges.push(("B5".to_string(), "J".to_string()));

    // Layers 6..25: C6..C25
    for i in 6..=25 {
        let c = format!("C{}", i);
        node_ids.push(c.clone());
        edges.push((format!("C{}", i - 1), c.clone()));
        layers.push(vec![c]);
    }
    edges.push(("C25".to_string(), "J".to_string()));

    // Layer 26: J
    layers.push(vec!["J".to_string()]);

    let analyzer = BranchAnalyzer::analyze(&node_ids, &edges, &layers, &[]);

    assert!(analyzer.is_fork("F"));
    assert!(analyzer.is_join("J"));

    // Invariant 1: 3-way fork slots are -1.0, 0.0, +1.0. Sum == 0.0
    let slot_a = analyzer.slot_for_node("A1");
    let slot_b = analyzer.slot_for_node("B1");
    let slot_c = analyzer.slot_for_node("C1");
    assert_eq!(slot_a, -1.0);
    assert_eq!(slot_b, 0.0);
    assert_eq!(slot_c, 1.0);
    assert!((slot_a + slot_b + slot_c).abs() < 1e-6);

    // Invariant 2: Virtual reservations
    // D_max = 25
    // Branch 1 (slot -1.0): occupies layer 1, missing layers 2..=25 (24 reservations)
    for l in 2..=25 {
        assert!(
            analyzer.is_slot_reserved(l, -1.0),
            "Branch 1 missing reservation at layer {}",
            l
        );
    }

    // Branch 2 (slot 0.0): occupies layers 1..=5, missing layers 6..=25 (20 reservations)
    for l in 6..=25 {
        assert!(
            analyzer.is_slot_reserved(l, 0.0),
            "Branch 2 missing reservation at layer {}",
            l
        );
    }

    // Branch 3 (slot 1.0): occupies layers 1..=25, 0 reservations
    for l in 1..=25 {
        assert!(!analyzer.is_slot_reserved(l, 1.0));
    }

    // Total reservations: 24 + 20 = 44
    assert_eq!(analyzer.reservations.len(), 44);
    assert_eq!(analyzer.slot_for_node("J"), 0.0);
}

#[test]
fn test_stress_cascading_sequential_fork_join_diamonds() {
    // 10 sequential diamonds chained end-to-end:
    // D0 -> (L0, R0) -> J0 -> D1 -> (L1, R1) -> J1 -> ... -> J9
    let n = 10;
    let mut node_ids = Vec::new();
    let mut edges = Vec::new();
    let mut layers = Vec::new();

    for d in 0..n {
        let fork = format!("D{}", d);
        let left = format!("L{}", d);
        let right = format!("R{}", d);
        let join = format!("J{}", d);

        node_ids.push(fork.clone());
        node_ids.push(left.clone());
        node_ids.push(right.clone());
        node_ids.push(join.clone());

        if d > 0 {
            // Connect previous join to current fork
            edges.push((format!("J{}", d - 1), fork.clone()));
        }

        edges.push((fork.clone(), left.clone()));
        edges.push((fork.clone(), right.clone()));
        edges.push((left.clone(), join.clone()));
        edges.push((right.clone(), join.clone()));

        layers.push(vec![fork]);
        layers.push(vec![left, right]);
        layers.push(vec![join]);
    }

    let analyzer = BranchAnalyzer::analyze(&node_ids, &edges, &layers, &[]);

    assert_eq!(analyzer.fork_nodes.len(), n);
    assert_eq!(analyzer.join_nodes.len(), n);
    assert_eq!(analyzer.lanes.len(), 2 * n);

    for d in 0..n {
        let fork = format!("D{}", d);
        let left = format!("L{}", d);
        let right = format!("R{}", d);
        let join = format!("J{}", d);

        assert!(analyzer.is_fork(&fork));
        assert!(analyzer.is_join(&join));
        assert_eq!(analyzer.slot_for_node(&left), -0.5);
        assert_eq!(analyzer.slot_for_node(&right), 0.5);
        assert_eq!(analyzer.slot_for_node(&join), 0.0);

        // Invariant 1: sum(slot) == 0.0 for every fork diamond
        assert_eq!(
            analyzer.slot_for_node(&left) + analyzer.slot_for_node(&right),
            0.0
        );
    }
}

// ============================================================================
// 6. Adversarial Edge Cases
// ============================================================================

#[test]
fn test_stress_empty_graph() {
    let analyzer = BranchAnalyzer::analyze(&[], &[], &[], &[]);
    assert_eq!(analyzer.lanes.len(), 0);
    assert_eq!(analyzer.fork_nodes.len(), 0);
    assert_eq!(analyzer.join_nodes.len(), 0);
    assert_eq!(analyzer.reservations.len(), 0);
    assert_eq!(analyzer.slot_for_node("nonexistent"), 0.0);
}

#[test]
fn test_stress_single_node() {
    let nodes = to_s(&["Single"]);
    let analyzer = BranchAnalyzer::analyze(&nodes, &[], &[], &[]);
    assert_eq!(analyzer.slot_for_node("Single"), 0.0);
    assert!(!analyzer.is_fork("Single"));
    assert!(!analyzer.is_join("Single"));
}

#[test]
fn test_stress_disconnected_islands() {
    let nodes = to_s(&["A1", "A2", "B1", "B2", "C1"]);
    let edges = to_edges(&[("A1", "A2"), ("B1", "B2")]);
    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &[], &[]);
    assert_eq!(analyzer.lanes.len(), 0);
    assert_eq!(analyzer.slot_for_node("C1"), 0.0);
}

#[test]
fn test_stress_self_loops_and_multi_cycles() {
    // Self loop A -> A
    // Mutual cycle B <-> C
    // Triangle cycle D -> E -> F -> D
    let nodes = to_s(&["A", "B", "C", "D", "E", "F"]);
    let edges = to_edges(&[
        ("A", "A"),
        ("B", "C"),
        ("C", "B"),
        ("D", "E"),
        ("E", "F"),
        ("F", "D"),
    ]);
    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &[], &[]);
    // All cycle back-edges isolated, no panic
    assert!(analyzer.back_edges.len() >= 3);
}

#[test]
fn test_stress_join_is_also_fork() {
    // F1 -> A -> M; F1 -> B -> M (M is join of F1)
    // M -> X -> J2; M -> Y -> J2 (M is fork of F2)
    let nodes = to_s(&["F1", "A", "B", "M", "X", "Y", "J2"]);
    let edges = to_edges(&[
        ("F1", "A"),
        ("F1", "B"),
        ("A", "M"),
        ("B", "M"),
        ("M", "X"),
        ("M", "Y"),
        ("X", "J2"),
        ("Y", "J2"),
    ]);
    let layers = to_layers(&[&["F1"], &["A", "B"], &["M"], &["X", "Y"], &["J2"]]);

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);

    assert!(analyzer.is_fork("F1"));
    assert!(analyzer.is_join("M"));
    assert!(analyzer.is_fork("M"));
    assert!(analyzer.is_join("J2"));

    // Upstream fork
    assert_eq!(analyzer.slot_for_node("A"), -0.5);
    assert_eq!(analyzer.slot_for_node("B"), 0.5);

    // M converges to 0.0
    assert_eq!(analyzer.slot_for_node("M"), 0.0);

    // Downstream fork from M branches symmetrically from 0.0
    assert_eq!(analyzer.slot_for_node("X"), -0.5);
    assert_eq!(analyzer.slot_for_node("Y"), 0.5);

    // J2 converges to 0.0
    assert_eq!(analyzer.slot_for_node("J2"), 0.0);
}

#[test]
fn test_stress_unbalanced_branch_with_intermediate_layer_holes() {
    // Fork F (layer 0) -> Branch A (A1 at layer 1, A4 at layer 4; holes at layers 2, 3) -> Join J (layer 5)
    //                  -> Branch B (B1, B2, B3, B4 at layers 1, 2, 3, 4) -> Join J (layer 5)
    let nodes = to_s(&["F", "A1", "A4", "B1", "B2", "B3", "B4", "J"]);
    let edges = to_edges(&[
        ("F", "A1"),
        ("A1", "A4"),
        ("A4", "J"),
        ("F", "B1"),
        ("B1", "B2"),
        ("B2", "B3"),
        ("B3", "B4"),
        ("B4", "J"),
    ]);
    let layers = to_layers(&[
        &["F"],
        &["A1", "B1"],
        &["B2"],
        &["B3"],
        &["A4", "B4"],
        &["J"],
    ]);

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);

    assert!(analyzer.is_fork("F"));
    assert!(analyzer.is_join("J"));
    assert_eq!(analyzer.slot_for_node("A1"), -0.5);
    assert_eq!(analyzer.slot_for_node("A4"), -0.5);
    assert_eq!(analyzer.slot_for_node("J"), 0.0);

    // Branch A occupies layers 1 and 4. Missing intermediate layers are 2 and 3!
    assert!(
        analyzer.is_slot_reserved(2, -0.5),
        "Layer 2 missing reservation for Branch A"
    );
    assert!(
        analyzer.is_slot_reserved(3, -0.5),
        "Layer 3 missing reservation for Branch A"
    );
    assert!(
        !analyzer.is_slot_reserved(1, -0.5),
        "Layer 1 occupied by A1; should not be reserved"
    );
    assert!(
        !analyzer.is_slot_reserved(4, -0.5),
        "Layer 4 occupied by A4; should not be reserved"
    );
    assert_eq!(analyzer.reservations.len(), 2);
}

#[test]
fn test_stress_nested_fork_three_way_outward_spill() {
    // F0 -> L0 (-0.5), R0 (+0.5)
    // L0 is 3-way nested fork: sub0, sub1, sub2
    // R0 is 3-way nested fork: sub0, sub1, sub2
    let s_l0 = mermaid_cli::renderer::layout::branch::BranchAnalyzer::calculate_nested_slot(
        -0.5,
        0,
        3,
        mermaid_cli::renderer::layout::branch::NestedForkStrategy::OutwardSpill,
    );
    let s_l1 = mermaid_cli::renderer::layout::branch::BranchAnalyzer::calculate_nested_slot(
        -0.5,
        1,
        3,
        mermaid_cli::renderer::layout::branch::NestedForkStrategy::OutwardSpill,
    );
    let s_l2 = mermaid_cli::renderer::layout::branch::BranchAnalyzer::calculate_nested_slot(
        -0.5,
        2,
        3,
        mermaid_cli::renderer::layout::branch::NestedForkStrategy::OutwardSpill,
    );

    assert_eq!(s_l0, -0.5);
    assert_eq!(s_l1, -1.5);
    assert_eq!(s_l2, -2.5);

    let s_r0 = mermaid_cli::renderer::layout::branch::BranchAnalyzer::calculate_nested_slot(
        0.5,
        0,
        3,
        mermaid_cli::renderer::layout::branch::NestedForkStrategy::OutwardSpill,
    );
    let s_r1 = mermaid_cli::renderer::layout::branch::BranchAnalyzer::calculate_nested_slot(
        0.5,
        1,
        3,
        mermaid_cli::renderer::layout::branch::NestedForkStrategy::OutwardSpill,
    );
    let s_r2 = mermaid_cli::renderer::layout::branch::BranchAnalyzer::calculate_nested_slot(
        0.5,
        2,
        3,
        mermaid_cli::renderer::layout::branch::NestedForkStrategy::OutwardSpill,
    );

    assert_eq!(s_r0, 0.5);
    assert_eq!(s_r1, 1.5);
    assert_eq!(s_r2, 2.5);

    // Sum across symmetric nested forks
    assert_eq!(s_l0 + s_l1 + s_l2 + s_r0 + s_r1 + s_r2, 0.0);
}

#[test]
fn test_stress_overlapping_independent_forks() {
    // Fork F1 (layer 0) -> A1 (layer 1) -> J1 (layer 3)
    //                   -> B1 (layer 1) -> B2 (layer 2) -> J1 (layer 3)
    // Fork F2 (layer 1) -> C1 (layer 2) -> J2 (layer 4)
    //                   -> D1 (layer 2) -> D2 (layer 3) -> J2 (layer 4)
    let nodes = to_s(&["F1", "A1", "B1", "B2", "J1", "F2", "C1", "D1", "D2", "J2"]);
    let edges = to_edges(&[
        ("F1", "A1"),
        ("F1", "B1"),
        ("A1", "J1"),
        ("B1", "B2"),
        ("B2", "J1"),
        ("F2", "C1"),
        ("F2", "D1"),
        ("C1", "J2"),
        ("D1", "D2"),
        ("D2", "J2"),
    ]);
    let layers = to_layers(&[
        &["F1"],
        &["A1", "B1", "F2"],
        &["B2", "C1", "D1"],
        &["J1", "D2"],
        &["J2"],
    ]);

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);

    assert!(analyzer.is_fork("F1"));
    assert!(analyzer.is_fork("F2"));
    assert!(analyzer.is_join("J1"));
    assert!(analyzer.is_join("J2"));

    // F1 branches: A1 (-0.5), B1 (0.5), sum = 0.0
    assert_eq!(analyzer.slot_for_node("A1"), -0.5);
    assert_eq!(analyzer.slot_for_node("B1"), 0.5);
    assert_eq!(analyzer.slot_for_node("B2"), 0.5);
    assert_eq!(analyzer.slot_for_node("J1"), 0.0);

    // F2 branches: C1 (-0.5), D1 (0.5), sum = 0.0
    assert_eq!(analyzer.slot_for_node("C1"), -0.5);
    assert_eq!(analyzer.slot_for_node("D1"), 0.5);
    assert_eq!(analyzer.slot_for_node("D2"), 0.5);
    assert_eq!(analyzer.slot_for_node("J2"), 0.0);

    // Reservations:
    // F1 Branch A: missing at layer 2
    assert!(analyzer.is_slot_reserved(2, -0.5));
    // F2 Branch C: missing at layer 3
    assert!(analyzer.is_slot_reserved(3, -0.5));
}

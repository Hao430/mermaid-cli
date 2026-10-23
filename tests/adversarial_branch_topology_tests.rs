//! Adversarial stress test harness for `src/renderer/layout/branch.rs`.
//! Probes edge-case topologies:
//! 1. Disconnected graphs (multiple disjoint components)
//! 2. Graphs with 0 edges, isolated nodes
//! 3. Self-loops (A -> A) and multi-edges (A -> B repeated)
//! 4. Complex cyclic graphs (figure-8, nested feedback loops, cliques, cycles without roots)
//! 5. Direct bypass edges (fork connecting directly to downstream join across varying depth spans)
//! 6. Property-based stress testing verifying structural invariants

use mermaid_cli::renderer::layout::branch::ForwardGraph;
use mermaid_cli::renderer::layout::BranchAnalyzer;

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

// ============================================================================
// TOPOLOGY 1: DISCONNECTED GRAPHS (Multiple Disjoint Components)
// ============================================================================

#[test]
fn test_disconnected_multiple_independent_fork_join_components() {
    // Component 1: A1 -> B1 -> D1, A1 -> C1 -> D1 (Fork A1, Join D1)
    // Component 2: A2 -> B2 -> D2, A2 -> C2 -> D2 (Fork A2, Join D2)
    // Component 3: Linear chain L1 -> L2 -> L3
    // Component 4: Isolated node ISO
    let nodes = to_s(&[
        "A1", "B1", "C1", "D1", "A2", "B2", "C2", "D2", "L1", "L2", "L3", "ISO",
    ]);
    let edges = to_edges(&[
        ("A1", "B1"),
        ("A1", "C1"),
        ("B1", "D1"),
        ("C1", "D1"),
        ("A2", "B2"),
        ("A2", "C2"),
        ("B2", "D2"),
        ("C2", "D2"),
        ("L1", "L2"),
        ("L2", "L3"),
    ]);
    let layers = to_layers(&[
        &["A1", "A2", "L1", "ISO"],
        &["B1", "C1", "B2", "C2", "L2"],
        &["D1", "D2", "L3"],
    ]);

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);

    // Component 1 verification
    assert!(analyzer.is_fork("A1"));
    assert!(analyzer.is_join("D1"));
    assert_eq!(analyzer.slot_for_node("A1"), 0.0);
    assert_eq!(analyzer.slot_for_node("B1"), -0.5);
    assert_eq!(analyzer.slot_for_node("C1"), 0.5);
    assert_eq!(analyzer.slot_for_node("D1"), 0.0);

    // Component 2 verification
    assert!(analyzer.is_fork("A2"));
    assert!(analyzer.is_join("D2"));
    assert_eq!(analyzer.slot_for_node("A2"), 0.0);
    assert_eq!(analyzer.slot_for_node("B2"), -0.5);
    assert_eq!(analyzer.slot_for_node("C2"), 0.5);
    assert_eq!(analyzer.slot_for_node("D2"), 0.0);

    // Component 3 & 4 verification
    assert_eq!(analyzer.slot_for_node("L1"), 0.0);
    assert_eq!(analyzer.slot_for_node("L2"), 0.0);
    assert_eq!(analyzer.slot_for_node("L3"), 0.0);
    assert_eq!(analyzer.slot_for_node("ISO"), 0.0);
    assert!(!analyzer.is_fork("ISO"));
    assert!(!analyzer.is_join("ISO"));

    // Verify 4 distinct branch lanes (2 for A1, 2 for A2)
    assert_eq!(analyzer.lanes.len(), 4);
    assert_eq!(analyzer.lane_for_node("B1").unwrap().fork_node, "A1");
    assert_eq!(analyzer.lane_for_node("C1").unwrap().fork_node, "A1");
    assert_eq!(analyzer.lane_for_node("B2").unwrap().fork_node, "A2");
    assert_eq!(analyzer.lane_for_node("C2").unwrap().fork_node, "A2");
}

#[test]
fn test_disconnected_components_with_topological_depth_fallback() {
    // 3 disconnected components with empty layers:
    // Comp 1: X1 -> Y1, X1 -> Z1 (Fork)
    // Comp 2: M1 -> M2 -> M3 -> M4 (Long chain)
    // Comp 3: P1 -> Q1 -> R1, P1 -> Q2 -> R1 (Fork & Join)
    let nodes = to_s(&[
        "X1", "Y1", "Z1", "M1", "M2", "M3", "M4", "P1", "Q1", "Q2", "R1",
    ]);
    let edges = to_edges(&[
        ("X1", "Y1"),
        ("X1", "Z1"),
        ("M1", "M2"),
        ("M2", "M3"),
        ("M3", "M4"),
        ("P1", "Q1"),
        ("P1", "Q2"),
        ("Q1", "R1"),
        ("Q2", "R1"),
    ]);

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &[], &[]);

    assert!(analyzer.is_fork("X1"));
    assert_eq!(analyzer.slot_for_node("Y1"), -0.5);
    assert_eq!(analyzer.slot_for_node("Z1"), 0.5);

    assert!(analyzer.is_fork("P1"));
    assert!(analyzer.is_join("R1"));
    assert_eq!(analyzer.slot_for_node("Q1"), -0.5);
    assert_eq!(analyzer.slot_for_node("Q2"), 0.5);
    assert_eq!(analyzer.slot_for_node("R1"), 0.0);

    for m in &["M1", "M2", "M3", "M4"] {
        assert_eq!(analyzer.slot_for_node(m), 0.0);
    }
}

#[test]
fn test_disconnected_50_independent_components() {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for i in 0..50 {
        let root = format!("R{}", i);
        let left = format!("L{}", i);
        let right = format!("T{}", i);
        edges.push((root.clone(), left.clone()));
        edges.push((root.clone(), right.clone()));
        nodes.push(root);
        nodes.push(left);
        nodes.push(right);
    }

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &[], &[]);

    assert_eq!(analyzer.fork_nodes.len(), 50);
    assert_eq!(analyzer.lanes.len(), 100);

    for i in 0..50 {
        let left = format!("L{}", i);
        let right = format!("T{}", i);
        assert_eq!(analyzer.slot_for_node(&left), -0.5);
        assert_eq!(analyzer.slot_for_node(&right), 0.5);
    }
}

// ============================================================================
// TOPOLOGY 2: ZERO EDGES AND ISOLATED NODES
// ============================================================================

#[test]
fn test_completely_empty_graph() {
    let nodes: Vec<String> = Vec::new();
    let edges: Vec<(String, String)> = Vec::new();
    let layers: Vec<Vec<String>> = Vec::new();

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);

    assert_eq!(analyzer.lanes.len(), 0);
    assert_eq!(analyzer.fork_nodes.len(), 0);
    assert_eq!(analyzer.join_nodes.len(), 0);
    assert_eq!(analyzer.reservations.len(), 0);
    assert_eq!(analyzer.slot_for_node("NonExistent"), 0.0);
    assert!(!analyzer.is_fork("NonExistent"));
    assert!(!analyzer.is_join("NonExistent"));
}

#[test]
fn test_single_isolated_node() {
    let nodes = to_s(&["SingleNode"]);
    let edges: Vec<(String, String)> = Vec::new();

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &[], &[]);

    assert_eq!(analyzer.slot_for_node("SingleNode"), 0.0);
    assert!(!analyzer.is_fork("SingleNode"));
    assert!(!analyzer.is_join("SingleNode"));
    assert_eq!(analyzer.lanes.len(), 0);
    assert_eq!(analyzer.reservations.len(), 0);
}

#[test]
fn test_100_isolated_nodes_without_edges() {
    let nodes: Vec<String> = (0..100).map(|i| format!("Node{}", i)).collect();
    let edges = Vec::new();

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &[], &[]);

    assert_eq!(analyzer.lanes.len(), 0);
    assert_eq!(analyzer.fork_nodes.len(), 0);
    assert_eq!(analyzer.join_nodes.len(), 0);
    for id in &nodes {
        assert_eq!(analyzer.slot_for_node(id), 0.0);
        assert!(!analyzer.is_fork(id));
        assert!(!analyzer.is_join(id));
    }
}

#[test]
fn test_edges_referencing_nonexistent_nodes_cleanly_ignored() {
    let nodes = to_s(&["A", "B"]);
    let edges = to_edges(&[
        ("A", "Ghost1"),
        ("Ghost2", "B"),
        ("Ghost3", "Ghost4"),
        ("A", "B"),
    ]);

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &[], &[]);

    assert_eq!(analyzer.fork_nodes.len(), 0);
    assert_eq!(analyzer.join_nodes.len(), 0);
    assert_eq!(analyzer.slot_for_node("A"), 0.0);
    assert_eq!(analyzer.slot_for_node("B"), 0.0);
}

// ============================================================================
// TOPOLOGY 3: SELF-LOOPS AND MULTI-EDGES
// ============================================================================

#[test]
fn test_single_node_self_loop() {
    let nodes = to_s(&["A"]);
    let edges = to_edges(&[("A", "A")]);

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &[], &[]);

    assert!(analyzer.is_back_edge("A", "A"));
    assert!(!analyzer.is_fork("A"));
    assert!(!analyzer.is_join("A"));
    assert_eq!(analyzer.slot_for_node("A"), 0.0);
    assert_eq!(analyzer.lanes.len(), 0);
}

#[test]
fn test_self_loops_on_fork_and_join_nodes() {
    // Fork F has self-loop F -> F, and branches F -> L, F -> R.
    // Join J has self-loop J -> J, and converges L -> J, R -> J.
    let nodes = to_s(&["F", "L", "R", "J"]);
    let edges = to_edges(&[
        ("F", "F"), // self-loop on fork
        ("F", "L"),
        ("F", "R"),
        ("L", "J"),
        ("R", "J"),
        ("J", "J"), // self-loop on join
    ]);
    let layers = to_layers(&[&["F"], &["L", "R"], &["J"]]);

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);

    // Self loops must be classified as back-edges
    assert!(analyzer.is_back_edge("F", "F"));
    assert!(analyzer.is_back_edge("J", "J"));

    // Self-loop on F must not inflate fork count to 3
    assert!(analyzer.is_fork("F"));
    let fwd_neighbors = analyzer.forward_neighbors("F").unwrap();
    assert_eq!(fwd_neighbors.len(), 2);
    assert_eq!(fwd_neighbors, &["L", "R"]);

    // Symmetric slots
    assert_eq!(analyzer.slot_for_node("L"), -0.5);
    assert_eq!(analyzer.slot_for_node("R"), 0.5);

    // Self-loop on J must not disrupt join status or center alignment
    assert!(analyzer.is_join("J"));
    assert_eq!(analyzer.slot_for_node("J"), 0.0);
}

#[test]
fn test_multi_edges_between_same_nodes() {
    let nodes = to_s(&["A", "B"]);
    // A -> B defined 15 times
    let mut raw_edges = Vec::new();
    for _ in 0..15 {
        raw_edges.push(("A", "B"));
    }
    let edges = to_edges(&raw_edges);

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &[], &[]);

    // A is NOT a fork, B is NOT a join
    assert!(!analyzer.is_fork("A"));
    assert!(!analyzer.is_join("B"));
    assert_eq!(analyzer.lanes.len(), 0);
    assert_eq!(analyzer.slot_for_node("A"), 0.0);
    assert_eq!(analyzer.slot_for_node("B"), 0.0);
}

#[test]
fn test_multi_edges_on_fork_and_join_branches() {
    let nodes = to_s(&["F", "L", "R", "J"]);
    // Duplicate branch edges 5 times each
    let edges = to_edges(&[
        ("F", "L"),
        ("F", "L"),
        ("F", "L"),
        ("F", "R"),
        ("F", "R"),
        ("L", "J"),
        ("L", "J"),
        ("L", "J"),
        ("L", "J"),
        ("R", "J"),
        ("R", "J"),
    ]);

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &[], &[]);

    assert!(analyzer.is_fork("F"));
    assert!(analyzer.is_join("J"));
    assert_eq!(analyzer.lanes.len(), 2);
    assert_eq!(analyzer.slot_for_node("L"), -0.5);
    assert_eq!(analyzer.slot_for_node("R"), 0.5);
    assert_eq!(analyzer.slot_for_node("J"), 0.0);
}

#[test]
fn test_bidirectional_two_node_cycle_with_multi_edges() {
    let nodes = to_s(&["A", "B"]);
    let edges = to_edges(&[("A", "B"), ("A", "B"), ("B", "A"), ("B", "A"), ("B", "A")]);

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &[], &[]);

    // One direction must be a back-edge
    let back_edge_found = analyzer.is_back_edge("B", "A") || analyzer.is_back_edge("A", "B");
    assert!(back_edge_found);
    assert_eq!(analyzer.fork_nodes.len(), 0);
    assert_eq!(analyzer.join_nodes.len(), 0);
}

// ============================================================================
// TOPOLOGY 4: COMPLEX CYCLIC GRAPHS
// ============================================================================

#[test]
fn test_figure_eight_cycle_sharing_central_hub() {
    // Loop 1: A -> B -> C -> A
    // Loop 2: A -> D -> E -> A
    let nodes = to_s(&["A", "B", "C", "D", "E"]);
    let edges = to_edges(&[
        ("A", "B"),
        ("B", "C"),
        ("C", "A"), // Cycle 1 back-edge
        ("A", "D"),
        ("D", "E"),
        ("E", "A"), // Cycle 2 back-edge
    ]);

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &[], &[]);

    // At least two back-edges must be eliminated to break both loops
    assert!(analyzer.back_edges.len() >= 2);
    // Graph must not contain cyclic corruption or infinite recursion
    assert!(analyzer.slot_for_node("A").is_finite());
    assert!(analyzer.slot_for_node("B").is_finite());
    assert!(analyzer.slot_for_node("C").is_finite());
    assert!(analyzer.slot_for_node("D").is_finite());
    assert!(analyzer.slot_for_node("E").is_finite());
}

#[test]
fn test_figure_eight_with_entry_and_exit_trunks() {
    // Start -> Hub -> Loop1 (B -> C -> Hub)
    //              -> Loop2 (D -> E -> Hub)
    //              -> Exit
    let nodes = to_s(&["Start", "Hub", "B", "C", "D", "E", "Exit"]);
    let edges = to_edges(&[
        ("Start", "Hub"),
        ("Hub", "B"),
        ("B", "C"),
        ("C", "Hub"), // back-edge 1
        ("Hub", "D"),
        ("D", "E"),
        ("E", "Hub"), // back-edge 2
        ("Hub", "Exit"),
    ]);
    let layers = to_layers(&[&["Start"], &["Hub"], &["B", "D", "Exit"], &["C", "E"]]);

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);

    assert!(analyzer.is_back_edge("C", "Hub"));
    assert!(analyzer.is_back_edge("E", "Hub"));

    // Hub forward out-degree = 3 (B, D, Exit)
    assert!(analyzer.is_fork("Hub"));
    let fwd_neighbors = analyzer.forward_neighbors("Hub").unwrap();
    assert_eq!(fwd_neighbors.len(), 3);
    assert_eq!(fwd_neighbors, &["B", "D", "Exit"]);

    // Symmetric 3-way fork: B (-1.0), D (0.0), Exit (+1.0)
    assert_eq!(analyzer.slot_for_node("B"), -1.0);
    assert_eq!(analyzer.slot_for_node("D"), 0.0);
    assert_eq!(analyzer.slot_for_node("Exit"), 1.0);

    // C inherits B's slot
    assert_eq!(analyzer.slot_for_node("C"), -1.0);
    // E inherits D's slot
    assert_eq!(analyzer.slot_for_node("E"), 0.0);
}

#[test]
fn test_nested_feedback_loops_three_tiers() {
    // Outer loop: 1 -> 2 -> 3 -> 4 -> 1
    // Mid loop:   2 -> 3 -> 2
    // Inner loop: 3 -> 4 -> 3
    // Forward exit: 4 -> 5
    let nodes = to_s(&["N1", "N2", "N3", "N4", "N5"]);
    let edges = to_edges(&[
        ("N1", "N2"),
        ("N2", "N3"),
        ("N3", "N4"),
        ("N4", "N1"), // outer back-edge
        ("N3", "N2"), // mid back-edge
        ("N4", "N3"), // inner back-edge
        ("N4", "N5"), // forward exit
    ]);

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &[], &[]);

    assert!(analyzer.is_back_edge("N4", "N1"));
    assert!(analyzer.is_back_edge("N3", "N2"));
    assert!(analyzer.is_back_edge("N4", "N3"));

    // Forward path is N1 -> N2 -> N3 -> N4 -> N5
    // None should be misclassified as a forward fork or join
    assert_eq!(analyzer.fork_nodes.len(), 0);
    assert_eq!(analyzer.join_nodes.len(), 0);
    for n in &nodes {
        assert_eq!(analyzer.slot_for_node(n), 0.0);
    }
}

#[test]
fn test_pure_cycle_with_no_source_roots() {
    // Ring: R1 -> R2 -> R3 -> R4 -> R1 (all raw in-degrees == 1)
    let nodes = to_s(&["R1", "R2", "R3", "R4"]);
    let edges = to_edges(&[("R1", "R2"), ("R2", "R3"), ("R3", "R4"), ("R4", "R1")]);

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &[], &[]);

    // Exactly one back-edge must be identified
    assert_eq!(analyzer.back_edges.len(), 1);
    assert!(analyzer.is_back_edge("R4", "R1"));

    // The forward graph is R1 -> R2 -> R3 -> R4
    assert_eq!(analyzer.fork_nodes.len(), 0);
    assert_eq!(analyzer.join_nodes.len(), 0);
    for r in &nodes {
        assert_eq!(analyzer.slot_for_node(r), 0.0);
    }
}

#[test]
fn test_fully_connected_four_node_clique_k4() {
    // Every pair of nodes has directed edges in both directions
    let nodes = to_s(&["K1", "K2", "K3", "K4"]);
    let mut raw_edges = Vec::new();
    for &u in &["K1", "K2", "K3", "K4"] {
        for &v in &["K1", "K2", "K3", "K4"] {
            if u != v {
                raw_edges.push((u, v));
            }
        }
    }
    let edges = to_edges(&raw_edges);

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &[], &[]);

    // Must not panic, must terminate, all nodes must have valid finite slots
    for k in &nodes {
        assert!(analyzer.slot_for_node(k).is_finite());
    }
    // Forward graph must be an exact DAG
    let fwd = ForwardGraph::build(&nodes, &edges);
    assert!(fwd.back_edges.len() >= 3);
}

// ============================================================================
// TOPOLOGY 5: DIRECT BYPASS EDGES (Fork -> Join)
// ============================================================================

#[test]
fn test_direct_bypass_across_five_intermediate_layers() {
    // Fork F at Layer 0
    // Long branch: F -> N1 -> N2 -> N3 -> N4 -> N5 -> J (Layers 1..5)
    // Direct bypass: F -> J (bypasses Layers 1..5 directly)
    let nodes = to_s(&["F", "N1", "N2", "N3", "N4", "N5", "J"]);
    let edges = to_edges(&[
        ("F", "N1"),
        ("N1", "N2"),
        ("N2", "N3"),
        ("N3", "N4"),
        ("N4", "N5"),
        ("N5", "J"),
        ("F", "J"), // direct bypass
    ]);
    let layers = to_layers(&[&["F"], &["N1"], &["N2"], &["N3"], &["N4"], &["N5"], &["J"]]);

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);

    assert!(analyzer.is_fork("F"));
    assert!(analyzer.is_join("J"));

    // Long branch gets slot -0.5
    for n in &["N1", "N2", "N3", "N4", "N5"] {
        assert_eq!(analyzer.slot_for_node(n), -0.5);
    }
    // Join node converges to 0.0
    assert_eq!(analyzer.slot_for_node("J"), 0.0);

    // Bypass branch is slot +0.5. Must have reservations across all intermediate layers 1..5!
    for layer in 1..=5 {
        assert!(
            analyzer.is_slot_reserved(layer, 0.5),
            "Expected virtual lane reservation at layer {} with slot +0.5",
            layer
        );
        // Long branch occupied layer, must NOT have reservation at -0.5
        assert!(
            !analyzer.is_slot_reserved(layer, -0.5),
            "Should not reserve slot -0.5 at layer {} where node is present",
            layer
        );
    }

    // Join layer 6 must have NO reservations
    assert!(analyzer.reservations_at_layer(6).is_empty());
}

#[test]
fn test_three_way_fork_with_two_bypass_edges_to_two_different_joins() {
    // Fork F -> Join1 (direct bypass 1)
    //      F -> Mid -> Join2
    //      F -> Join2 (direct bypass 2)
    // Other inputs to joins:
    // Extra1 -> Join1
    // Extra2 -> Join2
    let nodes = to_s(&["F", "Mid", "Join1", "Join2", "Extra1", "Extra2"]);
    let edges = to_edges(&[
        ("F", "Join1"),
        ("Extra1", "Join1"),
        ("F", "Mid"),
        ("Mid", "Join2"),
        ("F", "Join2"),
        ("Extra2", "Join2"),
    ]);
    let layers = to_layers(&[&["Extra1", "Extra2", "F"], &["Mid", "Join1"], &["Join2"]]);

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);

    assert!(analyzer.is_fork("F"));
    assert!(analyzer.is_join("Join1"));
    assert!(analyzer.is_join("Join2"));

    // 3-way fork: slot offsets are -1.0, 0.0, +1.0
    assert_eq!(analyzer.slot_for_node("Mid"), 0.0);
    assert_eq!(analyzer.slot_for_node("Join1"), 0.0);
    assert_eq!(analyzer.slot_for_node("Join2"), 0.0);

    // Bypass edge F -> Join2 skips layer 1, reserving slot +1.0 at layer 1
    assert!(analyzer.is_slot_reserved(1, 1.0));
}

#[test]
fn test_bypass_edge_with_feedback_retry_loop() {
    // Fork F -> A -> J
    // Fork F -> J (bypass)
    // Join J -> F (retry back-edge)
    // Join J -> Exit (forward exit)
    let nodes = to_s(&["F", "A", "J", "Exit"]);
    let edges = to_edges(&[
        ("F", "A"),
        ("A", "J"),
        ("F", "J"),    // bypass
        ("J", "F"),    // retry loop
        ("J", "Exit"), // forward exit
    ]);
    let layers = to_layers(&[&["F"], &["A"], &["J"], &["Exit"]]);

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);

    assert!(analyzer.is_back_edge("J", "F"));
    assert!(analyzer.is_fork("F"));
    assert!(analyzer.is_join("J"));

    assert_eq!(analyzer.slot_for_node("A"), -0.5);
    assert_eq!(analyzer.slot_for_node("J"), 0.0);
    assert_eq!(analyzer.slot_for_node("Exit"), 0.0);

    // Bypass edge reserves slot +0.5 at layer 1
    assert!(analyzer.is_slot_reserved(1, 0.5));
}

#[test]
fn test_nested_fork_with_internal_bypass_edge() {
    // Outer Fork F1 -> LeftBranch -> EndJ
    //           F1 -> F2 (Nested Fork)
    // Nested Fork F2 -> N1 -> EndJ
    //             F2 -> EndJ (Direct Bypass within nested branch)
    let nodes = to_s(&["F1", "LeftBranch", "F2", "N1", "EndJ"]);
    let edges = to_edges(&[
        ("F1", "LeftBranch"),
        ("LeftBranch", "EndJ"),
        ("F1", "F2"),
        ("F2", "N1"),
        ("N1", "EndJ"),
        ("F2", "EndJ"), // Bypass inside nested fork F2
    ]);
    let layers = to_layers(&[&["F1"], &["LeftBranch", "F2"], &["N1"], &["EndJ"]]);

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);

    assert!(analyzer.is_fork("F1"));
    assert!(analyzer.is_fork("F2"));
    assert!(analyzer.is_join("EndJ"));

    // F1 branches: LeftBranch (-0.5), F2 (+0.5)
    assert_eq!(analyzer.slot_for_node("LeftBranch"), -0.5);
    assert_eq!(analyzer.slot_for_node("F2"), 0.5);

    // F2 nested outward spill:
    // child 0 (N1): inherits +0.5
    // child 1 (EndJ): bypass to join at slot +1.5
    assert_eq!(analyzer.slot_for_node("N1"), 0.5);
    assert_eq!(analyzer.slot_for_node("EndJ"), 0.0);

    // Bypass branch (child 1 of F2) at slot +1.5 skips layer 2
    assert!(analyzer.is_slot_reserved(2, 1.5));
}

// ============================================================================
// TOPOLOGY 6: PROPERTY-BASED INVARIANT STRESS TEST
// ============================================================================

#[test]
fn test_property_invariants_randomized_topologies() {
    // Pseudo-random linear congruential generator for reproducibility
    let mut rng_state: u64 = 0xDEADBEEFCAFE;
    let mut next_rnd = || {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (rng_state >> 33) as usize
    };

    for test_idx in 0..50 {
        let node_count = 10 + (next_rnd() % 30);
        let nodes: Vec<String> = (0..node_count)
            .map(|i| format!("V_{}_{}", test_idx, i))
            .collect();

        let edge_count = next_rnd() % (node_count * 2);
        let mut edges = Vec::new();
        for _ in 0..edge_count {
            let u = &nodes[next_rnd() % node_count];
            let v = &nodes[next_rnd() % node_count];
            edges.push((u.clone(), v.clone()));
        }

        let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &[], &[]);

        // Invariant 1: slot_for_node must be finite for every node
        for u in &nodes {
            let slot = analyzer.slot_for_node(u);
            assert!(slot.is_finite(), "Node {} has non-finite slot: {}", u, slot);
        }

        // Invariant 2: Every join node must converge strictly to 0.0
        for join in &analyzer.join_nodes {
            let slot = analyzer.slot_for_node(join);
            assert_eq!(slot, 0.0, "Join node {} did not converge to 0.0", join);
        }

        // Invariant 3: For all lanes, depth_span.0 <= depth_span.1
        for lane in &analyzer.lanes {
            assert!(
                lane.depth_span.0 <= lane.depth_span.1,
                "Lane {} has inverted depth_span: {:?}",
                lane.branch_id,
                lane.depth_span
            );
        }

        // Invariant 4: Reservations must be uniquely checkable via is_slot_reserved
        for res in &analyzer.reservations {
            assert!(
                analyzer.is_slot_reserved(res.layer, res.slot_offset),
                "Reservation at layer {} slot {} not recognized by is_slot_reserved",
                res.layer,
                res.slot_offset
            );
        }
    }
}
